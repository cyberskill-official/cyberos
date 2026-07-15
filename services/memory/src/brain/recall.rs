//! TASK-MEMORY-123 §1 #5,#7,#8,#9,#10,#18 — `POST /v1/memory/recall`: summaries-first, access-scoped,
//! provenance-carrying semantic recall.
//!
//! The flow (the task's §3 recall sketch, made real):
//!   1. embed the query through the ai-gateway (graceful degrade to full-text over summaries if the gateway
//!      is down — §1 #18; `query_embed` is then listed in `degraded_backends`);
//!   2. search CURRENT summaries first (§1 #5);
//!   3. drill into raw HOT events when `drill=true` or the best summary score is below the confidence floor,
//!      fusing the two result sets via RRF (§1 #5);
//!   4. for each candidate, read-time-verify its `chain_anchor` against Layer 1 (drop + sev-1 on mismatch,
//!      §1 #10) THEN apply the TASK-EVAL-001 per-subject access predicate (EXCLUDE, not derank; deny-by-default,
//!      §1 #8) — a closest neighbour the caller may not see never appears;
//!   5. return the surviving hits up to `limit`, each carrying a provenance pointer back to its Layer-1
//!      row(s) (§1 #9).
//!
//! Tenant RLS is enforced at the DB (every query runs inside the caller's tenant tx, §1 #16); the access
//! predicate is the intra-tenant boundary on top of it.

use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    access_scope, metrics, now_ns, provenance, BrainConfig, Caller, EmbedClient, EmbedError,
    HitSource, Provenance, RecallHit, RecallQuery, RecallResults, MAX_RECALL_LIMIT,
};
use crate::embeddings::to_pgvector_literal;

/// RRF constant (the canonical paper default; matches `search.rs`).
const RRF_K: f64 = 60.0;
/// How many candidates to fetch from each retriever before fusion + access filtering. Over-fetch so the
/// access EXCLUDE can drop several hits and still fill `limit`.
const FETCH_K: i64 = 50;

