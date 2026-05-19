//! `cyberos-email-cli` — slice-1 user provisioning + DKIM rotation entry point.
//!
//! Per FR-EMAIL-001 §1 #16:
//!   cyberos-email-cli provision \
//!     --tenant-id <uuid> --local-part <name> --display-name <text>
//!
//! And slice-1 DKIM rotation:
//!   cyberos-email-cli rotate-dkim --tenant-id <uuid> [--selector cyberos]
//!
//! The CLI exits per cyberos-cli-exit conventions: 0 success, 2 user-error,
//! 3 internal-error. The provisioning RTT MUST be ≤ 5s per §4 #27 — the
//! slice-1 PEM generator returns immediately; slice-2 RSA-2048 generation
//! is the actual cost and tested in `tests/cli_perf_test.rs`.

use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "cyberos-email-cli",
    version,
    about = "EMAIL slice-1 operator CLI"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Provision a new local-part + generate the per-tenant DKIM key if
    /// not already present.
    Provision {
        #[arg(long)]
        tenant_id: Uuid,
        #[arg(long)]
        local_part: String,
        #[arg(long)]
        display_name: String,
    },
    /// Rotate the active DKIM key for a tenant. Atomic: prior key marked
    /// `rotated`, new key inserted as `active`, memory audit row emitted.
    RotateDkim {
        #[arg(long)]
        tenant_id: Uuid,
        #[arg(long, default_value = "cyberos")]
        selector: String,
    },
    /// Print the resolved residency binding for a tenant — useful when
    /// debugging cross-residency leak suspicions.
    ResolveResidency {
        #[arg(long)]
        tenant_id: Uuid,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let exit_code = match run(cli).await {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {e}");
            // Surface specific error codes to operators per cyberos-cli-exit.
            match &e {
                cyberos_email::EmailError::UnknownResidency(_)
                | cyberos_email::EmailError::NoResidencyForTenant(_)
                | cyberos_email::EmailError::DkimKeyAlreadyExists(..)
                | cyberos_email::EmailError::BodyTooLarge(_)
                | cyberos_email::EmailError::DkimKeyNotFound(..) => 2,
                _ => 3,
            }
        }
    };
    std::process::exit(exit_code);
}

async fn run(cli: Cli) -> Result<(), cyberos_email::EmailError> {
    let db_url = std::env::var("DATABASE_URL").map_err(|_| {
        cyberos_email::EmailError::Other(
            "DATABASE_URL env var required (postgres://… string)".into(),
        )
    })?;
    let pool = sqlx::PgPool::connect(&db_url)
        .await
        .map_err(cyberos_email::EmailError::Db)?;

    match cli.cmd {
        Cmd::Provision {
            tenant_id,
            local_part,
            display_name,
        } => {
            // §1 #16 — slice-1 provisioning: ensure DKIM key exists,
            // surface the local-part for the operator to create in
            // Stalwart's admin UI (real Stalwart admin API wires in
            // slice 2).
            println!("✓ provisioning local-part={local_part} display-name={display_name} for tenant {tenant_id}");
            ensure_dkim_active(&pool, tenant_id).await?;
            // memory audit row email.user_provisioned is constructed by the
            // caller in slice 2 when the JWT subject context is available;
            // for now we surface a placeholder line so operators can spot it.
            println!("  → DKIM active key ensured");
            println!(
                "  → memory audit row email.user_provisioned: pending (wired in FR-EMAIL-002)"
            );
            Ok(())
        }
        Cmd::RotateDkim {
            tenant_id,
            selector,
        } => {
            let enc = cyberos_email::dkim::keystore::MockKmsEncryptor;
            let (old, new) = cyberos_email::dkim::keystore::rotate_key(
                &pool,
                &enc,
                tenant_id,
                &selector,
                &format!("alias/cyberos-email-{selector}"),
            )
            .await?;
            println!("✓ dkim rotated: old={old:?} new={new}");
            Ok(())
        }
        Cmd::ResolveResidency { tenant_id } => {
            let b = cyberos_email::residency::resolve(tenant_id, &pool).await?;
            println!("tenant={tenant_id}");
            println!("  residency={}", b.residency);
            println!("  region={}", b.region);
            println!("  bucket={}", b.bucket);
            println!("  kms_key_id={}", b.kms_key_id);
            Ok(())
        }
    }
}

async fn ensure_dkim_active(
    pool: &sqlx::PgPool,
    tenant_id: Uuid,
) -> Result<(), cyberos_email::EmailError> {
    let active: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM dkim_keys WHERE tenant_id = $1 AND status = 'active' AND dkim_selector = 'cyberos'",
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await?;

    if active.is_none() {
        let enc = cyberos_email::dkim::keystore::MockKmsEncryptor;
        cyberos_email::dkim::keystore::provision_key(
            pool,
            &enc,
            tenant_id,
            "cyberos",
            cyberos_email::KeyAlgorithm::Rsa2048,
            "alias/cyberos-email-cyberos",
        )
        .await?;
    }
    Ok(())
}
