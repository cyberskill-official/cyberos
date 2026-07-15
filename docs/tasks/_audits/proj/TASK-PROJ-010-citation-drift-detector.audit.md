---
task_id: TASK-PROJ-010
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 17
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..017 added)
---

## §1 — Verdict summary

TASK-PROJ-010 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 22 §1 clauses (nightly cron, on-demand CLI, 3 drift kinds, audit row, drift_state table, sev-2/sev-3 routing, no auto-remove, REST, metrics, deterministic, 5min latency, delta sweeps, drift_status column, assignee notifications, suppress workflow, severity hierarchy, per-tenant config override, suggested remediation, dry-run, drift_remediated audit, drift_trend metric, 1-min small-tenant fast path). 17 §2 rationale paragraphs. §3 contains migration + DriftKind enum + sweep_tenant with per-link detect + drift_state upsert + per-severity alert routing. 27 ACs. §10 lists 32 failure rows. §11 lists 25 implementation notes covering delta sweep mechanics, drift_status materialisation, assignee notification dedup, suppression table semantics, severity-hierarchy implementation, suggested-paths bounded, dry-run handler wiring, trend window storage, fast-path threshold, real-time-vs-batch rationale.

## §2 — Findings (all resolved)

### ISS-001 — Inline vs batch
Inline = expensive. Resolved: §1 #1 + DEC-310 nightly batch.

### ISS-002 — Auto-remediate vs flag
Auto = risky. Resolved: §1 #7 + DEC-310 flag-only.

### ISS-003 — Drift kind coverage
Without enumeration, drift checks are ad-hoc. Resolved: §1 #3 + DEC-311 3 kinds; AC #1 #2 #3.

### ISS-004 — Notification severity threshold
Without threshold, every drift = sev-1 spam. Resolved: §1 #6 + DEC-312 sev-2 at ≥ 10; AC #8 #9.

### ISS-005 — Determinism
Non-deterministic sweep defeats verification rerun. Resolved: §1 #10 + AC #5.

### ISS-006 — Latency budget
Without budget, 100K-link tenants silently degrade. Resolved: §1 #11 + AC #15.

### ISS-007 — Re-check overhead unavoidable (strict-redo pass)
Operators iterating fixes re-check entire link set every time. Resolved: §1 #12 + delta sweep + AC #17.

### ISS-008 — UI badge rendering = re-sweep (strict-redo pass)
Without materialised status, every UI render = expensive sweep. Resolved: §1 #13 + drift_status column + AC #18.

### ISS-009 — Admin-only notification misses operator (strict-redo pass)
The person who can fix is the assignee; admins are summary-only. Resolved: §1 #14 + per-assignee notification via CUO + AC #19.

### ISS-010 — No way to silence known drift (strict-redo pass)
Drifts that are accepted (archived intentionally) re-alert forever. Resolved: §1 #15 + suppress workflow + 90d expiry + AC #20.

### ISS-011 — All drift kinds equally severe (strict-redo pass)
TargetMissing >> TargetSuperseded in severity; flat threshold mis-prioritises. Resolved: §1 #16 + severity hierarchy + AC #21.

### ISS-012 — One-size-fits-all config (strict-redo pass)
Tenants have different cadence/threshold needs. Resolved: §1 #17 + per-tenant overrides + AC #22.

### ISS-013 — Operators triage without guidance (strict-redo pass)
"Memory missing — what should I do?" had no actionable answer. Resolved: §1 #18 + suggested_paths via TASK-MEMORY-108 fuzzy + AC #23.

### ISS-014 — No safe-preview sweep (strict-redo pass)
Operator running ad-hoc sweep generated alerts + audit spam. Resolved: §1 #19 + dry-run + AC #24.

### ISS-015 — Remediation untrackable (strict-redo pass)
"Did the team respond to drifts?" had no metric. Resolved: §1 #20 + drift_remediated audit + AC #25.

### ISS-016 — Single-day count is noisy (strict-redo pass)
Trend signal needed for capacity planning. Resolved: §1 #21 + 7-day trend + AC #26.

### ISS-017 — Small tenants wait same as large (strict-redo pass)
1K-link tenant doesn't need 5-min sweep budget. Resolved: §1 #22 + fast path + AC #27.

## §3 — Resolution

All 17 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (3 drift kinds × suppress workflow × severity hierarchy × delta sweeps × dry-run × assignee notification × remediation suggestions × trend metric × per-tenant config × fast path), not by line targets.

---

*End of TASK-PROJ-010 audit.*
