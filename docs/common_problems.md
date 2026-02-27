# Common Problems

## `cortex` command not found

Close terminal and open a new one after install.

If still missing on Windows, run directly:

```powershell
& "$env:LOCALAPPDATA\Programs\cortex\cortex.exe" --help
```

## `openai planner mode requires CORTEX_PLANNER_API_KEY or OPENAI_API_KEY`

Cause: provider is `openai` and planner key is missing.

Fix (PowerShell current terminal):

```powershell
$env:CORTEX_PLANNER_API_KEY="sk-..."
cortex stop --all
cortex up
```

Persistent on Windows:

```powershell
setx CORTEX_PLANNER_API_KEY "sk-..."
```

Open a new terminal after `setx`.

## `API key is not mapped`

Cause: proxy key (`ctx_...`) is not mapped to a brain.

Fix:

```bash
cortex brain current
cortex auth map-key --api-key <ctx_key> --tenant local --brain <brain_id> --subject user:local
```

## I get `STALL`

Meaning: required handles are not ready (offline/archival pending).

Run:

```bash
cortex doctor
cortex logs --service all --tail 200
```

Retry when handles are available.

## I get `REJECTED`

Meaning: RMVM validation/safety gate blocked execution.

Run:

```bash
cortex doctor
cortex logs --service proxy --tail 200
```

Check response `error.code` and `X-Cortex-Error-Code`.

## Browser extension says Cortex is not reachable

Checklist:

- `cortex up` is running
- `cortex status --verbose` shows healthy proxy
- extension Base URL is `http://127.0.0.1:8080/v1`
- extension API key exactly matches your `ctx_...` key
- if using non-default port, update extension Base URL

## Port 8080 is in use

Start proxy on another port:

```bash
cortex up --proxy-addr 127.0.0.1:8081
```

Then use Base URL:

```text
http://127.0.0.1:8081/v1
```

## `ollama serve` says port 11434 is already in use

Usually Ollama is already running, which is fine.

Check:

```bash
ollama list
```

## I forgot my brain passphrase

Brains are encrypted. Without the secret, encrypted state cannot be decrypted.

You can:

- create a new brain
- import from existing `.cbrain` export

You cannot recover encrypted state without the secret.
