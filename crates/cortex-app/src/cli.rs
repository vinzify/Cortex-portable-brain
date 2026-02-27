use std::path::PathBuf;
use std::time::Duration;

use adapter_rmvm::RmvmAdapter;
use anyhow::{Result, bail};
use brain_store::{AttachmentGrant, BrainStore, CreateBrainRequest, MergeStrategy};
use clap::{Args, Parser, Subcommand, ValueEnum};
use planner_guard::deterministic_plan_from_manifest;
use reqwest::Client;
use rmvm_grpc::{
    AppendEventRequest, GetManifestRequest, GrpcKernelService, RmvmExecutorServer,
};
use rmvm_proto::{ExecuteRequest, ExecutionStatus, Scope};
use tonic::transport::Server;
use uuid::Uuid;

use crate::product::{
    ConnectRequest, ConnectSetRequest, ConnectStatusRequest, LogsRequest, ModeSetRequest,
    ModeStatusRequest, RestartPolicy, SetupRequest, StatusRequest, StopRequest, UpRequest,
    brain_current, ensure_saved_brain_secret_env, load_saved_proxy_api_key, open_config,
    provider_list, provider_set_model, provider_use, run_connect, run_connect_set,
    run_connect_status, run_logs, run_mode_set, run_mode_status, run_setup, run_status, run_stop,
    run_uninstall, run_up,
};
use crate::proxy::{PlannerConfig, PlannerMode, ProxyConfig, parse_addr, serve};

