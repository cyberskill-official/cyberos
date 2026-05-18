---
id: NFR-MCP-002
title: "MCP tool annotation gating — destructive ops require explicit human approval annotation"
module: MCP
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of tools annotated destructive=true gate behind human approval; 0 silent destructions"
owner: CTO
created: 2026-05-18
related_frs: [FR-MCP-006]
---

## §1 — Statement (BCP-14 normative)

1. Every MCP tool exposed by CyberOS **MUST** declare annotations: `{destructive: bool, readOnly: bool, openWorld: bool}`.
2. Tools with `destructive: true` **MUST NOT** execute without an explicit human-approval token presented by the client (per the MCP elicitation protocol or out-of-band approval).
3. The host shell (Claude Desktop, IDE, etc.) **MUST** be informed of the destructive annotation in the `tools/list` response so it can show a confirmation UI.
4. Approval tokens **MUST** be single-use and short-lived (TTL ≤ 60s).
5. An attempted destructive call without valid approval **MUST** return JSON-RPC error `-32000` with `data.reason = "approval_required"` and **MUST** be audited.

## §2 — Why this constraint

MCP tools can delete data, transfer money, send emails, change configurations. Without explicit gating, an LLM hallucinating a tool call could cause real-world harm. The annotation-driven approval flow makes the human-in-the-loop step structural, not optional. The single-use + short-TTL token prevents replay attacks. Audit of denied calls surfaces patterns of attempted abuse.

## §3 — Measurement

- Counter `mcp_destructive_call_attempt_total{tool, approved}` — surfaces approval rates.
- Counter `mcp_approval_token_replay_total` — must be 0 (replay attempt indicates compromised token).
- Audit row for every destructive call (approved or denied).

## §4 — Verification

- Unit test `modules/mcp/tests/test_destructive_gate.py` (T) — call destructive tool without token → reject; with valid token → admit.
- Integration test (T) — token replay; assert second call rejected.
- CI gate — every tool in the catalog has declared annotations.

## §5 — Failure handling

- Destructive call without approval → JSON-RPC error + audit row + counter increment.
- Token replay → sev-3; token issuer may be compromised; rotate.
- Tool missing annotations → CI block.

---

*End of NFR-MCP-002.*
