# Browser Extension Connector

Use the extension when you want to keep chatting in browser UIs (ChatGPT, Claude, Gemini) and still attach Cortex memory.

## Prerequisites

- `cortex up` is running locally
- Proxy settings available from `cortex status --copy`

## Load the extension

The extension source is in:
- `extension/chrome`

In Chrome/Edge:
1. Open `chrome://extensions` (or `edge://extensions`).
2. Enable **Developer mode**.
3. Click **Load unpacked**.
4. Select `extension/chrome`.

## Configure it

Open the extension popup and set:
- Base URL: `http://127.0.0.1:8080/v1`
- API key: your Cortex key
- Model: `cortex-brain`

Click **Save Settings** and **Test Connection**.

## Use it

- In supported web chats, click the floating **Cortex** button.
- Use **Remember This Chat** to capture visible chat text into Cortex.
- Use **Ask Memory** for a quick memory query.
- You can also highlight text and use context menu: **Remember Selection with Cortex**.

## What this connector does in v1

- Attaches via local extension UI.
- Calls Cortex proxy through OpenAI-compatible chat completions.
- Returns normal model output plus Cortex verification roots where available.

## Scope and limits

- This is a connector MVP for browser chat surfaces.
- DOM extraction depends on host page structure and can vary by provider updates.
- For production-style integration, OpenAI-compatible API clients remain the most stable path.
