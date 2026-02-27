use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use adapter_rmvm::RmvmAdapter;
use anyhow::{Context, Result, anyhow, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use brain_store::{BrainStore, CreateBrainRequest};
use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use keyring::Entry;
use rand::rngs::OsRng;
use reqwest::Client;
use rmvm_grpc::GetManifestRequest;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use uuid::Uuid;

const CONFIG_VERSION: u32 = 1;
const CONFIG_FILE: &str = "config.json";
const RUNTIME_FILE: &str = "runtime.json";
const LOG_DIR: &str = "logs";
const FALLBACK_SECRETS_FILE: &str = "secrets.enc.json";
const FALLBACK_KEY_FILE: &str = "secrets.key";
const KEYRING_SERVICE: &str = "cortex-brain";

const DEFAULT_PROXY_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_RMVM_HOST: &str = "127.0.0.1";
const DEFAULT_RMVM_PORT: u16 = 50051;
const DEFAULT_BRAIN_SECRET_ENV: &str = "CORTEX_BRAIN_SECRET";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductConfig {
    pub version: u32,
    pub tenant: String,
    pub active_brain: Option<String>,
    pub active_provider: String,
    pub proxy_addr: String,
    pub proxy_api_key: Option<String>,
    pub brain_secret_env: String,
    pub brain_secret_ref: String,
    pub rmvm: RmvmSettings,
    pub providers: BTreeMap<String, ProviderProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RmvmSettings {
    pub mode: String,
    pub endpoint: Option<String>,
    pub host: String,
    pub port: u16,
    pub sidecar_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub name: String,
    pub planner_mode: String,
    pub planner_base_url: String,
    pub planner_model: String,
    pub planner_api_key_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeState {
    pub proxy_pid: Option<u32>,
    pub rmvm_pid: Option<u32>,
    pub rmvm_mode: String,
    pub rmvm_endpoint: String,
    pub proxy_addr: String,
    pub last_started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupResult {
    pub brain_id: String,
    pub provider: String,
    pub model: String,
    pub proxy_addr: String,
    pub rmvm_mode: String,
    pub rmvm_endpoint: String,
}

#[derive(Debug, Clone)]
pub struct SetupRequest {
    pub non_interactive: bool,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub planner_base_url: Option<String>,
    pub planner_api_key: Option<String>,
    pub planner_api_key_env: Option<String>,
    pub brain: Option<String>,
    pub tenant: String,
    pub api_key: Option<String>,
    pub rmvm_endpoint: Option<String>,
    pub proxy_addr: Option<String>,
    pub rmvm_port: Option<u16>,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct UpRequest {
    pub detached: bool,
    pub proxy_addr: Option<String>,
    pub rmvm_endpoint: Option<String>,
    pub rmvm_port: Option<u16>,
    pub brain: Option<String>,
    pub provider: Option<String>,
    pub reuse_external_rmvm: bool,
}

#[derive(Debug, Clone)]
pub struct StopRequest {
    pub all: bool,
    pub proxy_only: bool,
    pub rmvm_only: bool,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct StatusRequest {
    pub json: bool,
    pub verbose: bool,
    pub copy: bool,
}

#[derive(Debug, Clone)]
pub struct LogsRequest {
    pub service: String,
    pub tail: usize,
    pub follow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    Auto,
    Never,
}

#[derive(Debug, Clone, Serialize)]
struct StatusView {
    active_brain: Option<String>,
    active_provider: String,
    planner_model: Option<String>,
    proxy_addr: String,
    dashboard_url: String,
    proxy_healthy: bool,
    rmvm_endpoint: String,
    rmvm_mode: String,
    rmvm_healthy: bool,
    runtime_proxy_pid: Option<u32>,
    runtime_rmvm_pid: Option<u32>,
    config_path: String,
    state_path: String,
}

#[derive(Debug, Clone)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
}

impl Paths {
    fn config_file(&self) -> PathBuf {
        self.config_dir.join(CONFIG_FILE)
    }

    fn runtime_file(&self) -> PathBuf {
        self.state_dir.join(RUNTIME_FILE)
    }

    fn logs_dir(&self) -> PathBuf {
        self.state_dir.join(LOG_DIR)
    }

    fn proxy_log_file(&self) -> PathBuf {
        self.logs_dir().join("proxy.log")
    }

    fn rmvm_log_file(&self) -> PathBuf {
        self.logs_dir().join("rmvm.log")
    }

    fn fallback_secrets_file(&self) -> PathBuf {
        self.state_dir.join(FALLBACK_SECRETS_FILE)
    }

    fn fallback_key_file(&self) -> PathBuf {
        self.state_dir.join(FALLBACK_KEY_FILE)
    }
}

pub fn default_paths() -> Result<Paths> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("failed to resolve config dir"))?
        .join("cortex");
    let state_base = dirs::state_dir().unwrap_or_else(|| config_dir.clone());
    let state_dir = state_base.join("cortex");
    Ok(Paths {
        config_dir,
        state_dir,
    })
}

fn ensure_dirs(paths: &Paths) -> Result<()> {
    fs::create_dir_all(&paths.config_dir)?;
    fs::create_dir_all(&paths.state_dir)?;
    fs::create_dir_all(paths.logs_dir())?;
    Ok(())
}

fn sidecar_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "rmvm-grpc-server.exe"
    } else {
        "rmvm-grpc-server"
    }
}

fn default_sidecar_path() -> Result<PathBuf> {
    let mut path = env::current_exe().context("failed to resolve current executable path")?;
    path.set_file_name(sidecar_binary_name());
    Ok(path)
}

fn normalize_grpc_endpoint(input: &str) -> String {
    if input.starts_with("grpc://") {
        input.to_string()
    } else if let Some(rest) = input.strip_prefix("http://") {
        format!("grpc://{rest}")
    } else if let Some(rest) = input.strip_prefix("https://") {
        format!("grpc://{rest}")
    } else {
        format!("grpc://{input}")
    }
}

fn default_providers() -> BTreeMap<String, ProviderProfile> {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "openai".to_string(),
        ProviderProfile {
            name: "openai".to_string(),
            planner_mode: "openai".to_string(),
            planner_base_url: "https://api.openai.com/v1".to_string(),
            planner_model: "gpt-4o-mini".to_string(),
            planner_api_key_ref: Some("provider.openai.api_key".to_string()),
        },
    );
    profiles.insert(
        "claude".to_string(),
        ProviderProfile {
            name: "claude".to_string(),
            planner_mode: "openai".to_string(),
            planner_base_url: "https://api.anthropic.com/v1/".to_string(),
            planner_model: "claude-opus-4-6".to_string(),
            planner_api_key_ref: Some("provider.claude.api_key".to_string()),
        },
    );
    profiles.insert(
        "gemini".to_string(),
        ProviderProfile {
            name: "gemini".to_string(),
            planner_mode: "openai".to_string(),
            planner_base_url: "https://generativelanguage.googleapis.com/v1beta/openai/".to_string(),
            planner_model: "gemini-3-flash-preview".to_string(),
            planner_api_key_ref: Some("provider.gemini.api_key".to_string()),
        },
    );
    profiles.insert(
        "ollama".to_string(),
        ProviderProfile {
            name: "ollama".to_string(),
            planner_mode: "openai".to_string(),
            planner_base_url: "http://127.0.0.1:11434/v1".to_string(),
            planner_model: "llama3.1".to_string(),
            planner_api_key_ref: None,
        },
    );
    profiles.insert(
        "byo".to_string(),
        ProviderProfile {
            name: "byo".to_string(),
            planner_mode: "byo".to_string(),
            planner_base_url: "http://unused".to_string(),
            planner_model: "byo-plan".to_string(),
            planner_api_key_ref: None,
        },
    );
    profiles
}

