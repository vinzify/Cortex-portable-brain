# Gemini Planner Recipe (Google OpenAI Compatibility)

## Prerequisites
- `cortex` binary installed
- RMVM gRPC server running on `grpc://127.0.0.1:50051`
- Valid Gemini API key

## Environment
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://generativelanguage.googleapis.com/v1beta/openai/
export CORTEX_PLANNER_API_KEY=<your-gemini-key>
export CORTEX_PLANNER_MODEL=gemini-3-flash-preview
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
```

## Smoke Test
```bash
cortex brain create personal
cortex auth map-key --api-key ctx_demo_key --tenant local --brain personal
OPENAI_API_KEY=ctx_demo_key cortex proxy serve --brain personal --endpoint grpc://127.0.0.1:50051
```

In another shell:
```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer ctx_demo_key" \
  -H "Content-Type: application/json" \
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I like tea and lemon"}]}'
```

## Common Errors
- `STALL`: RMVM cannot complete because required handles are offline/archival pending; retry when available.
- `REJECTED`: request or generated plan violated RMVM constraints; check error code header and response JSON.
