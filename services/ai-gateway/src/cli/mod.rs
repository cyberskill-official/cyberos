//! FR-AI-021 — `cyberos-ai` operator CLI structure.
//!
//! Subcommands: usage, models, policy, failover, invoice, breaker, expiry, memory, completions.

pub mod auth;
pub mod breaker;
pub mod completions;
pub mod exit_codes;
pub mod expiry;
pub mod failover;
pub mod flag_tenant;
pub mod invoice;
pub mod json_schemas;
pub mod memory;
pub mod models;
pub mod output;
pub mod policy;
pub mod usage;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "cyberos-ai",
    version,
    about = "CyberOS AI Gateway operator CLI"
)]
pub struct Cli {
    /// Output in JSON format.
    #[arg(long, global = true)]
    pub json: bool,
    /// Confirm mutating operations.
    #[arg(long, global = true)]
    pub confirm: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// MTD spend, call count, top models.
    Usage(UsageArgs),
    /// Model catalogue and pricing.
    Models(ModelsArgs),
    /// Tenant policy management.
    Policy(PolicyArgs),
    /// Failover drill management.
    Failover(FailoverArgs),
    /// Invoice generation.
    Invoice(InvoiceArgs),
    /// Circuit breaker management.
    Breaker(BreakerArgs),
    /// Hold-expiry job management.
    Expiry(ExpiryArgs),
    /// Memory audit row operations.
    Memory(MemoryArgs),
    /// Flag a tenant for 100% trace sampling (FR-OBS-006).
    FlagTenant(FlagTenantArgs),
    /// Generate shell completions.
    Completions(CompletionsArgs),
}

/// `flag-tenant <tenant_id> --confirm [--remove]` (FR-OBS-006 §1 #3).
#[derive(Debug, clap::Args)]
pub struct FlagTenantArgs {
    /// The tenant id to flag (or unflag with --remove).
    pub tenant_id: String,
    /// Remove the tenant from the flagged list instead of adding it.
    #[arg(long)]
    pub remove: bool,
    /// Confirm the mutation to the sampling config.
    #[arg(long)]
    pub confirm: bool,
}

#[derive(Debug, clap::Args)]
pub struct UsageArgs {
    /// Filter by tenant.
    #[arg(long)]
    pub tenant: Option<String>,
    /// Month to query (YYYY-MM).
    #[arg(long)]
    pub month: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct ModelsArgs {
    #[command(subcommand)]
    pub action: ModelsAction,
}

#[derive(Debug, Subcommand)]
pub enum ModelsAction {
    /// List supported aliases x providers x models.
    List,
    /// Show cost-table rates.
    Pricing,
}

#[derive(Debug, clap::Args)]
pub struct PolicyArgs {
    #[command(subcommand)]
    pub action: PolicyAction,
}

#[derive(Debug, Subcommand)]
pub enum PolicyAction {
    /// Update tenant policy fields.
    Set {
        /// Tenant identifier.
        tenant: String,
        /// Monthly USD cap.
        #[arg(long)]
        cap_usd: Option<f64>,
        /// Require ZDR.
        #[arg(long)]
        zdr_required: Option<bool>,
        /// Residency pin.
        #[arg(long)]
        residency: Option<String>,
        /// Allowed persona IDs.
        #[arg(long, num_args = 1..)]
        allowed_personas: Option<Vec<String>>,
    },
    /// Validate a YAML file without applying.
    Validate {
        /// Path to the YAML file.
        yaml_file: PathBuf,
    },
    /// Compare tenant policy against a YAML file.
    Diff {
        /// Tenant identifier.
        tenant: String,
        /// YAML file to compare against.
        #[arg(long = "vs")]
        vs: PathBuf,
    },
}

#[derive(Debug, clap::Args)]
pub struct FailoverArgs {
    #[command(subcommand)]
    pub action: FailoverAction,
}

#[derive(Debug, Subcommand)]
pub enum FailoverAction {
    /// Force a 5xx storm to test failover.
    Drill {
        /// Target in provider:model format.
        target: String,
        /// Duration in seconds.
        #[arg(long, default_value_t = 60)]
        duration: u32,
        /// Acknowledge production impact.
        #[arg(long)]
        prod_confirmed_aware: bool,
    },
}

#[derive(Debug, clap::Args)]
pub struct InvoiceArgs {
    #[command(subcommand)]
    pub action: InvoiceAction,
}

#[derive(Debug, Subcommand)]
pub enum InvoiceAction {
    /// Generate invoice.
    Export {
        /// Tenant identifier.
        tenant: String,
        /// Billing period (YYYY-MM).
        #[arg(long)]
        period: String,
        /// Output format.
        #[arg(long, default_value = "json")]
        format: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct BreakerArgs {
    #[command(subcommand)]
    pub action: BreakerAction,
}

#[derive(Debug, Subcommand)]
pub enum BreakerAction {
    /// Show all breaker states.
    Status,
    /// Force breaker to Closed.
    Reset {
        /// Target in provider:model format.
        target: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct ExpiryArgs {
    #[command(subcommand)]
    pub action: ExpiryAction,
}

#[derive(Debug, Subcommand)]
pub enum ExpiryAction {
    /// Show hold-expiry job health.
    Status,
    /// Deduplicate duplicate hold_expired rows.
    Repair,
}

#[derive(Debug, clap::Args)]
pub struct MemoryArgs {
    #[command(subcommand)]
    pub action: MemoryAction,
}

#[derive(Debug, Subcommand)]
pub enum MemoryAction {
    /// Validate and emit a canonical audit row.
    Emit {
        /// YAML payload file.
        yaml_file: PathBuf,
        /// Validate only, do not emit.
        #[arg(long)]
        dry_run: bool,
    },
    /// Search memory audit rows.
    AuditTrail {
        /// Tenant to filter by.
        tenant: String,
        /// Start of time range (ISO 8601).
        #[arg(long)]
        since: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct CompletionsArgs {
    /// Target shell.
    #[arg(value_enum)]
    pub shell: Shell,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

/// FR-AI-021 §1 #7 — CLI error type with exit code mapping.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("auth_failed: {reason}")]
    AuthFailed { reason: String },
    #[error("insufficient_role: needed {needed:?}; have {has:?}")]
    InsufficientRole {
        needed: auth::Role,
        has: Vec<auth::Role>,
    },
    #[error("user_error: {reason}")]
    UserError { reason: String },
    #[error("remote_unreachable: {reason}")]
    RemoteUnreachable { reason: String },
    #[error("destructive_without_confirm")]
    DestructiveWithoutConfirm,
    #[error("schema_violation: {reason}")]
    SchemaViolation { reason: String },
    #[error("internal_error: {reason}")]
    InternalError { reason: String },
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::AuthFailed { .. } | CliError::InsufficientRole { .. } => {
                exit_codes::ExitCode::AuthError as i32
            }
            CliError::UserError { .. } => exit_codes::ExitCode::Generic as i32,
            CliError::RemoteUnreachable { .. } => exit_codes::ExitCode::NetworkError as i32,
            CliError::DestructiveWithoutConfirm => exit_codes::ExitCode::UsageError as i32,
            CliError::SchemaViolation { .. } => exit_codes::ExitCode::PreconditionFailed as i32,
            CliError::InternalError { .. } => exit_codes::ExitCode::Interrupted as i32,
        }
    }
}
