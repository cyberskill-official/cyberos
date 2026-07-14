---
task_id: TASK-TIME-007
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands VN Labour Code Art. 107 OT cap enforcement on top of TASK-TIME-001. 410 lines (focused regulatory single-purpose FR; sanctioned per task-audit skill §0 narrow-surface exception), 13 §1 clauses, 20 ACs, 5 tests, 15 failure modes, 10 notes. 1 migration, 2 endpoints, 3 memory audit kinds.

6 issues resolved.

## §2 — Findings (all resolved)

### ISS-001 — Concurrent race on aggregator row
§11.6 — SELECT FOR UPDATE locks per-day row before write.

### ISS-002 — Year boundary timezone
§11.3 — tenant timezone (default Asia/Ho_Chi_Minh for VN).

### ISS-003 — Approval expiry handling
§11.7 — expired falls back to 'standard' tier silently + warning.

### ISS-004 — Bulk import bypass risk
§11.8 — CI lint enforces invocation; sev-2 alert if missing.

### ISS-005 — Daily vs weekly vs monthly check order
§11.4 — most-granular first (daily → weekly → monthly → yearly).

### ISS-006 — Tracking desync detection
§11.9 — nightly reconciliation job with sev-2 drift alert.

## §3 — Resolution

All 6 mechanical concerns addressed. Single-purpose regulatory FR with focused narrow surface — task-audit skill §0 narrow-FR exception applies.

**Score = 10/10.**

---

*End of TASK-TIME-007 audit.*
