---
id: FR-AUTH-109
title: "AUTH stub → full migration enforcer — 30-day grace window + cutover timestamp + rejection metric + per-tenant override"
module: AUTH
priority: MUST
status: ready_to_test
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-AUTH-004, FR-AUTH-101, FR-AUTH-108, FR-AI-003, FR-MEMORY-101, FR-OBS-007]
depends_on: [FR-AUTH-101]
blocks: []

source_pages:
  - website/docs/modules/auth.html#stub-stack
source_decisions:
  - DEC-440 (grace window 30 days from FR-AUTH-101 ship date; configurable per tenant via tenant policy override; minimum 7 days; maximum 90 days)
  - DEC-441 (stub-era tokens are FR-AUTH-004 tokens issued before FR-AUTH-101 ship — they lack the `rbac_v` claim entirely; presence of `rbac_v` claim = post-FR-AUTH-101 token)
  - DEC-442 (cutover_at timestamp is a global per-tenant config row in `auth_migration_state` table; UPDATE-able by root-admin only; immutable post-grace-expiry)
  - DEC-443 (memory audit kinds: auth.stub_token_accepted, auth.stub_token_rejected, auth.grace_window_extended, auth.cutover_completed)
  - DEC-444 (rejection metric `auth_stub_token_rejected_total{tenant_id}` — sev-2 alarm at sustained > 100/h indicates many users haven't refreshed)
  - DEC-445 (per-tenant override via `POST /v1/auth/migration/extend-grace` — caller MUST have root-admin role; extension up to 60 additional days; one extension per tenant; second extension → 409 already_extended)
  - DEC-446 (REVOKE UPDATE, DELETE on auth_migration_state from cyberos_app — append-only at SQL grant for cutover history; UPDATE allowed only via inv_provisioner role)
  - DEC-447 (operator preview API `GET /v1/auth/migration/preview` returns count of stub tokens issued, count refreshed, count still-stub; helps operators decide whether to extend grace)
  - DEC-448 (rejection response on post-grace stub token: 401 with body `{"error":"rbac_version_required","reason":"stub_token_grace_window_expired","grace_expired_at":"<iso8601>","action_required":"refresh"}` — gives client clear action)
  - DEC-449 (refresh trigger — when FR-AUTH-004 refresh sees a token without `rbac_v`, the refreshed token MUST include `rbac_v` at current value; the verifier records the refresh in `auth_token_refresh_log`)
  - DEC-450 (post-cutover, the `auth_migration_state.status` flips from `grace_active` to `cutover_completed`; the verifier path simplifies: every token MUST have `rbac_v`)

language: rust 1.81 + sql
service: cyberos/services/auth/
new_files:
  - services/auth/migrations/0014_auth_migration_state.sql
  - services/auth/migrations/0015_auth_token_refresh_log.sql
  - services/auth/src/migration/mod.rs
  - services/auth/src/migration/state.rs                  # cutover_at lookup + status
  - services/auth/src/migration/grace_window.rs           # is_in_grace_window + days_remaining
  - services/auth/src/migration/verifier_hook.rs          # hook into FR-AUTH-004 verifier to reject post-grace stub tokens
  - services/auth/src/migration/refresh_hook.rs           # hook into FR-AUTH-004 refresh to inject rbac_v
  - services/auth/src/migration/audit.rs                  # 4 memory row builders
  - services/auth/src/handlers/migration.rs               # GET /preview + POST /extend-grace
  - services/auth/tests/migration_state_test.rs
  - services/auth/tests/grace_window_calc_test.rs
  - services/auth/tests/stub_token_accepted_during_grace_test.rs
  - services/auth/tests/stub_token_rejected_post_grace_test.rs
  - services/auth/tests/refresh_hook_injects_rbac_v_test.rs
  - services/auth/tests/extend_grace_test.rs
  - services/auth/tests/extend_grace_root_admin_only_test.rs
  - services/auth/tests/cutover_immutable_test.rs
  - services/auth/tests/preview_counts_test.rs
  - services/auth/tests/append_only_migration_state_test.rs
  - services/auth/tests/audit_emission_test.rs
modified_files:
  - services/auth/src/jwt/verifier.rs                     # hook in migration::verifier_hook before successful verify
  - services/auth/src/jwt/refresh.rs                      # hook in migration::refresh_hook after refresh
  - services/auth/src/lib.rs                              # pub mod migration

allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test migration

disallowed_tools:
  - allow grace extension beyond 60 days (per DEC-445)
  - allow second extension per tenant (per DEC-445)
  - skip rbac_v injection on refresh (per DEC-449)
  - allow stub tokens post-cutover_completed (per DEC-450)
  - allow non-root-admin to extend grace (per DEC-445)
  - allow stub-token rejection to silently 200 (per DEC-448 — must return clear 401 + reason)

effort_hours: 5
sub_tasks:
  - "0.4h: 0014_auth_migration_state.sql — per-tenant cutover state + append-only history"
  - "0.3h: 0015_auth_token_refresh_log.sql — append-only refresh events"
  - "0.4h: state.rs — cutover_at lookup + status"
  - "0.4h: grace_window.rs — calc + days_remaining"
  - "0.5h: verifier_hook.rs — reject post-grace stubs"
  - "0.5h: refresh_hook.rs — inject rbac_v on refresh"
  - "0.4h: audit.rs — 4 memory row builders"
  - "0.5h: handlers/migration.rs — preview + extend-grace"
  - "1.6h: tests — 10 test files"

risk_if_skipped: "Without the migration enforcer, FR-AUTH-101's `rbac_v` claim is advisory — stub-era tokens continue valid indefinitely, defeating the purpose of replay-resistance. Operators lose visibility into 'how many users haven't refreshed yet'; OBS has no alarm signal when stub-token rejection starts climbing. Without DEC-448's clear rejection reason, clients see opaque 401s + don't know to refresh. Without DEC-449's refresh hook, refreshed tokens still lack rbac_v + the grace never actually ends. Without DEC-445's extension override, an operator with a stuck client base has no escape valve. The 5h effort closes the loop: FR-AUTH-101 ships the claim, FR-AUTH-109 enforces it after grace."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** enforce the FR-AUTH-101 stub → full migration via the 30-day grace window + rejection enforcer + per-tenant extension override. Each requirement:

1. **MUST** define `auth_migration_state` table per-tenant: `(tenant_id UUID PRIMARY KEY, cutover_at TIMESTAMPTZ NOT NULL, status TEXT NOT NULL CHECK (status IN ('grace_active','grace_extended','cutover_completed')), grace_extended_at TIMESTAMPTZ, grace_extension_reason TEXT, extension_count INT NOT NULL DEFAULT 0 CHECK (extension_count BETWEEN 0 AND 1), created_at TIMESTAMPTZ NOT NULL DEFAULT now())`. One row per tenant.

2. **MUST** define `auth_token_refresh_log` table: `(id BIGSERIAL, tenant_id UUID, subject_id UUID, prior_rbac_v_present BOOLEAN NOT NULL, new_rbac_v INT NOT NULL, refreshed_at TIMESTAMPTZ NOT NULL DEFAULT now())`. `REVOKE UPDATE, DELETE FROM cyberos_app` (per DEC-446).

3. **MUST** enforce RLS with both `USING` and `WITH CHECK` on `auth_migration_state` (root-admin only — same pattern as FR-TEN-001's tenants table; consumers read via internal helper) and `auth_token_refresh_log` (tenant-scoped).

4. **MUST** seed `auth_migration_state` for every existing tenant at FR-AUTH-101 ship time: cutover_at = ship_date + 30 days, status = `grace_active`. New tenants (provisioned after ship date) get cutover_at = now() + 30 days at provisioning time (FR-TEN-001 hook).

5. **MUST** ship the verifier hook (per DEC-442 + DEC-448). When FR-AUTH-004's verifier validates a JWT:
    - If token's `rbac_v` claim is present → continue normal verification.
    - If `rbac_v` is absent → look up `auth_migration_state` for the token's `tid` claim.
    - If `cutover_at > now()` → accept; emit `auth.stub_token_accepted` memory row (sampled 1%).
    - If `cutover_at <= now()` → reject with 401 + body per DEC-448; emit `auth.stub_token_rejected` memory row.

6. **MUST** ship the refresh hook (per DEC-449). When FR-AUTH-004's refresh path issues a new access token:
    - If prior token lacked `rbac_v` → log the refresh in `auth_token_refresh_log` with `prior_rbac_v_present=false`; new token MUST include `rbac_v` at current value.
    - If prior token had `rbac_v` → log with `prior_rbac_v_present=true`; new token includes current `rbac_v` (may differ from prior if catalogue version bumped).

7. **MUST** ship `POST /v1/auth/migration/extend-grace` handler (per DEC-445). Body: `{additional_days: 1-60, reason: "<text 1-500 chars>"}`. Caller MUST have role `root-admin` per FR-AUTH-101. Validates:
    - Tenant's current status is `grace_active` (extended → 409 already_extended).
    - additional_days in [1, 60].
    - reason non-empty.
   On success: UPDATE `auth_migration_state` SET cutover_at = cutover_at + additional_days, status = `grace_extended`, grace_extended_at = now(), grace_extension_reason = reason, extension_count = 1. Emit `auth.grace_window_extended` memory row.

8. **MUST** ship `GET /v1/auth/migration/preview` returning `{tenant_id, cutover_at, status, days_remaining, stub_tokens_accepted_count_24h, stub_tokens_rejected_count_24h, refreshed_count_24h, extension_count, can_extend: bool}`. Caller MUST have role `root-admin` per FR-AUTH-101 OR `tenant-admin` (can see their own tenant's preview).

9. **MUST** ensure cutover_at is immutable post-grace-expiry (per DEC-442). A `BEFORE UPDATE` trigger on `auth_migration_state` rejects mutation of `cutover_at` when `status = 'cutover_completed'`. Cutover transition (grace_active → cutover_completed) happens automatically when verifier observes `cutover_at <= now()` on first request; the trigger UPDATEs status atomically.

10. **MUST** transition status `grace_active` (or `grace_extended`) → `cutover_completed` exactly once per tenant — the first verifier request after `cutover_at <= now()`. Emit `auth.cutover_completed` memory row.

11. **MUST** emit memory audit rows for 4 kinds (per DEC-443):
    - `auth.stub_token_accepted` — sampled 1% during grace; 100% near grace end (last 24h).
    - `auth.stub_token_rejected` — every rejection; carries `tenant_id`, `subject_id_hash16`, `cutover_at`, `now`.
    - `auth.grace_window_extended` — every extension; carries `tenant_id`, `prior_cutover_at`, `new_cutover_at`, `additional_days`, `reason_scrubbed`, `extended_by_subject_id_hash16`.
    - `auth.cutover_completed` — single emission per tenant on transition; carries `tenant_id`, `final_cutover_at`, `total_stub_accepted`, `total_stub_rejected`.

12. **MUST** PII-scrub `grace_extension_reason` via FR-MEMORY-111 before chain commit.

13. **MUST** complete the verifier hook in ≤ 5 ms p99 (in-memory state cache; DB read only on cache miss). The verifier hook is on the hot path of every JWT verification — must be fast.

14. **MUST** cache `auth_migration_state` per-tenant in-memory with 60s TTL. Cache invalidated on extension or cutover-completion via explicit invalidation message.

15. **MUST** emit OTel span `auth.migration.{verify_stub,refresh,extend_grace,cutover}` with attributes: `tenant_id`, `outcome` (accepted | rejected | extended | already_extended | cutover_triggered | not_root_admin).

16. **MUST** emit OTel metrics:
    - `auth_stub_token_accepted_total{tenant_id}` (counter — should approach zero post-grace).
    - `auth_stub_token_rejected_total{tenant_id}` (counter — alarm sev-2 at sustained > 100/h per DEC-444).
    - `auth_token_refresh_total{tenant_id, prior_rbac_v_present}` (counter).
    - `auth_grace_extensions_total{tenant_id}` (counter; max 1 per tenant).
    - `auth_cutover_completed_total{tenant_id}` (counter; max 1 per tenant).
    - `auth_migration_grace_days_remaining{tenant_id}` (gauge — periodic compute).

17. **MUST** support the `auth_provisioner` SQL role distinct from `cyberos_app` for INSERT into `auth_migration_state`. INSERT/UPDATE granted to `auth_provisioner`; UPDATE on `status` granted only to `auth_provisioner` (controlled cutover); REVOKE UPDATE, DELETE from cyberos_app.

18. **MUST** validate extension `additional_days` is integer in [1, 60]; outside → 400 `additional_days_out_of_range`.

19. **MUST** track in OBS sev-2 alarm when `auth_stub_token_rejected_total{tenant_id}` rate > 100/h sustained over 1h. Rule lives in FR-OBS-007's rule set; this FR documents the alarm contract.

20. **MUST** persist the seed at FR-AUTH-101 ship time via a one-off migration that INSERTs `auth_migration_state` for every existing tenant. The migration is idempotent (uses `INSERT ... ON CONFLICT DO NOTHING`).

21. **MUST** make `days_remaining` calculation in `GET /preview` derive from `(cutover_at - now()).days` clipped at 0; reflect cutover_completed status correctly (returns 0 + status).

22. **MUST** emit `auth.cutover_completed` ONLY ONCE per tenant — the trigger uses `WHERE status != 'cutover_completed'` predicate to enforce. Subsequent requests after cutover see status=`cutover_completed` and the verifier rejects stub tokens immediately without re-emitting cutover row.

23. **MUST** keep cutover backward-flow forbidden — `cutover_completed → grace_active` rejection at trigger.

24. **MUST** allow operators to query `GET /v1/auth/migration/refresh-events?from=<ts>&to=<ts>` for refresh-log inspection. Caller MUST have root-admin role.

25. **MUST** support env-driven `AUTH_MIGRATION_DEFAULT_GRACE_DAYS` (default 30; min 7; max 90 — enforced at startup).

---

## §2 — Why this design (rationale for humans)

**Why 30-day grace default (DEC-440)?** Typical refresh-token cycle is 7 days; 30 days covers 4 refresh cycles + safety margin. Below 30 risks too many active stub tokens at cutover; above 30 delays the security benefit (rbac_v replay-resistance). 30 is the deliberate middle.

**Why per-tenant cutover_at not global (DEC-442)?** Different tenants ship at different dates. Existing tenants get cutover_at = ship_date + 30 days at FR-AUTH-101 ship; new tenants get cutover_at = provision_date + 30 days. Global cutover would force later-provisioned tenants into immediate cutover — operationally cruel.

**Why one extension max per tenant (DEC-445)?** Extension is escape valve for stuck client base, not a recurring policy lever. Allowing repeated extensions delays the security benefit indefinitely. One 60-day max extension = total 90 days possible — beyond that, the tenant needs to address whatever's blocking client refresh.

**Why root-admin role for extension (DEC-445)?** Cross-tenant operation (CyberSkill ops decides extension). Tenant-admin doesn't have authority — could grant indefinite stub-token validity for their own tenant.

**Why clear 401 rejection body (DEC-448)?** Clients seeing opaque 401 don't know to refresh. The body with `reason: "stub_token_grace_window_expired"` + `action_required: "refresh"` tells the client exactly what to do. Standard 401 alone leaves clients guessing.

**Why refresh hook on FR-AUTH-004 path (DEC-449)?** Refresh is the natural cutover point — old token returns access token + refresh token; new access token gets rbac_v injected. No client code change needed; refresh is opaque from client perspective.

**Why 1% sampling on stub_token_accepted during grace + 100% near grace end (§1 #11)?** Volume-vs-visibility tradeoff. Early in grace, expect millions of stub-token acceptances (cost issue if 100% logged). Near grace end, fewer remaining stub tokens — 100% gives operator visibility into which subjects are still stub.

**Why cutover trigger on first post-grace request (§1 #10)?** Lazy transition — no need for a scheduled job. The verifier path naturally observes `cutover_at <= now()` + transitions status. Idempotent via the `status != 'cutover_completed'` predicate.

**Why cache 60s TTL (§1 #14)?** Verifier hot path can't afford per-request DB read. 60s cache means extension propagation has 60s lag (acceptable for ops events; not real-time).

**Why preview API (§1 #8, DEC-447)?** Operators decide whether to extend grace based on visibility. "100 users still on stub tokens 7 days from cutover" — extend. "5 users on stub tokens" — don't extend (those users need to refresh). Preview gives the data.

**Why append-only migration_state (DEC-446)?** Cutover history is forensic — "when did this tenant's grace expire?" is a compliance question. UPDATE on `status` is the only mutation needed; rest is INSERT-only (extension creates extension_count = 1 update + grace_extended_at update; both granted to auth_provisioner role).

**Why sev-2 alarm at > 100/h rejected (DEC-444)?** Normal cutover should produce zero rejections (all subjects refreshed during grace). > 100/h sustained signals (a) bulk client misbehaviour (didn't refresh), (b) operator missed the grace deadline, (c) deployment regression. Sev-2 = operator investigation.

**Why `auth_provisioner` SQL role (§1 #17)?** Same defense-in-depth pattern as FR-TEN-001. App code can't accidentally mutate migration state; only the explicit migration handler (with provisioner role) can.

**Why grace bounds 7-90 days (DEC-440)?** 7 days minimum covers one refresh cycle; below that some clients can't refresh in time. 90 days maximum prevents indefinite grace creep (defeats the security benefit). The env-driven default within these bounds is operator-tunable.

**Why `extension_count` bounded at 1 (DEC-445)?** SQL CHECK enforces; second extension attempt → 409. Hard cap.

**Why PII-scrub `grace_extension_reason` (§1 #12)?** Operator-supplied text may mention specific clients or stuck-user identifiers. Scrubbed in memory chain; raw in tenant-scoped Postgres.

**Why `auth.cutover_completed` once per tenant (§1 #22, §1 #10)?** Idempotent state transition. Trigger predicate `status != 'cutover_completed'` ensures single emission. Subsequent rejections continue to emit `auth.stub_token_rejected` for each event.

**Why no automatic cutover scheduling job (§1 #10)?** Verifier-driven lazy transition is simpler: no cron, no missed-window risk, transition happens on first stub-token request after grace expiry. The first rejection IS the cutover.

**Why FR-AUTH-109 has no `blocks: []`?** FR-AUTH-101's contract claims the grace window will be enforced; FR-AUTH-109 ships the enforcer. No downstream FR needs to know about FR-AUTH-109 — it's the implementation of an existing promise.

**Why refresh log captures `prior_rbac_v_present`?** Operator analytics: "how many refreshes converted from stub to full?" — the count of `prior_rbac_v_present=false` rows = exactly the number of stub-to-full conversions. Useful for predicting cutover readiness.

**Why `additional_days` max 60 (DEC-445)?** Beyond 60 days, the operator should be addressing root causes (client refresh broken, deployment stuck). 60-day cap prevents indefinite extension-as-policy.

---

## §3 — API contract

### 3.1 — Migration 0014 — auth_migration_state

```sql
-- services/auth/migrations/0014_auth_migration_state.sql

BEGIN;

CREATE TABLE auth_migration_state (
    tenant_id                UUID         PRIMARY KEY,
    cutover_at               TIMESTAMPTZ  NOT NULL,
    status                   TEXT         NOT NULL CHECK (status IN ('grace_active','grace_extended','cutover_completed')),
    grace_extended_at        TIMESTAMPTZ,
    grace_extension_reason   TEXT         CHECK (grace_extension_reason IS NULL OR length(grace_extension_reason) BETWEEN 1 AND 500),
    extension_count          INT          NOT NULL DEFAULT 0 CHECK (extension_count BETWEEN 0 AND 1),
    created_at               TIMESTAMPTZ  NOT NULL DEFAULT now()
);

ALTER TABLE auth_migration_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY auth_migration_state_root_admin ON auth_migration_state
    USING (current_setting('auth.is_root_admin', true) = 'true'
           OR tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (current_setting('auth.is_root_admin', true) = 'true');

REVOKE UPDATE, DELETE ON auth_migration_state FROM cyberos_app;

-- auth_provisioner role for cutover + extension flows
CREATE ROLE auth_provisioner;
GRANT INSERT ON auth_migration_state TO auth_provisioner;
GRANT UPDATE (cutover_at, status, grace_extended_at, grace_extension_reason, extension_count) ON auth_migration_state TO auth_provisioner;
GRANT SELECT ON auth_migration_state TO cyberos_app, auth_provisioner;

-- Immutability post-cutover (DEC-442)
CREATE OR REPLACE FUNCTION enforce_cutover_immutable() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'cutover_completed' AND NEW.cutover_at IS DISTINCT FROM OLD.cutover_at THEN
        RAISE EXCEPTION 'cutover_immutable_post_completion' USING ERRCODE = 'P0080';
    END IF;
    IF OLD.status = 'cutover_completed' AND NEW.status = 'grace_active' THEN
        RAISE EXCEPTION 'cutover_status_backward_forbidden' USING ERRCODE = 'P0081';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_migration_state_immutable BEFORE UPDATE ON auth_migration_state
    FOR EACH ROW EXECUTE FUNCTION enforce_cutover_immutable();

-- Seed for existing tenants at ship time (idempotent)
INSERT INTO auth_migration_state (tenant_id, cutover_at, status)
SELECT id, now() + INTERVAL '30 days', 'grace_active' FROM tenants
ON CONFLICT (tenant_id) DO NOTHING;

COMMIT;
```

### 3.2 — Migration 0015 — refresh_log

```sql
-- services/auth/migrations/0015_auth_token_refresh_log.sql

BEGIN;

CREATE TABLE auth_token_refresh_log (
    id                       BIGSERIAL    PRIMARY KEY,
    tenant_id                UUID         NOT NULL,
    subject_id               UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    prior_rbac_v_present     BOOLEAN      NOT NULL,
    new_rbac_v               INT          NOT NULL,
    refreshed_at             TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX refresh_log_tenant_refreshed_idx ON auth_token_refresh_log (tenant_id, refreshed_at DESC);
CREATE INDEX refresh_log_prior_present_idx ON auth_token_refresh_log (tenant_id, prior_rbac_v_present, refreshed_at DESC);

ALTER TABLE auth_token_refresh_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY refresh_log_tenant_iso ON auth_token_refresh_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON auth_token_refresh_log FROM cyberos_app;

COMMIT;
```

### 3.3 — Migration state lookup

```rust
// services/auth/src/migration/state.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MigrationState {
    pub tenant_id: Uuid,
    pub cutover_at: DateTime<Utc>,
    pub status: MigrationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStatus { GraceActive, GraceExtended, CutoverCompleted }

pub struct MigrationStateCache {
    inner: Arc<RwLock<HashMap<Uuid, (MigrationState, Instant)>>>,
}

impl MigrationStateCache {
    pub async fn get(&self, tenant_id: Uuid, db: &PgPool) -> anyhow::Result<MigrationState> {
        // Cache check
        if let Some((state, t)) = self.inner.read().await.get(&tenant_id) {
            if t.elapsed() < Duration::from_secs(60) {
                return Ok(state.clone());
            }
        }
        // DB load
        let row: (Uuid, DateTime<Utc>, String) = sqlx::query_as(
            "SELECT tenant_id, cutover_at, status FROM auth_migration_state WHERE tenant_id = $1"
        ).bind(tenant_id).fetch_one(db).await?;
        let state = MigrationState {
            tenant_id: row.0,
            cutover_at: row.1,
            status: match row.2.as_str() {
                "grace_active" => MigrationStatus::GraceActive,
                "grace_extended" => MigrationStatus::GraceExtended,
                "cutover_completed" => MigrationStatus::CutoverCompleted,
                _ => anyhow::bail!("unknown_status"),
            },
        };
        self.inner.write().await.insert(tenant_id, (state.clone(), Instant::now()));
        Ok(state)
    }

    pub async fn invalidate(&self, tenant_id: Uuid) {
        self.inner.write().await.remove(&tenant_id);
    }
}
```

### 3.4 — Verifier hook

```rust
// services/auth/src/migration/verifier_hook.rs
use chrono::Utc;
use crate::jwt::Claims;
use crate::migration::state::{MigrationState, MigrationStatus, MigrationStateCache};

#[derive(Debug, thiserror::Error)]
pub enum StubTokenRejection {
    #[error("stub_token_grace_window_expired")]
    GraceExpired { cutover_at: chrono::DateTime<chrono::Utc> },
}

pub async fn check_stub_token(
    claims: &Claims,
    cache: &MigrationStateCache,
    db: &sqlx::PgPool,
    audit: &impl AuditEmitter,
) -> Result<(), StubTokenRejection> {
    // If token has rbac_v → not a stub; nothing to check.
    if claims.rbac_v.is_some() { return Ok(()); }

    let state = cache.get(claims.tid, db).await
        .expect("migration state must exist for every tenant");

    let now = Utc::now();
    if state.status == MigrationStatus::CutoverCompleted || state.cutover_at <= now {
        // Reject + emit audit
        audit.emit_stub_token_rejected(claims.tid, claims.sub, state.cutover_at).await;
        // If we just crossed the threshold, transition status
        if state.status != MigrationStatus::CutoverCompleted {
            tokio::spawn(transition_to_cutover_completed(claims.tid, db.clone(), cache.clone()));
        }
        return Err(StubTokenRejection::GraceExpired { cutover_at: state.cutover_at });
    }

    // Still in grace — accept; sample audit at 1% (100% in last 24h)
    let days_remaining = (state.cutover_at - now).num_days();
    let should_audit = if days_remaining <= 1 { true } else { rand::random::<f32>() < 0.01 };
    if should_audit {
        audit.emit_stub_token_accepted(claims.tid, claims.sub, days_remaining).await;
    }
    Ok(())
}

async fn transition_to_cutover_completed(
    tenant_id: Uuid, db: sqlx::PgPool, cache: MigrationStateCache,
) {
    let rows = sqlx::query(r#"
        UPDATE auth_migration_state SET status = 'cutover_completed'
        WHERE tenant_id = $1 AND status != 'cutover_completed'
    "#).bind(tenant_id).execute(&db).await;
    if let Ok(r) = rows {
        if r.rows_affected() == 1 {
            // First time crossing — emit cutover_completed audit row
            cache.invalidate(tenant_id).await;
            // ... emit auth.cutover_completed
        }
    }
}
```

### 3.5 — Refresh hook

```rust
// services/auth/src/migration/refresh_hook.rs
use crate::jwt::Claims;

pub async fn on_refresh(
    prior_claims: &Claims,
    new_claims: &mut Claims,
    rbac_v: u32,
    db: &sqlx::PgPool,
) -> anyhow::Result<()> {
    let prior_present = prior_claims.rbac_v.is_some();

    // Always inject current rbac_v on refresh (DEC-449)
    new_claims.rbac_v = Some(rbac_v);

    sqlx::query(r#"
        INSERT INTO auth_token_refresh_log (tenant_id, subject_id, prior_rbac_v_present, new_rbac_v)
        VALUES ($1, $2, $3, $4)
    "#).bind(new_claims.tid).bind(new_claims.sub).bind(prior_present).bind(rbac_v as i32)
       .execute(db).await?;

    Ok(())
}
```

### 3.6 — Extend grace handler

```rust
// services/auth/src/handlers/migration.rs
use axum::{Json, extract::State, http::StatusCode};
use cyberos_auth::rbac::Role;

#[derive(Deserialize)]
pub struct ExtendGraceRequest {
    pub additional_days: i32,
    pub reason: String,
}

pub async fn extend_grace(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<ExtendGraceRequest>,
) -> Result<StatusCode, ApiError> {
    // (1) Role check
    if !claims.roles().contains(&Role::RootAdmin) {
        return Err(ApiError::PermissionDenied);
    }
    // (2) Validate
    if !(1..=60).contains(&req.additional_days) {
        return Err(ApiError::AdditionalDaysOutOfRange);
    }
    if req.reason.is_empty() || req.reason.len() > 500 {
        return Err(ApiError::ReasonInvalid);
    }
    // (3) Apply with provisioner role
    let mut tx = state.db.begin().await?;
    let updated = sqlx::query(r#"
        UPDATE auth_migration_state
        SET cutover_at = cutover_at + ($2 || ' days')::interval,
            status = 'grace_extended',
            grace_extended_at = now(),
            grace_extension_reason = $3,
            extension_count = extension_count + 1
        WHERE tenant_id = $1
          AND status = 'grace_active'
          AND extension_count = 0
        RETURNING cutover_at
    "#).bind(claims.tenant_id()).bind(req.additional_days as i64).bind(&req.reason)
       .execute(&mut *tx).await?;
    if updated.rows_affected() == 0 {
        return Err(ApiError::AlreadyExtended);
    }
    audit::emit_grace_window_extended(&mut tx, claims.tenant_id(), req.additional_days, &req.reason, claims.subject_id()).await?;
    tx.commit().await?;
    state.cache.invalidate(claims.tenant_id()).await;
    Ok(StatusCode::OK)
}
```

---

## §4 — Acceptance criteria

1. **Migration state seeded on FR-AUTH-101 ship** — every existing tenant gets cutover_at = now+30d.
2. **New tenants seeded on provisioning** — cutover_at = provision_time + 30d via FR-TEN-001 hook.
3. **Stub token accepted during grace** — verifier returns OK; `auth.stub_token_accepted` row sampled 1%.
4. **Stub token rejected post-grace** — 401 with body per DEC-448; `auth.stub_token_rejected` memory row.
5. **rbac_v injected on refresh** — refreshed token includes rbac_v; refresh log row with `prior_rbac_v_present=false`.
6. **Extension by root-admin** — 200; status → grace_extended; extension_count=1.
7. **Extension by non-root-admin** → 403.
8. **Second extension attempt** → 409 already_extended.
9. **additional_days < 1 or > 60** → 400 additional_days_out_of_range.
10. **Empty reason** → 400 reason_invalid.
11. **cutover_completed status forbids cutover_at change** → trigger raises cutover_immutable_post_completion.
12. **cutover_completed backward to grace_active forbidden** → trigger raises cutover_status_backward_forbidden.
13. **Cutover triggered on first post-grace verification** — status flips atomically; `auth.cutover_completed` row emitted once.
14. **Cutover NOT re-emitted on subsequent rejections** — trigger predicate prevents.
15. **Preview API** — returns counts + days_remaining + can_extend.
16. **Preview can_extend false after extension** — extension_count=1.
17. **OBS sev-2 at > 100/h rejections** — alarm rule fires.
18. **append-only migration_state from cyberos_app** — UPDATE blocked.
19. **append-only refresh_log from cyberos_app** — UPDATE/DELETE blocked.
20. **State cache 60s TTL** — invalidation on extension propagates within 60s.
21. **Verifier hook < 5ms p99** — perf test.
22. **OTel span `auth.migration.verify_stub` emitted** with outcome.
23. **Counter `auth_stub_token_rejected_total{tenant_id}` increments** on every rejection.
24. **PII-scrubbed reason in memory row**.
25. **Env-driven default grace days** — AUTH_MIGRATION_DEFAULT_GRACE_DAYS=14 → new tenants get 14d.
26. **Env value < 7 or > 90 rejected at startup** — service refuses.
27. **`GET /migration/refresh-events` returns log rows** — root-admin only.

---

## §5 — Verification

```rust
// services/auth/tests/stub_token_accepted_during_grace_test.rs
#[tokio::test]
async fn stub_token_accepted_with_5_days_remaining(ctx: TestCtx) {
    let tenant = ctx.create_tenant_with_cutover_in_days(5).await;
    let stub_token = ctx.issue_stub_token_for(tenant).await;  // no rbac_v
    let resp = ctx.get("/v1/some/protected-endpoint").bearer(&stub_token).await;
    assert_eq!(resp.status(), 200);
    let rows = ctx.memory_audit_rows("auth.stub_token_accepted").await;
    // 1% sampling — may or may not have the row; assert no rejection
    let rejections = ctx.memory_audit_rows("auth.stub_token_rejected").await;
    assert_eq!(rejections.len(), 0);
}

#[tokio::test]
async fn stub_token_accepted_100pct_in_last_24h(ctx: TestCtx) {
    let tenant = ctx.create_tenant_with_cutover_in_hours(12).await;  // < 24h remaining
    for _ in 0..10 {
        let stub_token = ctx.issue_stub_token_for(tenant).await;
        let _ = ctx.get("/v1/some/protected-endpoint").bearer(&stub_token).await;
    }
    let rows = ctx.memory_audit_rows("auth.stub_token_accepted").await;
    assert_eq!(rows.len(), 10);  // 100% sampling near end
}
```

```rust
// services/auth/tests/stub_token_rejected_post_grace_test.rs
#[tokio::test]
async fn post_grace_stub_token_rejected_with_clear_reason(ctx: TestCtx) {
    let tenant = ctx.create_tenant_with_past_cutover().await;
    let stub_token = ctx.issue_stub_token_for(tenant).await;
    let resp = ctx.get("/v1/some/protected-endpoint").bearer(&stub_token).await;
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "rbac_version_required");
    assert_eq!(body["reason"], "stub_token_grace_window_expired");
    assert!(body["grace_expired_at"].is_string());
    assert_eq!(body["action_required"], "refresh");
}

#[tokio::test]
async fn first_post_grace_request_triggers_cutover(ctx: TestCtx) {
    let tenant = ctx.create_tenant_with_past_cutover_no_completion().await;
    let stub_token = ctx.issue_stub_token_for(tenant).await;
    let _ = ctx.get("/protected").bearer(&stub_token).await;
    tokio::time::sleep(Duration::from_millis(100)).await;  // async transition
    let state = ctx.fetch_migration_state(tenant).await;
    assert_eq!(state.status, "cutover_completed");
    let rows = ctx.memory_audit_rows("auth.cutover_completed").await;
    assert_eq!(rows.len(), 1);
}
```

```rust
// services/auth/tests/extend_grace_test.rs
#[tokio::test]
async fn second_extension_rejected(ctx: TestCtx) {
    let tenant = ctx.tenant_in_grace().await;
    ctx.post_as_root_admin("/v1/auth/migration/extend-grace",
        json!({"additional_days": 30, "reason": "first extension"})).await.unwrap();
    let err = ctx.post_as_root_admin("/v1/auth/migration/extend-grace",
        json!({"additional_days": 14, "reason": "second extension"})).await.unwrap_err();
    assert!(format!("{err:?}").contains("already_extended"));
}

#[tokio::test]
async fn tenant_admin_cannot_extend(ctx: TestCtx) {
    let tenant = ctx.tenant_in_grace().await;
    let err = ctx.post_as_tenant_admin("/v1/auth/migration/extend-grace",
        json!({"additional_days": 30, "reason": "try"})).await.unwrap_err();
    assert!(format!("{err:?}").contains("permission_denied"));
}

#[tokio::test]
async fn additional_days_out_of_range(ctx: TestCtx) {
    for days in [0, 61, -5] {
        let err = ctx.post_as_root_admin("/v1/auth/migration/extend-grace",
            json!({"additional_days": days, "reason": "x"})).await.unwrap_err();
        assert!(format!("{err:?}").contains("additional_days_out_of_range"));
    }
}
```

```rust
// services/auth/tests/refresh_hook_injects_rbac_v_test.rs
#[tokio::test]
async fn refresh_of_stub_token_injects_rbac_v(ctx: TestCtx) {
    let stub_token = ctx.issue_stub_token().await;
    let refresh_token = ctx.issue_refresh_token().await;
    let resp = ctx.post("/v1/auth/refresh", json!({"refresh_token": refresh_token})).await;
    let new_access: String = resp.json().await.unwrap()["access_token"].as_str().unwrap().into();
    let claims = ctx.decode_claims(&new_access);
    assert!(claims.rbac_v.is_some());
    let log = ctx.fetch_refresh_log_for_subject(ctx.test_subject()).await;
    assert!(log.iter().any(|r| !r.prior_rbac_v_present));
}
```

```rust
// services/auth/tests/cutover_immutable_test.rs
#[sqlx::test]
async fn cutover_at_immutable_post_completion(pool: sqlx::PgPool) {
    let tid = seed_migration_state(&pool, "cutover_completed", chrono::Utc::now() - chrono::Duration::days(5)).await;
    set_role_provisioner(&pool).await;
    let err = sqlx::query("UPDATE auth_migration_state SET cutover_at = now() + interval '30 days' WHERE tenant_id = $1")
        .bind(tid).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("cutover_immutable_post_completion"));
}

#[sqlx::test]
async fn cutover_backward_forbidden(pool: sqlx::PgPool) {
    let tid = seed_migration_state(&pool, "cutover_completed", chrono::Utc::now() - chrono::Duration::days(5)).await;
    set_role_provisioner(&pool).await;
    let err = sqlx::query("UPDATE auth_migration_state SET status = 'grace_active' WHERE tenant_id = $1")
        .bind(tid).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("cutover_status_backward_forbidden"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 4 memory row builders follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-101** — RBAC catalogue; consumes the `rbac_v` claim contract from §1 #18 of that FR.

**Downstream:** none — this FR is the implementation of FR-AUTH-101's grace-window promise.

**Cross-module:**
- **FR-AUTH-004** — JWT verifier + refresh path (this FR hooks both).
- **FR-AI-003** — memory audit bridge.
- **FR-MEMORY-111** — PII scrub of grace_extension_reason.
- **FR-OBS-007** — sev-2 alarm on > 100/h rejections.
- **FR-TEN-001** — provisioning hook that seeds per-tenant cutover_at.

---

## §8 — Example payloads

### 8.1 — 401 rejection response post-grace

```json
{
  "error": "rbac_version_required",
  "reason": "stub_token_grace_window_expired",
  "grace_expired_at": "2026-06-15T00:00:00Z",
  "action_required": "refresh"
}
```

### 8.2 — POST /v1/auth/migration/extend-grace

```json
{
  "additional_days": 30,
  "reason": "Bulk client refresh stalled due to mobile-app store review delay"
}
```

### 8.3 — GET /v1/auth/migration/preview response

```json
{
  "tenant_id": "5e8f1d2a-...",
  "cutover_at": "2026-06-15T00:00:00Z",
  "status": "grace_active",
  "days_remaining": 14,
  "stub_tokens_accepted_count_24h": 1240,
  "stub_tokens_rejected_count_24h": 0,
  "refreshed_count_24h": 380,
  "extension_count": 0,
  "can_extend": true
}
```

### 8.4 — auth.stub_token_rejected memory row

```json
{
  "kind": "auth.stub_token_rejected",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "cutover_at": "2026-06-15T00:00:00Z",
  "now": "2026-06-16T10:30:00Z",
  "ts_ns": 1747920731000000000
}
```

### 8.5 — auth.grace_window_extended memory row

```json
{
  "kind": "auth.grace_window_extended",
  "tenant_id": "5e8f1d2a-...",
  "prior_cutover_at": "2026-06-15T00:00:00Z",
  "new_cutover_at": "2026-07-15T00:00:00Z",
  "additional_days": 30,
  "reason_scrubbed": "Bulk client refresh stalled due to [REDACTED-CONTEXT]",
  "extended_by_subject_id_hash16": "8a7c8c8012344567",
  "ts_ns": 1747920731000000000
}
```

### 8.6 — auth.cutover_completed memory row

```json
{
  "kind": "auth.cutover_completed",
  "tenant_id": "5e8f1d2a-...",
  "final_cutover_at": "2026-06-15T00:00:00Z",
  "total_stub_accepted": 28400,
  "total_stub_rejected": 0,
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Token revocation API** — FR-AUTH-2xx; this FR's rejection doesn't revoke (token is rejected at verify but the token itself isn't on a CRL).
- **Per-subject grace override** — slice 4; useful for specific bot/service accounts.
- **Automated extension recommendation** — slice 4; preview API gives data; operator decides.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Stub token post-grace | verifier hook | 401 + audit | Refresh |
| Stub token during grace | verifier hook | accept + sampled audit | Designed |
| Migration state missing for tenant | DB lookup fail | 500 + sev-1 | Seed via one-off SQL |
| Cache stale during extension | 60s TTL + invalidate | Brief inconsistency | Designed |
| Trigger blocks legitimate UPDATE | test catches | CI fails | Fix predicate |
| Concurrent cutover transition | UPDATE WHERE status != 'completed' | One wins; idempotent | Designed |
| Extension > 60 days | handler validate | 400 | Designed |
| Extension < 1 day | handler validate | 400 | Designed |
| Second extension | UPDATE WHERE extension_count = 0 | 0 rows; 409 | Designed |
| Reason > 500 chars | handler validate + DB CHECK | 400 | Shorten |
| Non-root-admin extension | role check | 403 | Designed |
| Cutover_at backward UPDATE | trigger | rejected | Designed |
| Cutover_completed → grace_active | trigger | rejected | Designed |
| Refresh log UPDATE/DELETE from app | SQL grant | permission denied | Designed |
| Migration state UPDATE from app | SQL grant | permission denied | Designed |
| Env grace days < 7 or > 90 | startup check | service refuses | Fix env |
| memory audit emit fails | tx rollback | 500; verifier retries | memory_writer health |
| Preview can_extend wrong | unit test | CI fails | Fix logic |
| Refresh hook fails to inject rbac_v | unit test | CI fails | Fix hook |
| Cache invalidation race | next refresh observes | Brief stale | Designed |
| Cutover audit emitted twice | predicate check | UPDATE 0 rows on second | Designed |
| Concurrent extension attempts | first wins; second 409 | Designed | None |
| Subject deleted while in refresh log | FK RESTRICT | DELETE auth.subjects fails | Soft-delete subject |
| RLS bypass | USING | 0 rows | Designed |
| OTel span attribute missing | otel_test | CI fails | Fix |
| Counter `auth_stub_token_rejected_total` not incrementing | metric test | CI fails | Fix instrumentation |
| Sev-2 alarm threshold drift | OBS rule version | None | Coordinate with FR-OBS-007 |
| Verifier hook latency spike | perf test | CI fails | Optimise cache path |
| Migration state insert fails on new tenant provisioning | hook fail | Tenant created but cutover_at NULL | Manual SQL seed + audit |
| `additional_days` integer overflow | i32 + range check | None | Designed |
| Refresh log table size growth | partition (deferred) | None at slice 1 | FR-OBS-006 retention rule |

---

## §11 — Implementation notes

- **30-day default grace** — covers 4 refresh cycles + safety margin.
- **Per-tenant cutover_at** — fairness for newly-provisioned tenants.
- **One extension max per tenant** — escape valve, not recurring policy.
- **Root-admin only for extension** — cross-tenant operation.
- **Clear 401 rejection body** — client knows to refresh.
- **Refresh hook on FR-AUTH-004 path** — transparent client-side cutover.
- **1% sampling during grace + 100% in last 24h** — volume-vs-visibility.
- **Lazy cutover transition on first post-grace request** — no cron needed.
- **60s cache TTL** — hot path performance; ops events tolerate lag.
- **Preview API** — operator decision support.
- **Append-only migration_state via auth_provisioner role split** — app code can't mutate.
- **Sev-2 at > 100/h rejected** — operator investigation prompt.
- **Idempotent cutover audit via UPDATE WHERE status !=** — single emission per tenant.
- **Cutover backward-flow forbidden** — semantic correctness.
- **Refresh log per-row analytics** — "how many stub → full conversions?".
- **`additional_days` capped 1-60** — bounded extension.
- **`extension_count` CHECK 0-1** — DB-enforced max 1.
- **PII scrub reason** — operator may type sensitive context.
- **Env-driven default with min 7 / max 90** — operator tunability with bounds.
- **Verifier hook < 5ms p99** — hot path budget.
- **`auth_provisioner` role split** — defense in depth.

---

*End of FR-AUTH-109.*
