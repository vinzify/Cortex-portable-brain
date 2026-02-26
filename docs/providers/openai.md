# OpenAI Planner Recipe

## Prerequisites
- `cortex` binary installed
- Valid OpenAI API key

## Environment
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://api.openai.com/v1
export CORTEX_PLANNER_API_KEY=<your-openai-key>
export CORTEX_PLANNER_MODEL=gpt-4o-mini
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
```

## Smoke Test
```bash
cortex setup --non-interactive --provider openai --brain personal --api-key ctx_demo_key
cortex up
```

In another shell:
```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer ctx_demo_key" \
  -H "Content-Type: application/json" \
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I prefer tea"}]}'
```

## Common Errors
- `STALL`: upstream handles are not ready (`X-Cortex-Stall-*` headers explain what is pending). Retry when availability changes.
- `REJECTED`: plan or policy validation failed. Inspect `error.code` and `X-Cortex-Error-Code`.