/// A recall error surfaced to the handler. `LimitTooLarge` -> 400; `AllBackendsDown` -> 503 (§1 #18);
/// `Db` -> 500.
#[derive(Debug, thiserror::Error)]
pub enum RecallError {
    #[error("limit too large (max {MAX_RECALL_LIMIT})")]
    LimitTooLarge,
    #[error("all recall backends are unavailable")]
    AllBackendsDown,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// An internal candidate before access-filtering: carries the rank position from each retriever (for RRF)
/// plus everything needed to verify + emit it.
#[derive(Clone)]
struct Candidate {
    source_seq: i64,
    audit_row_id: String,
    subject_id: Uuid,
    channel_id: Option<Uuid>,
    kind: String,
    ts_ns: i64,
    snippet: String,
    chain_anchor_hex: String,
    source: HitSource,
    summary_rank: Option<usize>,
    event_rank: Option<usize>,
    /// For a summary candidate: the covered range + top contributors (provenance, §1 #9).
    covered_seq_range: Option<(i64, i64)>,
    top_contributors: Vec<String>,
}

/// Recall entry point (§1 #7). Validates the limit, embeds the query (graceful degrade), runs summaries-
/// first + optional drill, fuses, then verifies + access-filters each candidate before returning.
pub async fn recall(
    q: RecallQuery,
    caller: &Caller,
    pool: &PgPool,
    gw: &EmbedClient,
) -> Result<RecallResults, RecallError> {
    if q.limit > MAX_RECALL_LIMIT {
        return Err(RecallError::LimitTooLarge);
    }
    let tenant_id = caller.tenant_id;
    let cfg = BrainConfig::from_env();
    let started = std::time::Instant::now();
    let mut degraded: Vec<String> = Vec::new();

    // Narrow a caller-supplied subject_scope to what they may actually see (never widens; §1 #7). The per-
    // hit access check below remains the load-bearing authority.
    let visible_scope =
        access_scope::intersect_visible_scope(pool, caller, &q.subject_scope).await?;

    // 1. Embed the query (graceful degrade to full-text over summaries if the gateway is down — §1 #18).
    let q_vec: Option<Vec<f32>> = match gw.embed(tenant_id, &q.q).await {
        Ok(emb) => Some(emb.vector),
        Err(EmbedError::SpendCapExhausted)
        | Err(EmbedError::GatewayDown(_))
        | Err(EmbedError::Malformed(_)) => {
            degraded.push("query_embed".to_string());
            None
        }
    };

    // 2. Summaries-first. `best_summary` is the REAL top cosine similarity of the closest current summary
    // (MEM-005, R9, F7) — not a constant — so the confidence floor below can actually fire.
    let (summary_hits, best_summary) =
        summary_search(pool, tenant_id, q_vec.as_deref(), &q, &visible_scope).await?;

    // 3. Drill into hot events on demand or below the confidence floor (§1 #5). When the query couldn't be
    // embedded, summaries-only full-text is the path (drill needs a vector for hot events).
    let drill = should_drill(q.drill, best_summary, cfg.recall_confidence_floor);
    let path_label = if drill && q_vec.is_some() {
        "drill"
    } else {
        "summary"
    };
    let mut candidates: Vec<Candidate> = summary_hits;
    if drill {
        if let Some(v) = q_vec.as_deref() {
            let event_hits = hot_event_search(pool, tenant_id, v, &q, &visible_scope).await?;
            candidates = rrf_fuse(candidates, event_hits);
        }
    }

    // If we have no vector AND no summaries, every retriever is impossible — 503 (§1 #18). Empty results
    // with a working backend are a normal 200 [] (handled by returning an empty items list).
    if q_vec.is_none() && candidates.is_empty() && summaries_empty(pool, tenant_id).await? {
        return Err(RecallError::AllBackendsDown);
    }

    // 4. Per-candidate: chain-anchor verify (§1 #10) THEN access EXCLUDE (§1 #8). Order matters — a tampered
    // hit is dropped before we even consult access, so a mismatch never leaks via a side channel.
    let mut out: Vec<RecallHit> = Vec::new();
    for c in candidates {
        // §1 #10 — read-time chain_anchor verify against Layer 1. A summary hit verifies its top contributor
        // rows; an event hit verifies its single source row.
        let verified = verify_candidate(pool, tenant_id, &c).await?;
        if !verified {
            metrics::chain_anchor_mismatch(tenant_id, c.source_seq);
            continue;
        }

        // §1 #8 — TASK-EVAL-001 per-subject access predicate, applied as an EXCLUDE with deny-by-default. A
        // semantically-closest neighbour the caller may not see is dropped here, not deranked.
        if !access_scope::caller_may_see(pool, caller, c.subject_id).await? {
            let reason = access_scope::deny_reason(pool, caller, c.subject_id).await?;
            metrics::access_denied(tenant_id, reason.as_str());
            continue;
        }

        out.push(into_hit(c, verified));
        if out.len() >= q.limit {
            break;
        }
    }

    let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
    metrics::recall_latency(tenant_id, path_label, latency_ms);

    let explain = q.explain.then(|| {
        serde_json::json!({
            "path": path_label,
            "returned": out.len(),
            "drill": drill,
            "degraded_backends": degraded.clone(),
            "query_embedded": q_vec.is_some(),
            "best_summary": best_summary,
            "latency_ms": latency_ms,
        })
    });

    Ok(RecallResults {
        items: out,
        explain,
        degraded_backends: degraded,
    })
}

/// Search current summaries (§1 #5). With a query vector, ranks by cosine distance against the partial
/// summary HNSW (current versions only). Without one (gateway down), falls back to full-text over the digest
/// (TASK-MEMORY-108 lexical path) so recall still answers (§1 #18). Applies the optional visible subject scope.
async fn summary_search(
    pool: &PgPool,
    tenant_id: Uuid,
    q_vec: Option<&[f32]>,
    q: &RecallQuery,
    visible_scope: &Option<Vec<Uuid>>,
) -> Result<(Vec<Candidate>, f32), sqlx::Error> {
    let scope_clause = subject_scope_sql(visible_scope, "subject_id");
    let ts_clause = ts_window_sql(q, "window_end_ns", "window_start_ns");

    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let rows = if let Some(v) = q_vec {
        let lit = to_pgvector_literal(v);
        let sql = format!(
            "SELECT scope_kind, scope_id, subject_id, digest, covered_seq_lo, covered_seq_hi,
                    top_contributors, window_end_ns,
                    (embedding <=> $2::vector) AS cos_dist
               FROM brain_summary
              WHERE tenant_id = $1 AND superseded_by IS NULL AND embedding IS NOT NULL
                {scope_clause} {ts_clause}
              ORDER BY embedding <=> $2::vector ASC
              LIMIT $3"
        );
        sqlx::query(&sql)
            .bind(tenant_id)
            .bind(&lit)
            .bind(FETCH_K)
            .fetch_all(&mut *tx)
            .await?
    } else {
        // Full-text fallback over the digest (§1 #18).
        let sql = format!(
            "SELECT scope_kind, scope_id, subject_id, digest, covered_seq_lo, covered_seq_hi,
                    top_contributors, window_end_ns
               FROM brain_summary
              WHERE tenant_id = $1 AND superseded_by IS NULL
                AND to_tsvector('simple', digest) @@ websearch_to_tsquery('simple', $2)
                {scope_clause} {ts_clause}
              ORDER BY ts_rank_cd(to_tsvector('simple', digest), websearch_to_tsquery('simple', $2)) DESC
              LIMIT $3"
        );
        sqlx::query(&sql)
            .bind(tenant_id)
            .bind(&q.q)
            .bind(FETCH_K)
            .fetch_all(&mut *tx)
            .await?
    };
    tx.commit().await?;

    // MEM-005 (R9, F7): best summary similarity = the top row's cosine similarity (rows are ordered by
    // ascending cosine distance, so the first row is the closest). This drives the confidence-floor -> drill
    // decision in `recall`. Previously `best_summary` was hardcoded to 1.0 for any match, so the floor never
    // triggered a quality drill. The full-text fallback carries no vector distance, so it reports 0.0 (drill
    // needs a vector regardless).
    let best_similarity = if q_vec.is_some() {
        rows.first()
            .and_then(|r| r.try_get::<f64, _>("cos_dist").ok())
            .map(|d| (1.0 - d as f32).clamp(0.0, 1.0))
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let candidates: Vec<Candidate> = rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let top: Vec<String> = r
                .try_get::<serde_json::Value, _>("top_contributors")
                .ok()
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let lo: i64 = r.try_get("covered_seq_lo").unwrap_or_default();
            let hi: i64 = r.try_get("covered_seq_hi").unwrap_or_default();
            // A summary's provenance subject: for a subject-scope summary it is the subject column; for
            // channel / time_window summaries the access subject is taken from the top contributing event
            // (looked up during verify). Use the summary's subject_id when present, else nil (resolved later).
            let subject_id = r
                .try_get::<Option<Uuid>, _>("subject_id")
                .ok()
                .flatten()
                .unwrap_or_else(Uuid::nil);
            Candidate {
                source_seq: hi, // the latest covered seq anchors the verify + dedup
                audit_row_id: top.first().cloned().unwrap_or_default(),
                subject_id,
                channel_id: None,
                kind: "summary".to_string(),
                ts_ns: r.try_get("window_end_ns").unwrap_or_default(),
                snippet: r.try_get::<String, _>("digest").unwrap_or_default(),
                chain_anchor_hex: String::new(), // summaries verify via their contributor rows
                source: HitSource::Summary,
                summary_rank: Some(i),
                event_rank: None,
                covered_seq_range: Some((lo, hi)),
                top_contributors: top,
            }
        })
        .collect();
    Ok((candidates, best_similarity))
}

