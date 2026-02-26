# Gemini Planner Recipe (Google OpenAI Compatibility)

## Prerequisites
- `cortex` binary installed
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
cortex setup --non-interactive --provider gemini --brain personal --api-key ctx_demo_key
cortex up
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
