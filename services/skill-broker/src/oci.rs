//! TASK-SKILL-102/201 — OCI bundle publishing primitives.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleRef {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishPlan {
    pub bundle: BundleRef,
    pub immutable: bool,
    pub cosign_required: bool,
    pub tenant_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OciError {
    #[error("mutable tag forbidden: {0}")]
    MutableTag(String),
    #[error("tenant scope required")]
    TenantScopeRequired,
}

pub fn digest_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex(&hasher.finalize()))
}

pub fn plan_publish(
    registry: impl Into<String>,
    repository: impl Into<String>,
    tag: impl Into<String>,
    bytes: &[u8],
    tenant_scope: impl Into<String>,
) -> Result<PublishPlan, OciError> {
    let tag = tag.into();
    if matches!(tag.as_str(), "latest" | "dev" | "main") {
        return Err(OciError::MutableTag(tag));
    }
    let tenant_scope = tenant_scope.into();
    if tenant_scope.trim().is_empty() {
        return Err(OciError::TenantScopeRequired);
    }
    Ok(PublishPlan {
        bundle: BundleRef {
            registry: registry.into(),
            repository: repository.into(),
            tag,
            digest: digest_bytes(bytes),
        },
        immutable: true,
        cosign_required: true,
        tenant_scope,
    })
}

fn hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}
