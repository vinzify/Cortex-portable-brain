# Dashboard

Use the local dashboard to confirm Cortex is running and copy the exact app settings.

## Open Dashboard

```bash
cortex open
```

URL only:

```bash
cortex open --url
```

## What You Should Check

- Proxy endpoint (`http://127.0.0.1:8080/v1` by default)
- Current `ctx_...` proxy key
- Current brain
- Planner provider and model
- RMVM endpoint and health

If health is bad, run:

```bash
cortex doctor
cortex logs --service all --tail 200 --follow
```
