"use strict";

// Tauri 2 exposes the API on window.__TAURI__ when `app.withGlobalTauri` is true. `core.invoke` is the
// 2.x path; fall back to the top-level invoke for older 2.x point releases.
function invoke(cmd, args) {
  const t = window.__TAURI__;
  if (!t) return Promise.reject(new Error("Tauri API not available (open this through the desktop app, not a browser)"));
  const fn = (t.core && t.core.invoke) || t.invoke;
  return fn(cmd, args);
}

const $ = (id) => document.getElementById(id);
const gatewayUrl = () => $("cfg-gateway").value.replace(/\/+$/, "");
const mcpUrl = () => $("cfg-mcp").value.replace(/\/+$/, "");
const tenant = () => $("cfg-tenant").value.trim();
const alias = () => $("cfg-alias").value.trim();

// ---- view toggle ----------------------------------------------------------
function show(view) {
  $("view-chat").classList.toggle("active", view === "chat");
  $("view-tools").classList.toggle("active", view === "tools");
  $("nav-chat").classList.toggle("active", view === "chat");
  $("nav-tools").classList.toggle("active", view === "tools");
  if (view === "tools" && !toolsLoaded) refreshTools();
}
$("nav-chat").addEventListener("click", () => show("chat"));
$("nav-tools").addEventListener("click", () => show("tools"));

// ---- chat -----------------------------------------------------------------
const log = $("log");
const form = $("chat-form");
const input = $("input");
const sendBtn = $("send");
const messages = [];

function addMessage(role, content, meta) {
  const el = document.createElement("div");
  el.className = "msg " + role;
  if (role !== "system") {
    const who = document.createElement("div");
    who.className = "who";
    who.textContent = role;
    el.appendChild(who);
  }
  const body = document.createElement("div");
  body.textContent = content;
  el.appendChild(body);
  if (meta) {
    const m = document.createElement("div");
    m.className = "meta";
    m.textContent = meta;
    el.appendChild(m);
  }
  log.appendChild(el);
  log.scrollTop = log.scrollHeight;
  return el;
}

async function sendChat(text) {
  messages.push({ role: "user", content: text });
  addMessage("user", text);
  sendBtn.disabled = true;
  const pending = addMessage("assistant", "...", null);
  try {
    const data = await invoke("chat", { gateway: gatewayUrl(), tenant: tenant(), alias: alias(), messages: messages });
    const content = data && data.content ? data.content : "(empty)";
    messages.push({ role: "assistant", content: content });
    pending.querySelector(".who").textContent = "assistant";
    pending.querySelector("div:last-child").textContent = content;
    const meta = [];
    if (data.model) meta.push(data.model);
    if (typeof data.prompt_tokens === "number") meta.push(data.prompt_tokens + " in / " + data.completion_tokens + " out tokens");
    if (data.finish_reason) meta.push(String(data.finish_reason).toLowerCase());
    if (meta.length) {
      const m = document.createElement("div");
      m.className = "meta";
      m.textContent = meta.join(" - ");
      pending.appendChild(m);
    }
  } catch (e) {
    pending.className = "msg system";
    pending.querySelector("div:last-child").textContent = "provider call failed: " + (e && e.message ? e.message : e);
  } finally {
    sendBtn.disabled = false;
    input.focus();
  }
}

form.addEventListener("submit", (e) => {
  e.preventDefault();
  const text = input.value.trim();
  if (!text) return;
  input.value = "";
  sendChat(text);
});
input.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); form.requestSubmit(); }
});

// ---- tools ----------------------------------------------------------------
let toolsLoaded = false;
let selected = null;

