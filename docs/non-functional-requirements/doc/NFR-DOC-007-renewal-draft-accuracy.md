---
id: NFR-DOC-007
title: "DOC renewal-draft accuracy — generated renewal MUST preserve original terms unless flagged"
module: DOC
category: reliability
priority: MUST
verification: T
phase: P1
slo: "100% of renewal drafts preserve original terms (parties, scope, term length) modulo explicit deltas"
owner: CLO-Legal
created: 2026-05-18
related_tasks: [TASK-DOC-009]
---

## §1 — Statement (BCP-14 normative)

1. The renewal-proposal CUO workflow (`TASK-DOC-009`) **MUST** generate a renewal draft that preserves the original document's core terms unless the workflow explicitly flags a delta.
2. Preserved terms include: parties, scope of services, governing law, dispute resolution clause, payment terms.
3. Delta proposals (e.g., updated term length, updated pricing) **MUST** be highlighted in the draft with `old → new` redline.
4. The draft **MUST NOT** be auto-sent — it requires CLO-Legal review + approval before circulation.
5. Renewal drafts **MUST** be tied to the original document via `renews_doc: <doc_id>` metadata for traceability.

## §2 — Why this constraint

Renewal drafts that silently mutate terms produce surprises at signing — bad for trust and possibly bad legally. The preserve-with-explicit-delta rule keeps the LLM-assisted draft predictable. The mandatory CLO review preserves human accountability for legal artifacts. The traceability metadata enables audit chains.

## §3 — Measurement

- Sample audit: CLO-Legal reviews 10% of generated drafts for unexpected mutations; counter `doc_renewal_unexpected_mutation_total`.
- Counter `doc_renewal_auto_sent_total` — must be 0 (auto-send forbidden).
- Counter `doc_renewal_traceability_missing_total` — must be 0.

## §4 — Verification

- Integration test (T) — fixture original + renewal generation; assert preserved + flagged.
- Sample audit (T, quarterly) — manual review.
- CI gate (T) — every renewal carries `renews_doc:`.

## §5 — Failure handling

- Unexpected mutation found → sev-2; LLM prompt or template bug.
- Auto-send detected → sev-1; safety control bypassed.
- Missing traceability → block draft generation; fix workflow.

---

*End of NFR-DOC-007.*
