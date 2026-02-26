# Cortex Brain

Portable, encrypted memory with a local chat-completions proxy.

Use one stable local endpoint for your AI clients while switching providers and models without changing your app integration.

Cortex currently supports the OpenAI-compatible `POST /v1/chat/completions` shape.
That is the most common compatibility format used by many AI tools.
It is not a full clone of every OpenAI endpoint.

## What Is Cortex Brain? (In Plain English)

Cortex Brain is a memory layer that sits between your AI app and your model provider.
It gives your assistant durable memory without tying you to one model vendor.
Your app always calls one local OpenAI-compatible endpoint, and Cortex handles memory, execution, and provider routing.

You can think of it like this:
- your app talks to `http://127.0.0.1:8080/v1`
- Cortex reads/writes encrypted brain memory
- Cortex executes memory logic deterministically through RMVM
- Cortex returns normal chat output plus verification fields

## Why Cortex Is Different

- It is model-provider independent.
Your memory is in the brain, not inside one model account.

- It is deterministic and auditable.
Memory execution produces proof roots (`semantic_root`, `trace_root`) so you can verify what happened.

- It keeps app integration stable.
Your client keeps one Base URL and API key while you switch provider/model behind Cortex.

- It has explicit forget semantics.
Forget is deterministic suppression policy, not silent hidden deletion.

## How It Works Under The Hood

1. Your AI app sends a standard OpenAI-compatible request to Cortex.
2. Cortex appends the user event and fetches a manifest of available memory handles/selectors.
3. Cortex resolves a plan (`openai`, `byo`, or `fallback`) and validates it against the manifest.
4. RMVM executes deterministically and returns assertions, status, and proof roots.
5. Cortex returns an OpenAI-compatible response with a `cortex` metadata block.

## Why This Works For Long-Term Memory

- Memory is stored in an encrypted local brain, not in a short-lived model session.
- The endpoint your app uses stays constant even when you switch providers.
- Execution is deterministic, so behavior is stable and auditable.
- Responses include proof roots (`semantic_root`, `trace_root`) for traceability.
- Brains are export/import portable across machines.
- Forget uses deterministic suppression rules instead of silent deletion.

## Sample Example (What Actually Happens)

Write memory:
```text
"Remember that I prefer tea."
```

When this request reaches Cortex:
- Cortex appends the event to RMVM input state.
- RMVM creates/updates memory handles linked to your subject.
- The brain now has durable memory for this preference.

Later query:
```text
"What drink do I prefer?"
```

When this query reaches Cortex:
- Cortex gets the memory manifest for available handles/selectors.
- Cortex builds or receives a plan (`openai`, `byo`, or `fallback`) and validates it.
- RMVM executes the plan deterministically and returns verified assertions.
- Cortex returns a normal assistant response in chat-completions format.

Typical outcome in the response:
- Assistant returns a normal answer (for example: "You prefer tea.")
- Response also includes `cortex.status`, `cortex.semantic_root`, and `cortex.trace_root`

So you get both human-friendly output and machine-verifiable memory evidence.

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

## Switch AI Provider In 10 Seconds

Switch provider:
```bash
cortex provider use claude
```

Optional model change:
```bash
cortex provider set-model claude-opus-4-6
```

Your AI app settings do not change. Only the planner behind Cortex changes.

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
