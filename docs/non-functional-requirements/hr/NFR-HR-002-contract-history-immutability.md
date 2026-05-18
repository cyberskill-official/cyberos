---
id: NFR-HR-002
title: "HR contract-history immutability — contract changes MUST create new versions, never edit old"
module: HR
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of contract changes create new versioned row; 0 in-place edits"
owner: CHRO
created: 2026-05-18
related_frs: [FR-HR-002]
---

## §1 — Statement (BCP-14 normative)

1. Contract changes (promotion, comp change, role change) **MUST** create a new contract row with `effective_from` timestamp; the prior row is closed with `effective_to`.
2. The contract table is append-only — no UPDATE/DELETE.
3. Contract history per member **MUST** be retrievable as a chronological timeline.
4. Overlapping contract periods are forbidden; the closing/opening transition must be atomic.
5. Retention: contract history retained for member-life + 7 years (regulatory).

## §2 — Why this constraint

Contract history is the legal record of employment terms. Editing in place rewrites history — bad for disputes, bad for audits. The closed-and-new pattern creates a clean ledger. The no-overlap rule prevents periods where a member has two simultaneously-active contracts. The 7-year retention matches the typical statute-of-limitations for employment claims.

## §3 — Measurement

- Counter `hr_contract_in_place_edit_attempt_total` — must be 0.
- Counter `hr_contract_overlap_total` — must be 0.
- Retention scan.

## §4 — Verification

- Integration test (T) — change contract; assert new row + prior closed.
- Property test (T) — random changes; assert no overlap.
- Mutation test (T) — direct UPDATE attempt → blocked.

## §5 — Failure handling

- In-place edit attempt → block + sev-2.
- Overlap detected → sev-1; data integrity.
- Retention violation → sev-1; legal liability.

---

*End of NFR-HR-002.*
