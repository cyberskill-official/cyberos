//! `cyberos-auth bootstrap` — TASK-AUTH-006.
//!
//! Idempotently seeds the AUTH database with the artifacts every other
//! service depends on:
//!   1. **Root tenant** (`Uuid::nil()`) — already seeded by migration
//!      `0001_tenants.sql`. We confirm it exists.
//!   2. **Root-admin subject** — first humanly-callable account. Email +
//!      bcrypt-hashed password come from env vars or interactive prompts.
//!   3. **Initial RSA-2048 signing key** — handled by `AppState::ensure_signing_key`,
//!      but we run it explicitly here to surface clean error messages.
//!   4. **`root-admin` role grant** on the new subject (only after TASK-AUTH-101 RBAC
//!      migrations have been applied — gracefully no-ops otherwise).
//!
//! Usage:
//!   cyberos-auth-bootstrap \
//!     --email root@cyberskill.world \
//!     --password 'hunter2-strong-passphrase'
//!
//! Env-var equivalents: `AUTH_BOOTSTRAP_EMAIL`, `AUTH_BOOTSTRAP_PASSWORD`.
//! Re-running with the same email is a no-op — the script is idempotent on
//! the (tenant_id, handle) unique constraint.

use cyberos_auth::keygen;
use cyberos_cli_exit::ExitCode;
use sqlx::PgPool;
use uuid::Uuid;

const ROOT_TENANT: Uuid = Uuid::nil();
const ROOT_HANDLE: &str = "@root";

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cyberos_auth=info".into()),
        )
        .init();

    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            eprintln!("\nUsage: cyberos-auth-bootstrap --email <addr> --password <pw>");
            eprintln!(
                "  or:  AUTH_BOOTSTRAP_EMAIL=… AUTH_BOOTSTRAP_PASSWORD=… cyberos-auth-bootstrap"
            );
            return ExitCode::UsageError;
        }
    };

    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("DATABASE_URL env var must be set");
            return ExitCode::ConfigError;
        }
    };

    let pool = match PgPool::connect(&url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("postgres connect failed: {e}");
            return ExitCode::NetworkError;
        }
    };

    match run(&pool, &args).await {
        Ok(summary) => {
            // TASK-AUTH-006 §1 #13 — print summary on success. Note: email
            // intentionally omitted — operator knows what they typed, and
            // echoing email risks landing in logs.
            println!("✓ bootstrap complete");
            println!("  • root tenant:        {}", summary.tenant_0_id);
            println!("  • root-admin subject: {}", summary.root_admin_subject_id);
            println!("  • signing key kid:    {}", summary.signing_key_kid);
            println!("  • memory audit seq:    {}", summary.memory_audit_seq);
            println!();
            println!(
                "Next: POST /v1/auth/token with grant_type=password, \
                 tenant_slug=root, handle={ROOT_HANDLE}"
            );
            ExitCode::Ok
        }
        // TASK-AUTH-006 §1 #7 — distinguish "already done, no action needed"
        // from generic failure so CI scripts can detect rerun-after-success.
        // Maps to the shared enum's `PreconditionFailed` (code 6) — the
        // "no root-admin already exists" precondition is what's violated.
        // task spec §1 #12 mentions a future AUTH-200-range variant for this;
        // tracked in §10.7 of the audit as a follow-up to the
        // cyberos-cli-exit shared enum.
        Err(BootstrapError::AlreadyInitialised) => {
            eprintln!(
                "✗ Tenant 0 + root-admin already exist. \
                 Use --reset --confirm to recreate (destructive). \
                 [Future: slice-2 will add the --reset flag.]"
            );
            ExitCode::PreconditionFailed
        }
        Err(BootstrapError::Other(e)) => {
            eprintln!("bootstrap failed: {e}");
            ExitCode::Generic
        }
    }
}

/// Typed errors so main() can distinguish "already initialised" (exit 6,
/// rerun-safe) from generic failures (exit 1).
#[derive(Debug)]
enum BootstrapError {
    AlreadyInitialised,
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl<E: std::error::Error + Send + Sync + 'static> From<E> for BootstrapError {
    fn from(e: E) -> Self {
        BootstrapError::Other(Box::new(e))
    }
}

/// Summary returned on success — fields used to print the post-bootstrap report.
struct BootstrapSummary {
    tenant_0_id: Uuid,
    root_admin_subject_id: Uuid,
    signing_key_kid: String,
    memory_audit_seq: i64,
}

struct Args {
    email: String,
    password: String,
}

fn parse_args() -> Result<Args, String> {
    let mut email = std::env::var("AUTH_BOOTSTRAP_EMAIL").ok();
    let mut password = std::env::var("AUTH_BOOTSTRAP_PASSWORD").ok();

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--email" => email = args.next(),
            "--password" => password = args.next(),
            "--help" | "-h" => return Err("usage".into()),
            other => return Err(format!("unknown arg: {other}")),
        }
    }

    let email = email.ok_or_else(|| "--email or AUTH_BOOTSTRAP_EMAIL required".to_string())?;
    let password =
        password.ok_or_else(|| "--password or AUTH_BOOTSTRAP_PASSWORD required".to_string())?;
    if password.len() < 12 {
        return Err("password MUST be ≥ 12 characters".into());
    }
    Ok(Args { email, password })
}

