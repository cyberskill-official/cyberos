---
id: NFR-CUO-001
title: "CUO catalog scan latency — full persona+workflow scan completes < 3s"
module: CUO
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 3s for full cuo/ + skill/ filesystem catalog scan + validation"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-CUO-101, TASK-CUO-104]
---

## §1 — Statement (BCP-14 normative)

1. The CUO supervisor's filesystem catalog scan (`modules/cuo/cuo/catalog.py::scan_catalog()`) **MUST** complete a full pass over `cuo/<persona>/<workflow>.md` + `skill/public/<skill>/SKILL.md` at **p95 < 3s** for a catalog of up to 500 personas and 5000 workflows.
2. The scan **MUST** parse + validate frontmatter for every file in the catalog; partial scans are not acceptable.
3. Scan results **MUST** be cached in-process and invalidated only on explicit reload or filesystem-change signal.
4. The scan **MUST** be idempotent — re-running with no filesystem changes returns identical catalog object.
5. Scan failure (any file unparseable) **MUST** raise a structured error with `{file, line, reason}` — the supervisor does not silently skip broken files.

## §2 — Why this constraint

The CUO supervisor scans the catalog at startup + on every `cyberos-cuo route` invocation when run without warm cache. A slow scan means a slow CLI (frustrating for operators) or a slow Phase-3 LLM round-trip. 3s for 500 personas + 5000 workflows is generous — current catalog is 47 personas + 221 workflows + 104 skill pairs, so we have ample headroom. The fail-fast on unparseable files prevents the supervisor from running with a silently-broken catalog and routing requests to nonexistent skills.

## §3 — Measurement

- Histogram `cuo_catalog_scan_latency_seconds{stage=walk|parse|validate}` — surfaces which sub-step dominates.
- Counter `cuo_catalog_scan_error_total{file, reason}`.
- Gauge `cuo_catalog_entry_count{kind=persona|workflow|skill}` — tracks catalog growth.

## §4 — Verification

- Benchmark `modules/cuo/tests/test_catalog_scan_perf.py` (T) — synthetic 500-persona catalog; asserts p95 < 3s.
- Smoke test (T) — current production catalog must scan in < 1.5s on the CI runner.
- Property test (T) — random catalog generator; assert scan is idempotent.

## §5 — Failure handling

- p95 > 3s → sev-3; catalog has grown beyond benchmark; profile + optimize.
- Scan error on a single file → CLI returns non-zero exit; operator fixes the file.
- Cache invalidation storm (full rescan > 10/min) → sev-3; investigate the filesystem-change signal source.

---

*End of NFR-CUO-001.*
