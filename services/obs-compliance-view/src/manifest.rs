//! Chain-of-custody manifest for a compliance export (FR-OBS-009 §1 #1). The manifest pins what was
//! exported (regulation, window, row count, the chain head at export time) and a SHA-256 of the rows,
//! and is then Ed25519-signed (manifest_signing.rs). An auditor verifies it offline with the
//! `verify_manifest` binary. Canonical, sorted-key JSON makes the signed bytes deterministic (§1 #8).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::proof::to_hex;

/// Whether the export completed; an interrupted export is `Incomplete` and fails verification (§1 #6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportState {
    Complete,
    Incomplete,
}

impl ExportState {
    fn label(self) -> &'static str {
        match self {
            ExportState::Complete => "Complete",
            ExportState::Incomplete => "Incomplete",
        }
    }
}

/// The manifest fields (§1 #1). `ed25519_signature` is set by `manifest_signing::sign`; everything else
/// is the signed payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub export_id: String,
    pub tenant_id: String,
    pub regulation: String,
    pub time_range_start: String,
    pub time_range_end: String,
    pub row_count: u64,
    pub chain_head_at_export: String,
    pub exporter_subject_id: String,
    pub exporter_email: String,
    pub exported_at: String,
    pub sha256_of_rows: String,
    pub public_key_id: String,
    pub state: ExportState,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ed25519_signature: Option<String>,
}

impl Manifest {
    /// SHA-256 (hex) of the canonical JSON bytes of the rows - deterministic over the same rows (§1 #8).
    pub fn sha256_of_rows(canonical_rows: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(canonical_rows);
        to_hex(&hasher.finalize())
    }

    /// The canonical bytes that are signed: every field except the signature, as a sorted-key JSON
    /// object with no insignificant whitespace (RFC 8785-style, §1 #8). The same manifest always
    /// produces the same bytes, so the signature is reproducible by an auditor.
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut m: BTreeMap<&str, serde_json::Value> = BTreeMap::new();
        m.insert("export_id", self.export_id.clone().into());
        m.insert("tenant_id", self.tenant_id.clone().into());
        m.insert("regulation", self.regulation.clone().into());
        m.insert("time_range_start", self.time_range_start.clone().into());
        m.insert("time_range_end", self.time_range_end.clone().into());
        m.insert("row_count", self.row_count.into());
        m.insert(
            "chain_head_at_export",
            self.chain_head_at_export.clone().into(),
        );
        m.insert(
            "exporter_subject_id",
            self.exporter_subject_id.clone().into(),
        );
        m.insert("exporter_email", self.exporter_email.clone().into());
        m.insert("exported_at", self.exported_at.clone().into());
        m.insert("sha256_of_rows", self.sha256_of_rows.clone().into());
        m.insert("public_key_id", self.public_key_id.clone().into());
        m.insert("state", self.state.label().into());
        serde_json::to_vec(&m).expect("a BTreeMap of strings/u64 always serialises")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(state: ExportState) -> Manifest {
        Manifest {
            export_id: "01J0EXPORTULID".into(),
            tenant_id: "00000000-0000-0000-0000-000000000001".into(),
            regulation: "SOC 2".into(),
            time_range_start: "2026-01-01T00:00:00Z".into(),
            time_range_end: "2026-03-31T23:59:59Z".into(),
            row_count: 42,
            chain_head_at_export: "ab".repeat(32),
            exporter_subject_id: "auditor-1".into(),
            exporter_email: "auditor@firm.example".into(),
            exported_at: "2026-06-20T12:00:00Z".into(),
            sha256_of_rows: Manifest::sha256_of_rows(b"[]"),
            public_key_id: "cyberos-infra-2026-Q2".into(),
            state,
            ed25519_signature: None,
        }
    }

    #[test]
    fn sha256_of_rows_is_deterministic() {
        assert_eq!(
            Manifest::sha256_of_rows(b"[]"),
            Manifest::sha256_of_rows(b"[]")
        );
        assert_ne!(
            Manifest::sha256_of_rows(b"[]"),
            Manifest::sha256_of_rows(b"[1]")
        );
        assert_eq!(Manifest::sha256_of_rows(b"[]").len(), 64); // 32 bytes hex
    }

    #[test]
    fn signable_bytes_are_deterministic_and_exclude_the_signature() {
        let mut m = sample(ExportState::Complete);
        let a = m.signable_bytes();
        m.ed25519_signature = Some("set-after-signing".into());
        let b = m.signable_bytes();
        assert_eq!(a, b, "the signature must not be part of the signed bytes");
        // sorted keys: export_id comes before tenant_id in the canonical object
        let s = String::from_utf8(a).unwrap();
        assert!(s.find("export_id").unwrap() < s.find("tenant_id").unwrap());
    }

    #[test]
    fn state_changes_the_signed_bytes() {
        assert_ne!(
            sample(ExportState::Complete).signable_bytes(),
            sample(ExportState::Incomplete).signable_bytes()
        );
    }
}
