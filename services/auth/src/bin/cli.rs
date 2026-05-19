//! `cyberos-authctl` — unified operations CLI for the AUTH module.
//!
//! Per FR-AUTH-006 §1 #1, #8, #9, #10, #11 — single binary with subcommands
//! for `bootstrap`, `rotate-keys`, `sweepers`. Slice-2 of FR-AUTH-006.
//!
//! Naming note: the spec'd binary name `cyberos-auth` is already taken by
//! the HTTP daemon (`services/auth/src/main.rs`). Following industry
//! convention (`systemctl` / `journalctl` / `kubectl`), this CLI is
//! `cyberos-authctl`. See FR-AUTH-006-bootstrap-cli.audit.md §10 for the
//! divergence note. The previous `cyberos-auth-bootstrap` binary remains
//! as a transitional alias for slice-1 scripts.

use clap::{Parser, Subcommand};
use cyberos_cli_exit::ExitCode;
use sqlx::PgPool;

#[derive(Parser, Debug)]
#[command(
    name = "cyberos-authctl",
    version,
    about = "Operations CLI for the CyberOS AUTH module — bootstrap, rotate keys, sweepers"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// FR-AUTH-006 §1 #1 — Initialise tenant 0 + root-admin + initial signing key.
    Bootstrap {
        /// Root-admin email. Defaults to `AUTH_BOOTSTRAP_EMAIL` env var.
        #[arg(long)]
        email: Option<String>,
        /// Root-admin password. Defaults to `AUTH_BOOTSTRAP_PASSWORD` env var.
        #[arg(long)]
        password: Option<String>,
        /// FR-AUTH-006 §1 #10 — Destructive re-bootstrap: drops tenant 0 + cascades.
        /// Requires `--confirm` alongside. In production also requires `--force-prod-reset`.
        #[arg(long, requires = "confirm")]
        reset: bool,
        /// Second-flag confirmation for `--reset`.
        #[arg(long)]
        confirm: bool,
        /// FR-AUTH-006 §1 #11 — Production-environment override. Without this,
        /// `--reset --confirm` exits with PreconditionFailed when
        /// `CYBEROS_DEPLOYMENT_TIER=production`.
        #[arg(long)]
        force_prod_reset: bool,
    },
    /// FR-AUTH-006 §1 #8 — Rotate the active RSA-2048 signing key. The new
    /// key becomes `status='active'`; the previous active key is marked
    /// `status='retired'` with `retired_at = NOW()`. Useful for emergency
    /// rotation (suspected compromise) — the same path the quarterly cron uses.
    RotateKeys,
    /// FR-AUTH-006 §1 #9 — Sweep stale rows from three tables:
    /// expired sessions, old `admin_idempotency_keys` (>24h), retired
    /// `auth_signing_keys` (>7d after retirement). Reports per-table counts.
    /// Cron runs hourly; manual invocation is for ops investigation.
    Sweepers,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cyberos_auth=info".into()),
        )
        .init();

    let cli = Cli::parse();

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

    match cli.command {
        Command::Bootstrap {
            email,
            password,
            reset,
            confirm,
            force_prod_reset,
        } => bootstrap_cmd(&pool, email, password, reset, confirm, force_prod_reset).await,
        Command::RotateKeys => rotate_keys_cmd(&pool).await,
        Command::Sweepers => sweepers_cmd(&pool).await,
    }
}

// ===========================================================================
// `bootstrap` subcommand — delegates to the slice-1 bootstrap logic.
// ===========================================================================

