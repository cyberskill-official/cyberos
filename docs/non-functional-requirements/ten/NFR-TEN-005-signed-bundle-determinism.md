---
id: NFR-TEN-005
title: "TEN signed bundle determinism — same tenant export MUST produce same hash"
module: TEN
category: reliability
priority: MUST
verification: T
phase: P0
slo: "Re-running export with same parameters produces byte-identical bundle hash"
owner: CTO
created: 2026-05-18
related_frs: [FR-TEN-105]
---

## §1 — Statement (BCP-14 normative)

1. The signed-bundle export (`FR-TEN-105`) **MUST** be deterministic — re-running with the same `{tenant_id, export_at}` produces byte-identical output (same SHA-256).
2. Determinism requires: sorted iteration order, canonical JSON formatting, no embedded wall-clock timestamps in the payload (only the declared `export_at` timestamp).
3. The bundle hash **MUST** be signed (PAdES) and the signature **MUST** be verifiable for ≥ 10 years (PAdES-LT).
4. The bundle **MUST** include a manifest of all included objects + their individual hashes for verification.
5. Bundle export latency for a typical tenant (10GB data) **MUST** complete within 4 hours; larger scale linearly.

## §2 — Why this constraint

The signed bundle is the legal proof of "what data the platform held for this tenant on this date." Determinism is what makes the hash + signature meaningful — non-deterministic exports could produce different hashes for the same data, defeating verification. The 10-year signature life is the regulatory floor for export claims. Per-object hashes make partial verification possible.

## §3 — Measurement

- Counter `ten_bundle_export_hash_mismatch_total` — must be 0 on re-runs.
- Histogram `ten_bundle_export_duration_hours{tenant_size_gb_bucket}`.
- Bundle signature verification on every produced bundle.

## §4 — Verification

- Integration test (T) — export tenant twice; assert hashes match.
- Property test (T) — random tenants; assert determinism.
- Long-term signature test (T) — restamp + verify after simulated 10 years.

## §5 — Failure handling

- Hash mismatch → sev-2; determinism broken; investigate.
- Export > 4h for normal size → sev-3; investigate worker capacity.
- Signature invalid → sev-1; bundle cannot serve as legal proof.

---

*End of NFR-TEN-005.*