#[derive(Debug, Parser)]
#[command(name = "cortex", about = "Portable Brain + Proxy UX CLI")]
pub struct Cli {
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Debug, Subcommand)]
enum TopCommand {
    Brain {
        #[command(subcommand)]
        command: BrainCommand,
    },
    Proxy {
        #[command(subcommand)]
        command: ProxyCommand,
    },
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    Doctor(DoctorCmd),
    Setup(SetupCmd),
    Connect {
        #[command(subcommand)]
        command: Option<ConnectCommand>,
        #[arg(long)]
        non_interactive: bool,
    },
    Mode {
        #[command(subcommand)]
        command: ModeCommand,
    },
    Up(UpCmd),
    Stop(StopCmd),
    Uninstall(UninstallCmd),
    Status(StatusCmd),
    Logs(LogsCmd),
    Provider {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Open(OpenCmd),
    #[command(hide = true)]
    Rmvm {
        #[command(subcommand)]
        command: RmvmCommand,
    },
}

#[derive(Debug, Subcommand)]
enum BrainCommand {
    Create(CreateCmd),
    #[command(alias = "open")]
    Use(UseCmd),
    List(ListCmd),
    Export(ExportCmd),
    Import(ImportCmd),
    Branch(BranchCmd),
    Merge(MergeCmd),
    Forget(ForgetCmd),
    Attach(AttachCmd),
    Detach(DetachCmd),
    Audit(AuditCmd),
    Current(CurrentCmd),
}

#[derive(Debug, Subcommand)]
enum ProxyCommand {
    Serve(ServeCmd),
}

#[derive(Debug, Subcommand)]
enum AuthCommand {
    MapKey(MapKeyCmd),
}

#[derive(Debug, Subcommand)]
enum ProviderCommand {
    List(ProviderListCmd),
    Use(ProviderUseCmd),
    SetModel(ProviderSetModelCmd),
}

#[derive(Debug, Subcommand)]
enum ConnectCommand {
    Status(ConnectStatusCmd),
    Enable(ConnectToggleCmd),
    Disable(ConnectToggleCmd),
}

#[derive(Debug, Subcommand)]
enum ModeCommand {
    Set(ModeSetCmd),
    Status(ModeStatusCmd),
}

#[derive(Debug, Subcommand)]
enum RmvmCommand {
    Serve(RmvmServeCmd),
}

#[derive(Debug, Args)]
struct CreateCmd {
    name: String,
    #[arg(long)]
    path: Option<PathBuf>,
    #[arg(long, default_value = "local")]
    tenant: String,
    #[arg(long)]
    passphrase_env: Option<String>,
}

#[derive(Debug, Args)]
struct UseCmd {
    brain: String,
}

#[derive(Debug, Args)]
struct ListCmd {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ExportCmd {
    brain: String,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    signing_key: Option<String>,
}

#[derive(Debug, Args)]
struct ImportCmd {
    #[arg(long = "in")]
    input: PathBuf,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    verify_only: bool,
}

#[derive(Debug, Args)]
struct BranchCmd {
    brain: String,
    #[arg(long = "new")]
    new_branch: String,
}

#[derive(Debug, ValueEnum, Clone)]
enum MergeStrategyArg {
    Ours,
    Theirs,
    Manual,
}

#[derive(Debug, Args)]
struct MergeCmd {
    #[arg(long)]
    source: String,
    #[arg(long)]
    target: String,
    #[arg(long, value_enum, default_value = "ours")]
    strategy: MergeStrategyArg,
    #[arg(long)]
    brain: Option<String>,
}

#[derive(Debug, Args)]
struct ForgetCmd {
    #[arg(long)]
    subject: String,
    #[arg(long = "predicate")]
    predicate: String,
    #[arg(long, default_value = "SCOPE_GLOBAL")]
    scope: String,
    #[arg(long, default_value = "suppress preference")]
    reason: String,
    #[arg(long)]
    brain: Option<String>,
}

#[derive(Debug, Args)]
struct AttachCmd {
    #[arg(long = "agent")]
    agent: String,
    #[arg(long = "model")]
    model: String,
    #[arg(long)]
    read: String,
    #[arg(long)]
    write: String,
    #[arg(long)]
    sinks: String,
    #[arg(long)]
    ttl: Option<String>,
    #[arg(long)]
    brain: Option<String>,
}

#[derive(Debug, Args)]
struct DetachCmd {
    #[arg(long = "agent")]
    agent: String,
    #[arg(long = "model")]
    model: Option<String>,
    #[arg(long)]
    brain: Option<String>,
}

#[derive(Debug, Args)]
struct AuditCmd {
    #[arg(long)]
    since: Option<String>,
    #[arg(long)]
    until: Option<String>,
    #[arg(long)]
    subject: Option<String>,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    brain: Option<String>,
}

#[derive(Debug, Args)]
struct ServeCmd {
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: String,
    #[arg(
        long,
        env = "CORTEX_ENDPOINT",
        default_value = "grpc://127.0.0.1:50051"
    )]
    endpoint: String,
    #[arg(long, env = "CORTEX_BRAIN")]
    brain: Option<String>,
    #[arg(long, env = "CORTEX_PLANNER_MODE", default_value = "fallback")]
    planner_mode: String,
    #[arg(
        long,
        env = "CORTEX_PLANNER_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    planner_base_url: String,
    #[arg(long, env = "CORTEX_PLANNER_MODEL", default_value = "gpt-4o-mini")]
    planner_model: String,
    #[arg(long, env = "CORTEX_PLANNER_API_KEY")]
    planner_api_key: Option<String>,
    #[arg(long, env = "CORTEX_PLANNER_TIMEOUT_SECS", default_value = "30")]
    planner_timeout_secs: u64,
    #[arg(long, hide = true)]
    provider_name: Option<String>,
    #[arg(long, hide = true)]
    proxy_api_key: Option<String>,
}

#[derive(Debug, Args)]
struct MapKeyCmd {
    #[arg(long = "api-key")]
    api_key: String,
    #[arg(long)]
    tenant: String,
    #[arg(long)]
    brain: String,
    #[arg(long, default_value = "user:local")]
    subject: String,
}

#[derive(Debug, Args)]
struct DoctorCmd {
    #[arg(long, env = "OPENAI_BASE_URL", default_value = "http://127.0.0.1:8080/v1")]
    proxy_base_url: String,
    #[arg(
        long,
        env = "CORTEX_ENDPOINT",
        default_value = "grpc://127.0.0.1:50051"
    )]
    endpoint: String,
    #[arg(long, env = "CORTEX_BRAIN")]
    brain: Option<String>,
    #[arg(long, env = "OPENAI_API_KEY")]
    api_key: Option<String>,
    #[arg(long, env = "CORTEX_PLANNER_MODE", default_value = "fallback")]
    planner_mode: String,
    #[arg(
        long,
        env = "CORTEX_PLANNER_BASE_URL",
        default_value = "https://api.openai.com/v1"
    )]
    planner_base_url: String,
    #[arg(long, env = "CORTEX_PLANNER_MODEL", default_value = "gpt-4o-mini")]
    planner_model: String,
    #[arg(long, env = "CORTEX_PLANNER_API_KEY")]
    planner_api_key: Option<String>,
    #[arg(long, default_value = "10")]
    timeout_secs: u64,
}

