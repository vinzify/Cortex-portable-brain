# OpenAI Planner Recipe

If you already ran `cortex setup`, switch to OpenAI with one command:

```bash
cortex provider use openai
```

Optional model change:

```bash
cortex provider set-model gpt-4o-mini
```

Your client Base URL and API key do not change.

## First-Time Setup (if needed)

```bash
cortex setup --non-interactive --provider openai --brain personal --api-key ctx_demo_key
cortex up
```

## Optional Environment Overrides
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://api.openai.com/v1
export CORTEX_PLANNER_API_KEY=<your-openai-key>
export CORTEX_PLANNER_MODEL=gpt-4o-mini
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
```

## Smoke Test
```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer ctx_demo_key" \
  -H "Content-Type: application/json" \
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I prefer tea"}]}'
```

## Common Errors
- `STALL`: upstream handles are not ready (`X-Cortex-Stall-*` headers explain what is pending). Retry when availability changes.
- `REJECTED`: plan or policy validation failed. Inspect `error.code` and `X-Cortex-Error-Code`.
