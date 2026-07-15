---
id: TASK-AUTH-005
title: "Admin REST: list tenants + list subjects + revoke subject + unrevoke + cursor pagination + jti deny-list"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: AUTH
priority: p0
status: done
verify: T
phase: P0
milestone: P0 · slice 2
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: 2026-05-19
memory_chain_hash: null
related_tasks: [TASK-AUTH-001, TASK-AUTH-002, TASK-AUTH-003, TASK-AUTH-004, TASK-AUTH-006]
depends_on: [TASK-AUTH-001, TASK-AUTH-002, TASK-AUTH-003, TASK-AUTH-004]
blocks: [TASK-AUTH-101, TASK-AUTH-107]

source_pages:
  - website/docs/modules/auth.html#admin-rest
source_decisions:
  - DEC-124 (revoke = `suspended:true` + jti deny-list + auth.subject_revoked row; reversible via unrevoke)
  - DEC-125 (cursor pagination — opaque base64 of last_id; no offset-based paging)
  - DEC-126 (deny-list TTL = JWT max-exp = 1h; expires automatically when no JWTs could still be valid)

language: rust 1.81
service: cyberos/services/auth/
new_files:
  - services/auth/src/admin/list.rs
  - services/auth/src/admin/revoke.rs
  - services/auth/src/admin/cursor.rs
  - services/auth/src/jwt/deny_list.rs
  - services/auth/migrations/0007_sessions.sql
  - services/auth/tests/admin_list_test.rs
  - services/auth/tests/admin_revoke_test.rs
  - services/auth/tests/admin_cursor_pagination_test.rs
  - services/auth/tests/admin_deny_list_test.rs
modified_files:
  - services/auth/src/jwt/verify.rs                 # consult deny-list during verify
  - services/auth/src/jwt/issue.rs                  # insert into sessions table
  - services/auth/src/rls/registry.rs               # add `sessions` to TENANT_SCOPED_TABLES
allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test admin
disallowed_tools:
  - allow tenant-admin to list/revoke OUT-OF-tenant subjects (per §1 #2 + #3 — RLS blocks too)
  - allow offset-based pagination (per DEC-125 — cursor only; offset bleeds duplicates on concurrent insert)
  - skip memory audit row on revoke OR unrevoke (per §1 #5 — both are auditable mutations)
  - return password_hash in subject list responses (per §1 #2 — never expose hash)

effort_hours: 8
subtasks:
  - "0.5h: 0007_sessions.sql migration (jti, subject_id, tenant_id, expires_at, created_at)"
  - "1.0h: cursor.rs — opaque base64 cursor (encodes last seen id + signature)"
  - "1.0h: list.rs — list_tenants (root-admin) + list_subjects (tenant-admin scoped)"
  - "1.0h: revoke.rs — revoke (suspended=true + deny-list + audit) + unrevoke"
  - "1.0h: jwt/deny_list.rs — Redis-backed jti deny-list with TTL"
  - "0.5h: jwt/verify.rs integration — check deny-list during verify path"
  - "0.5h: jwt/issue.rs integration — insert sessions row on token issue"
  - "0.5h: Idempotency on revoke (Idempotency-Key header)"
  - "0.5h: canonical::subject_revoked + canonical::subject_unrevoked builders"
  - "1.5h: Tests — list-tenants + list-subjects + cross-tenant-403 + revoke + unrevoke + cursor stability + deny-list propagation < 30s + p95"
risk_if_skipped: "Operators have no UI/API to manage tenants + subjects post-creation. Suspending a compromised subject requires manual SQL — error-prone, no audit trail. Cursor-less pagination produces duplicate or skipped records on concurrent insert (operator listing subjects while another admin creates one — record either appears twice or not at all)."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** expose three admin REST endpoints for tenant + subject management. Each endpoint:

1. **MUST** `GET /v1/admin/tenants?cursor=<opaque>&limit=<int>` — root-admin only; paginated list of all tenants. Tenant-admin role gets `403`. Returns `{"items": [...], "next_cursor": <opaque or null>}`.
2. **MUST** `GET /v1/admin/subjects?tenant_id=<id>&cursor=<opaque>&limit=<int>` — tenant-admin (same tenant) OR root-admin (with `X-Switch-Tenant` set); paginated list of subjects in the specified tenant. Cross-tenant requests (tenant-admin of A querying tenant B) return `403`. RLS at the DB layer (TASK-AUTH-003) provides defence-in-depth — even if the API check is bypassed, the SELECT returns zero rows. Subject body NEVER includes `password_hash`.
3. **MUST** `POST /v1/admin/subjects/{id}/revoke` — tenant-admin (same tenant) or root-admin; sets `subjects.suspended = true` AND populates the jti deny-list with all currently-active jtis for that subject. Subsequent JWT verifications check the deny-list and reject revoked jtis with `401 token_revoked`.
4. **MUST** `POST /v1/admin/subjects/{id}/unrevoke` — tenant-admin (same tenant) or root-admin; sets `subjects.suspended = false`. Does NOT remove jtis from the deny-list (existing tokens stay revoked; new logins issue fresh jtis).
5. **MUST** include cursor-based pagination via opaque base64-encoded cursor. The cursor encodes `(table, last_id, hmac_signature)`. Limit defaults to 50; max 200. Offset-based paging is forbidden (cursor only) because concurrent inserts during paging would produce duplicates or skips.
6. **MUST** emit memory audit rows:
    - `auth.subject_revoked` per revoke — payload: `subject_id`, `tenant_id`, `revoked_by_subject_id`, `reason` (optional caller-supplied), `revoked_jti_count`, `request_id`.
    - `auth.subject_unrevoked` per unrevoke — payload: `subject_id`, `tenant_id`, `unrevoked_by_subject_id`, `request_id`.
7. **MUST** complete each endpoint in ≤ 100ms p95 (list ops bounded by limit; revoke bounded by deny-list inserts ~1ms each).
8. **MUST** support `Idempotency-Key` header on revoke + unrevoke (mirrors TASK-AUTH-001 §1 #5 pattern). Repeat revoke with same key + same subject_id → no-op return; same key + different subject_id → 409.
9. **MUST** validate cursor signature: a malformed or tampered cursor returns `400 BAD_REQUEST` with `{"error":"invalid_cursor"}`. The HMAC signature uses the same deployment secret as JWT signing (separate scope: cursor signing key derived via HKDF).
10. **MUST** maintain `sessions` table tracking active jtis: `(jti TEXT PK, subject_id UUID, tenant_id UUID, issued_at, expires_at, source_ip_hash16)`. Inserted on every token issue (TASK-AUTH-004); used by revoke to enumerate active jtis to deny-list.
11. **MUST** propagate revocation to all consuming services within 30 seconds. Mechanism: Redis pub/sub on channel `jwt_deny` with the jti as message; consuming services subscribe and add to their local deny-list cache. Cache eviction is automatic at jti's `exp`.
12. **MUST** require revoked subjects to re-authenticate via TASK-AUTH-004's `/v1/auth/token` with fresh credentials AFTER unrevoke. The deny-list does NOT clear on unrevoke (security default — explicit re-auth proves the operator's intent, not just the unrevoke action).
13. **MUST** include `sessions` table in `TENANT_SCOPED_TABLES` registry (TASK-AUTH-003 §1 #1) so RLS applies. Tenant-admin can only list sessions for their own tenant.
14. **SHOULD** support `?include_suspended=false` (default) and `?include_suspended=true` query param on `/v1/admin/subjects`. Default hides suspended subjects (the common case); explicit opt-in shows them (for revoke-management workflows).
15. **SHOULD** emit OTel metrics:
    - `auth_admin_list_total{endpoint, outcome, tenant_id}` (counter).
    - `auth_admin_revoke_total{outcome, tenant_id}` (counter).
    - `auth_admin_revoke_jti_count` (histogram; how many jtis revoked per call).
    - `auth_admin_deny_list_size{service}` (gauge from each consuming service).
    - `auth_admin_revoke_propagation_latency_ms` (histogram; revoke-call to deny-list-presence-on-consumer; SLO p99 < 30s).

---

## §2 — Why this design (rationale for humans)

**Why cursor pagination, not offset (§1 #5)?** Offset-based paging (`OFFSET 50`) on a table that's being inserted into produces duplicates AND skips. Example: list page 1 (50 items), then between page-1 fetch and page-2 fetch, 5 new tenants are created at the start of the table; page 2 (`OFFSET 50`) now starts AFTER 5 fresh rows that were never seen. Cursor-based paging encodes "the last id I saw"; the next page starts strictly after that id, regardless of inserts. Stripe + GitHub use this pattern for the same reason.

**Why HMAC-signed cursors (§1 #9)?** Without signing, an operator can craft a cursor pointing at any id — useful for fishing attacks ("paginate forward from id X to see what's there"). HMAC ensures the cursor came from a previous list response. Tampered cursors fail validation immediately.

**Why deny-list TTL = JWT max-exp = 1h (DEC-126)?** A jti only matters while a JWT carrying it could still be valid. Once the JWT expires, the jti is meaningless — checking the deny-list for an expired jti would reject already-rejected tokens. Setting the TTL = JWT max-exp (1h) ensures the deny-list naturally garbage-collects without explicit sweeping.

**Why does unrevoke NOT clear the deny-list (§1 #12)?** Unrevoke is "subject is allowed again." Clearing the deny-list would re-validate all the previously-issued JWTs that the operator deliberately revoked. The security default is "operator's revoke intent persists for 1h"; if the operator wants the subject to keep using their old session, they shouldn't have revoked. Explicit re-auth makes the new session deliberate.

**Why Redis pub/sub for revocation propagation (§1 #11)?** A central Postgres-polling approach would be slow (poll interval = propagation delay) AND scale poorly (every consuming service polls every N seconds). Pub/sub is push-based and near-instant. The 30s SLO is conservative; typical propagation is <100ms.

**Why `sessions` table (§1 #10)?** Without tracking issued jtis, revoke can't enumerate which jtis to deny-list. The sessions table is the registry; revoke joins on `subject_id` to find active jtis. Cleanup is automatic via the JWT exp (expired sessions are pruned by TASK-AUTH-006 sweeper).

**Why `password_hash` excluded from list responses (§1 #2)?** Listing subjects exposes the hash to anyone with admin access — a hash-cracking offline-attack surface. The list endpoint never includes the hash; the hash only exits the DB during JWT issuance (where it's needed for verify).

**Why `?include_suspended=false` default (§1 #14)?** Suspended subjects are operationally hidden — they shouldn't show up in routine "list my team" requests. The default-hide pattern keeps the common case clean; explicit opt-in surfaces them when needed (revoke-management UI).

**Why idempotency on revoke (§1 #8)?** Operator clicks "revoke" twice; without idempotency, the second click might emit a duplicate audit row OR fail with 409 (depending on implementation). Idempotency-Key makes the duplicate-click safe — second click returns the same result, no new audit row.

**Why cross-tenant defense at BOTH API and RLS layers (§1 #2)?** API check is the fast path (immediate 403 without DB query). RLS is the safety net (catches API-check bugs). Defense in depth — neither alone is sufficient.

---

## §3 — API contract

```rust
// services/auth/src/admin/list.rs
#[derive(Deserialize)]
pub struct ListQuery {
    pub cursor: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}
fn default_limit() -> u32 { 50 }

#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct TenantListItem {
    pub id: Uuid, pub slug: String, pub name: String,
    pub created_at: DateTime<Utc>, pub suspended: bool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SubjectListItem {
    pub id: Uuid, pub tenant_id: Uuid, pub email: String,
    pub roles: Vec<String>, pub suspended: bool, pub created_at: DateTime<Utc>,
    // password_hash deliberately excluded
}
```

```sql
-- services/auth/migrations/0007_sessions.sql
CREATE TABLE sessions (
    jti              TEXT PRIMARY KEY,                 -- ULID from TASK-AUTH-004
    subject_id       UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    tenant_id        UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    issued_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at       TIMESTAMPTZ NOT NULL,
    source_ip_hash16 CHAR(16) NOT NULL
);

CREATE INDEX sessions_subject_id_idx ON sessions(subject_id);
CREATE INDEX sessions_expires_at_idx ON sessions(expires_at);
```

```rust
// services/auth/src/admin/cursor.rs
pub fn encode(table: &str, last_id: Uuid, secret: &[u8]) -> String {
    let payload = format!("{table}:{last_id}");
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    base64::encode_config(format!("{payload}:{}", hex::encode(sig)), base64::URL_SAFE_NO_PAD)
}

pub fn decode(cursor: &str, table: &str, secret: &[u8]) -> Result<Uuid, AdminError> {
    let raw = base64::decode_config(cursor, base64::URL_SAFE_NO_PAD)
        .map_err(|_| AdminError::InvalidCursor)?;
    let s = std::str::from_utf8(&raw).map_err(|_| AdminError::InvalidCursor)?;
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() != 3 || parts[0] != table { return Err(AdminError::InvalidCursor); }
    let last_id: Uuid = parts[1].parse().map_err(|_| AdminError::InvalidCursor)?;
    // Verify HMAC
    let expected = format!("{}:{}", parts[0], parts[1]);
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(expected.as_bytes());
    let sig_bytes = hex::decode(parts[2]).map_err(|_| AdminError::InvalidCursor)?;
    mac.verify_slice(&sig_bytes).map_err(|_| AdminError::InvalidCursor)?;
    Ok(last_id)
}
```

```rust
// services/auth/src/admin/revoke.rs
pub async fn revoke_subject(
    subject_id: Uuid, claims: &Claims, idem_key: Option<String>,
    pool: &PgPool, redis: &RedisPool, request_id: &str,
) -> Result<(), AdminError> {
    let subject: Subject = rls::with_tenant(pool, claims.tenant_id, |tx| async move {
        sqlx::query_as("SELECT * FROM subjects WHERE id = $1").bind(subject_id)
            .fetch_optional(&mut **tx).await
            .map(|opt| opt.ok_or(AdminError::SubjectNotFound))?
    }).await??;

    if !can_revoke(claims, &subject) { return Err(AdminError::Forbidden); }

    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.tenant_id = $1::text").bind(claims.tenant_id.to_string()).execute(&mut *tx).await?;
    sqlx::query("UPDATE subjects SET suspended = TRUE WHERE id = $1").bind(subject_id).execute(&mut *tx).await?;

    let active_jtis: Vec<(String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT jti, expires_at FROM sessions WHERE subject_id = $1 AND expires_at > NOW()",
    ).bind(subject_id).fetch_all(&mut *tx).await?;

    // §1 #11: Redis pub/sub propagation
    for (jti, exp) in &active_jtis {
        let ttl = (*exp - Utc::now()).num_seconds().max(1) as u64;
        deny_list::add(redis, jti, ttl).await?;
        deny_list::publish(redis, jti).await?;
    }

    memory::emit_in_tx(&mut tx, memory::canonical::subject_revoked(
        subject_id, subject.tenant_id, claims.subject_id, active_jtis.len() as u32, request_id,
    )).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn unrevoke_subject(
    subject_id: Uuid, claims: &Claims, idem_key: Option<String>,
    pool: &PgPool, request_id: &str,
) -> Result<(), AdminError> {
    let subject: Subject = /* same RLS-scoped lookup */;
    if !can_revoke(claims, &subject) { return Err(AdminError::Forbidden); }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE subjects SET suspended = FALSE WHERE id = $1").bind(subject_id).execute(&mut *tx).await?;
    // §1 #12: deny-list NOT cleared
    memory::emit_in_tx(&mut tx, memory::canonical::subject_unrevoked(
        subject_id, subject.tenant_id, claims.subject_id, request_id,
    )).await?;
    tx.commit().await?;
    Ok(())
}
```

```rust
// services/auth/src/jwt/deny_list.rs
pub async fn add(redis: &RedisPool, jti: &str, ttl_seconds: u64) -> Result<(), redis::RedisError> {
    let mut conn = redis.get().await?;
    conn.set_ex(format!("jwt_deny:{jti}"), "revoked", ttl_seconds as usize).await
}

pub async fn contains(redis: &RedisPool, jti: &str) -> Result<bool, redis::RedisError> {
    let mut conn = redis.get().await?;
    Ok(conn.exists(format!("jwt_deny:{jti}")).await?)
}

pub async fn publish(redis: &RedisPool, jti: &str) -> Result<(), redis::RedisError> {
    let mut conn = redis.get().await?;
    conn.publish("jwt_deny", jti).await
}
```

---

## §4 — Acceptance criteria

1. Root-admin lists all tenants paginated; non-root-admin → 403.
2. Tenant-admin lists subjects in own tenant; cross-tenant attempt → 403.
3. Cross-tenant blocked at RLS too (no rows even if API check bypassed).
4. Revoke as tenant-admin: subject `suspended=true`; jtis added to Redis deny-list.
5. Revoke propagation: subject's existing JWTs verify-fail within 30s on every consuming service.
6. Audit row `auth.subject_revoked` emitted with `revoked_jti_count`.
7. Unrevoke restores `suspended=false`; deny-list NOT cleared (existing tokens still rejected).
8. Audit row `auth.subject_unrevoked` emitted on unrevoke.
9. After unrevoke, fresh `/v1/auth/token` issues new JWT with new jti.
10. Pagination cursor is HMAC-signed; tampering produces 400 invalid_cursor.
11. Cursor pagination is stable: insert during paging neither duplicates nor skips.
12. List response NEVER includes `password_hash`.
13. `?include_suspended=false` default hides suspended subjects.
14. `?include_suspended=true` shows suspended subjects.
15. Latency p95 < 100ms per endpoint.
16. Idempotent revoke (same key + same subject_id) = no-op return.
17. Idempotency key reuse with different subject_id → 409.
18. `sessions` table populated by TASK-AUTH-004 token issue.
19. `sessions` table is RLS-protected (tenant-admin sees only own tenant's sessions).

---

## §5 — Verification

```rust
#[tokio::test]
async fn root_admin_lists_all_tenants() {
    let pool = test_pool().await;
    for slug in ["t1", "t2", "t3"] { test_helper::create_tenant(slug).await; }
    let resp = list_tenants(ListQuery { cursor: None, limit: 50 }, &root_admin_claims(), &pool).await.unwrap();
    assert!(resp.items.len() >= 3);
}

#[tokio::test]
async fn tenant_admin_cannot_list_tenants() {
    let err = list_tenants(ListQuery::default(), &tenant_admin_claims(test_tenant()), &pool).await.expect_err("expected Forbidden");
    assert!(matches!(err, AdminError::Forbidden));
}

#[tokio::test]
async fn cross_tenant_subject_list_blocked() {
    let pool = test_pool().await;
    let a = test_helper::create_tenant("a").await;
    let b = test_helper::create_tenant("b").await;
    let _ = test_helper::create_subject(b.id, "b@x.com", "Pwd9!Aa").await;

    let claims_a = tenant_admin_claims(a.id);
    let err = list_subjects(b.id, ListQuery::default(), &claims_a, &pool).await.expect_err("expected Forbidden");
    assert!(matches!(err, AdminError::Forbidden));
}

#[tokio::test]
async fn revoke_invalidates_active_jwts_within_30s() {
    let pool = test_pool().await;
    let redis = test_redis().await;
    let s = test_helper::create_subject_with_active_jwt().await;

    revoke_subject(s.id, &tenant_admin_claims(s.tenant_id), None, &pool, &redis, "r").await.unwrap();

    let jti = test_helper::jti_of(&s).await;
    let denied = deny_list::contains(&redis, &jti).await.unwrap();
    assert!(denied);

    // Verify-side: simulating downstream service
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let result = jwt::verify::verify(&test_helper::jwt_of(&s).await).await;
    assert!(matches!(result, Err(jwt::verify::VerifyError::TokenRevoked)));
}

#[tokio::test]
async fn unrevoke_does_not_clear_deny_list() {
    let pool = test_pool().await;
    let redis = test_redis().await;
    let s = test_helper::create_subject_with_active_jwt().await;
    let jti = test_helper::jti_of(&s).await;

    revoke_subject(s.id, &tenant_admin_claims(s.tenant_id), None, &pool, &redis, "r").await.unwrap();
    unrevoke_subject(s.id, &tenant_admin_claims(s.tenant_id), None, &pool, "r").await.unwrap();

    // §1 #12: deny-list still has the jti
    assert!(deny_list::contains(&redis, &jti).await.unwrap());
}

#[tokio::test]
async fn cursor_tampering_returns_invalid_cursor() {
    let mut cursor = cursor::encode("tenants", Uuid::new_v4(), b"secret");
    cursor.push_str("garbage");
    let err = list_tenants(ListQuery { cursor: Some(cursor), limit: 50 }, &root_admin_claims(), &pool).await.expect_err("expected InvalidCursor");
    assert!(matches!(err, AdminError::InvalidCursor));
}

#[tokio::test]
async fn cursor_pagination_no_duplicates_under_concurrent_insert() {
    let pool = test_pool().await;
    for i in 0..100 { test_helper::create_tenant(&format!("c{i}")).await; }

    let page1 = list_tenants(ListQuery { cursor: None, limit: 50 }, &root_admin_claims(), &pool).await.unwrap();
    test_helper::create_tenant("inserted-during-paging").await;
    let page2 = list_tenants(ListQuery { cursor: page1.next_cursor.clone(), limit: 50 }, &root_admin_claims(), &pool).await.unwrap();

    let p1_ids: HashSet<_> = page1.items.iter().map(|t| t.id).collect();
    let p2_ids: HashSet<_> = page2.items.iter().map(|t| t.id).collect();
    assert!(p1_ids.is_disjoint(&p2_ids), "duplicates between pages");
}

#[tokio::test]
async fn list_response_excludes_password_hash() {
    let pool = test_pool().await;
    let _ = test_helper::create_subject_with_password_hash().await;
    let resp = list_subjects(test_tenant(), ListQuery::default(), &tenant_admin_claims(test_tenant()), &pool).await.unwrap();
    let json = serde_json::to_value(&resp).unwrap();
    let s = json.to_string();
    assert!(!s.contains("password_hash"));
    assert!(!s.contains("$2b$"));   // bcrypt prefix
}
```

---

## §6 — Implementation skeleton

See §3. Listed handler:

```rust
pub async fn list_tenants(query: ListQuery, claims: &Claims, pool: &PgPool)
    -> Result<Page<TenantListItem>, AdminError>
{
    if claims.tenant_id != Uuid::nil() || !claims.roles.contains(&"root-admin".into()) {
        return Err(AdminError::Forbidden);
    }
    let limit = query.limit.min(200) as i64;
    let last_id = query.cursor.map(|c| cursor::decode(&c, "tenants", &cursor_secret())).transpose()?;
    let items: Vec<TenantListItem> = match last_id {
        None => sqlx::query_as("SELECT * FROM tenants ORDER BY id LIMIT $1").bind(limit).fetch_all(pool).await?,
        Some(id) => sqlx::query_as("SELECT * FROM tenants WHERE id > $1 ORDER BY id LIMIT $2").bind(id).bind(limit).fetch_all(pool).await?,
    };
    let next_cursor = items.last().map(|t| cursor::encode("tenants", t.id, &cursor_secret()));
    Ok(Page { items, next_cursor })
}
```

---

## §7 — Dependencies

- **TASK-AUTH-001..004** — tenants/subjects/RLS/JWT all required.
- **TASK-AUTH-006 (downstream)** — sweeper deletes expired sessions rows.
- Crates: `axum`, `sqlx`, `redis`, `hmac@0.12`, `sha2`, `base64`.
- Redis 7+ (deny-list + pub/sub).

---

## §8 — Example payloads

### List tenants (paginated)

```http
GET /v1/admin/tenants?limit=50 HTTP/1.1
Authorization: Bearer <root-admin-jwt>

→ 200 OK
{
  "items": [
    { "id": "550e...", "slug": "cyberskill-jsc", "name": "CyberSkill JSC", "created_at": "...", "suspended": false }
  ],
  "next_cursor": "dGVuYW50czo1NTBlLi4uOmFiYzEyMy4uLg"
}
```

### Revoke subject

```http
POST /v1/admin/subjects/abc-id/revoke HTTP/1.1
Authorization: Bearer <tenant-admin-jwt>
Idempotency-Key: revoke-001

→ 204 No Content
```

### Audit row `auth.subject_revoked`

```json
{
  "kind": "auth.subject_revoked",
  "payload": {
    "subject_id": "abc-...",
    "tenant_id": "550e...",
    "revoked_by_subject_id": "...",
    "revoked_jti_count": 3,
    "request_id": "req_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Bulk revoke (revoke all subjects in tenant) — slice 4+.
- Time-bounded revoke (auto-unrevoke after N hours) — slice 4+.
- Reason taxonomy for revoke (`compromised`, `terminated`, `policy-violation`) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Redis unreachable on revoke | Redis error | Suspended in DB OK; deny-list propagation FAILS; sev-1 (revoke incomplete) | Operator investigates Redis; manual deny-list publish |
| Redis pub/sub subscriber lost | gap in propagation | sev-2 alarm | Restart subscriber; re-replay from sessions table |
| Concurrent revoke (same subject) | DB-level idempotent UPDATE | No-op second call | By design |
| Page cursor invalid (tampered) | HMAC verify fails | 400 invalid_cursor | Caller restarts paging |
| Page cursor signed by previous secret (rotation) | HMAC verify fails | 400 invalid_cursor | Caller restarts paging |
| Cross-tenant list/revoke (API bypass) | RLS at DB | 0 rows / RLS violation | By design (defense in depth) |
| Subject not found | DB lookup miss | 404 subject_not_found | Caller fixes id |
| Tenant-admin trying to revoke root-admin | role hierarchy check | 403 cannot_revoke_higher_role | Operator escalates to root-admin |
| Unrevoke clears deny-list (regression) | §5 test asserts | PR blocked | By design |
| Sessions table unbounded growth | sweeper deletes expired | Storage OK | TASK-AUTH-006 sweeper |
| Idempotency key reuse different subject | hash mismatch | 409 idempotency_key_reuse | Caller uses different key |
| Listing while RLS context unset | empty result (RLS filters all) | 0 items returned | Handler MUST set `app.tenant_id` |
| password_hash leaked in response (regression) | §5 test grep | PR blocked | By design |
| Suspended subject listed by default | `?include_suspended=false` filter | Hidden | By design |
| Revoke propagation > 30s | OTel histogram alarm | sev-2 | Operator investigates Redis pub/sub |
| Subject re-uses JWT after unrevoke | deny-list still has jti | 401 token_revoked | Subject re-authenticates |
| List endpoint slow (>100ms p95) | OTel | sev-3 | Operator investigates DB indexes |

---

## §11 — Notes

- Cursor pagination IS stable under concurrent inserts. Offset paging is not. The HMAC signature catches tampering attempts; a cursor from one deployment doesn't validate against another (signing key per deployment).
- The deny-list TTL = JWT max-exp (1h) means deny-list naturally garbage-collects. No sweeper needed for deny-list.
- Unrevoke does NOT clear the deny-list — security default is "operator's revoke intent persists." If the operator wanted the subject to keep their old session, they shouldn't have revoked. Forcing fresh login proves the operator's restoration intent.
- Redis pub/sub for revocation propagation gives near-instant (<100ms) propagation in practice. The 30s SLO is conservative; alarms fire if propagation degrades.
- The `sessions` table is the registry of issued JWTs. Without it, revoke can't enumerate which jtis to deny-list. The table is RLS-protected (tenant-admin sees only own sessions).
- `password_hash` exclusion from list responses is enforced by the `SubjectListItem` struct shape — the field doesn't exist in the response type. Compile-time guarantee, not a runtime check.
- The `?include_suspended=false` default keeps routine UIs clean. Revoke-management UIs explicitly set `?include_suspended=true`.
- Idempotent revoke + unrevoke means operator double-clicks are safe. Without idempotency, a network glitch could produce duplicate audit rows.
- HMAC cursor signing uses HKDF derivation from the deployment secret; rotation of the deployment secret invalidates all outstanding cursors (acceptable — clients restart paging).

---

*End of TASK-AUTH-005. Status: draft (10/10 target).*
