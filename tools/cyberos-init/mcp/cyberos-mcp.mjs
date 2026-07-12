#!/usr/bin/env node
// cyberos-mcp - a zero-dependency stdio MCP server that exposes the CyberOS
// ship-feature-requests workflow as tools, so ANY MCP-capable agent (Codex, zcode,
// Antigravity, Cursor, Claude Code, Command Code...) can trigger it with no files.
//
// Transport: newline-delimited JSON-RPC 2.0 over stdin/stdout (the MCP stdio transport).
// Tools:
//   fr_init   {repo?}           - vendor the CyberOS machine into a repo (needs the payload)
//   fr_gates  {repo?}           - run the machine gates (repo's own build/lint/test + coverage)
//   fr_status {repo?}           - summarize the FR backlog + installed version
//   ship_fr   {repo?, fr_id?}   - return the canonical, HITL-gated trigger for the next FR
//
// HITL is preserved: ship_fr never drives or accepts a feature-request itself. It hands the
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
// find init.sh: explicit CYBEROS_PAYLOAD, then next to this server (payload-hosted), then
// a copied installer inside the target repo. Vendored servers may not have it - that's fine.
function findInit(repo) {
  const cands = [
    process.env.CYBEROS_PAYLOAD && join(process.env.CYBEROS_PAYLOAD, "init.sh"),
    join(HERE, "..", "init.sh"),
    join(repo, ".cyberos-init", "init.sh"),
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

function toolFrInit(a = {}) {
  const repo = repoRoot(a.repo);
  const init = findInit(repo);
  if (!init) {
    return err(
      `No init.sh reachable. Run the MCP server from the payload (dist/cyberos/mcp/) or set ` +
      `CYBEROS_PAYLOAD to the payload dir. Post-install, use fr_gates / fr_status / ship_fr - ` +
      `they need only ${repo}/.cyberos/.`,
    );
  }
  const { code, out } = run("bash", [init, repo], dirname(init));
  return text(`fr_init on ${repo} (exit ${code})\n\n${out}`, code !== 0);
}

function toolFrGates(a = {}) {
  const repo = repoRoot(a.repo);
  const gates = join(repo, ".cyberos", "cuo", "gates", "run-gates.sh");
  if (!isFile(gates)) return err(`No gates at ${gates}. Run fr_init first.`);
  const { code, out } = run("bash", [gates], repo);
  return text(`gates on ${repo} (exit ${code} - green is necessary, never sufficient)\n\n${out}`, code !== 0);
}

function toolFrStatus(a = {}) {
  const repo = repoRoot(a.repo);
  const ver = readIf(join(repo, ".cyberos", "VERSION")).trim() || "not installed";
  const bl = readIf(join(repo, "docs", "feature-requests", "BACKLOG.md"));
  if (!bl) return text(`CyberOS ${ver} @ ${repo}\nNo docs/feature-requests/BACKLOG.md yet - run fr_init, then add FRs.`);
  const rows = bl.split("\n").filter((l) => /^\s*-\s*\[/.test(l));
  const counts = {};
  for (const l of rows) { const m = l.match(/\[([a-z_]+)\]/); if (m) counts[m[1]] = (counts[m[1]] || 0) + 1; }
  const summary = Object.entries(counts).map(([k, v]) => `  ${k}: ${v}`).join("\n") || "  (no status rows found)";
  const next = rows.find((l) => /\[ready_to_implement\]/.test(l));
  return text(`CyberOS ${ver} @ ${repo}\nbacklog: ${rows.length} rows\n${summary}\nnext eligible: ${next ? next.trim() : "(none ready_to_implement)"}`);
}

function toolShipFr(a = {}) {
  const repo = repoRoot(a.repo);
  const bl = readIf(join(repo, "docs", "feature-requests", "BACKLOG.md"));
  let next = a.fr_id ? `FR ${a.fr_id}` : "the next eligible FR";
  if (!a.fr_id && bl) {
    const row = bl.split("\n").find((l) => /\[ready_to_implement\]/.test(l));
    if (row) next = row.replace(/^\s*-\s*/, "").trim();
  }
  const prompt =
    `Follow .cyberos/cuo/ship-feature-requests.md and drive ${next} in ` +
    `docs/feature-requests/BACKLOG.md through implement -> review -> test. ` +
    `HITL is required: HALT at review acceptance (reviewing -> ready_to_test) and at final ` +
    `acceptance (testing -> done) for a recorded human verdict, and never set done yourself. ` +
    `Run gates with bash .cyberos/cuo/gates/run-gates.sh. repo_root is ${repo}. ` +
    `Never push, deploy, or merge without an explicit operator instruction.`;
  return text(
    `ship_fr hands you the trigger below (it does NOT auto-run - the human holds the two ` +
    `acceptance gates). Follow it now:\n\n${prompt}`,
  );
}

function readIf(p) { try { return readFileSync(p, "utf8"); } catch { return ""; } }

// --- MCP tool registry ---------------------------------------------------------
const repoArg = { repo: { type: "string", description: "Absolute path to the target repo (default: cwd, walked up to the repo root)." } };
const TOOLS = [
  { name: "fr_init",   description: "Vendor the CyberOS machine into a repo (gate autodetect, FR backlog, agent surface, BRAIN). Needs the payload reachable.", inputSchema: { type: "object", properties: { ...repoArg } }, run: toolFrInit },
  { name: "fr_gates",  description: "Run the CyberOS machine gates (the repo's own build/lint/test + coverage, plus caf/awh if present). Green is necessary, never sufficient.", inputSchema: { type: "object", properties: { ...repoArg } }, run: toolFrGates },
  { name: "fr_status", description: "Summarize the feature-request backlog (counts by status, next eligible FR) and the installed CyberOS version.", inputSchema: { type: "object", properties: { ...repoArg } }, run: toolFrStatus },
  { name: "ship_fr",   description: "Return the canonical, HITL-gated trigger to drive the next (or a named) feature-request. Does NOT auto-run or self-accept.", inputSchema: { type: "object", properties: { ...repoArg, fr_id: { type: "string", description: "Optional FR id, e.g. FR-012-slug. Omit to take the next ready_to_implement row." } } }, run: toolShipFr },
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

// --- remote connector mode: `--http [port]` (FR-IMP-076) -----------------------
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
