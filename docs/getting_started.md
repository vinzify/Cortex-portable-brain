# Getting Started

This guide is the fastest way to a working Cortex setup.

## What You Are Setting Up

- Cortex proxy: local endpoint your app calls (`http://127.0.0.1:8080/v1`)
- Brain store: encrypted local memory state
- Planner provider: model backend Cortex uses for planning

## Important Credential Split

- Cortex proxy key (`ctx_...`): used by your app/extension to call local Cortex
- Planner key (provider API key): used by Cortex internally

Provider requirements:

- `openai`: OpenAI API key required
- `claude`: Anthropic API key required
- `gemini`: Google AI API key required
- `ollama`: no cloud API key required

A ChatGPT website subscription is not an OpenAI API key.

## 1) Install

macOS/Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/vinzify/Cortex-portable-brain/main/install/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://github.com/vinzify/Cortex-portable-brain/raw/main/install/install.ps1 | iex
```

If script fetch is blocked:

```powershell
git clone https://github.com/vinzify/Cortex-portable-brain.git
powershell -NoProfile -ExecutionPolicy Bypass -File .\Cortex-portable-brain\install\install.ps1
```

## 2) Setup

```bash
cortex setup
```

First-time recommendation:

- choose `ollama` if you want to test without cloud API keys
- choose `openai`/`claude`/`gemini` only if you already have provider API key

## 3) Start

```bash
cortex up
```

You will see:

```text
Base URL: http://127.0.0.1:8080/v1
API Key: ctx_...
Provider: ...
Brain: personal
```

You can reprint this anytime:

```bash
cortex status --copy
```

## 4) Connect Your Chat Surface

### Option A: Any OpenAI-compatible app

Put these in app settings:

- Base URL: `http://127.0.0.1:8080/v1`
- API key: `ctx_...`
- Model: `cortex-brain`

Do not paste them inside chat text.

### Option B: Browser chat (ChatGPT/Claude/Gemini)

Use extension from this repo:

- path: `extension/chrome`
- load unpacked in `chrome://extensions` or `edge://extensions`
- set Base URL/API key/model in popup
- if you installed binaries only, clone this repo to get `extension/chrome`

Guide:

- `docs/connectors/browser_extension.md`

If you only have ChatGPT subscription (no OpenAI API key), switch to local planner:

```bash
cortex provider use ollama
cortex provider set-model qwen3.5
cortex stop --all
cortex up
```

## 5) Verify

```bash
curl -sS -i http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <your-cortex-proxy-api-key>" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}"
```

Expected: `HTTP/1.1 200 OK` and a `chat.completion` response.

## Common First-Run Errors

### `openai planner mode requires CORTEX_PLANNER_API_KEY or OPENAI_API_KEY`

You selected OpenAI planner but did not provide API key.

PowerShell example:

```powershell
$env:CORTEX_PLANNER_API_KEY="sk-..."
cortex stop --all
cortex up
```

### `API key is not mapped`

Map your proxy key to the current brain:

```bash
cortex brain current
cortex auth map-key --api-key <ctx_key> --tenant local --brain <brain_id> --subject user:local
```

### `STALL` or `REJECTED`

```bash
cortex doctor
cortex logs --service all --tail 200 --follow
```

## Provider Switch (same app settings)

```bash
cortex provider use claude
cortex provider set-model claude-opus-4-6
```

Your app still uses same Base URL and `ctx_...` key.

## Optional UX Commands

```bash
cortex connect
cortex connect status
cortex mode set auto
cortex mode status
cortex open
```

## Uninstall

Stop services:

```bash
cortex uninstall
```

Remove all local data and binaries:

```bash
cortex uninstall --all --yes
```
