---
id: NFR-MCP-001
title: "MCP protocol compliance — runtime MUST conform to spec 2025-11-25"
module: MCP
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% pass on official MCP conformance suite for spec version 2025-11-25"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-MCP-001, TASK-MCP-002]
---

## §1 — Statement (BCP-14 normative)

1. The CyberOS MCP runtime **MUST** implement the JSON-RPC 2.0 message envelope, capability negotiation, and tool/resource/prompt primitives per the MCP spec version **2025-11-25**.
2. Spec version **MUST** be advertised in the server's `initialize` response (`serverInfo.protocolVersion = "2025-11-25"`).
3. A client requesting a different protocol version **MUST** receive a `protocolVersion` mismatch in the `initialize` response and either negotiate down or fail cleanly — never silently downgrade behaviour.
4. Spec changes (new versions published by anthropics/modelcontextprotocol) **MUST** be evaluated within 30 days; adoption (full or rejection with rationale) **MUST** be recorded in `modules/mcp/SPEC-VERSION-LOG.md`.
5. The runtime **MUST** pass the official MCP conformance test suite for the declared spec version; any failure blocks release.

## §2 — Why this constraint

MCP is an emerging standard with active versioning. Drift from spec means our servers don't interop with mainstream MCP clients (Claude Desktop, IDE plugins, third-party tools). The 30-day evaluation window keeps the platform in step with upstream without committing to immediate adoption of every alpha change. The spec-version-log preserves the audit trail of "we knew about version X, we chose A or B."

## §3 — Measurement

- CI: full conformance suite pass/fail per spec version.
- Counter `mcp_protocol_version_mismatch_total{client_version, server_version}` — surfaces real-world client diversity.
- Spec-version-log review every quarter.

## §4 — Verification

- CI gate (T) — every PR runs the official MCP conformance suite; failure blocks merge.
- Integration test (T) — clients claiming older/newer versions get correct negotiation responses.
- Quarterly audit — review SPEC-VERSION-LOG.md against upstream commits.

## §5 — Failure handling

- Conformance regression → CI block until fixed.
- Real-world mismatch counter rising → sev-3; product brief on whether to broaden compat range.
- Quarterly audit finds undeclared spec drift → sev-3; remediate within 7 days.

---

*End of NFR-MCP-001.*
