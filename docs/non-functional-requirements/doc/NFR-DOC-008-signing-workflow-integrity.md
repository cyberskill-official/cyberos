---
id: NFR-DOC-008
title: "DOC signing workflow integrity — multi-party sign MUST complete in declared order"
module: DOC
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of sequential-signing workflows complete in declared signer order; 0 out-of-order signatures"
owner: CTO
created: 2026-05-18
related_frs: [FR-DOC-005]
---

## §1 — Statement (BCP-14 normative)

1. Multi-party signing flows declared as `sequence: ordered` **MUST** require signers to sign in declared order; signer N cannot sign before signers 1..N-1 have completed.
2. Out-of-order sign attempts **MUST** return `E_ORDER_VIOLATION` with `data.expected_signer = <id>`.
3. Parallel flows declared `sequence: parallel` permit any order; each signer signs once.
4. Signed bundles **MUST** be hashed at each stage; the next signer signs over the prior signed bundle (signature on signature chain).
5. Workflow timeouts **MUST** be configurable per workflow (default 7 days); timeouts trigger workflow expiry + notification.

## §2 — Why this constraint

Multi-party sign order matters legally: a contract counter-signed before the primary signs has different legal weight. The order enforcement prevents UI mistakes from producing invalid sequences. The signature-on-signature chain makes tampering detectable — flipping signer 2 and 3 changes the hash chain.

## §3 — Measurement

- Counter `doc_sign_order_violation_total` — must be 0.
- Histogram `doc_signing_workflow_duration_days{kind=ordered|parallel}`.
- Counter `doc_signing_workflow_timeout_total`.

## §4 — Verification

- Integration test (T) — ordered workflow; signer 2 attempts first → reject.
- Integration test (T) — parallel workflow; any order accepted.
- Property test (T) — random multi-party workflows; assert invariants.

## §5 — Failure handling

- Order violation → block + audit; user re-routed.
- Workflow timeout → expire + notify originator.
- Hash chain corruption → sev-1; sign chain broken.

---

*End of NFR-DOC-008.*
