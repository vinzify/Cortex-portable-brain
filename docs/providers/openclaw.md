# OpenClaw Integration Recipe

## Prerequisites
- `cortex` binary installed
- OpenClaw installed
- RMVM gRPC server running on `grpc://127.0.0.1:50051`

## Environment
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://api.openai.com/v1
export CORTEX_PLANNER_API_KEY=<planner-key>
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
export OPENAI_API_KEY=ctx_demo_key
```

## OpenClaw Provider Snippet
Add this provider in your OpenClaw config:
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

## Smoke Test
```bash
cortex brain create personal
cortex auth map-key --api-key ctx_demo_key --tenant local --brain personal
cortex proxy serve --brain personal --endpoint grpc://127.0.0.1:50051
```

Then set OpenClaw to provider `cortex`, model `cortex-brain`, and send one user message.

## Common Errors
- `STALL`: proxy returns `503` with stall headers; OpenClaw should retry/backoff.
- `REJECTED`: proxy returns `400`; inspect response `error.code` and `X-Cortex-Error-Code`.
