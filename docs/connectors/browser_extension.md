# Browser Extension Connector

Use this when you want to keep chatting in ChatGPT/Claude/Gemini web UI and still use Cortex memory.

## What It Does

- adds a Cortex popup
- adds in-page Cortex panel on supported chat sites
- sends memory requests to local Cortex (`/v1/chat/completions`)

## Prerequisites

- `cortex up` is running
- you have a proxy API key (`ctx_...`) from `cortex status --copy`
- planner provider is configured (`cortex setup`)

Planner note:

- if provider is `openai`, you still need OpenAI API key for planner
- ChatGPT web subscription alone does not satisfy planner API-key requirement
- if you want no cloud key, use provider `ollama`

No-cloud setup example:

```bash
cortex provider use ollama
cortex provider set-model qwen3.5
cortex stop --all
cortex up
```

## Load Extension (Chrome/Edge)

1. Open `chrome://extensions` or `edge://extensions`
2. Enable **Developer mode**
3. Click **Load unpacked**
4. Select folder: `extension/chrome`

If you installed binaries only, clone this repo first to access `extension/chrome`.

## Configure Extension

Open extension popup and set:

- Base URL: `http://127.0.0.1:8080/v1`
- API key: your `ctx_...` key
- Model: `cortex-brain`

Then click:

- **Save Settings**
- **Test Connection**

## Use It

- `Ask Cortex`: query memory quickly
- `Remember This Chat`: capture visible page content to memory
- `Remember Selection with Cortex`: highlight text and use context menu

## If It Fails

- `Error: API key is not mapped`
  - run `cortex brain current`
  - run `cortex auth map-key --api-key <ctx_key> --tenant local --brain <brain_id> --subject user:local`

- `Error: openai planner mode requires CORTEX_PLANNER_API_KEY or OPENAI_API_KEY`
  - set planner API key and restart Cortex

See also:

- `docs/common_problems.md`
