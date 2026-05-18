---
id: NFR-RES-003
title: "RES OT consent capture — VN overtime MUST require + persist worker consent"
module: RES
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of VN overtime hours carry recorded worker consent + supervisor signoff"
owner: CHRO
created: 2026-05-18
related_frs: [FR-RES-005]
---

## §1 — Statement (BCP-14 normative)

1. VN-resident workers' overtime **MUST NOT** be allocated without explicit, recorded consent of the worker, per the VN labour code OT cap (`FR-RES-005`).
2. The consent record carries `{worker_id, period, requested_hours, consented_at, supervisor_id, supervisor_signed_at, justification}`.
3. Hard-cap enforcement: cumulative monthly OT > legal cap (VN: 40h/month, 200h/year, varies by industry) **MUST** be blocked at allocation time.
4. Consent **MUST** be electronic-signed; verbal/informal consent is insufficient.
5. Annual report to CHRO + CLO-Legal: total OT, per-worker distribution, cap utilization.

## §2 — Why this constraint

VN labour law strictly regulates overtime: employer-side coercion, uncompensated OT, cap violations are major liabilities. The explicit-consent + signoff pattern provides legal-grade record. Hard-cap enforcement at allocation time prevents the problem at source. Annual reporting closes the feedback loop with CHRO + Legal.

## §3 — Measurement

- Counter `res_ot_consent_missing_total` — must be 0.
- Counter `res_ot_cap_block_total` — surfaces near-cap workers.
- Per-worker monthly cumulative gauge.

## §4 — Verification

- Integration test (T) — OT alloc without consent → block.
- Integration test (T) — cap exceeded → block.
- Annual report drill.

## §5 — Failure handling

- Missing consent attempt → block.
- Cap violation observed → sev-1; HR + Legal investigation.
- Annual report drift → CHRO retrospective.

---

*End of NFR-RES-003.*