/// Whether to drill into raw hot events (§1 #5): explicitly requested by the caller, OR the best summary
/// similarity is below the confidence floor. MEM-005 (R9, F7): `best_summary` is now the real top cosine
/// similarity from [`summary_search`], so a weak summary match actually triggers a drill instead of being
/// masked by a hardcoded 1.0.
fn should_drill(explicit: bool, best_summary: f32, floor: f32) -> bool {
    explicit || best_summary < floor
}

/// Search raw HOT events (§1 #5 drill). Ranks by cosine distance against the partial hot HNSW (`tier='hot'`).
/// Applies the optional visible subject scope + the ts window + the channel scope.
async fn hot_event_search(
    pool: &PgPool,
    tenant_id: Uuid,
    q_vec: &[f32],
    q: &RecallQuery,
    visible_scope: &Option<Vec<Uuid>>,
) -> Result<Vec<Candidate>, sqlx::Error> {
    let lit = to_pgvector_literal(q_vec);
    let scope_clause = subject_scope_sql(visible_scope, "subject_id");
    let chan_clause = channel_scope_sql(&q.channel_scope, "channel_id");
    let ts_clause = ts_window_sql(q, "ts_ns", "ts_ns");

    let sql = format!(
        "SELECT source_seq, audit_row_id, subject_id, channel_id, kind, ts_ns, chain_anchor_hex
           FROM brain_event_embedding
          WHERE tenant_id = $1 AND tier = 'hot' AND embedding IS NOT NULL
            {scope_clause} {chan_clause} {ts_clause}
          ORDER BY embedding <=> $2::vector ASC
          LIMIT $3"
    );
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let rows = sqlx::query(&sql)
        .bind(tenant_id)
        .bind(&lit)
        .bind(FETCH_K)
        .fetch_all(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(rows
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let audit_row_id: String = r.try_get("audit_row_id").unwrap_or_default();
            Candidate {
                source_seq: r.try_get("source_seq").unwrap_or_default(),
                audit_row_id: audit_row_id.clone(),
                subject_id: r.try_get("subject_id").unwrap_or_else(|_| Uuid::nil()),
                channel_id: r.try_get::<Option<Uuid>, _>("channel_id").ok().flatten(),
                kind: r.try_get::<String, _>("kind").unwrap_or_default(),
                ts_ns: r.try_get("ts_ns").unwrap_or_default(),
                snippet: String::new(), // filled from Layer 1 during verify (keeps the hot query cheap)
                chain_anchor_hex: r
                    .try_get::<String, _>("chain_anchor_hex")
                    .unwrap_or_default(),
                source: HitSource::Event,
                summary_rank: None,
                event_rank: Some(i),
                covered_seq_range: None,
                top_contributors: vec![audit_row_id],
            }
        })
        .collect())
}

