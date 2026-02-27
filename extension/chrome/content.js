(function () {
  const PANEL_ID = "cortex-memory-panel";
  const TOGGLE_ID = "cortex-memory-toggle";

  if (document.getElementById(PANEL_ID) || document.getElementById(TOGGLE_ID)) {
    return;
  }

  function createEl(tag, props = {}, text = "") {
    const el = document.createElement(tag);
    Object.assign(el, props);
    if (text) {
      el.textContent = text;
    }
    return el;
  }

  function collectHostChatText() {
    const host = location.hostname;
    let selectors = [
      "[data-message-author-role]",
      "[data-testid='message-content']",
      "article",
      "main"
    ];
    if (host.includes("chatgpt.com") || host.includes("openai.com")) {
      selectors = ["[data-message-author-role]", "article"];
    } else if (host.includes("claude.ai")) {
      selectors = ["[data-testid='message-content']", "article", "main"];
    } else if (host.includes("gemini.google.com")) {
      selectors = ["message-content", "[data-turn-role]", "article", "main"];
    }

    const chunks = [];
    for (const selector of selectors) {
      const nodes = document.querySelectorAll(selector);
      if (!nodes.length) {
        continue;
      }
      nodes.forEach((node) => {
        const text = (node.innerText || "").trim();
        if (text && text.length > 8) {
          chunks.push(text.replace(/\s+/g, " "));
        }
      });
      if (chunks.length > 2) {
        break;
      }
    }
    if (!chunks.length) {
      const fallback = (document.querySelector("main")?.innerText || "").trim();
      if (fallback) {
        chunks.push(fallback);
      }
    }
    const combined = chunks.join("\n\n");
    return combined.slice(0, 6000);
  }

  async function sendToBackground(payload) {
    return chrome.runtime.sendMessage(payload);
  }

  const style = createEl("style");
  style.textContent = `
    #${TOGGLE_ID} {
      position: fixed;
      right: 16px;
      bottom: 16px;
      z-index: 2147483646;
      background: #1f6fff;
      color: #fff;
      border: none;
      border-radius: 999px;
      padding: 10px 14px;
      font-weight: 600;
      cursor: pointer;
      box-shadow: 0 6px 18px rgba(0,0,0,0.35);
    }
    #${PANEL_ID} {
      position: fixed;
      right: 16px;
      bottom: 68px;
      z-index: 2147483646;
      width: 320px;
      max-height: 70vh;
      overflow: auto;
      background: #0f172a;
      color: #e2e8f0;
      border: 1px solid #334155;
      border-radius: 12px;
      padding: 12px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      box-shadow: 0 12px 28px rgba(0,0,0,0.45);
      display: none;
    }
    #${PANEL_ID} h3 {
      margin: 0 0 8px 0;
      font-size: 15px;
    }
    #${PANEL_ID} .status {
      font-size: 12px;
      margin-bottom: 10px;
      color: #93c5fd;
    }
    #${PANEL_ID} textarea {
      width: 100%;
      min-height: 88px;
      box-sizing: border-box;
      border-radius: 8px;
      border: 1px solid #475569;
      background: #020617;
      color: #e2e8f0;
      padding: 8px;
      margin-bottom: 8px;
      resize: vertical;
    }
    #${PANEL_ID} .row {
      display: flex;
      gap: 8px;
      margin-bottom: 8px;
    }
    #${PANEL_ID} button {
      flex: 1;
      border: none;
      border-radius: 8px;
      padding: 8px;
      cursor: pointer;
      background: #1e293b;
      color: #e2e8f0;
      font-weight: 600;
    }
    #${PANEL_ID} button.primary {
      background: #2563eb;
      color: #fff;
    }
    #${PANEL_ID} pre {
      white-space: pre-wrap;
      word-break: break-word;
      background: #020617;
      border: 1px solid #334155;
      border-radius: 8px;
      padding: 8px;
      min-height: 70px;
      font-size: 12px;
    }
  `;
  document.documentElement.appendChild(style);

  const toggle = createEl("button", { id: TOGGLE_ID }, "Cortex");
  const panel = createEl("div", { id: PANEL_ID });
  const title = createEl("h3", {}, "Cortex Memory");
  const status = createEl("div", { className: "status" }, "Checking local Cortex...");
  const prompt = createEl("textarea", { placeholder: "Ask Cortex memory..." });
  const output = createEl("pre", {}, "");
  const row1 = createEl("div", { className: "row" });
  const rememberBtn = createEl("button", { className: "primary" }, "Remember This Chat");
  const askBtn = createEl("button", {}, "Ask Memory");
  const row2 = createEl("div", { className: "row" });
  const refreshBtn = createEl("button", {}, "Refresh Status");
  const closeBtn = createEl("button", {}, "Close");

  row1.appendChild(rememberBtn);
  row1.appendChild(askBtn);
  row2.appendChild(refreshBtn);
  row2.appendChild(closeBtn);

  panel.appendChild(title);
  panel.appendChild(status);
  panel.appendChild(prompt);
  panel.appendChild(row1);
  panel.appendChild(row2);
  panel.appendChild(output);

  document.body.appendChild(toggle);
  document.body.appendChild(panel);

  toggle.addEventListener("click", () => {
    panel.style.display = panel.style.display === "none" ? "block" : "none";
  });
  closeBtn.addEventListener("click", () => {
    panel.style.display = "none";
  });

  async function refreshStatus() {
    const result = await sendToBackground({ type: "cortex:health" });
    if (result?.ok && result.health?.ok) {
      status.textContent = "Connected to local Cortex.";
      status.style.color = "#6ee7b7";
    } else {
      status.textContent = "Cortex not reachable. Run: cortex up";
      status.style.color = "#fca5a5";
    }
  }

  rememberBtn.addEventListener("click", async () => {
    const text = collectHostChatText();
    if (!text) {
      output.textContent = "No chat text detected on this page.";
      return;
    }
    output.textContent = "Sending memory capture to Cortex...";
    const result = await sendToBackground({
      type: "cortex:remember",
      payload: {
        text,
        source: location.href
      }
    });
    if (!result?.ok) {
      output.textContent = `Error: ${result?.error || "unknown error"}`;
      return;
    }
    const sem = result.result?.cortex?.semantic_root || "<none>";
    output.textContent = `Captured to Cortex.\nsemantic_root: ${sem}`;
  });

  askBtn.addEventListener("click", async () => {
    const q = prompt.value.trim();
    if (!q) {
      output.textContent = "Enter a question first.";
      return;
    }
    output.textContent = "Querying Cortex...";
    const result = await sendToBackground({
      type: "cortex:chat",
      payload: {
        messages: [{ role: "user", content: q }]
      }
    });
    if (!result?.ok) {
      output.textContent = `Error: ${result?.error || "unknown error"}`;
      return;
    }
    const content = result.result?.choices?.[0]?.message?.content || "";
    const sem = result.result?.cortex?.semantic_root || "<none>";
    output.textContent = `${content}\n\nsemantic_root: ${sem}`;
  });

  refreshBtn.addEventListener("click", () => {
    refreshStatus().catch((error) => {
      status.textContent = `Status error: ${String(error)}`;
      status.style.color = "#fca5a5";
    });
  });

  refreshStatus().catch((error) => {
    status.textContent = `Status error: ${String(error)}`;
    status.style.color = "#fca5a5";
  });
})();
