//! FR-AI-021 — `cyberos-ai policy` subcommand.

use std::path::Path;

use rust_decimal::Decimal;
use serde_json::json;

use super::auth::{OperatorClaims, Role};
use super::output;
use super::{CliError, PolicyAction};
use crate::policy;
use sqlx::PgPool;

#[derive(serde::Serialize)]
struct DiffOutput {
    schema_version: &'static str,
    tenant: String,
    changes: Vec<PolicyChange>,
}

#[derive(serde::Serialize)]
struct PolicyChange {
    field: String,
    before: serde_json::Value,
    after: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_changed: Option<bool>,
}

impl std::fmt::Display for DiffOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "DIFF for tenant {}:", self.tenant)?;
        for change in &self.changes {
            writeln!(
                f,
                "  {}: {} \u{2192} {}",
                change.field, change.before, change.after
            )?;
        }
        writeln!(f)?;
        writeln!(f, "To apply, re-run with --confirm")
    }
}

pub async fn run(
    args: PolicyAction,
    json: bool,
    confirm: bool,
    claims: &OperatorClaims,
    pool: &PgPool,
) -> Result<(), CliError> {
    match args {
        PolicyAction::Set {
            tenant,
            cap_usd,
            zdr_required,
            residency,
            allowed_personas,
        } => {
            super::auth::require_role(claims, &Role::Mutate).map_err(|e| {
                CliError::InsufficientRole {
                    needed: e.needed(),
                    has: e.has(),
                }
            })?;
            set(
                pool,
                &tenant,
                cap_usd,
                zdr_required,
                residency,
                allowed_personas,
                json,
                confirm,
            )
            .await
        }
        PolicyAction::Validate { yaml_file } => validate(&yaml_file),
        PolicyAction::Diff { tenant, vs } => diff(pool, &tenant, &vs, json).await,
    }
}

fn validate(yaml_file: &Path) -> Result<(), CliError> {
    let yaml = std::fs::read_to_string(yaml_file).map_err(|e| CliError::UserError {
        reason: format!("read {}: {e}", yaml_file.display()),
    })?;

    match policy::validate_yaml(&yaml) {
        Ok(p) => {
            println!(
                "OK tenant_id={} (cap=${})",
                p.tenant_id, p.ai_policy.monthly_cap_usd
            );
            Ok(())
        }
        Err(errs) => {
            for e in &errs {
                eprintln!("ERROR {e}");
            }
            Err(CliError::SchemaViolation {
                reason: errs.join("; "),
            })
        }
    }
}

async fn diff(pool: &PgPool, tenant: &str, yaml_file: &Path, json: bool) -> Result<(), CliError> {
    let current = query_policy(pool, tenant).await?;

    let yaml = std::fs::read_to_string(yaml_file).map_err(|e| CliError::UserError {
        reason: format!("read {}: {e}", yaml_file.display()),
    })?;
    let proposed: policy::TenantPolicy =
        serde_yaml::from_str(&yaml).map_err(|e| CliError::SchemaViolation {
            reason: e.to_string(),
        })?;

    let mut changes = Vec::new();

    if current.ai_policy.monthly_cap_usd != proposed.ai_policy.monthly_cap_usd {
        changes.push(PolicyChange {
            field: "cap_usd".into(),
            before: json!(current.ai_policy.monthly_cap_usd.to_string()),
            after: json!(proposed.ai_policy.monthly_cap_usd.to_string()),
            secret_changed: None,
        });
    }

    if current.ai_policy.zdr_required != proposed.ai_policy.zdr_required {
        changes.push(PolicyChange {
            field: "zdr_required".into(),
            before: json!(current.ai_policy.zdr_required),
            after: json!(proposed.ai_policy.zdr_required),
            secret_changed: None,
        });
    }

    if current.ai_policy.residency != proposed.ai_policy.residency {
        changes.push(PolicyChange {
            field: "residency".into(),
            before: json!(format!("{:?}", current.ai_policy.residency)),
            after: json!(format!("{:?}", proposed.ai_policy.residency)),
            secret_changed: None,
        });
    }

    if current.ai_policy.allowed_personas != proposed.ai_policy.allowed_personas {
        changes.push(PolicyChange {
            field: "allowed_personas".into(),
            before: json!(current.ai_policy.allowed_personas),
            after: json!(proposed.ai_policy.allowed_personas),
            secret_changed: None,
        });
    }

    let data = DiffOutput {
        schema_version: "v1",
        tenant: tenant.to_string(),
        changes,
    };

    output::emit_output(json, &data, |d| println!("{d}"));
    Ok(())
}

