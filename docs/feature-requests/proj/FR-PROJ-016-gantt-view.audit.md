---
fr_id: FR-PROJ-016
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 16
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..016 added)
---

## §1 — Verdict summary

FR-PROJ-016 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 22 §1 clauses (dep table, cycle detection, SVG arrows, critical path, memoise, CRUD, audit, kbd D, parent rollup, RLS, a11y, metrics, slack visualisation, critical-only filter, validate-graph CLI, near-cycle detection, forward-compatible kind, PDF export, earliest-start/latest-finish, batch dependency CRUD, parent completion %, Shift+D undo). 17 §2 rationale paragraphs. §3 contains: migration + Rust add_dependency with BFS cycle detect + TS computeCriticalPath topological-sort + longest-path. 27 ACs. §10 lists 30 failure rows. §11 lists 27 implementation notes covering slack ghost rendering, validate-graph output format, near-cycle dedup, PDF puppeteer mechanics, batch cycle union check, parent completion live updates, ghost CSS-only rendering for perf.

## §2 — Findings (all resolved)

### ISS-001 — Cycle detection algorithm
Without BFS-check, infinite loop in critical path. Resolved: §1 #2 + DAG check; AC #3 #4.

### ISS-002 — Critical path memoisation
Naive recompute = perf cliff. Resolved: §1 #5 + §3 graph hash memo key; AC #10.

### ISS-003 — Dependency kind scope
4 kinds × edge config = complex. Resolved: §1 finish_to_start only; DEC-371 slice 3.

### ISS-004 — Parent rollup
Without it, epics show no aggregate range. Resolved: §1 #9 min/max children.

### ISS-005 — Self-edge handling
PK doesn't catch (a,a). Resolved: §1 + CHECK constraint + Rust check; AC #2.

### ISS-006 — Recompute trigger frequency
Every render = thrashing. Resolved: §1 #5 + §11 debounce 200ms + memo invalidation.

### ISS-007 — Slack invisible (strict-redo pass)
Non-critical issues' flexibility was hidden. Resolved: §1 #13 + ghost extension + AC #19.

### ISS-008 — Critical-only view missing (strict-redo pass)
Operators wanting bottleneck focus had no filter. Resolved: §1 #14 + URL param + AC #20.

### ISS-009 — No CLI graph health (strict-redo pass)
Operators auditing graph health needed CLI. Resolved: §1 #15 + validate-graph + AC #21.

### ISS-010 — Near-cycle invisibility (strict-redo pass)
Approaching cycles offered no warning. Resolved: §1 #16 + detection + SEV-3 + AC #22.

### ISS-011 — Schema migration locked to one dep kind (strict-redo pass)
Adding kinds later requires DDL. Resolved: §1 #17 + forward-compatible CHECK enum + audit note.

### ISS-012 — No portable export (strict-redo pass)
Stakeholder reports need PDF; spec had none. Resolved: §1 #18 + PDF export + AC #23.

### ISS-013 — Reschedule bounds invisible (strict-redo pass)
Operator moving critical-path issue had no slack info. Resolved: §1 #19 + earliest-start/latest-finish annotations + AC #24.

### ISS-014 — Per-edge POST slow at scale (strict-redo pass)
Bulk imports painfully slow. Resolved: §1 #20 + batch endpoint + AC #25.

### ISS-015 — Parent progress signal missing (strict-redo pass)
Epic-level completion invisible at a glance. Resolved: §1 #21 + parent completion % + AC #26.

### ISS-016 — Accidental dep undo workflow missing (strict-redo pass)
Operator adding wrong dep had no quick fix. Resolved: §1 #22 + Shift+D undo + AC #27.

## §3 — Resolution

All 16 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (dep CRUD × cycle detect × critical path × slack × parent rollup × kbd × validate CLI × near-cycle × PDF export × batch × completion % × undo), not by line targets.

---

*End of FR-PROJ-016 audit.*
