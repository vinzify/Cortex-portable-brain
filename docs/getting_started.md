# Getting Started

This is the fastest path to a working setup.

## What Is A Brain?

A brain is your local encrypted memory workspace.
It stores memory state independent of model provider.
You can export/import it between machines.

## What Is The Base URL?

The Base URL is the endpoint your AI app calls.
With Cortex, use:

```bash
http://127.0.0.1:8080/v1
```

Your AI app talks to Cortex, and Cortex handles RMVM + planner routing.

## 1) Install

macOS/Linux:
```bash
curl -fsSL https://raw.githubusercontent.com/vinzify/Cortex-portable-brain/main/install/install.sh | sh
```

Windows PowerShell:
```powershell
irm https://github.com/vinzify/Cortex-portable-brain/raw/main/install/install.ps1 | iex
```

If your network blocks raw script fetch:
```powershell
git clone https://github.com/vinzify/Cortex-portable-brain.git
powershell -NoProfile -ExecutionPolicy Bypass -File .\Cortex-portable-brain\install\install.ps1
```

## 2) Setup

```bash
cortex setup
```

Typical output:

```bash
Setup complete:
  brain=personal-xxxx
  provider=openai model=gpt-4o-mini
  proxy=http://127.0.0.1:8080
  rmvm=managed (grpc://127.0.0.1:50051)
Next: cortex up
```

## 3) Start

```bash
cortex up
```

Typical output:

```bash
Copy/paste client settings:
Base URL: http://127.0.0.1:8080/v1
API Key: ctx_...
Provider: OpenAI (gpt-4o-mini)
Brain: personal
```

## 4) Paste Into Your AI App

Use these values in:
- OpenWebUI
- OpenClaw
- Any OpenAI-compatible client

Set:
- Base URL: `http://127.0.0.1:8080/v1`
- API key: value shown by `cortex up` or `cortex status --copy`

## 5) Verify

```bash
curl -sS -i http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <your-cortex-proxy-api-key>" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}"
```

Expected result: `HTTP/1.1 200 OK` and a `chat.completion` JSON response.

## Switch Provider Later

```bash
cortex provider use claude
cortex provider set-model claude-opus-4-6
```

Your AI app settings stay the same.
