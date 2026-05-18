---
id: NFR-EMAIL-007
title: "EMAIL DSAR export completeness — export MUST contain all subject's messages + metadata"
module: EMAIL
category: privacy
priority: MUST
verification: T
phase: P0
slo: "100% of DSAR exports include all related messages; reconciliation = 0 missing rows"
owner: CPO-Privacy
created: 2026-05-18
related_frs: [FR-EMAIL-011]
---

## §1 — Statement (BCP-14 normative)

1. A DSAR export for a data subject **MUST** include all email messages where the subject appears in `From`, `To`, `Cc`, `Bcc`, or as referenced in body text via PII matching.
2. The export **MUST** include message metadata: headers, timestamps, thread IDs, attachments, labels, CaMeL classifications.
3. The export **MUST** be signed (PAdES or equivalent) so the recipient can verify integrity.
4. Reconciliation: re-running the same DSAR (same subject, same time window) **MUST** produce identical message list (modulo new mail between runs).
5. Exports **MUST** complete within 72 hours of request submission (GDPR Art. 12 §3 baseline; stricter for some VN regulations).

## §2 — Why this constraint

DSAR (Data Subject Access Request) is a regulatory promise; incomplete responses are legal liability. The "all five envelope positions + body PII match" rule is the coverage floor. Signed export prevents tampering claims. The 72h SLA matches the strictest applicable regulatory window. Reconciliation between runs is the proof of completeness.

## §3 — Measurement

- Counter `email_dsar_export_request_total{result=success|partial|failed}`.
- Histogram `email_dsar_export_completion_hours`.
- Reconciliation script: rerun every export within 1 hour of original; assert match.

## §4 — Verification

- Integration test (T) — fixture subject with mail in all positions + body refs; assert export contains all.
- Property test (T) — random message sets; assert reconciliation.
- Quarterly drill — operator-driven DSAR; assert SLA + completeness.

## §5 — Failure handling

- Completion > 72h → sev-2; regulatory window risk; CPO + CLO engaged.
- Reconciliation mismatch → sev-1; export trust broken.
- Partial export shipped to subject → sev-2; remediate + supplement.

---

*End of NFR-EMAIL-007.*
