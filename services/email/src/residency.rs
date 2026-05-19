//! FR-EMAIL-001 §3.5 — per-tenant residency resolver.
//!
//! Slice 1 ships its own lookup against `tenant_residency`; slice 2
//! delegates to FR-AI-016's broader residency-policy framework.

use crate::errors::{EmailError, EmailResult};
use crate::types::EmailStorageBinding;
use uuid::Uuid;

/// Returns the storage binding for a tenant. The bucket + KMS key naming
/// convention is `cyberos-email-<residency>-bodies` + KMS alias of the
/// same shape per DEC-302.
pub async fn resolve(tenant_id: Uuid, db: &sqlx::PgPool) -> EmailResult<EmailStorageBinding> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT residency FROM tenant_residency WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_optional(db)
            .await?;

    let residency = row.ok_or(EmailError::NoResidencyForTenant(tenant_id))?.0;

    binding_for_residency(&residency).map_err(|_| EmailError::UnknownResidency(residency))
}

/// Pure mapping from residency tag to (region, bucket, KMS key id).
/// Separated so unit tests don't need a DB.
pub fn binding_for_residency(residency: &str) -> Result<EmailStorageBinding, &'static str> {
    let (region, bucket_prefix) = match residency {
        "vn-1" => ("ap-southeast-1", "cyberos-email-vn-1-bodies"),
        "sg-1" => ("ap-southeast-1", "cyberos-email-sg-1-bodies"),
        "eu-1" => ("eu-west-1", "cyberos-email-eu-1-bodies"),
        "us-1" => ("us-east-1", "cyberos-email-us-1-bodies"),
        _ => return Err("unknown_residency"),
    };
    Ok(EmailStorageBinding {
        residency: residency.to_owned(),
        region: region.to_owned(),
        bucket: bucket_prefix.to_owned(),
        kms_key_id: format!("alias/{bucket_prefix}"),
    })
}

/// Assert that an actual storage write destination matches the expected
/// residency. The Stalwart inbound handler calls this BEFORE the S3 PUT
/// per §1 #12 (fail-closed cross-residency).
pub fn assert_residency_match(
    tenant_id: Uuid,
    expected_binding: &EmailStorageBinding,
    actual_bucket: &str,
) -> EmailResult<()> {
    if expected_binding.bucket != actual_bucket {
        return Err(EmailError::ResidencyMismatch {
            tenant_id,
            expected: expected_binding.bucket.clone(),
            actual: actual_bucket.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_residencies_resolve() {
        for r in ["vn-1", "sg-1", "eu-1", "us-1"] {
            let b = binding_for_residency(r).unwrap();
            assert_eq!(b.residency, r);
            assert!(b.bucket.starts_with("cyberos-email-"));
            assert_eq!(b.kms_key_id, format!("alias/{}", b.bucket));
        }
    }

    #[test]
    fn unknown_residency_rejected() {
        assert!(binding_for_residency("zz-1").is_err());
        assert!(binding_for_residency("").is_err());
    }

    #[test]
    fn vn_residency_lands_in_ap_southeast_1() {
        let b = binding_for_residency("vn-1").unwrap();
        assert_eq!(b.region, "ap-southeast-1");
        assert_eq!(b.bucket, "cyberos-email-vn-1-bodies");
    }

    #[test]
    fn eu_residency_lands_in_eu_west_1() {
        let b = binding_for_residency("eu-1").unwrap();
        assert_eq!(b.region, "eu-west-1");
    }

    #[test]
    fn residency_mismatch_is_fail_closed() {
        let tid = Uuid::new_v4();
        let vn = binding_for_residency("vn-1").unwrap();
        // VN tenant's body must NOT land in eu-1 bucket.
        let err = assert_residency_match(tid, &vn, "cyberos-email-eu-1-bodies").unwrap_err();
        match err {
            EmailError::ResidencyMismatch {
                tenant_id,
                expected,
                actual,
            } => {
                assert_eq!(tenant_id, tid);
                assert_eq!(expected, "cyberos-email-vn-1-bodies");
                assert_eq!(actual, "cyberos-email-eu-1-bodies");
            }
            _ => panic!("expected ResidencyMismatch"),
        }
    }

    #[test]
    fn residency_match_passes() {
        let tid = Uuid::new_v4();
        let vn = binding_for_residency("vn-1").unwrap();
        assert!(assert_residency_match(tid, &vn, "cyberos-email-vn-1-bodies").is_ok());
    }
}
