---
id: TASK-AUTH-002
title: "Subject create — POST /v1/admin/subjects with bcrypt + role allow-list + idempotency + RLS-enforced cross-tenant blocking"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
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
related_tasks: [TASK-AUTH-001, TASK-AUTH-003, TASK-AUTH-004, TASK-AUTH-005, TASK-AUTH-006, TASK-AUTH-101]
depends_on: [TASK-AUTH-001]
blocks: [TASK-AUTH-004, TASK-AUTH-005, TASK-AUTH-006, TASK-AUTH-102, TASK-AUTH-106, TASK-AUTH-107]

source_pages:
  - website/docs/modules/auth.html#subject-create
source_decisions:
  - DEC-115 (passwords hashed with bcrypt cost 12; argon2 deferred to slice 4)
  - DEC-116 (slice-1 role allow-list: tenant-admin + tenant-member only)
  - DEC-117 (cross-tenant subject creation forbidden — even root-admin must create in target tenant via tenant-scoped admin context)
  - PDPL Art. 6 (data minimisation: password hash only, never plaintext echo)
  - NIST SP 800-63B (password storage: bcrypt cost ≥ 10; we use 12)

language: rust 1.81
service: cyberos/services/auth/
new_files:
  - services/auth/src/admin/subjects.rs
  - services/auth/src/admin/password.rs
  - services/auth/src/admin/roles.rs
  - services/auth/migrations/0003_subjects.sql
  - services/auth/tests/admin_subject_create_test.rs
  - services/auth/tests/admin_subject_create_test.rs
  - services/auth/tests/admin_subject_create_test.rs
modified_files:
  # add subjects to TENANT_SCOPED_TABLES
  - services/auth/src/rls/templates.rs
allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test admin_subject
disallowed_tools:
  - echo `password` in any response, log, or audit row
  - lower bcrypt cost below 12 (per DEC-115)
  #5; TASK-AUTH-101 expands)
  - allow roles outside the slice-1 allow-list (per §1
  #1; root-admin must switch tenant context)
  - allow cross-tenant creation by any role (per §1
  #6)
  - skip idempotency support (per §1

effort_hours: 6
subtasks:
  - "0.5h: 0003_subjects.sql migration (table + UNIQUE(tenant_id, email) + CHECK email)"
  - "0.5h: roles.rs allow-list + validation"
  - "0.5h: password.rs (validate complexity + bcrypt cost-12 hash + verify helper)"
  - "1.0h: create_subject handler with role-gate + RLS context + transaction"
  - "0.5h: Idempotency-Key handling (mirrors TASK-AUTH-001 §1 #5)"
  - "0.5h: canonical::subject_created memory audit row builder (NO password fields)"
  - "0.5h: TLS-required check (refuse if request not over HTTPS in non-test env)"
  - "1.5h: Tests — happy + 401 + 403 cross-tenant + 409 dupe + 400 invalid email/role/password + idempotent + p95 + audit-row-no-password + bcrypt-verify"
risk_if_skipped: "AUTH has no users beyond bootstrap operator. Every downstream module (TASK-AI-006 JWT extraction, TASK-AUTH-004 JWT issuance, TASK-AUTH-005 admin REST) has nothing to authenticate. Multi-tenancy works structurally (TASK-AUTH-001) but no humans can log in. Without bcrypt-cost discipline + role allow-list, password security degrades to 'whatever the developer felt like that week.'"
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** expose `POST /v1/admin/subjects` for creating new authenticated subjects (humans + service accounts). Each invocation:

1. **MUST** require caller's `tenant_id == target_tenant_id` AND caller's `roles` contains `"tenant-admin"` (or caller is root-admin in tenant 0 AND has explicitly switched into the target tenant via a separate `X-Switch-Tenant` header — root-admin cannot create subjects from tenant-0 context). Cross-tenant creation is structurally forbidden — RLS on the `subjects` table additionally enforces this at the DB layer.
2. **MUST** accept request body `{ "tenant_id": <uuid>, "email": <string>, "password": <string>, "roles": [<string>] }`. Validation: email matches `^[^@\s]+@[^@\s]+\.[^@\s]+$`; password meets complexity rules per §1 #4; roles are all in the allow-list per §1 #5.
3. **MUST** hash the password using bcrypt at cost 12 (DEC-115; matches NIST SP 800-63B floor). The plaintext password is zeroised from memory immediately after hashing (using the `zeroize` crate). The bcrypt hash is the ONLY representation written to disk.
4. **MUST** validate password complexity:
    - Length: 12 ≤ len ≤ 128 (NIST SP 800-63B; longer than 8 floor; cap protects against bcrypt input limits).
    - Must contain at least 3 of: lowercase, uppercase, digit, special char.
    - Must NOT match the user's email local-part (case-insensitive).
    - Must NOT be in the top-10K-common-passwords list embedded at compile time.
   Failures return `400 BAD_REQUEST` with body `{"error":"weak_password","reasons":["too_short","no_digit"]}` — multiple reasons reported in one response.
5. **MUST** restrict assignable roles to a closed allow-list defined in `roles.rs`. Slice 1: `{"tenant-admin", "tenant-member"}`. Unknown role returns `400 BAD_REQUEST` with `{"error":"unknown_role","role":"<name>","allowed":[...]}`. The allow-list expands in TASK-AUTH-101 to 22 roles.
6. **MUST** support idempotency via `Idempotency-Key` header (same semantics as TASK-AUTH-001 §1 #5). Repeat POST with same key + same body → return prior subject (same id); same key + different body → `409` with `idempotency_key_reuse`.
7. **MUST** emit exactly one `auth.subject_created` memory audit row per new subject. The row carries `subject_id`, `tenant_id`, `email_hash16` (SHA-256[..16] of email — privacy-preserving identifier), `roles`, `created_by_subject_id`, `request_id`. The row MUST NOT contain plaintext password, password hash, OR the full email — `email_hash16` is the privacy-safe identifier.
8. **MUST** return `{ "id": <uuid>, "tenant_id": <uuid>, "email": <string>, "roles": [...], "suspended": false, "created_at": <ISO8601> }` — NEVER the password hash, NEVER the plaintext password.
9. **MUST** return `409 CONFLICT` with `{"error":"email_taken","tenant_id":<uuid>,"email":"<email>"}` if the (tenant_id, email) pair already exists. UNIQUE constraint on `(tenant_id, email)` enforces at DB layer.
10. **MUST** complete, **excluding the password-hashing cost**, in ≤ 200ms p95. The budget covers validation, the HIBP breach check, the DB write, the audit row, RLS and JWT verification — everything the endpoint does *besides* hashing. bcrypt at cost 12 is a deliberate spend on top, and it **MUST NOT** be reduced to fit a latency budget. *(Amended 2026-07-11 — see §12.)*
11. **MUST** require HTTPS transport (refuse with `400 BAD_REQUEST` `{"error":"https_required"}` if `X-Forwarded-Proto: https` not present in non-test environments). Plaintext password over HTTP is a categorical no.
12. **MUST** atomically apply `subjects` insert + idempotency record + audit row in a SINGLE Postgres transaction (mirrors TASK-AUTH-001 §1 #12).
13. **MUST** emit OTel span `auth.create_subject` with attributes `tenant_id`, `email_hash16`, `roles_count`, `outcome` (created | idempotent_replay | conflict | forbidden | invalid_input | weak_password). Span MUST NOT carry email or password.
14. **SHOULD** emit OTel metrics:
    - `auth_subject_create_total{outcome, tenant_id}` (counter).
    - `auth_subject_create_latency_ms` (histogram; SLO p95 < 200ms **net of hashing**, per §1 #10 as amended).
    - `auth_subject_count{tenant_id}` (gauge).

---

## §2 — Why this design (rationale for humans)

**Why bcrypt cost 12 (§1 #3)?** NIST SP 800-63B floor is cost 10 (~50ms in 2026); cost 12 (~150ms) gives 4× the work factor — meaningfully more resistant to offline cracking attempts. Cost 14 (~600ms) would be paranoid; cost 10 would be barely-compliant. 12 is the deliberate middle. Argon2id is preferred per modern guidance; deferred to slice 4 (TASK-AUTH-114) to avoid introducing two password formats simultaneously. The DEC-115 trade-off is explicit.

**Why the password complexity rules in §1 #4?** NIST SP 800-63B walks back from arbitrary complexity rules ("must contain symbol", etc.) toward "length + breach-list check." We adopt the modern approach: 12-char minimum (length is the dominant factor) + character-class diversity (defence against the most common weak patterns) + breach-list check via embedded top-10K. The breach list is small (~80KB compressed) and embedded in the binary; lookups are constant-time hash-set membership checks.

**Why a role allow-list (§1 #5)?** Free-form role strings invite typos (`"tenant_admin"` vs `"tenant-admin"`) and accidental privilege creep (operator typos `"tenant-superadmin"` and creates a role nothing checks for). The closed allow-list catches typos at the API boundary; expansion is a deliberate task (TASK-AUTH-101 ships 22 roles).

**Why cross-tenant creation forbidden even for root-admin (§1 #1)?** Root-admin's privilege is "I can do anything in tenant 0 AND I can switch tenant context." But creating a subject in tenant X from tenant-0 context is two operations conflated into one — and the audit row would attribute the creation to "root-admin in tenant 0," obscuring which actual tenant the subject belongs to. Forcing root-admin to switch context first (via `X-Switch-Tenant`) makes the action explicit and the audit clean.

**Why `email_hash16` in audit instead of plaintext email (§1 #7)?** Email is PII. Audit rows are queried, mirrored to OBS, possibly exported. Storing plaintext email in every audit row creates an everywhere-PII problem. The 16-hex prefix of SHA-256(email) is enough to disambiguate ~1 in 10⁹ subjects (collision-safe at our scale) without exposing the actual address. Forensic operations needing the actual email join via `subject_id` against the `subjects` table (where the email is stored once, RLS-protected).

**Why HTTPS-required (§1 #11)?** Plaintext password over HTTP is a credentials-on-the-wire failure mode. `X-Forwarded-Proto: https` is the standard reverse-proxy signal; the test environment skip is necessary for unit tests but production refuses without it. The check is at the API boundary, not somewhere downstream — reject as early as possible.

**Why password-zeroising after hashing (§1 #3)?** Plaintext passwords lingering in process memory get into core dumps, tracing tools, panic backtraces. The `zeroize` crate's `Zeroizing<String>` wrapper ensures the bytes are overwritten on Drop. The cost is one allocation; the benefit is "passwords don't leak via heap inspection."

**Why does the response not include the password hash (§1 #8)?** No legitimate caller needs the hash. Including it in the response would let a compromised admin client extract every subject's hash by listing subjects — an offline-cracking surface that doesn't need to exist. The hash is for `bcrypt::verify` only, called from the JWT issuance path (TASK-AUTH-004).

**Why bcrypt and not Argon2 right now?** Argon2id is technically superior (memory-hard; resistant to GPU cracking). Bcrypt is widely supported, well-understood, and has a large ecosystem. Slice 1 ships bcrypt; slice 4 (TASK-AUTH-114) introduces argon2 with a migration path (existing subjects re-hash on next login). Shipping both at slice 1 doubles the verification path complexity for marginal security gain at our threat model.

---

## §3 — API contract (formal spec for AI-agent implementers)

```rust
// services/auth/src/admin/subjects.rs
#[derive(Deserialize)]
pub struct CreateSubjectRequest {
    pub tenant_id: Uuid,
    pub email: String,
    #[serde(serialize_with = "redact")]   // never serialize back
    pub password: Zeroizing<String>,
    pub roles: Vec<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CreateSubjectResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub suspended: bool,
    pub created_at: DateTime<Utc>,
    // password_hash deliberately NOT in response
}

#[derive(Debug, thiserror::Error)]
pub enum SubjectError {
    #[error("forbidden: cross-tenant creation requires X-Switch-Tenant context")]
    Forbidden,
    #[error("invalid email")]
    InvalidEmail,
    #[error("weak password: {reasons:?}")]
    WeakPassword { reasons: Vec<&'static str> },
    #[error("unknown role: {role!r}")]
    UnknownRole { role: String },
    #[error("email taken: tenant={tenant_id} email={email}")]
    Conflict { tenant_id: Uuid, email: String },
    #[error("https required")]
    HttpsRequired,
    #[error("idempotency key reuse")]
    IdempotencyKeyReuse { prior_hash: String },
    #[error("bcrypt failed: {0}")]
    Bcrypt(String),
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
}
```

```sql
-- services/auth/migrations/0003_subjects.sql
CREATE TABLE subjects (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
    email         TEXT NOT NULL CHECK (email ~ '^[^@\s]+@[^@\s]+\.[^@\s]+$'),
    password_hash TEXT NOT NULL,
    roles         TEXT[] NOT NULL DEFAULT '{}',
    suspended     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, email)
);

CREATE INDEX subjects_tenant_email_idx ON subjects(tenant_id, lower(email));
```

```rust
// services/auth/src/admin/roles.rs
pub const SLICE_1_ALLOWED_ROLES: &[&str] = &["tenant-admin", "tenant-member"];

pub fn validate_role_slice1(role: &str) -> Result<(), SubjectError> {
    if !SLICE_1_ALLOWED_ROLES.contains(&role) {
        return Err(SubjectError::UnknownRole { role: role.into() });
    }
    Ok(())
}
```

```rust
// services/auth/src/admin/password.rs
const TOP_10K_COMMON: &[&str] = include!(concat!(env!("OUT_DIR"), "/top_10k_passwords.rs"));
static COMMON_SET: LazyLock<HashSet<&'static str>> = LazyLock::new(|| TOP_10K_COMMON.iter().copied().collect());

pub fn validate_complexity(password: &str, email: &str) -> Result<(), SubjectError> {
    let mut reasons = vec![];
    if password.len() < 12 { reasons.push("too_short"); }
    if password.len() > 128 { reasons.push("too_long"); }

    let mut classes = 0;
    if password.chars().any(|c| c.is_ascii_lowercase()) { classes += 1; }
    if password.chars().any(|c| c.is_ascii_uppercase()) { classes += 1; }
    if password.chars().any(|c| c.is_ascii_digit())     { classes += 1; }
    if password.chars().any(|c| !c.is_alphanumeric())   { classes += 1; }
    if classes < 3 { reasons.push("insufficient_character_classes"); }

    if let Some(local) = email.split('@').next() {
        if password.to_lowercase().contains(&local.to_lowercase()) {
            reasons.push("contains_email_localpart");
        }
    }
    if COMMON_SET.contains(password) { reasons.push("breach_list_match"); }

    if reasons.is_empty() { Ok(()) } else { Err(SubjectError::WeakPassword { reasons }) }
}

pub fn hash_password(password: &str) -> Result<String, SubjectError> {
    bcrypt::hash(password, 12).map_err(|e| SubjectError::Bcrypt(e.to_string()))
}
```

---

## §4 — Acceptance criteria

1. **Tenant-admin creates subject in own tenant** → 201 + body (no password fields).
2. **Cross-tenant attempt (admin of A creates in B) returns 403** with `forbidden`.
3. **Root-admin in tenant 0 without X-Switch-Tenant returns 403**.
4. **Duplicate (tenant, email) returns 409** with `email_taken`.
5. **Invalid email (no @) returns 400** with `invalid_email`.
6. **Unknown role returns 400** with `unknown_role` + allowed list.
7. **Weak password (too short) returns 400** with `reasons: [too_short]`.
8. **Weak password (in top-10K breach list) returns 400** with `reasons: [breach_list_match]`.
9. **Weak password (contains email local-part) returns 400** with `reasons: [contains_email_localpart]`.
10. **Password complexity: only 2 char classes returns 400** (e.g., all lowercase + digits).
11. **HTTP (no X-Forwarded-Proto: https) in non-test env returns 400** with `https_required`.
12. **Password stored as bcrypt cost 12 hash** — `bcrypt::verify(plaintext, stored_hash)` returns true; hash starts with `$2b$12$`.
13. **Response NEVER contains password or password_hash** — JSON output assertion.
14. **memory audit row emitted** with `email_hash16` (NOT plaintext email) and NO password fields.
15. **Idempotent replay returns same subject id** with same key + body.
16. **Latency p95 < 200ms EXCLUDING the bcrypt hash** (§1 #10 as amended, §12 A1). The test calibrates the hash cost on the host and subtracts it, and serves the HIBP breach check from a local stub — so the assertion measures our code, not the CPU's hashing speed or the distance to Cloudflare.
17. **Cross-tenant blocked at RLS layer** — even if API check is bypassed, `subjects.tenant_id != current_setting('app.tenant_id')::uuid` filters out.

---

## §5 — Verification

```rust
#[tokio::test]
async fn tenant_admin_creates_subject() {
    let pool = test_pool().await;
    let claims = tenant_admin_claims(test_tenant_a());
    let resp = create_subject(
        CreateSubjectRequest {
            tenant_id: test_tenant_a(), email: "alice@cyberos.world".into(),
            password: Zeroizing::new("CorrectHorseBatteryStaple9!".into()),
            roles: vec!["tenant-member".into()],
        }, None, &pool, &claims, "req",
    ).await.unwrap();
    assert_eq!(resp.email, "alice@cyberos.world");
    assert!(memory_test_helper::has_row("auth.subject_created", &resp.id.to_string()).await);

    // Verify response doesn't contain password fields
    let json = serde_json::to_value(&resp).unwrap();
    assert!(json.get("password").is_none());
    assert!(json.get("password_hash").is_none());
}

#[tokio::test]
async fn cross_tenant_creation_forbidden() {
    let pool = test_pool().await;
    let claims = tenant_admin_claims(test_tenant_a());   // admin of A
    let err = create_subject(
        CreateSubjectRequest {
            tenant_id: test_tenant_b(),                  // tries to create in B
            email: "x@y.com".into(),
            password: Zeroizing::new("LongStrongPwd9!".into()),
            roles: vec!["tenant-member".into()],
        }, None, &pool, &claims, "req",
    ).await.expect_err("expected Forbidden");
    assert!(matches!(err, SubjectError::Forbidden));
}

#[tokio::test]
async fn weak_password_returns_multiple_reasons() {
    let pool = test_pool().await;
    let claims = tenant_admin_claims(test_tenant_a());
    let err = create_subject(
        CreateSubjectRequest {
            tenant_id: test_tenant_a(), email: "alice@x.com".into(),
            password: Zeroizing::new("alice".into()),    // short + breach + email-localpart
            roles: vec!["tenant-member".into()],
        }, None, &pool, &claims, "req",
    ).await.expect_err("expected WeakPassword");
    match err {
        SubjectError::WeakPassword { reasons } => {
            assert!(reasons.contains(&"too_short"));
            assert!(reasons.contains(&"contains_email_localpart"));
        }
        e => panic!("wrong: {e:?}"),
    }
}

#[tokio::test]
async fn breach_list_match_rejected() {
    let pool = test_pool().await;
    let claims = tenant_admin_claims(test_tenant_a());
    let err = create_subject(
        CreateSubjectRequest {
            tenant_id: test_tenant_a(), email: "x@y.com".into(),
            password: Zeroizing::new("password123!Aa".into()),   // common variant
            roles: vec!["tenant-member".into()],
        }, None, &pool, &claims, "req",
    ).await.expect_err("expected WeakPassword");
    match err {
        SubjectError::WeakPassword { reasons } => assert!(reasons.contains(&"breach_list_match")),
        e => panic!("wrong: {e:?}"),
    }
}

#[tokio::test]
async fn password_stored_as_bcrypt_cost_12() {
    let pool = test_pool().await;
    let claims = tenant_admin_claims(test_tenant_a());
    let resp = create_subject(
        CreateSubjectRequest {
            tenant_id: test_tenant_a(), email: "b@c.com".into(),
            password: Zeroizing::new("CorrectHorseBatteryStaple9!".into()),
            roles: vec!["tenant-member".into()],
        }, None, &pool, &claims, "req",
    ).await.unwrap();
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM subjects WHERE id = $1")
        .bind(resp.id).fetch_one(&pool).await.unwrap();
    assert!(hash.starts_with("$2b$12$"), "wrong bcrypt format/cost: {hash}");
    assert!(bcrypt::verify("CorrectHorseBatteryStaple9!", &hash).unwrap());
}

#[tokio::test]
async fn audit_row_has_no_password_or_email() {
    let resp = ...; // create as above
    let row = memory_test_helper::find_latest_row("auth.subject_created").unwrap();
    let payload_json = serde_json::to_string(&row.payload).unwrap();
    assert!(!payload_json.contains("CorrectHorseBatteryStaple9!"));
    assert!(!payload_json.contains("alice@cyberos.world"));
    assert!(payload_json.contains("email_hash16"));
}

#[tokio::test]
async fn p95_latency_under_200ms() {
    let mut samples = vec![];
    for i in 0..200 {
        let t0 = std::time::Instant::now();
        let _ = create_subject(/* ... unique email per iteration ... */).await.unwrap();
        samples.push(t0.elapsed().as_millis() as u64);
    }
    samples.sort();
    let p95 = samples[(samples.len() as f64 * 0.95) as usize];
    assert!(p95 < 200, "p95 {p95}ms exceeds 200ms");
}

#[tokio::test]
async fn rls_blocks_cross_tenant_select() {
    let pool = test_pool().await;
    let claims_a = tenant_admin_claims(test_tenant_a());
    let _ = create_subject(/* in tenant A */).await.unwrap();

    // Switch context to tenant B; try to SELECT
    sqlx::query("SET app.tenant_id = $1").bind(test_tenant_b()).execute(&pool).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subjects").fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0, "RLS must block tenant B from seeing tenant A's subjects");
}
```

---

## §6 — Implementation skeleton

```rust
pub async fn create_subject(
    req: CreateSubjectRequest, idempotency_key: Option<String>, pool: &PgPool,
    claims: &Claims, request_id: &str,
) -> Result<CreateSubjectResponse, SubjectError> {
    // §1 #11
    if !is_test_env() && !claims.is_https_request {
        return Err(SubjectError::HttpsRequired);
    }

    // §1 #1
    if claims.tenant_id != req.tenant_id || !claims.roles.contains(&"tenant-admin".into()) {
        return Err(SubjectError::Forbidden);
    }

    // §1 #2 + §1 #4
    validate_email(&req.email)?;
    password::validate_complexity(&req.password, &req.email)?;
    for r in &req.roles { roles::validate_role_slice1(r)?; }

    // §1 #3 (zeroising happens via Drop on Zeroizing<String>)
    let hash = password::hash_password(&req.password)?;

    let mut tx = pool.begin().await?;
    let body_hash = hex::encode(sha256(/* without password */));
    if let Some(key) = &idempotency_key {
        if let Some(prior) = idempotency::lookup(&mut tx, key, "/v1/admin/subjects").await? {
            if prior.request_body_hash != body_hash {
                return Err(SubjectError::IdempotencyKeyReuse { prior_hash: prior.request_body_hash[..16].into() });
            }
            return Ok(serde_json::from_value(prior.response_body).unwrap());
        }
    }

    // RLS context for INSERT
    sqlx::query("SET LOCAL app.tenant_id = $1").bind(req.tenant_id).execute(&mut *tx).await?;

    let row: CreateSubjectResponse = sqlx::query_as(
        "INSERT INTO subjects (tenant_id, email, password_hash, roles)
         VALUES ($1, $2, $3, $4)
         RETURNING id, tenant_id, email, roles, suspended, created_at",
    ).bind(req.tenant_id).bind(&req.email).bind(&hash).bind(&req.roles)
     .fetch_one(&mut *tx).await
     .map_err(|e| if is_unique_violation(&e) {
         SubjectError::Conflict { tenant_id: req.tenant_id, email: req.email.clone() }
     } else { SubjectError::Db(e) })?;

    let email_hash16 = hex::encode(&sha256(req.email.as_bytes())[..8]);
    memory::emit_in_tx(&mut tx, memory::canonical::subject_created(
        row.id, row.tenant_id, &email_hash16, &row.roles, claims.subject_id, request_id,
    )).await?;

    if let Some(key) = &idempotency_key {
        idempotency::insert(&mut tx, key, "/v1/admin/subjects", &body_hash, &row).await?;
    }

    tx.commit().await?;
    Ok(row)
    // `req.password` (Zeroizing<String>) drops here → bytes overwritten
}
```

---

## §7 — Dependencies

- **TASK-AUTH-001** — Tenants table + RLS templates (subjects MUST be added to `TENANT_SCOPED_TABLES`).
- **TASK-AUTH-004 (downstream)** — JWT issuance reads `password_hash` for `bcrypt::verify` during login.
- **TASK-AUTH-101 (downstream)** — Expands the role allow-list from 2 to 22.
- **TASK-AUTH-114 (downstream)** — Argon2id migration; this task ships bcrypt only.
- Crates: `bcrypt@0.15`, `zeroize@1`, `regex@1`, `sha2@0.10`, `hex@0.4`, `axum`, `sqlx`, `serde`, `thiserror`.
- Build dependency: `top_10k_passwords.rs` generated at build time from a vendored breach list (~80KB compressed).

---

## §8 — Example payloads

### Successful create

```http
POST /v1/admin/subjects HTTP/1.1
Authorization: Bearer <tenant-admin-jwt>
Content-Type: application/json
X-Forwarded-Proto: https
Idempotency-Key: 7e57c0de-...

{
  "tenant_id": "550e...", "email": "alice@cyberos.world",
  "password": "CorrectHorseBatteryStaple9!",
  "roles": ["tenant-member"]
}

→ 201 Created
{
  "id": "abc-...", "tenant_id": "550e...", "email": "alice@cyberos.world",
  "roles": ["tenant-member"], "suspended": false, "created_at": "2026-05-15T..."
}
```

### Weak password

```http
→ 400 Bad Request
{ "error": "weak_password", "reasons": ["too_short", "insufficient_character_classes", "contains_email_localpart"] }
```

### Audit row `auth.subject_created` (no password, no plaintext email)

```json
{
  "kind": "auth.subject_created",
  "payload": {
    "subject_id": "abc-...",
    "tenant_id": "550e...",
    "email_hash16": "4b8c0d2f1a7e9c3b",
    "roles": ["tenant-member"],
    "created_by_subject_id": "...",
    "request_id": "req_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Argon2id migration (TASK-AUTH-114).
- 22-role allow-list expansion (TASK-AUTH-101).
- MFA enrolment (TASK-AUTH-115).
- Password rotation policy + history (TASK-AUTH-116).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cross-tenant create attempt | claims check + RLS | 403 forbidden | Caller switches tenant context |
| Duplicate (tenant, email) | UNIQUE | 409 email_taken | Caller uses different email |
| Invalid email format | regex | 400 invalid_email | Caller fixes email |
| Weak password (any of 4 reasons) | complexity check | 400 weak_password with reasons | Caller picks stronger password |
| Unknown role | allow-list check | 400 unknown_role | Caller uses allowed role |
| HTTPS missing | header check | 400 https_required | Caller uses HTTPS |
| bcrypt failure | bcrypt::hash error | 500 bcrypt | Operator investigates (rare) |
| memory outbox insert fails | sqlx error | tx rollback; 500 memory_failed | Operator investigates outbox |
| RLS blocks INSERT (tenant_id mismatch) | postgres error | 403 forbidden | Caller correct tenant_id |
| Latency > 200ms | OTel histogram | sev-3 alarm | Operator investigates bcrypt cost OR DB |
| Idempotent replay | idempotency lookup | 201 with prior id | By design |
| Idempotency key reuse w/ different body | hash mismatch | 409 idempotency_key_reuse | Caller uses different key |
| Plaintext password in request log | `Zeroizing` + custom Debug impl scrubs | Log shows `<REDACTED>` | By design |
| Plaintext password in response | response struct doesn't contain field | N/A | By design (compile-time guarantee) |
| Plaintext password in audit row | builder doesn't include field | N/A | By design |
| Plaintext email in audit row | `email_hash16` only | N/A | By design |
| Subject suspended (`suspended: true`) at creation | not supported at slice 1 | 400 if `suspended` in request body | Caller omits field |
| Top-10K list out of date | manual refresh process | False negatives possible | Refresh quarterly via TASK-AUTH-006 sweeper |
| Concurrent creates with same email | UNIQUE serializes | One succeeds; other 409 | By design |

---

## §11 — Notes

- ~~Bcrypt cost 12 ≈ 150ms hash time. The 200ms p95 budget (§1 #10) leaves ~50ms for DB + validation + audit. Cost 14 would blow the budget; cost 10 would weaken security below NIST floor.~~ **Superseded by §12 A1.** The 150ms figure was wrong: bcrypt cost 12 measures **209ms** on a server-class core, so the old budget left *negative* room for the DB, validation and audit. The budget now excludes hashing entirely, and the cost factor is fixed at 12 — OWASP's floor is 10 and the recommendation is to go higher, so it may not be lowered to buy latency.
- Top-10K breach list embedded at compile time keeps the check offline + constant-time. The list is refreshed quarterly via a build-time script pulling from HaveIBeenPwned's vetted lists.
- `Zeroizing<String>` (zeroize crate) ensures plaintext passwords are overwritten in heap memory on Drop. Combined with custom Debug impl that prints `<REDACTED>`, the password never appears in logs, panic backtraces, or core dumps.
- The `email_hash16` audit field is the privacy-vs-debuggability balance. 16 hex chars (8 bytes) of SHA-256 disambiguates 1-in-10⁹ subjects (collision-safe at our scale) without exposing the address. Forensic needs (e.g., "what was the email of subject_id=X") query the subjects table directly under appropriate authorisation.
- Cross-tenant creation is blocked at TWO layers: API claims check (§1 #1) AND RLS at the DB level. Even if the API check has a bug, RLS prevents the INSERT from reaching the wrong tenant. Defence in depth.
- The subjects table is added to `TENANT_SCOPED_TABLES` in `rls/templates.rs` (TASK-AUTH-001 §3) — tenant create automatically applies the standard tenant_id RLS policy.
- Argon2id migration (slice 4, TASK-AUTH-114) will work as: on next successful login, if `password_hash` starts with `$2b$`, compute argon2 hash and update the row. Both formats coexist during migration; TASK-AUTH-004's verify path tries argon2 first then bcrypt.
- The slice-1 role allow-list (`tenant-admin`, `tenant-member`) is intentionally minimal. Most operational roles (`finance-admin`, `pii-officer`, etc.) ship in TASK-AUTH-101 with the 22-role expansion.

---

## §12 — Amendments

### A1 — 2026-07-11 — the p95 SLO was arithmetically impossible (§1 #10, §5, §11)

**Approved by:** Stephen Cheng (CTO), 2026-07-11.
**Found by:** TASK-AUTH-111's gate run, where `create_subject_p95_latency_under_200ms` failed persistently on a
change that does not touch this endpoint.

**What was wrong.** §1 #10 required p95 ≤ 200ms *including* the bcrypt hash, on the stated basis that "bcrypt
cost 12 ≈ 150ms; remaining 50ms for DB + validation + audit". That figure was wrong. Measured on a
server-class core:

| bcrypt cost | time per hash |
|---|---|
| 10 | 52 ms |
| 11 | 104 ms |
| **12 (ours)** | **209 ms** |
| 13 | 421 ms |

Cost 12 alone exceeds the entire 200ms budget. The remaining budget for the DB write, the breach check, the
audit row and validation was **negative**. No implementation could have satisfied this SLO; the test was not
detecting slow code, it was reporting that the arithmetic did not close. It had been failing locally and
passing in CI only because CI silently applied a 500ms threshold — so the gate was green on the machine that
gates merges and red on every developer's machine, which is the worst of both worlds.

**Second defect, found on the way.** The test timed a live HTTPS call to `api.pwnedpasswords.com` on each of
its 100 iterations. A build gate whose verdict depends on the round-trip time from the developer's chair to
Cloudflare is not a gate. Worse, the code behind it built a **new `reqwest::Client` per call** — discarding
the connection pool and paying a fresh DNS + TCP + TLS handshake on *every password set in production*. Fixed
in `hibp.rs`; the client is now built once and reused. That is a real latency win for every signup and
password change, and it existed only because a latency test nobody could satisfy was being tolerated.

**What we did NOT do: lower the cost factor.** The original §1 #10 offered "switch to cost 10 ONLY via task
amendment" as the escape hatch. This amendment explicitly closes that hatch. [OWASP's Password Storage Cheat
Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) sets bcrypt's
minimum work factor at 10 and says it should be "as large as verification server performance will allow";
OWASP now treats bcrypt itself as the legacy choice and recommends Argon2 for new systems, with 2026
commentary pointing at work factors of 13-14. Dropping 12 → 10 would walk backwards to OWASP's floor to
satisfy a budget that was miscalculated in the first place. **The SLO was wrong, not the hashing.**

**The amended rule.** §1 #10 now budgets 200ms p95 for everything the endpoint does *besides* hashing.
`create_subject_p95_overhead_under_200ms_above_hashing` enforces it by (a) serving HIBP from a local stub, so
the number contains no internet, and (b) **calibrating bcrypt on the host at run time** and subtracting it,
so the verdict is identical on a laptop, a CI runner and a prod cell. The hash cost is a feature we are
choosing to pay for, not a regression to be detected.

**Open question deliberately left open.** Whether to migrate from bcrypt to Argon2id, per OWASP's current
recommendation for new systems. That is a security decision with a migration path attached (rehash-on-login),
and it is not this amendment's business. Logged in §9.

---

*End of TASK-AUTH-002. Status: draft (10/10 target). Amended A1 (2026-07-11).*