struct DoctorCheck {
    label: &'static str,
    ok: bool,
    details: String,
}

#[derive(Debug, Args)]
struct SetupCmd {
    #[arg(long)]
    non_interactive: bool,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    planner_base_url: Option<String>,
    #[arg(long)]
    planner_api_key: Option<String>,
    #[arg(long)]
    planner_api_key_env: Option<String>,
    #[arg(long)]
    brain: Option<String>,
    #[arg(long, default_value = "local")]
    tenant: String,
    #[arg(long = "api-key")]
    api_key: Option<String>,
    #[arg(long)]
    rmvm_endpoint: Option<String>,
    #[arg(long)]
    proxy_addr: Option<String>,
    #[arg(long)]
    rmvm_port: Option<u16>,
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct UpCmd {
    #[arg(long, default_value = "true")]
    detached: String,
    #[arg(long)]
    proxy_addr: Option<String>,
    #[arg(long)]
    rmvm_endpoint: Option<String>,
    #[arg(long)]
    rmvm_port: Option<u16>,
    #[arg(long)]
    brain: Option<String>,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long, default_value_t = true)]
    reuse_external_rmvm: bool,
}

#[derive(Debug, Args)]
struct StopCmd {
    #[arg(long)]
    all: bool,
    #[arg(long)]
    proxy_only: bool,
    #[arg(long)]
    rmvm_only: bool,
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct UninstallCmd {
    #[arg(long)]
    all: bool,
    #[arg(long)]
    yes: bool,
}

#[derive(Debug, Args)]
struct StatusCmd {
    #[arg(long)]
    json: bool,
    #[arg(long)]
    verbose: bool,
    #[arg(long)]
    copy: bool,
}

#[derive(Debug, Args)]
struct LogsCmd {
    #[arg(long, default_value = "all")]
    service: String,
    #[arg(long, default_value_t = 100)]
    tail: usize,
    #[arg(long)]
    follow: bool,
}

#[derive(Debug, Args)]
struct ConnectStatusCmd {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ConnectToggleCmd {
    name: String,
}

#[derive(Debug, Args)]
struct ModeSetCmd {
    mode: String,
}

#[derive(Debug, Args)]
struct ModeStatusCmd {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ProviderListCmd {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct ProviderUseCmd {
    name: String,
    #[arg(long)]
    model: Option<String>,
    #[arg(long, default_value = "auto")]
    restart: String,
}

#[derive(Debug, Args)]
struct ProviderSetModelCmd {
    model: String,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long, default_value = "auto")]
    restart: String,
}

#[derive(Debug, Args)]
struct CurrentCmd {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct OpenCmd {
    #[arg(long)]
    print_only: bool,
    #[arg(long)]
    url: bool,
}

#[derive(Debug, Args)]
struct RmvmServeCmd {
    #[arg(long, env = "RMVM_SERVER_ADDR", default_value = "127.0.0.1:50051")]
    addr: String,
    #[arg(long, env = "RMVM_MAX_DECODING_BYTES", default_value_t = 4 * 1024 * 1024)]
    max_decoding_bytes: usize,
    #[arg(long, env = "RMVM_MAX_ENCODING_BYTES", default_value_t = 4 * 1024 * 1024)]
    max_encoding_bytes: usize,
    #[arg(long, env = "RMVM_REQUEST_TIMEOUT_SECS", default_value_t = 30)]
    request_timeout_secs: u64,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        TopCommand::Brain { command } => handle_brain(command).await,
        TopCommand::Proxy { command } => handle_proxy(command).await,
        TopCommand::Auth { command } => handle_auth(command).await,
        TopCommand::Doctor(command) => handle_doctor(command).await,
        TopCommand::Setup(command) => handle_setup(command).await,
        TopCommand::Connect {
            command,
            non_interactive,
        } => handle_connect(command, non_interactive).await,
        TopCommand::Mode { command } => handle_mode(command).await,
        TopCommand::Up(command) => handle_up(command).await,
        TopCommand::Stop(command) => handle_stop(command).await,
        TopCommand::Uninstall(command) => handle_uninstall(command).await,
        TopCommand::Status(command) => handle_status(command).await,
        TopCommand::Logs(command) => handle_logs(command).await,
        TopCommand::Provider { command } => handle_provider(command).await,
        TopCommand::Open(command) => handle_open(command).await,
        TopCommand::Rmvm { command } => handle_rmvm(command).await,
    }
}

async fn handle_brain(cmd: BrainCommand) -> Result<()> {
    let _ = ensure_saved_brain_secret_env();
    let store = BrainStore::new(None)?;
    match cmd {
        BrainCommand::Create(c) => {
            let store = if let Some(path) = c.path {
                BrainStore::new(Some(path))?
            } else {
                store
            };
            let created = store.create_brain(CreateBrainRequest {
                name: c.name,
                tenant_id: c.tenant,
                passphrase_env: c.passphrase_env,
            })?;
            println!("Created brain {} ({})", created.name, created.brain_id);
            println!("Set active with: cortex brain use {}", created.brain_id);
        }
        BrainCommand::Use(c) => {
            let s = store.set_active_brain(&c.brain)?;
            println!("Active brain set: {} ({})", s.name, s.brain_id);
        }
        BrainCommand::List(c) => {
            let list = store.list_brains()?;
            if c.json {
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else {
                let active = store.active_brain_id()?;
                for b in list {
                    let marker = if active.as_ref() == Some(&b.brain_id) {
                        "*"
                    } else {
                        " "
                    };
                    println!(
                        "{} {} [{}] tenant={} branch={}",
                        marker, b.name, b.brain_id, b.tenant_id, b.active_branch
                    );
                }
            }
        }
        BrainCommand::Export(c) => {
            let _ = c.signing_key;
            store.export_brain(&c.brain, &c.out)?;
            println!("Exported brain {} to {}", c.brain, c.out.display());
        }
        BrainCommand::Import(c) => {
            let res = store.import_brain(&c.input, c.name, c.verify_only)?;
            if c.verify_only {
                println!("Import verification passed: {}", c.input.display());
            } else if let Some(summary) = res {
                println!("Imported brain {} ({})", summary.name, summary.brain_id);
            }
        }
        BrainCommand::Branch(c) => {
            store.branch(&c.brain, &c.new_branch)?;
            println!("Created branch {} in {}", c.new_branch, c.brain);
        }
        BrainCommand::Merge(c) => {
            let strategy = match c.strategy {
                MergeStrategyArg::Ours => MergeStrategy::Ours,
                MergeStrategyArg::Theirs => MergeStrategy::Theirs,
                MergeStrategyArg::Manual => MergeStrategy::Manual,
            };
            let brain = store.resolve_brain_or_active(c.brain.as_deref())?;
            let report = store.merge(&brain.brain_id, &c.source, &c.target, strategy)?;
            println!(
                "Merged source={} target={} merged={} conflicts={}",
                c.source,
                c.target,
                report.merged,
                report.conflicts.len()
            );
        }
        BrainCommand::Forget(c) => {
            let brain = store.resolve_brain_or_active(c.brain.as_deref())?;
            let suppressed = store.forget_suppress(
                &brain.brain_id,
                &c.subject,
                &c.predicate,
                &c.scope,
                &c.reason,
            )?;
            println!(
                "Suppressed {} objects for subject={} predicate={}",
                suppressed, c.subject, c.predicate
            );
        }
        BrainCommand::Attach(c) => {
            let brain = store.resolve_brain_or_active(c.brain.as_deref())?;
            store.attach(
                &brain.brain_id,
                AttachmentGrant {
                    agent_id: c.agent,
                    model_id: c.model,
                    read_classes: split_csv(&c.read),
                    write_classes: split_csv(&c.write),
                    sinks: split_csv(&c.sinks),
                    expires_at: c.ttl,
                },
            )?;
            println!("Attachment saved for brain {}", brain.brain_id);
        }
        BrainCommand::Detach(c) => {
            let brain = store.resolve_brain_or_active(c.brain.as_deref())?;
            let removed = store.detach(&brain.brain_id, &c.agent, c.model.as_deref())?;
            println!("Removed {} attachment(s)", removed);
        }
        BrainCommand::Audit(c) => {
            let brain = store.resolve_brain_or_active(c.brain.as_deref())?;
            let mut rows = store.audit_trace(&brain.brain_id)?;
            if let Some(subject) = c.subject {
                rows.retain(|r| r.details.to_string().contains(&subject));
            }
            if c.since.is_some() || c.until.is_some() {
                // v0: filters accepted for UX compatibility; strict timestamp filtering lands in next cut.
            }
            if c.json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                for row in rows {
                    println!("{} {} {} {}", row.ts, row.actor, row.action, row.details);
                }
            }
        }
        BrainCommand::Current(c) => {
            brain_current(c.json)?;
        }
    }
    Ok(())
}

async fn handle_proxy(cmd: ProxyCommand) -> Result<()> {
    match cmd {
        ProxyCommand::Serve(c) => {
            let _ = RmvmAdapter::new(c.endpoint.clone());
            let bind_addr = parse_addr(&c.addr)?;
            let planner_mode = PlannerMode::parse(&c.planner_mode)?;
            serve(ProxyConfig {
                bind_addr,
                endpoint: c.endpoint,
                default_brain: c.brain,
                brain_home: None,
                planner: PlannerConfig {
                    mode: planner_mode,
                    base_url: c.planner_base_url,
                    model: c.planner_model,
                    api_key: c
                        .planner_api_key
                        .or_else(|| std::env::var("OPENAI_API_KEY").ok()),
                    timeout: Duration::from_secs(c.planner_timeout_secs),
                },
                provider_name: c.provider_name,
                proxy_api_key: c.proxy_api_key,
            })
            .await
        }
    }
}

async fn handle_auth(cmd: AuthCommand) -> Result<()> {
    let store = BrainStore::new(None)?;
    match cmd {
        AuthCommand::MapKey(c) => {
            let brain = store.resolve_brain(&c.brain)?;
            if brain.tenant_id != c.tenant {
                bail!(
                    "tenant mismatch: brain tenant={} but --tenant={}",
                    brain.tenant_id,
                    c.tenant
                );
            }
            store.map_api_key(&c.api_key, &c.tenant, &brain.brain_id, &c.subject)?;
            println!("Mapped API key to brain {}", brain.brain_id);
        }
    }
    Ok(())
}

async fn handle_setup(cmd: SetupCmd) -> Result<()> {
    let out = run_setup(SetupRequest {
        non_interactive: cmd.non_interactive,
        provider: cmd.provider,
        model: cmd.model,
        planner_base_url: cmd.planner_base_url,
        planner_api_key: cmd.planner_api_key,
        planner_api_key_env: cmd.planner_api_key_env,
        brain: cmd.brain,
        tenant: cmd.tenant,
        api_key: cmd.api_key,
        rmvm_endpoint: cmd.rmvm_endpoint,
        proxy_addr: cmd.proxy_addr,
        rmvm_port: cmd.rmvm_port,
        force: cmd.force,
    })?;
    println!("Setup complete:");
    println!("  brain={}", out.brain_id);
    println!("  provider={} model={}", out.provider, out.model);
    println!("  proxy=http://{}", out.proxy_addr);
    println!("  rmvm={} ({})", out.rmvm_mode, out.rmvm_endpoint);
    println!("Next: cortex up");
    Ok(())
}

async fn handle_connect(command: Option<ConnectCommand>, non_interactive: bool) -> Result<()> {
    match command {
        None => run_connect(ConnectRequest { non_interactive }),
        Some(ConnectCommand::Status(c)) => run_connect_status(ConnectStatusRequest { json: c.json }),
        Some(ConnectCommand::Enable(c)) => run_connect_set(ConnectSetRequest {
            name: c.name,
            enabled: true,
        }),
        Some(ConnectCommand::Disable(c)) => run_connect_set(ConnectSetRequest {
            name: c.name,
            enabled: false,
        }),
    }
}

async fn handle_mode(command: ModeCommand) -> Result<()> {
    match command {
        ModeCommand::Set(c) => run_mode_set(ModeSetRequest { mode: c.mode }),
        ModeCommand::Status(c) => run_mode_status(ModeStatusRequest { json: c.json }),
    }
}

async fn handle_up(cmd: UpCmd) -> Result<()> {
    run_up(UpRequest {
        detached: parse_bool_flag("detached", &cmd.detached)?,
        proxy_addr: cmd.proxy_addr,
        rmvm_endpoint: cmd.rmvm_endpoint,
        rmvm_port: cmd.rmvm_port,
        brain: cmd.brain,
        provider: cmd.provider,
        reuse_external_rmvm: cmd.reuse_external_rmvm,
    })
    .await
}

async fn handle_stop(cmd: StopCmd) -> Result<()> {
    run_stop(StopRequest {
        all: cmd.all,
        proxy_only: cmd.proxy_only,
        rmvm_only: cmd.rmvm_only,
        force: cmd.force,
    })
}

async fn handle_uninstall(cmd: UninstallCmd) -> Result<()> {
    run_uninstall(crate::product::UninstallRequest {
        all: cmd.all,
        yes: cmd.yes,
    })
}

async fn handle_status(cmd: StatusCmd) -> Result<()> {
    run_status(StatusRequest {
        json: cmd.json,
        verbose: cmd.verbose,
        copy: cmd.copy,
    })
    .await
}

async fn handle_logs(cmd: LogsCmd) -> Result<()> {
    run_logs(LogsRequest {
        service: cmd.service,
        tail: cmd.tail,
        follow: cmd.follow,
    })
    .await
}

async fn handle_provider(cmd: ProviderCommand) -> Result<()> {
    match cmd {
        ProviderCommand::List(c) => provider_list(c.json).await,
        ProviderCommand::Use(c) => {
            provider_use(&c.name, c.model, parse_restart_policy(&c.restart)?).await
        }
        ProviderCommand::SetModel(c) => {
            provider_set_model(c.provider, c.model, parse_restart_policy(&c.restart)?).await
        }
    }
}

async fn handle_open(cmd: OpenCmd) -> Result<()> {
    open_config(cmd.print_only, cmd.url).await
}

async fn handle_rmvm(cmd: RmvmCommand) -> Result<()> {
    match cmd {
        RmvmCommand::Serve(c) => {
            let addr = c
                .addr
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid RMVM address '{}': {e}", c.addr))?;
            let service = GrpcKernelService::default();
            let service = RmvmExecutorServer::new(service)
                .max_decoding_message_size(c.max_decoding_bytes)
                .max_encoding_message_size(c.max_encoding_bytes);
            println!(
                "RMVM gRPC server listening on {} (decode={} encode={} timeout={}s)",
                addr, c.max_decoding_bytes, c.max_encoding_bytes, c.request_timeout_secs
            );
            Server::builder()
                .timeout(Duration::from_secs(c.request_timeout_secs))
                .add_service(service)
                .serve(addr)
                .await?;
            Ok(())
        }
    }
}

async fn handle_doctor(cmd: DoctorCmd) -> Result<()> {
    let _ = ensure_saved_brain_secret_env();
    let timeout = Duration::from_secs(cmd.timeout_secs);
    let http = Client::builder().timeout(timeout).build()?;
    let store = BrainStore::new(None)?;

    let planner_mode = PlannerMode::parse(&cmd.planner_mode)?;
    let proxy_base_url = cmd.proxy_base_url.trim_end_matches('/').to_string();
    let healthz_url = derive_healthz_url(&proxy_base_url);
    let planner_api_key = cmd
        .planner_api_key
        .clone()
        .or_else(|| std::env::var("OPENAI_API_KEY").ok());
    let resolved_proxy_api_key = cmd
        .api_key
        .clone()
        .or_else(|| std::env::var("OPENAI_API_KEY").ok())
        .or_else(|| load_saved_proxy_api_key().ok().flatten());

    let mut failures = 0usize;
    let mut subject_for_dry_run = "user:local".to_string();
    let mut active_brain_id: Option<String> = None;

    let brain_check = match store.resolve_brain_or_active(cmd.brain.as_deref()) {
        Ok(brain) => {
            active_brain_id = Some(brain.brain_id.clone());
            DoctorCheck {
                label: "brain_unlocked",
                ok: true,
                details: format!("brain={} tenant={}", brain.brain_id, brain.tenant_id),
            }
        }
        Err(e) => DoctorCheck {
            label: "brain_unlocked",
            ok: false,
            details: format!("could not resolve active brain: {e}"),
        },
    };
    failures += print_doctor_check(brain_check);

    let api_key_check = match resolved_proxy_api_key.as_deref() {
        Some(api_key) => match store.resolve_api_key(api_key) {
            Ok(Some(mapping)) => {
                if let Some(expected_brain) = active_brain_id.as_deref() {
                    if mapping.brain_id != expected_brain {
                        DoctorCheck {
                            label: "api_key_mapped",
                            ok: false,
                            details: format!(
                                "mapped to brain {} but active brain is {}",
                                mapping.brain_id, expected_brain
                            ),
                        }
                    } else {
                        subject_for_dry_run = mapping.subject.clone();
                        DoctorCheck {
                            label: "api_key_mapped",
                            ok: true,
                            details: format!(
                                "tenant={} brain={} subject={}",
                                mapping.tenant_id, mapping.brain_id, mapping.subject
                            ),
                        }
                    }
                } else {
                    subject_for_dry_run = mapping.subject.clone();
                    DoctorCheck {
                        label: "api_key_mapped",
                        ok: true,
                        details: format!(
                            "tenant={} brain={} subject={}",
                            mapping.tenant_id, mapping.brain_id, mapping.subject
                        ),
                    }
                }
            }
            Ok(None) => DoctorCheck {
                label: "api_key_mapped",
                ok: false,
                details: "API key is not mapped; run cortex auth map-key".to_string(),
            },
            Err(e) => DoctorCheck {
                label: "api_key_mapped",
                ok: false,
                details: format!("failed to resolve API key: {e}"),
            },
        },
        None => DoctorCheck {
            label: "api_key_mapped",
            ok: false,
            details: "missing API key; set OPENAI_API_KEY or pass --api-key".to_string(),
        },
    };
    failures += print_doctor_check(api_key_check);

    let planner_check = match planner_mode {
        PlannerMode::Fallback => DoctorCheck {
            label: "planner_reachable",
            ok: true,
            details: "planner mode is fallback (no remote planner required)".to_string(),
        },
        PlannerMode::ByoHeader => DoctorCheck {
            label: "planner_reachable",
            ok: true,
            details: "planner mode is byo; per-request X-Cortex-Plan header expected".to_string(),
        },
        PlannerMode::OpenAi => {
            let planner_url = format!(
                "{}/chat/completions",
                cmd.planner_base_url.trim_end_matches('/')
            );
            let payload = serde_json::json!({
                "model": cmd.planner_model,
                "messages": [{"role": "user", "content": "Return only {}"}],
                "temperature": 0,
                "max_tokens": 1
            });
            let requires_key = planner_base_url_requires_api_key(&cmd.planner_base_url);
            if planner_api_key.is_none() && requires_key {
                DoctorCheck {
                    label: "planner_reachable",
                    ok: false,
                    details: format!(
                        "planner API key required for {} (set CORTEX_PLANNER_API_KEY or OPENAI_API_KEY)",
                        cmd.planner_base_url
                    ),
                }
            } else {
                let request = http.post(&planner_url).json(&payload);
                let request = if let Some(api_key) = planner_api_key.clone() {
                    request.bearer_auth(api_key)
                } else {
                    request
                };
                match request.send().await {
                    Ok(response) => {
                        let status = response.status();
                        if status.is_server_error() {
                            DoctorCheck {
                                label: "planner_reachable",
                                ok: false,
                                details: format!("planner endpoint returned HTTP {}", status),
                            }
                        } else {
                            DoctorCheck {
                                label: "planner_reachable",
                                ok: true,
                                details: format!("planner endpoint reachable (HTTP {})", status),
                            }
                        }
                    }
                    Err(e) => DoctorCheck {
                        label: "planner_reachable",
                        ok: false,
                        details: format!("planner request failed: {e}"),
                    },
                }
            }
        }
    };
    failures += print_doctor_check(planner_check);

    let proxy_check = match http.get(&healthz_url).send().await {
        Ok(response) => {
            let status = response.status();
            match response.text().await {
                Ok(body) if status.is_success() && body.trim().eq_ignore_ascii_case("ok") => {
                    DoctorCheck {
                        label: "proxy_reachable",
                        ok: true,
                        details: format!("{} -> {}", healthz_url, status),
                    }
                }
                Ok(body) => DoctorCheck {
                    label: "proxy_reachable",
                    ok: false,
                    details: format!(
                        "{} responded HTTP {} body='{}'",
                        healthz_url,
                        status,
                        body.trim()
                    ),
                },
                Err(e) => DoctorCheck {
                    label: "proxy_reachable",
                    ok: false,
                    details: format!("failed to read /healthz response: {e}"),
                },
            }
        }
        Err(e) => DoctorCheck {
            label: "proxy_reachable",
            ok: false,
            details: format!("could not reach {}: {e}", healthz_url),
        },
    };
    failures += print_doctor_check(proxy_check);

    let dry_run_check = run_dry_execute_check(&cmd.endpoint, &subject_for_dry_run).await;
    failures += print_doctor_check(dry_run_check);

    if failures > 0 {
        bail!("doctor found {} failing check(s)", failures);
    }

    println!("doctor summary: all checks passed");
    Ok(())
}

async fn run_dry_execute_check(endpoint: &str, subject: &str) -> DoctorCheck {
    let adapter = RmvmAdapter::new(endpoint.to_string());
    let request_id = format!("doctor-{}", Uuid::new_v4().simple());

    if let Err(e) = adapter
        .append_event(AppendEventRequest {
            request_id: request_id.clone(),
            subject: subject.to_string(),
            text: "[doctor] dry-run memory check".to_string(),
            scope: Scope::Global as i32,
        })
        .await
    {
        return DoctorCheck {
            label: "dry_run_execute",
            ok: false,
            details: format!("append_event failed on {}: {e}", adapter.endpoint()),
        };
    }

    let manifest = match adapter
        .get_manifest(GetManifestRequest {
            request_id: request_id.clone(),
        })
        .await
    {
        Ok(resp) => match resp.manifest {
            Some(manifest) => manifest,
            None => {
                return DoctorCheck {
                    label: "dry_run_execute",
                    ok: false,
                    details: "get_manifest returned no manifest".to_string(),
                };
            }
        },
        Err(e) => {
            return DoctorCheck {
                label: "dry_run_execute",
                ok: false,
                details: format!("get_manifest failed: {e}"),
            };
        }
    };

    let plan = match deterministic_plan_from_manifest(&request_id, subject, &manifest) {
        Ok(plan) => plan,
        Err(e) => {
            return DoctorCheck {
                label: "dry_run_execute",
                ok: false,
                details: format!("could not build deterministic plan from manifest: {e}"),
            };
        }
    };

    match adapter
        .execute(ExecuteRequest {
            manifest: Some(manifest),
            plan: Some(plan),
        })
        .await
    {
        Ok(execute) => {
            let status =
                ExecutionStatus::try_from(execute.status).unwrap_or(ExecutionStatus::Unspecified);
            let semantic_root = execute
                .proof
                .as_ref()
                .map(|p| p.semantic_root.clone())
                .unwrap_or_else(|| "<none>".to_string());
            DoctorCheck {
                label: "dry_run_execute",
                ok: true,
                details: format!("status={} semantic_root={}", status.as_str_name(), semantic_root),
            }
        }
        Err(e) => DoctorCheck {
            label: "dry_run_execute",
            ok: false,
            details: format!("execute failed: {e}"),
        },
    }
}

fn derive_healthz_url(proxy_base_url: &str) -> String {
    let mut base = proxy_base_url.trim_end_matches('/').to_string();
    if base.ends_with("/v1") {
        base.truncate(base.len() - 3);
    }
    format!("{}/healthz", base.trim_end_matches('/'))
}

fn planner_base_url_requires_api_key(base_url: &str) -> bool {
    let normalized = base_url.to_ascii_lowercase();
    !(normalized.contains("127.0.0.1")
        || normalized.contains("localhost")
        || normalized.contains("ollama"))
}

fn print_doctor_check(check: DoctorCheck) -> usize {
    if check.ok {
        println!("[OK]   {} {}", check.label, check.details);
        0
    } else {
        println!("[FAIL] {} {}", check.label, check.details);
        1
    }
}

fn parse_restart_policy(value: &str) -> Result<RestartPolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(RestartPolicy::Auto),
        "never" => Ok(RestartPolicy::Never),
        other => bail!("invalid restart policy '{}'; expected auto|never", other),
    }
}

fn parse_bool_flag(name: &str, value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        other => bail!("invalid --{} value '{}'; expected true|false", name, other),
    }
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
