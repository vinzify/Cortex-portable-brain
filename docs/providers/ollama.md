# Ollama Planner Recipe

Use this when you want local planning with Ollama.

## Prerequisites
- `ollama` installed
- model pulled locally

Example:
```bash
ollama pull qwen3.5
ollama list
```

## Setup + start Cortex

```bash
cortex setup --provider ollama --model qwen3.5 --brain personal --api-key ctx_demo_key
cortex up
```

If `ollama serve` reports port `11434` already in use, Ollama is usually already running.

## Connect your AI app

Paste in app settings:
- Base URL: `http://127.0.0.1:8080/v1`
- API key: `ctx_demo_key`
- Model: `cortex-brain`

Do not paste these values inside chat text.

## Verify

```bash
curl -sS -i http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer ctx_demo_key" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}"
```

Expected: `HTTP 200` and `chat.completion` response.
