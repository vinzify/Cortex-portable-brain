# Cortex Browser Extension (MVP)

This extension connects browser chat surfaces to local Cortex memory.

## Load unpacked

1. Open `chrome://extensions` (or `edge://extensions`).
2. Enable **Developer mode**.
3. Click **Load unpacked**.
4. Select this folder: `extension/chrome`.

## Configure

Set these in extension popup/options:
- Base URL: `http://127.0.0.1:8080/v1`
- API key: value from `cortex status --copy`
- Model: `cortex-brain`

## Usage

- Use popup **Quick Ask** to query memory.
- Use in-page **Cortex** panel on supported sites to capture/query.
- Highlight text and use context menu **Remember Selection with Cortex**.

## Supported web surfaces (current)

- `chatgpt.com`
- `chat.openai.com`
- `claude.ai`
- `gemini.google.com`
