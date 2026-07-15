//! Per-category retention / erasure sweeper (TASK-EVAL-001 clause 6, 16). The bounded-recall half of the
//! governance gate: for each `(tenant, category)` retention policy the operator configured, erase the
//! Layer-2 / derived BRAIN projections older than `retain_days`. Nothing is retained without a policy; a
//! tenant with no policy sweeps nothing; the sweep is idempotent (a second run with no fresh past-retention
//! rows erases zero).
//!
//! WHAT IT ERASES, AND WHAT IT NEVER TOUCHES (clause 6, 11; §2). The sweeper deletes ONLY from the
//! rebuildable derived layer, never from the hash chain:
//!
//! - `l2_memory` - the Layer-2 projection (TASK-BRAIN-101), matched by category via `frontmatter->>'eval_category'`.
//! - `brain_event_embedding` - the per-interaction embedding lens (TASK-MEMORY-123), matched by its `kind` column.
//! - `brain_summary` - the rolling-summary lens (TASK-MEMORY-123), matched by its scope `kind` (see [`SWEEP_TARGETS`]).
//!
//! It NEVER issues a DELETE or UPDATE against `l1_audit_log`: that hash-chained table is the append-only
//! system of record and stays immutable, so you bound what is recallable without breaking what is provable.
//! The erasure itself is appended to L1 (`eval.retention_swept` per category, `eval.subject_erased` per
//! affected subject), so the fact of erasure is permanent even though the content is gone.
//!
//! DISABLED BY DEFAULT (clause 6, §10). Nothing here runs on its own. [`run_retention_sweep`] is invoked
//! ONLY when the operator explicitly calls it - the `cyberos-eval-sweep` binary (`src/bin/sweep.rs`), a
//! cron that shells out to it, or a test. There is no background task, no timer, no auto-start; the service
//! binary (`main.rs`) never calls it. This is the deliberate "the only automatic erasure is scheduled
//! retention, and it acts only on the policy the operator configured" posture (clause 11).
//!
//! WHICH POOLS (current topology). Retention policies live in the EVAL governance Postgres (`policy_pool`);
//! the derived projections + `l1_audit_log` live in the memory Postgres (`derived_pool` == `AppState
//! ::audit_pool`). Auth, memory, eval, and chat share one Postgres deployment today, so in practice the two
//! handles point at the same database - but the sweep keeps them as separate parameters so it does not
//! assume colocation. The derived-table DELETEs set BOTH tenant GUCs tx-local (`app.current_tenant_id` and
//! `app.tenant_id`), mirroring `cyberos_memory::brain::access_scope::set_access_guc`, so the RLS predicate
//! fires for the right tenant no matter which GUC each table keys on (`l1_audit_log` itself has no RLS).
//!
//! CATEGORY-TAGGING CAVEAT (task §10 failure mode "L2 row missing eval_category frontmatter"). A derived row
//! is erased ONLY when it positively matches the policy's category AND is past `retain_days`. An untagged
//! row (no `eval_category` frontmatter / a `kind` that does not equal the category) is NOT swept - it is
//! retained, the documented fail-safe (never over-delete). Backfilling the category tag at ingest
//! (TASK-MEMORY-122) is what brings such rows under a policy; until then the sweeper simply cannot see them,
//! which is the safe direction.

use uuid::Uuid;

use crate::audit;
use crate::db::Pool;

/// A derived table the sweeper can erase from, and the column it matches the policy category against. The
/// match column differs per table: `l2_memory` tags the category in `frontmatter->>'eval_category'` (a JSONB
/// lookup), while `brain_event_embedding` and `brain_summary` carry the interaction `kind` as a plain column.
/// `age_column` is the row's ingest/creation timestamp the `retain_days` cutoff compares against.
struct SweepTarget {
    /// Table name (a fixed compile-time constant, never user input - it is interpolated into SQL).
    table: &'static str,
    /// SQL boolean predicate, parameterised on `$2` = category, selecting rows OF that category. Kept as a
    /// fragment (not the whole statement) so the table + age predicate are assembled once in [`sweep_table`].
    category_match: &'static str,
    /// The timestamp column the `retain_days` cutoff compares against (`< now() - retain_days`).
    age_column: &'static str,
    /// The subject column to attribute an `eval.subject_erased` row to, when the table has one. `None` for
    /// tables (`l2_memory`) whose rows are not per-subject - they still count toward the swept total and the
    /// `eval.retention_swept` row, but emit no per-subject erasure event.
    subject_column: Option<&'static str>,
}

