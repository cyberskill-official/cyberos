---
id: TASK-MCP-001
title: "MCP Gateway 2025-11-25 spec compliance — initialize + tools/list + tools/call + capabilities negotiation + Streamable HTTP transport + tool annotations"
module: MCP
priority: MUST
status: done
verify: T
phase: P0
milestone: P0 · slice 4 (MCP Gateway)
slice: 4
owner: Stephen Cheng (CTO)
created: 2026-05-16
shipped: 2026-06-24
memory_chain_hash: pending
related_tasks: [TASK-AUTH-004, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-MCP-002, TASK-MCP-003, TASK-MCP-004, TASK-MCP-005, TASK-MCP-006, TASK-MCP-007, TASK-MCP-008]
depends_on: [TASK-AUTH-004]
blocks: [TASK-MCP-002, TASK-MCP-003, TASK-MCP-004, TASK-MCP-006, TASK-MCP-007, TASK-MCP-008]

source_pages:
  - website/docs/modules/mcp.html#what
  - website/docs/modules/mcp.html#client-federation
  - website/docs/modules/mcp.html#tool-discovery
  - https://modelcontextprotocol.io/specification/2025-11-25 (referenced for protocol shape; cited verbatim)
source_decisions:
  - DEC-260 (MCP spec version pinned at 2025-11-25; client capability negotiation requires matching `protocolVersion` field on initialize)
  - DEC-261 (Streamable HTTP transport with SSE for server-pushed messages; legacy HTTP+SSE deprecated paths NOT implemented)
  - DEC-262 (federation strategy: gateway holds the public endpoint; per-module servers register via TASK-MCP-002 and the gateway maintains the federated tool catalog in-memory)
  - DEC-263 (JSON-RPC 2.0 batch requests supported per spec; concurrent dispatch with per-tool isolation)
  - "DEC-264 (tool annotations are part of `tools/list` response: `destructive`, `readOnly`, `idempotent`, `openWorld` per spec)"
  - DEC-265 (memory audit row `mcp.tool_call_started` + `mcp.tool_call_completed` pair per invocation — operators tracing crashes need both bookends per task-audit skill rule 26)
  - "DEC-266 (capabilities response declares: `tools` (with listChanged subscription), `prompts`, `resources`, `logging`; `sampling` deferred to slice 5; `roots` deferred)"
  - DEC-267 (every `tools/call` invocation MUST validate caller scope per TASK-AUTH-101 + tool's required scope; missing scope → JSON-RPC error code -32001 unauthorized)
  - DEC-268 (rate-limit per (tenant, tool) configurable; default 100 calls/min; soft burst 200; exceeded → JSON-RPC -32002 rate_limited)
  - DEC-269 (tool name format `cyberos.<module>.<verb>_<noun>` per SEP-986 — validator enforced by TASK-MCP-003; this FR ships the registration contract that requires it)
  - DEC-270 (protocol version 2025-11-25 negotiation: client and server exchange `protocolVersion`; mismatch on initialize → JSON-RPC error -32600 with `supported` field)
  - DEC-271 (the gateway is stateless per-request; session state (Tasks, sampling continuations) lives in TASK-MCP-007's Tasks store; no in-process session)
  - DEC-272 (errors follow JSON-RPC 2.0: -32700 parse_error, -32600 invalid_request, -32601 method_not_found, -32602 invalid_params, -32603 internal_error + spec-defined -32001..-32099 for MCP-specific)
  - modelcontextprotocol.io/specification/2025-11-25 (canonical reference)

language: rust 1.81
service: cyberos/services/mcp-gateway/
new_files:
  - services/mcp-gateway/src/lib.rs                                    # crate root
  - services/mcp-gateway/src/transport/http.rs                         # Streamable HTTP + SSE transport per spec
  - services/mcp-gateway/src/transport/sse.rs                          # Server-Sent Events writer
  - services/mcp-gateway/src/protocol/jsonrpc.rs                       # JSON-RPC 2.0 request/response/error/batch parser
  - services/mcp-gateway/src/protocol/initialize.rs                    # initialize handshake + capabilities negotiation
  - services/mcp-gateway/src/protocol/capabilities.rs                  # capabilities advertisement struct (closed shape)
  - services/mcp-gateway/src/protocol/tools_list.rs                    # tools/list handler — returns federated catalog
  - services/mcp-gateway/src/protocol/tools_call.rs                    # tools/call handler — dispatches to module server
  - services/mcp-gateway/src/protocol/errors.rs                        # JSON-RPC error code mapping + spec-defined -32001..-32099
  - services/mcp-gateway/src/federation/registry.rs                    # in-memory federated tool catalog (modules register via TASK-MCP-002)
  - services/mcp-gateway/src/federation/dispatch.rs                    # routes tools/call to the owning module's MCP server endpoint
  - services/mcp-gateway/src/auth.rs                                   # JWT verification per TASK-AUTH-004; scope_grants extraction
  - services/mcp-gateway/src/ratelimit.rs                              # per-(tenant, tool) sliding-window rate limiter
  - services/mcp-gateway/src/annotations.rs                            # ToolAnnotations struct: destructive | readOnly | idempotent | openWorld
  - services/mcp-gateway/src/audit/mcp_events.rs                       # canonical mcp.tool_call_{started,completed} memory row builders
  - services/mcp-gateway/src/handlers/router.rs                        # axum router mounting /mcp endpoint
  - services/mcp-gateway/Cargo.toml                                    # +axum, +tokio, +serde, +serde_json, +futures, +tracing, +reqwest, +tower-http, +cyberos-cli-exit
  - services/mcp-gateway/tests/initialize_test.rs                      # initialize handshake; protocol version match + mismatch
  - services/mcp-gateway/tests/tools_list_test.rs                      # returns federated catalog; pagination via cursor
  - services/mcp-gateway/tests/tools_call_test.rs                      # dispatches to module; returns result + isError
  - services/mcp-gateway/tests/jsonrpc_batch_test.rs                   # batch request returns batch response
  - services/mcp-gateway/tests/transport_http_test.rs                  # Streamable HTTP POST + SSE GET
  - services/mcp-gateway/tests/error_codes_test.rs                     # JSON-RPC errors -32600..-32099 covered
  - services/mcp-gateway/tests/tool_annotations_test.rs                # annotations present in tools/list response
  - services/mcp-gateway/tests/jwt_verification_test.rs                # missing JWT → -32001; invalid JWT → -32001
  - services/mcp-gateway/tests/scope_check_test.rs                     # tool without scope → -32001
  - services/mcp-gateway/tests/rate_limit_test.rs                      # 101st call in 1min → -32002
  - services/mcp-gateway/tests/audit_emission_test.rs                  # every call emits started + completed
  - services/mcp-gateway/tests/spec_conformance_test.rs                # full spec conformance test against modelcontextprotocol/inspector

modified_files:
  - services/auth/src/jwt.rs                                           # expose `Claims::scope_grants()` accessor for MCP gateway use

allowed_tools:
  - file_read: services/mcp-gateway/**
  - file_read: services/auth/src/**
  - file_write: services/mcp-gateway/{src,tests}/**
  - bash: cd services/mcp-gateway && cargo test
  - bash: cd services/mcp-gateway && cargo test --test spec_conformance_test (requires running gateway)

disallowed_tools:
  - implement legacy HTTP+SSE transport (deprecated in 2025-11-25; per DEC-261)
  - bypass JWT verification on tools/call (per §1 #11)
  - introduce a 6th capability beyond {tools, prompts, resources, logging} at slice 4 (per DEC-266)
  - direct provider/skill invocation in the gateway (per DEC-262 — gateway routes, never executes)
  - hold session state in process (per DEC-271 — Tasks store at TASK-MCP-007)

effort_hours: 12
subtasks:
  - "1.0h: transport/http.rs — Streamable HTTP POST + SSE GET conformance to spec"
  - "0.5h: transport/sse.rs — SSE writer with reconnect-id support"
  - "1.0h: protocol/jsonrpc.rs — request/response/error/batch parsing with spec-conformant shapes"
  - "1.0h: protocol/initialize.rs — handshake + capabilities negotiation + protocol version match"
  - "0.5h: protocol/capabilities.rs — capabilities struct matching spec exactly"
  - "0.8h: protocol/tools_list.rs — list handler + cursor pagination"
  - "1.0h: protocol/tools_call.rs — call handler + dispatch + error mapping"
  - "0.5h: protocol/errors.rs — code mapping per JSON-RPC + MCP spec"
  - "0.5h: federation/registry.rs — in-memory catalog; module registration entry points"
  - "1.0h: federation/dispatch.rs — HTTP client to module servers with timeout + retry"
  - "0.5h: auth.rs — JWT verification + scope extraction"
  - "0.5h: ratelimit.rs — sliding-window per (tenant, tool)"
  - "0.4h: annotations.rs — ToolAnnotations struct + serialisation"
  - "0.5h: audit/mcp_events.rs — 2 row builders + chain to memory_writer"
  - "0.4h: handlers/router.rs — axum mount"
  - "2.4h: tests — 12 test files covering protocol handshake, batch, transport, errors, annotations, JWT, scope, rate limit, audit, federation, spec conformance"

risk_if_skipped: "The MCP Gateway is the external-agent door. Without spec-compliant initialize/tools/list/tools/call, Claude/Cursor/Codex/Cline cannot connect to CyberOS at all — every external-agent use-case is blocked. Every downstream MCP FR (TASK-MCP-002 module registration, TASK-MCP-003 SEP-986 naming, TASK-MCP-006 destructive-op gating, TASK-MCP-007 Tasks primitive, TASK-MCP-008 Elicitation) depends on this baseline. Without DEC-261's Streamable HTTP transport, legacy clients lock out; without DEC-262's federation, every module spins up a separate endpoint (operationally untenable at 22 modules); without DEC-265's audit pair, debugging tool-call crashes becomes impossible. The 12h effort lands the foundational JSON-RPC + transport + capability skeleton on which everything else builds."
---

## §1 — Description (BCP-14 normative)

The MCP Gateway service **MUST** ship compliance with MCP specification 2025-11-25 — `initialize` handshake, `tools/list`, `tools/call`, capabilities negotiation, Streamable HTTP transport. Each requirement:

1. **MUST** advertise `protocolVersion: "2025-11-25"` in every `initialize` response (per DEC-260 + DEC-270). On client `initialize` request with a non-matching `protocolVersion`, return JSON-RPC error -32600 `protocol_version_mismatch` with body `{"supported":["2025-11-25"]}`. Future versions will be added to the `supported` array; clients can negotiate the highest mutual version.

2. **MUST** implement the **JSON-RPC 2.0** wire protocol (per DEC-263). Requests, responses, errors, and batch requests conform to https://www.jsonrpc.org/specification. Specifically:
   - Request: `{"jsonrpc":"2.0","method":"<name>","params":<obj>,"id":<id>}`.
   - Response: `{"jsonrpc":"2.0","result":<obj>,"id":<id>}`.
   - Error: `{"jsonrpc":"2.0","error":{"code":<int>,"message":"<text>","data":<obj?>},"id":<id|null>}`.
   - Batch: a JSON array of request objects; response is a JSON array of response objects in the same order (notifications omitted from response).

3. **MUST** implement **Streamable HTTP transport** per spec 2025-11-25 (per DEC-261):
   - POST `/mcp` with `Content-Type: application/json` for client→server requests; response is a single JSON document with `Content-Type: application/json`.
   - GET `/mcp` with `Accept: text/event-stream` opens a Server-Sent Events stream for server→client messages (notifications, sampling requests, elicitation requests).
   - The legacy HTTP+SSE (separate POST + GET pair) transport is NOT implemented.

4. **MUST** support **session resumption** via the `Mcp-Session-Id` request header. Initial `initialize` response sets `Mcp-Session-Id: <uuid>`; subsequent requests carry the same id. Server may reject expired session ids with HTTP 404 (client re-initialises).

5. **MUST** implement the `initialize` method returning capabilities (per DEC-266). The response advertises:
   ```json
   {
     "protocolVersion": "2025-11-25",
     "capabilities": {
       "tools": {"listChanged": true},
       "prompts": {"listChanged": true},
       "resources": {"listChanged": true, "subscribe": true},
       "logging": {}
     },
     "serverInfo": {
       "name": "cyberos.mcp-gateway",
       "version": "<semver>",
       "title": "CyberOS MCP Gateway"
     },
     "instructions": "Federation of 22 CyberOS modules. All calls audit-chained. OAuth 2.1 PKCE auth via TASK-MCP-004."
   }
   ```
   `sampling` and `roots` capabilities are deferred (slice 5+).

6. **MUST** implement `tools/list` returning the federated catalog (per DEC-262). Response shape per spec:
   ```json
   {
     "tools": [
       {
         "name": "cyberos.memory.search_memory",
         "description": "Search memory audit-chained memories by query string.",
         "inputSchema": {"type":"object","properties":{"query":{"type":"string"}},"required":["query"]},
         "annotations": {
           "title": "Search memory",
           "readOnlyHint": true,
           "destructiveHint": false,
           "idempotentHint": true,
           "openWorldHint": false
         }
       }
       /* ... */
     ],
     "nextCursor": "<opaque>"   // present when paginating
   }
   ```
   Cursor pagination is mandatory when catalog > 100 tools.

7. **MUST** implement `tools/call` dispatching to the owning module's MCP server (per DEC-262). Request: `{"name":"cyberos.<module>.<verb>_<noun>", "arguments":<obj>}`. Response per spec:
   ```json
   {
     "content": [
       {"type": "text", "text": "..."},
       {"type": "image", "data": "<base64>", "mimeType": "image/png"},
       {"type": "resource", "resource": {"uri":"..."}}
     ],
     "isError": false,
     "structuredContent": <obj?>
   }
   ```
   `isError: true` on tool-side error; transport-level errors return JSON-RPC error responses instead.

8. **MUST** implement closed JSON-RPC error code mapping (per DEC-272):
   - `-32700 parse_error` — malformed JSON.
   - `-32600 invalid_request` — request not a valid Request object; includes protocol_version_mismatch.
   - `-32601 method_not_found` — method not implemented (e.g. `sampling/createMessage` at slice 4).
   - `-32602 invalid_params` — bad arguments shape.
   - `-32603 internal_error` — gateway-internal error.
   - `-32001 unauthorized` — missing or invalid JWT, missing scope (per §1 #11).
   - `-32002 rate_limited` — per (tenant, tool) rate limit exceeded (per DEC-268).
   - `-32003 tool_not_found` — tool name not in registry.
   - `-32004 module_unreachable` — owning module server returned 5xx or timed out.
   - `-32005 elicitation_required` — destructive tool requires Elicitation flow (per TASK-MCP-006); slice 4 stub returns this when tool annotation `destructiveHint: true` AND `Elicitation-Confirmed: 1` header absent.

9. **MUST** support **JSON-RPC batch requests** (per DEC-263). Server receives a JSON array, dispatches each request concurrently (with per-tool isolation), and returns an array of responses in the **same order**. Notifications (no `id` field) are omitted from the response array.

10. **MUST** implement `notifications/initialized` (client → server, no response expected). After receiving it, the gateway considers the session active and may push server-initiated messages over the SSE stream.

11. **MUST** verify the OAuth 2.1 access token (TASK-AUTH-004 JWT) on every request EXCEPT `initialize`. Verification:
    - JWT signature valid against the AUTH JWKS.
    - `aud` claim matches `https://mcp.cyberos.com` (audience-bound per TASK-MCP-004).
    - `exp` claim not past; `nbf` claim not in future.
    - `scope` claim contains `mcp:tools` (baseline) AND any tool-specific scope per tool annotation.
   Missing/invalid → `-32001 unauthorized` with `data: {"reason":"<token_invalid|aud_mismatch|expired|insufficient_scope>", "required_scopes":[...]}`. The `initialize` exemption allows clients to negotiate before auth, but any subsequent method requires the token.

