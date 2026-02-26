# Claude Planner Recipe (Anthropic OpenAI Compatibility)

If you already ran `cortex setup`, switch to Claude with one command:

```bash
cortex provider use claude
```

Optional model change:

```bash
cortex provider set-model claude-opus-4-6
```

Your client Base URL and API key do not change.

## First-Time Setup (if needed)

```bash
cortex setup --non-interactive --provider claude --brain personal --api-key ctx_demo_key
cortex up
```

## Optional Environment Overrides
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://api.anthropic.com/v1/
export CORTEX_PLANNER_API_KEY=<your-anthropic-key>
export CORTEX_PLANNER_MODEL=claude-opus-4-6
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
```

## Smoke Test
```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer ctx_demo_key" \
  -H "Content-Type: application/json" \
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I prefer black coffee"}]}'
```

## Common Errors
- `STALL`: manifested handle availability is pending; proxy returns `503` with stall headers for retry logic.
- `REJECTED`: RMVM denied execution due to contract/policy checks; inspect `error.message` and `X-Cortex-Error-Code`.
