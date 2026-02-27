const DEFAULT_SETTINGS = {
  baseUrl: "http://127.0.0.1:8080/v1",
  apiKey: "",
  model: "cortex-brain",
  autoCapture: false
};

const MENU_ID = "cortex-remember-selection";

function rootFromBase(baseUrl) {
  const trimmed = (baseUrl || "").replace(/\/+$/, "");
  if (trimmed.endsWith("/v1")) {
    return trimmed.slice(0, -3);
  }
  return trimmed;
}

async function getSettings() {
  const stored = await chrome.storage.sync.get(DEFAULT_SETTINGS);
  return { ...DEFAULT_SETTINGS, ...stored };
}

async function saveSettings(next) {
  await chrome.storage.sync.set(next);
  return getSettings();
}

async function cortexChat(messages) {
  const settings = await getSettings();
  if (!settings.apiKey) {
    throw new Error("Missing API key in extension settings.");
  }
  const endpoint = `${settings.baseUrl.replace(/\/+$/, "")}/chat/completions`;
  const payload = {
    model: settings.model || "cortex-brain",
    messages
  };
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${settings.apiKey}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify(payload)
  });
  const json = await response.json().catch(() => ({}));
  if (!response.ok) {
    const msg = json?.error?.message || `HTTP ${response.status}`;
    throw new Error(msg);
  }
  return json;
}

async function rememberText(text, source) {
  const cleaned = (text || "").trim();
  if (!cleaned) {
    throw new Error("No text to remember.");
  }
  const prompt = [
    "Remember this from an external chat surface.",
    `source=${source || "unknown"}`,
    "",
    cleaned
  ].join("\n");
  return cortexChat([{ role: "user", content: prompt }]);
}

async function checkHealth() {
  const settings = await getSettings();
  const url = `${rootFromBase(settings.baseUrl)}/healthz`;
  const response = await fetch(url);
  const text = await response.text();
  return { ok: response.ok && text.trim().toLowerCase() === "ok", status: response.status };
}

chrome.runtime.onInstalled.addListener(async () => {
  try {
    await chrome.contextMenus.remove(MENU_ID);
  } catch (_) {
    // Ignore if menu does not exist.
  }
  chrome.contextMenus.create({
    id: MENU_ID,
    title: "Remember Selection with Cortex",
    contexts: ["selection"]
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId !== MENU_ID) {
    return;
  }
  try {
    await rememberText(info.selectionText || "", tab?.url || "context_menu");
  } catch (error) {
    console.error("Cortex context menu remember failed:", error);
  }
});

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  (async () => {
    switch (message?.type) {
      case "cortex:get-settings": {
        sendResponse({ ok: true, settings: await getSettings() });
        return;
      }
      case "cortex:save-settings": {
        const settings = await saveSettings(message.payload || {});
        sendResponse({ ok: true, settings });
        return;
      }
      case "cortex:health": {
        const health = await checkHealth();
        sendResponse({ ok: true, health });
        return;
      }
      case "cortex:chat": {
        const messages = message.payload?.messages || [];
        const result = await cortexChat(messages);
        sendResponse({ ok: true, result });
        return;
      }
      case "cortex:remember": {
        const text = message.payload?.text || "";
        const source = message.payload?.source || sender?.tab?.url || "extension";
        const result = await rememberText(text, source);
        sendResponse({ ok: true, result });
        return;
      }
      default:
        sendResponse({ ok: false, error: "Unknown message type." });
    }
  })().catch((error) => {
    sendResponse({ ok: false, error: String(error?.message || error) });
  });
  return true;
});
