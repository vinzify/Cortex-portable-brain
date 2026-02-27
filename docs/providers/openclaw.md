# OpenClaw Integration

OpenClaw can use Cortex as an OpenAI-compatible provider.

## 1) Start Cortex

```bash
cortex up
```

## 2) Configure OpenClaw Provider

Use:

- Base URL: `http://127.0.0.1:8080/v1`
- API key: your `ctx_...` key
- Model id: `cortex-brain`

Example provider snippet:

```json
{
  "models": {
    "mode": "merge",
    "providers": {
      "cortex": {
        "baseUrl": "http://127.0.0.1:8080/v1",
        "apiKey": "ctx_demo_key",
        "api": "openai-completions",
        "models": [
          {
            "id": "cortex-brain",
            "name": "Cortex Brain Proxy",
            "contextWindow": 131072,
            "maxTokens": 8192
          }
        ]
      }
    }
  }
}
```

## 3) Pick Planner Provider

Example:

```bash
cortex provider use ollama
cortex provider set-model qwen3.5
```

or

```bash
cortex provider use openai
```

If provider is `openai`/`claude`/`gemini`, configure planner API key.

## 4) Verify

Send one message through OpenClaw with provider `cortex`, model `cortex-brain`.

## Common Errors

- `STALL`: proxy returns `503` with stall headers
- `REJECTED`: proxy returns `400` with deterministic error code
- `API key is not mapped`: map `ctx_...` key with `cortex auth map-key`