async function refreshTools() {
  const listEl = $("tools-list");
  listEl.innerHTML = '<div class="tool"><div class="muted">loading...</div></div>';
  try {
    const tools = await invoke("list_tools", { mcp: mcpUrl() });
    toolsLoaded = true;
    if (!tools.length) {
      listEl.innerHTML = '<div class="tool"><div class="muted">No tools registered yet. A module registers its tools via POST /v1/mcp/register (enable with MCP_DEV_REGISTRATION=1); they appear here once it does.</div></div>';
      return;
    }
    listEl.innerHTML = "";
    tools.forEach((tool) => {
      const el = document.createElement("div");
      el.className = "tool";
      const name = document.createElement("div");
      name.className = "name";
      name.textContent = tool.name;
      el.appendChild(name);
      if (tool.description) {
        const d = document.createElement("div");
        d.className = "desc";
        d.textContent = tool.description;
        el.appendChild(d);
      }
      const ann = tool.annotations || {};
      if (ann.readOnlyHint) { const b = document.createElement("span"); b.className = "badge ro"; b.textContent = "read-only"; el.appendChild(b); }
      if (ann.destructiveHint) { const b = document.createElement("span"); b.className = "badge destr"; b.textContent = "destructive"; el.appendChild(b); }
      el.addEventListener("click", () => {
        document.querySelectorAll(".tool.selected").forEach((n) => n.classList.remove("selected"));
        el.classList.add("selected");
        selectTool(tool);
      });
      listEl.appendChild(el);
    });
  } catch (e) {
    toolsLoaded = false;
    listEl.innerHTML = '<div class="tool"><div class="muted">could not list tools: ' + (e && e.message ? e.message : e) + ' (is the mcp-gateway running on ' + mcpUrl() + '?)</div></div>';
  }
}

function selectTool(tool) {
  selected = tool;
  const d = $("tool-detail");
  d.innerHTML = "";

  const h = document.createElement("h2");
  h.textContent = tool.name;
  d.appendChild(h);

  if (tool.description) {
    const desc = document.createElement("div");
    desc.className = "muted";
    desc.textContent = tool.description;
    d.appendChild(desc);
  }

  const schemaLabel = document.createElement("div");
  schemaLabel.className = "muted";
  schemaLabel.textContent = "Input schema";
  d.appendChild(schemaLabel);
  const schema = document.createElement("pre");
  schema.textContent = JSON.stringify(tool.inputSchema || {}, null, 2);
  d.appendChild(schema);

  const argsLabel = document.createElement("div");
  argsLabel.className = "muted";
  argsLabel.textContent = "Arguments (JSON)";
  d.appendChild(argsLabel);
  const args = document.createElement("textarea");
  args.id = "tool-args";
  args.value = "{}";
  d.appendChild(args);

  const runBtn = document.createElement("button");
  runBtn.className = "run";
  runBtn.textContent = "Run";
  d.appendChild(runBtn);

  const result = document.createElement("div");
  result.className = "result muted";
  d.appendChild(result);

  runBtn.addEventListener("click", async () => {
    let parsed;
    try { parsed = JSON.parse(args.value || "{}"); }
    catch (e) { result.textContent = "arguments must be valid JSON: " + e.message; return; }
    runBtn.disabled = true;
    result.textContent = "running...";
    try {
      const r = await invoke("call_tool", { mcp: mcpUrl(), name: tool.name, arguments: parsed });
      const blocks = (r && r.content) || [];
      const text = blocks.filter((b) => b.type === "text").map((b) => b.text).join("\n");
      result.textContent = text || JSON.stringify(r, null, 2);
    } catch (e) {
      result.textContent = "call failed: " + (e && e.message ? e.message : e);
    } finally {
      runBtn.disabled = false;
    }
  });
}

$("refresh-tools").addEventListener("click", refreshTools);

// ---- token + health -------------------------------------------------------
$("save-token").addEventListener("click", async () => {
  const t = $("cfg-token").value.trim();
  if (!t) return;
  try { await invoke("save_token", { token: t }); $("cfg-token").value = ""; addMessage("system", "Token saved to the OS keychain."); }
  catch (e) { addMessage("system", "Could not save token: " + (e && e.message ? e.message : e)); }
});
$("clear-token").addEventListener("click", async () => {
  try { await invoke("clear_token", {}); addMessage("system", "Token cleared from the keychain."); }
  catch (e) { addMessage("system", "Could not clear token: " + (e && e.message ? e.message : e)); }
});

async function refreshHealth() {
  try {
    const ok = await invoke("health", { gateway: gatewayUrl() });
    $("health-dot").className = "dot " + (ok ? "ok" : "bad");
    $("health-text").textContent = ok ? "gateway reachable" : "gateway unreachable";
  } catch (e) {
    $("health-dot").className = "dot bad";
    $("health-text").textContent = "gateway unreachable";
  }
}

refreshHealth();
setInterval(refreshHealth, 5000);
input.focus();
