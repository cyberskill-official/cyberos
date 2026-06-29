//! FR-MCP-004 OAuth persistence: clients, authorization codes, refresh families, the access-token
//! revocation list, and consent records.
//!
//! Every query is runtime-checked `sqlx::query`/`query_as` (no compile-time database), so this module
//! compiles without a live database; the integration tests exercise it against Postgres. Errors are
//! plain `sqlx::Error` - the endpoint layer maps them to RFC 6749 responses. Lifecycle for an
//! authorization code is derived from `consumed_at` + `expires_at` (the table has no state column);
//! [`consume_code`] does the one-time-use check inside a `SELECT ... FOR UPDATE` transaction.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---- clients (RFC 7591 dynamic registration) ---------------------------------------

/// A registered OAuth client, as the authorize and token endpoints need to read it.
#[derive(Debug, Clone)]
pub struct ClientRow {
    /// Client id (primary key).
    pub id: Uuid,
    /// Owning tenant (NULL for public CLI clients).
    pub tenant_id: Option<Uuid>,
    /// `public` or `confidential`.
    pub client_type: String,
    /// Argon2 hash of the client secret (None for public clients).
    pub client_secret_hash: Option<String>,
    /// The exact-match redirect URIs registered for this client.
    pub redirect_uris: Vec<String>,
    /// Space-separated registered scope set.
    pub scope: String,
    /// Set when the client is revoked; None while active.
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Insert a new client and return its generated id. `client_type` is bound as text and cast to the
/// `client_type` enum. `redirect_uris` is stored as JSONB.
#[allow(clippy::too_many_arguments)]
pub async fn insert_client(
    pool: &PgPool,
    client_type: &str,
    tenant_id: Option<Uuid>,
    client_secret_hash: Option<&str>,
    redirect_uris: &[String],
    client_name: Option<&str>,
    scope: &str,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO oauth_clients
            (tenant_id, client_type, client_secret_hash, redirect_uris, client_name, scope)
         VALUES ($1, $2::client_type, $3, $4::jsonb, $5, $6)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(client_type)
    .bind(client_secret_hash)
    .bind(serde_json::to_string(redirect_uris).unwrap_or_else(|_| "[]".to_string()))
    .bind(client_name)
    .bind(scope)
    .fetch_one(pool)
    .await
}

/// Load a client by id, or None if it does not exist.
pub async fn get_client(pool: &PgPool, id: Uuid) -> Result<Option<ClientRow>, sqlx::Error> {
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            Option<Uuid>,
            String,
            Option<String>,
            String,
            String,
            Option<DateTime<Utc>>,
        ),
    >(
        "SELECT id, tenant_id, client_type::text, client_secret_hash, redirect_uris::text, scope, revoked_at
           FROM oauth_clients WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(id, tenant_id, client_type, client_secret_hash, redirect_uris, scope, revoked_at)| {
            ClientRow {
                id,
                tenant_id,
                client_type,
                client_secret_hash,
                redirect_uris: serde_json::from_str(&redirect_uris).unwrap_or_default(),
                scope,
                revoked_at,
            }
        },
    ))
}

// ---- authorization codes (one-time-use, 30s TTL) -----------------------------------

/// The fields a consumed authorization code yields for the token exchange.
#[derive(Debug, Clone)]
pub struct ConsumedCode {
    /// The client the code was issued to.
    pub client_id: Uuid,
    /// The authorizing subject.
    pub subject_id: Uuid,
    /// The tenant binding.
    pub tenant_id: Uuid,
    /// The redirect URI the code was bound to (must match at exchange).
    pub redirect_uri: String,
    /// The PKCE S256 challenge to verify the verifier against.
    pub code_challenge: String,
    /// The granted scope.
    pub scope: String,
    /// The resource-server audience the access token must be bound to.
    pub audience: String,
}