async fn bootstrap_cmd(
    pool: &PgPool,
    email: Option<String>,
    password: Option<String>,
    reset: bool,
    confirm: bool,
    force_prod_reset: bool,
) -> ExitCode {
    let email = email.or_else(|| std::env::var("AUTH_BOOTSTRAP_EMAIL").ok());
    let password = password.or_else(|| std::env::var("AUTH_BOOTSTRAP_PASSWORD").ok());

    let (email, password) = match (email, password) {
        (Some(e), Some(p)) if p.len() >= 12 => (e, p),
        (Some(_), Some(_)) => {
            eprintln!("--password / AUTH_BOOTSTRAP_PASSWORD must be ≥ 12 chars");
            return ExitCode::UsageError;
        }
        _ => {
            eprintln!(
                "--email + --password required (or AUTH_BOOTSTRAP_EMAIL + AUTH_BOOTSTRAP_PASSWORD env)"
            );
            return ExitCode::UsageError;
        }
    };

    // FR-AUTH-006 §1 #10 + #11 — Destructive --reset path.
    if reset {
        if !confirm {
            // clap's `requires = "confirm"` should prevent this, but defence-in-depth.
            eprintln!("--reset requires --confirm");
            return ExitCode::PreconditionFailed;
        }
        let deployment_tier =
            std::env::var("CYBEROS_DEPLOYMENT_TIER").unwrap_or_else(|_| "development".into());
        if deployment_tier == "production" && !force_prod_reset {
            eprintln!(
                "✗ Refused: CYBEROS_DEPLOYMENT_TIER=production. Add --force-prod-reset \
                 to acknowledge the blast radius (drops tenant 0 + cascades = wipes \
                 every tenant + subject + signing key + audit row)."
            );
            return ExitCode::PreconditionFailed;
        }
        eprintln!(
            "⚠ Destructive --reset --confirm{} in tier '{deployment_tier}'.",
            if force_prod_reset {
                " --force-prod-reset"
            } else {
                ""
            }
        );
        if let Err(e) = perform_reset(pool).await {
            eprintln!("reset failed: {e}");
            return ExitCode::Generic;
        }
        eprintln!("✓ reset complete; proceeding to fresh bootstrap");
    }

    // Delegate to the slice-1 binary's run() logic via subprocess for now —
    // slice-3 (future) folds the body of bin/bootstrap.rs::run into a shared
    // `cyberos_auth::cli::bootstrap` helper. For slice-2 we keep slice-1 intact.
    // Simplest path: re-invoke the existing binary with the resolved args.
    use std::process::Command as ShellCommand;
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("cyberos-auth-bootstrap")));
    let bootstrap_exe = match exe {
        Some(p) if p.exists() => p,
        _ => {
            eprintln!(
                "✗ Sibling binary `cyberos-auth-bootstrap` not found next to `cyberos-authctl`. \
                 Ensure both are built (`cargo build --bin cyberos-auth-bootstrap --bin cyberos-authctl`)."
            );
            return ExitCode::ConfigError;
        }
    };
    let status = match ShellCommand::new(&bootstrap_exe)
        .arg("--email")
        .arg(&email)
        .arg("--password")
        .arg(&password)
        .env("AUTH_BOOTSTRAP_EMAIL", &email)
        .env("AUTH_BOOTSTRAP_PASSWORD", &password)
        .status()
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to spawn cyberos-auth-bootstrap: {e}");
            return ExitCode::Generic;
        }
    };
    match status.code() {
        Some(0) => ExitCode::Ok,
        Some(6) => ExitCode::PreconditionFailed, // AlreadyInitialised propagated
        Some(_) => ExitCode::Generic,
        None => ExitCode::Generic,
    }
}

