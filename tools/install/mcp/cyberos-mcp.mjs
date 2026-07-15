#!/usr/bin/env node
// cyberos-mcp - a zero-dependency stdio MCP server that exposes the CyberOS
// ship-tasks workflow as tools, so ANY MCP-capable agent (Codex, zcode,
// Antigravity, Cursor, Claude Code, Command Code...) can trigger it with no files.
//
// Transport: newline-delimited JSON-RPC 2.0 over stdin/stdout (the MCP stdio transport).
// Tools:
//   task_install   {repo?}           - vendor the CyberOS machine into a repo (needs the payload)
//   task_gates  {repo?}           - run the machine gates (repo's own build/lint/test + coverage)
//   task_status {repo?}           - summarize the task backlog + installed version
//   ship_task   {repo?, task_id?}   - return the canonical, HITL-gated trigger for the next task
//
// HITL is preserved: ship_task never drives or accepts a task itself. It hands the
// calling agent the exact instruction to follow; the human still holds the two acceptance gates.
//
// Nothing but protocol JSON is ever written to stdout. Diagnostics go to stderr.

import { spawnSync } from "node:child_process";
import { readFileSync, existsSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const SERVER = { name: "cyberos", version: readVersion() };
const PROTOCOL = "2025-06-18";
const GATE_TIMEOUT_MS = Number(process.env.CYBEROS_MCP_TIMEOUT_MS || 30 * 60 * 1000);

function readVersion() {
  for (const p of [join(HERE, "..", "VERSION"), join(HERE, "VERSION")]) {
    try { return readFileSync(p, "utf8").trim() || "0.0.0"; } catch { /* keep looking */ }
  }
  return "0.0.0";
}

// --- repo + payload resolution -------------------------------------------------
function repoRoot(start) {
  let dir = resolve(start || process.cwd());
  for (;;) {
    if (existsSync(join(dir, ".cyberos")) || existsSync(join(dir, ".git"))) return dir;
    const up = dirname(dir);
    if (up === dir) return resolve(start || process.cwd());
    dir = up;
  }
}
// find install.sh: explicit CYBEROS_PAYLOAD, then next to this server (payload-hosted), then
// a copied installer inside the target repo. Vendored servers may not have it - that's fine.
function findInstall(repo) {
  const cands = [
    process.env.CYBEROS_PAYLOAD && join(process.env.CYBEROS_PAYLOAD, "install.sh"),
    join(HERE, "..", "install.sh"),
    join(repo, ".cyberos-install", "install.sh"),
  ].filter(Boolean);
  return cands.find((p) => existsSync(p)) || null;
}
function isFile(p) { try { return statSync(p).isFile(); } catch { return false; } }

// --- tool implementations ------------------------------------------------------
function run(cmd, args, cwd) {
  const r = spawnSync(cmd, args, { cwd, timeout: GATE_TIMEOUT_MS, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  const out = `${r.stdout || ""}${r.stderr ? `\n[stderr]\n${r.stderr}` : ""}`.trim();
  const code = r.status == null ? (r.error ? `error: ${r.error.message}` : "signal") : r.status;
  return { code, out };
}

/** Soft CyberOS update-check whenever any MCP tool touches a repo's .cyberos. */
function softUpdateCheck(repo) {
  const uc = join(repo, ".cyberos", "lib", "update-check.sh");
  if (!existsSync(uc)) return;
  try {
    spawnSync("bash", ["-c", `source '${uc.replace(/'/g, "'\\''")}'; _cyberos_update_check || true`], {
      cwd: repo,
      encoding: "utf8",
      timeout: 15_000,
      env: { ...process.env },
    });
  } catch { /* soft */ }
}

function toolInstall(a = {}) {
  const repo = repoRoot(a.repo);
  softUpdateCheck(repo);
  const init = findInstall(repo);
  if (!init) {
    return err(
      `No install.sh reachable. Run the MCP server from the payload (dist/cyberos/mcp/) or set ` +
      `CYBEROS_PAYLOAD to the payload dir. Post-install, use task_gates / task_status / ship_task - ` +
      `they need only ${repo}/.cyberos/.`,
    );
  }
  const { code, out } = run("bash", [init, repo], dirname(init));
  return text(`task_install on ${repo} (exit ${code})\n\n${out}`, code !== 0);
}

function toolGates(a = {}) {
  const repo = repoRoot(a.repo);
  softUpdateCheck(repo);
  const gates = join(repo, ".cyberos", "cuo", "gates", "run-gates.sh");
  if (!isFile(gates)) return err(`No gates at ${gates}. Run task_install first.`);
  const { code, out } = run("bash", [gates], repo);
  return text(`gates on ${repo} (exit ${code} - green is necessary, never sufficient)\n\n${out}`, code !== 0);
}

function toolStatus(a = {}) {
  const repo = repoRoot(a.repo);
  softUpdateCheck(repo);
  const ver = readIf(join(repo, ".cyberos", "VERSION")).trim() || "not installed";
  const bl = readIf(join(repo, "docs", "tasks", "BACKLOG.md"));
  if (!bl) return text(`CyberOS ${ver} @ ${repo}\nNo docs/tasks/BACKLOG.md yet - run task_install, then add tasks.`);
  const rows = bl.split("\n").filter((l) => /^\s*-\s*\[/.test(l));
  const counts = {};
  for (const l of rows) { const m = l.match(/\[([a-z_]+)\]/); if (m) counts[m[1]] = (counts[m[1]] || 0) + 1; }
  const summary = Object.entries(counts).map(([k, v]) => `  ${k}: ${v}`).join("\n") || "  (no status rows found)";
  const next = rows.find((l) => /\[ready_to_implement\]/.test(l));
  return text(`CyberOS ${ver} @ ${repo}\nbacklog: ${rows.length} rows\n${summary}\nnext eligible: ${next ? next.trim() : "(none ready_to_implement)"}`);
}

function toolShipTask(a = {}) {
  const repo = repoRoot(a.repo);
  softUpdateCheck(repo);
  const bl = readIf(join(repo, "docs", "tasks", "BACKLOG.md"));
  let next = a.task_id ? `task ${a.task_id}` : "the next eligible task";
  if (!a.task_id && bl) {
    const row = bl.split("\n").find((l) => /\[ready_to_implement\]/.test(l));
    if (row) next = row.replace(/^\s*-\s*/, "").trim();
  }
  const prompt =
    `Follow .cyberos/cuo/ship-tasks.md and drive ${next} in ` +
    `docs/tasks/BACKLOG.md through implement -> review -> test. ` +
    `HITL is required: HALT at review acceptance (reviewing -> ready_to_test) and at final ` +
    `acceptance (testing -> done) for a recorded human verdict, and never set done yourself. ` +
    `Run gates with bash .cyberos/cuo/gates/run-gates.sh. repo_root is ${repo}. ` +
    `Never push, deploy, or merge without an explicit operator instruction.`;
  return text(
    `ship_task hands you the trigger below (it does NOT auto-run - the human holds the two ` +
    `acceptance gates). Follow it now:\n\n${prompt}`,
  );
}

function readIf(p) { try { return readFileSync(p, "utf8"); } catch { return ""; } }

// --- MCP tool registry ---------------------------------------------------------
const repoArg = { repo: { type: "string", description: "Absolute path to the target repo (default: cwd, walked up to the repo root)." } };
const TOOLS = [
  { name: "task_install",   description: "Install the CyberOS machine into a repo (gate autodetect, task backlog, agent surface, BRAIN). Needs the payload reachable.", inputSchema: { type: "object", properties: { ...repoArg } }, run: toolInstall },
  { name: "task_gates",  description: "Run the CyberOS machine gates (the repo's own build/lint/test + coverage, plus caf/awh if present). Green is necessary, never sufficient.", inputSchema: { type: "object", properties: { ...repoArg } }, run: toolGates },
  { name: "task_status", description: "Summarize the task backlog (counts by status, next eligible task) and the installed CyberOS version.", inputSchema: { type: "object", properties: { ...repoArg } }, run: toolStatus },
  { name: "ship_task",   description: "Return the canonical, HITL-gated trigger to drive the next (or a named) task. Does NOT auto-run or self-accept.", inputSchema: { type: "object", properties: { ...repoArg, task_id: { type: "string", description: "Optional task id, e.g. TASK-012-slug. Omit to take the next ready_to_implement row." } } }, run: toolShipTask },
];

// --- helpers -------------------------------------------------------------------
function text(t, isError = false) { return { content: [{ type: "text", text: t }], ...(isError ? { isError: true } : {}) }; }
function err(t) { return text(t, true); }

// --- JSON-RPC dispatch ---------------------------------------------------------
function handle(msg) {
  const { id, method, params } = msg;
  const isRequest = id !== undefined && id !== null;
  try {
    switch (method) {
      case "initialize":
        return reply(id, { protocolVersion: (params && params.protocolVersion) || PROTOCOL, capabilities: { tools: {} }, serverInfo: SERVER });
      case "notifications/initialized":
      case "notifications/cancelled":
        return null; // notifications: no response
      case "ping":
        return reply(id, {});
      case "tools/list":
        return reply(id, { tools: TOOLS.map(({ name, description, inputSchema }) => ({ name, description, inputSchema })) });
      case "tools/call": {
        const t = TOOLS.find((x) => x.name === (params && params.name));
        if (!t) return reply(id, text(`unknown tool: ${params && params.name}`, true));
        return reply(id, t.run((params && params.arguments) || {}));
      }
      default:
        if (!isRequest) return null;
        return rpcError(id, -32601, `method not found: ${method}`);
    }
  } catch (e) {
    return isRequest ? rpcError(id, -32603, `internal error: ${e && e.message}`) : null;
  }
}
function reply(id, result) { return id === undefined || id === null ? null : { jsonrpc: "2.0", id, result }; }
function rpcError(id, code, message) { return { jsonrpc: "2.0", id, error: { code, message } }; }

// --- remote connector mode: `--http [port]` (TASK-IMP-076) -----------------------
// MCP streamable-HTTP transport, zero-dep: POST /mcp carries one JSON-RPC message
// (or a batch array) and gets application/json back; GET /healthz for probes.
// This is the endpoint agent UIs' "custom connector" dialogs point at (Claude:
// remote MCP server URL; Grok: MCP server URL). Serve it behind TLS + a reverse
// proxy in production - transport here, deployment/auth are the operator's
// (docs/deploy/mcp-connector.md).
if (process.argv.includes("--http")) {
  const { createServer } = await import("node:http");
  const port = Number(process.argv[process.argv.indexOf("--http") + 1]) || 8799;
  createServer((req, res) => {
    if (req.method === "GET" && req.url === "/healthz") {
      res.writeHead(200, { "content-type": "application/json" });
      return res.end(JSON.stringify({ ok: true, server: SERVER }));
    }
    if (req.method !== "POST") {
      res.writeHead(405, { "content-type": "application/json", allow: "POST" });
      return res.end(JSON.stringify({ error: "POST JSON-RPC to this endpoint (MCP streamable HTTP)" }));
    }
    let body = "";
    req.on("data", (c) => { body += c; if (body.length > 1_000_000) req.destroy(); });
    req.on("end", () => {
      let msg;
      try { msg = JSON.parse(body); } catch {
        res.writeHead(400, { "content-type": "application/json" });
        return res.end(JSON.stringify(rpcError(null, -32700, "parse error")));
      }
      const out = Array.isArray(msg) ? msg.map(handle).filter(Boolean) : handle(msg);
      if (out === null || (Array.isArray(out) && out.length === 0)) {
        res.writeHead(202); return res.end(); // notification(s): accepted, no body
      }
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify(out));
    });
  }).listen(port, () => process.stderr.write(`cyberos-mcp ${SERVER.version} ready (http :${port}/mcp-style POST, /healthz)\n`));
} else {
  // --- stdio loop (newline-delimited JSON) ------------------------------------
  let buf = "";
  process.stdin.setEncoding("utf8");
  process.stdin.on("data", (chunk) => {
    buf += chunk;
    let nl;
    while ((nl = buf.indexOf("\n")) >= 0) {
      const line = buf.slice(0, nl).trim();
      buf = buf.slice(nl + 1);
      if (!line) continue;
      let msg;
      try { msg = JSON.parse(line); } catch { continue; }
      const res = handle(msg);
      if (res) process.stdout.write(`${JSON.stringify(res)}\n`);
    }
  });
  process.stdin.on("end", () => process.exit(0));
  process.stderr.write(`cyberos-mcp ${SERVER.version} ready (stdio)\n`);
}
