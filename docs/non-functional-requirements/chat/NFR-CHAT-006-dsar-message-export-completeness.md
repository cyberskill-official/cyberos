---
id: NFR-CHAT-006
title: "CHAT DSAR message export completeness — every message subject authored OR received + chain proof"
module: CHAT
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of messages where subject is author OR recipient are included; memory chain proof attached"
owner: CSO
created: 2026-05-18
related_frs: [FR-CHAT-012]
---

## §1 — Statement (BCP-14 normative)

1. A CHAT DSAR (Data Subject Access Request) export for subject `S` **MUST** include every message in the platform where `S` is either the author OR a member of the channel (recipient) at the time the message was posted.
2. The export **MUST NOT** include messages from channels where `S` joined after the message — only messages `S` had access to at the time.
3. The export **MUST** carry a memory chain proof: a Merkle inclusion proof showing the exported messages are the complete set per the platform's audit chain (NFR-MEMORY-006 STH inclusion).
4. The export bundle **MUST** be deterministic per NFR-MEMORY-005 semantics — two exports of the same subject's data on the same chain state produce byte-identical bundles.
5. Export must complete within 24 hours of request per the platform's DSAR SLO (compliance NFR).

## §2 — Why this constraint

DSAR completeness is a GDPR Art. 15 / PDPL Art. 14 hard requirement — incomplete exports are grounds for regulatory fine. The "author OR recipient at the time" rule is the canonical access scope — subjects have a right to data about them, not data they could now access via channel rejoin. The chain proof differentiates the platform from competitors: regulators can verify cryptographically that the export is the complete set, not a redacted subset. The 24h SLO is the platform's contractual response time.

## §3 — Measurement

- Counter `chat_dsar_exports_total{result}` per request.
- Counter `chat_dsar_messages_exported_total` summed across requests.
- memory audit row `chat.dsar.exported` per export with `{subject_id, message_count, chain_root, completed_at}`.
- Sev-1 alarm on any export > 24h or any audit row missing chain_root.

## §4 — Verification

- Integration test `services/chat-plugins/dsar-export/tests/completeness_test.rs` (T) — seeds 1000 messages across channels + memberships; asserts subject S's export contains exactly the qualifying subset and no more.
- Chain-proof test (T) — verifies the Merkle inclusion proof against the platform's STH chain.
- Determinism test (T) — exports same subject twice; asserts bundle hashes match.

## §5 — Failure handling

- Export missing messages (false negative) → sev-1 compliance; immediate re-export; root cause investigation.
- Export contains messages outside access scope (false positive) → sev-1; the leak is itself a breach; CSO notification.
- Export > 24h SLO → sev-2; manual escalation; tenant + subject notified of delay.

---

*End of NFR-CHAT-006.*
