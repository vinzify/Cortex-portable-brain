# OpenAI Planner

Use this when Cortex planner should run on OpenAI API.

## Required Credential

- OpenAI API key is required.
- ChatGPT website subscription is not an API key.

Set planner key:

```powershell
$env:CORTEX_PLANNER_API_KEY="sk-..."
```

Or:

```powershell
$env:OPENAI_API_KEY="sk-..."
```

## Switch to OpenAI

```bash
cortex provider use openai
cortex provider set-model gpt-4o-mini
```

Then restart:

```bash
cortex stop --all
cortex up
```

Your chat app settings do not change.

## Verify

```bash
curl -sS http://127.0.0.1:8080/v1/chat/completions \
  -H "Authorization: Bearer <ctx_key>" \
  -H "Content-Type: application/json" \
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I prefer tea"}]}'
```

## Common Errors

- `openai planner mode requires CORTEX_PLANNER_API_KEY or OPENAI_API_KEY`
  - planner key missing
- `API key is not mapped`
  - map `ctx_...` key with `cortex auth map-key`