fn default_config() -> ProductConfig {
    ProductConfig {
        version: CONFIG_VERSION,
        tenant: "local".to_string(),
        active_brain: None,
        active_provider: "openai".to_string(),
        proxy_addr: DEFAULT_PROXY_ADDR.to_string(),
        proxy_api_key: None,
        brain_secret_env: DEFAULT_BRAIN_SECRET_ENV.to_string(),
        brain_secret_ref: "brain.default.secret".to_string(),
        rmvm: RmvmSettings {
            mode: "managed".to_string(),
            endpoint: None,
            host: DEFAULT_RMVM_HOST.to_string(),
            port: DEFAULT_RMVM_PORT,
            sidecar_path: None,
        },
        providers: default_providers(),
    }
}

fn load_config(paths: &Paths) -> Result<ProductConfig> {
    ensure_dirs(paths)?;
    let path = paths.config_file();
    if !path.exists() {
        let cfg = default_config();
        save_config(paths, &cfg)?;
        return Ok(cfg);
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut cfg: ProductConfig =
        serde_json::from_str(&raw).with_context(|| format!("invalid {}", path.display()))?;
    if cfg.providers.is_empty() {
        cfg.providers = default_providers();
    }
    Ok(cfg)
}

fn save_config(paths: &Paths, cfg: &ProductConfig) -> Result<()> {
    ensure_dirs(paths)?;
    let raw = serde_json::to_string_pretty(cfg)?;
    fs::write(paths.config_file(), raw)?;
    Ok(())
}

fn load_runtime(paths: &Paths) -> Result<Option<RuntimeState>> {
    let path = paths.runtime_file();
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    let state: RuntimeState = serde_json::from_str(&raw)?;
    Ok(Some(state))
}

fn save_runtime(paths: &Paths, state: &RuntimeState) -> Result<()> {
    ensure_dirs(paths)?;
    fs::write(paths.runtime_file(), serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn clear_runtime(paths: &Paths) -> Result<()> {
    let path = paths.runtime_file();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn resolve_provider<'a>(cfg: &'a ProductConfig, name: Option<&str>) -> Result<&'a ProviderProfile> {
    let provider_name = name.unwrap_or(&cfg.active_provider);
    cfg.providers
        .get(provider_name)
        .ok_or_else(|| anyhow!("unknown provider '{}'", provider_name))
}

fn secret_entry(key: &str) -> Result<Entry> {
    Entry::new(KEYRING_SERVICE, key).context("failed to initialize keyring entry")
}

fn ensure_fallback_key(paths: &Paths) -> Result<[u8; 32]> {
    ensure_dirs(paths)?;
    let path = paths.fallback_key_file();
    if path.exists() {
        let raw = fs::read(&path)?;
        if raw.len() != 32 {
            bail!("invalid fallback key file length at {}", path.display());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&raw);
        return Ok(key);
    }
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    fs::write(&path, key)?;
    Ok(key)
}

fn load_fallback_secrets(paths: &Paths) -> Result<BTreeMap<String, String>> {
    let path = paths.fallback_secrets_file();
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let raw = fs::read_to_string(path)?;
    let map: BTreeMap<String, String> = serde_json::from_str(&raw)?;
    Ok(map)
}

fn save_fallback_secrets(paths: &Paths, map: &BTreeMap<String, String>) -> Result<()> {
    ensure_dirs(paths)?;
    fs::write(
        paths.fallback_secrets_file(),
        serde_json::to_string_pretty(map)?,
    )?;
    Ok(())
}

fn encrypt_secret(paths: &Paths, plaintext: &str) -> Result<String> {
    let key = ensure_fallback_key(paths)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| anyhow!("failed to encrypt secret"))?;
    Ok(format!(
        "{}:{}",
        B64.encode(nonce_bytes),
        B64.encode(ciphertext)
    ))
}

fn decrypt_secret(paths: &Paths, sealed: &str) -> Result<String> {
    let mut parts = sealed.splitn(2, ':');
    let nonce = parts
        .next()
        .ok_or_else(|| anyhow!("invalid encrypted secret format"))?;
    let ciphertext = parts
        .next()
        .ok_or_else(|| anyhow!("invalid encrypted secret format"))?;
    let nonce_bytes = B64.decode(nonce)?;
    let ciphertext = B64.decode(ciphertext)?;
    let key = ensure_fallback_key(paths)?;
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .map_err(|_| anyhow!("failed to decrypt fallback secret"))?;
    String::from_utf8(plaintext).context("fallback secret is not utf-8")
}

fn put_secret(paths: &Paths, key: &str, value: &str) -> Result<()> {
    let mut map = load_fallback_secrets(paths)?;
    map.insert(key.to_string(), encrypt_secret(paths, value)?);
    save_fallback_secrets(paths, &map)?;
    if let Ok(entry) = secret_entry(key) {
        let _ = entry.set_password(value);
    }
    Ok(())
}

fn get_secret(paths: &Paths, key: &str) -> Result<Option<String>> {
    let map = load_fallback_secrets(paths)?;
    if let Some(sealed) = map.get(key) {
        return Ok(Some(decrypt_secret(paths, sealed)?));
    }
    if let Ok(entry) = secret_entry(key) {
        if let Ok(value) = entry.get_password() {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn ensure_brain_secret_env(paths: &Paths, cfg: &ProductConfig) -> Result<()> {
    if let Some(stored) = get_secret(paths, &cfg.brain_secret_ref)? {
        unsafe {
            env::set_var(&cfg.brain_secret_env, stored);
        }
        return Ok(());
    }
    let value = if let Ok(existing) = env::var(&cfg.brain_secret_env) {
        put_secret(paths, &cfg.brain_secret_ref, &existing)?;
        existing
    } else {
        let generated = format!("brain-{}", Uuid::new_v4().simple());
        put_secret(paths, &cfg.brain_secret_ref, &generated)?;
        generated
    };
    unsafe {
        env::set_var(&cfg.brain_secret_env, value);
    }
    Ok(())
}

fn random_api_key() -> String {
    format!("ctx_{}", Uuid::new_v4().simple())
}

fn prompt_with_default(label: &str, default: &str) -> Result<String> {
    print!("{label} [{default}]: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_optional(label: &str) -> Result<Option<String>> {
    print!("{label}: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn probe_tcp(addr: &str) -> bool {
    if let Ok(parsed) = addr.parse::<SocketAddr>() {
        TcpStream::connect_timeout(&parsed, Duration::from_millis(500)).is_ok()
    } else {
        false
    }
}

async fn probe_rmvm(endpoint: &str) -> bool {
    let adapter = RmvmAdapter::new(endpoint.to_string());
    adapter
        .get_manifest(GetManifestRequest {
            request_id: format!("probe-{}", Uuid::new_v4().simple()),
        })
        .await
        .is_ok()
}

async fn probe_proxy(proxy_addr: &str) -> bool {
    let healthz = format!("http://{}/healthz", proxy_addr.trim_end_matches('/'));
    let client = match Client::builder().timeout(Duration::from_secs(2)).build() {
        Ok(c) => c,
        Err(_) => return false,
    };
    let Ok(resp) = client.get(healthz).send().await else {
        return false;
    };
    resp.status().is_success()
}

fn rmvm_endpoint(cfg: &ProductConfig) -> String {
    if cfg.rmvm.mode == "external" {
        cfg.rmvm
            .endpoint
            .clone()
            .unwrap_or_else(|| format!("grpc://{}:{}", cfg.rmvm.host, cfg.rmvm.port))
    } else {
        format!("grpc://{}:{}", cfg.rmvm.host, cfg.rmvm.port)
    }
}

fn provider_display_name(name: &str) -> String {
    match name.to_ascii_lowercase().as_str() {
        "openai" => "OpenAI".to_string(),
        "claude" => "Claude".to_string(),
        "gemini" => "Gemini".to_string(),
        "ollama" => "Ollama".to_string(),
        "byo" => "BYO".to_string(),
        other => other.to_string(),
    }
}

fn active_brain_label(cfg: &ProductConfig) -> String {
    let Some(active) = cfg.active_brain.as_ref() else {
        return "<none>".to_string();
    };
    if let Ok(store) = BrainStore::new(None) {
        if let Ok(summary) = store.resolve_brain(active) {
            return summary.name;
        }
    }
    active.clone()
}

fn dashboard_url(cfg: &ProductConfig) -> String {
    format!("http://{}/dashboard", cfg.proxy_addr)
}

fn print_connect_info_block(cfg: &ProductConfig, provider: Option<&ProviderProfile>) {
    let provider_name = provider_display_name(&cfg.active_provider);
    let model = provider
        .map(|p| p.planner_model.as_str())
        .unwrap_or("<unknown-model>");
    let api_key = cfg.proxy_api_key.as_deref().unwrap_or("<not-set>");
    println!("Copy/paste client settings:");
    println!("Base URL: http://{}/v1", cfg.proxy_addr);
    println!("API Key: {}", api_key);
    println!("Provider: {} ({})", provider_name, model);
    println!("Brain: {}", active_brain_label(cfg));
}

fn sidecar_path(cfg: &ProductConfig) -> Result<PathBuf> {
    if let Some(path) = cfg.rmvm.sidecar_path.as_ref() {
        Ok(PathBuf::from(path))
    } else {
        default_sidecar_path()
    }
}

fn open_log(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open log {}", path.display()))
}

fn spawn_rmvm_sidecar(cfg: &ProductConfig, paths: &Paths) -> Result<u32> {
    let bin = sidecar_path(cfg)?;
    let addr = format!("{}:{}", cfg.rmvm.host, cfg.rmvm.port);
    let stdout = open_log(&paths.rmvm_log_file())?;
    let stderr = open_log(&paths.rmvm_log_file())?;
    let mut cmd = if bin.exists() {
        let mut cmd = Command::new(bin);
        cmd.env("RMVM_SERVER_ADDR", addr);
        cmd
    } else {
        let mut cmd = Command::new(
            env::current_exe().context("failed to resolve cortex executable path for RMVM fallback")?,
        );
        cmd.arg("rmvm").arg("serve").arg("--addr").arg(addr);
        cmd
    };
    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .context("failed to spawn rmvm runtime")?;
    Ok(child.id())
}

fn spawn_proxy(
    cfg: &ProductConfig,
    paths: &Paths,
    endpoint: &str,
    provider: &ProviderProfile,
    planner_api_key: Option<String>,
) -> Result<u32> {
    let exe = env::current_exe().context("failed to resolve cortex executable path")?;
    let stdout = open_log(&paths.proxy_log_file())?;
    let stderr = open_log(&paths.proxy_log_file())?;
    let mut cmd = Command::new(exe);
    cmd.arg("proxy")
        .arg("serve")
        .arg("--addr")
        .arg(&cfg.proxy_addr)
        .arg("--endpoint")
        .arg(endpoint)
        .arg("--planner-mode")
        .arg(&provider.planner_mode)
        .arg("--planner-base-url")
        .arg(&provider.planner_base_url)
        .arg("--planner-model")
        .arg(&provider.planner_model)
        .arg("--provider-name")
        .arg(&cfg.active_provider)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    if let Some(brain) = cfg.active_brain.as_ref() {
        cmd.arg("--brain").arg(brain);
    }
    if let Some(api_key) = cfg.proxy_api_key.as_ref() {
        cmd.arg("--proxy-api-key").arg(api_key);
    }
    if let Some(api_key) = planner_api_key {
        cmd.env("CORTEX_PLANNER_API_KEY", api_key);
    }
    let child = cmd.spawn().context("failed to spawn cortex proxy")?;
    Ok(child.id())
}

fn kill_pid(pid: u32, force: bool) {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("taskkill");
        cmd.arg("/PID").arg(pid.to_string());
        if force {
            cmd.arg("/F");
        }
        let _ = cmd.output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let signal = if force { "-9" } else { "-15" };
        let _ = Command::new("kill")
            .arg(signal)
            .arg(pid.to_string())
            .output();
    }
}

async fn wait_for_rmvm(endpoint: &str, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if probe_rmvm(endpoint).await {
            return true;
        }
        sleep(Duration::from_millis(250)).await;
    }
    false
}

async fn wait_for_proxy(addr: &str, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if probe_proxy(addr).await {
            return true;
        }
        sleep(Duration::from_millis(250)).await;
    }
    false
}

fn is_interactive(non_interactive: bool) -> bool {
    !non_interactive && atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout)
}

fn planner_api_key(paths: &Paths, provider: &ProviderProfile) -> Result<Option<String>> {
    let Some(secret_ref) = provider.planner_api_key_ref.as_ref() else {
        return Ok(None);
    };
    get_secret(paths, secret_ref)
}

fn provider_requires_planner_key(provider: &ProviderProfile) -> bool {
    if provider.planner_mode != "openai" {
        return false;
    }
    let base_url = provider.planner_base_url.to_ascii_lowercase();
    !(base_url.contains("127.0.0.1")
        || base_url.contains("localhost")
        || base_url.contains("ollama"))
}

pub fn run_setup(req: SetupRequest) -> Result<SetupResult> {
    let paths = default_paths()?;
    let mut cfg = load_config(&paths)?;
    ensure_brain_secret_env(&paths, &cfg)?;

    let interactive = is_interactive(req.non_interactive);
    let default_provider = req
        .provider
        .clone()
        .unwrap_or_else(|| cfg.active_provider.clone());
    let provider_name = if interactive {
        prompt_with_default("Provider (openai/claude/gemini/ollama/byo)", &default_provider)?
    } else {
        default_provider
    };
    if !cfg.providers.contains_key(&provider_name) {
        bail!("unknown provider '{}'", provider_name);
    }
    let model = if let Some(model) = req.model.clone() {
        model
    } else if interactive {
        let default_model = cfg
            .providers
            .get(&provider_name)
            .map(|p| p.planner_model.clone())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());
        prompt_with_default("Planner model", &default_model)?
    } else {
        cfg.providers
            .get(&provider_name)
            .map(|p| p.planner_model.clone())
            .unwrap_or_else(|| "gpt-4o-mini".to_string())
    };
    let brain_name = if let Some(brain) = req.brain.clone() {
        brain
    } else if interactive {
        prompt_with_default("Brain name", "personal")?
    } else {
        "personal".to_string()
    };
    let api_key = if let Some(k) = req.api_key.clone() {
        k
    } else if interactive {
        prompt_with_default(
            "Proxy API key",
            &cfg.proxy_api_key.clone().unwrap_or_else(random_api_key),
        )?
    } else {
        cfg.proxy_api_key.clone().unwrap_or_else(random_api_key)
    };

    if let Some(addr) = req.proxy_addr.as_ref() {
        cfg.proxy_addr = addr.clone();
    }
    if let Some(endpoint) = req.rmvm_endpoint.as_ref() {
        cfg.rmvm.mode = "external".to_string();
        cfg.rmvm.endpoint = Some(normalize_grpc_endpoint(endpoint));
    } else if let Some(port) = req.rmvm_port {
        cfg.rmvm.mode = "managed".to_string();
        cfg.rmvm.port = port;
        cfg.rmvm.endpoint = None;
    }
    cfg.tenant = req.tenant.clone();

    if let Some(profile) = cfg.providers.get_mut(&provider_name) {
        profile.planner_model = model.clone();
        if let Some(base_url) = req.planner_base_url.as_ref() {
            profile.planner_base_url = base_url.clone();
        }
        if provider_requires_planner_key(profile) && profile.planner_api_key_ref.is_none() {
            profile.planner_api_key_ref = Some(format!("provider.{}.api_key", provider_name));
        } else if !provider_requires_planner_key(profile) {
            profile.planner_api_key_ref = None;
        }
    }
    cfg.active_provider = provider_name.clone();

    let planner_key = req
        .planner_api_key
        .clone()
        .or_else(|| {
            req.planner_api_key_env
                .as_ref()
                .and_then(|env_name| env::var(env_name).ok())
        })
        .or_else(|| env::var("CORTEX_PLANNER_API_KEY").ok());
    if let Some(value) = planner_key {
        if let Some(secret_ref) = cfg
            .providers
            .get(&provider_name)
            .and_then(|p| p.planner_api_key_ref.clone())
        {
            put_secret(&paths, &secret_ref, &value)?;
        }
    } else if interactive
        && cfg
            .providers
            .get(&provider_name)
            .map(provider_requires_planner_key)
            .unwrap_or(false)
    {
        if let Some(value) = prompt_optional("Planner API key (optional)")? {
            if let Some(secret_ref) = cfg
                .providers
                .get(&provider_name)
                .and_then(|p| p.planner_api_key_ref.clone())
            {
                put_secret(&paths, &secret_ref, &value)?;
            }
        }
    }
    if cfg
        .providers
        .get(&provider_name)
        .map(provider_requires_planner_key)
        .unwrap_or(false)
        && cfg
            .providers
            .get(&provider_name)
            .and_then(|p| p.planner_api_key_ref.as_ref())
            .and_then(|k| get_secret(&paths, k).ok().flatten())
            .is_none()
    {
        println!(
            "Warning: provider '{}' has no planner API key configured. Set CORTEX_PLANNER_API_KEY or rerun setup with --planner-api-key.",
            provider_name
        );
    }

    let store = BrainStore::new(None)?;
    let mut brain_summary = match store.resolve_brain(&brain_name) {
        Ok(summary) => summary,
        Err(_) => store.create_brain(CreateBrainRequest {
            name: brain_name.clone(),
            tenant_id: cfg.tenant.clone(),
            passphrase_env: Some(cfg.brain_secret_env.clone()),
        })?,
    };
    if store.audit_trace(&brain_summary.brain_id).is_err() {
        let replacement_name = if req.force {
            brain_name.clone()
        } else {
            format!("{}-{}", brain_name, Uuid::new_v4().simple())
        };
        brain_summary = store.create_brain(CreateBrainRequest {
            name: replacement_name.clone(),
            tenant_id: cfg.tenant.clone(),
            passphrase_env: Some(cfg.brain_secret_env.clone()),
        })?;
        println!(
            "Existing brain could not be unlocked with current secret; created fresh brain {} ({})",
            replacement_name, brain_summary.brain_id
        );
    }
    if req.force {
        // accepted for forward compatibility; setup always refreshes active mapping
    }
    let _ = store.set_active_brain(&brain_summary.brain_id)?;
    store.map_api_key(&api_key, &cfg.tenant, &brain_summary.brain_id, "user:local")?;

    cfg.active_brain = Some(brain_summary.brain_id.clone());
    cfg.proxy_api_key = Some(api_key);
    save_config(&paths, &cfg)?;

    let rmvm_ep = rmvm_endpoint(&cfg);
    Ok(SetupResult {
        brain_id: brain_summary.brain_id,
        provider: provider_name,
        model,
        proxy_addr: cfg.proxy_addr,
        rmvm_mode: cfg.rmvm.mode.clone(),
        rmvm_endpoint: rmvm_ep,
    })
}

pub async fn run_up(req: UpRequest) -> Result<()> {
    if !req.detached {
        bail!(
            "--detached=false is not supported yet; use detached mode (default) and run `cortex logs --follow`"
        );
    }

    let paths = default_paths()?;
    let mut cfg = load_config(&paths)?;
    ensure_brain_secret_env(&paths, &cfg)?;

    if let Some(brain) = req.brain.as_ref() {
        cfg.active_brain = Some(brain.clone());
    }
    if let Some(provider) = req.provider.as_ref() {
        if !cfg.providers.contains_key(provider) {
            bail!("unknown provider '{}'", provider);
        }
        cfg.active_provider = provider.clone();
    }
    if let Some(addr) = req.proxy_addr.as_ref() {
        cfg.proxy_addr = addr.clone();
    }
    if let Some(port) = req.rmvm_port {
        cfg.rmvm.port = port;
    }
    if let Some(endpoint) = req.rmvm_endpoint.as_ref() {
        cfg.rmvm.mode = "external".to_string();
        cfg.rmvm.endpoint = Some(normalize_grpc_endpoint(endpoint));
    }

    if cfg.active_brain.is_none() {
        let store = BrainStore::new(None)?;
        if let Ok(active) = store.active_brain_id() {
            cfg.active_brain = active;
        }
    }
    if cfg.active_brain.is_none() {
        bail!("no active brain configured; run `cortex setup` first");
    }

    let provider = resolve_provider(&cfg, None)?.clone();
    let planner_key = planner_api_key(&paths, &provider)?;
    save_config(&paths, &cfg)?;

    let mut runtime = load_runtime(&paths)?.unwrap_or_default();

    let endpoint = if cfg.rmvm.mode == "external" {
        rmvm_endpoint(&cfg)
    } else {
        let bind = format!("{}:{}", cfg.rmvm.host, cfg.rmvm.port);
        let ep = format!("grpc://{}", bind);
        if probe_tcp(&bind) {
            if probe_rmvm(&ep).await && req.reuse_external_rmvm {
                runtime.rmvm_pid = None;
                runtime.rmvm_mode = "external".to_string();
            } else if !probe_rmvm(&ep).await {
                bail!(
                    "port {} is in use by a non-RMVM process; use --rmvm-port to avoid conflict",
                    bind
                );
            }
        } else {
            let pid = spawn_rmvm_sidecar(&cfg, &paths)?;
            runtime.rmvm_pid = Some(pid);
            runtime.rmvm_mode = "managed".to_string();
            if !wait_for_rmvm(&ep, Duration::from_secs(10)).await {
                bail!(
                    "managed RMVM failed health check; see {}",
                    paths.rmvm_log_file().display()
                );
            }
        }
        ep
    };

    if let Some(pid) = runtime.proxy_pid {
        kill_pid(pid, true);
    }
    let proxy_pid = spawn_proxy(&cfg, &paths, &endpoint, &provider, planner_key)?;
    if !wait_for_proxy(&cfg.proxy_addr, Duration::from_secs(10)).await {
        bail!(
            "proxy failed health check; see {}",
            paths.proxy_log_file().display()
        );
    }
    runtime.proxy_pid = Some(proxy_pid);
    runtime.proxy_addr = cfg.proxy_addr.clone();
    runtime.rmvm_endpoint = endpoint.clone();
    if runtime.rmvm_mode.is_empty() {
        runtime.rmvm_mode = if cfg.rmvm.mode == "external" {
            "external".to_string()
        } else {
            "managed".to_string()
        };
    }
    runtime.last_started_at = Some(chrono::Utc::now().to_rfc3339());
    save_runtime(&paths, &runtime)?;

    println!("RMVM: {} ({})", runtime.rmvm_mode, runtime.rmvm_endpoint);
    println!("Proxy: running on http://{}", cfg.proxy_addr);
    println!("Dashboard: {}", dashboard_url(&cfg));
    print_connect_info_block(&cfg, Some(&provider));
    println!("Tip: paste Base URL and API Key in your AI app settings (not in chat text).");
    Ok(())
}

pub fn run_stop(req: StopRequest) -> Result<()> {
    let paths = default_paths()?;
    let state = load_runtime(&paths)?;
    let Some(state) = state else {
        println!("Nothing running.");
        return Ok(());
    };

    let stop_proxy = req.all || (!req.rmvm_only && !req.proxy_only) || req.proxy_only;
    let stop_rmvm = req.all || (!req.rmvm_only && !req.proxy_only) || req.rmvm_only;

    if stop_proxy {
        if let Some(pid) = state.proxy_pid {
            kill_pid(pid, req.force);
            println!("Stopped proxy pid={}", pid);
        } else {
            println!("Proxy not running.");
        }
    }
    if stop_rmvm {
        if let Some(pid) = state.rmvm_pid {
            kill_pid(pid, req.force);
            println!("Stopped rmvm pid={}", pid);
        } else {
            println!("RMVM is external or not running.");
        }
    }

    if stop_proxy && stop_rmvm {
        clear_runtime(&paths)?;
    } else {
        let mut next = state;
        if stop_proxy {
            next.proxy_pid = None;
        }
        if stop_rmvm {
            next.rmvm_pid = None;
        }
        save_runtime(&paths, &next)?;
    }
    Ok(())
}

pub async fn run_status(req: StatusRequest) -> Result<()> {
    let paths = default_paths()?;
    let cfg = load_config(&paths)?;
    let runtime = load_runtime(&paths)?.unwrap_or_default();
    let endpoint = if runtime.rmvm_endpoint.is_empty() {
        rmvm_endpoint(&cfg)
    } else {
        runtime.rmvm_endpoint.clone()
    };
    let provider = resolve_provider(&cfg, None).ok().cloned();
    if req.copy {
        print_connect_info_block(&cfg, provider.as_ref());
        return Ok(());
    }
    let planner_model = provider.as_ref().map(|p| p.planner_model.clone());
    let view = StatusView {
        active_brain: cfg.active_brain.clone(),
        active_provider: cfg.active_provider.clone(),
        planner_model,
        proxy_addr: cfg.proxy_addr.clone(),
        dashboard_url: dashboard_url(&cfg),
        proxy_healthy: probe_proxy(&cfg.proxy_addr).await,
        rmvm_endpoint: endpoint.clone(),
        rmvm_mode: if runtime.rmvm_mode.is_empty() {
            cfg.rmvm.mode.clone()
        } else {
            runtime.rmvm_mode.clone()
        },
        rmvm_healthy: probe_rmvm(&endpoint).await,
        runtime_proxy_pid: runtime.proxy_pid,
        runtime_rmvm_pid: runtime.rmvm_pid,
        config_path: paths.config_file().display().to_string(),
        state_path: paths.state_dir.display().to_string(),
    };
    if req.json {
        println!("{}", serde_json::to_string_pretty(&view)?);
    } else {
        println!("brain={}", view.active_brain.as_deref().unwrap_or("<none>"));
        println!(
            "provider={} model={}",
            view.active_provider,
            view.planner_model.as_deref().unwrap_or("<none>")
        );
        println!("proxy={} healthy={}", view.proxy_addr, view.proxy_healthy);
        println!(
            "rmvm_endpoint={} mode={} healthy={}",
            view.rmvm_endpoint, view.rmvm_mode, view.rmvm_healthy
        );
        println!(
            "runtime proxy_pid={:?} rmvm_pid={:?}",
            view.runtime_proxy_pid, view.runtime_rmvm_pid
        );
        println!("dashboard={}", view.dashboard_url);
        let overall = if view.proxy_healthy && view.rmvm_healthy {
            "healthy"
        } else {
            "degraded"
        };
        println!("health={}", overall);
        if req.verbose {
            println!("config={}", view.config_path);
            println!("state={}", view.state_path);
            println!("hint=run `cortex open` to view the local dashboard");
        }
    }
    Ok(())
}

fn print_tail(path: &Path, tail: usize) -> Result<()> {
    if !path.exists() {
        println!("{} not found", path.display());
        return Ok(());
    }
    let content = fs::read_to_string(path)?;
    let lines = content.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(tail);
    for line in &lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn print_new_bytes(path: &Path, offset: u64) -> Result<u64> {
    if !path.exists() {
        return Ok(offset);
    }
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    if len <= offset {
        return Ok(offset);
    }
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let start = offset as usize;
    if start < buffer.len() {
        let chunk = &buffer[start..];
        print!("{}", String::from_utf8_lossy(chunk));
        std::io::stdout().flush()?;
    }
    Ok(len)
}

pub async fn run_logs(req: LogsRequest) -> Result<()> {
    let paths = default_paths()?;
    let service = req.service.to_ascii_lowercase();
    if service != "proxy" && service != "rmvm" && service != "all" {
        bail!("--service must be proxy|rmvm|all");
    }
    if service == "proxy" || service == "all" {
        println!("== proxy ==");
        print_tail(&paths.proxy_log_file(), req.tail)?;
    }
    if service == "rmvm" || service == "all" {
        println!("== rmvm ==");
        print_tail(&paths.rmvm_log_file(), req.tail)?;
    }
    if !req.follow {
        return Ok(());
    }
    let mut offset_proxy = file_len(&paths.proxy_log_file());
    let mut offset_rmvm = file_len(&paths.rmvm_log_file());
    loop {
        if service == "proxy" || service == "all" {
            offset_proxy = print_new_bytes(&paths.proxy_log_file(), offset_proxy)?;
        }
        if service == "rmvm" || service == "all" {
            offset_rmvm = print_new_bytes(&paths.rmvm_log_file(), offset_rmvm)?;
        }
        sleep(Duration::from_millis(750)).await;
    }
}

async fn maybe_restart_proxy(paths: &Paths, cfg: &ProductConfig) -> Result<()> {
    let runtime = load_runtime(paths)?;
    let Some(mut runtime) = runtime else {
        println!("Proxy is not running; config updated.");
        return Ok(());
    };
    if let Some(pid) = runtime.proxy_pid {
        kill_pid(pid, true);
    }
    let provider = resolve_provider(cfg, None)?.clone();
    let planner_key = planner_api_key(paths, &provider)?;
    let endpoint = if runtime.rmvm_endpoint.is_empty() {
        rmvm_endpoint(cfg)
    } else {
        runtime.rmvm_endpoint.clone()
    };
    let proxy_pid = spawn_proxy(cfg, paths, &endpoint, &provider, planner_key)?;
    if !wait_for_proxy(&cfg.proxy_addr, Duration::from_secs(10)).await {
        bail!(
            "proxy restart failed health check; see {}",
            paths.proxy_log_file().display()
        );
    }
    runtime.proxy_pid = Some(proxy_pid);
    save_runtime(paths, &runtime)?;
    println!("Proxy restarted on {}", cfg.proxy_addr);
    Ok(())
}

pub async fn provider_list(json: bool) -> Result<()> {
    let paths = default_paths()?;
    let cfg = load_config(&paths)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&cfg.providers)?);
    } else {
        for (name, profile) in &cfg.providers {
            let marker = if name == &cfg.active_provider { "*" } else { " " };
            println!(
                "{} {} mode={} model={} base_url={}",
                marker, name, profile.planner_mode, profile.planner_model, profile.planner_base_url
            );
        }
    }
    Ok(())
}

