---
id: TASK-PROJ-013
title: "Estimate calibration snapshot — per-member per-task-class nightly batch with Bayesian update and operator-visible accuracy trend"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PROJ
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-001, TASK-PROJ-002, TASK-PROJ-004, TASK-TIME-001, TASK-MEMORY-101]
depends_on: [TASK-PROJ-002]
blocks: [TASK-HR-008, TASK-LEARN-003]

source_pages:
  - website/docs/modules/proj.html#estimate-calibration
source_decisions:
  - DEC-340 (calibration is per-member per-task-class — different members have different estimation biases)
  - DEC-341 (Bayesian update — posterior accuracy reweights as new data arrives; old data exponentially decayed)
  - DEC-342 (snapshots are append-only nightly history; never UPDATE)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/migrations/0013_estimate_calibration.sql
  - services/proj-sync/src/calibration/mod.rs
  - services/proj-sync/src/calibration/bayes.rs
  - services/proj-sync/src/calibration/nightly.rs
  - services/proj/tests/audit_row_test.rs
modified_files:
  # cron schedule
  - services/proj-sync/src/main.rs
allowed_tools:
  - file_read: services/proj-sync/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test calibration
disallowed_tools:
  - UPDATE existing calibration snapshots (per DEC-342)
  - publish calibration multipliers without operator review (advisory only)

effort_hours: 6
subtasks:
  - "0.5h: 0013_estimate_calibration.sql migration"
  - "0.5h: mod.rs — CalibrationSnapshot struct"
  - "1.5h: bayes.rs — posterior mean + std-dev of (actual_hours / estimate); exp-decay weight by recency"
  - "1.0h: nightly.rs — sweep completed issues since last snapshot; group by (member, task_class); compute"
  - "0.5h: memory audit row 'proj.estimate_calibration_computed'"
  - "0.5h: REST endpoint GET /api/proj/calibration/:member_id/:task_class → latest snapshot"
  - "1.5h: calibration_test.rs — synthetic dataset; assert Bayesian update converges"
risk_if_skipped: "Estimates without calibration drift indefinitely — a member who consistently underestimates by 30% will keep doing so. With calibration, the operator sees 'Alice's bug-fix estimates are 28% optimistic over 90 days; consider × 1.3'. Without per-member, team aggregates mask individual biases. Without per-task-class, code-review estimates muddy bug-fix accuracy. Without exp-decay, old data dominates years later."
---

## §1 — Description (BCP-14 normative)

The calibration layer **MUST** compute nightly snapshots of estimate accuracy per (member, task_class). The contract:

