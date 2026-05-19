---
id: NFR-MEMORY-004
title: "memory doctor invariant runtime — full invariant pass < 30s on 100k-row store"
module: memory
category: performance
priority: MUST
verification: T
phase: P0
slo: "cyberos-memory doctor full invariant pass completes < 30s on a 100k-row test store"
owner: CTO
created: 2026-05-18
related_frs: [FR-MEMORY-105, FR-MEMORY-102]
---

## §1 — Statement (BCP-14 normative)

1. The `cyberos-memory doctor` command **MUST** complete a full invariant scan in **< 30s** on a Layer-2 store of 100,000 rows. The scan includes: chain-anchor verification (NFR-MEMORY-002), cursor consistency (NFR-MEMORY-003), per-tenant RLS sanity, and PII pre-ingest detection (NFR-MEMORY-007).
2. The runtime budget **MUST** scale linearly: a 1M-row store completes in < 300s; a 10M-row store in < 3000s.
3. Doctor **MUST** emit a JSON report `docs/audits/memory-doctor/YYYY-MM-DD.json` with per-invariant pass/fail count and any quarantined rows.
4. Doctor **MUST** be safe to run concurrent with normal traffic — no DB locks beyond standard read serialisation.
5. Doctor failures (any invariant violation) **MUST** emit a memory audit row `memory.doctor.scan` with the result; CI invokes doctor weekly on the staging store and fails the pipeline on any violation.

## §2 — Why this constraint

Doctor is the load-bearing platform-health primitive — it's the "everything okay?" command operators run before major releases and after incidents. A 30s budget at 100k rows means doctor is usable during incident response (operator runs it, doesn't wait 10 minutes). The linear-scale guarantee means doctor remains usable as the platform grows. The concurrency rule means doctor doesn't itself cause incidents by locking the store.

## §3 — Measurement

- Histogram `memory_doctor_scan_seconds{result, rows_n}` per run.
- Counter `memory_doctor_invariant_violations_total{invariant_name}` from each report.
- The JSON report archive at `docs/audits/memory-doctor/` provides a quarter-over-quarter trend.

## §4 — Verification

- Benchmark `services/memory/benches/doctor_runtime.rs` (T) — runs doctor on a seeded 100k-row store; asserts < 30s.
- CI gate (T) — weekly cron runs `cyberos-memory doctor` against the staging store; PR/release blocks on violations.

## §5 — Failure handling

- Doctor exceeds 30s on 100k → sev-3; benchmark to identify which invariant is slow.
- Doctor reports invariant violation → sev-2 (or sev-1 for chain-anchor mismatch); follow per-invariant runbook.
- Doctor cannot complete due to DB error → sev-2; investigate DB health independent of doctor.

---

*End of NFR-MEMORY-004.*
