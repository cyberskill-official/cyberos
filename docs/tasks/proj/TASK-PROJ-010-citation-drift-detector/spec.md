---
id: TASK-PROJ-010
title: "Citation drift detector — nightly sweep flags stale MEMORY_LINKs (deleted target, superseded chain, broken memory_row_id) with operator notification"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: proj
priority: p1
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-009, TASK-MEMORY-101, TASK-MEMORY-108, TASK-OBS-007]
depends_on: [TASK-PROJ-009]
blocks: []

source_pages:
  - website/docs/modules/proj.html#citation-drift
source_decisions:
  - DEC-310 (drift detection runs nightly + on-demand; never inline with link writes)
  - DEC-311 (drift kinds — target_missing | target_superseded | scope_revoked — emit dedicated audit rows)
  - DEC-312 (notifications routed to issue assignees + tenant admins via TASK-OBS-007 CUO triage)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/src/drift/mod.rs
  - services/proj-sync/src/drift/sweep.rs
  - services/proj/tests/audit_row_test.rs
modified_files:
  # spawn nightly cron task
  - services/proj-sync/src/main.rs
  # last-sweep-at, last-known-target-version cache
  - services/proj-sync/migrations/0010_drift_state.sql
allowed_tools:
  - file_read: services/proj-sync/**, services/memory/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test drift
disallowed_tools:
  - run drift check inline with link write (per DEC-310 — async sweep only)
  - auto-remove stale links (per DEC-310 — flag only; operator decides)

effort_hours: 4
subtasks:
  - "0.5h: 0010_drift_state.sql migration"
  - "0.5h: drift/mod.rs — DriftKind enum + report struct"
  - "1.0h: sweep.rs — iterate active memory_links; check target existence + supersession chain"
  - "0.5h: notification integration via TASK-OBS-007 (sev-2 if > 10 stale; sev-3 otherwise)"
  - "0.5h: memory audit row 'proj.citation_drift_detected'"
  - "1.0h: drift_test.rs — synthetic drift fixtures + sweep correctness"
risk_if_skipped: "Without drift detection, stale links accumulate silently — operators investigating 'why is this issue linked here?' find the linked memory deleted months ago. Without supersession-chain awareness, issues link to obsolete decisions while a fresher version exists. Without auto-notification, drift hides until a manual audit; by then links may number in thousands."
---

## §1 — Description (BCP-14 normative)

The drift detector **MUST** run a nightly sweep over all active memory_links and flag stale ones. The contract:

1. **MUST** schedule a nightly cron task at 02:00 local time (configurable per `CYBEROS_DRIFT_SWEEP_CRON` env var; default `0 2 * * *`).
2. **MUST** also support on-demand sweep via `cyberos drift sweep [--tenant-id <uuid>]` CLI.
3. **MUST** detect three drift kinds:
- `TargetMissing`: linked memory_path no longer exists in memory (deletion / unwatched folder).
- `TargetSuperseded`: linked memory has been superseded by a newer memory (task-AGENTS §8 correction_to row); the link points at the older revision.
- `ScopeRevoked`: the issue's tenant lost read scope to the memory (tenant permission change since link was created).
4. **MUST** emit `proj.citation_drift_detected` memory audit row PER detected drift with payload `{link_id, issue_id, memory_path, drift_kind, detected_at_ns, prior_check_at_ns, trace_id}`.
5. **MUST** record sweep state in `drift_state` table: per-tenant `last_sweep_at`, `last_total_links_checked`, `last_drift_count_by_kind`. Operators query for "when did the last sweep run."
6. **MUST** notify via TASK-OBS-007:
- If `drift_count >= 10` in one sweep for one tenant → sev-2 alert.
- Otherwise → sev-3 (informational; surfaces in daily digest).
7. **MUST** NOT auto-remove stale links. Drift is informational; the operator decides whether to remove or accept the stale state.
8. **MUST** expose REST `GET /api/proj/drift?tenant_id=...&since=...` returning detected drifts (paginated).
9. **MUST** emit OTel metrics:
- `proj_drift_sweep_duration_seconds` (histogram).
- `proj_drift_links_checked_total` (counter).
- `proj_drift_detected_total{kind}` (counter).
10. **MUST** be deterministic given fixed memory state: same input = same drift report (no Date.now()-keyed randomness).
11. **MUST** complete within 5 minutes for a tenant with ≤ 10K active links; exceeded → sev-2 latency alarm.
12. **MUST** support delta sweeps: `cyberos drift sweep --since <timestamp>` only re-checks links touched after `<timestamp>`. Used by operators iterating fixes without re-checking everything.
13. **MUST** track per-link drift status separately from the audit row: `memory_links.drift_status` column (`healthy | target_missing | target_superseded | scope_revoked | unchecked`) updated after each sweep. UI uses this to render "stale link" badges.
14. **MUST** notify assignees (not just admins) via TASK-OBS-007: each drift event creates a per-assignee notification (queued via CUO triage per DEC-312). Assignee = current `assignee_subject_id` on the linked issue.
15. **MUST** support "suppress" workflow: operators can suppress a known-stale drift (`POST /api/proj/drift/:link_id/suppress` with reason) so it doesn't re-alert. Suppression expires after 90 days unless renewed.
16. **MUST** include a "drift severity hierarchy": within a sweep, `TargetMissing > ScopeRevoked > TargetSuperseded` (highest to lowest priority). Notifications group by severity; SEV-2 fires only on highest-severity counts.
17. **MUST** support per-tenant config override: `cyberos_proj_tenant_settings.drift_sweep_cron` overrides the default cron; `drift_sev2_threshold` overrides the 10-drift threshold.
18. **MUST** include suggested remediation in each drift event: for TargetSuperseded → suggest the successor memory_path; for TargetMissing → suggest similar memories via TASK-MEMORY-108 fuzzy search; for ScopeRevoked → suggest contacting the memory owner.
19. **MUST** support a "dry-run" mode: `cyberos drift sweep --dry-run` performs the check but skips audit row emission + notifications. Used for operator preview before committing to a sweep.
20. **MUST** include a `proj.drift_remediated` audit row when a stale link is removed OR retargeted; tracks remediation rate per tenant.
21. **MUST** support drift-trend metric: `proj_drift_trend_total{tenant_id, kind, direction}` where direction ∈ `increasing | stable | decreasing` based on 7-day rolling comparison.
22. **MUST** complete within 1 minute for tenants with ≤ 1K active links (smaller-tenant fast path); larger tenants get the 5-min budget. Smaller tenants get faster feedback.

---

## §2 — Why this design (rationale for humans)

**Why nightly + on-demand (DEC-310)?** Drift accumulates slowly (memories aren't deleted often). Nightly cadence catches the bulk; on-demand for "we just did a memory purge; please re-check now."

**Why not inline (DEC-310)?** Inline drift checks on every link query would burn memory-read budget for negligible benefit (drift detection at query-time finds the same drift drift sweep finds — but slower per-call). Async batch is the right model.

**Why flag-only (DEC-310, §1 #7)?** Auto-removing stale links is dangerous: a temporary memory outage during sweep would mass-remove valid links. Flag-only = operator-controlled remediation.

**Why three drift kinds (DEC-311)?** `TargetMissing` is the obvious one. `TargetSuperseded` is value-add: the link is technically valid but stale; operator should retarget to the newer memory. `ScopeRevoked` catches the rare-but-meaningful case where a memory was demoted from shareable to private.

**Why sev-2 at ≥ 10 drifts (§1 #6)?** Empirical: 1-2 drifts per tenant per sweep is normal background (operators delete obsolete memories). 10+ is a signal of bulk action (purge sweep, scope policy change) that operator should review.

**Why deterministic (§1 #10)?** Operator reruns sweep to confirm fix → must get the same answer. Non-determinism (e.g. random sampling) defeats the verification workflow.

**Why delta sweeps (§1 #12)?** Operators iterating on fixes don't want to re-check thousands of healthy links. `--since` confines the check to recently-touched links.

**Why drift_status column (§1 #13)?** UI rendering "stale link" badges per row needs O(1) lookup; without column, every row render = re-running sweep. Materialised column is the cache.

**Why notify assignees (§1 #14)?** Admins see drift counts; the actual operator with context to fix is the issue assignee. Per-assignee notification ensures the right person gets the alert.

**Why suppress workflow (§1 #15)?** Some drifts are known-and-accepted (linked memory archived intentionally; no fix planned). Suppression silences repeat alerts. 90-day expiry forces periodic re-evaluation.

**Why drift severity hierarchy (§1 #16)?** A tenant with 5 TargetMissing + 100 TargetSuperseded shouldn't fire SEV-2 on the latter; missing is far worse than superseded.

**Why per-tenant config (§1 #17)?** Different tenants have different drift tolerance (legal-compliance tenant wants daily sweep; SMB tenant fine with weekly). Per-tenant flexibility.

**Why suggested remediation (§1 #18)?** Operators receiving an alert want to know what to do; "memory missing — here are 3 similar paths" cuts triage time.

**Why dry-run (§1 #19)?** Operator running a sweep at non-cron time may want preview without spamming notifications.

**Why drift_remediated audit (§1 #20)?** Tracks operational health: increasing remediation rate = ops responding; flat rate with rising detection = backlog growing.

**Why trend metric (§1 #21)?** Single-day drift counts are noisy; week-over-week trend tells operators whether they're keeping up.

**Why fast-path for small tenants (§1 #22)?** Quick feedback on small tenants (1K links: 6s instead of 60s); large tenants tolerate longer.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0010_drift_state.sql

CREATE TABLE drift_state (
    tenant_id                    UUID PRIMARY KEY,
    last_sweep_at                TIMESTAMPTZ NOT NULL,
    last_total_links_checked     INT NOT NULL,
    last_drift_count_target_missing      INT NOT NULL DEFAULT 0,
    last_drift_count_target_superseded   INT NOT NULL DEFAULT 0,
    last_drift_count_scope_revoked       INT NOT NULL DEFAULT 0,
    last_sweep_duration_ms       BIGINT NOT NULL
);
```

### Rust API

```rust
// services/proj-sync/src/drift/mod.rs
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftKind { TargetMissing, TargetSuperseded, ScopeRevoked }

#[derive(Clone, Debug, Serialize)]
pub struct DriftReport {
    pub tenant_id:      uuid::Uuid,
    pub swept_at_ns:    i64,
    pub links_checked:  i32,
    pub drifts:         Vec<DriftEvent>,
    pub duration_ms:    i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DriftEvent {
    pub link_id:       uuid::Uuid,
    pub issue_id:      uuid::Uuid,
    pub memory_path:   String,
    pub drift_kind:    DriftKind,
    pub detected_at_ns: i64,
}

pub async fn sweep_tenant(
    pool: &sqlx::PgPool,
    tenant_id: uuid::Uuid,
) -> anyhow::Result<DriftReport> {
    let start = std::time::Instant::now();
    let swept_at_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap();

    // Set tenant_id context for RLS
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string()).execute(pool).await?;

    let links: Vec<crate::memory_link::MemoryLink> = sqlx::query_as(
        "SELECT * FROM memory_links WHERE removed_at IS NULL"
    ).fetch_all(pool).await?;

    let mut drifts = Vec::new();
    for link in &links {
        let drift = detect_drift_for_link(link).await;
        if let Some(kind) = drift {
            let ev = DriftEvent {
                link_id: link.id, issue_id: link.issue_id,
                memory_path: link.memory_path.clone(),
                drift_kind: kind, detected_at_ns: swept_at_ns,
            };
            emit_memory_row("proj.citation_drift_detected", serde_json::json!({
                "link_id": ev.link_id, "issue_id": ev.issue_id,
                "memory_path": ev.memory_path, "drift_kind": ev.drift_kind,
                "detected_at_ns": ev.detected_at_ns,
                "trace_id": current_trace_id(),
            })).await;
            metrics::counter!("proj_drift_detected_total", "kind" => format!("{:?}", kind)).increment(1);
            drifts.push(ev);
        }
    }

    let duration_ms = start.elapsed().as_millis() as i64;
    sqlx::query(
        "INSERT INTO drift_state
           (tenant_id, last_sweep_at, last_total_links_checked,
            last_drift_count_target_missing, last_drift_count_target_superseded,
            last_drift_count_scope_revoked, last_sweep_duration_ms)
         VALUES ($1, NOW(), $2, $3, $4, $5, $6)
         ON CONFLICT (tenant_id) DO UPDATE SET
           last_sweep_at = EXCLUDED.last_sweep_at,
           last_total_links_checked = EXCLUDED.last_total_links_checked,
           last_drift_count_target_missing = EXCLUDED.last_drift_count_target_missing,
           last_drift_count_target_superseded = EXCLUDED.last_drift_count_target_superseded,
           last_drift_count_scope_revoked = EXCLUDED.last_drift_count_scope_revoked,
           last_sweep_duration_ms = EXCLUDED.last_sweep_duration_ms"
    )
    .bind(tenant_id).bind(links.len() as i32)
    .bind(drifts.iter().filter(|d| d.drift_kind == DriftKind::TargetMissing).count() as i32)
    .bind(drifts.iter().filter(|d| d.drift_kind == DriftKind::TargetSuperseded).count() as i32)
    .bind(drifts.iter().filter(|d| d.drift_kind == DriftKind::ScopeRevoked).count() as i32)
    .bind(duration_ms).execute(pool).await?;

    metrics::histogram!("proj_drift_sweep_duration_seconds").record(start.elapsed().as_secs_f64());
    metrics::counter!("proj_drift_links_checked_total").increment(links.len() as u64);

    // Notification
    if drifts.len() >= 10 {
        obs::alert(obs::Severity::Sev2, "citation_drift_high",
            serde_json::json!({"tenant_id": tenant_id, "drift_count": drifts.len()})).await;
    } else if !drifts.is_empty() {
        obs::alert(obs::Severity::Sev3, "citation_drift_detected",
            serde_json::json!({"tenant_id": tenant_id, "drift_count": drifts.len()})).await;
    }

    Ok(DriftReport { tenant_id, swept_at_ns, links_checked: links.len() as i32, drifts, duration_ms })
}

async fn detect_drift_for_link(link: &crate::memory_link::MemoryLink) -> Option<DriftKind> {
    // 1. TargetMissing: memory doesn't exist
    let memory = match memory_reader::find_memory(&link.memory_path).await {
        Some(m) => m,
        None    => return Some(DriftKind::TargetMissing),
    };

    // 2. ScopeRevoked: memory's sync_class changed to private (was shareable when linked)
    if memory.sync_class == memory::SyncClass::Private {
        // Compare to historical scope; if previously shareable and now private → drift
        if was_shareable_at_link_create(&memory, &link.created_at).await {
            return Some(DriftKind::ScopeRevoked);
        }
    }

    // 3. TargetSuperseded: a newer memory exists with `correction_to` pointing at this one
    if memory_reader::has_successor(&memory.row_id).await {
        return Some(DriftKind::TargetSuperseded);
    }

    None
}
```

---

## §4 — Acceptance criteria

1. **TargetMissing detected** — link to deleted memory → DriftKind::TargetMissing reported.
2. **TargetSuperseded detected** — newer memory with correction_to → DriftKind::TargetSuperseded.
3. **ScopeRevoked detected** — memory flipped shareable → private after link → DriftKind::ScopeRevoked.
4. **Healthy links not flagged** — happy memory + healthy link → no drift events.
5. **Sweep deterministic** — same DB state → same report on rerun.
6. **drift_state row updated** — sweep completion → drift_state has row with counts.
7. **memory audit per drift** — N drifts → N `proj.citation_drift_detected` rows.
8. **Sev-2 alert at ≥ 10** — fixture with 10 drifts → TASK-OBS-007 sev-2 alert fired.
9. **Sev-3 alert at < 10** — fixture with 3 drifts → sev-3 alert.
10. **No alert on zero drifts** — happy sweep → no alert.
11. **CLI works** — `cyberos drift sweep --tenant-id <uuid>` → exit 0 + report JSON.
12. **REST GET filters by tenant** — tenant A's report invisible to tenant B.
13. **RLS isolates drift_state** — tenant A cannot read tenant B's drift_state.
14. **Auto-removal NOT performed** — sweep run + drift detected → memory_links row unchanged (still active).
15. **Sweep latency ≤ 5min for 10K links** — fixture; assert duration_ms < 300000.
16. **OTel metrics emitted** — histogram + counters populated.
17. **Delta sweep checks only recent links** — `--since <ts>` → only links touched after ts re-checked (AC for §1 #12).
18. **drift_status updated post-sweep** — links marked appropriately (AC for §1 #13).
19. **Assignee notifications fired** — drift on issue with assignee → notification routed to assignee (AC for §1 #14).
20. **Suppress silences alert** — suppressed link doesn't re-alert; 90d expiry then re-alerts (AC for §1 #15).
21. **Severity hierarchy gates SEV-2** — 5 TargetMissing + 100 TargetSuperseded → SEV-2 fires for missing not superseded (AC for §1 #16).
22. **Tenant config override** — set tenant `drift_sev2_threshold=20`; 10 drifts no longer triggers SEV-2 (AC for §1 #17).
23. **Suggested remediation included** — TargetMissing event has `suggested_paths` array (AC for §1 #18).
24. **Dry-run skips audit + notification** — `--dry-run` → no audit row, no alert (AC for §1 #19).
25. **drift_remediated audit on operator fix** — operator removes stale link → audit row emitted (AC for §1 #20).
26. **Trend metric reflects 7d direction** — 7 days of increasing drifts → direction=increasing (AC for §1 #21).
27. **Small-tenant fast path** — tenant with 500 links → sweep completes in <60s (AC for §1 #22).

---

## §5 — Verification

```rust
#[tokio::test]
async fn target_missing_detected() {
    let env = TestEnv::new().await;
    let (issue, mem) = env.setup_link().await;
    env.delete_memory(&mem).await;   // make target missing
    let report = sweep_tenant(&env.pool, env.tenant_id()).await.unwrap();
    let drift = report.drifts.iter().find(|d| d.memory_path == mem).unwrap();
    assert_eq!(drift.drift_kind, DriftKind::TargetMissing);
}

#[tokio::test]
async fn superseded_detected() {
    let env = TestEnv::new().await;
    let mem_v1 = env.create_memory_v1().await;
    let _ = env.create_link_to(mem_v1.clone()).await;
    env.create_correction_memory(&mem_v1).await;
    let report = sweep_tenant(&env.pool, env.tenant_id()).await.unwrap();
    assert!(report.drifts.iter().any(|d| d.drift_kind == DriftKind::TargetSuperseded));
}

#[tokio::test]
async fn deterministic_rerun_same_report() {
    let env = TestEnv::new().await;
    env.setup_two_drifts().await;
    let r1 = sweep_tenant(&env.pool, env.tenant_id()).await.unwrap();
    let r2 = sweep_tenant(&env.pool, env.tenant_id()).await.unwrap();
    let kinds1: Vec<_> = r1.drifts.iter().map(|d| (d.link_id, d.drift_kind)).collect();
    let kinds2: Vec<_> = r2.drifts.iter().map(|d| (d.link_id, d.drift_kind)).collect();
    assert_eq!(kinds1, kinds2);
}

#[tokio::test]
async fn no_auto_remove() {
    let env = TestEnv::new().await;
    let (link, _) = env.setup_drift().await;
    let _ = sweep_tenant(&env.pool, env.tenant_id()).await.unwrap();
    let still_active: bool = sqlx::query_scalar(
        "SELECT removed_at IS NULL FROM memory_links WHERE id = $1"
    ).bind(link).fetch_one(&env.pool).await.unwrap();
    assert!(still_active);
}

#[tokio::test]
async fn sev_2_alert_at_high_drift() {
    let env = TestEnv::new().await;
    env.setup_n_drifts(12).await;
    let _ = sweep_tenant(&env.pool, env.tenant_id()).await.unwrap();
    let alert = env.obs.latest_alert().await;
    assert_eq!(alert.severity, "sev-2");
    assert_eq!(alert.kind, "citation_drift_high");
}
```

---

## §6 — Implementation skeleton

(API + DB above.)

---

## §7 — Dependencies

- **TASK-PROJ-009** — memory_links table consumed.
- **TASK-MEMORY-101** — MemoryReader for find_memory + has_successor.
- **TASK-MEMORY-108** — search API.
- **TASK-OBS-007** — alert routing.

---

## §8 — Example payloads

```json
{
  "kind": "proj.citation_drift_detected",
  "payload": {
    "link_id": "lk-...",
    "issue_id": "iss-...",
    "memory_path": "memories/projects/cyberos/decisions/DEC-220.md",
    "drift_kind": "target_superseded",
    "detected_at_ns": 1747407137483000000,
    "prior_check_at_ns": 1747320737483000000,
    "trace_id": "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Periodic sub-tenant sweep (per-engagement) — slice 4+.
- Drift remediation workflow (auto-retarget to successor) — slice 4+; risky.
- Real-time drift detection via memory chain hooks — slice 4+; more complex.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| memory reader unavailable | find_memory Err | Sweep aborts; sev-1 alarm | Operator restores memory |
| Tenant has 100K+ links | sweep exceeds 5min | sev-2 latency alarm | Slice 4+ paginate sweep |
| drift_state INSERT fails | sqlx Err | Sweep result lost; sev-2 | Operator investigates DB |
| Concurrent sweep + new link write | RLS isolates per-tenant; race tolerable | Either link checked or skipped (consistent next sweep) | None |
| memory search transient error | retry 3× | Drift report unreliable for that link | Re-run sweep |
| Massive supersession event (1000 memories) | drifts spike | sev-2 alert fires; ops notified | Operator reviews |
| Sweep starts but server restart | partial state | drift_state shows old sweep; next sweep covers | None |
| Drift detection itself bugged (false positives) | property tests catch | CI blocked | Author fixes |
| Operator deletes drift_state | recreated on next sweep | Historical sweep info lost | Acceptable |
| OBS alert exporter down | metric buffered | Notification delayed | Operator restores TASK-OBS-001 |
| RLS bypass | RLS policy | 0 rows | None |
| Memory path encoding inconsistencies | normalise at TASK-MEMORY-108 | Consistent | None |
| Delta sweep with invalid `--since` | rejects | 400 | Caller |
| drift_status column out of sync (race) | next sweep corrects | brief inconsistency | None |
| Assignee subject_id stale (deleted user) | notification falls back to admin | None | None |
| Suppress expires mid-sweep | counted as un-suppressed | next sweep alerts | None |
| Multiple suppressions for one link | latest wins | None | None |
| Severity hierarchy disabled (config error) | all drifts contribute | SEV-2 may fire on superseded only | Operator |
| Per-tenant cron config invalid (e.g. `0 99 * * *`) | startup rejects | log + tenant uses default | None |
| Remediation suggestion fuzzy-search fails | fallback to empty | drift event still emitted | Operator |
| Dry-run leaks audit (bug) | property test catches | None | Author fixes |
| drift_remediated audit on no-op (false detection) | rare; matched against state | None | None |
| Trend metric direction calculation off-by-one | tested via fixtures | None | Author |
| Small-tenant fast-path crosses threshold | switches to slow path | brief reclassification | None |
| Suppress without reason | 400 | None | Caller |
| Tenant deleted mid-sweep | sweep aborts gracefully | None | None |
| Concurrent drift sweep + link create | new link unchecked this run | next sweep covers | None |
| Drift on archived issue | still counted (issue closed but link active) | operator reviews | None |
| memory reader 5xx burst | exp backoff retry | sweep slower; eventually completes | Operator |
| Suggested remediation with PII in path | should be redacted | event emit safe | None |
| Operator suppresses then deletes link | suppression auto-cleared | None | None |

---

## §11 — Implementation notes

- Cron is implemented via tokio-cron-scheduler in the proj-sync main loop.
- `was_shareable_at_link_create` queries memory audit chain for `meta.sync_class` history at the link's `created_at` timestamp.
- `has_successor` queries memory for rows with `correction_to = <row_id>`.
- The 5-minute latency budget for 10K links assumes memory search latency p99 ≤ 30ms — borderline; slice 4+ may need pagination + parallel sweep.
- Multi-tenant sweep iterates tenants serially to avoid memory-side rate limits.
- The `proj.citation_drift_detected` audit row's `prior_check_at_ns` field is the prior sweep's timestamp from drift_state; null if no prior sweep.
- Drift remediation is intentionally manual; auto-remediation pattern reserved for slice 4+.
- Delta sweep uses `memory_links.updated_at` (touched on any mutation including drift_status update); `--since` filters by that.
- drift_status column materialises the latest sweep result; UI badge rendering reads it in O(1).
- Assignee notification queues via TASK-OBS-007's CUO triage; the notification appears in the assignee's daily digest.
- Suppression rows live in `memory_link_drift_suppressions` table with link_id + suppressed_at + reason + expires_at; sweep joins this table.
- Severity hierarchy is implemented in the alert-decision function; not a column.
- Per-tenant config lives in `cyberos_proj_tenant_settings` (extended); cron validates at startup.
- Suggested remediation for TargetMissing uses TASK-MEMORY-108 `search(path_fragments)` with edit-distance scoring; returns top 3.
- Dry-run is a CLI flag wired through to the sweep function; toggled before audit emit + alert.
- `proj.drift_remediated` is emitted by the `remove_link` and (slice 4+) `retarget_link` handlers; carries `prior_drift_kind`.
- Trend metric uses a 7-day rolling window stored in `drift_trend_state` table; comparison is rate-of-change.
- Small-tenant fast path is selected based on `last_total_links_checked` at sweep start; threshold 1K.
- Suppression auto-expiry runs in a separate cleanup task hourly.
- We considered triggering sweep on every link deletion in memory but rejected: link write rate is high; sweep on every event = O(N²) churn.
- Drift detection ON_DEMAND CLI is intended for operators investigating an issue ("did this drift since last sweep?"); per-link granular check.
- The drift_state row updates atomically with the sweep completion; no partial rows.
- Assignee notification dedups within 24h: one drift event per (link, assignee) per day, no spam.
- The remediation `suggested_paths` array is bounded at 3; UI shows them as quick-action buttons.
- We rejected real-time drift detection via memory chain hooks because: (a) added complexity; (b) batch sweep finds the same drift; (c) memory event firehose would need filtering anyway.

---

*End of TASK-PROJ-010.*