1. **MUST** define `estimate_calibration_snapshots` table: `id UUID PK`, `member_id UUID`, `task_class TEXT`, `snapshot_date DATE`, `posterior_mean f64`, `posterior_std f64`, `sample_size i32`, `effective_sample_size f64` (after exp decay), `recommended_multiplier f64`, `created_at`, `tenant_id`. Composite key `(member_id, task_class, snapshot_date)`.
2. **MUST** schedule a nightly cron at 03:00 local time (after TASK-PROJ-010 drift sweep).
3. **MUST** compute per-cell as follows:
- Query all issues with `status = Done`, `cycle_id != NULL`, `assignee = member`, `task_class = class`, completed in last 365 days.
- For each issue: `ratio = actual_hours / estimate_hours` where actual = sum of billable time entries; estimate from issue.estimate field. Skip issues without estimate or actual.
- Apply exp-decay weight: `w_i = exp(-Δt / 90)` where Δt = days since completion.
- Posterior mean = `Σ(w_i × ratio_i) / Σ(w_i)`.
- Posterior std = weighted std of ratios.
- Effective sample size = `Σ(w_i)`.
- Recommended multiplier = posterior mean (rounded to 2 decimals).
4. **MUST** require minimum `effective_sample_size >= 3` for a cell to publish a snapshot; below threshold → no row written (insufficient data; UI shows "not enough data yet").
5. **MUST** persist snapshots APPEND-ONLY. Multiple snapshots per cell across days form a trend.
6. **MUST** emit `proj.estimate_calibration_computed` memory audit row per published snapshot.
7. **MUST** expose REST endpoints:
- `GET /api/proj/calibration/:member_id/:task_class` — latest snapshot.
- `GET /api/proj/calibration/:member_id/:task_class/history?since=YYYY-MM-DD` — trend.
- `GET /api/proj/calibration/team?engagement_id=...` — aggregate trends per team.
8. **MUST** NEVER auto-apply multipliers to issue estimates — calibration is advisory. UI surfaces the recommended multiplier when creating new estimates.
9. **MUST** RLS-enforce.
10. **MUST** emit OTel metrics:
- `proj_calibration_cells_computed_total` (counter).
- `proj_calibration_cells_below_threshold_total` (counter).
- `proj_calibration_posterior_mean_drift{member_bucket, task_class}` (histogram — magnitude of change vs prior snapshot).
11. **MUST** support outlier filtering: data points with `ratio > 5.0` OR `ratio < 0.2` are excluded from the posterior compute by default (catches clear data-entry errors). Tenant policy `cyberos_proj_tenant_settings.calibration_outlier_threshold` overrides bounds.
12. **MUST** include `posterior_ci_lower` + `posterior_ci_upper` (95% confidence interval) on each snapshot — UI shows "recommended × 1.28 (CI: 1.10–1.46)" so operators see uncertainty.
13. **MUST** support per-engagement opt-out: `cyberos_proj_engagement_settings.calibration_enabled = false` excludes that engagement's issues from data points. Used for unusual engagements (R&D experiments) that would bias calibration.
14. **MUST** include `data_points_summary` JSONB on snapshot: `{ratio_p25, ratio_p50, ratio_p75, oldest_data_days, newest_data_days}` for operator inspection without re-fetching raw data.
15. **MUST** support new-hire bootstrap: members with < 3 data points use the team-average multiplier (per task_class) as a stub recommendation; `bootstrap = true` flag on the response indicates it's a stub.
16. **MUST** compute team-level calibration: `GET /api/proj/calibration/team/:task_class` returns weighted average across all team members in the engagement.
17. **MUST** emit `proj.calibration_drift_alert` SEV-3 when posterior_mean shifts > 30% from prior snapshot (rapid skill change OR data corruption signal).
18. **MUST** support backfill: `cyberos calibration backfill --member <id> --task-class <c> --since <date>` recomputes snapshots for past dates from historical data. Audit row `proj.calibration_backfilled`.
19. **MUST** include `multiplier_applied_count` metric: how often operators actually used the recommended multiplier when creating new estimates. Adoption signal.
20. **MUST** include `multiplier_acceptance_rate` per-member metric (rolling 30 days): how often the operator's estimate was within ±10% of `original × recommended_multiplier`. Calibration-effectiveness signal.

---

## §2 — Why this design (rationale for humans)

**Why Bayesian / exp-decay (DEC-341)?** Simple averaging gives equal weight to 2-year-old and yesterday's data. Members' estimation skill changes (new hires improve; some plateau). Exp-decay with 90-day half-life means recent data dominates while history influences.

**Why per-member per-task-class (DEC-340)?** Individual estimation bias is the actionable signal. Team averages hide that some members are consistent and some erratic. Task-class separates "bug fixes I do at 1.2× estimate" from "feature work I do at 0.9× estimate" — different skill curves.

