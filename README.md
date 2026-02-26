# Cortex Brain

Portable, encrypted memory with a local chat-completions proxy.

Use one stable local endpoint for your AI clients while switching providers and models without changing your app integration.

## Compatibility

Cortex Brain works with:
- OpenAI
- Claude (via OpenAI-compatible endpoint)
- Gemini (via OpenAI-compatible endpoint)
- Ollama (local)
- OpenClaw (point OpenClaw Base URL to Cortex)
- Any tool that supports OpenAI-compatible chat completions

Scope:
- Supported: OpenAI-compatible `POST /v1/chat/completions`
- Not supported: full OpenAI API surface

## Why Cortex Is Different From Typical Memory

Most memory today:
- stores chat chunks in a vector database
- retrieves top-k similar text
- asks the model to answer from retrieved text
- can be inconsistent, hard to debug, and easier to poison

Cortex memory:
- executes a validated plan against a manifest
- uses deterministic RMVM execution instead of free-form recall
- returns verified output derived from execution
- includes proof roots (`semantic_root`, `trace_root`) for auditability

Concrete guarantees:
- no memory-based claim without evidence (verified assertions only)
- deterministic behavior (same inputs produce the same `semantic_root`)
- safe forget semantics (deterministic suppression, not silent deletion)

## Quick Start (3 Steps)

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

### 2) Run guided setup

```bash
cortex setup
```

This creates or selects your brain, stores local config, and maps your proxy API key.

### 3) Start everything

```bash
cortex up
```

`cortex up` starts:
- RMVM gRPC runtime (managed sidecar by default)
- Cortex proxy on `http://127.0.0.1:8080`

You will see a copy/paste block like:
```bash
Copy/paste client settings:
Base URL: http://127.0.0.1:8080/v1
API Key: ctx_...
Provider: OpenAI (gpt-4o-mini)
Brain: personal
```

Use it anytime:
```bash
cortex status --copy
```

### Point your existing OpenAI-compatible client to Cortex

```bash
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
export OPENAI_API_KEY=<your-cortex-proxy-api-key>
```

## Does It Work?

```bash
curl -sS -i http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <your-cortex-proxy-api-key>" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}"
```

Expected result: `HTTP/1.1 200 OK` and a `chat.completion` JSON response with a `cortex` block.

If you get `STALL` or `REJECTED`, run:
```bash
cortex doctor
```

## Trust Test (10 Seconds)

Run the same request twice:

```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <your-cortex-proxy-api-key>" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}" > resp1.json

curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <your-cortex-proxy-api-key>" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}" > resp2.json
```

Check `cortex.semantic_root` in both responses.
For the same inputs, it should match.

## Switch AI Provider In 10 Seconds

Switch from OpenAI to Claude:
```bash
cortex provider use claude
```

Optional model change:
```bash
cortex provider set-model claude-opus-4-6
```

Your AI app settings do not change.
Base URL and API key stay the same.

Supported profiles:
- `openai`
- `claude`
- `gemini`
- `ollama`
- `byo`

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

## Troubleshooting

```bash
cortex doctor
```

If needed:
```bash
cortex logs --service all --tail 200 --follow
```

More fixes:
- `docs/common_problems.md`

## Docs

Getting started:
- `docs/getting_started.md`
- `docs/common_problems.md`

Provider guides:
- OpenAI planner: `docs/providers/openai.md`
- Claude planner: `docs/providers/claude.md`
- Gemini planner: `docs/providers/gemini.md`
- OpenClaw integration: `docs/providers/openclaw.md`

Operations and security:
- `docs/operations/server_config.md`
- `docs/operations/baseline_update_policy.md`
- `docs/security/controls.md`
- `docs/security_model.md`
- `docs/proxy_mode.md`
- `docs/portable_brain_format.md`
- `docs/forget_ux.md`
- `docs/use_cases.md`

Compatibility:
- `core_version.lock`
- `docs/compatibility_matrix.md`

Migration:
- `docs/migration_from_cortex_rmvm.md`

## What Cortex Brain Includes

- `cortex` CLI (`setup`, `up`, `stop`, `status`, `logs`, `provider`, `brain`, `auth`, `doctor`)
- Encrypted local brain store with export/import
- OpenAI-compatible `POST /v1/chat/completions` proxy
- Planner modes: `openai`, `byo`, `fallback`
- Managed RMVM sidecar runtime by default

## Advanced Reference

### Full CLI Surface

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

### Environment Variables

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

### Developer Build

```bash
cargo test --locked
cargo run -p cortex-app -- setup --non-interactive --provider ollama --brain demo --api-key ctx_demo_key
cargo run -p cortex-app -- up
```