/// Outcome of attempting to consume an authorization code at the token endpoint.
#[derive(Debug, Clone)]
pub enum CodeConsumption {
    /// First, valid use - proceed with the exchange.
    Consumed(ConsumedCode),
    /// No such code.
    NotFound,
    /// The code's 30-second TTL had elapsed.
    Expired,
    /// The code was already consumed - a replay (clause #15): the caller compromises the client's
    /// refresh family and emits the sev-2 audit. The fields are returned so the caller can locate it.
    Replay(ConsumedCode),
}

/// Insert a freshly issued authorization code. `code_challenge_method` is fixed to `S256`; `state` is
/// the client's CSRF value. `expires_at` should be `now + 30s`.
#[allow(clippy::too_many_arguments)]
pub async fn insert_code(
    pool: &PgPool,
    code: &str,
    client_id: Uuid,
    subject_id: Uuid,
    tenant_id: Uuid,
    redirect_uri: &str,
    code_challenge: &str,
    scope: &str,
    audience: &str,
    nonce: &str,
    state: &str,
    expires_at: DateTime<Utc>,
    memory_chain_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO oauth_codes
            (code, client_id, subject_id, tenant_id, redirect_uri, code_challenge,
             code_challenge_method, scope, audience, nonce, state, expires_at, memory_chain_hash)
         VALUES ($1,$2,$3,$4,$5,$6,'S256',$7,$8,$9,$10,$11,$12)",
    )
    .bind(code)
    .bind(client_id)
    .bind(subject_id)
    .bind(tenant_id)
    .bind(redirect_uri)
    .bind(code_challenge)
    .bind(scope)
    .bind(audience)
    .bind(nonce)
    .bind(state)
    .bind(expires_at)
    .bind(memory_chain_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Consume an authorization code exactly once. Locks the row `FOR UPDATE`, then: missing -> NotFound;
/// already consumed -> Replay; past `expires_at` -> Expired; otherwise marks `consumed_at = now()` and
/// returns Consumed. Atomic, so two concurrent exchanges cannot both succeed.
pub async fn consume_code(pool: &PgPool, code: &str) -> Result<CodeConsumption, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            Uuid,
            String,
            String,
            String,
            String,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
        ),
    >(
        "SELECT client_id, subject_id, tenant_id, redirect_uri, code_challenge, scope, audience,
                expires_at, consumed_at
           FROM oauth_codes WHERE code = $1 FOR UPDATE",
    )
    .bind(code)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((
        client_id,
        subject_id,
        tenant_id,
        redirect_uri,
        code_challenge,
        scope,
        audience,
        expires_at,
        consumed_at,
    )) = row
    else {
        tx.rollback().await?;
        return Ok(CodeConsumption::NotFound);
    };

    let fields = ConsumedCode {
        client_id,
        subject_id,
        tenant_id,
        redirect_uri,
        code_challenge,
        scope,
        audience,
    };

    if consumed_at.is_some() {
        tx.commit().await?;
        return Ok(CodeConsumption::Replay(fields));
    }
    if Utc::now() >= expires_at {
        tx.rollback().await?;
        return Ok(CodeConsumption::Expired);
    }

    sqlx::query("UPDATE oauth_codes SET consumed_at = now() WHERE code = $1")
        .bind(code)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(CodeConsumption::Consumed(fields))
}

// ---- refresh families (rotation + reuse detection) ---------------------------------

/// A refresh token's family row, as the refresh grant needs to read it.
#[derive(Debug, Clone)]
pub struct RefreshRow {
    /// The family this token belongs to (compromise is per-family).
    pub family_id: Uuid,
    /// The client.
    pub client_id: Uuid,
    /// The subject.
    pub subject_id: Uuid,
    /// The tenant binding.
    pub tenant_id: Uuid,
    /// The bound resource-server audience.
    pub audience: String,
    /// The granted scope.
    pub scope: String,
    /// `active` | `used` | `compromised`.
    pub state: String,
    /// Expiry (30 days from issue).
    pub expires_at: DateTime<Utc>,
}