**Why min sample size 3 (§1 #4)?** Below 3 data points, posterior is dominated by prior — meaningless multiplier. 3 is the minimum useful sample for variance estimation.

**Why never auto-apply (§1 #8)?** Multipliers are mathematical artifacts; humans interpret context (vacation, illness, new tech stack). Auto-applying multipliers ignores context. Advisory UI = operator sees the trend, decides.

**Why per-snapshot history (DEC-342)?** "Is Alice getting better at estimating?" requires trend. Single-value-overwrite loses history. Append-only is the standard pattern.

**Why outlier filtering (§1 #11)?** Data-entry errors (typed 0.5h instead of 5h) skew posterior heavily; defaults of 5× / 0.2× catch the obvious cases without losing legitimate variance.

**Why CI in snapshot (§1 #12)?** Operators making decisions on point estimates without uncertainty miss "this is 1.28 ± 0.05" vs "1.28 ± 0.50." Different actions.

**Why per-engagement opt-out (§1 #13)?** R&D engagements have unpredictable scope; including them in calibration biases regular engineering work toward "always slower than estimated."

**Why data_points_summary (§1 #14)?** Operators inspecting a snapshot want quick stats without re-running queries; pre-computed summary in the snapshot row.

**Why new-hire bootstrap (§1 #15)?** Below 3 data points = no snapshot per §1 #4; but the UI still needs to show *something*. Team average is a reasonable stub.

**Why team-level calibration (§1 #16)?** Operators planning by team need an aggregate ("our team's bug-fix estimates average 1.2× actual"); per-member is too granular for sprint planning.

**Why drift alert (§1 #17)?** A 30%+ shift signals: (a) member skill changed, (b) data corruption, (c) work-type shift. Either way, operator should know.

**Why backfill (§1 #18)?** New tenants joining mid-year have history; backfill computes snapshots so the trend is visible from day 1.

**Why multiplier-applied count (§1 #19)?** Calibration is advisory; if operators ignore it, the feature has no impact. Tracking adoption surfaces UX issues.

**Why multiplier-acceptance rate (§1 #20)?** Even with adoption, the recommendation may be wrong; tracking whether following it produced accurate estimates is the feedback loop.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0013_estimate_calibration.sql

CREATE TABLE estimate_calibration_snapshots (
    id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id                UUID NOT NULL,
    task_class               TEXT NOT NULL,
    snapshot_date            DATE NOT NULL,
    posterior_mean           DOUBLE PRECISION NOT NULL,
    posterior_std            DOUBLE PRECISION NOT NULL,
    sample_size              INT NOT NULL,
    effective_sample_size    DOUBLE PRECISION NOT NULL,
    recommended_multiplier   DOUBLE PRECISION NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id                UUID NOT NULL,
    UNIQUE (member_id, task_class, snapshot_date)
);
CREATE INDEX idx_calibration_latest ON estimate_calibration_snapshots (member_id, task_class, snapshot_date DESC);

ALTER TABLE estimate_calibration_snapshots ENABLE ROW LEVEL SECURITY;
CREATE POLICY calibration_tenant_iso ON estimate_calibration_snapshots
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Rust

```rust
// services/proj-sync/src/calibration/bayes.rs
pub struct DataPoint { pub actual_hours: f64, pub estimate_hours: f64, pub days_ago: f64 }

pub fn compute_posterior(points: &[DataPoint]) -> Option<Posterior> {
    if points.is_empty() { return None; }
    let half_life_days = 90.0;
    let decay = |days: f64| (-days * 2f64.ln() / half_life_days).exp();

    let weighted_ratios: Vec<(f64, f64)> = points.iter()
        .filter(|p| p.estimate_hours > 0.0 && p.actual_hours > 0.0)
        .map(|p| {
            let ratio = p.actual_hours / p.estimate_hours;
            let w = decay(p.days_ago);
            (ratio, w)
        }).collect();

    if weighted_ratios.len() < 3 { return None; }

    let sum_w: f64 = weighted_ratios.iter().map(|(_, w)| w).sum();
    let weighted_mean: f64 = weighted_ratios.iter().map(|(r, w)| r * w).sum::<f64>() / sum_w;
    let weighted_var: f64 = weighted_ratios.iter()
        .map(|(r, w)| w * (r - weighted_mean).powi(2))
        .sum::<f64>() / sum_w;
    let weighted_std = weighted_var.sqrt();

    Some(Posterior {
        mean: weighted_mean,
        std: weighted_std,
        sample_size: weighted_ratios.len() as i32,
        effective_sample_size: sum_w,
    })
}

#[derive(Clone, Debug)]
pub struct Posterior {
    pub mean: f64,
    pub std: f64,
    pub sample_size: i32,
    pub effective_sample_size: f64,
}
```

### Nightly job

```rust
// services/proj-sync/src/calibration/nightly.rs
pub async fn run_nightly(pool: &sqlx::PgPool, tenant_id: uuid::Uuid) -> anyhow::Result<i32> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string()).execute(pool).await?;

    let cells: Vec<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT DISTINCT assignee_subject_id, task_class
         FROM issues
         WHERE assignee_subject_id IS NOT NULL AND task_class IS NOT NULL
           AND status = 'done' AND status_updated_at_ns > $1",
        (chrono::Utc::now() - chrono::Duration::days(365)).timestamp_nanos_opt().unwrap()
    ).fetch_all(pool).await?;

    let today = chrono::Utc::now().date_naive();
    let mut published = 0;
    for (member_id, task_class) in cells {
        let points = fetch_data_points(pool, member_id, &task_class).await?;
        if let Some(post) = compute_posterior(&points) {
            sqlx::query(
                "INSERT INTO estimate_calibration_snapshots
                  (member_id, task_class, snapshot_date,
                   posterior_mean, posterior_std, sample_size, effective_sample_size,
                   recommended_multiplier, tenant_id)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8, current_setting('app.tenant_id')::uuid)
                 ON CONFLICT (member_id, task_class, snapshot_date) DO NOTHING"
            )
            .bind(member_id).bind(&task_class).bind(today)
            .bind(post.mean).bind(post.std).bind(post.sample_size).bind(post.effective_sample_size)
            .bind((post.mean * 100.0).round() / 100.0)
            .execute(pool).await?;

            emit_memory_row("proj.estimate_calibration_computed", serde_json::json!({
                "member_id": member_id, "task_class": task_class,
                "snapshot_date": today, "posterior_mean": post.mean,
                "sample_size": post.sample_size,
                "effective_sample_size": post.effective_sample_size,
                "recommended_multiplier": (post.mean * 100.0).round() / 100.0,
            })).await;
            metrics::counter!("proj_calibration_cells_computed_total").increment(1);
            published += 1;
        } else {
            metrics::counter!("proj_calibration_cells_below_threshold_total").increment(1);
        }
    }
    Ok(published)
}
```

---

## §4 — Acceptance criteria

1. **Compute posterior for happy cell** — 5 issues with ratios [1.0, 1.2, 1.5, 1.3, 1.1] → posterior_mean ≈ 1.22 (weighted by recency).
2. **Effective sample size from decay** — 3 issues 1 day ago + 3 issues 180 days ago → effective_sample_size ≈ 3 + 3 × 0.25 = 3.75.
3. **Threshold of 3 enforced** — only 2 issues completed → no snapshot written.
4. **Snapshot append-only** — second nightly run same day → ON CONFLICT DO NOTHING; no row added.
5. **History endpoint returns trend** — 10 days of snapshots → 10 rows ASC by snapshot_date.
6. **memory audit per published snapshot** — `proj.estimate_calibration_computed` row appears.
7. **Recommended multiplier ≈ posterior mean** — rounded to 2 decimals.
8. **Per-cell isolation** — Alice + bug-fix has its own row distinct from Alice + feature-work.
9. **No auto-apply to estimates** — UI surfaces multiplier; issue.estimate field unchanged.
10. **RLS isolates** — tenant A invisible to tenant B.
11. **Skip issues without estimate** — issue with estimate = null → skipped (not in posterior).
12. **Skip issues without actual** — completed but 0 billable hours → skipped.
13. **Drift metric** — second snapshot with mean differing from prior → histogram records magnitude.
14. **Counter per cell processed** — even below-threshold cells counted.
15. **Outlier excluded** — data point with ratio=10 → excluded from posterior; included in `data_points_summary.outlier_count` (AC for §1 #11).
16. **CI in snapshot** — snapshot includes `posterior_ci_lower` + `posterior_ci_upper` 95% bounds (AC for §1 #12).
17. **Engagement opt-out excludes data** — set `calibration_enabled=false`; that engagement's issues skipped (AC for §1 #13).
18. **data_points_summary populated** — snapshot has p25/p50/p75/oldest/newest (AC for §1 #14).
19. **New-hire bootstrap returns team average** — member with 1 data point → response has `bootstrap=true` + team-average multiplier (AC for §1 #15).
20. **Team-level calibration** — GET /team/:task_class → weighted average across team (AC for §1 #16).
21. **Drift alert at > 30% shift** — fixture: posterior_mean shifts from 1.0 to 1.4 → `proj.calibration_drift_alert` SEV-3 (AC for §1 #17).
22. **Backfill produces past snapshots** — `cyberos calibration backfill --since 2026-01-01` → snapshots for each prior day (AC for §1 #18).
23. **Multiplier-applied count metric** — operator uses recommended → counter increments (AC for §1 #19).
24. **Multiplier acceptance rate** — operator's estimate within 10% of recommended → counted in success (AC for §1 #20).

---

## §5 — Verification

```rust
#[test]
fn posterior_with_uniform_ratios() {
    let points = vec![
        DataPoint { actual_hours: 1.0, estimate_hours: 1.0, days_ago: 0.0 },
        DataPoint { actual_hours: 2.0, estimate_hours: 2.0, days_ago: 0.0 },
        DataPoint { actual_hours: 3.0, estimate_hours: 3.0, days_ago: 0.0 },
    ];
    let p = compute_posterior(&points).unwrap();
    assert!((p.mean - 1.0).abs() < 1e-6);
    assert_eq!(p.sample_size, 3);
}

#[test]
fn old_data_decays() {
    let recent = vec![
        DataPoint { actual_hours: 2.0, estimate_hours: 1.0, days_ago: 0.0 },
        DataPoint { actual_hours: 2.0, estimate_hours: 1.0, days_ago: 0.0 },
        DataPoint { actual_hours: 2.0, estimate_hours: 1.0, days_ago: 0.0 },
    ];
    let old = vec![
        DataPoint { actual_hours: 1.0, estimate_hours: 1.0, days_ago: 180.0 },
        DataPoint { actual_hours: 1.0, estimate_hours: 1.0, days_ago: 180.0 },
        DataPoint { actual_hours: 1.0, estimate_hours: 1.0, days_ago: 180.0 },
    ];
    let mixed: Vec<DataPoint> = recent.iter().chain(old.iter()).cloned().collect();
    let p = compute_posterior(&mixed).unwrap();
    assert!(p.mean > 1.5, "recent data (ratio 2.0) should dominate; got mean = {}", p.mean);
}

#[test]
fn below_threshold_returns_none() {
    let points = vec![
        DataPoint { actual_hours: 1.0, estimate_hours: 1.0, days_ago: 0.0 },
        DataPoint { actual_hours: 1.0, estimate_hours: 1.0, days_ago: 0.0 },
    ];
    assert!(compute_posterior(&points).is_none());
}

#[tokio::test]
async fn nightly_publishes_snapshot() {
    let env = TestEnv::new().await;
    let member = env.create_member().await;
    for i in 0..5 {
        env.create_completed_issue(member, "bug_fix", estimate=2.0, actual=2.5,
                                    completed_days_ago=i*7).await;
    }
    let published = run_nightly(&env.pool, env.tenant_id()).await.unwrap();
    assert_eq!(published, 1);
    let snap: CalibrationSnapshot = env.read_latest_snapshot(member, "bug_fix").await;
    assert!((snap.recommended_multiplier - 1.25).abs() < 0.05);
}

#[tokio::test]
async fn idempotent_same_day() {
    let env = TestEnv::new().await;
    let _ = env.bootstrap_calibration_data().await;
    let r1 = run_nightly(&env.pool, env.tenant_id()).await.unwrap();
    let r2 = run_nightly(&env.pool, env.tenant_id()).await.unwrap();
    assert!(r1 >= 1);
    assert_eq!(r2, 0);   // ON CONFLICT DO NOTHING
}
```

---

## §6 — Implementation skeleton

(API + DB above.)

---

## §7 — Dependencies

- **TASK-PROJ-001** — issues schema (estimate, task_class, assignee).
- **TASK-PROJ-004** — status FSM (Done detection).
- **TASK-TIME-001** — billable hours source.
- **TASK-MEMORY-101** — audit emission.

---

## §8 — Example payloads

```json
{
  "kind": "proj.estimate_calibration_computed",
  "payload": {
    "member_id": "mb-...",
    "task_class": "bug_fix",
    "snapshot_date": "2026-05-16",
    "posterior_mean": 1.28,
    "sample_size": 12,
    "effective_sample_size": 8.7,
    "recommended_multiplier": 1.28
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Cell-aware new-hire bootstrap (use team average as prior until 3 data points) — slice 4+.
- Variance-aware multiplier (recommend × 1.28 ± 0.15) — slice 4+; UI complexity.
- Estimation method exploration (planning poker vs absolute) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Member has < 3 cells | threshold check | No snapshot | None — gather more data |
| All issues have ratio = 0 or NaN | NaN-skip in compute_posterior | Cell skipped | Operator investigates time-tracking |
| Insert race (concurrent nightly) | ON CONFLICT DO NOTHING | One winner; idempotent | None |
| Daylight-saving day-counting | days_ago uses UTC | Consistent | None |
| Extreme outlier (ratio = 100) | included but std reflects | UI shows wide CI | Operator considers excluding |
| Member changes role | calibration carries on; new task_class cell starts fresh | Trend shows change | None |
| memory emit fails | snapshot stored; audit lost; sev-2 | None | Operator restores memory |
| RLS bypass | RLS policy | 0 rows | None |
| Aggregate query slow at 100K issues | nightly batch acceptable | None | Slice 4+ optimise |
| Issue without `estimate` field | filtered out | Doesn't affect computation | None |
| Multiple time entries same issue (split work) | sum gives true actual | Correct | None |
| memory-PII concerns (member name in audit) | member_id is UUID, not name | Safe | None |
| Tenant offboarding | their cell snapshots persist | Audit trail retained per legal | None until cleanup script |
| Outlier threshold misconfigured (negative) | startup validation | use default | Operator |
| CI computation produces NaN (zero variance) | guard: return CI = mean ± 0 | None | None |
| Engagement opt-out toggled mid-cycle | next nightly excludes | snapshot uses snapshot-time state | None |
| data_points_summary > 1KB | bounded | None | None |
| New-hire member with 0 team data either | bootstrap returns no recommendation | UI shows "no data" | None |
| Team-level calibration with single member | uses that member's data only | None | None |
| Drift alert false positive (new hire transition) | dedup if member changed task_class | None | None |
| Backfill on date with existing snapshot | ON CONFLICT DO NOTHING | None | None |
| Multiplier-applied tracking races create-issue | best-effort metric | None | None |
| Acceptance rate metric over short window | bounded; require min 5 issues | None | None |
| All cells below threshold | counter only | sweep completes | None |
| Posterior compute with all-zero weights (edge case) | division by zero guard | None | None |
| CI bounds clamped at [0.1, 10.0] | sanity clamp | None | None |
| Snapshot row size > 10KB (huge summary) | bounded | None | None |

---

## §11 — Implementation notes

- 90-day half-life chosen empirically: covers ~3 cycles for typical 2-3 week cycles; weighting goes ~0.5 at 90 days, ~0.25 at 180.
- Exp-decay formula: `w = exp(-days × ln(2) / 90)`. At days=0 → w=1.0; days=90 → w=0.5; days=270 → w=0.125.
- Posterior std is weighted variance (Bessel correction not applied because weighted; effective_sample_size accounts for it).
- Cell-aware new-hire (use team-average prior) is slice 4+; current implementation just waits for 3 data points.
- The UI hint in §1 #8 lives in TASK-PROJ-001's issue create form: when assignee + task_class chosen, fetches latest snapshot and displays "recommended × <multiplier>".
- Issue estimate units are points/hours depending on engagement convention; this task treats them opaquely (just numbers; ratio is dimensionless).
- Outlier thresholds (5×, 0.2×) were chosen empirically from observed data; data-entry errors typically cluster at integer ratios (5, 10, 100).
- CI is computed using normal approximation (mean ± 1.96 × std / sqrt(effective_n)); good enough for the sample sizes typical.
- Per-engagement opt-out is rare; R&D engagements specifically. Most engagements should contribute.
- data_points_summary is computed once per snapshot at compute time; not regenerated on read.
- New-hire bootstrap uses team-average across same task_class for the engagement; if no team data either, returns no recommendation.
- Team-level calibration weights members equally (could weight by sample size, slice 4+).
- Drift alert threshold (30%) is conservative; tunable per tenant in slice 4+.
- Backfill is single-tenant; running for all tenants requires explicit iteration.
- Multiplier-applied count is incremented at issue.create when operator's estimate matches recommended × original (within 1%); approximate but useful.
- Multiplier-acceptance rate is computed at issue.completion: if actual is within 10% of (original × recommended), counted as success.
- The 95% CI bounds may be wide for small effective_sample_size; operators reading them know to discount.
- We considered storing raw data points alongside snapshot but rejected: bloats storage; can re-fetch from issues if needed.
- The snapshot is per-day; if no data changes in a day, no snapshot written (ON CONFLICT DO NOTHING + no-data-points returns None).
- Calibration is per-tenant scope; cross-tenant member calibration not supported (member is unique within tenant).
- We don't track per-engagement calibration because individual estimation skill is the actionable variable; per-engagement noise.
- The recommended multiplier rounding (2 decimals) matches operator UI display; raw mean stored unrounded for trend computation.
- Posterior std stored for CI computation + trend analysis; not surfaced in main UI.
- The cell-level audit row emits even for cells that produce no snapshot (counter increments).

---

*End of TASK-PROJ-013.*
