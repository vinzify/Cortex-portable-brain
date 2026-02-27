const byId = (id) => document.getElementById(id);

async function send(message) {
  return chrome.runtime.sendMessage(message);
}

function setStatus(text, ok = true) {
  const node = byId("status");
  node.textContent = text;
  node.style.color = ok ? "#a7f3d0" : "#fca5a5";
}

function setOutput(text) {
  byId("output").textContent = text || "";
}

async function loadSettings() {
  const res = await send({ type: "cortex:get-settings" });
  if (!res?.ok) {
    setStatus("Failed to load settings.", false);
    return;
  }
  byId("baseUrl").value = res.settings.baseUrl || "";
  byId("apiKey").value = res.settings.apiKey || "";
  byId("model").value = res.settings.model || "cortex-brain";
}

async function saveSettings() {
  const payload = {
    baseUrl: byId("baseUrl").value.trim(),
    apiKey: byId("apiKey").value.trim(),
    model: byId("model").value.trim() || "cortex-brain"
  };
  const res = await send({ type: "cortex:save-settings", payload });
  if (!res?.ok) {
    setStatus(`Save failed: ${res?.error || "unknown error"}`, false);
    return;
  }
  setStatus("Settings saved.");
}

async function checkHealth() {
  const res = await send({ type: "cortex:health" });
  if (!res?.ok) {
    setStatus(`Health check failed: ${res?.error || "unknown error"}`, false);
    return;
  }
  if (res.health?.ok) {
    setStatus(`Connected (HTTP ${res.health.status}).`);
  } else {
    setStatus("Cortex not reachable. Run `cortex up`.", false);
  }
}

async function askCortex() {
  const prompt = byId("prompt").value.trim();
  if (!prompt) {
    setOutput("Enter a prompt first.");
    return;
  }
  setOutput("Sending...");
  const res = await send({
    type: "cortex:chat",
    payload: {
      messages: [{ role: "user", content: prompt }]
    }
  });
  if (!res?.ok) {
    setOutput(`Error: ${res?.error || "unknown error"}`);
    return;
  }
  const content = res.result?.choices?.[0]?.message?.content || "";
  const sem = res.result?.cortex?.semantic_root || "<none>";
  setOutput(`${content}\n\nsemantic_root: ${sem}`);
}

byId("saveBtn").addEventListener("click", () => {
  saveSettings().catch((error) => setStatus(String(error), false));
});
byId("healthBtn").addEventListener("click", () => {
  checkHealth().catch((error) => setStatus(String(error), false));
});
byId("askBtn").addEventListener("click", () => {
  askCortex().catch((error) => setOutput(String(error)));
});

loadSettings()
  .then(() => checkHealth())
  .catch((error) => setStatus(String(error), false));