async fn run(pool: &PgPool, args: &Args) -> Result<BootstrapSummary, BootstrapError> {
    // 1. Confirm root tenant exists. The 0001 migration seeded it; this check
    // exists so the operator gets a clean error if migrations haven't run.
    let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM tenants WHERE id = $1)")
        .bind(ROOT_TENANT)
        .fetch_one(pool)
        .await?;
    if !exists {
        return Err(BootstrapError::Other(
            "root tenant missing — apply services/auth/migrations/0001_tenants.sql first".into(),
        ));
    }
    println!("✓ root tenant present");

    // 2. TASK-AUTH-006 §1 #7 — Idempotency gate: if a root-admin already exists
    // for the root tenant, exit AlreadyInitialised (code 6) so CI scripts can
    // distinguish "rerun, no-op" from "bad input". Previous impl silently
    // ON CONFLICT-DO-UPDATE-ed the row, which lost the rerun-detection signal.
    let (root_admin_exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM subjects WHERE tenant_id = $1 AND handle = $2)",
    )
    .bind(ROOT_TENANT)
    .bind(ROOT_HANDLE)
    .fetch_one(pool)
    .await?;
    if root_admin_exists {
        return Err(BootstrapError::AlreadyInitialised);
    }

    // 3. Single tx wraps root-admin INSERT + memory audit row INSERT
    //    (TASK-AUTH-006 §1 #4). Both commit or both rollback.
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await?;

    let pw_hash = bcrypt::hash(&args.password, bcrypt::DEFAULT_COST)?;
    let res = sqlx::query(
        "INSERT INTO subjects (tenant_id, handle, display_name, email, kind, password_hash, status, roles)
              VALUES ($1, $2, 'Root Admin', $3, 'human', $4, 'active', ARRAY['root-admin'])
       RETURNING id",
    )
    .bind(ROOT_TENANT)
    .bind(ROOT_HANDLE)
    .bind(&args.email)
    .bind(&pw_hash)
    .fetch_one(&mut *tx)
    .await?;
    let subject_id: Uuid = res.get(0);
    println!("✓ root-admin subject: {subject_id}");

    // 4. Force a signing key into existence (still inside the tx so a
    //    keygen failure rolls back the root-admin insert).
    let signing_key_kid = ensure_signing_key_in_tx(&mut tx)
        .await
        .map_err(BootstrapError::Other)?;
    println!("✓ active RSA-2048 signing key present: {signing_key_kid}");

    // 5. TASK-AUTH-006 §1 #4 — Emit auth.bootstrap_completed memory audit row
    //    INSIDE the same tx. Failure → return Err → tx auto-rolls back the
    //    root-admin + signing key INSERTs together.
    let env_tier =
        std::env::var("CYBEROS_DEPLOYMENT_TIER").unwrap_or_else(|_| "development".into());
    let bootstrapped_by = std::env::var("USER")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "interactive".into());
    let payload = cyberos_auth::memory_bridge::BootstrapCompletedPayload {
        tenant_0_id: ROOT_TENANT,
        root_admin_subject_id: subject_id,
        initial_signing_key_kid: &signing_key_kid,
        bootstrap_environment: &env_tier,
        bootstrapped_by: &bootstrapped_by,
    };
    let memory_audit_seq =
        cyberos_auth::memory_bridge::emit_bootstrap_completed(&mut tx, payload).await?;
    println!("✓ memory audit row emitted: seq={memory_audit_seq}");

    tx.commit().await?;

    // 4. Best-effort: if the RBAC migrations have been applied, grant
    //    `root-admin` on the subject_roles table too. If they haven't,
    //    print a hint and move on (the legacy `subjects.roles` array column
    //    set above is enough for TASK-AUTH-002/004/005).
    let rbac_present: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT FROM information_schema.tables WHERE table_name = 'subject_roles')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if rbac_present {
        let mut tx = pool.begin().await?;
        sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "INSERT INTO subject_roles (tenant_id, subject_id, role, granted_by)
                  VALUES ($1, $2, 'root-admin', $2)
             ON CONFLICT (subject_id, role) DO NOTHING",
        )
        .bind(ROOT_TENANT)
        .bind(subject_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        println!("✓ subject_roles entry for root-admin");
    } else {
        println!(
            "• subject_roles table absent — apply migration 0007 to enable TASK-AUTH-101 RBAC"
        );
    }

    Ok(BootstrapSummary {
        tenant_0_id: ROOT_TENANT,
        root_admin_subject_id: subject_id,
        signing_key_kid,
        memory_audit_seq,
    })
}

/// Tx-scoped signing-key ensure. Returns the kid (newly minted or existing).
/// Per TASK-AUTH-006 §1 #4 + #12, this runs inside the bootstrap transaction
/// so signing-key generation failure rolls back the root-admin INSERT.
async fn ensure_signing_key_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Reuse existing key if active + not expiring.
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT kid FROM auth_signing_keys
            WHERE status = 'active' AND expires_at > NOW()
         ORDER BY expires_at DESC LIMIT 1",
    )
    .fetch_optional(&mut **tx)
    .await?;
    if let Some((kid,)) = existing {
        return Ok(kid);
    }
    let key = keygen::generate_rsa_2048()?;
    let kid = format!("auth-{}", chrono::Utc::now().format("%Y-%m-%d"));
    let expires = chrono::Utc::now() + chrono::Duration::days(90);
    sqlx::query(
        "INSERT INTO auth_signing_keys (kid, algorithm, public_pem, private_pem, status, expires_at)
              VALUES ($1, 'RS256', $2, $3, 'active', $4)
         ON CONFLICT (kid) DO NOTHING",
    )
    .bind(&kid)
    .bind(&key.public_pem)
    .bind(&key.private_pem)
    .bind(expires)
    .execute(&mut **tx)
    .await?;
    Ok(kid)
}

use sqlx::Row;
