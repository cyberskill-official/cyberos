//! FR-OBS-006 — `cyberos-ai flag-tenant` sampling override.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::auth::{OperatorClaims, Role};
use super::output;
use super::{CliError, FlagTenantArgs};

#[derive(Debug, Serialize)]
struct FlagTenantOutput {
    schema_version: &'static str,
    tenant_id: String,
    file: String,
    already_flagged: bool,
    request_id: String,
}

impl std::fmt::Display for FlagTenantOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.already_flagged {
            writeln!(
                f,
                "Tenant {} was already flagged in {}.",
                self.tenant_id, self.file
            )
        } else {
            writeln!(
                f,
                "Tenant {} flagged for 100% trace sampling in {}.",
                self.tenant_id, self.file
            )
        }
    }
}

pub async fn run(
    args: FlagTenantArgs,
    json: bool,
    confirm: bool,
    claims: &OperatorClaims,
    _pool: &sqlx::PgPool,
) -> Result<(), CliError> {
    super::auth::require_role(claims, &Role::Admin).map_err(|e| CliError::InsufficientRole {
        needed: e.needed(),
        has: e.has(),
    })?;

    let tenant = validate_tenant(&args.tenant)?;
    let file = resolve_flagged_tenants_path(&args.file);

    if !confirm {
        println!("Flag tenant preview:");
        println!("  tenant: {tenant}");
        println!("  file:   {}", file.display());
        eprintln!("To apply, re-run with --confirm");
        return Err(CliError::DestructiveWithoutConfirm);
    }

    let mut tenants = read_flagged_tenants(&file)?;
    let already_flagged = !tenants.insert(tenant.clone());
    if !already_flagged {
        write_flagged_tenants(&file, &tenants)?;
    }

    let command_line = super::current_command_line();
    let command_sha256 = super::command_sha256(&command_line);
    let request_id = super::request_id();

    if !already_flagged {
        crate::memory_writer::emit(
            crate::memory_writer::builders::obs_tenant_flagged_for_sampling(
                &tenant,
                &claims.operator_id,
                &request_id,
                &command_sha256,
            ),
        )
        .await
        .map_err(super::memory_writer_error)?;
    }

    let out = FlagTenantOutput {
        schema_version: "v1",
        tenant_id: tenant,
        file: file.display().to_string(),
        already_flagged,
        request_id,
    };
    output::emit_output(json, &out, |value| println!("{value}"));
    Ok(())
}

fn validate_tenant(tenant: &str) -> Result<String, CliError> {
    let trimmed = tenant.trim();
    if trimmed.is_empty()
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains('\n')
        || trimmed.contains('\r')
    {
        return Err(CliError::UserError {
            reason: "invalid tenant id for flagged_tenants.yaml".to_string(),
        });
    }
    Ok(trimmed.to_string())
}

fn resolve_flagged_tenants_path(path: &Path) -> PathBuf {
    if path.exists() || path.parent().is_some_and(Path::exists) {
        return path.to_path_buf();
    }
    let from_services = Path::new("..").join(path);
    if from_services.parent().is_some_and(Path::exists) {
        return from_services;
    }
    path.to_path_buf()
}

fn read_flagged_tenants(path: &Path) -> Result<BTreeSet<String>, CliError> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(CliError::UserError {
                reason: format!("read {}: {err}", path.display()),
            })
        }
    };
    let mut tenants = BTreeSet::new();
    for line in raw.lines() {
        let value = line
            .split('#')
            .next()
            .unwrap_or_default()
            .trim()
            .strip_prefix('-')
            .unwrap_or_else(|| line.split('#').next().unwrap_or_default().trim())
            .trim();
        if !value.is_empty() {
            tenants.insert(value.to_string());
        }
    }
    Ok(tenants)
}

fn write_flagged_tenants(path: &Path, tenants: &BTreeSet<String>) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| CliError::UserError {
            reason: format!("create {}: {err}", parent.display()),
        })?;
    }
    let mut out = String::from("# FR-OBS-006 flagged tenants for 100% trace capture.\n");
    for tenant in tenants {
        out.push_str("- ");
        out.push_str(tenant);
        out.push('\n');
    }
    std::fs::write(path, out).map_err(|err| CliError::UserError {
        reason: format!("write {}: {err}", path.display()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_tenant_rejects_path_shapes() {
        assert!(validate_tenant("tenant-a").is_ok());
        assert!(validate_tenant("../tenant").is_err());
        assert!(validate_tenant("tenant/a").is_err());
    }

    #[test]
    fn flagged_tenants_round_trip_sorted_and_deduped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("flagged_tenants.yaml");
        let tenants = BTreeSet::from(["tenant-b".to_string(), "tenant-a".to_string()]);
        write_flagged_tenants(&path, &tenants).unwrap();
        let loaded = read_flagged_tenants(&path).unwrap();
        assert_eq!(loaded, tenants);
    }
}
