# Dashboard

Use the local dashboard to confirm Cortex runtime health and copy client settings.

## Open dashboard

```bash
cortex open
```

Or print URL only:

```bash
cortex open --url
```

## What it shows
- Proxy base URL (`.../v1`)
- Chat completions URL
- API key saved in local config
- Current brain
- Provider + planner mode/model
- RMVM endpoint + health

The dashboard is local-only by default because Cortex binds to `127.0.0.1`.
