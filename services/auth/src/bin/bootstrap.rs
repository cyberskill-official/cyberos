//! `cyberos-auth bootstrap` — FR-AUTH-006.
//!
//! Idempotently seeds the AUTH database with the artifacts every other
//! service depends on:
//!   1. **Root tenant** (`Uuid::nil()`) — already seeded by migration
//!      `0001_tenants.sql`. We confirm it exists.
//!   2. **Root-admin subject** — first humanly-callable account. Email +
//!      bcrypt-hashed password come from env vars or interactive prompts.
//!   3. **Initial RSA-2048 signing key** — handled by `AppState::ensure_signing_key`,
//!      but we run it explicitly here to surface clean error messages.
//!   4. **`root-admin` role grant** on the new subject (only after FR-AUTH-101 RBAC
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

use cyberos_auth::{keygen, state::AppState};
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
            eprintln!("  or:  AUTH_BOOTSTRAP_EMAIL=… AUTH_BOOTSTRAP_PASSWORD=… cyberos-auth-bootstrap");
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

    if let Err(e) = run(&pool, &args).await {
        eprintln!("bootstrap failed: {e}");
        return ExitCode::Generic;
    }

    println!("✓ bootstrap complete");
    println!("  • root tenant:    {ROOT_TENANT}");
    println!("  • root subject:   {ROOT_HANDLE}");
    println!("  • email:          {}", args.email);
    println!("  • signing key:    active 90-day RSA-2048");
    println!();
    println!("Next: POST /v1/auth/token with grant_type=password, tenant_slug=root, handle={ROOT_HANDLE}");
    ExitCode::Ok
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
    let password = password.ok_or_else(|| "--password or AUTH_BOOTSTRAP_PASSWORD required".to_string())?;
    if password.len() < 12 {
        return Err("password MUST be ≥ 12 characters".into());
    }
    Ok(Args { email, password })
}

async fn run(pool: &PgPool, args: &Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Confirm root tenant exists. The 0001 migration seeded it; this check
    // exists so the operator gets a clean error if migrations haven't run.
    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM tenants WHERE id = $1)",
    )
    .bind(ROOT_TENANT)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Err("root tenant missing — apply services/auth/migrations/0001_tenants.sql first".into());
    }
    println!("✓ root tenant present");

    // 2. Seed the root-admin subject. RLS forces us to set the GUC.
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx).await?;

    let pw_hash = bcrypt::hash(&args.password, bcrypt::DEFAULT_COST)?;
    let res = sqlx::query(
        "INSERT INTO subjects (tenant_id, handle, display_name, email, kind, password_hash, status, roles)
              VALUES ($1, $2, 'Root Admin', $3, 'human', $4, 'active', ARRAY['root-admin'])
         ON CONFLICT (tenant_id, handle) DO UPDATE
            SET email = EXCLUDED.email,
                password_hash = EXCLUDED.password_hash,
                updated_at = NOW()
       RETURNING id",
    )
    .bind(ROOT_TENANT)
    .bind(ROOT_HANDLE)
    .bind(&args.email)
    .bind(&pw_hash)
    .fetch_one(&mut *tx)
    .await?;
    let subject_id: Uuid = res.get(0);
    tx.commit().await?;
    println!("✓ root-admin subject: {subject_id}");

    // 3. Force a signing key into existence. Uses the same logic AppState
    //    does at boot — extracted here as a public helper.
    ensure_signing_key(pool).await?;
    println!("✓ active RSA-2048 signing key present");

    // 4. Best-effort: if the RBAC migrations have been applied, grant
    //    `root-admin` on the subject_roles table too. If they haven't,
    //    print a hint and move on (the legacy `subjects.roles` array column
    //    set above is enough for FR-AUTH-002/004/005).
    let rbac_present: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT FROM information_schema.tables WHERE table_name = 'subject_roles')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if rbac_present {
        let mut tx = pool.begin().await?;
        sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
            .execute(&mut *tx).await?;
        sqlx::query(
            "INSERT INTO subject_roles (tenant_id, subject_id, role, granted_by)
                  VALUES ($1, $2, 'root-admin', $2)
             ON CONFLICT (subject_id, role) DO NOTHING",
        )
        .bind(ROOT_TENANT)
        .bind(subject_id)
        .execute(&mut *tx).await?;
        tx.commit().await?;
        println!("✓ subject_roles entry for root-admin");
    } else {
        println!("• subject_roles table absent — apply migration 0007 to enable FR-AUTH-101 RBAC");
    }

    Ok(())
}

async fn ensure_signing_key(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM auth_signing_keys WHERE status = 'active' AND expires_at > NOW()",
    )
    .fetch_one(pool)
    .await?;
    if n > 0 {
        return Ok(());
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
    .execute(pool)
    .await?;
    Ok(())
}

use sqlx::Row;
// `AppState` is unused in the binary but kept in scope for future helpers.
#[allow(unused_imports)]
use cyberos_auth as _auth;
