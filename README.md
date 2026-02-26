# Cortex Brain

Portable Brain + OpenAI-compatible proxy layer for Cortex RMVM.

This repository is intentionally separate from RMVM core to keep release cadence, onboarding, and security boundaries independent.

## What You Get
- `cortex` CLI (`brain`, `proxy`, `auth` commands)
- encrypted local brain store
- OpenAI-compatible `POST /v1/chat/completions` proxy
- planner modes: `openai`, `byo`, `fallback`

## Zero Integration (Happy Path)
```bash
cortex brain create personal
cortex proxy serve --brain personal --endpoint grpc://127.0.0.1:50051
```

Then point any OpenAI-compatible client to:
```bash
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
```

## Install (No Rust Required)

### macOS/Linux
```bash
curl -fsSL https://raw.githubusercontent.com/vinzify/Cortex-portable-brain/main/install/install.sh | sh
```

### Windows PowerShell
```powershell
irm https://raw.githubusercontent.com/vinzify/Cortex-portable-brain/main/install/install.ps1 | iex
```

### Docker
```bash
docker run --rm -p 8080:8080 ghcr.io/vinzify/cortex-portable-brain:latest
```

## Developer Build
```bash
cargo test --locked
cargo run -p cortex-app -- proxy serve --addr 127.0.0.1:8080 --endpoint grpc://127.0.0.1:50051 --planner-mode openai
```

## CLI Surface
```bash
cortex brain create <name> [--tenant <id>] [--passphrase-env <ENV>]
cortex brain use <brain-id-or-name>
cortex brain list [--json]
cortex brain export <brain-id-or-name> --out <file.cbrain>
cortex brain import --in <file.cbrain> [--name <alias>] [--verify-only]
cortex brain branch <brain-id-or-name> --new <branch-name>
cortex brain merge --source <branch> --target <branch> [--strategy ours|theirs|manual] [--brain <id>]
cortex brain forget --subject <subject> --predicate <predicate> [--scope <scope>] [--reason <text>] [--brain <id>]
cortex brain attach --agent <id> --model <id> --read <csv> --write <csv> --sinks <csv> [--ttl <duration>] [--brain <id>]
cortex proxy serve --addr 127.0.0.1:8080 --endpoint grpc://127.0.0.1:50051 --planner-mode openai
cortex auth map-key --api-key <key> --tenant <tenant> --brain <brain-id>
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

## Core Compatibility
Pinned RMVM core contract is defined in:
- `core_version.lock`
- `docs/compatibility_matrix.md`

## Security + Operations
- `docs/security/controls.md`
- `docs/operations/server_config.md`
- `docs/operations/baseline_update_policy.md`
- `docs/forget_ux.md`

## Migration
If you were using `portable-brain-proxy` in `cortex-rmvm`, use:
- `docs/migration_from_cortex_rmvm.md`
