# Cortex Brain

Portable Brain + OpenAI-compatible proxy layer for Cortex RMVM.

This repository is intentionally separate from RMVM core to keep release cadence, onboarding, and security boundaries independent.

## What You Get
- `cortex` CLI (`brain`, `proxy`, `auth` commands)
- encrypted local brain store
- OpenAI-compatible `POST /v1/chat/completions` proxy
- planner modes: `openai`, `byo`, `fallback`

## Why Portable
- The same brain works across OpenAI, Claude, Gemini, and local models because the interface is the proxy plus the RMVM plan contract.
- Export and import are encrypted and signed so you can move a brain between machines safely.
- Attachments and permissions let you connect multiple AIs without giving every model full access.
- Proof roots let you audit what memory was used across providers.
- Forget is deterministic suppression, not silent deletion.

## Zero Integration (Happy Path)
```bash
cortex brain create personal
cortex auth map-key --api-key ctx_demo_key --tenant local --brain personal
cortex proxy serve --brain personal --endpoint grpc://127.0.0.1:50051
```

Then point any OpenAI-compatible client to:
```bash
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
export OPENAI_API_KEY=ctx_demo_key
```

## Bring Your Brain To Another AI
Export on machine A:
```bash
cortex brain export personal --out personal.cbrain
```

Import on machine B:
```bash
cortex brain import --in personal.cbrain --name personal
cortex brain use personal
```

Switch planner from OpenAI to Claude while keeping the same brain:
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://api.anthropic.com/v1/
export CORTEX_PLANNER_API_KEY=$ANTHROPIC_API_KEY
export CORTEX_PLANNER_MODEL=claude-opus-4-6
cortex proxy serve --brain personal --addr 127.0.0.1:8080
```

Your existing OpenAI-compatible client keeps working; only `OPENAI_BASE_URL` changes to point at Cortex.

## Provider Recipes
- OpenAI planner: `docs/providers/openai.md`
- Claude planner (Anthropic OpenAI-compatible endpoint): `docs/providers/claude.md`
- Gemini planner (Google OpenAI-compatible endpoint): `docs/providers/gemini.md`
- OpenClaw integration: `docs/providers/openclaw.md`

## Doctor (One Command Diagnostics)
Run this before filing an issue:
```bash
cortex doctor
```

It verifies:
- proxy reachability (`/healthz`)
- planner reachability (mode-aware)
- brain unlock state
- API key mapping
- one dry-run `appendEvent -> getManifest -> execute` and prints `semantic_root`

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
cortex doctor [--proxy-base-url <url>] [--endpoint <grpc-url>] [--brain <id>] [--planner-mode openai|byo|fallback]
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

## Core Compatibility
Pinned RMVM core contract is defined in:
- `core_version.lock`
- `docs/compatibility_matrix.md`

## Security + Operations
- `docs/security/controls.md`
- `docs/operations/server_config.md`
- `docs/operations/baseline_update_policy.md`
- `docs/forget_ux.md`
- `docs/use_cases.md`

## Migration
If you were using `portable-brain-proxy` in `cortex-rmvm`, use:
- `docs/migration_from_cortex_rmvm.md`
