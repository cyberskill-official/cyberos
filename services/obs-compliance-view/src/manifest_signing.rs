//! Ed25519 signing and offline verification of a chain-of-custody manifest (TASK-OBS-009 §1 #2, #7). The
//! signature (base64) covers the manifest's canonical signable bytes; an auditor verifies it offline
//! with the published public key plus the exported rows - no CyberOS access needed (DEC-183).

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::manifest::{ExportState, Manifest};

/// Sign a manifest in place: set `ed25519_signature` (base64) over the canonical signable bytes
/// (§1 #1, #8). Signing is microseconds, well inside the 100ms budget (§1 #9).
pub fn sign(secret_key: &[u8; 32], manifest: &mut Manifest) {
    let signing_key = SigningKey::from_bytes(secret_key);
    let signature: Signature = signing_key.sign(&manifest.signable_bytes());
    manifest.ed25519_signature = Some(STANDARD.encode(signature.to_bytes()));
}

/// A verification verdict with a reason, so `verify_manifest` can print PASS or FAIL with a cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail(&'static str),
}

/// Verify a manifest offline (§1 #7): the export must be `Complete`, the rows must hash to
/// `sha256_of_rows`, and the Ed25519 signature must check against `public_key`. `canonical_rows` is the
/// same canonical JSON bytes used at export.
pub fn verify(public_key: &[u8; 32], manifest: &Manifest, canonical_rows: &[u8]) -> Verdict {
    if manifest.state != ExportState::Complete {
        return Verdict::Fail("export state is Incomplete");
    }
    if Manifest::sha256_of_rows(canonical_rows) != manifest.sha256_of_rows {
        return Verdict::Fail("rows do not match sha256_of_rows");
    }
    let Some(sig_b64) = manifest.ed25519_signature.as_ref() else {
        return Verdict::Fail("manifest is unsigned");
    };
    let Ok(sig_bytes) = STANDARD.decode(sig_b64) else {
        return Verdict::Fail("signature is not valid base64");
    };
    let Ok(sig_arr) = <[u8; 64]>::try_from(sig_bytes.as_slice()) else {
        return Verdict::Fail("signature is not 64 bytes");
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(public_key) else {
        return Verdict::Fail("public key is invalid");
    };
    let signature = Signature::from_bytes(&sig_arr);
    if verifying_key
        .verify(&manifest.signable_bytes(), &signature)
        .is_ok()
    {
        Verdict::Pass
    } else {
        Verdict::Fail("ed25519 signature mismatch")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    const SEED: [u8; 32] = [3u8; 32];

    fn public_key() -> [u8; 32] {
        SigningKey::from_bytes(&SEED).verifying_key().to_bytes()
    }

    fn manifest(state: ExportState, rows: &[u8]) -> Manifest {
        Manifest {
            export_id: "01EXPORT".into(),
            tenant_id: "t".into(),
            regulation: "PDPL".into(),
            time_range_start: "2026-01-01T00:00:00Z".into(),
            time_range_end: "2026-02-01T00:00:00Z".into(),
            row_count: 3,
            chain_head_at_export: "cd".repeat(32),
            exporter_subject_id: "auditor-1".into(),
            exporter_email: "a@firm.example".into(),
            exported_at: "2026-06-20T00:00:00Z".into(),
            sha256_of_rows: Manifest::sha256_of_rows(rows),
            public_key_id: "cyberos-infra-2026-Q2".into(),
            state,
            ed25519_signature: None,
        }
    }

    #[test]
    fn sign_then_verify_passes() {
        let rows = br#"[{"kind":"auth.token_issued"}]"#;
        let mut m = manifest(ExportState::Complete, rows);
        sign(&SEED, &mut m);
        assert_eq!(verify(&public_key(), &m, rows), Verdict::Pass);
    }

    #[test]
    fn tampered_rows_fail_on_hash() {
        let rows = br#"[{"kind":"auth.token_issued"}]"#;
        let mut m = manifest(ExportState::Complete, rows);
        sign(&SEED, &mut m);
        assert_eq!(
            verify(&public_key(), &m, br#"[{"kind":"auth.token_failed"}]"#),
            Verdict::Fail("rows do not match sha256_of_rows")
        );
    }

    #[test]
    fn a_tampered_field_fails_on_signature() {
        let rows = b"[]";
        let mut m = manifest(ExportState::Complete, rows);
        sign(&SEED, &mut m);
        m.row_count = 9999; // changed after signing
        assert_eq!(
            verify(&public_key(), &m, rows),
            Verdict::Fail("ed25519 signature mismatch")
        );
    }

    #[test]
    fn incomplete_export_fails_closed() {
        let rows = b"[]";
        let mut m = manifest(ExportState::Incomplete, rows);
        sign(&SEED, &mut m);
        assert_eq!(
            verify(&public_key(), &m, rows),
            Verdict::Fail("export state is Incomplete")
        );
    }

    #[test]
    fn an_unsigned_manifest_fails() {
        let rows = b"[]";
        let m = manifest(ExportState::Complete, rows);
        assert_eq!(
            verify(&public_key(), &m, rows),
            Verdict::Fail("manifest is unsigned")
        );
    }
}
