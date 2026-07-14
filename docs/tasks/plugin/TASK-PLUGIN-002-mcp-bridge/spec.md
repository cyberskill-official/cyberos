---
id: TASK-PLUGIN-002
title: "CyberOS MCP bridge server — exposes CUO/memory/SKILL tools over MCP 2025-11-25 protocol; single binary with stdio + HTTP transports"
module: PLUGIN
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PLUGIN-001, TASK-PLUGIN-005, TASK-PLUGIN-006, TASK-MCP-001, TASK-MCP-003, TASK-MCP-006, TASK-MCP-007]
depends_on: [TASK-PLUGIN-001, TASK-MCP-001, TASK-MCP-003]
blocks: [TASK-PLUGIN-003, TASK-PLUGIN-007]

source_pages:
  - "[Plugin docs](https://cyberos-wiki.cyberskill.world/modules/plugin/) §1"
  - "[Plugin docs](https://cyberos-wiki.cyberskill.world/modules/plugin/) INTEROP section (universal constraints)"
  - "[CUO docs](https://cyberos-wiki.cyberskill.world/modules/cuo/) (supervisor v3.0.0-a4 surface)"

source_decisions:
  - DEC-2410 2026-05-19 — Bridge ships as ONE Rust binary `cyberos-mcp-bridge` at services/plugin-host/ — supports both stdio and HTTP transports
  - DEC-2411 2026-05-19 — Default transport for desktop hosts (Claude Code, Cursor, Codex CLI) is stdio; HTTP is for Cowork + future cloud hosts
  - DEC-2412 2026-05-19 — Bridge implements MCP 2025-11-25 spec — initialize + tools/list + tools/call + capabilities negotiation per TASK-MCP-001
  - DEC-2413 2026-05-19 — Initial tool surface = 8 tools across CUO (4) + memory (2) + SKILL (2); expansion via FR-PLUGIN-002a/b/c successor FRs
  - DEC-2414 2026-05-19 — Long-running tools (CUO execute_workflow) implement MCP Tasks primitive per TASK-MCP-007 — async handle + status poll + resume-on-reconnect
  - DEC-2415 2026-05-19 — Bridge state is stateless — every request carries tenant_id (from JWT) and trace_id; no in-memory session affinity
  - DEC-2416 2026-05-19 — Tool errors MUST distinguish 4 classes (input_validation / authz_denied / upstream_unavailable / internal_error) for host-side UX
  - DEC-2417 2026-05-19 — Bridge MUST emit OpenTelemetry spans for every tool call with attributes plugin_id, tool_name, tenant_id, trace_id; goes to TASK-OBS-001 collector

