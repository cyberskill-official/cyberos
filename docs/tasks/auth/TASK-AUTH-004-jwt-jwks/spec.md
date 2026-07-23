---
id: TASK-AUTH-004
title: "JWT issuance + JWKS endpoint (RS256) with tenant_id + agent_persona + scope_grants + dual-rate-limit + jti dedup"
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
module: auth
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
related_tasks: [TASK-AUTH-001, TASK-AUTH-002, TASK-AUTH-003, TASK-AUTH-005, TASK-AUTH-006, TASK-AUTH-007, TASK-MCP-004, TASK-AI-001, TASK-AI-006]
depends_on: [TASK-AUTH-002, TASK-AUTH-003]
blocks: [TASK-AUTH-005, TASK-AI-006, TASK-MCP-004, TASK-AUTH-006, TASK-OBS-002, TASK-CHAT-002, TASK-MCP-001, TASK-AUTH-104, TASK-AUTH-103, TASK-EMAIL-002]

source_pages:
  - website/docs/modules/auth.html#jwt
  - RFC 7519 (JWT)
  - RFC 7517 (JWKS)
  - RFC 7518 (JWA — RS256)
source_decisions:
  - DEC-120 (RS256 + RSA-2048; ECDSA deferred to TASK-AUTH-110)
  - DEC-121 (1h access token + 24h key-rotation overlap)
  - DEC-122 (dual rate-limit: per-IP AND per-account; either trips)
  - DEC-123 (jti dedup at consuming services using bloom filter; no central dedup store)

language: rust 1.81
service: cyberos/services/auth/
new_files:
  - services/auth/src/jwt/mod.rs
  - services/auth/src/jwt/issue.rs
  - services/auth/src/jwt/verify.rs
  - services/auth/src/jwks/mod.rs
  - services/auth/src/jwks/rotation.rs
  - services/auth/src/rate_limit.rs
  - services/auth/migrations/0006_signing_keys.sql
  - services/auth/tests/jwt_roundtrip_test.rs
  - services/auth/tests/jwt_roundtrip_test.rs
  - services/auth/tests/rls_isolation_test.rs
  - services/auth/tests/admin_list_test.rs
modified_files:
  # add suspended check
  - services/auth/src/admin/subjects.rs
allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test jwt && cargo test jwks
disallowed_tools:
  #14)
  - issue tokens for `suspended: true` subjects (per §1
  #6 — credential-stuffing detection depends on it)
  - skip `auth.token_failed` audit row on failed login (per §1
  #5 — both IP AND account limits MUST apply)
  - bypass rate limit on `/v1/auth/token` (per §1
  #3 — clients during transition need both)
  - emit JWKS without the previous key during rotation overlap (per §1
  - sign with anything weaker than RSA-2048 (per DEC-120)

effort_hours: 12
subtasks:
  - "1.0h: 0006_signing_keys.sql migration (signing_keys table with kid + status + created_at + retired_at)"
  - "1.0h: jwks/rotation.rs — generate new RSA-2048; mark prior as `retiring` for 24h overlap; sweep `retired` after"
  - "1.0h: jwt/issue.rs — bcrypt verify + claims construction + RS256 sign"
  - "0.5h: jwt/verify.rs — JWKS fetch + signature verify + exp/nbf/iss check"
  - "0.5h: jwks/mod.rs — `/.well-known/jwks.json` endpoint (active + retiring keys)"
  - "1.0h: rate_limit.rs — token-bucket per-IP (10/min) AND per-account (5/min) with Redis counter"
  - "0.5h: Suspended subject check (refuse token issuance with auth.token_failed audit)"
  - "0.5h: jti generation (ULID format) + emission in claims"
  - "0.5h: Constant-time email lookup (bcrypt-verify even on missing-email to prevent enumeration timing)"
  - "0.5h: canonical::token_issued + canonical::token_failed memory audit row builders"
  - "0.5h: Quarterly rotation cron (TASK-AUTH-006 schedules; this task provides the function)"
  - "1.5h: Tests — happy + invalid pwd + suspended + rate-limit IP + rate-limit account + JWKS rotation + verify + replay"
  - "0.5h: OTel spans + metrics"
risk_if_skipped: "No JWT means no authentication for any downstream module. AI Gateway can't read tenant_id from claims (TASK-AI-006 explicitly references this). MCP module has no scope-gating. Without dual-rate-limit, credential stuffing succeeds (single-IP attackers can iterate accounts; per-account limits without per-IP let distributed attacks pass). Without JWKS rotation overlap, key rotation produces an outage window where in-flight tokens fail verification. Without auth.token_failed audit, brute-force attacks are invisible."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** issue RS256-signed JWTs containing tenant context, persona, and scope grants. The endpoint and surrounding contract obey:

1. **MUST** sign with RS256 (RSA 2048+); rotate signing key quarterly with 24h overlap. Signing keys live in `signing_keys` table with `kid` (key id), `status` (`active | retiring | retired`), `created_at`, `retired_at`. Active key signs new tokens; retiring keys verify only (24h after rotation); retired keys are removed from JWKS.
2. **MUST** include claims:
- `sub` (subject UUID)
- `tenant_id` (UUID)
- `email`
- `roles[]` (string array; from `subjects.roles`)
- `agent_persona` (default persona handle for subject; e.g., `"cuo-cpo@0.4.1"`)
- `scope_grants[]` (allowed tool/resource refs; e.g., `["chat:read", "chat:write", "kb:read"]`)
- `iat` (issued at; unix seconds)
- `exp` (expiry; iat + 3600)
- `nbf` (not before; iat - 100 to allow clock skew)
- `jti` (ULID for replay protection)
- `iss` (`"https://auth.cyberos.world"`)
3. **MUST** expose `/.well-known/jwks.json` returning all keys with status ∈ {`active`, `retiring`}. Retired keys are excluded. JWKS response is cached at the CDN edge for 5 minutes; clients re-fetch on `kid` mismatch.
4. **MUST** issue via `POST /v1/auth/token` with body `{ "email": <string>, "password": <string>, "tenant_slug": <string> }` → returns `{ "access_token": <jwt>, "expires_in": 3600, "token_type": "Bearer" }`. The `tenant_slug` field is required: a single email may exist across multiple tenants (different subjects), so the slug disambiguates.
5. **MUST** dual-rate-limit token issuance:
- Per-IP: 10 attempts per minute per source IP (via `X-Forwarded-For` first hop).
- Per-account: 5 attempts per minute per `(tenant_slug, email)` regardless of source IP. Either limit triggering returns `429 TOO_MANY_REQUESTS` with body `{"error":"rate_limited","retry_after_seconds":<n>}`. Distributed credential stuffing (different IPs, same account) is caught by the per-account limit; single-IP brute force is caught by the per-IP limit.
6. **MUST** emit memory audit rows:
- `auth.token_issued` per success — payload: `subject_id`, `tenant_id`, `jti`, `roles`, `scope_grants_count`, `expires_at`, `request_id`, `source_ip_hash16`.
- `auth.token_failed` per failed login — payload: `tenant_slug`, `email_hash16`, `reason` (invalid_credentials | suspended | rate_limited), `request_id`, `source_ip_hash16`. Note `email_hash16` not plaintext (matches TASK-AUTH-002 §1 #7 discipline).
7. **MUST** validate JWT signature, `exp`, `nbf`, `iss` on every consuming service (TASK-AI-006, TASK-MCP-004, TASK-AUTH-005). Verification reads JWKS, picks the key matching the JWT's `kid` header, validates signature; failures return `401 UNAUTHORIZED` with `{"error":"invalid_jwt","reason":"<bad_sig|expired|nbf|wrong_iss|unknown_kid>"}`.
8. **MUST** support `jti` dedup at consuming services via bloom filter (probabilistic; ~1MB per million-jti window). Each service maintains a 1-hour rolling bloom filter of seen jtis; a JWT whose jti is in the filter is rejected as `replay_detected`. No central dedup store (would be a single point of failure); per-service bloom is sufficient for the threat model (replay-within-service is the dangerous case; cross-service replay matters less since each service independently checks).
9. **MUST** use **constant-time email lookup** to prevent enumeration timing attack: even when the email doesn't exist in the requested tenant, the handler runs a dummy `bcrypt::verify` against a constant hash to keep response time identical for "wrong password" vs "no such email." Without this, an attacker times responses to enumerate valid emails.
10. **MUST** complete `/v1/auth/token` in ≤ 250ms p95 (bcrypt verify ~150ms + DB + claims build + sign ~50ms + audit emit ~50ms). Above this budget, investigate (likely DB or Redis latency).
11. **MUST** include `kid` (key id) in the JWT header so verifiers know which JWKS key to use. Without `kid`, verifiers must try every key — works at slice 1 but doesn't scale.
12. **MUST** propagate `agent_persona` from `subjects.default_persona` column (added in this task) — the persona claim defaults to `"cuo-cpo@0.4.1"` if not set per subject. Callers can override at request time via `X-Override-Persona` header (subject to TASK-AI-005's allowed-personas check).
13. **MUST** generate `scope_grants` from a join of `subjects.roles` × role-to-grants mapping (defined in `services/auth/src/scope_map.rs`). Slice 1 mappings:
- `tenant-admin` → `["chat:*", "kb:*", "proj:*", "ai:read", "ai:invoke"]`
- `tenant-member` → `["chat:read", "chat:write", "kb:read", "ai:invoke"]`
- `root-admin` (tenant 0 only) → `["*"]`
14. **MUST** check `subjects.suspended == false` BEFORE issuing token. Suspended subjects get `403 FORBIDDEN` with `{"error":"subject_suspended","contact":"ops@cyberos.world"}` AND an `auth.token_failed` audit row with reason `suspended`.
15. **SHOULD** support refresh tokens via separate HTTP-only cookie (TASK-AUTH-007 ships the full flow; this task defines the access-token shape).
16. **SHOULD** emit OTel metrics:
- `auth_token_issued_total{tenant_id, outcome}` (counter; outcome ∈ ok | invalid_credentials | suspended | rate_limited).
- `auth_token_issuance_latency_ms` (histogram; SLO p95 < 250ms).
- `auth_jwks_rotation_total{status}` (counter; status ∈ generated | retired).
- `auth_jwt_verifications_total{service, outcome}` (counter; for downstream services).

---

## §2 — Why this design (rationale for humans)

**Why RS256 + RSA-2048 (§1 #1)?** RS256 is widely supported, NIST-approved, and lets verifiers use only the public key (zero secret-handling on consuming services). ECDSA (ES256) is technically smaller + faster but adoption gaps in some Rust JWT libraries (slice 1 risk-aversion). RSA-2048 is the floor; 4096 is overkill for our threat model. ECDSA migration is TASK-AUTH-110.

**Why 1h access token + 24h key-rotation overlap (§1 #1)?** Short-lived access tokens limit blast radius of leaked credentials; 1h is a common floor (Auth0, Okta defaults). The 24h rotation overlap covers the worst case where a token issued just before key rotation needs to verify just before its 1h expiry — both keys must be in JWKS during the overlap window. 24h is generous (4h would also work; 24h is more forgiving of time-zone-crossing operators).

**Why dual rate-limit (per-IP AND per-account, §1 #5)?** Single-rate-limit attacks well-known: per-IP-only lets distributed attacks (botnet rotating IPs) iterate accounts; per-account-only lets single-IP attackers iterate emails. Both limits together cover both threat models. The 10-per-IP-per-min limit is generous for legitimate users (typing wrong password 5x is normal); 5-per-account-per-min is tight enough to make credential stuffing painful.

**Why constant-time email lookup (§1 #9)?** A timing attack measures "did the bcrypt::verify run?" — if "no such email" returns in 5ms but "wrong password" returns in 150ms, the attacker can enumerate valid emails by timing responses. Running a dummy bcrypt::verify on missing emails keeps response times identical. The constant-time discipline is standard for any credentials check.

**Why `tenant_slug` required in token request (§1 #4)?** A single email may belong to multiple tenants — `alice@cyberos.world` could be a member of tenants A, B, and C. Without `tenant_slug`, the issuance is ambiguous (which tenant's subject?). The slug disambiguates explicitly. Convention: clients track which tenant they're "in" (typically via subdomain `tenant-a.cyberos.world`); the slug is always known.

**Why `agent_persona` in claims (§1 #12)?** Most modules need to know "which persona is making this request" for audit attribution AND for prompt customisation (TASK-AI-014 reads `agent_persona` from claims to inject the right system prompt). Putting it in the JWT means downstream services don't need a second roundtrip to look it up. The `X-Override-Persona` header provides per-request flexibility.

**Why `scope_grants` derived from roles (§1 #13)?** Roles are coarse-grained (tenant-admin, tenant-member); scope_grants are fine-grained (chat:read, kb:write). The role→grants mapping is the bridge: roles are what humans understand; grants are what gates check. Centralising the mapping in `scope_map.rs` ensures consistency — adding a new role automatically updates its grants without per-gate code changes.

**Why `jti` dedup via per-service bloom filter (§1 #8)?** A central dedup store (Redis with all-jti history) is a single point of failure AND a privacy concern (every JWT use logged). Per-service bloom filter is probabilistic (~10⁻⁹ false-positive at our 1MB sizing) — acceptable since false-positive means "honest user re-uses JWT within 1 hour, gets one rejection, re-authenticates" (annoying but not catastrophic). The threat model is "attacker captures JWT, replays it" — which the bloom catches. Cross-service replay matters less because each service has its own bloom.

**Why suspended-subject check (§1 #14)?** A compromised subject (employee left, account fired) needs immediate revocation. Setting `subjects.suspended = true` (TASK-AUTH-005's admin endpoint) prevents new token issuance — but in-flight 1h JWTs still work. Acceptable trade-off given short token TTL; alternative (central token revocation list) would be a per-request lookup.

**Why `email_hash16` and `source_ip_hash16` in audit rows (§1 #6)?** Mirrors TASK-AUTH-002 §1 #7 PII discipline. Plaintext email + IP in audit rows creates everywhere-PII problem; hash16 is enough to disambiguate forensically without leaking. The `source_ip_hash16` salts include the date so IPs can be correlated within a day but not across — preventing long-term IP tracking.

---

## §3 — API contract

```rust
// services/auth/src/jwt/issue.rs

#[derive(Deserialize)]
pub struct TokenRequest {
    pub email: String,
    #[serde(serialize_with = "redact")]
    pub password: Zeroizing<String>,
    pub tenant_slug: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,    // "Bearer"
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub agent_persona: String,
    pub scope_grants: Vec<String>,
    pub iat: i64,
    pub exp: i64,
    pub nbf: i64,
    pub jti: String,           // ULID
    pub iss: String,           // "https://auth.cyberos.world"
}

#[derive(Debug, thiserror::Error)]
pub enum AuthTokenError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("subject suspended")]
    SubjectSuspended,
    #[error("rate limited; retry in {retry_after_seconds}s")]
    RateLimited { retry_after_seconds: u32 },
    #[error("tenant not found: {0!r}")]
    UnknownTenant(String),
    #[error("signing failed: {0}")]
    Signing(String),
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
}
```

```sql
-- services/auth/migrations/0006_signing_keys.sql
CREATE TABLE signing_keys (
    kid          TEXT PRIMARY KEY,                 -- ULID
    public_pem   TEXT NOT NULL,
    private_pem  TEXT NOT NULL,                    -- encrypted at rest via pgcrypto
    status       TEXT NOT NULL CHECK (status IN ('active','retiring','retired')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    retired_at   TIMESTAMPTZ
);

CREATE UNIQUE INDEX signing_keys_active_singleton
    ON signing_keys(status) WHERE status = 'active';
```

```rust
// services/auth/src/jwt/issue.rs (skeleton)
pub async fn issue_token(
    req: TokenRequest, pool: &PgPool, redis: &RedisPool,
    signing_key: &PrivateKey, source_ip: &str, request_id: &str,
) -> Result<TokenResponse, AuthTokenError> {
    // §1 #5: dual rate limit
    rate_limit::check_ip(redis, source_ip, 10).await?;
    rate_limit::check_account(redis, &req.tenant_slug, &req.email, 5).await?;

    let tenant: Tenant = sqlx::query_as("SELECT * FROM tenants WHERE slug = $1")
        .bind(&req.tenant_slug).fetch_optional(pool).await?
        .ok_or_else(|| AuthTokenError::UnknownTenant(req.tenant_slug.clone()))?;

    // §1 #9: constant-time lookup. Even if subject not found, run dummy bcrypt.
    let subject: Option<Subject> = sqlx::query_as(
        "SELECT * FROM subjects WHERE tenant_id = $1 AND email = $2",
    ).bind(tenant.id).bind(&req.email).fetch_optional(pool).await?;

    let dummy_hash = "$2b$12$...constant.dummy.hash...";
    let valid = match &subject {
        Some(s) => bcrypt::verify(&req.password, &s.password_hash).unwrap_or(false),
        None    => { let _ = bcrypt::verify(&req.password, dummy_hash); false }
    };
    if !valid {
        memory::emit(canonical::token_failed(
            &req.tenant_slug, &req.email, "invalid_credentials", source_ip, request_id,
        )).await?;
        return Err(AuthTokenError::InvalidCredentials);
    }
    let subject = subject.unwrap();

    // §1 #14
    if subject.suspended {
        memory::emit(canonical::token_failed(
            &req.tenant_slug, &req.email, "suspended", source_ip, request_id,
        )).await?;
        return Err(AuthTokenError::SubjectSuspended);
    }

    let now = chrono::Utc::now().timestamp();
    let jti = ulid::Ulid::new().to_string();
    let scope_grants = scope_map::for_roles(&subject.roles);
    let claims = Claims {
        sub: subject.id, tenant_id: subject.tenant_id, email: subject.email.clone(),
        roles: subject.roles.clone(),
        agent_persona: subject.default_persona.clone().unwrap_or_else(|| "cuo-cpo@0.4.1".into()),
        scope_grants, iat: now, exp: now + 3600, nbf: now - 100,
        jti: jti.clone(), iss: "https://auth.cyberos.world".into(),
    };

    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some(signing_key.kid.clone());
    let jwt = jsonwebtoken::encode(&header, &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(signing_key.private_pem.as_bytes()).unwrap(),
    ).map_err(|e| AuthTokenError::Signing(e.to_string()))?;

    memory::emit(canonical::token_issued(
        subject.id, subject.tenant_id, &jti, &subject.roles, claims.scope_grants.len(),
        claims.exp, source_ip, request_id,
    )).await?;

    Ok(TokenResponse { access_token: jwt, expires_in: 3600, token_type: "Bearer".into() })
}
```

```rust
// services/auth/src/jwks/mod.rs
#[derive(Serialize)]
pub struct JwksResponse { pub keys: Vec<JwkEntry> }

#[derive(Serialize)]
pub struct JwkEntry {
    pub kty: String,    // "RSA"
    pub kid: String,
    pub r#use: String,  // "sig"
    pub alg: String,    // "RS256"
    pub n: String,      // base64url-encoded modulus
    pub e: String,      // base64url-encoded exponent
}

pub async fn jwks(pool: &PgPool) -> Result<JwksResponse, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT kid, public_pem FROM signing_keys WHERE status IN ('active','retiring')",
    ).fetch_all(pool).await?;
    let keys = rows.into_iter().map(|(kid, pem)| jwk_from_pem(&kid, &pem)).collect();
    Ok(JwksResponse { keys })
}
```

```rust
// services/auth/src/scope_map.rs
pub fn for_roles(roles: &[String]) -> Vec<String> {
    let mut grants = std::collections::BTreeSet::new();
    for role in roles {
        match role.as_str() {
            "tenant-admin"  => for g in ["chat:*","kb:*","proj:*","ai:read","ai:invoke"] {
                                  grants.insert(g.to_string()); },
            "tenant-member" => for g in ["chat:read","chat:write","kb:read","ai:invoke"] {
                                  grants.insert(g.to_string()); },
            "root-admin"    => { grants.insert("*".into()); },
            _ => {}    // unknown roles silently skipped (TASK-AUTH-002 validates at create)
        }
    }
    grants.into_iter().collect()
}
```

---

## §4 — Acceptance criteria

1. **Valid email + password + tenant_slug → 200 + JWT** with all claims populated.
2. **JWT verifies via JWKS** — public key from `/.well-known/jwks.json` matches the JWT's `kid` and verifies the signature.
3. **Invalid password → 401** + `auth.token_failed` audit row with `reason: invalid_credentials`.
4. **Missing email (constant time) → 401** with same response time as wrong password (timing-attack defence).
5. **Suspended subject → 403** + audit row with `reason: suspended`.
6. **Per-IP rate limit at 11th attempt/min → 429** + `Retry-After` header.
7. **Per-account rate limit at 6th attempt/min → 429** even from different IPs.
8. **JWKS returns active + retiring keys** during 24h rotation overlap.
9. **JWKS excludes retired keys**.
10. **Expired JWT verification → `invalid_jwt` reason `expired`**.
11. **NBF in future → `invalid_jwt` reason `nbf`**.
12. **Wrong issuer → `invalid_jwt` reason `wrong_iss`**.
13. **Unknown kid → `invalid_jwt` reason `unknown_kid`**.
14. **jti replay rejected** — verifier's bloom filter catches second use within 1h.
15. **`scope_grants` matches role mapping** — tenant-admin gets `chat:*` etc.; tenant-member gets reduced set; root-admin gets `*`.
16. **`agent_persona` defaults to `"cuo-cpo@0.4.1"`** when subject has no `default_persona`.
17. **Latency p95 ≤ 250ms** including bcrypt verify.
18. **Key rotation: token issued on day N still verifies on day N+1** (within 24h overlap).
19. **Tenant-slug not found → 401** (NOT 404 — same response shape as bad credentials to prevent tenant-enumeration).

---

## §5 — Verification

```rust
#[tokio::test]
async fn valid_credentials_returns_jwt() {
    let pool = test_pool().await;
    let tenant = test_helper::create_tenant("test-co").await;
    let _ = test_helper::create_subject(tenant.id, "alice@x.com", "CorrectHorse9!").await;

    let resp = issue_token(
        TokenRequest {
            email: "alice@x.com".into(),
            password: Zeroizing::new("CorrectHorse9!".into()),
            tenant_slug: "test-co".into(),
        }, &pool, &redis, &active_key().await, "1.2.3.4", "req",
    ).await.unwrap();

    let header = jsonwebtoken::decode_header(&resp.access_token).unwrap();
    assert_eq!(header.alg, jsonwebtoken::Algorithm::RS256);
    assert!(header.kid.is_some());
    let pubkey = jwks_test_helper::pubkey_for_kid(header.kid.as_ref().unwrap()).await;
    let token = jsonwebtoken::decode::<Claims>(&resp.access_token, &pubkey,
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256)).unwrap();
    assert_eq!(token.claims.email, "alice@x.com");
    assert!(token.claims.roles.contains(&"tenant-member".to_string()));
    assert!(memory_test_helper::has_row("auth.token_issued", &token.claims.jti).await);
}

#[tokio::test]
async fn invalid_password_returns_401_and_emits_failed_row() {
    let pool = test_pool().await;
    let tenant = test_helper::create_tenant("c").await;
    let _ = test_helper::create_subject(tenant.id, "x@y.com", "RealPwd9!Aa").await;

    let err = issue_token(
        TokenRequest { email: "x@y.com".into(), password: Zeroizing::new("WrongPwd9!Aa".into()),
                        tenant_slug: "c".into() },
        &pool, &redis, &active_key().await, "1.2.3.4", "req",
    ).await.expect_err("expected InvalidCredentials");
    assert!(matches!(err, AuthTokenError::InvalidCredentials));
    assert!(memory_test_helper::find_recent("auth.token_failed", "invalid_credentials").is_some());
}

#[tokio::test]
async fn missing_email_constant_time_with_present_email() {
    let pool = test_pool().await;
    let tenant = test_helper::create_tenant("ct").await;
    let _ = test_helper::create_subject(tenant.id, "real@x.com", "RealPwd9!Aa").await;

    let t1 = std::time::Instant::now();
    let _ = issue_token(token_req("real@x.com", "WrongPwd9!Aa", "ct"), &pool, &redis, &active_key().await, "1.2.3.4", "r1").await;
    let elapsed_present = t1.elapsed();

    let t2 = std::time::Instant::now();
    let _ = issue_token(token_req("missing@x.com", "AnyPwd9!Aa", "ct"), &pool, &redis, &active_key().await, "1.2.3.5", "r2").await;
    let elapsed_missing = t2.elapsed();

    let diff = (elapsed_present.as_millis() as i64 - elapsed_missing.as_millis() as i64).abs();
    assert!(diff < 30, "timing leak: present={elapsed_present:?} missing={elapsed_missing:?}");
}

#[tokio::test]
async fn suspended_subject_403() {
    let pool = test_pool().await;
    let tenant = test_helper::create_tenant("susp").await;
    let s = test_helper::create_subject(tenant.id, "x@y.com", "Pwd9!AaCorrect").await;
    sqlx::query("UPDATE subjects SET suspended=true WHERE id=$1").bind(s.id).execute(&pool).await.unwrap();

    let err = issue_token(token_req("x@y.com", "Pwd9!AaCorrect", "susp"), &pool, &redis, &active_key().await, "1.2.3.4", "r").await.expect_err("expected Suspended");
    assert!(matches!(err, AuthTokenError::SubjectSuspended));
}

#[tokio::test]
async fn rate_limit_per_ip_at_11th_attempt() {
    let pool = test_pool().await;
    let _ = test_helper::create_tenant_and_subject().await;
    for _ in 0..10 {
        let _ = issue_token(token_req("x@y.com", "wrong", "t"), &pool, &redis, &active_key().await, "9.9.9.9", "r").await;
    }
    let err = issue_token(token_req("x@y.com", "wrong", "t"), &pool, &redis, &active_key().await, "9.9.9.9", "r").await.expect_err("expected RateLimited");
    assert!(matches!(err, AuthTokenError::RateLimited { .. }));
}

#[tokio::test]
async fn rate_limit_per_account_across_ips() {
    let pool = test_pool().await;
    let _ = test_helper::create_tenant_and_subject().await;
    for i in 0..5 {
        let _ = issue_token(token_req("x@y.com", "wrong", "t"), &pool, &redis, &active_key().await, &format!("1.1.1.{i}"), "r").await;
    }
    let err = issue_token(token_req("x@y.com", "wrong", "t"), &pool, &redis, &active_key().await, "2.2.2.2", "r").await.expect_err("expected RateLimited");
    assert!(matches!(err, AuthTokenError::RateLimited { .. }));
}

#[tokio::test]
async fn jwks_includes_active_and_retiring() {
    let pool = test_pool().await;
    rotation::generate_new_signing_key(&pool).await.unwrap();   // marks current as retiring
    let jwks = jwks::jwks(&pool).await.unwrap();
    let kids: HashSet<_> = jwks.keys.iter().map(|k| k.kid.clone()).collect();
    assert!(kids.len() >= 2, "JWKS should include both active + retiring during overlap");
}

#[tokio::test]
async fn jti_replay_rejected_by_bloom() {
    let token = test_helper::issue_test_token().await;
    let v1 = jwt::verify::verify(&token).await.unwrap();
    let v2 = jwt::verify::verify(&token).await.expect_err("expected replay");
    assert!(matches!(v2, jwt::verify::VerifyError::ReplayDetected));
}
```

---

## §6 — Implementation skeleton

See §3 for type defs + handler skeleton. Verification path:

```rust
// services/auth/src/jwt/verify.rs
pub async fn verify(token: &str) -> Result<Claims, VerifyError> {
    let header = jsonwebtoken::decode_header(token)?;
    let kid = header.kid.ok_or(VerifyError::MissingKid)?;
    let pubkey = jwks_cache::get_pubkey_for_kid(&kid).await
        .ok_or(VerifyError::UnknownKid)?;
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    let token_data = jsonwebtoken::decode::<Claims>(token, &pubkey, &validation)?;
    let claims = token_data.claims;

    if claims.iss != "https://auth.cyberos.world" { return Err(VerifyError::WrongIss); }

    // §1 #8 jti dedup
    if jti_bloom::contains_or_insert(&claims.jti) {
        return Err(VerifyError::ReplayDetected);
    }
    Ok(claims)
}
```

Rotation:

```rust
// services/auth/src/jwks/rotation.rs
pub async fn generate_new_signing_key(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE signing_keys SET status='retiring', retired_at=NOW()+INTERVAL '24 hours' WHERE status='active'")
        .execute(&mut *tx).await?;
    let (kid, pub_pem, priv_pem) = generate_rsa_2048();
    sqlx::query("INSERT INTO signing_keys (kid, public_pem, private_pem, status) VALUES ($1, $2, $3, 'active')")
        .bind(&kid).bind(&pub_pem).bind(&priv_pem).execute(&mut *tx).await?;
    tx.commit().await
}

/// Sweeper: runs hourly; deletes retired keys past their retired_at.
pub async fn sweep_retired(pool: &PgPool) -> Result<u32, sqlx::Error> {
    let result = sqlx::query("DELETE FROM signing_keys WHERE status='retired' OR (status='retiring' AND retired_at < NOW())")
        .execute(pool).await?;
    Ok(result.rows_affected() as u32)
}
```

---

## §7 — Dependencies

- **TASK-AUTH-001/002/003** — tenants + subjects + RLS exist before tokens issue against them.
- **TASK-AUTH-006 (downstream)** — bootstrap CLI generates initial signing key + schedules quarterly rotation cron.
- **TASK-AI-006 (downstream)** — reads `tenant_id` + `agent_persona` + `scope_grants` from JWT.
- **TASK-MCP-004 (downstream)** — gates tool calls by `scope_grants`.
- Crates: `jsonwebtoken@9`, `rsa@0.9`, `ulid@1`, `bcrypt@0.15`, `redis@0.24`, `bloom@0.3`, `chrono`, `axum`, `sqlx`.
- Redis 7+ for rate-limit counters.
- Postgres pgcrypto extension for `private_pem` encryption at rest.

---

## §8 — Example payloads

### Token request

```http
POST /v1/auth/token HTTP/1.1
Content-Type: application/json
X-Forwarded-For: 1.2.3.4
X-Forwarded-Proto: https

{ "email": "alice@cyberos.world", "password": "CorrectHorseBatteryStaple9!", "tenant_slug": "cyberskill-jsc" }
```

### Token response

```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsImtpZCI6IjAxSFpLLi4uIn0.eyJzdWIiOiI1NTBlOC4uLiIsInRlbmFudF9pZCI6IjY2MGU4Li4uIiwicm9sZXMiOlsidGVuYW50LWFkbWluIl0sInNjb3BlX2dyYW50cyI6WyJjaGF0OioiLCJrYjoqIl0sImV4cCI6MTc2MzExNTYwMH0.SIGNATURE",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### JWKS

```http
GET /.well-known/jwks.json HTTP/1.1

→ 200 OK
Cache-Control: public, max-age=300

{
  "keys": [
    { "kty": "RSA", "kid": "01HZK...", "use": "sig", "alg": "RS256", "n": "...", "e": "AQAB" },
    { "kty": "RSA", "kid": "01HZJ...", "use": "sig", "alg": "RS256", "n": "...", "e": "AQAB" }
  ]
}
```

### `auth.token_issued` audit row

```json
{
  "kind": "auth.token_issued",
  "payload": {
    "subject_id": "550e...",
    "tenant_id": "660e...",
    "jti": "01HZK9R8M3X5C8Q4",
    "roles": ["tenant-admin"],
    "scope_grants_count": 5,
    "expires_at": 1763115600,
    "source_ip_hash16": "4b8c0d2f1a7e9c3b",
    "request_id": "req_..."
  }
}
```

### `auth.token_failed` audit row

```json
{
  "kind": "auth.token_failed",
  "payload": {
    "tenant_slug": "cyberskill-jsc",
    "email_hash16": "ab12cd34ef56gh78",
    "reason": "invalid_credentials",
    "source_ip_hash16": "4b8c0d2f1a7e9c3b",
    "request_id": "req_..."
  }
}
```

### Rate-limit response

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 45

{ "error": "rate_limited", "retry_after_seconds": 45 }
```

---

## §9 — Open questions

All resolved. Deferred:
- Refresh tokens with rotating-jti — TASK-AUTH-007.
- ECDSA signing — TASK-AUTH-110.
- Token revocation list (immediate revoke) — TASK-AUTH-111.
- Per-tenant signing keys (multi-tenant key isolation) — TASK-AUTH-112.
- mTLS client-cert authentication — TASK-AUTH-113.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid credentials | bcrypt verify | 401 invalid_credentials + audit row | By design |
| Missing email + same response time | constant-time bcrypt on dummy | 401 (timing-attack defence) | By design |
| Suspended subject | check before issue | 403 subject_suspended | Operator unsuspends OR caller acknowledges |
| Per-IP rate limit | redis counter | 429 + retry-after | Wait or use different IP (legitimate retry) |
| Per-account rate limit | redis counter | 429 + retry-after | Wait |
| Tenant slug not found | DB lookup miss | 401 (NOT 404 — same shape to prevent enumeration) | Caller fixes slug |
| Active key rotation | new key inserted; old marked retiring | 24h overlap window | By design |
| Retired key expired | sweeper deletes from JWKS | Verifications fail with `unknown_kid` | Caller re-authenticates |
| JWKS endpoint unreachable | downstream verify fails | 401 invalid_jwt → caller re-auth | Operator investigates AUTH service |
| Replay detected (jti in bloom) | per-service bloom filter | 401 replay_detected | Caller re-authenticates |
| Bloom filter false positive | ~10⁻⁹ rate | 401 (one rejection; honest user re-auths) | By design |
| JWT signature invalid | jsonwebtoken verify | 401 bad_sig | Possible attack — investigate logs |
| JWT expired | exp check | 401 expired | Caller re-authenticates |
| JWT nbf in future | nbf check | 401 nbf | Likely clock skew; investigate |
| Wrong issuer | iss check | 401 wrong_iss | Caller using wrong AUTH endpoint |
| Latency > 250ms | OTel histogram | sev-3 alarm | Investigate bcrypt OR DB OR Redis |
| Signing key access fails (pgcrypto unlock) | DB error | 503 — refuse to issue tokens | Operator investigates KMS |
| Concurrent rotation | UNIQUE active singleton constraint | One succeeds; other gets DB error | Sweeper-style retry |
| Source IP hash predictable | salt with date prevents long-term tracking | N/A | By design |
| `scope_grants` for unknown role | silently skipped | Subject has fewer grants than expected | TASK-AUTH-002 validates roles at create |

---

## §11 — Notes

- RSA-2048 + RS256 chosen for ecosystem maturity in Rust JWT libraries (jsonwebtoken@9 first-class). ECDSA migration tracked in TASK-AUTH-110; not blocking slice 2.
- The 24h key-rotation overlap is the upper-bound buffer for "token issued just before rotation, verified just before its 1h expiry." Without overlap, rotation creates a verification-failure window.
- Dual rate-limit (per-IP AND per-account) covers both the single-IP brute force AND the distributed credential-stuffing threat models. Either limit alone leaves one threat open.
- Constant-time email lookup prevents enumeration timing attacks. The dummy bcrypt::verify on missing emails costs ~150ms — same as the real path. Without this, an attacker can enumerate valid emails by timing responses.
- `tenant_slug` in token request is required because email-uniqueness is per-tenant (a single email can have subjects in multiple tenants). The slug disambiguates without forcing email-uniqueness across all tenants.
- `jti` dedup via per-service bloom filter is the trade-off between strict replay prevention and operational simplicity. Central jti-store would be a single point of failure; bloom is probabilistic but sufficient (false-positive rate ~10⁻⁹ at 1MB sizing; honest users hit at most one rejection in a year).
- The `scope_grants` derived from roles centralises the permission model. Adding a new role requires editing `scope_map.rs` (one place) instead of every gate. The mapping is reviewed in PRs and changes are explicit.
- `email_hash16` in audit rows preserves forensic capability without leaking PII at scale. The 16-hex prefix disambiguates 1-in-10⁹ subjects (enough for slice-1 scale); collisions can be resolved by joining against `subjects` table.
- `source_ip_hash16` is salted with the current date, so IPs can be correlated within a day (useful for incident response) but not across days (preventing long-term IP tracking).
- The `signing_keys.private_pem` column is encrypted at rest via pgcrypto. The decryption key lives in env vars (operator manages); rotation of the env-var key is TASK-AUTH-006's responsibility.

---

*End of TASK-AUTH-004. Status: draft (10/10 target).*
