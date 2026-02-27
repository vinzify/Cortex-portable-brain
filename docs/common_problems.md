# Common Problems

## I get `STALL`

`STALL` means required memory handles are not ready yet.

Run:
```bash
cortex doctor
cortex logs --service all --tail 200
```

Then retry the request.

## I get `REJECTED`

`REJECTED` means RMVM safety/validation gates blocked execution.
This is expected behavior when plan or data constraints are violated.

Run:
```bash
cortex doctor
cortex logs --service proxy --tail 200
```

Inspect error code/message in response and headers.

## Port 8080 is already in use

Start on another proxy address:
```bash
cortex up --proxy-addr 127.0.0.1:8081
```

Then use:
```bash
OPENAI_BASE_URL=http://127.0.0.1:8081/v1
```

## `ollama serve` says port 11434 is already in use

This usually means Ollama is already running, which is fine.

Check:
```bash
ollama list
```

Then keep using Cortex with:
```bash
cortex status --verbose
```

If provider is `ollama`, ensure the configured planner model exists in `ollama list`.

## I forgot my passphrase

Brains are encrypted. If the encryption secret is lost, existing encrypted brain state cannot be decrypted.

What you can do:
- create a fresh brain with `cortex setup`
- import from a previous `.cbrain` export if you have one

What you cannot do:
- recover encrypted state without the secret

## Browser extension says Cortex is not reachable

Checklist:
- ensure `cortex up` is running
- check health with `cortex status --verbose`
- confirm extension Base URL is `http://127.0.0.1:8080/v1`
- confirm extension API key matches `cortex status --copy`
- if you changed proxy port, update extension Base URL accordingly