/// Insert a refresh token row. `parent_token_hash` is None for the root of a family and Some for each
/// rotation. `token_hash` is the SHA-256 hex of the opaque token.
#[allow(clippy::too_many_arguments)]
pub async fn insert_refresh(
    pool: &PgPool,
    family_id: Uuid,
    client_id: Uuid,
    subject_id: Uuid,
    tenant_id: Uuid,
    audience: &str,
    scope: &str,
    token_hash: &str,
    parent_token_hash: Option<&str>,
    expires_at: DateTime<Utc>,
    memory_chain_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO oauth_refresh_families
            (family_id, client_id, subject_id, tenant_id, audience, scope, token_hash,
             parent_token_hash, expires_at, state, memory_chain_hash)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'active'::oauth_refresh_state,$10)",
    )
    .bind(family_id)
    .bind(client_id)
    .bind(subject_id)
    .bind(tenant_id)
    .bind(audience)
    .bind(scope)
    .bind(token_hash)
    .bind(parent_token_hash)
    .bind(expires_at)
    .bind(memory_chain_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Look up a refresh token by its SHA-256 hash.
pub async fn get_refresh_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<RefreshRow>, sqlx::Error> {
    let row = sqlx::query_as::<
        _,
        (Uuid, Uuid, Uuid, Uuid, String, String, String, DateTime<Utc>),
    >(
        "SELECT family_id, client_id, subject_id, tenant_id, audience, scope, state::text, expires_at
           FROM oauth_refresh_families WHERE token_hash = $1",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(family_id, client_id, subject_id, tenant_id, audience, scope, state, expires_at)| {
            RefreshRow {
                family_id,
                client_id,
                subject_id,
                tenant_id,
                audience,
                scope,
                state,
                expires_at,
            }
        },
    ))
}

/// Mark a refresh token `used` (it has just been rotated out).
pub async fn mark_refresh_used(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE oauth_refresh_families
            SET state = 'used'::oauth_refresh_state, state_changed_at = now()
          WHERE token_hash = $1",
    )
    .bind(token_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Compromise an entire refresh family (reuse detected): every member becomes `compromised` and no
/// descendant can be exchanged (clause #9, DEC-806).
pub async fn compromise_family(pool: &PgPool, family_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE oauth_refresh_families
            SET state = 'compromised'::oauth_refresh_state, state_changed_at = now()
          WHERE family_id = $1",
    )
    .bind(family_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ---- access-token revocation list (clauses #21, #24) -------------------------------

/// Add an access token's `jti` to the revocation list until its natural expiry.
pub async fn insert_revocation(
    pool: &PgPool,
    jti: Uuid,
    expires_at: DateTime<Utc>,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO oauth_revocation_list (jti, expires_at, reason)
         VALUES ($1, $2, $3)
         ON CONFLICT (jti) DO NOTHING",
    )
    .bind(jti)
    .bind(expires_at)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Whether an access token's `jti` is on the revocation list (checked on every verified request).
pub async fn is_jti_revoked(pool: &PgPool, jti: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM oauth_revocation_list WHERE jti = $1)",
    )
    .bind(jti)
    .fetch_one(pool)
    .await
}

// ---- consents (clause #29) ---------------------------------------------------------

/// The scopes a subject has previously granted a client, or None if no consent exists yet.
pub async fn get_consent(
    pool: &PgPool,
    subject_id: Uuid,
    client_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT scopes FROM oauth_consents WHERE subject_id = $1 AND client_id = $2",
    )
    .bind(subject_id)
    .bind(client_id)
    .fetch_optional(pool)
    .await
}

/// Record (or update) a subject's consent for a client to the given scope set.
pub async fn upsert_consent(
    pool: &PgPool,
    subject_id: Uuid,
    client_id: Uuid,
    scopes: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO oauth_consents (subject_id, client_id, scopes)
         VALUES ($1, $2, $3)
         ON CONFLICT (subject_id, client_id) DO UPDATE
            SET scopes = EXCLUDED.scopes, granted_at = now()",
    )
    .bind(subject_id)
    .bind(client_id)
    .bind(scopes)
    .execute(pool)
    .await?;
    Ok(())
}
