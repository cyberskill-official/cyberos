//! FR-OBS-006 §1 #3 / FR-AI-021 - `cyberos-ai flag-tenant` subcommand. Flags (or unflags) a tenant for
//! 100% trace sampling by editing the collector's `flagged_tenants.yaml`; the collector hot-reloads on
//! the change (no restart). The file path is `OBS_FLAGGED_TENANTS_FILE`, defaulting to the in-repo
//! config. Requires the Mutate role and `--confirm`.

use super::auth::{OperatorClaims, Role};
use super::{CliError, FlagTenantArgs};

const DEFAULT_FILE: &str = "services/obs-collector/config/flagged_tenants.yaml";

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct FlaggedTenants {
    #[serde(default)]
    flagged_tenants: Vec<String>,
}

pub async fn run(
    args: FlagTenantArgs,
    json: bool,
    claims: &OperatorClaims,
) -> Result<(), CliError> {
    super::auth::require_role(claims, &Role::Mutate).map_err(|e| CliError::InsufficientRole {
        needed: e.needed(),
        has: e.has(),
    })?;
    if !args.confirm {
        return Err(CliError::UserError {
            reason: "flag-tenant mutates the sampling config; re-run with --confirm".into(),
        });
    }

    let path =
        std::env::var("OBS_FLAGGED_TENANTS_FILE").unwrap_or_else(|_| DEFAULT_FILE.to_string());
    let mut doc: FlaggedTenants = match std::fs::read_to_string(&path) {
        Ok(s) => serde_yaml::from_str(&s).map_err(|e| CliError::UserError {
            reason: format!("flagged_tenants.yaml is malformed: {e}"),
        })?,
        Err(_) => FlaggedTenants::default(),
    };

    let action = if args.remove {
        let before = doc.flagged_tenants.len();
        doc.flagged_tenants.retain(|t| t != &args.tenant_id);
        if doc.flagged_tenants.len() == before {
            "not_present"
        } else {
            "removed"
        }
    } else if doc.flagged_tenants.contains(&args.tenant_id) {
        "already_flagged"
    } else {
        doc.flagged_tenants.push(args.tenant_id.clone());
        "flagged"
    };

    let yaml = serde_yaml::to_string(&doc).map_err(|e| CliError::InternalError {
        reason: e.to_string(),
    })?;
    std::fs::write(&path, yaml).map_err(|e| CliError::RemoteUnreachable {
        reason: format!("cannot write {path}: {e}"),
    })?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "schema_version": "flag-tenant@1",
                "action": action,
                "tenant_id": args.tenant_id,
                "flagged_count": doc.flagged_tenants.len(),
                "file": path,
            })
        );
    } else {
        println!(
            "{action}: tenant {} - {} tenant(s) now flagged at 100% sampling",
            args.tenant_id,
            doc.flagged_tenants.len()
        );
    }
    Ok(())
}