build_envelope:
  language: rust 1.81
  service: services/plugin-host/
  new_files:
    - services/plugin-host/Cargo.toml
    - services/plugin-host/src/main.rs
    - services/plugin-host/src/transport/mod.rs
    - services/plugin-host/src/transport/stdio.rs
    - services/plugin-host/src/transport/http.rs
    - services/plugin-host/src/handlers/initialize.rs
    - services/plugin-host/src/handlers/tools_list.rs
    - services/plugin-host/src/handlers/tools_call.rs
    - services/plugin-host/src/tools/cuo.rs
    - services/plugin-host/src/tools/memory.rs
    - services/plugin-host/src/tools/skill.rs
    - services/plugin-host/src/error.rs
    - services/plugin-host/src/otel.rs
    - services/plugin-host/tests/initialize_handshake_test.rs
    - services/plugin-host/tests/tools_list_returns_8_tools_test.rs
    - services/plugin-host/tests/execute_workflow_as_task_test.rs
    - services/plugin-host/tests/error_class_taxonomy_test.rs
    - services/plugin-host/tests/cross_tenant_denied_test.rs

  modified_files:
    - services/Cargo.toml (add plugin-host as workspace member)

  allowed_tools:
    - file_read: services/plugin-host/**
    - file_write: services/plugin-host/{src,tests}/**
    - bash: cd services && cargo test -p cyberos-plugin-host

  disallowed_tools:
    - stateful session affinity (per DEC-2415)
    - mix protocol versions in one binary (per DEC-2412 — single spec)
    - call MCP gateway for own protocol (per architecture — bridge IS the MCP server-side)

effort_hours: 10
subtasks:
  - "0.5h: Cargo.toml + workspace wiring"
  - "0.8h: transport/mod.rs trait + stdio + http impl skeletons"
  - "1.2h: handlers/initialize.rs (capabilities negotiation)"
  - "1.0h: handlers/tools_list.rs (8 tools)"
  - "1.5h: handlers/tools_call.rs (dispatch + Tasks primitive)"
  - "1.0h: tools/cuo.rs (4 tools — list_personas, list_workflows, route, execute_workflow)"
  - "0.6h: tools/memory.rs (2 tools — read_audit, append_audit)"
  - "0.4h: tools/skill.rs (2 tools — list_catalog, invoke_skill)"
  - "0.5h: error.rs (4-class taxonomy)"
  - "0.5h: otel.rs (span emission)"
  - "2.0h: 5 test files"

risk_if_skipped: "Without this bridge, the plugin manifest declares tools that have no server-side implementation — hosts call /tools/list and get empty array. Strategy §4 Level 1 distribution stalls because there's nothing for downloaded plugins to call. Without DEC-2414 Tasks primitive, long-running workflow executions (CUO chains running 30+ seconds) block the host's request thread, breaking UX. Without DEC-2415 statelessness, scaling to multiple bridge instances requires sticky sessions — operational tax. Without DEC-2416 error taxonomy, hosts surface 'something went wrong' to users instead of actionable errors. Without DEC-2417 OTel spans, debugging cross-system tool-call failures requires correlating logs by hand."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** ship the MCP bridge server at `services/plugin-host/` as a single Rust binary `cyberos-mcp-bridge`. The bridge implements MCP 2025-11-25 protocol and exposes 8 CyberOS tools (CUO + memory + SKILL) to any MCP-compliant host.

1. **MUST** ship as single binary supporting BOTH transports per DEC-2410 + DEC-2411:
   - **stdio** — default for desktop hosts. Binary reads JSON-RPC frames from stdin, writes responses to stdout. Activated via `--transport stdio` or default when no flag.
   - **HTTP** — for cloud/server hosts. Listens on `--listen 0.0.0.0:8082`. Uses MCP HTTP-streaming transport per spec. Activated via `--transport http`.

2. **MUST** implement the MCP 2025-11-25 handshake per TASK-MCP-001 + DEC-2412:
   - `initialize` request: client sends `protocolVersion`, `capabilities`, `clientInfo`. Bridge responds with own `protocolVersion: "2025-11-25"`, `capabilities: {tools: {listChanged: false}, logging: {}}`, `serverInfo: {name: "cyberos-mcp-bridge", version: <semver>}`.
   - `initialized` notification from client → bridge marks session ready.
   - Subsequent `tools/list` and `tools/call` requests proceed.

3. **MUST** expose exactly 8 tools at first ship per DEC-2413. Names follow SEP-986 per TASK-MCP-003:
   - `cyberos.cuo.list_personas` — returns the 47 personas + metadata
   - `cyberos.cuo.list_workflows` — for a given persona, return workflows + skill chains
   - `cyberos.cuo.route` — two-stage natural-language router; returns persona+workflow
   - `cyberos.cuo.execute_workflow` — runs a workflow chain (Tasks primitive — see clause 5)
   - `cyberos.memory.read_audit` — returns audit rows for given (actor, kind, since_seq, limit); read-only
   - `cyberos.memory.append_audit` — appends one audit row (write — requires `cyberos:memory:write` scope)
   - `cyberos.skill.list_catalog` — returns 104 author+audit pairs + descriptions
   - `cyberos.skill.invoke_skill` — invokes one skill by id with given inputs (Tasks primitive)

4. **MUST** wire each tool to its source-of-truth module per architecture:
   - CUO tools → call `modules/cuo/` Python supervisor via subprocess invocation through `cyberos-cuo` binary, OR via in-process FFI if the supervisor ships as a library by then
   - memory tools → call `services/memory/` HTTP REST at `MEMORY_ENDPOINT` env var
   - SKILL tools → call `services/skill-broker/` (in flight per TASK-SKILL-103) OR file-scan `modules/skill/` catalog if broker not ready

5. **MUST** implement MCP Tasks primitive per TASK-MCP-007 for `cyberos.cuo.execute_workflow` + `cyberos.skill.invoke_skill` per DEC-2414. Long-running tools:
   - Return `{task_id, status: "running"}` immediately on call
   - Support `tasks/get?id=<task_id>` for status polling (status: running / completed / failed / cancelled)
   - Support `tasks/cancel?id=<task_id>`
   - Persist task state to PostgreSQL (`plugin_host.tasks` table) so reconnects resume in-flight tasks
   - 10-minute default timeout; tools can override via input field

6. **MUST** be stateless across requests per DEC-2415. No in-memory session affinity; every request authenticates via OAuth-PKCE JWT (TASK-PLUGIN-005) carrying `tenant_id` claim. The bridge MUST NOT cache identity between requests. Long-running tasks persist their state in Postgres, not in memory.

7. **MUST** return errors in 4 distinct classes per DEC-2416 + clause 8 error envelope below:
   - `input_validation` — input doesn't match tool's input_schema; HTTP 422 / JSON-RPC code -32602
   - `authz_denied` — token scope insufficient for the tool; HTTP 403 / JSON-RPC code -32000
   - `upstream_unavailable` — downstream service (memory HTTP, CUO subprocess, SKILL broker) failed or timed out; HTTP 503 / JSON-RPC code -32001
   - `internal_error` — bridge bug; HTTP 500 / JSON-RPC code -32603
   Every error response carries `error.class`, `error.message`, `error.trace_id`, and an actionable `error.hint`.

8. **MUST** emit OpenTelemetry spans per DEC-2417 for every tool call with attributes `cyberos.plugin_id`, `cyberos.tool_name`, `cyberos.tenant_id`, `cyberos.trace_id`, `cyberos.duration_ms`, `cyberos.outcome` (success / error_class). Spans flow to TASK-OBS-001 collector via OTLP gRPC.

9. **MUST** honour tool-annotation gating per TASK-MCP-006. Tools with `destructive: true` or `external_effect: true` annotations (`cyberos.memory.append_audit` for example) require the host to set `mcp/sampling/elicit_confirm: true` in the call envelope OR include a pre-signed confirmation token. Bridge rejects un-confirmed destructive calls with `authz_denied`.

10. **MUST** validate every `tools/call` request against the tool's declared `input_schema` per TASK-PLUGIN-001 clause 1 before invoking the upstream. Schema-violation calls produce `input_validation` errors WITHOUT touching the upstream.

11. **MUST** include CORS headers when running in HTTP transport with `--cors-origin <pattern>` flag. Default: deny all origins (no CORS headers). Operators set the flag explicitly per deployment.

12. **MUST NOT** retain client-supplied state across requests beyond what task persistence requires (DEC-2415). No session cookies, no auth caches longer than the JWT's natural validity.

13. **MUST NOT** include CyberOS-internal data in error messages — error bodies are user-safe but do NOT leak: memory audit row contents, JWT bytes, internal DB IDs (use trace_id instead), tenant data from other tenants.

14. **MUST NOT** speak any MCP protocol version other than 2025-11-25 in v1 per DEC-2412. Future protocol versions land via FR-MCP-001b/c successor FRs.

---

## §2 — Why this design

**Why single binary, both transports (DEC-2410)?** Two binaries doubles the maintenance + signing + audit surface. The transport abstraction (`transport/mod.rs` trait) costs ~80 lines and saves duplicating the handler stack. Hosts pick the transport that fits — desktop hosts get stdio (no port binding), cloud hosts get HTTP (load-balanced).

**Why stdio default (DEC-2411)?** Claude Code, Cursor, Codex CLI all use stdio in 2026 — process-per-plugin model. HTTP would require port allocation per plugin, exposes attack surface, and requires the host to manage process supervision differently. Stdio matches existing host expectations.

**Why MCP 2025-11-25 only (DEC-2412, clause 14)?** Mixing protocol versions in one binary multiplies code paths quadratically. TASK-MCP-001 already commits the gateway to this version. Plugin bridge inherits.

**Why exactly 8 tools at first ship (DEC-2413)?** Each tool is an API contract that must be stable (clients depend on the name + schema). Shipping 8 well-considered tools is far better than 30 hastily-named ones. The 8 cover the most common host workflows: orchestration (CUO 4), memory (memory 2), skill discovery (SKILL 2). Future tools land via FR-PLUGIN-002a/b/c.

**Why subprocess invocation for CUO (clause 4)?** CUO supervisor is Python; bridge is Rust. Without a Python embedding (PyO3 is heavy), the cleanest IPC is subprocess + JSON-RPC over stdin/stdout. Subprocess invocation also gives clean process isolation — a CUO crash doesn't take the bridge down.

**Why HTTP REST for memory (clause 4)?** memory service ships as a Fargate binary (per TASK-MEMORY-104). HTTP REST is the natural client surface. Bridge can call it from any deployment context.

**Why Tasks primitive for long-running tools (DEC-2414, clause 5)?** CUO workflow chains run for 30 seconds to several minutes. A synchronous JSON-RPC response would either timeout or hold the connection. MCP Tasks primitive (TASK-MCP-007) is the canonical solution: client receives a handle, polls status, optionally cancels. Reconnect-resume is essential for desktop hosts that may disconnect.

**Why Postgres-backed task state (clause 5)?** In-memory task state breaks reconnect-resume. SQLite works for single-instance, but bridge scaling to multiple instances (Fargate) requires shared store. Postgres is already in the stack for AUTH + memory; reusing it costs nothing.

**Why stateless across requests (DEC-2415, clause 6)?** Stateful bridges require sticky sessions, complicating load balancers and rolling deploys. Statelessness with JWT auth is the cloud-native default.

**Why 4 error classes (DEC-2416, clause 7)?** Hosts surface different UI for different error classes: input_validation → "fix your input" inline; authz_denied → "request more scopes" CTA; upstream_unavailable → "try again in a moment" retry banner; internal_error → silent telemetry + generic apology. Without the taxonomy, hosts default to generic messages and users don't know what to do.

**Why required OTel spans (DEC-2417, clause 8)?** Plugin tool-call failures span 4 systems (host → bridge → upstream module → downstream service). Trace correlation requires every hop to emit spans. The bridge is the middle hop and must emit; otherwise the trace is fragmented and root-cause analysis is hand-stitched.

**Why CORS off by default (clause 11)?** HTTP transport is for cloud hosts that authenticate before reaching the bridge. Open CORS would let any web page call the bridge with the user's token. Closed-by-default + explicit per-deployment opt-in is safe.

**Why don't error messages leak audit data (clause 13)?** Plugins run with user-scoped tokens. An error message that includes "audit row seq 4827 was about <subject>" would leak data the user is not authorised to read in this context. Trace_id is the safe correlation token.

---

## §3 — API contract

### Initialize handshake

Client → bridge:
```json
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
  "protocolVersion": "2025-11-25",
  "capabilities": {"sampling": {}, "elicitation": {}},
  "clientInfo": {"name": "claude-code", "version": "0.42.0"}
}}
```

Bridge → client:
```json
{"jsonrpc": "2.0", "id": 1, "result": {
  "protocolVersion": "2025-11-25",
  "capabilities": {"tools": {"listChanged": false}, "logging": {}},
  "serverInfo": {"name": "cyberos-mcp-bridge", "version": "1.0.0"}
}}
```

### tools/list response (shape)

```json
{"jsonrpc": "2.0", "id": 2, "result": {"tools": [
  {
    "name": "cyberos.cuo.list_personas",
    "description": "List the 47 active CyberOS personas (CTO, CFO, CPO, ...) with metadata for each.",
    "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false},
    "annotations": {"destructive": false, "write": false, "external_effect": false}
  },
  // ... 7 more tools
]}}
```

### tools/call for long-running execute_workflow

Client → bridge:
```json
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
  "name": "cyberos.cuo.execute_workflow",
  "arguments": {
    "persona": "chief-technology-officer",
    "workflow": "architect-new-system",
    "inputs": {"context": "Build a payment routing system for SEA markets"}
  }
}}
```

Bridge → client (immediate, before workflow completes):
```json
{"jsonrpc": "2.0", "id": 3, "result": {
  "content": [{"type": "text", "text": "{\"task_id\":\"t-abc123\",\"status\":\"running\"}"}],
  "isError": false
}}
```

Client polls:
```json
{"jsonrpc": "2.0", "id": 4, "method": "tasks/get", "params": {"id": "t-abc123"}}
```

Bridge → client when complete:
```json
{"jsonrpc": "2.0", "id": 4, "result": {
  "task_id": "t-abc123",
  "status": "completed",
  "output": {"steps_executed": 10, "final_artifacts": [...]},
  "duration_ms": 47230
}}
```

### Error envelope

```json
{"jsonrpc": "2.0", "id": 5, "error": {
  "code": -32000,
  "message": "Tool 'cyberos.memory.append_audit' requires scope 'cyberos:memory:write' which is not in your token.",
  "data": {
    "class": "authz_denied",
    "trace_id": "01HXXXXXXXXXXXXXXXXXXXXXXX",
    "hint": "Re-authorise the plugin with the 'cyberos:memory:write' scope, or have an admin grant it.",
    "missing_scopes": ["cyberos:memory:write"]
  }
}}
```

### Postgres schema for task persistence

```sql
CREATE TABLE plugin_host.tasks (
  task_id TEXT PRIMARY KEY,
  tenant_id UUID NOT NULL,
  plugin_id TEXT NOT NULL,
  tool_name TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('running','completed','failed','cancelled')),
  input JSONB NOT NULL,
  output JSONB,
  error JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at TIMESTAMPTZ,
  timeout_at TIMESTAMPTZ NOT NULL,
  trace_id CHAR(32) NOT NULL
);
ALTER TABLE plugin_host.tasks ENABLE ROW LEVEL SECURITY;
CREATE POLICY tasks_rls ON plugin_host.tasks
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
CREATE INDEX ON plugin_host.tasks (tenant_id, status, created_at DESC);
```

---

## §4 — Acceptance criteria

1. **Initialize handshake succeeds** — bridge returns protocolVersion 2025-11-25 on `initialize`.
2. **tools/list returns 8 tools** — exactly 8 entries, all with SEP-986-conformant names.
3. **Tool name pattern enforced** — bridge rejects renaming a tool to a non-SEP-986 string at compile time.
4. **execute_workflow returns task_id** — synchronous response contains running status + task_id within 100ms.
5. **Task status polling works** — `tasks/get` returns current status; transitions running → completed.
6. **Task cancellation works** — `tasks/cancel` aborts upstream subprocess and marks task cancelled.
7. **Reconnect-resume works** — disconnect/reconnect on same task_id retrieves current state from Postgres.
8. **Cross-tenant denied** — token for tenant A cannot retrieve task created by tenant B (RLS).
9. **Input validation rejects malformed args** — calling `cyberos.cuo.execute_workflow` with missing required `persona` returns input_validation error.
10. **Authz denied on missing scope** — calling `cyberos.memory.append_audit` with read-only token returns authz_denied with `missing_scopes` array.
11. **Upstream unavailable on memory down** — when memory HTTP is unreachable, bridge returns upstream_unavailable (not internal_error).
12. **Internal error on bridge bug** — panic in handler returns internal_error envelope (panic_hook catches).
13. **All errors carry trace_id** — 4 error classes, 4 fixture tests, all assert trace_id present.
14. **OTel span emitted per tool call** — test inspects OTel exporter, finds span with cyberos.tool_name attribute.
15. **Destructive tool requires confirmation** — `cyberos.memory.append_audit` without `elicit_confirm: true` returns authz_denied.
16. **Stateless across requests** — second `tools/call` with new JWT for different tenant returns different tenant's results.
17. **Task timeout enforced** — task running > timeout_at marked failed; status: failed; error.class: internal_error with timeout hint.
18. **CORS denies by default** — HTTP transport without `--cors-origin` flag returns no CORS headers; browser-origin OPTIONS denied.
19. **CORS allows configured origin** — `--cors-origin https://example.com` adds Access-Control-Allow-Origin header on matching requests.
20. **stdio transport echoes JSON-RPC** — send initialize via stdin → response on stdout; no port binding.
21. **HTTP transport binds port** — `--listen 0.0.0.0:8082` opens port; `/healthz` returns 200.
22. **Tool error message does not leak audit content** — failing `read_audit` for missing row returns generic message + trace_id, no body bytes.
23. **Initialize negotiates capabilities** — client sends `capabilities.sampling: {}`; bridge responds without sampling in its own caps; client knows.
24. **Protocol version mismatch rejected** — client sends `protocolVersion: "2024-01-01"`; bridge returns error code -32602 with hint.

---

## §5 — Verification

```rust
// services/plugin-host/tests/initialize_handshake_test.rs
#[tokio::test]
async fn initialize_returns_2025_11_25() {
    let bridge = TestBridge::new_stdio().await;
    let resp = bridge.send(json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}
    })).await;
    assert_eq!(resp["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(resp["result"]["serverInfo"]["name"], "cyberos-mcp-bridge");
}
```

```rust
// services/plugin-host/tests/tools_list_returns_8_tools_test.rs
#[tokio::test]
async fn tools_list_has_exactly_8() {
    let bridge = TestBridge::new_stdio().await.initialized().await;
    let resp = bridge.send_method("tools/list", json!({})).await;
    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 8);
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    let sep_pattern = regex::Regex::new(r"^cyberos\.[a-z][a-z0-9]*\.[a-z][a-z0-9_]*$").unwrap();
    for name in &names {
        assert!(sep_pattern.is_match(name), "name '{}' violates SEP-986", name);
    }
    assert!(names.contains(&"cyberos.cuo.execute_workflow"));
}
```

```rust
// services/plugin-host/tests/execute_workflow_as_task_test.rs
#[tokio::test]
async fn execute_workflow_returns_task_id_and_completes() {
    let bridge = TestBridge::new_with_mock_cuo().await.initialized().await;
    let resp = bridge.tools_call("cyberos.cuo.execute_workflow", json!({
        "persona": "chief-technology-officer",
        "workflow": "adr-quick-capture",
        "inputs": {"title": "test"}
    })).await;
    let body: serde_json::Value = serde_json::from_str(
        resp["result"]["content"][0]["text"].as_str().unwrap()
    ).unwrap();
    let task_id = body["task_id"].as_str().unwrap().to_string();
    assert_eq!(body["status"], "running");

    // Poll until complete (≤ 5s)
    for _ in 0..50 {
        let s = bridge.tasks_get(&task_id).await;
        if s["status"] == "completed" { return; }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("task did not complete");
}
```

```rust
// services/plugin-host/tests/error_class_taxonomy_test.rs
#[tokio::test]
async fn missing_scope_returns_authz_denied() {
    let bridge = TestBridge::new_with_read_only_token().await.initialized().await;
    let resp = bridge.tools_call("cyberos.memory.append_audit", json!({
        "kind": "decision", "body": {}
    })).await;
    assert_eq!(resp["error"]["data"]["class"], "authz_denied");
    assert!(resp["error"]["data"]["missing_scopes"]
        .as_array().unwrap().contains(&json!("cyberos:memory:write")));
    assert!(resp["error"]["data"]["trace_id"].is_string());
}

#[tokio::test]
async fn malformed_input_returns_input_validation() {
    let bridge = TestBridge::new_with_full_token().await.initialized().await;
    let resp = bridge.tools_call("cyberos.cuo.execute_workflow", json!({
        // missing required 'persona'
        "workflow": "adr-quick-capture", "inputs": {}
    })).await;
    assert_eq!(resp["error"]["data"]["class"], "input_validation");
}
```

```rust
// services/plugin-host/tests/cross_tenant_denied_test.rs
#[tokio::test]
async fn rls_prevents_cross_tenant_task_read() {
    let tenant_a = TestBridge::new_for_tenant("a").await.initialized().await;
    let resp = tenant_a.tools_call("cyberos.cuo.execute_workflow", workflow_args()).await;
    let task_id = extract_task_id(&resp);

    let tenant_b = TestBridge::new_for_tenant("b").await.initialized().await;
    let resp_b = tenant_b.tasks_get(&task_id).await;
    assert_eq!(resp_b["error"]["data"]["class"], "input_validation"); // task not found from b's view
}
```

---

## §6 — Implementation skeleton

Bridge crate layout:

```
services/plugin-host/
├── Cargo.toml                              (cyberos-plugin-host crate)
├── src/
│   ├── main.rs                             (transport dispatch: stdio | http)
│   ├── transport/
│   │   ├── mod.rs                          (Transport trait)
│   │   ├── stdio.rs                        (read stdin frames, write stdout)
│   │   └── http.rs                         (axum router with /mcp endpoint)
│   ├── handlers/
│   │   ├── initialize.rs                   (capabilities negotiation)
│   │   ├── tools_list.rs                   (static 8-tool registry)
│   │   └── tools_call.rs                   (dispatch + Tasks for long-running)
│   ├── tools/
│   │   ├── cuo.rs                          (4 tools via cyberos-cuo subprocess)
│   │   ├── memory.rs                        (2 tools via memory HTTP)
│   │   └── skill.rs                        (2 tools via skill-broker / fs scan)
│   ├── error.rs                            (4-class taxonomy + envelope)
│   └── otel.rs                             (span emission)
└── tests/                                  (5 integration tests)
```

Tool registry is static (`tools_list.rs::TOOLS: &[ToolDef; 8]`) — no dynamic registration in v1.

---

## §7 — Dependencies

- **Upstream:** TASK-PLUGIN-001 (manifest schema — bridge tools MUST match manifest tool list); TASK-MCP-001 (MCP 2025-11-25 spec); TASK-MCP-003 (SEP-986 naming).
- **Downstream:** TASK-PLUGIN-003 (slash commands invoke bridge tools); TASK-PLUGIN-007 (multi-runtime adapters bundle this binary).
- **Cross-module:** TASK-MCP-006 (tool annotation gating — clause 9 enforces); TASK-MCP-007 (Tasks primitive — clause 5 implements); TASK-AUTH-004 (JWT verification, shipped); TASK-MEMORY-101 (audit ingest, shipped); TASK-OBS-001 (OTel collector — clause 8 emits to it).

---

## §8 — Example payloads

(See §3 for handshake, tools/list, tools/call, error envelopes, and DB schema.)

### Sample tool: `cyberos.cuo.route` call

Request:
```json
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{
  "name":"cyberos.cuo.route",
  "arguments":{"query":"Architect a new payment routing system for Southeast Asian markets"}
}}
```

Response:
```json
{"jsonrpc":"2.0","id":7,"result":{
  "content":[{"type":"text","text":"{\"persona\":\"chief-technology-officer\",\"workflow\":\"architect-new-system\",\"confidence\":0.92,\"alternates\":[\"chief-product-officer/product-roadmap\"]}"}],
  "isError":false
}}
```

### Sample memory audit row from a plugin tool invocation

Emitted by `cyberos.memory.append_audit` itself, but also by `cyberos.cuo.execute_workflow` upon completion:
```json
{
  "kind": "plugin.invoked",
  "actor_id": "00000000-...",
  "tenant_id": "11111111-...",
  "body": {
    "plugin_id": "cyberos",
    "plugin_version": "1.0.0",
    "tool_name": "cyberos.cuo.execute_workflow",
    "trace_id": "01HX...",
    "duration_ms": 47230,
    "outcome": "success"
  }
}
```

---

## §9 — Open questions

All resolved.

- ~~Should the bridge embed CUO supervisor via PyO3?~~ → No, subprocess invocation per clause 4. Lighter dependency footprint, clean process isolation.
- ~~Should we support MCP 2024-11-05 alongside 2025-11-25?~~ → No, single version per DEC-2412 + clause 14. Multi-version handled in future FR.
- ~~Should task state live in SQLite for single-instance simplicity?~~ → No, Postgres per clause 5. Multi-instance is the deploy target.
- ~~Should CORS default to permissive for dev ergonomics?~~ → No, deny by default per clause 11. Dev opt-in is explicit `--cors-origin '*'`.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Client sends wrong protocol version | initialize handler version check | error -32602 with hint | Client upgrades to 2025-11-25 |
| stdio frame malformed | JSON-RPC parser fails | error -32700 (parse error) | Client retries with valid frame |
| CUO subprocess crashes | tokio::process child exit code != 0 | task marked failed; error class upstream_unavailable | Bridge spawns fresh subprocess on next call |
| memory HTTP times out | reqwest 5s timeout | upstream_unavailable | Retry with backoff or surface to user |
| Postgres unavailable | sqlx connection error | upstream_unavailable for task ops; degraded mode for short tools | Recovery on next call; alerting via TASK-OBS-007 |
| JWT signature invalid | auth middleware | authz_denied with hint "re-authenticate" | Client refreshes token |
| JWT expired | auth middleware exp claim check | authz_denied with hint "token expired" | Client refreshes via OAuth-PKCE (TASK-PLUGIN-005) |
| Scope missing for destructive tool | tools_call.rs scope check | authz_denied with missing_scopes array | Client re-authorises with broader scope |
| Cross-tenant task fetch | Postgres RLS denies | input_validation "task not found" (intentionally generic) | Inherent — no recovery, no leak |
| Task timeout exceeded | tokio::time::timeout future | task marked failed; error.message includes timeout hint | Client invokes again with longer timeout if tool supports |
| Tool input fails JSON schema | tools_call.rs schema validator | input_validation with JSON-pointer to bad field | Client fixes input |
| Bridge panic in handler | catch_unwind in transport layer | internal_error envelope; OTel span tagged with outcome=panic | Service restart by Fargate health check |
| OTel exporter unreachable | exporter buffer fills | spans dropped (logged), tool call still succeeds | OTel recovers; spans buffered up to capacity |
| Concurrent task_id collision | Postgres PK collision | retry with new ID; if persistent, internal_error | Inherent (PK retry) |
| CORS preflight from disallowed origin | http handler checks origin allowlist | 403 with explicit reason | Operator extends `--cors-origin` allowlist |

---

## §11 — Implementation notes

- §11.1 **Transport trait.** `transport/mod.rs::Transport` exposes `async fn next_request(&mut self) -> Result<Request>` and `async fn send_response(&mut self, resp: Response)`. stdio impl wraps tokio::io::BufReader; http impl wraps axum::Router state. Handlers are transport-agnostic.

- §11.2 **Static tool registry.** `tools_list.rs::TOOLS: &[ToolDef; 8]` is a `phf` or simple `static` array. Each entry has `name`, `description`, `input_schema_json`, `annotations`, `handler_fn: fn(&Args) -> Result<Output>`. Adding tools is a code change, not config — keeps the contract tight in v1.

- §11.3 **CUO subprocess invocation.** Bridge spawns `cyberos-cuo execute <persona>/<workflow> --output-dir /tmp/<task_id> --invoker llm --memory-emit --actor <subject_id> --tenant <tenant_id>` and reads JSON output. Long-running tasks tail the output dir for step completions.

- §11.4 **memory HTTP client.** `reqwest::Client` with default 5s timeout, 3 retries with exponential backoff (250ms / 1s / 4s). Body bytes flow as `application/json`.

- §11.5 **Task polling design.** Client polls `tasks/get` every 500ms-1s typically. Bridge serves from Postgres directly (cheap read). For chatty clients, optional server-sent events (`tasks/stream?id=...`) lands in FR-PLUGIN-002a.

- §11.6 **Reconnect resume.** Task state in Postgres is the canonical record. On client reconnect, client re-sends `tasks/get`; bridge looks up by task_id (scoped by tenant_id via RLS). State is current as of last status update. Final output is delivered via the same poll loop.

- §11.7 **Error envelope hints.** Every error class has a `hint` field with an actionable suggestion. The hint text is in `error.rs::ERROR_HINTS` (HashMap<&str, &str>). This is the difference between "permission denied" and "permission denied — re-authorise with scope X."

- §11.8 **OTel span attributes.** Each span captures: cyberos.plugin_id, cyberos.plugin_version, cyberos.tool_name, cyberos.tenant_id (hashed for privacy if TASK-AI-011 PII redaction triggers), cyberos.trace_id, cyberos.duration_ms, cyberos.outcome. Spans flow via OTLP gRPC to TASK-OBS-001 collector.

- §11.9 **CORS implementation.** axum's `tower-http::cors::CorsLayer` with allowlist origins. Default config has no `Allow-Origin` headers, which is browser-side denial.

- §11.10 **Why no in-process Python.** PyO3 adds heavy build dependencies (Python lib at link time) and complicates static binary distribution. Subprocess invocation costs ~10ms per call (acceptable for tool calls in the 100ms-30s range) and avoids the dependency.

- §11.11 **Static binary.** Compile with `RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-musl` to produce a 100% static binary that installs without runtime deps. Matches the `services/auth/`, `services/memory/` patterns.

---

*End of TASK-PLUGIN-002 spec.*
