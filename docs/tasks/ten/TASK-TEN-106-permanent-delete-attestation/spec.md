---
id: TASK-TEN-106
title: "TEN permanent-delete attestation — CSO + CLO dual-sign + chain-anchored evidence + cascade hard-purge across all tenant data with verification"
module: TEN
priority: MUST
status: draft
verify: T
phase: P4
milestone: P4 · slice 2
slice: 2
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-104, TASK-TEN-105, TASK-AUTH-101, TASK-AUTH-002, TASK-MEMORY-101, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007, TASK-OBS-009]
depends_on: [TASK-TEN-104, TASK-TEN-105]
blocks: []

source_pages:
  - website/docs/modules/ten.html#permanent-delete
  - https://gdpr.eu/article-17-right-to-be-forgotten/

source_decisions:
  - DEC-1340 2026-05-17 — Permanent-delete attestation = the final, irreversible step in tenant lifecycle; transitions tenant from `terminating` (TASK-TEN-104) to fully hard-purged; dual-sign (CSO + CLO) required because the action is unrecoverable
  - DEC-1341 2026-05-17 — Pre-conditions: tenant.status='terminating'; recent bundle export within 90d (TASK-TEN-105 §1 #14); 30-day cool-off since tenant.status flipped to terminating
  - DEC-1342 2026-05-17 — CSO + CLO dual-signature (distinct subjects per CHECK constraint mirror of TASK-PORTAL-008 DEC denial pattern)
  - DEC-1343 2026-05-17 — Cascade hard-purge across: tenant Postgres schema (DROP CASCADE), tenant S3 prefix (recursive delete + lifecycle policy), KMS keys (schedule deletion 30d), NATS subject namespace (purge JetStream), audit chain (TOMBSTONE entries — chain integrity preserved)
  - DEC-1344 2026-05-17 — Chain integrity: audit rows for the deleted tenant remain in memory chain (cannot be deleted — chain integrity property); replaced with tombstone records containing only `(tenant_id, deleted_at, attestation_id)` — original content scrubbed
  - DEC-1345 2026-05-17 — Closed enum `attestation_status` = {pending_cso_sign, pending_clo_sign, ready_to_execute, executing, completed, cancelled}; CI cardinality asserts 6
  - "DEC-1346 2026-05-17 — Verification: post-purge, attestation row remains forever (chain-anchored); verification endpoint shows: timestamps, signatures, bundle reference, executed cascade summary"
  - DEC-1347 2026-05-17 — Cancel allowed before status='executing'; after that, irreversible
  - DEC-1348 2026-05-17 — Closed enum `cascade_target` = {postgres_schema, s3_prefix, kms_keys, nats_subjects, audit_chain_tombstone}; cardinality 5
  - DEC-1349 2026-05-17 — Per-target execution log persisted in `permanent_delete_cascade_log` with status (pending|executing|completed|failed); CSO can re-trigger failed targets
  - DEC-1350 2026-05-17 — memory audit kinds: ten.permanent_delete_attestation_initiated, ten.permanent_delete_cso_signed, ten.permanent_delete_clo_signed, ten.permanent_delete_executing, ten.permanent_delete_completed, ten.permanent_delete_cancelled, ten.permanent_delete_cascade_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0028_permanent_delete_attestations.sql
    - services/ten/migrations/0029_permanent_delete_cascade_log.sql
    - services/ten/src/permanent_delete/mod.rs
    - services/ten/src/permanent_delete/initiate.rs
    - services/ten/src/permanent_delete/sign.rs
    - services/ten/src/permanent_delete/execute.rs
    - services/ten/src/permanent_delete/cascade.rs
    - services/ten/src/permanent_delete/audit_tombstone.rs
    - services/ten/src/permanent_delete/verify.rs
    - services/ten/src/audit/permanent_delete_events.rs
    - services/ten/src/handlers/permanent_delete_routes.rs
    - services/ten/tests/perm_delete_dual_sign_test.rs
    - services/ten/tests/perm_delete_30d_cool_off_test.rs
    - services/ten/tests/perm_delete_bundle_required_test.rs
    - services/ten/tests/perm_delete_cascade_test.rs
    - services/ten/tests/perm_delete_audit_tombstone_test.rs
    - services/ten/tests/perm_delete_chain_integrity_test.rs
    - services/ten/tests/perm_delete_cancel_test.rs
    - services/ten/tests/perm_delete_cascade_failure_test.rs
    - services/ten/tests/perm_delete_status_enum_test.rs
    - services/ten/tests/perm_delete_audit_emission_test.rs

  modified_files:
    - services/ten/src/lib.rs

  allowed_tools:
    - file_read: services/{ten,auth,memory}/**
    - file_write: services/ten/{src,tests,migrations}/**
    - bash: cd services/ten && cargo test permanent_delete

  disallowed_tools:
    - skip dual-sign (per DEC-1342)
    - skip 30d cool-off (per DEC-1341)
    - skip bundle precondition (per TASK-TEN-105 DEC-1330)
    - delete audit chain rows (per DEC-1344 — tombstone only)

effort_hours: 5
subtasks:
  - "0.4h: 0028 + 0029 migrations"
  - "0.4h: permanent_delete/mod.rs + closed enums"
  - "0.4h: initiate.rs (preconditions check)"
  - "0.5h: sign.rs (dual-sign workflow)"
  - "0.6h: execute.rs (cascade orchestrator)"
  - "0.5h: cascade.rs (per-target logic)"
  - "0.4h: audit_tombstone.rs (chain integrity)"
  - "0.3h: verify.rs"
  - "0.3h: audit/permanent_delete_events.rs (7 builders)"
  - "0.3h: handlers/permanent_delete_routes.rs"
  - "1.0h: tests — 10 test files"

risk_if_skipped: "Without permanent-delete attestation, GDPR Art. 17 erasure (PORTAL-008 deletion_request) cannot be completed irreversibly — tenant data lingers indefinitely → ongoing breach risk + storage cost + legal liability. Without DEC-1342 dual-sign, a single compromised admin account = catastrophic data loss with no recovery (no other option but restore from bundle which may not exist). Without DEC-1344 chain integrity preservation, audit history breaks → unprovable past actions. Without DEC-1341 30d cool-off, accidental clicks could destroy tenant data — recoverable only from bundle (which may be in 90d window but could fail to restore). The 5h effort lands the legally-bound permanent-deletion primitive that completes the offboarding lifecycle."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship permanent-delete attestation at `services/ten/src/permanent_delete/` requiring CSO + CLO dual-signature + 30-day cool-off + recent bundle existence (TASK-TEN-105 §1 #14 gate), then cascade hard-purge across 5 targets with audit-chain tombstone preservation, and 7 memory audit kinds.

1. **MUST** define closed `attestation_status` enum: `('pending_cso_sign','pending_clo_sign','ready_to_execute','executing','completed','cancelled')` per DEC-1345. Cardinality 6.

2. **MUST** define closed `cascade_target` enum: `('postgres_schema','s3_prefix','kms_keys','nats_subjects','audit_chain_tombstone')` per DEC-1348. Cardinality 5.

3. **MUST** define `permanent_delete_attestations` at migration `0028`: `(attestation_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, status attestation_status NOT NULL DEFAULT 'pending_cso_sign', initiated_at TIMESTAMPTZ NOT NULL DEFAULT now(), initiated_by_subject_id UUID NOT NULL, cso_subject_id UUID, cso_signed_at TIMESTAMPTZ, clo_subject_id UUID, clo_signed_at TIMESTAMPTZ, bundle_ref UUID NOT NULL, executed_at TIMESTAMPTZ, completed_at TIMESTAMPTZ, cancellation_reason TEXT, trace_id CHAR(32), CHECK (cso_subject_id IS NULL OR clo_subject_id IS NULL OR cso_subject_id != clo_subject_id))`. Append-only.

4. **MUST** define `permanent_delete_cascade_log` at migration `0029`: `(id BIGSERIAL PRIMARY KEY, attestation_id UUID NOT NULL REFERENCES permanent_delete_attestations(attestation_id), target cascade_target NOT NULL, status TEXT NOT NULL CHECK (status IN ('pending','executing','completed','failed')), execution_started_at TIMESTAMPTZ, execution_completed_at TIMESTAMPTZ, failure_reason TEXT, item_count INT, UNIQUE(attestation_id, target))`. One row per (attestation, target).

5. **MUST** expose `POST /v1/admin/tenants/{tid}/permanent-delete/initiate` body `{ reason }`. Caller has `cso` OR `tenant_admin` role. Handler validates preconditions:
   - `tenants.status = 'terminating'` (set by TASK-TEN-104) — else 412 + `tenant_not_terminating`.
   - `now() - tenant.terminating_at >= 30 days` — else 412 + `cool_off_not_elapsed`.
   - Recent bundle within 90d per TASK-TEN-105 §1 #14 — else 412 + `bundle_export_required_before_attestation` (referenced from TASK-TEN-105 cross-FR contract).
   - INSERTs attestation row with status='pending_cso_sign'.
   - Emits `ten.permanent_delete_attestation_initiated`.

6. **MUST** expose `POST /v1/admin/permanent-delete/{attestation_id}/sign-cso` for CSO signature. Caller has `cso` role. Handler:
   - Validates status='pending_cso_sign'.
   - Sets `cso_subject_id`, `cso_signed_at`; transitions to 'pending_clo_sign'.
   - Emits `ten.permanent_delete_cso_signed` sev-1.

7. **MUST** expose `POST /v1/admin/permanent-delete/{attestation_id}/sign-clo` for CLO signature. Caller has `clo` role. Handler:
   - Validates status='pending_clo_sign'.
   - CHECK clo_subject_id ≠ cso_subject_id (CHECK constraint enforces).
   - Sets `clo_subject_id`, `clo_signed_at`; transitions to 'ready_to_execute'.
   - Emits `ten.permanent_delete_clo_signed` sev-1.

8. **MUST** expose `POST /v1/admin/permanent-delete/{attestation_id}/execute` for actual deletion. Caller has `cso` OR `clo` role. Handler:
   - Validates status='ready_to_execute'.
   - Transitions to 'executing'; emits `ten.permanent_delete_executing` sev-1.
   - For each `cascade_target` in fixed order: invoke per-target handler per §1 #10.
   - On all-success: status='completed' + emit `ten.permanent_delete_completed` sev-1.
   - On any-failure: status remains 'executing' (operator re-triggers failed target).

9. **MUST** support cancellation per DEC-1347 at `POST /v1/admin/permanent-delete/{attestation_id}/cancel` body `{ reason }` BEFORE status='executing'. Handler:
   - Validates status NOT IN ('executing','completed').
   - Sets status='cancelled' + `cancellation_reason`.
   - Emits `ten.permanent_delete_cancelled` sev-1.

10. **MUST** cascade purge in fixed order per DEC-1343 + DEC-1348:
    1. **`postgres_schema`** — `DROP SCHEMA tenant_<slug> CASCADE` after row counts logged to `cascade_log.item_count`.
    2. **`s3_prefix`** — recursive delete of `s3://cyberos-{residency}-tenants/{tenant_id}/*` AND `cyberos-{residency}-audit/{tenant_id}/*`.
    3. **`kms_keys`** — `kms schedule-key-deletion` for tenant's keys (signing key from TASK-TEN-105, encryption keys); 30d grace before AWS final deletion.
    4. **`nats_subjects`** — purge JetStream subject `tenant.<slug>.>` permanently.
    5. **`audit_chain_tombstone`** — UPDATE audit_rows SET payload='{"tombstoned":true}' WHERE tenant_id=$1; chain hashes preserved per DEC-1344.

11. **MUST** preserve chain integrity per DEC-1344. Audit rows are NOT deleted; their `payload` JSONB is replaced with `{"tombstoned": true, "deleted_at": "...", "attestation_id": "..."}`. Chain hashes intact; verifier can replay chain and see tombstones in original positions.

12. **MUST** log each cascade target per DEC-1349. INSERT `cascade_log` row at target-start; UPDATE on completion/failure. On failure: status='failed' + reason; CSO can `POST /v1/admin/permanent-delete/{attestation_id}/retry-cascade/{target}`.

13. **MUST** expose verification endpoint `GET /v1/admin/permanent-delete/{attestation_id}/verify` per DEC-1346 — returns signed timestamps, signatures, bundle reference, executed cascade summary. Accessible by `cso`, `clo`, OR external auditors via signed-URL.

14. **MUST** emit 7 memory audit kinds per DEC-1350. ALL sev-1 (regulatory-critical) — initiation, CSO sign, CLO sign, executing, completed, cancelled, cascade_failed.

15. **MUST** PII-scrub `reason` + `cancellation_reason` via TASK-MEMORY-111 — hashed in chain, raw in DB.

16. **MUST** thread trace_id across initiate → sign → execute → cascade audit.

17. **MUST** be RLS-scoped — `cso/clo` see all attestations; `tenant_admin` sees own tenant only.

18. **MUST NOT** allow same subject as both CSO + CLO per DEC-1342 (CHECK constraint enforces).

19. **MUST NOT** delete audit chain rows per DEC-1344 (tombstone only).

20. **MUST NOT** allow re-attempt of cancelled attestation — new attestation required (preserves history).

---

## §2 — Why this design (rationale)

**Why dual-signature CSO+CLO (§1 #6-7, DEC-1342)?** Permanent deletion is unrecoverable. Single-signature = single compromised admin destroys tenant. Two distinct C-level roles = compromise of one alone insufficient.

**Why 30-day cool-off (§1 #5, DEC-1341)?** Buyer's remorse + accidental termination. 30 days gives the tenant time to reverse termination (TASK-TEN-104 reverse flow) before destruction.

**Why bundle precondition (§1 #5, DEC-1341 + TASK-TEN-105 DEC-1330)?** Recovery path. Bundle = last copy of tenant data. Without recent bundle, no way to restore if deletion was wrongful.

**Why tombstone vs delete audit rows (§1 #11, DEC-1344)?** Chain integrity. Deleting rows breaks the Merkle chain. Tombstone preserves chain; payload-scrubbed satisfies GDPR while keeping audit verifiability.

**Why fixed cascade order (§1 #10)?** Idempotency + failure recovery — postgres first (the source of truth) so even if S3 cascade fails, no tenant data accessible; KMS last so encrypted bundles remain decryptable for restore if cancellation needed mid-cascade.

**Why operator can retry failed targets (§1 #12, DEC-1349)?** Cascade failures are transient (KMS rate limits, NATS pause). Re-trigger lets operator complete the work without restarting the whole attestation flow.

---

## §3 — API contract

```sql
-- 0028_permanent_delete_attestations.sql
CREATE TYPE attestation_status AS ENUM ('pending_cso_sign','pending_clo_sign','ready_to_execute','executing','completed','cancelled');
CREATE TYPE cascade_target AS ENUM ('postgres_schema','s3_prefix','kms_keys','nats_subjects','audit_chain_tombstone');

CREATE TABLE permanent_delete_attestations (
  attestation_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  status attestation_status NOT NULL DEFAULT 'pending_cso_sign',
  initiated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  initiated_by_subject_id UUID NOT NULL,
  reason TEXT NOT NULL,
  cso_subject_id UUID,
  cso_signed_at TIMESTAMPTZ,
  clo_subject_id UUID,
  clo_signed_at TIMESTAMPTZ,
  bundle_ref UUID NOT NULL,
  executed_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  cancellation_reason TEXT,
  trace_id CHAR(32),
  CHECK (cso_subject_id IS NULL OR clo_subject_id IS NULL OR cso_subject_id != clo_subject_id)
);
ALTER TABLE permanent_delete_attestations ENABLE ROW LEVEL SECURITY;
CREATE POLICY pda_rls ON permanent_delete_attestations
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON permanent_delete_attestations FROM cyberos_app;
GRANT UPDATE (status, cso_subject_id, cso_signed_at, clo_subject_id, clo_signed_at,
              executed_at, completed_at, cancellation_reason) ON permanent_delete_attestations TO cyberos_app;

-- 0029_permanent_delete_cascade_log.sql
CREATE TABLE permanent_delete_cascade_log (
  id BIGSERIAL PRIMARY KEY,
  attestation_id UUID NOT NULL REFERENCES permanent_delete_attestations(attestation_id),
  target cascade_target NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending','executing','completed','failed')),
  execution_started_at TIMESTAMPTZ,
  execution_completed_at TIMESTAMPTZ,
  failure_reason TEXT,
  item_count INT,
  UNIQUE(attestation_id, target)
);
REVOKE DELETE ON permanent_delete_cascade_log FROM cyberos_app;
GRANT UPDATE (status, execution_started_at, execution_completed_at, failure_reason, item_count) ON permanent_delete_cascade_log TO cyberos_app;
```

Endpoints:
```text
POST   /v1/admin/tenants/{tid}/permanent-delete/initiate            (cso or tenant_admin)
POST   /v1/admin/permanent-delete/{id}/sign-cso                     (cso)
POST   /v1/admin/permanent-delete/{id}/sign-clo                     (clo)
POST   /v1/admin/permanent-delete/{id}/execute                      (cso or clo)
POST   /v1/admin/permanent-delete/{id}/cancel                       (cso or clo or tenant_admin)
POST   /v1/admin/permanent-delete/{id}/retry-cascade/{target}       (cso)
GET    /v1/admin/permanent-delete/{id}/verify                       (public via signed-URL)
```

---

## §4 — Acceptance criteria

1. **attestation_status cardinality 6**.
2. **cascade_target cardinality 5**.
3. **30d cool-off enforced** — tenant.terminating_at = now() - 29d → initiate returns 412.
4. **Bundle required** — no recent bundle → initiate returns 412 + bundle_export_required.
5. **CSO sign** — pending → pending_clo_sign after CSO signs.
6. **CLO sign** — pending_clo_sign → ready_to_execute after CLO signs.
7. **Same person dual-sign rejected** — CSO + CLO same subject_id → CHECK fails.
8. **Execute cascades all 5 targets** — cascade_log has 5 rows all status='completed'.
9. **Postgres schema dropped** — schema `tenant_<slug>` no longer exists.
10. **S3 prefix deleted** — list returns 0 objects.
11. **KMS keys scheduled** — kms describe-key shows PendingDeletion.
12. **NATS subjects purged** — JetStream subject empty.
13. **Audit chain tombstoned** — audit rows for tenant have payload `{"tombstoned":true}`.
14. **Chain integrity preserved** — Merkle chain verifies despite tombstones.
15. **Cancellation before execute** — status='ready_to_execute' → cancel succeeds.
16. **Cancellation after execute rejected** — status='executing' → cancel 409.
17. **Cascade target retry** — failed target can be retried.
18. **7 memory audit kinds emitted** — full lifecycle.
19. **Trace_id end-to-end**.
20. **Verification endpoint** — returns signatures + cascade summary.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dual_sign_required() {
    let ctx = TestContext::with_terminating_tenant_with_bundle().await;
    let att = ctx.initiate_perm_delete().await;

    let r = ctx.execute_perm_delete(att).await;
    assert_eq!(r.status(), 409);  // not yet signed

    ctx.as_cso().sign_cso(att).await;
    let r = ctx.execute_perm_delete(att).await;
    assert_eq!(r.status(), 409);  // CLO missing

    ctx.as_clo().sign_clo(att).await;
    let r = ctx.execute_perm_delete(att).await;
    assert_eq!(r.status(), 200);
}

#[tokio::test]
async fn same_person_dual_sign_blocked() {
    let ctx = TestContext::with_terminating_tenant_with_bundle().await;
    let att = ctx.initiate_perm_delete().await;
    ctx.as_cso_and_clo_same_person().sign_cso(att).await;
    let r = ctx.as_cso_and_clo_same_person().sign_clo(att).await;
    assert_eq!(r.status(), 400);  // CHECK constraint violation surfaces
}

#[tokio::test]
async fn cool_off_enforced() {
    let ctx = TestContext::new().await;
    let tid = ctx.start_termination().await;
    ctx.travel_clock_forward(Duration::from_days(29)).await;
    let r = ctx.initiate_perm_delete_raw(tid).await;
    assert_eq!(r.status(), 412);
}

#[tokio::test]
async fn cascade_all_5_targets() {
    let ctx = TestContext::with_terminating_tenant_with_bundle().await;
    let att = ctx.initiate_and_dual_sign().await;
    ctx.execute_perm_delete(att).await;

    let cascade_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT target::text, status FROM permanent_delete_cascade_log WHERE attestation_id=$1"
    ).bind(att).fetch_all(&ctx.pool).await.unwrap();
    assert_eq!(cascade_rows.len(), 5);
    assert!(cascade_rows.iter().all(|(_, s)| s == "completed"));
}

#[tokio::test]
async fn audit_chain_tombstoned_but_integrity_preserved() {
    let ctx = TestContext::with_terminating_tenant_with_bundle_and_audit_history().await;
    let att = ctx.initiate_and_dual_sign().await;
    ctx.execute_perm_delete(att).await;

    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT payload FROM memory_audit_rows WHERE tenant_id=$1 ORDER BY id"
    ).bind(ctx.tenant_id).fetch_all(&ctx.pool).await.unwrap();
    assert!(rows.iter().all(|r| r["tombstoned"] == true));

    let chain_ok = ctx.verify_memory_chain_integrity(ctx.tenant_id).await;
    assert!(chain_ok);
}

#[tokio::test]
async fn cancel_before_execute_succeeds() {
    let ctx = TestContext::with_terminating_tenant_with_bundle().await;
    let att = ctx.initiate_perm_delete().await;
    ctx.as_cso().sign_cso(att).await;
    let r = ctx.cancel_perm_delete(att, "operator error").await;
    assert_eq!(r.status(), 200);
    let status: String = sqlx::query_scalar("SELECT status::text FROM permanent_delete_attestations WHERE attestation_id=$1")
        .bind(att).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(status, "cancelled");
}

// 5.7 cancel during execute rejected
// 5.8 cascade retry on fail
// 5.9 status enum cardinality
// 5.10 audit emission
```

---

## §7 — Dependencies

**Upstream:** TASK-TEN-104 (terminating status), TASK-TEN-105 (bundle precondition).
**Cross-module:** TASK-AUTH-101 (cso + clo + tenant_admin roles), TASK-AUTH-002 (subject hard-purge), TASK-MEMORY-101 (chain integrity), TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007, TASK-OBS-009 (chain-of-custody anchor).
**Downstream:** None.

---

## §8 — Example payloads

`ten.permanent_delete_completed`:
```json
{
  "kind": "ten.permanent_delete_completed",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "user.cso.789",
  "trace_id": "...",
  "occurred_at": "2026-06-17T...",
  "payload": {
    "attestation_id": "0190...",
    "bundle_ref": "0190...",
    "cso_subject_id_hash16": "f8a1...",
    "clo_subject_id_hash16": "9c4e...",
    "cascade_summary": {
      "postgres_schema": { "item_count": 1, "status": "completed" },
      "s3_prefix": { "item_count": 48273, "status": "completed" },
      "kms_keys": { "item_count": 3, "status": "completed" },
      "nats_subjects": { "item_count": 1, "status": "completed" },
      "audit_chain_tombstone": { "item_count": 184729, "status": "completed" }
    },
    "completed_at": "2026-06-17T09:42:18.221Z"
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Multi-tenant batch permanent-delete (slice 3).
- **Deferred:** Configurable cool-off per tenant (slice 3).
- **Deferred:** Pre-execute final summary email to former tenant contact (slice 3).
- **Deferred:** Per-region cascade parallelisation (slice 3).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| 30d cool-off not met | precondition | 412 | Wait |
| No bundle in 90d | precondition | 412 | Request bundle |
| Tenant not terminating | precondition | 412 | Initiate termination via TASK-TEN-104 |
| Same person dual-sign | CHECK | 400 + CHECK violation | Different signatories |
| Postgres DROP SCHEMA fails | sql error | Cascade marks postgres_schema as failed; other targets paused | Operator investigates |
| S3 recursive delete fails (eventual consistency) | API error | Marked failed; retry succeeds | Inherent |
| KMS schedule-deletion already pending | API error | Marked as completed (idempotent) | Inherent |
| NATS purge fails (subject already empty) | not error | Marked completed | Inherent |
| Audit chain tombstone UPDATE breaks chain | hash verify | Sev-1 alarm; cancel cascade; chain rollback | Critical — operator intervention |
| Cancellation race with executing | tx isolation | Last write wins; cancel rejected if status='executing' | Inherent |
| Retry cascade after partial completion | per-target idempotency | Each target retry-safe | Inherent |
| CSO signs but CLO doesn't within 30d | watchdog | Sev-2 alert; attestation expires after 30d (slice 3 enhancement) | Slice 2 = manual operator review |
| Bundle reference no longer valid (expired/deleted) | check at initiate | 412 + invalid_bundle | Request new bundle |
| Cross-tenant attestation attempt | RLS | 403 | Inherent |
| Verification endpoint accessed years later | signed URL re-signed | Always queryable | Persistent verification |

---

## §11 — Implementation notes

**§11.1** Cascade order matters: postgres first (data inaccessible immediately even if S3 incomplete); KMS last (encrypted bundles remain decryptable mid-cascade for emergency rollback).

**§11.2** Audit chain tombstone is a single UPDATE; chain hashes computed over original (pre-tombstone) content remain valid for chain replay.

**§11.3** S3 delete uses lifecycle policy + manual recursive — defense-in-depth.

**§11.4** KMS schedule-deletion has 7-30d AWS minimum window; our code requests 30d (max).

**§11.5** Per-target retry endpoint enables operator recovery without restarting full attestation.

**§11.6** Verification endpoint uses long-lived signed URL (years); operator distributes to legal counsel as needed.

**§11.7** Tombstone payload structure: `{"tombstoned": true, "deleted_at": "...", "attestation_id": "..."}` — fixed shape for query.

**§11.8** Cascade orchestrator runs synchronously (5 targets, ~1-5 minutes total); not async because operator visibility wanted at each step.

**§11.9** CHECK constraint on dual-sign prevents one of the most catastrophic failure modes (insider attack) at schema level.

**§11.10** Reason + cancellation_reason hashed via TASK-MEMORY-111 before memory row write; raw text retained in DB for forensic.

---

*End of TASK-TEN-106 spec.*
