---
id: NFR-OKR-004
title: "OKR KR check-in latency — manual check-in MUST persist within 2s"
module: OKR
category: performance
priority: SHOULD
verification: T
phase: P1
slo: "p95 < 2s from check-in submit to visible in the OKR dashboard"
owner: CEO
created: 2026-05-18
related_frs: [FR-OKR-005]
---

## §1 — Statement (BCP-14 normative)

1. Manual KR check-ins (`FR-OKR-005`) **MUST** persist + appear in the OKR dashboard within 2s p95.
2. Each check-in **MUST** capture: progress value, narrative (≥ 20 words encouraged but not required), confidence (1-5), submitted_at, submitter.
3. Check-ins **MUST** create an immutable history; corrections take the form of a new check-in.
4. Auto-reminders **MUST** fire weekly for KRs without check-in in 7+ days.
5. Check-in confidence over time **MUST** be charted to show trend.

## §2 — Why this constraint

Check-ins are the qualitative complement to auto-progress. 2s latency keeps the UX responsive. The narrative + confidence fields are the human judgment that numerical progress alone can't capture. Immutable history prevents "I said it would be fine!" revisionism. The trend chart turns confidence shifts into a leading indicator.

## §3 — Measurement

- Histogram `okr_checkin_persist_latency_ms`.
- Counter `okr_checkin_total{kr, confidence_bucket}`.
- Gauge `okr_kr_days_since_last_checkin{kr}`.

## §4 — Verification

- Integration test (T) — submit check-in; assert visible < 2s.
- Snapshot test (T) — confidence trend chart renders.
- Reminder test (T) — stale KR triggers weekly reminder.

## §5 — Failure handling

- Latency > 2s → sev-3; investigate.
- Stale KR > 14d → flag for objective owner.
- Mutation of historic check-in → sev-1; immutability broken.

---

*End of NFR-OKR-004.*