pub async fn provider_use(name: &str, model: Option<String>, restart: RestartPolicy) -> Result<()> {
    let paths = default_paths()?;
    let mut cfg = load_config(&paths)?;
    if !cfg.providers.contains_key(name) {
        bail!("unknown provider '{}'", name);
    }
    cfg.active_provider = name.to_string();
    if let Some(model) = model {
        if let Some(profile) = cfg.providers.get_mut(name) {
            profile.planner_model = model;
        }
    }
    save_config(&paths, &cfg)?;
    println!("Active provider set to {}", name);
    if restart == RestartPolicy::Auto {
        maybe_restart_proxy(&paths, &cfg).await?;
    }
    Ok(())
}

pub async fn provider_set_model(
    provider: Option<String>,
    model: String,
    restart: RestartPolicy,
) -> Result<()> {
    let paths = default_paths()?;
    let mut cfg = load_config(&paths)?;
    let provider_name = provider.unwrap_or_else(|| cfg.active_provider.clone());
    let profile = cfg
        .providers
        .get_mut(&provider_name)
        .ok_or_else(|| anyhow!("unknown provider '{}'", provider_name))?;
    profile.planner_model = model.clone();
    save_config(&paths, &cfg)?;
    println!("Provider {} model set to {}", provider_name, model);
    if restart == RestartPolicy::Auto {
        maybe_restart_proxy(&paths, &cfg).await?;
    }
    Ok(())
}