/// The derived projections the sweeper erases, in deletion order. Order is not load-bearing (each runs in
/// its own statement); embeddings + summaries (the BRAIN lens) first, then the generic L2 projection.
const SWEEP_TARGETS: &[SweepTarget] = &[
    // TASK-MEMORY-123 per-event embedding lens. Category tag = the interaction `kind` column.
    SweepTarget {
        table: "brain_event_embedding",
        category_match: "kind = $2",
        age_column: "created_at",
        subject_column: Some("subject_id"),
    },
    // TASK-MEMORY-123 rolling-summary lens. Only subject-scoped summaries carry a comparable `kind`; match the
    // scope_kind so a category that names a subject scope is swept, and attribute to its `subject_id`.
    SweepTarget {
        table: "brain_summary",
        category_match: "scope_kind = $2",
        age_column: "created_at",
        subject_column: Some("subject_id"),
    },
    // TASK-BRAIN-101 Layer-2 memory projection. Category tag = the `eval_category` frontmatter the
    // TASK-MEMORY-122 emitters write (§3 sketch). No per-subject column on this table.
    SweepTarget {
        table: "l2_memory",
        category_match: "frontmatter->>'eval_category' = $2",
        age_column: "ingested_at",
        subject_column: None,
    },
];

/// What one sweep erased, for the operator / the caller's logs. `rows_erased` is the total across every
/// target table; `subjects_erased` is the count of distinct subjects an `eval.subject_erased` row was
/// emitted for; `categories_swept` is how many `(tenant, category)` policies actually erased >= 1 row.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SweepReport {
    pub rows_erased: u64,
    pub subjects_erased: u64,
    pub categories_swept: u64,
}

/// The OTel metric name for swept rows (clause 16: `eval_retention_swept_rows_total{category}`). Emitted as
/// a structured `tracing` event (this workspace's metrics path is OTel via the obs pipeline, matching
/// `cyberos_memory::interaction::emit` and `cyberos_capture::emitter` - NOT the `metrics` facade).
pub const METRIC_RETENTION_SWEPT_ROWS: &str = "eval_retention_swept_rows_total";

/// Build the SQL `interval` cutoff predicate text for a target's age column. `retain_days` is bound as a
/// parameter (`$N`), not interpolated, so it is injection-safe; the `make_interval(days => ..)` form keeps
/// the cast explicit and avoids string-concatenated intervals.
fn age_cutoff_sql(age_column: &str, param_idx: usize) -> String {
    format!("{age_column} < now() - make_interval(days => ${param_idx})")
}

/// Whether a policy is even worth sweeping: a non-positive `retain_days` is treated as "no policy" (the
/// table CHECK forbids it, but a defensive guard means a bad row never deletes everything). Pure; unit-tested.
fn policy_is_sweepable(retain_days: i32) -> bool {
    retain_days > 0
}