async fn perform_reset(pool: &PgPool) -> Result<(), sqlx::Error> {
    // DELETE tenant 0 — every tenant-scoped table cascades via FK or RLS-policy
    // teardown. Each tenant migration is responsible for `ON DELETE CASCADE` on
    // its tenant_id FK (verified by the rls_isolation_test).
    sqlx::query("DELETE FROM tenants WHERE id = '00000000-0000-0000-0000-000000000000'::uuid")
        .execute(pool)
        .await?;
    // Tenant 0 is re-seeded by migration 0001 on next `sqlx migrate run`. For
    // a pure reset (no schema replay), re-insert the root row by hand:
    sqlx::query(
        "INSERT INTO tenants (id, slug, display_name, country, plan_tier, status, residency)
              VALUES ('00000000-0000-0000-0000-000000000000'::uuid, 'root', 'Root Tenant',
                      'XX', 'enterprise', 'active', 'global')
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ===========================================================================
// `rotate-keys` subcommand (FR-AUTH-006 §1 #8)
// ===========================================================================

async fn rotate_keys_cmd(pool: &PgPool) -> ExitCode {
    use cyberos_auth::keygen;

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("postgres begin failed: {e}");
            return ExitCode::NetworkError;
        }
    };

    // 1. Mark current active key as retired.
    let prev_kid: Option<(String,)> = sqlx::query_as(
        "UPDATE auth_signing_keys
            SET status = 'retired', retired_at = NOW()
          WHERE status = 'active'
        RETURNING kid",
    )
    .fetch_optional(&mut *tx)
    .await
    .unwrap_or(None);

    // 2. Generate new key.
    let key = match keygen::generate_rsa_2048() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("keygen failed: {e}");
            return ExitCode::Generic;
        }
    };
    let kid = format!(
        "auth-{}-{}",
        chrono::Utc::now().format("%Y-%m-%d"),
        uuid::Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(6)
            .collect::<String>()
    );
    let expires = chrono::Utc::now() + chrono::Duration::days(90);

    if let Err(e) = sqlx::query(
        "INSERT INTO auth_signing_keys (kid, algorithm, public_pem, private_pem, status, expires_at)
              VALUES ($1, 'RS256', $2, $3, 'active', $4)",
    )
    .bind(&kid)
    .bind(&key.public_pem)
    .bind(&key.private_pem)
    .bind(expires)
    .execute(&mut *tx)
    .await
    {
        eprintln!("insert new key failed: {e}");
        return ExitCode::Generic;
    }

    if let Err(e) = tx.commit().await {
        eprintln!("commit failed: {e}");
        return ExitCode::Generic;
    }

    if let Some((prev,)) = prev_kid {
        println!("✓ rotated: retired={prev}, active={kid}");
    } else {
        println!("✓ first key created: active={kid}");
    }
    println!("  • expires_at: {expires}");
    ExitCode::Ok
}

// ===========================================================================
// `sweepers` subcommand (FR-AUTH-006 §1 #9)
// ===========================================================================

async fn sweepers_cmd(pool: &PgPool) -> ExitCode {
    let mut total_swept = 0i64;

    // 1. Old `admin_idempotency_keys` rows (>24h).
    match sqlx::query(
        "DELETE FROM admin_idempotency_keys WHERE created_at < NOW() - INTERVAL '24 hours'",
    )
    .execute(pool)
    .await
    {
        Ok(r) => {
            let n = r.rows_affected() as i64;
            total_swept += n;
            println!("✓ admin_idempotency_keys: deleted {n} rows (>24h old)");
        }
        Err(e) => {
            eprintln!("sweep admin_idempotency_keys failed: {e}");
            return ExitCode::Generic;
        }
    }

    // 2. Retired `auth_signing_keys` (>7d after retirement).
    match sqlx::query(
        "DELETE FROM auth_signing_keys
          WHERE status = 'retired' AND retired_at < NOW() - INTERVAL '7 days'",
    )
    .execute(pool)
    .await
    {
        Ok(r) => {
            let n = r.rows_affected() as i64;
            total_swept += n;
            println!("✓ auth_signing_keys: deleted {n} rows (retired >7d ago)");
        }
        Err(e) => {
            eprintln!("sweep auth_signing_keys failed: {e}");
            return ExitCode::Generic;
        }
    }

    // 3. Expired `sessions` rows. Note: a `sessions` table is FR-AUTH-004
    // slice-3 work; today the table may not exist. Skip cleanly if absent
    // (information_schema check).
    let sessions_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT FROM information_schema.tables WHERE table_name = 'sessions')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    if sessions_exists {
        match sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(pool)
            .await
        {
            Ok(r) => {
                let n = r.rows_affected() as i64;
                total_swept += n;
                println!("✓ sessions: deleted {n} rows (expired)");
            }
            Err(e) => {
                eprintln!("sweep sessions failed: {e}");
                return ExitCode::Generic;
            }
        }
    } else {
        println!("• sessions table absent — FR-AUTH-004 slice-3 will introduce it");
    }

    println!();
    println!("Total rows swept: {total_swept}");
    ExitCode::Ok
}
