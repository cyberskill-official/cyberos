---
id: NFR-DOC-002
title: "DOC PAdES-LT timestamp freshness — restamp before any timestamp expiry"
module: DOC
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of PAdES-LT documents restamped at least 30 days before any contained timestamp expires"
owner: CLO-Legal
created: 2026-05-18
related_tasks: [TASK-DOC-011]
---

## §1 — Statement (BCP-14 normative)

1. Documents signed with PAdES-LT (Long-Term) **MUST** be restamped (PAdES-LTA extension) at least 30 days before any contained timestamp or CRL nears its validity end.
2. The restamp scheduler **MUST** scan persisted signed docs daily and queue restamping for any approaching expiry.
3. Restamps preserve the original signature integrity — restamp adds a new outer TSA timestamp over the existing document state.
4. Failed restamps **MUST** be retried with backoff; sustained failure (> 7 days before expiry) triggers sev-1.
5. The signing/restamping audit chain **MUST** be preserved indefinitely (regulatory).

## §2 — Why this constraint

PAdES-LT is the "still-verifiable-decades-from-now" signature format. The whole point is that the document remains legally valid even after the original certs expire — but only if you restamp before expiry. Without restamping, the signature loses long-term verifiability and the document is legally vulnerable. 30 days lead time is the safety margin; sev-1 escalation at 7 days is the panic line.

## §3 — Measurement

- Gauge `doc_pades_lt_days_until_restamp_needed{doc_id}` — min across all timestamps.
- Counter `doc_restamp_total{result=success|failed}`.
- Counter `doc_restamp_overdue_total` — must be 0.

## §4 — Verification

- Integration test (T) — sign + advance clock; assert restamp scheduled + executed.
- Daily prod scan + report.
- Annual external audit of restamp chain integrity.

## §5 — Failure handling

- Restamp failure → retry with backoff.
- < 7 days to expiry → sev-1; manual intervention.
- Restamp chain corruption → sev-1; halt; rebuild from audit chain.

---

*End of NFR-DOC-002.*