/// Set BOTH tenant GUCs (`app.current_tenant_id` for the eval-keyed tables, `app.tenant_id` for the
/// brain-keyed tables) tx-local on the derived pool, so a DELETE fires under the right tenant's RLS no
/// matter which GUC the deployed table keys on. Mirrors `cyberos_memory::brain::access_scope::set_access_guc`
/// exactly. `set_config(..., true)` is transaction-local, so it never leaks across pooled connections.
async fn set_both_tenant_gucs(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Erase one target table's past-retention rows for `(tenant, category)` and return `(rows_erased,
/// subjects_erased)`. Runs in a single tenant-scoped transaction on the derived pool: set both GUCs, collect
/// the distinct affected subjects (for the per-subject erasure audit) via a `DELETE ... RETURNING`, commit.
///
/// The statement only ever names `target.table` (one of the three derived projections in [`SWEEP_TARGETS`])
/// - `l1_audit_log` is never referenced here, by construction, so the chain cannot be touched.
async fn sweep_table(
    derived_pool: &Pool,
    tenant_id: Uuid,
    category: &str,
    retain_days: i32,
    target: &SweepTarget,
) -> Result<(u64, Vec<Uuid>), sqlx::Error> {
    let mut tx = derived_pool.begin().await?;
    set_both_tenant_gucs(&mut tx, tenant_id).await?;

    // $1 = tenant_id, $2 = category, $3 = retain_days. The age cutoff and category match are fixed fragments;
    // only values are bound. When the table has a subject column we RETURN it so we can emit one
    // eval.subject_erased per distinct subject; otherwise we only need the row count.
    let cutoff = age_cutoff_sql(target.age_column, 3);
    let (rows, subjects) = match target.subject_column {
        Some(subject_col) => {
            let sql = format!(
                "DELETE FROM {table}
                  WHERE tenant_id = $1 AND {cat_match} AND {cutoff}
                  RETURNING {subject_col}",
                table = target.table,
                cat_match = target.category_match,
            );
            let returned: Vec<(Option<Uuid>,)> = sqlx::query_as(&sql)
                .bind(tenant_id)
                .bind(category)
                .bind(retain_days)
                .fetch_all(&mut *tx)
                .await?;
            tx.commit().await?;
            let rows = returned.len() as u64;
            // Distinct, non-null subjects - the set we attribute eval.subject_erased rows to.
            let mut subjects: Vec<Uuid> = returned.into_iter().filter_map(|(s,)| s).collect();
            subjects.sort();
            subjects.dedup();
            (rows, subjects)
        }
        None => {
            let sql = format!(
                "DELETE FROM {table}
                  WHERE tenant_id = $1 AND {cat_match} AND {cutoff}",
                table = target.table,
                cat_match = target.category_match,
            );
            let res = sqlx::query(&sql)
                .bind(tenant_id)
                .bind(category)
                .bind(retain_days)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            (res.rows_affected(), Vec::new())
        }
    };
    Ok((rows, subjects))
}

/// Run the retention/erasure sweep across every configured `(tenant, category)` policy. Disabled-by-default:
/// this only runs when an operator explicitly invokes it (the `cyberos-eval-sweep` binary, a cron, or a
/// test). Returns a [`SweepReport`] of what was erased.
///
/// Behaviour (clause 6):
///   * reads every `retention_policy` joined to its `data_category` (for the category name) from
///     `policy_pool` under the nil-tenant admin bypass (a cross-tenant operator job);
///   * for each policy, erases past-retention rows from each derived target on `derived_pool` (the memory
///     Postgres holding the projections + `l1_audit_log`);
///   * emits `eval.retention_swept` once per category that erased >= 1 row, and `eval.subject_erased` once
///     per distinct affected subject, onto the L1 chain (best-effort, the same contract as the handlers);
///   * emits the `eval_retention_swept_rows_total{category}` OTel counter (structured tracing event).
///
/// Defensive: a tenant/category with no policy sweeps nothing; a non-positive `retain_days` is skipped; an
/// untagged derived row (no `eval_category` / a non-matching `kind`) is retained, never over-deleted (task §10
/// fail-safe). The sweep NEVER deletes from `l1_audit_log` - it only ever targets the derived projections.
/// A missing derived table surfaces as the `sqlx::Error` (a deployment without the brain lens is a
/// misconfiguration to fix, not silently ignored).
pub async fn run_retention_sweep(
    policy_pool: &Pool,
    derived_pool: &Pool,
    audit_pool: Option<&Pool>,
) -> Result<SweepReport, sqlx::Error> {
    // Operator job: read policies across all tenants under the nil-tenant admin bypass the governance RLS
    // policies honour (USING ... OR current_setting = nil). One tx, read-only.
    let mut ptx = policy_pool.begin().await?;
    set_both_tenant_gucs(&mut ptx, Uuid::nil()).await?;
    let policies: Vec<(Uuid, String, i32)> = sqlx::query_as(
        "SELECT rp.tenant_id, dc.name, rp.retain_days
           FROM retention_policy rp
           JOIN data_category dc ON dc.id = rp.data_category_id",
    )
    .fetch_all(&mut *ptx)
    .await?;
    ptx.commit().await?;

    let mut report = SweepReport::default();
    for (tenant_id, category, retain_days) in policies {
        if !policy_is_sweepable(retain_days) {
            continue; // no-policy / bad row ⇒ erase nothing (fail-safe).
        }

        let mut category_rows: u64 = 0;
        let mut category_subjects: Vec<Uuid> = Vec::new();
        for target in SWEEP_TARGETS {
            let (rows, subjects) =
                sweep_table(derived_pool, tenant_id, &category, retain_days, target).await?;
            category_rows += rows;
            category_subjects.extend(subjects);
        }
        category_subjects.sort();
        category_subjects.dedup();

        if category_rows == 0 {
            continue; // nothing past retention for this category ⇒ no audit row, no metric.
        }
        report.rows_erased += category_rows;
        report.categories_swept += 1;
        report.subjects_erased += category_subjects.len() as u64;

        // clause 16 metric: eval_retention_swept_rows_total{category}. Structured tracing event (OTel path).
        tracing::info!(
            target: "cyberos_eval::retention",
            metric = METRIC_RETENTION_SWEPT_ROWS,
            category = %category,
            rows_erased = category_rows,
            "retention sweep erased derived rows"
        );

        // clause 6/12: the erasure itself is appended to the L1 chain (auditable erasure). One
        // eval.retention_swept per category, then one eval.subject_erased per affected subject. The actor on
        // these system rows is the nil subject (a scheduled job, no human actor), tenant-scoped.
        audit::emit_governance(
            audit_pool,
            tenant_id,
            Uuid::nil(),
            audit::kind::RETENTION_SWEPT,
            serde_json::json!({
                "category": category,
                "retain_days": retain_days,
                "rows_erased": category_rows,
                "subjects_erased": category_subjects.len(),
            }),
        )
        .await;
        for subject in &category_subjects {
            audit::emit_governance(
                audit_pool,
                tenant_id,
                *subject,
                audit::kind::SUBJECT_ERASED,
                serde_json::json!({
                    "subject_id": subject,
                    "category": category,
                    "retain_days": retain_days,
                }),
            )
            .await;
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_selection_skips_non_positive_retain_days() {
        // A policy is only sweepable with a positive retain_days; 0 / negative ⇒ erase nothing (fail-safe).
        assert!(policy_is_sweepable(1));
        assert!(policy_is_sweepable(365));
        assert!(!policy_is_sweepable(0));
        assert!(!policy_is_sweepable(-1));
    }

    #[test]
    fn age_cutoff_binds_retain_days_and_never_concatenates() {
        // The cutoff parameterises retain_days ($N) rather than concatenating it, so it is injection-safe and
        // compares the right column. (Proves the age-cutoff selection logic without a DB.)
        let sql = age_cutoff_sql("ingested_at", 3);
        assert_eq!(sql, "ingested_at < now() - make_interval(days => $3)");
        let sql = age_cutoff_sql("created_at", 3);
        assert_eq!(sql, "created_at < now() - make_interval(days => $3)");
        assert!(
            !sql.contains("' days'"),
            "must not string-concatenate an interval"
        );
    }

    #[test]
    fn sweep_targets_never_include_l1_audit_log() {
        // The load-bearing invariant: the sweeper's target set is exactly the rebuildable derived layer and
        // NEVER the immutable L1 chain. If a future edit adds l1_audit_log here, this test fails loudly.
        assert!(
            SWEEP_TARGETS.iter().all(|t| t.table != "l1_audit_log"),
            "the retention sweep must never target l1_audit_log (the immutable system of record)"
        );
        // And the only tables it does target are the three documented derived projections.
        let tables: Vec<&str> = SWEEP_TARGETS.iter().map(|t| t.table).collect();
        assert_eq!(
            tables,
            vec!["brain_event_embedding", "brain_summary", "l2_memory"],
            "the derived sweep set must be exactly the brain + L2 projections"
        );
    }

    #[test]
    fn every_target_has_a_category_match_and_age_column() {
        // Each target must positively match a category AND bound an age column - so an untagged or fresh row
        // is never erased (the task §10 fail-safe: no over-deletion).
        for t in SWEEP_TARGETS {
            assert!(
                t.category_match.contains("$2"),
                "category must be a bound parameter"
            );
            assert!(
                !t.age_column.is_empty(),
                "every target needs an age column for the cutoff"
            );
        }
    }

    #[test]
    fn empty_report_is_zero() {
        let r = SweepReport::default();
        assert_eq!(r.rows_erased, 0);
        assert_eq!(r.subjects_erased, 0);
        assert_eq!(r.categories_swept, 0);
    }
}
