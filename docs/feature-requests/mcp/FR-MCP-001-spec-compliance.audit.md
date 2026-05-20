---
fr_id: FR-MCP-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 11
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per feature-request-audit skill §0)
---

## §1 — Verdict summary

FR-MCP-001 ships the MCP Gateway 2025-11-25 spec baseline — initialize + tools/list + tools/call + capabilities + Streamable HTTP transport + tool annotations + JSON-RPC 2.0 + audit pair + JWT verification + per-tenant-tool rate limiting. Scope: 27 §1 normative clauses covering protocol version pinning, JSON-RPC 2.0 wire shape, Streamable HTTP + SSE, session resumption via Mcp-Session-Id, initialize handshake, capabilities advertisement (tools+prompts+resources+logging), tools/list with cursor pagination, tools/call dispatch with destructive-op stub gate, federated catalog in-memory, JSON-RPC error mapping -32700..-32099, batch requests with concurrent dispatch, JWT verify (FR-AUTH-004) + scope enforcement, sliding-window rate limit, memory audit pair (started+completed), W3C traceparent propagation, /healthz endpoint, graceful shutdown. 22 rationale paragraphs. §3 contains: JSON-RPC parser, initialize handler, capabilities struct, tools_list with cursor, tools_call with dispatch + audit. 34 ACs. 32 failure-mode rows. 25 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Protocol version not negotiated
First-pass had no `protocolVersion` field in initialize. Resolved: §1 #1 + DEC-260 + DEC-270 + AC #2 supported-array response.

### ISS-002 — Legacy HTTP+SSE transport leakage
First-pass implemented both Streamable HTTP and legacy paths. Resolved: §1 #3 + DEC-261 + disallowed_tools enforcement.

### ISS-003 — JWT verification missing for tools/call
First-pass exempted all methods from auth. Resolved: §1 #11 + DEC-267 + scope-per-tool enforcement; AC #10–13.

### ISS-004 — Destructive-op silent invocation
First-pass invoked destructive tools without confirmation. Resolved: §1 #8 + DEC-264 + `-32005 elicitation_required` stub gate; AC #21. FR-MCP-006 ships the full flow.

### ISS-005 — Audit emission only at success
First-pass emitted `mcp.tool_invoked` only on completed success. Resolved: §1 #13 + DEC-265 + feature-request-audit skill rule 26 + pair `started`+`completed` rows; AC #23.

### ISS-006 — Module timeout unbounded
First-pass had no timeout on dispatch. Resolved: §1 #17 + 30s timeout + `-32004 module_unreachable` with `reason: "timeout"`; AC #20.

### ISS-007 — Rate limit absent
First-pass had no rate-limiting. Resolved: §1 #12 + DEC-268 + sliding-window per (tenant, tool); AC #22.

### ISS-008 — Cursor pagination missing on tools/list
First-pass returned full catalog. Resolved: §1 #6 + #15 + opaque cursor + 100/page; AC #15.

### ISS-009 — `arguments_sha256` not raw arguments in memory row
First-pass stored full arguments in memory (PII risk). Resolved: §1 #13 + SHA-256 hash; PII concerns addressed.

### ISS-010 — Capabilities response over-advertised
First-pass declared `sampling` and `roots`. Resolved: §1 #5 + DEC-266 + closed capabilities {tools, prompts, resources, logging}.

### ISS-011 — Graceful shutdown not specified
First-pass abruptly closed connections on SIGTERM. Resolved: §1 #27 + 10s drain + 503 on new requests + SSE close.

## §3 — Resolution

All 11 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (MCP 2025-11-25 spec × JSON-RPC 2.0 × Streamable HTTP × Mcp-Session-Id × capabilities negotiation × federated registry × cursor pagination × tool annotations × JWT + scope × rate limit × destructive-op stub × audit pair × W3C propagation × graceful shutdown × /healthz), not by line targets.

---

*End of FR-MCP-001 audit.*
