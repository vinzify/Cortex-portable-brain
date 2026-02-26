# Cortex Brain

Portable, encrypted memory with an OpenAI-compatible local proxy.

Use one stable local endpoint for your AI clients while switching providers and models without changing your app integration.

## Quick Start (Beginner Path)

### 1) Install (No Rust Required)

macOS/Linux:
```bash
curl -fsSL https://raw.githubusercontent.com/vinzify/Cortex-portable-brain/main/install/install.sh | sh
```

Windows PowerShell:
```powershell
irm https://raw.githubusercontent.com/vinzify/Cortex-portable-brain/main/install/install.ps1 | iex
```

Docker:
```bash
docker run --rm -p 8080:8080 ghcr.io/vinzify/cortex-portable-brain:latest
```

The installer runs `cortex setup` automatically in interactive terminals.

### 2) Start everything

```bash
cortex up
```

`cortex up` starts:
- RMVM gRPC runtime (managed sidecar by default)
- Cortex proxy on `http://127.0.0.1:8080`

### 3) Point your existing OpenAI-compatible client to Cortex

```bash
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
export OPENAI_API_KEY=<your-cortex-proxy-api-key>
```

### 60-Second Smoke Test

```bash
curl -sS -i http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <your-cortex-proxy-api-key>" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}"
```

Expected shape:
```bash
HTTP/1.1 200 OK
...
{"id":"chatcmpl-...","object":"chat.completion","choices":[...],"cortex":{"status":"OK","semantic_root":"...","trace_root":"..."}}
```

If you get `STALL` or `REJECTED`, run:
```bash
cortex doctor
```

## One-Command Provider Switching

Switch planner provider:
```bash
cortex provider use claude
```

Set model:
```bash
cortex provider set-model claude-opus-4-6
```

Supported profiles:
- `openai`
- `claude`
- `gemini`
- `ollama`
- `byo`

Client-facing URL stays stable:
- `OPENAI_BASE_URL=http://127.0.0.1:8080/v1`

## Brain Management

Show current brain:
```bash
cortex brain current
```

List and switch:
```bash
cortex brain list
cortex brain use <brain-id-or-name>
```

Export and import:
```bash
cortex brain export <brain-id-or-name> --out personal.cbrain
cortex brain import --in personal.cbrain --name personal
```

Forget (suppression):
```bash
cortex brain forget --subject user:local --predicate prefers_beverage --reason "suppress preference"
```

## Runtime Controls

Start:
```bash
cortex up
```

Stop:
```bash
cortex stop --all
```

Status:
```bash
cortex status --verbose
cortex status --copy
```

Logs:
```bash
cortex logs --service all --tail 200 --follow
```

## Guided Setup

Interactive:
```bash
cortex setup
```

Non-interactive:
```bash
cortex setup --non-interactive --provider openai --brain personal --api-key ctx_demo_key
```

External RMVM endpoint (optional):
```bash
cortex setup --non-interactive --rmvm-endpoint grpc://127.0.0.1:50051
```

## What Cortex Brain Includes

- `cortex` CLI (`setup`, `up`, `stop`, `status`, `logs`, `provider`, `brain`, `auth`, `doctor`)
- Encrypted local brain store with export/import
- OpenAI-compatible `POST /v1/chat/completions` proxy
- Planner modes: `openai`, `byo`, `fallback`
- Managed RMVM sidecar runtime by default

## Three Common Use Cases

### 1) Personal assistant with evolving preferences
Keep durable preferences across providers and apply deterministic suppression when needed.

### 2) Coding agent safety
Control read/write classes and sink permissions per attached agent.

### 3) Enterprise auditability
Use `semantic_root` and `trace_root` for traceable memory-backed responses.

## Provider Docs

- OpenAI planner: `docs/providers/openai.md`
- Claude planner: `docs/providers/claude.md`
- Gemini planner: `docs/providers/gemini.md`
- OpenClaw integration: `docs/providers/openclaw.md`

## Diagnostics

```bash
cortex doctor
```

Checks include:
- proxy reachability
- planner reachability
- brain unlock state
- API key mapping
- dry-run `appendEvent -> getManifest -> execute`

## Full CLI Surface

```bash
cortex setup [--non-interactive] [--provider <name>] [--model <model>] [--brain <name>] [--api-key <key>] [--rmvm-endpoint <grpc-url>]
cortex up [--provider <name>] [--brain <name>] [--proxy-addr <host:port>] [--rmvm-endpoint <grpc-url>] [--rmvm-port <port>]
cortex stop [--all|--proxy-only|--rmvm-only] [--force]
cortex status [--json] [--verbose] [--copy]
cortex logs [--service proxy|rmvm|all] [--tail <n>] [--follow]

cortex provider list [--json]
cortex provider use <name> [--model <model>] [--restart auto|never]
cortex provider set-model <model> [--provider <name>] [--restart auto|never]

cortex brain create <name> [--tenant <id>] [--passphrase-env <ENV>]
cortex brain current [--json]
cortex brain use <brain-id-or-name>
cortex brain list [--json]
cortex brain export <brain-id-or-name> --out <file.cbrain>
cortex brain import --in <file.cbrain> [--name <alias>] [--verify-only]
cortex brain branch <brain-id-or-name> --new <branch-name>
cortex brain merge --source <branch> --target <branch> [--strategy ours|theirs|manual] [--brain <id>]
cortex brain forget --subject <subject> --predicate <predicate> [--scope <scope>] [--reason <text>] [--brain <id>]
cortex brain attach --agent <id> --model <id> --read <csv> --write <csv> --sinks <csv> [--ttl <duration>] [--brain <id>]
cortex brain detach --agent <id> [--model <id>] [--brain <id>]
cortex brain audit [--since <iso>] [--until <iso>] [--subject <subject>] [--json] [--brain <id>]

cortex auth map-key --api-key <key> --tenant <tenant> --brain <brain-id> [--subject <subject>]
cortex doctor [--proxy-base-url <url>] [--endpoint <grpc-url>] [--brain <id>] [--planner-mode openai|byo|fallback]
cortex open [--print-only] [--url]
```

## Environment Variables

- `CORTEX_BRAIN`
- `CORTEX_ENDPOINT`
- `CORTEX_BRAIN_SECRET`
- `CORTEX_PLANNER_MODE`
- `CORTEX_PLANNER_BASE_URL`
- `CORTEX_PLANNER_MODEL`
- `CORTEX_PLANNER_API_KEY`
- `OPENAI_BASE_URL`
- `OPENAI_API_KEY`
- `RMVM_SERVER_ADDR`
- `RMVM_MAX_DECODING_BYTES`
- `RMVM_MAX_ENCODING_BYTES`
- `RMVM_REQUEST_TIMEOUT_SECS`

## Developer Build

```bash
cargo test --locked
cargo run -p cortex-app -- setup --non-interactive --provider ollama --brain demo --api-key ctx_demo_key
cargo run -p cortex-app -- up
```

## Compatibility

Pinned RMVM core contract:
- `core_version.lock`
- `docs/compatibility_matrix.md`

## Operations and Security

- `docs/operations/server_config.md`
- `docs/operations/baseline_update_policy.md`
- `docs/security/controls.md`
- `docs/security_model.md`
- `docs/proxy_mode.md`
- `docs/portable_brain_format.md`
- `docs/forget_ux.md`
- `docs/use_cases.md`

## Migration

If you used `portable-brain-proxy` inside `cortex-rmvm`:
- `docs/migration_from_cortex_rmvm.md`