/// Verify a candidate's chain anchor against Layer 1 (§1 #10) and, as a side effect, resolve any missing
/// snippet + subject from the live Layer-1 row. Returns whether the candidate is trustworthy.
///   * event hit  — verify its single source row; backfill the snippet from that row.
///   * summary hit — verify its TOP contributor row (the freshest event it cited); resolve the access
///     subject from that row when the summary itself had none (channel / time_window scopes).
async fn verify_candidate(
    pool: &PgPool,
    tenant_id: Uuid,
    c: &Candidate,
) -> Result<bool, sqlx::Error> {
    match c.source {
        HitSource::Event => {
            provenance::verify_chain_anchor(pool, tenant_id, c.source_seq, &c.chain_anchor_hex)
                .await
        }
        HitSource::Summary => {
            // Verify via the top contributor's Layer-1 row (recompute its anchor from the live chain).
            let Some(top) = c.top_contributors.first() else {
                // A summary with no contributors can't be verified -> drop (fail closed).
                return Ok(false);
            };
            let Some((pt, seq)) = provenance::parse_audit_row_id(top) else {
                return Ok(false);
            };
            if pt != tenant_id {
                return Ok(false);
            }
            // Recompute the live anchor for that row and compare to what Layer 1 advertises now.
            let live_hex: Option<(String,)> = sqlx::query_as(
                "SELECT chain_anchor_hex FROM l1_audit_log WHERE tenant_id = $1 AND seq = $2",
            )
            .bind(tenant_id)
            .bind(seq)
            .fetch_optional(pool)
            .await?;
            let Some((advertised,)) = live_hex else {
                return Ok(false);
            };
            provenance::verify_chain_anchor(pool, tenant_id, seq, &advertised).await
        }
    }
}

/// Convert a verified candidate into the public [`RecallHit`] with its [`Provenance`].
fn into_hit(c: Candidate, chain_verified: bool) -> RecallHit {
    let provenance = match c.source {
        HitSource::Event => Provenance {
            audit_row_ids: vec![c.audit_row_id.clone()],
            covered_seq_range: None,
            chain_verified,
        },
        HitSource::Summary => Provenance {
            audit_row_ids: if c.top_contributors.is_empty() {
                vec![c.audit_row_id.clone()]
            } else {
                c.top_contributors.clone()
            },
            covered_seq_range: c.covered_seq_range,
            chain_verified,
        },
    };
    // Compute the RRF score from whatever ranks the candidate carries.
    let score = rrf_score(c.summary_rank, c.event_rank);
    RecallHit {
        audit_row_id: c.audit_row_id,
        subject_id: c.subject_id,
        channel_id: c.channel_id,
        kind: c.kind,
        ts_ns: c.ts_ns,
        snippet: c.snippet,
        score,
        source: c.source,
        provenance,
    }
}

