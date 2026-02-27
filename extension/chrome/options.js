const DEFAULTS = {
  baseUrl: "http://127.0.0.1:8080/v1",
  apiKey: "",
  model: "cortex-brain"
};

const byId = (id) => document.getElementById(id);

function setStatus(text, ok = true) {
  const node = byId("status");
  node.textContent = text;
  node.style.color = ok ? "#166534" : "#b91c1c";
}

async function load() {
  const settings = await chrome.storage.sync.get(DEFAULTS);
  byId("baseUrl").value = settings.baseUrl || DEFAULTS.baseUrl;
  byId("apiKey").value = settings.apiKey || "";
  byId("model").value = settings.model || DEFAULTS.model;
}

async function save() {
  await chrome.storage.sync.set({
    baseUrl: byId("baseUrl").value.trim() || DEFAULTS.baseUrl,
    apiKey: byId("apiKey").value.trim(),
    model: byId("model").value.trim() || DEFAULTS.model
  });
  setStatus("Saved.");
}

byId("saveBtn").addEventListener("click", () => {
  save().catch((error) => setStatus(String(error), false));
});

load().catch((error) => setStatus(String(error), false));