async fn set(
    pool: &PgPool,
    tenant: &str,
    cap_usd: Option<f64>,
    zdr_required: Option<bool>,
    residency: Option<String>,
    allowed_personas: Option<Vec<String>>,
    _json: bool,
    confirm: bool,
) -> Result<(), CliError> {
    let current = query_policy(pool, tenant).await?;

    let mut changes = Vec::new();

    if let Some(cap) = cap_usd {
        let new_val = Decimal::try_from(cap).map_err(|e| CliError::UserError {
            reason: format!("invalid cap_usd: {e}"),
        })?;
        if current.ai_policy.monthly_cap_usd != new_val {
            changes.push(PolicyChange {
                field: "cap_usd".into(),
                before: json!(current.ai_policy.monthly_cap_usd.to_string()),
                after: json!(new_val.to_string()),
                secret_changed: None,
            });
        }
    }

    if let Some(zdr) = zdr_required {
        if current.ai_policy.zdr_required != zdr {
            changes.push(PolicyChange {
                field: "zdr_required".into(),
                before: json!(current.ai_policy.zdr_required),
                after: json!(zdr),
                secret_changed: None,
            });
        }
    }

    if let Some(ref res) = residency {
        changes.push(PolicyChange {
            field: "residency".into(),
            before: json!(format!("{:?}", current.ai_policy.residency)),
            after: json!(res),
            secret_changed: None,
        });
    }

    if let Some(ref personas) = allowed_personas {
        changes.push(PolicyChange {
            field: "allowed_personas".into(),
            before: json!(current.ai_policy.allowed_personas),
            after: json!(personas),
            secret_changed: None,
        });
    }

    if changes.is_empty() {
        println!("No changes to apply.");
        return Ok(());
    }

    let diff = DiffOutput {
        schema_version: "v1",
        tenant: tenant.to_string(),
        changes,
    };

    if !confirm {
        println!("{diff}");
        eprintln!("To apply, re-run with --confirm");
        return Err(CliError::DestructiveWithoutConfirm);
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| CliError::RemoteUnreachable {
            reason: e.to_string(),
        })?;

    if let Some(cap) = cap_usd {
        let val = Decimal::try_from(cap).map_err(|e| CliError::UserError {
            reason: format!("invalid cap_usd: {e}"),
        })?;
        sqlx::query("UPDATE tenant_policies SET ai_policy = jsonb_set(ai_policy, '{monthly_cap_usd}', to_jsonb($1::text), true) WHERE tenant_id = $2")
            .bind(val.to_string())
            .bind(tenant)
            .execute(&mut *tx)
            .await
            .map_err(|e| CliError::RemoteUnreachable { reason: e.to_string() })?;
    }

    tx.commit().await.map_err(|e| CliError::RemoteUnreachable {
        reason: e.to_string(),
    })?;

    // Emit audit row
    let _ = crate::memory_writer::emit(crate::memory_writer::MemoryEmit {
        kind: crate::memory_writer::AiInvocationKind::Precheck,
        path: format!("memories/ai-policy-updates/{}_{}.md", tenant, chrono::Utc::now().timestamp_millis()),
        extra: serde_json::json!({
            "operator_id": "cli-operator",
            "tenant": tenant,
            "changes": diff.changes.iter().map(|c| serde_json::json!({"field": c.field, "before": c.before, "after": c.after})).collect::<Vec<_>>(),
        }),
    }).await;

    println!("{diff}");
    println!("Policy updated successfully.");
    Ok(())
}

async fn query_policy(pool: &PgPool, tenant: &str) -> Result<policy::TenantPolicy, CliError> {
    let row: (serde_json::Value,) =
        sqlx::query_as("SELECT ai_policy FROM tenant_policies WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_optional(pool)
            .await
            .map_err(|e| CliError::RemoteUnreachable {
                reason: e.to_string(),
            })?
            .ok_or_else(|| CliError::UserError {
                reason: format!("tenant not found: {tenant}"),
            })?;

    let ai_policy: policy::AiPolicy =
        serde_json::from_value(row.0).map_err(|e| CliError::InternalError {
            reason: format!("policy deserialization: {e}"),
        })?;

    Ok(policy::TenantPolicy {
        tenant_id: tenant.to_string(),
        tenant_jurisdiction: None,
        ai_policy,
    })
}

impl From<crate::policy::PolicyError> for CliError {
    fn from(e: crate::policy::PolicyError) -> Self {
        match e {
            crate::policy::PolicyError::PolicyMissing { tenant_id } => CliError::UserError {
                reason: format!("policy missing for tenant {tenant_id}"),
            },
            crate::policy::PolicyError::PolicyInvalid { tenant_id, .. } => {
                CliError::SchemaViolation {
                    reason: format!("policy invalid for tenant {tenant_id}"),
                }
            }
            crate::policy::PolicyError::InvalidTenantId { reason } => CliError::UserError {
                reason: format!("invalid tenant_id: {reason}"),
            },
            crate::policy::PolicyError::IoError { tenant_id, source } => {
                CliError::RemoteUnreachable {
                    reason: format!("io error for tenant {tenant_id}: {source}"),
                }
            }
            crate::policy::PolicyError::NotInitialised => CliError::InternalError {
                reason: "loader not initialised".into(),
            },
        }
    }
}
