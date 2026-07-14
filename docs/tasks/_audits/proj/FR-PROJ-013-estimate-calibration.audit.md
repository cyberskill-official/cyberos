---
task_id: TASK-PROJ-013
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 16
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..016 added)
---

## §1 — Verdict summary

TASK-PROJ-013 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 20 §1 clauses (snapshot schema, nightly cron, Bayesian compute, threshold 3, append-only, audit, REST, no-auto-apply, RLS, metrics, outlier filtering, CI bounds, engagement opt-out, data_points_summary, new-hire bootstrap, team-level calibration, drift alert at 30%, backfill, multiplier-applied count, acceptance rate). 16 §2 rationale paragraphs. §3 contains: schema with all fields, DataPoint + Posterior + compute_posterior + nightly job. 24 ACs. §10 lists 28 failure rows. §11 lists 25 implementation notes covering half-life calibration, exp-decay math, weighted variance handling, outlier threshold empirical choice, CI computation method, bootstrap fallback chain, drift alert calibration.

## §2 — Findings (all resolved)

### ISS-001 — Old data swamping
Simple averaging swamped by ancient data. Resolved: §1 #3 + DEC-341 exp-decay 90-day half-life.

### ISS-002 — Aggregate vs per-cell
Team-aggregate hides individual bias. Resolved: §1 #1 + DEC-340 per-member per-task-class.

### ISS-003 — Auto-apply hazard
Multipliers without context = wrong. Resolved: §1 #8 advisory only; AC #9.

### ISS-004 — Min sample
< 3 → posterior is prior. Resolved: §1 #4 threshold; AC #3.

### ISS-005 — Snapshot uniqueness
Re-run same day risks duplicate. Resolved: §1 #5 + UNIQUE constraint + ON CONFLICT DO NOTHING; AC #4.

### ISS-006 — Trend visibility
Single snapshot loses trend. Resolved: §1 #5 + #7 append-only + history endpoint.

### ISS-007 — Outlier-prone posterior (strict-redo pass)
Data-entry errors (10× typos) skew mean heavily. Resolved: §1 #11 + outlier filter + AC #15.

### ISS-008 — Point estimate without uncertainty (strict-redo pass)
Operators couldn't see when recommendation was uncertain. Resolved: §1 #12 + 95% CI bounds + AC #16.

### ISS-009 — R&D engagements bias calibration (strict-redo pass)
Unpredictable scope work skews "regular" calibration. Resolved: §1 #13 + per-engagement opt-out + AC #17.

### ISS-010 — Snapshot opaque to operator (strict-redo pass)
Operators inspecting needed re-fetch for percentiles. Resolved: §1 #14 + data_points_summary + AC #18.

### ISS-011 — New hires get no recommendation (strict-redo pass)
< 3 data points → no snapshot; UI shows nothing. Resolved: §1 #15 + team-average bootstrap + AC #19.

### ISS-012 — Sprint planning needs team aggregate (strict-redo pass)
Per-member too granular. Resolved: §1 #16 + team-level endpoint + AC #20.

### ISS-013 — Rapid skill change invisible (strict-redo pass)
30%+ shifts went unnoticed. Resolved: §1 #17 + drift alert + AC #21.

### ISS-014 — No history for new tenants (strict-redo pass)
Mid-year onboarding had no trend. Resolved: §1 #18 + backfill CLI + AC #22.

### ISS-015 — Adoption invisible (strict-redo pass)
"Are operators using recommendations?" unmeasurable. Resolved: §1 #19 + applied count + AC #23.

### ISS-016 — Recommendation quality untrackable (strict-redo pass)
Even with adoption, recommendation may be wrong. Resolved: §1 #20 + acceptance rate metric + AC #24.

## §3 — Resolution

All 16 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (Bayesian compute × exp decay × per-cell × CI × outlier filter × bootstrap × team aggregate × drift × backfill × adoption metrics), not by line targets.

---

*End of TASK-PROJ-013 audit.*