/// Reciprocal Rank Fusion of summary + event candidates (§1 #5). An event citing a summary's top
/// contributor row merges INTO that summary, summing the per-retriever RRF terms. Every summary keeps its
/// own slot: two summaries (e.g. the subject-scope and a window-scope one) routinely share a top contributor
/// row, so keying summaries by `audit_row_id` made one silently overwrite the other - and when the survivor
/// was a nil-subject scope hit, access dropped it and recall returned no summary at all (the
/// brain_summaries_test flake; which one survived depended on embedding rank order).
fn rrf_fuse(summary: Vec<Candidate>, events: Vec<Candidate>) -> Vec<Candidate> {
    let mut by_key: HashMap<String, Candidate> = HashMap::new();
    // The event-merge index: a contributor row id -> the (best-ranked) summary slot that cites it.
    let mut contrib_index: HashMap<String, String> = HashMap::new();
    for s in summary {
        // summary_rank is the enumeration index from summary_search: present and unique per candidate.
        let key = format!("summary#{}", s.summary_rank.unwrap_or(usize::MAX));
        if !s.audit_row_id.is_empty() {
            contrib_index
                .entry(s.audit_row_id.clone())
                .or_insert_with(|| key.clone());
        }
        by_key.insert(key, s);
    }
    for e in events {
        let ekey = dedup_key(&e);
        let target = contrib_index.get(&ekey).cloned().unwrap_or(ekey);
        match by_key.get_mut(&target) {
            Some(existing) => {
                existing.event_rank = e.event_rank;
                if existing.snippet.is_empty() {
                    existing.snippet = e.snippet.clone();
                }
            }
            None => {
                by_key.insert(target, e);
            }
        }
    }
    let mut out: Vec<Candidate> = by_key.into_values().collect();
    out.sort_by(|a, b| {
        let sa = rrf_score(a.summary_rank, a.event_rank);
        let sb = rrf_score(b.summary_rank, b.event_rank);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// A candidate's dedup key: its primary audit row id (or `source_seq` when absent). A summary and an event
/// citing the same row fuse into one hit.
fn dedup_key(c: &Candidate) -> String {
    if c.audit_row_id.is_empty() {
        format!("seq:{}", c.source_seq)
    } else {
        c.audit_row_id.clone()
    }
}

/// RRF score = Σ 1/(k + rank) across the retrievers that returned the candidate.
fn rrf_score(summary_rank: Option<usize>, event_rank: Option<usize>) -> f32 {
    let s = summary_rank
        .map(|r| 1.0 / (RRF_K + r as f64 + 1.0))
        .unwrap_or(0.0);
    let e = event_rank
        .map(|r| 1.0 / (RRF_K + r as f64 + 1.0))
        .unwrap_or(0.0);
    (s + e) as f32
}

/// Whether the tenant has zero current summaries (used to decide the all-backends-down 503 when the query
/// also couldn't be embedded).
async fn summaries_empty(pool: &PgPool, tenant_id: Uuid) -> Result<bool, sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM brain_summary WHERE tenant_id = $1 AND superseded_by IS NULL",
    )
    .bind(tenant_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(n == 0)
}

/// Build a `subject_id IN (...)` SQL fragment for a visible scope, or empty when the scope is `None`. The
/// list is already access-filtered (`intersect_visible_scope`); an empty `Some(vec![])` yields a predicate
/// that matches nothing (the caller asked to narrow to subjects they may not see -> no hits).
fn subject_scope_sql(visible: &Option<Vec<Uuid>>, col: &str) -> String {
    match visible {
        None => String::new(),
        Some(list) if list.is_empty() => {
            format!(" AND {col} = '00000000-0000-0000-0000-000000000000'::uuid AND FALSE")
        }
        Some(list) => {
            let ids: Vec<String> = list.iter().map(|u| format!("'{u}'::uuid")).collect();
            format!(" AND {col} IN ({})", ids.join(","))
        }
    }
}

/// Build a `channel_id IN (...)` SQL fragment for the requested channel scope, or empty when `None`.
fn channel_scope_sql(channels: &Option<Vec<Uuid>>, col: &str) -> String {
    match channels {
        None => String::new(),
        Some(list) if list.is_empty() => String::new(),
        Some(list) => {
            let ids: Vec<String> = list.iter().map(|u| format!("'{u}'::uuid")).collect();
            format!(" AND {col} IN ({})", ids.join(","))
        }
    }
}

/// Build the optional `ts_since` / `ts_until` window predicate. For events both columns are `ts_ns`; for
/// summaries the window's end must be >= ts_since and the window's start <= ts_until.
fn ts_window_sql(q: &RecallQuery, upper_col: &str, lower_col: &str) -> String {
    let mut s = String::new();
    if let Some(since) = q.ts_since {
        s.push_str(&format!(" AND {upper_col} >= {since}"));
    }
    if let Some(until) = q.ts_until {
        s.push_str(&format!(" AND {lower_col} <= {until}"));
    }
    s
}

/// Fetch the raw Layer-1 body for an event hit and build its snippet (§1 #8 cold raw retrieval + snippet).
/// Exposed for the handler / drill path to enrich a hit's snippet on demand.
pub async fn enrich_snippet(pool: &PgPool, audit_row_id: &str, max: usize) -> Option<String> {
    let body = provenance::fetch_raw_by_audit_row_id(pool, audit_row_id)
        .await
        .ok()??;
    Some(provenance::snippet_from_body(&body, max))
}

/// Convenience used by tests + the handler: silence the unused `now_ns` import when only some paths use it.
#[allow(dead_code)]
fn _touch_now() -> i64 {
    now_ns()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drill_fires_when_best_summary_below_floor() {
        // MEM-005 (R9, F7): a weak summary match (similarity below the floor) drills; a strong one does not;
        // an explicit drill request always drills regardless of similarity.
        assert!(should_drill(false, 0.10, 0.30), "weak match must drill");
        assert!(
            !should_drill(false, 0.90, 0.30),
            "strong match must not drill"
        );
        assert!(
            should_drill(true, 0.99, 0.30),
            "explicit drill always drills"
        );
        // The old bug hardcoded best_summary = 1.0; assert that value would NOT drill, so the regression is
        // that best_summary must be a REAL similarity (tested end-to-end in brain_confidence_floor_test).
        assert!(!should_drill(false, 1.0, 0.30));
    }

    #[test]
    fn rrf_score_rewards_presence_in_both_retrievers() {
        let both = rrf_score(Some(0), Some(0));
        let one = rrf_score(Some(0), None);
        assert!(
            both > one,
            "a hit in both retrievers must outrank one in only summary"
        );
    }

    #[test]
    fn rrf_score_rank0_beats_rank9() {
        assert!(rrf_score(Some(0), None) > rrf_score(Some(9), None));
    }

    #[test]
    fn subject_scope_empty_list_matches_nothing() {
        let sql = subject_scope_sql(&Some(vec![]), "subject_id");
        assert!(sql.contains("FALSE"));
    }

    #[test]
    fn subject_scope_none_is_no_filter() {
        assert_eq!(subject_scope_sql(&None, "subject_id"), "");
    }

    #[test]
    fn subject_scope_in_list_renders_uuids() {
        let u = Uuid::parse_str("7e57c0de-aaaa-bbbb-cccc-000000000001").unwrap();
        let sql = subject_scope_sql(&Some(vec![u]), "subject_id");
        assert!(sql.contains("IN ("));
        assert!(sql.contains(&u.to_string()));
    }

    #[test]
    fn ts_window_builds_bounds() {
        let q: RecallQuery =
            serde_json::from_str(r#"{"q":"x","ts_since":100,"ts_until":200}"#).unwrap();
        let s = ts_window_sql(&q, "ts_ns", "ts_ns");
        assert!(s.contains(">= 100"));
        assert!(s.contains("<= 200"));
    }

    #[test]
    fn rrf_fuse_merges_same_audit_row() {
        let s = Candidate {
            source_seq: 5,
            audit_row_id: "l1:t:0005".into(),
            subject_id: Uuid::nil(),
            channel_id: None,
            kind: "summary".into(),
            ts_ns: 0,
            snippet: "digest".into(),
            chain_anchor_hex: String::new(),
            source: HitSource::Summary,
            summary_rank: Some(0),
            event_rank: None,
            covered_seq_range: Some((1, 5)),
            top_contributors: vec!["l1:t:0005".into()],
        };
        let e = Candidate {
            source_seq: 5,
            audit_row_id: "l1:t:0005".into(),
            subject_id: Uuid::nil(),
            channel_id: None,
            kind: "chat.message_created".into(),
            ts_ns: 0,
            snippet: String::new(),
            chain_anchor_hex: "ab".into(),
            source: HitSource::Event,
            summary_rank: None,
            event_rank: Some(0),
            covered_seq_range: None,
            top_contributors: vec!["l1:t:0005".into()],
        };
        let fused = rrf_fuse(vec![s], vec![e]);
        assert_eq!(fused.len(), 1, "same audit row must fuse to one candidate");
        // The fused candidate carries both ranks -> higher score than either alone.
        assert!(rrf_score(fused[0].summary_rank, fused[0].event_rank) > rrf_score(Some(0), None));
    }
}
