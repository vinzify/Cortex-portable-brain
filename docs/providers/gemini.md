# Gemini Planner (Google OpenAI Compatibility)

Use this when Cortex planner should run on Gemini models.

## Required Credential

- Google AI API key is required.

Set planner key:

```powershell
$env:CORTEX_PLANNER_API_KEY="<gemini_key>"
```

## Switch to Gemini

```bash
cortex provider use gemini
cortex provider set-model gemini-3-flash-preview
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
  -d '{"model":"cortex-brain","messages":[{"role":"user","content":"remember I like tea and lemon"}]}'
```

## Common Errors

- `API key is not mapped`
  - map `ctx_...` key with `cortex auth map-key`
- `STALL`
  - retry when handles are ready
- `REJECTED`
  - inspect response error code/header
