# Ollama Planner (Local, No Cloud API Key)

Use this for the easiest local setup.

## Prerequisites

- Ollama installed
- target model pulled locally

Example:

```bash
ollama pull qwen3.5
ollama list
```

## Switch to Ollama

```bash
cortex provider use ollama
cortex provider set-model qwen3.5
```

Then restart:

```bash
cortex stop --all
cortex up
```

No cloud planner API key is required in this mode.

## Connect Your App

In app settings:

- Base URL: `http://127.0.0.1:8080/v1`
- API key: your `ctx_...` key
- Model: `cortex-brain`

Do not paste these values into chat text.

## Verify

```bash
curl -sS -i http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <ctx_key>" \
  -H "Content-Type: application/json" \
  -d "{\"model\":\"cortex-brain\",\"messages\":[{\"role\":\"user\",\"content\":\"remember I prefer tea\"}]}"
```

Expected: `HTTP 200` and `chat.completion` response.

## Common Errors

- `Error: API key is not mapped`
  - run `cortex brain current` and `cortex auth map-key ...`
- `ollama serve` port `11434` already in use
  - Ollama is usually already running; verify with `ollama list`