pub fn brain_current(json: bool) -> Result<()> {
    let paths = default_paths()?;
    let cfg = load_config(&paths)?;
    ensure_brain_secret_env(&paths, &cfg)?;
    let store = BrainStore::new(None)?;
    let summary = store.resolve_brain_or_active(cfg.active_brain.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!(
            "{} [{}] tenant={} branch={}",
            summary.name, summary.brain_id, summary.tenant_id, summary.active_branch
        );
    }
    Ok(())
}

pub async fn open_config(print_only: bool, url_only: bool) -> Result<()> {
    let paths = default_paths()?;
    let cfg = load_config(&paths)?;
    let url = dashboard_url(&cfg);
    if url_only {
        println!("{}", url);
        return Ok(());
    }
    println!("Dashboard URL: {}", url);
    println!("Proxy health URL: http://{}/healthz", cfg.proxy_addr);
    println!("Config file: {}", paths.config_file().display());
    println!("State dir: {}", paths.state_dir.display());
    if !print_only {
        if open_in_browser(&url) {
            println!("Opened dashboard in your browser.");
        } else {
            println!("Could not open browser automatically; copy the dashboard URL above.");
        }
    }
    Ok(())
}

fn open_in_browser(url: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        return Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(url)
            .spawn()
            .is_ok();
    }
    #[cfg(target_os = "macos")]
    {
        return Command::new("open").arg(url).spawn().is_ok();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return Command::new("xdg-open").arg(url).spawn().is_ok();
    }
    #[allow(unreachable_code)]
    false
}

pub fn load_saved_proxy_api_key() -> Result<Option<String>> {
    let paths = default_paths()?;
    let cfg = load_config(&paths)?;
    Ok(cfg.proxy_api_key)
}

pub fn ensure_saved_brain_secret_env() -> Result<()> {
    let paths = default_paths()?;
    let cfg = load_config(&paths)?;
    ensure_brain_secret_env(&paths, &cfg)
}
