# Claude Planner Recipe (Anthropic OpenAI Compatibility)

## Prerequisites
- `cortex` binary installed
- Valid Anthropic API key

## Environment
```bash
export CORTEX_PLANNER_MODE=openai
export CORTEX_PLANNER_BASE_URL=https://api.anthropic.com/v1/
export CORTEX_PLANNER_API_KEY=<your-anthropic-key>
export CORTEX_PLANNER_MODEL=claude-opus-4-6
export OPENAI_BASE_URL=http://127.0.0.1:8080/v1
```

## Smoke Test
```bash
cortex setup --non-interactive --provider claude --brain personal --api-key ctx_demo_key
cortex up
```

In another shell:
```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer ctx_demo_key" \
  -H "Content-Type: application/json" \
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I prefer black coffee"}]}'
```

## Common Errors
- `STALL`: manifested handle availability is pending; proxy returns `503` with stall headers for retry logic.
- `REJECTED`: RMVM denied execution due to contract/policy checks; inspect `error.message` and `X-Cortex-Error-Code`.
