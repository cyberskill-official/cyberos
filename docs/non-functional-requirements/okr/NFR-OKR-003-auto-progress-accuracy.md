---
id: NFR-OKR-003
title: "OKR auto-progress accuracy — batch-computed progress MUST match per-KR sample within ±2%"
module: OKR
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of auto-progress computations within ±2% of independent per-KR computation"
owner: CEO
created: 2026-05-18
related_frs: [FR-OKR-004]
---

## §1 — Statement (BCP-14 normative)

1. The nightly auto-progress batch (`FR-OKR-004`) **MUST** produce per-KR progress values within ±2% of an independent re-computation.
2. The batch **MUST** be idempotent — rerunning produces the same values (modulo source-data changes).
3. Per-KR computation errors **MUST** be isolated — one KR's data issue does not break the entire batch.
4. Progress changes vs prior day **MUST** be visible in the digest with delta highlighted.
5. The batch SLA: complete within 30 minutes for tenants up to 1000 KRs.

## §2 — Why this constraint

OKR progress is leadership's pulse on goal attainment. ±2% accuracy is the precision floor for trust. Idempotency lets us safely retry. Per-KR error isolation prevents one bad source killing the whole batch. The delta visibility makes the digest actionable.

## §3 — Measurement

- Daily reconciliation: batch values vs sample-recomputed; counter `okr_progress_delta_pct{kr}` histogrammed.
- Counter `okr_kr_batch_skip_total{reason}` — surfaces per-KR data issues.
- Histogram `okr_progress_batch_duration_seconds`.

## §4 — Verification

- Integration test (T) — fixture KRs; assert batch matches reference within 2%.
- Property test (T) — rerun batch; assert idempotent.
- Stress test (T) — 1000 KRs; assert ≤ 30 min.

## §5 — Failure handling

- Delta > 2% on a KR → flag KR; investigate.
- Batch duration > 30 min → sev-3; scale workers.
- Idempotency violation → sev-2; investigate.

---

*End of NFR-OKR-003.*
