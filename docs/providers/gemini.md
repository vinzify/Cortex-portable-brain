# Gemini Planner Recipe (Google OpenAI Compatibility)

If you already ran `cortex setup`, switch to Gemini with one command:

```bash
cortex provider use gemini
```

Optional model change:

```bash
cortex provider set-model gemini-3-flash-preview
```

Your client Base URL and API key do not change.

## First-Time Setup (if needed)

```bash
cortex setup --non-interactive --provider gemini --brain personal --api-key ctx_demo_key
cortex up
```

## Optional Environment Overrides
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://generativelanguage.googleapis.com/v1beta/openai/
export CORTEX_PLANNER_API_KEY=<your-gemini-key>
export CORTEX_PLANNER_MODEL=gemini-3-flash-preview
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
```

## Smoke Test
```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer ctx_demo_key" \
  -H "Content-Type: application/json" \
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I like tea and lemon"}]}'
```

## Common Errors
- `STALL`: RMVM cannot complete because required handles are offline/archival pending; retry when available.
- `REJECTED`: request or generated plan violated RMVM constraints; check error code header and response JSON.