12. **MUST** rate-limit per (tenant_id, tool_name) using a sliding window (per DEC-268). Default: 100 calls/min; soft burst: 200 in any 30-sec window. Exceeded → `-32002 rate_limited` with `data: {"retry_after_ms": <int>}`. Per-tenant override via tenant policy YAML (out of scope here; FR-MCP-2xx).

13. **MUST** emit `mcp.tool_call_started` memory audit row at the moment the gateway begins dispatch to the module server, AND `mcp.tool_call_completed` at completion (per DEC-265 + task-audit skill rule 26). Both rows carry: `{tenant_id, subject_id_hash16, tool_name, arguments_sha256, persona_version, request_id, trace_id, ts_ns}`. The completed row adds: `outcome` (success | tool_error | module_unreachable | timeout | rate_limited | unauthorized), `duration_ms`, `result_sha256` (SHA-256 of result JSON; for replay verification).

14. **MUST** propagate W3C `traceparent` header from inbound request to the outbound dispatch to the module server. If absent on inbound, generate fresh per task-audit skill rule 22.

15. **MUST** support cursor-based pagination on `tools/list` (per §1 #6). The `cursor` parameter is opaque to the client (base64-encoded internal pagination state); server returns `nextCursor: null` on last page.

16. **MUST** dispatch `tools/call` to the owning module server via HTTP POST `https://<module>.internal.cyberos/mcp` with timeout 30s. The module's response is wrapped in the gateway's JSON-RPC response (the gateway is transparent — adds audit metadata, doesn't transform the result).

17. **MUST** handle module-server timeouts with `-32004 module_unreachable` after 30s; the memory row carries `outcome=timeout`.

18. **MUST** maintain the federated tool catalog in-memory at the gateway. Modules register via `POST /v1/mcp/register` (TASK-MCP-002 ships the handler; this FR ships the registry struct). The catalog is rebuilt at gateway start by polling each module's `/mcp/heartbeat` endpoint.

19. **MUST** annotate every tool in the registry with its capability requirements: `requires_scope: [...]`, `requires_persona: <key?>`, `module: <module-name>`, `endpoint: <internal-url>`. These are NOT exposed in `tools/list` response (internal use only); the public `annotations` field carries the spec-defined `destructiveHint`/`readOnlyHint`/`idempotentHint`/`openWorldHint`.

20. **MUST** complete `initialize` handshake in ≤ 100 ms p95 (no LLM, no DB read; just signature verify + capabilities serialise). `initialize_perf_test` asserts.

21. **MUST** complete `tools/list` in ≤ 200 ms p95 from in-memory registry. `tools_list_perf_test` asserts.

22. **MUST** complete `tools/call` dispatch (gateway-side only — excluding module execution) in ≤ 50 ms p95 (JWT verify + scope check + audit emit + outbound dispatch). `tools_call_perf_test` asserts.

23. **MUST** emit OTel span `mcp.gateway.{initialize,tools_list,tools_call}` per request with attributes: `tenant_id`, `subject_id_hash16`, `tool_name`, `outcome`, `duration_ms`.

24. **MUST** emit OTel metrics:
    - `mcp_gateway_request_total{method, outcome}` (counter).
    - `mcp_gateway_request_latency_ms{method}` (histogram).
    - `mcp_gateway_active_sessions{tenant_id}` (gauge — sessions with `Mcp-Session-Id` still valid).
    - `mcp_gateway_tool_call_total{tool_name, outcome}` (counter).
    - `mcp_gateway_rate_limit_hits_total{tenant_id, tool_name}` (counter).
    - `mcp_gateway_module_unreachable_total{module}` (counter).

25. **MUST** expose `GET /mcp/healthz` returning `{"status":"ok","protocol_version":"2025-11-25","registered_modules":<int>,"registered_tools":<int>}` for liveness checks.

26. **MUST** publish the federation gateway at `https://mcp.cyberos.com/mcp` (production); local dev at `http://localhost:8090/mcp`.

27. **MUST** support graceful shutdown: SIGTERM triggers (a) refuse new requests with HTTP 503, (b) drain in-flight requests with 10s budget, (c) close SSE streams with `Connection: close`, (d) shut down. The shutdown deadline budget is configurable via env `MCP_GATEWAY_SHUTDOWN_BUDGET_SECONDS`.

---

## §2 — Why this design (rationale for humans)

**Why pin spec version 2025-11-25 (§1 #1, DEC-260)?** The MCP spec evolves; pinning a version makes the gateway's contract explicit. Clients negotiate via `protocolVersion` field; mismatches are explicit (`-32600` with `supported` array) rather than silent. Future versions get added to the supported set as deliberately as a Rust crate gets a new minor — never silently accepted.

**Why Streamable HTTP transport only (§1 #3, DEC-261)?** The 2025-11-25 spec deprecates the legacy HTTP+SSE pair (separate POST + GET endpoints). Streamable HTTP unifies them: POST for client→server (request/response), GET for server→client (SSE stream of notifications + samplings + elicitations). Implementing both would double the transport code; deprecated path is a maintenance liability. Modern clients support Streamable HTTP; legacy clients should upgrade.

**Why JSON-RPC 2.0 wire shape (§1 #2)?** It's the MCP-mandated wire protocol. Reusing the standard gives us a parser library + standard error codes; deviation would break every client.

**Why federation strategy in-memory (§1 #18, DEC-262)?** 22 module servers × hundreds of tools each = thousands of catalog entries. Each `tools/list` walking 22 module HTTP calls would be intolerably slow (~1s). In-memory cache, refreshed on TASK-MCP-002 registration events, gives < 200ms `tools/list`. Cache rebuild happens on gateway start + on every module heartbeat. Stale-by-up-to-heartbeat-interval is acceptable.

**Why module-server timeout 30s (§1 #17)?** Tool calls have varied latency: simple lookups ~50ms, LLM-backed tools 5-15s, long-running tasks > 30s (those use the Tasks primitive per TASK-MCP-007). 30s captures the medium-tail without making short failures wait forever; long-running flows use Tasks to decouple.

**Why audit pair started+completed (§1 #13, DEC-265)?** Per task-audit skill rule 26 — operators tracing crashes need both bookends. Started without completed = crash; the operator gets `tool_name`, `tenant_id`, `arguments_sha256` to investigate. Completed-only would hide pre-execution crashes (panic in argument parsing, etc.).

**Why `arguments_sha256` not full arguments in memory row (§1 #13)?** Tool arguments may carry PII (search query, customer name). Storing them raw in the audit chain would create everywhere-PII. The hash is sufficient for "did this exact call happen?" replay; forensic operations join via tenant_id + tool_name + ts.

**Why scope check enforced at gateway (§1 #11, DEC-267)?** Per defense in depth: the gateway IS the trust boundary for external agents. Even if the module server has its own auth, the gateway's check catches issues earlier (cheaper) and uniformly (one place to update scope rules). Tool-specific scope requirements are part of registration.

**Why `Mcp-Session-Id` header for resumption (§1 #4)?** Clients reconnect (network blip, browser refresh). Without session resumption, every reconnect re-runs `initialize`; not catastrophic but adds latency. Session id is a UUID; gateway maintains session metadata (tenant_id, persona) in-memory; expired sessions force re-init.

**Why batch dispatch concurrent with per-tool isolation (§1 #9, DEC-263)?** Batch requests benefit from parallelism (5 unrelated reads run in 100ms instead of 500ms). Per-tool isolation means one tool's failure doesn't fail the batch; each gets its own response slot. JSON-RPC mandates same-order response array — preserved.

**Why -32001..-32005 error codes (§1 #8, DEC-272)?** JSON-RPC reserves -32768..-32000 for protocol; the spec allocates the application-error range -32000..-32099 to MCP. Our usage stays within that allocation. Specific codes (-32001 unauthorized, -32002 rate_limited, -32003 tool_not_found, etc.) give clients actionable error categories without scraping message text.

**Why capabilities response carefully bounded (§1 #5, DEC-266)?** Advertising a capability commits the gateway to implementing the corresponding methods. Slice 4 ships tools + prompts + resources + logging; sampling and roots are deferred. Clients introspect the capabilities map and stop calling unsupported methods — false advertisement causes runtime failures.

**Why per-(tenant, tool) rate limit not per-tenant (§1 #12, DEC-268)?** Some tools are cheap (lookups), some expensive (LLM-backed). A tenant making 100 cheap calls/min is fine; the same tenant making 100 LLM calls/min may exceed budget. Per-tool granularity lets policy be tool-aware.

**Why federation in-memory and not Redis (§1 #18)?** The catalog is small (~1MB for 1000 tools); in-process access is < 1µs; Redis adds a network hop. The gateway is horizontally scalable — each instance maintains its own catalog from registry events; eventual consistency across instances is acceptable (registration events propagate via TASK-MCP-002).

**Why dispatch to internal module endpoints over HTTP not gRPC (§1 #16)?** HTTP is the spec-mandated transport for MCP; module servers ARE MCP servers (per TASK-MCP-002). Using gRPC internally would force every module to implement both. HTTP is consistent and observable (standard logs, traces, metrics).

**Why `tools/list` cursor pagination (§1 #6, §1 #15)?** Catalogs > 100 tools (any module with a rich CRUD surface) produce > 64KB responses. Single-page response inflates latency + memory; cursor pagination keeps each page under 50KB. Opaque cursor lets the gateway change pagination scheme without breaking clients.

**Why `tools/call` dispatch < 50ms gateway overhead (§1 #22)?** Module execution dominates; gateway overhead should be invisible. 50ms = JWT verify (5ms) + scope check (1ms) + audit emit (10ms) + outbound HTTP setup (15ms) + slack (19ms). Past 50ms, the gateway is the bottleneck.

**Why state-less gateway (§1 #4, DEC-271)?** Session resumption uses session id, not in-process session state; Tasks state lives in TASK-MCP-007's persistent store. State-less means horizontal scaling is trivial (any instance can serve any request); operational outages don't lose in-flight work.

**Why server-pushed messages via SSE not WebSocket (§1 #3)?** Streamable HTTP spec mandates SSE for server→client. WebSocket would be bidirectional but the spec deliberately uses SSE + HTTP POST to keep transport simple — separable channels, standard HTTP semantics for the request channel.

**Why audit BEFORE dispatch + AFTER completion not just at-completion (DEC-265)?** Crashes during dispatch (network failure, malformed result) would silently drop the call; the started row preserves the fact of attempt. Operators looking for "what tools were attempted on the affected tenant?" see all of them, not just completions.

**Why exempt `initialize` from auth (§1 #11)?** Clients need to discover capabilities before knowing what scopes to request. `initialize` returns server capabilities + protocol version; this lets the client (a) verify protocol match, (b) prepare an auth flow appropriate to the discovered capabilities. Subsequent methods require auth.

**Why 100 calls/min default (§1 #12)?** Standard interactive use is < 60 calls/min (one call per second is fast). 100 gives margin; 200 burst handles short rushes. Tenants needing more provision via per-tenant override.

**Why `mcp.tool_call_started/completed` not `mcp.tool_invoked`?** Per task-audit skill rule 26: pair-write history events. A single `tool_invoked` row would hide crashes — started without completed signals crash; both rows present = clean execution.

**Why `result_sha256` on completed row (§1 #13)?** Replay verification: given a stored memory row + the original arguments, re-running the tool should produce a result whose SHA-256 matches. Mismatch = the underlying tool changed; useful for EU AI Act Art. 12 replay claims.

---

## §3 — API contract

### 3.1 — JSON-RPC request shapes

```rust
// services/mcp-gateway/src/protocol/jsonrpc.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,                       // always "2.0"
    pub method: String,                        // e.g. "tools/list"
    #[serde(default)]
    pub params: Option<Value>,
    #[serde(default)]
    pub id: Option<Value>,                     // null = notification (no response)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,                       // always "2.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum JsonRpcPayload {
    Single(JsonRpcRequest),
    Batch(Vec<JsonRpcRequest>),
}

pub fn parse_payload(bytes: &[u8]) -> Result<JsonRpcPayload, JsonRpcError> {
    let v: Value = serde_json::from_slice(bytes).map_err(|_| JsonRpcError {
        code: -32700, message: "parse_error".into(), data: None,
    })?;
    match v {
        Value::Array(arr) => {
            let mut batch = Vec::with_capacity(arr.len());
            for item in arr {
                let req: JsonRpcRequest = serde_json::from_value(item).map_err(|_| JsonRpcError {
                    code: -32600, message: "invalid_request".into(), data: None,
                })?;
                batch.push(req);
            }
            Ok(JsonRpcPayload::Batch(batch))
        }
        _ => {
            let req: JsonRpcRequest = serde_json::from_value(v).map_err(|_| JsonRpcError {
                code: -32600, message: "invalid_request".into(), data: None,
            })?;
            Ok(JsonRpcPayload::Single(req))
        }
    }
}
```

### 3.2 — Initialize handler

```rust
// services/mcp-gateway/src/protocol/initialize.rs
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::protocol::capabilities::ServerCapabilities;

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

#[derive(Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
    #[serde(default)]
    pub capabilities: serde_json::Value,
}

#[derive(Deserialize)]
pub struct ClientInfo { pub name: String, pub version: String }

#[derive(Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
    pub instructions: String,
}

#[derive(Serialize)]
pub struct ServerInfo { pub name: String, pub version: String, pub title: String }

pub fn handle_initialize(params: InitializeParams) -> Result<InitializeResult, JsonRpcError> {
    if params.protocol_version != MCP_PROTOCOL_VERSION {
        return Err(JsonRpcError {
            code: -32600,
            message: "protocol_version_mismatch".into(),
            data: Some(json!({"supported": [MCP_PROTOCOL_VERSION]})),
        });
    }
    Ok(InitializeResult {
        protocol_version: MCP_PROTOCOL_VERSION.into(),
        capabilities: ServerCapabilities::current(),
        server_info: ServerInfo {
            name: "cyberos.mcp-gateway".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: "CyberOS MCP Gateway".into(),
        },
        instructions: "Federation of 22 CyberOS modules. All calls audit-chained. OAuth 2.1 PKCE auth via TASK-MCP-004.".into(),
    })
}
```

### 3.3 — Capabilities

```rust
// services/mcp-gateway/src/protocol/capabilities.rs
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolsCap,
    pub prompts: ListChangedCap,
    pub resources: ResourcesCap,
    pub logging: HashMap<String, serde_json::Value>,    // empty obj
}

#[derive(Serialize)]
pub struct ToolsCap { #[serde(rename = "listChanged")] pub list_changed: bool }

#[derive(Serialize)]
pub struct ListChangedCap { #[serde(rename = "listChanged")] pub list_changed: bool }

#[derive(Serialize)]
pub struct ResourcesCap {
    #[serde(rename = "listChanged")] pub list_changed: bool,
    pub subscribe: bool,
}

impl ServerCapabilities {
    pub fn current() -> Self {
        Self {
            tools: ToolsCap { list_changed: true },
            prompts: ListChangedCap { list_changed: true },
            resources: ResourcesCap { list_changed: true, subscribe: true },
            logging: HashMap::new(),
        }
    }
}
```

### 3.4 — tools/list

```rust
// services/mcp-gateway/src/protocol/tools_list.rs
use serde::{Deserialize, Serialize};
use crate::federation::registry::TOOL_REGISTRY;
use crate::annotations::ToolAnnotations;

#[derive(Deserialize)]
pub struct ToolsListParams {
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

#[derive(Serialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    pub annotations: ToolAnnotations,
}

const PAGE_SIZE: usize = 100;

pub fn handle_tools_list(params: ToolsListParams, tenant_id: uuid::Uuid) -> ToolsListResult {
    let reg = TOOL_REGISTRY.load();
    let tools_for_tenant = reg.visible_to_tenant(tenant_id);

    let offset = params.cursor.as_deref()
        .and_then(|c| base64::decode(c).ok())
        .and_then(|b| std::str::from_utf8(&b).ok().and_then(|s| s.parse::<usize>().ok()))
        .unwrap_or(0);

    let slice = tools_for_tenant.iter().skip(offset).take(PAGE_SIZE);
    let descriptors: Vec<ToolDescriptor> = slice.map(|t| t.to_descriptor()).collect();
    let next_cursor = if offset + descriptors.len() < tools_for_tenant.len() {
        Some(base64::encode((offset + descriptors.len()).to_string()))
    } else { None };

    ToolsListResult { tools: descriptors, next_cursor }
}
```

### 3.5 — tools/call dispatch

```rust
// services/mcp-gateway/src/protocol/tools_call.rs
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::federation::{registry::TOOL_REGISTRY, dispatch::dispatch_to_module};
use crate::audit::mcp_events;
use crate::ratelimit::check_rate_limit;
use crate::auth::Claims;

#[derive(Deserialize)]
pub struct ToolsCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Serialize)]
pub struct ToolsCallResult {
    pub content: Vec<ContentBlock>,
    #[serde(rename = "isError")]
    pub is_error: bool,
    #[serde(skip_serializing_if = "Option::is_none", rename = "structuredContent")]
    pub structured_content: Option<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { resource: serde_json::Value },
}

pub async fn handle_tools_call(
    params: ToolsCallParams,
    claims: &Claims,
    request_id: &str,
) -> Result<ToolsCallResult, JsonRpcError> {
    let reg = TOOL_REGISTRY.load();
    let tool = reg.get(&params.name).ok_or_else(|| JsonRpcError {
        code: -32003, message: "tool_not_found".into(),
        data: Some(serde_json::json!({"name": params.name})),
    })?;

    // Scope check
    for required in &tool.required_scopes {
        if !claims.scopes().contains(required) {
            return Err(JsonRpcError {
                code: -32001, message: "unauthorized".into(),
                data: Some(serde_json::json!({"reason":"insufficient_scope","required_scopes": tool.required_scopes})),
            });
        }
    }

    // Rate limit
    if !check_rate_limit(claims.tenant_id(), &params.name).await {
        return Err(JsonRpcError {
            code: -32002, message: "rate_limited".into(),
            data: Some(serde_json::json!({"retry_after_ms": 60_000})),
        });
    }

    // Destructive-op gate (TASK-MCP-006 ships full Elicitation; slice 4 stub)
    if tool.annotations.destructive_hint
       && claims.elicitation_confirmed_for(&params.name).is_none() {
        return Err(JsonRpcError {
            code: -32005, message: "elicitation_required".into(),
            data: Some(serde_json::json!({"tool": params.name})),
        });
    }

    // Audit: started
    let args_sha = hex::encode(Sha256::digest(serde_json::to_vec(&params.arguments).unwrap()));
    mcp_events::emit_tool_call_started(claims, &params.name, &args_sha, request_id).await;

    // Dispatch
    let t0 = std::time::Instant::now();
    let result = dispatch_to_module(&tool.endpoint, &params.name, &params.arguments).await;
    let duration_ms = t0.elapsed().as_millis() as u64;

    let (outcome, response) = match result {
        Ok(r) => ("success", r),
        Err(DispatchError::Timeout) => {
            mcp_events::emit_tool_call_completed(claims, &params.name, "timeout", duration_ms, None, request_id).await;
            return Err(JsonRpcError {
                code: -32004, message: "module_unreachable".into(),
                data: Some(serde_json::json!({"reason":"timeout","module": tool.module})),
            });
        }
        Err(e) => {
            mcp_events::emit_tool_call_completed(claims, &params.name, "module_error", duration_ms, None, request_id).await;
            return Err(JsonRpcError {
                code: -32004, message: "module_unreachable".into(),
                data: Some(serde_json::json!({"reason": format!("{e:?}"),"module": tool.module})),
            });
        }
    };

    let result_sha = hex::encode(Sha256::digest(serde_json::to_vec(&response).unwrap()));
    mcp_events::emit_tool_call_completed(claims, &params.name, outcome, duration_ms, Some(&result_sha), request_id).await;
    Ok(response)
}
```

---

## §4 — Acceptance criteria

1. **Initialize success** — client sends `initialize` with `protocolVersion: "2025-11-25"` → 200 with capabilities + serverInfo.
2. **Initialize version mismatch** — `protocolVersion: "2024-01-01"` → -32600 with `supported: ["2025-11-25"]`.
3. **Capabilities shape** — response contains `tools`, `prompts`, `resources`, `logging`; no `sampling` or `roots`.
4. **JSON-RPC parse error** — malformed JSON body → -32700 `parse_error`.
5. **JSON-RPC invalid request** — missing `method` field → -32600 `invalid_request`.
6. **Method not found** — `notifications/unknown_method` → -32601.
7. **Invalid params** — `tools/call` with non-object params → -32602.
8. **Batch request** — array of 3 valid requests → array of 3 responses in same order.
9. **Notification in batch** — request with no `id` field → omitted from response.
10. **JWT missing on tools/call** → -32001 with `reason: "token_invalid"`.
11. **JWT invalid signature** → -32001 with `reason: "token_invalid"`.
12. **JWT audience mismatch** → -32001 with `reason: "aud_mismatch"`.
13. **JWT scope missing** → -32001 with `required_scopes` array.
14. **Initialize exempt from JWT** — no JWT → `initialize` succeeds.
15. **tools/list cursor pagination** — 250-tool catalog → first page 100 + next_cursor; second page 100 + next_cursor; third page 50 + no next_cursor.
16. **tools/list per-tenant filtering** — tools requiring tenant-specific scope only appear when caller has that scope.
17. **tools/call happy path** — valid name + arguments → dispatched to module; result returned with content array.
18. **tools/call tool not found** → -32003.
19. **tools/call module unreachable** — module returns 5xx within 30s → -32004.
20. **tools/call module timeout** — module takes > 30s → -32004 with `reason: "timeout"`.
21. **tools/call destructive without elicitation** → -32005 `elicitation_required`.
22. **Rate limit hit** — 101st call in 1 min for (tenant, tool) → -32002 with `retry_after_ms`.
23. **memory audit pair emitted** — every tools/call writes both started + completed rows; started has `arguments_sha256`, completed has `outcome` + `duration_ms` + `result_sha256` (on success).
24. **W3C traceparent propagated** — outbound dispatch carries the same `traceparent` as inbound.
25. **Mcp-Session-Id header set on initialize response** — UUID format.
26. **Session re-init on expired Mcp-Session-Id** — gateway returns 404; client re-initialises.
27. **Initialize p95 < 100 ms** — `initialize_perf_test`.
28. **tools/list p95 < 200 ms** — `tools_list_perf_test`.
29. **tools/call gateway-side p95 < 50 ms** — excluding module execution; `tools_call_perf_test`.
30. **GET /mcp/healthz** — returns 200 with protocol_version + counts.
31. **Tool annotations present in tools/list** — every tool descriptor has `annotations: {title, readOnlyHint, destructiveHint, idempotentHint, openWorldHint}`.
32. **OTel span emitted per request** — `mcp.gateway.{initialize,tools_list,tools_call}` with `outcome` attribute.
33. **OTel counter `mcp_gateway_request_total{method=tools_call, outcome=success}` increments** — per call.
34. **Graceful shutdown** — SIGTERM → /healthz returns 503; in-flight requests drained ≤ 10 s; new requests rejected.

---

## §5 — Verification

```rust
// services/mcp-gateway/tests/initialize_test.rs
#[tokio::test]
async fn initialize_returns_protocol_version() {
    let gw = TestGateway::new().await;
    let resp = gw.rpc(json!({
        "jsonrpc":"2.0","method":"initialize","id":1,
        "params":{"protocolVersion":"2025-11-25","clientInfo":{"name":"test","version":"1"}}
    })).await;
    assert_eq!(resp["result"]["protocolVersion"], "2025-11-25");
    assert!(resp["result"]["capabilities"]["tools"]["listChanged"].as_bool().unwrap());
}

#[tokio::test]
async fn initialize_version_mismatch_returns_supported_list() {
    let gw = TestGateway::new().await;
    let resp = gw.rpc(json!({
        "jsonrpc":"2.0","method":"initialize","id":1,
        "params":{"protocolVersion":"2024-01-01","clientInfo":{"name":"test","version":"1"}}
    })).await;
    assert_eq!(resp["error"]["code"], -32600);
    assert_eq!(resp["error"]["data"]["supported"][0], "2025-11-25");
}
```

```rust
// services/mcp-gateway/tests/jsonrpc_batch_test.rs
#[tokio::test]
async fn batch_request_returns_ordered_responses() {
    let gw = TestGateway::new().await;
    let batch = json!([
        {"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2025-11-25","clientInfo":{"name":"a","version":"1"}}},
        {"jsonrpc":"2.0","method":"tools/list","id":2},
        {"jsonrpc":"2.0","method":"tools/list","id":3,"params":{"cursor":"<bad>"}}
    ]);
    let resp = gw.rpc_raw(batch).await;
    let arr = resp.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["id"], 1);
    assert_eq!(arr[1]["id"], 2);
    assert_eq!(arr[2]["id"], 3);
}
```

```rust
// services/mcp-gateway/tests/audit_emission_test.rs
#[tokio::test]
async fn tool_call_emits_started_and_completed_rows() {
    let gw = TestGateway::with_module_mock("memory", "search_memory", json!({"hits": []})).await;
    let resp = gw.tools_call("cyberos.memory.search_memory", json!({"query":"x"})).await;
    let rows = gw.memory_audit_rows().await;
    let started = rows.iter().find(|r| r["kind"] == "mcp.tool_call_started").unwrap();
    let completed = rows.iter().find(|r| r["kind"] == "mcp.tool_call_completed").unwrap();
    assert_eq!(started["tool_name"], "cyberos.memory.search_memory");
    assert!(started["arguments_sha256"].is_string());
    assert_eq!(completed["outcome"], "success");
    assert!(completed["result_sha256"].is_string());
    assert!(completed["duration_ms"].as_u64().unwrap() < 5_000);
}
```

```rust
// services/mcp-gateway/tests/rate_limit_test.rs
#[tokio::test]
async fn 101st_call_returns_rate_limited() {
    let gw = TestGateway::with_module_mock("memory", "search_memory", json!({"hits": []})).await;
    for _ in 0..100 {
        let resp = gw.tools_call("cyberos.memory.search_memory", json!({"query":"x"})).await;
        assert!(resp["result"].is_object());
    }
    let over = gw.tools_call("cyberos.memory.search_memory", json!({"query":"x"})).await;
    assert_eq!(over["error"]["code"], -32002);
    assert!(over["error"]["data"]["retry_after_ms"].as_u64().unwrap() > 0);
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; the federation registry struct, dispatch HTTP client, and SSE writer follow standard axum + tokio patterns.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-004** — JWT issuance + JWKS; this FR verifies tokens against the JWKS.

**Downstream (5 placeholders):**
- **TASK-MCP-002** — per-module server registration + heartbeat; populates the federated registry.
- **TASK-MCP-003** — SEP-986 naming convention validator (`cyberos.<module>.<verb>_<noun>`).
- **TASK-MCP-006** — tool-annotation gating (destructive requires confirm or Elicitation).
- **TASK-MCP-007** — Tasks primitive (long-running work).
- **TASK-MCP-008** — Elicitation server-initiated request/response.

**Cross-module:**
- **TASK-AUTH-101** — RBAC; `Resource::McpTool + Action::Invoke` patterns; agent-persona role on every call.
- **TASK-AI-003** — memory audit bridge; receives `mcp.tool_call_started` + `mcp.tool_call_completed`.
- **TASK-AI-022** — OTel trace emission; traceparent propagation.
- **TASK-MCP-004** — OAuth 2.1 PKCE (this FR consumes JWT shape; TASK-MCP-004 ships the issuance flow).
- **TASK-MCP-005** — Protected Resource Metadata; published at /.well-known/oauth-protected-resource.

---

## §8 — Example payloads

### 8.1 — initialize request

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "id": 1,
  "params": {
    "protocolVersion": "2025-11-25",
    "clientInfo": {"name": "claude-desktop", "version": "0.7.0"},
    "capabilities": {}
  }
}
```

### 8.2 — initialize response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "tools": {"listChanged": true},
      "prompts": {"listChanged": true},
      "resources": {"listChanged": true, "subscribe": true},
      "logging": {}
    },
    "serverInfo": {"name": "cyberos.mcp-gateway", "version": "0.1.0", "title": "CyberOS MCP Gateway"},
    "instructions": "Federation of 22 CyberOS modules. All calls audit-chained. OAuth 2.1 PKCE auth via TASK-MCP-004."
  }
}
```

### 8.3 — tools/list response (excerpt)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "cyberos.memory.search_memory",
        "description": "Search memory audit-chained memories by query string.",
        "inputSchema": {"type":"object","properties":{"query":{"type":"string"}},"required":["query"]},
        "annotations": {
          "title": "Search memory",
          "readOnlyHint": true,
          "destructiveHint": false,
          "idempotentHint": true,
          "openWorldHint": false
        }
      }
    ],
    "nextCursor": "MTAw"
  }
}
```

### 8.4 — tools/call request

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "id": 3,
  "params": {
    "name": "cyberos.memory.search_memory",
    "arguments": {"query": "TASK-AUTH-101"}
  }
}
```

### 8.5 — tools/call response

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {"type": "text", "text": "Found 3 memories matching 'TASK-AUTH-101'..."}
    ],
    "isError": false,
    "structuredContent": {"hits": [{"id": "...", "score": 0.92}]}
  }
}
```

### 8.6 — mcp.tool_call_started memory row

```json
{
  "kind": "mcp.tool_call_started",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "tool_name": "cyberos.memory.search_memory",
  "arguments_sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "persona_version": "cuo-cpo@0.4.1",
  "request_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "ts_ns": 1747920731000000000
}
```

### 8.7 — mcp.tool_call_completed memory row

```json
{
  "kind": "mcp.tool_call_completed",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "tool_name": "cyberos.memory.search_memory",
  "outcome": "success",
  "duration_ms": 47,
  "result_sha256": "a47c8e0c5f8a8e8e8c5f8a8e8e8c5f8a8e8e8c5f8a8e8e8c5f8a8e8e8c5f8a8e",
  "request_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "ts_ns": 1747920731047000000
}
```

---

## §9 — Open questions

Deferred:
- **OAuth 2.1 PKCE flow** — TASK-MCP-004 ships the issuance; this FR consumes the JWT.
- **Protected Resource Metadata (RFC 9728)** — TASK-MCP-005 publishes at `.well-known/oauth-protected-resource`.
- **Destructive-op Elicitation flow** — TASK-MCP-006 ships the full flow; this FR stubs with `-32005 elicitation_required`.
- **Tasks primitive** — TASK-MCP-007.
- **Elicitation primitive** — TASK-MCP-008.
- **Sampling capability** — slice 5+.
- **Roots capability** — slice 5+.
- **Per-tenant rate-limit override** — FR-MCP-2xx.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Malformed JSON request | `serde_json` parse | -32700 parse_error | None — designed |
| Missing `method` field | request shape validation | -32600 invalid_request | None — designed |
| Unknown method | router lookup | -32601 method_not_found | Implement method or upgrade client |
| `tools/call` with non-object params | params parse | -32602 invalid_params | Fix client |
| protocolVersion mismatch | initialize | -32600 with supported list | Client upgrades protocol |
| JWT missing | auth middleware | -32001 token_invalid | Provide token |
| JWT signature invalid | JWKS verify | -32001 token_invalid | Re-auth |
| JWT expired | exp claim | -32001 token_invalid | Refresh token |
| JWT aud mismatch | aud check | -32001 aud_mismatch | Use correct audience |
| JWT scope missing | scope check | -32001 insufficient_scope + required_scopes | Request broader scope |
| Tool not found in registry | registry lookup | -32003 tool_not_found | Client lists tools |
| Module endpoint unreachable | HTTP connect error | -32004 module_unreachable | Operator investigates module |
| Module timeout > 30s | tokio timeout | -32004 with reason=timeout | None — designed |
| Module returns 5xx | HTTP response | -32004 with reason=module_error | None — designed |
| Rate limit exceeded | sliding window | -32002 with retry_after_ms | Wait + retry |
| Destructive tool without Elicitation-Confirmed | annotations + header check | -32005 elicitation_required | Use TASK-MCP-006 flow |
| Audit row commit fails | memory_writer error | 500 internal; tools/call fails | memory_writer health check |
| Batch with empty array | parse | -32600 invalid_request | Send non-empty |
| Notification dispatched twice | id-absence check | second invocation no-op | None — designed |
| SSE stream disconnected | client close | server drops stream gracefully | Client re-opens |
| Mcp-Session-Id expired | session map lookup | HTTP 404 | Client re-initialises |
| Federation registry empty (no modules registered) | health check | tools/list returns empty | TASK-MCP-002 registration recovery |
| Module heartbeat missed | TASK-MCP-002 lifecycle | tools removed from registry; tools/list updates | Module recovers and re-registers |
| traceparent malformed | parse | regenerate at trust boundary per task-audit skill rule 22 | None — designed |
| Mcp-Session-Id format invalid | UUID parse | 400 invalid header | Caller fixes |
| Tool descriptor missing annotations | tool_annotations_test | CI fails | Fix tool registration |
| Streamable HTTP unsupported by client | content-type negotiation | 415 unsupported_media_type | Client upgrades |
| Capabilities response wrong shape | spec_conformance_test | CI fails | Fix struct |
| Result content array empty | tool returns; gateway preserves | content: [] allowed | None — designed |
| `result_sha256` mismatch on replay | analyst tooling | sev-3 alarm | Replay analysis |
| OTel span propagation broken | trace_id test | CI fails | Fix instrumentation |
| Concurrent batch dispatch overflow | tokio task limit | 503 server_busy | Back-off |
| Graceful shutdown drain timeout > 10s | shutdown handler force-aborts | in-flight requests fail with 503 | Designed — operator monitors |
| Module returns malformed JSON | dispatch parse | -32004 reason=module_invalid_json | Module-side investigation |
| Tool argument > 1MB | request body limit | 413 payload_too_large | Caller chunks |

---

## §11 — Implementation notes

- **Spec version pinning at 2025-11-25** is the contract. Future versions add to `supported[]`; never silently accept.
- **Streamable HTTP only** — legacy HTTP+SSE is explicitly NOT implemented. Modern clients support; legacy clients upgrade.
- **JSON-RPC 2.0 wire shape** is non-negotiable; matches the spec exactly.
- **Batch dispatch concurrent + per-tool isolated** — one tool's failure doesn't fail the batch.
- **`Mcp-Session-Id` UUID per session** — gateway-allocated; client preserves across requests; gateway expires after configurable TTL.
- **JWT verified at gateway** — defense in depth even though module servers may have their own auth.
- **Audit rows are PAIR** — `started` + `completed` per task-audit skill rule 26; operators tracing crashes need both.
- **`arguments_sha256` not raw args in memory** — PII protection; tool-side handlers can write the args if appropriate.
- **`result_sha256` enables EU AI Act Art. 12 replay** — re-running with stored args should produce matching hash.
- **`tools/list` cursor pagination** — opaque base64-encoded offset; gateway can change scheme without breaking clients.
- **Per-(tenant, tool) rate limit sliding window** — Redis-backed in production (out of scope for slice 4; in-memory at slice 4 acceptable per spec scope).
- **Federation registry in-memory** — < 1MB; refreshed on TASK-MCP-002 registration events; eventual consistency across gateway instances.
- **Dispatch HTTP client reuses connection pool** — reqwest with `pool_max_idle_per_host = 32`.
- **Tool annotations are spec-defined** (`title`, `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`) — exposed in `tools/list`; internal annotations (`required_scopes`, `required_persona`, `module`, `endpoint`) are NOT exposed.
- **`logging` capability is empty object** — declares support without specific config; FR-MCP-2xx ships log-level subscription.
- **`prompts` + `resources` capability flagged** — Prompts and Resources methods are not implemented at slice 4 (deferred); the capability flag signals support; calling them returns -32601.
- **Server graceful shutdown drains 10s** — configurable via env; tokio drop graceful.
- **GET /mcp/healthz** is unauthenticated — liveness probe target for k8s.
- **Audience-bound JWT** — `aud: https://mcp.cyberos.com` per TASK-MCP-004; gateway rejects tokens for other audiences.
- **Spec conformance test runs against modelcontextprotocol/inspector** — full reference client validates the gateway end-to-end at CI.
- **JSON-RPC errors -32001..-32099** are MCP-specific (per spec); custom codes outside that range = bug.
- **`elicitation_required` -32005 is a stub at slice 4** — TASK-MCP-006 ships the full flow with the `Elicitation-Confirmed` header check + back-and-forth.
- **`Mcp-Session-Id` allows re-init on expiry** — client treats HTTP 404 on a request with valid id as "re-initialise"; not a hard error.
- **Module endpoint discovered via TASK-MCP-002 registration** — at slice 4 this FR uses a static map for tests; production lookup via TASK-MCP-002.
- **No persistence in this FR** — sessions are in-memory; rate-limit windows in-memory; registry in-memory. Persistence (Tasks store, session re-distribution across instances) lands in later FRs.

---

*End of TASK-MCP-001.*
