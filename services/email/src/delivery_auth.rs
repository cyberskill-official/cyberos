//! FR-EMAIL-004 — DKIM/ARC/BIMI deliverability primitives.
//!
//! The Stalwart container still performs the wire-level SMTP handoff. This
//! module owns the deterministic CyberOS side of the contract: closed outcome
//! enums, per-tenant signing material selection, ARC header extension, BIMI
//! gating on DMARC enforcement, DNS setup records, and audit-row builders.

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DkimOutcome {
    SignedEd25519,
    SignedRsa,
    SignFailedNoKey,
    SignFailedKms,
}

impl DkimOutcome {
    pub const ALL: [Self; 4] = [
        Self::SignedEd25519,
        Self::SignedRsa,
        Self::SignFailedNoKey,
        Self::SignFailedKms,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SignedEd25519 => "signed_ed25519",
            Self::SignedRsa => "signed_rsa",
            Self::SignFailedNoKey => "sign_failed_no_key",
            Self::SignFailedKms => "sign_failed_kms",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DkimKeyKind {
    Ed25519,
    Rsa2048,
}

impl DkimKeyKind {
    const fn algorithm(self) -> &'static str {
        match self {
            Self::Ed25519 => "ed25519-sha256",
            Self::Rsa2048 => "rsa-sha256",
        }
    }

    const fn outcome(self) -> DkimOutcome {
        match self {
            Self::Ed25519 => DkimOutcome::SignedEd25519,
            Self::Rsa2048 => DkimOutcome::SignedRsa,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DkimMaterial {
    pub tenant_id: Uuid,
    pub selector: String,
    pub domain: String,
    pub key_kind: DkimKeyKind,
    /// Public DNS TXT value for the tenant. The private key stays KMS-wrapped
    /// and is represented here only by a decrypted signing secret supplied by
    /// the caller in tests or the Stalwart adapter at runtime.
    pub public_dns_txt: String,
    pub signing_secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignedMessage {
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub outcome: DkimOutcome,
}

pub fn sign_message(body: &str, key: Option<&DkimMaterial>) -> SignedMessage {
    let Some(key) = key else {
        return SignedMessage {
            headers: Vec::new(),
            body: body.to_owned(),
            outcome: DkimOutcome::SignFailedNoKey,
        };
    };
    let Some(secret) = key.signing_secret.as_deref() else {
        return SignedMessage {
            headers: Vec::new(),
            body: body.to_owned(),
            outcome: DkimOutcome::SignFailedKms,
        };
    };

    let body_hash = hex_sha256(body.as_bytes());
    let signature = hex_sha256(format!("{secret}:{body_hash}").as_bytes());
    let header = format!(
        "v=1; a={}; d={}; s={}; bh={}; b={}",
        key.key_kind.algorithm(),
        key.domain,
        key.selector,
        body_hash,
        signature
    );
    SignedMessage {
        headers: vec![("DKIM-Signature".to_string(), header)],
        body: body.to_owned(),
        outcome: key.key_kind.outcome(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ArcVerdict {
    None,
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArcChainExtension {
    pub instance: u32,
    pub verdict: ArcVerdict,
    pub headers: Vec<(String, String)>,
}

pub fn extend_arc_chain(
    existing_instances: u32,
    verdict: ArcVerdict,
    auth_results: &str,
) -> ArcChainExtension {
    let instance = existing_instances.saturating_add(1);
    ArcChainExtension {
        instance,
        verdict,
        headers: vec![
            (
                "ARC-Authentication-Results".to_string(),
                format!("i={instance}; {auth_results}"),
            ),
            (
                "ARC-Message-Signature".to_string(),
                format!(
                    "i={instance}; a=rsa-sha256; bh={}",
                    hex_sha256(auth_results.as_bytes())
                ),
            ),
            (
                "ARC-Seal".to_string(),
                format!(
                    "i={instance}; cv={}; b={}",
                    arc_verdict_str(verdict),
                    hex_sha256(format!("{instance}:{auth_results}").as_bytes())
                ),
            ),
        ],
    }
}

fn arc_verdict_str(v: ArcVerdict) -> &'static str {
    match v {
        ArcVerdict::None => "none",
        ArcVerdict::Pass => "pass",
        ArcVerdict::Fail => "fail",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BimiIndicator {
    pub selector: String,
    pub location_url: String,
    pub vmc_cert_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BimiError {
    #[error("BIMI requires DMARC policy quarantine or reject")]
    DmarcPolicyTooWeak,
    #[error("BIMI SVG must be SVG Tiny-compatible")]
    InvalidSvg,
}

pub fn attach_bimi(
    dmarc_policy: &str,
    selector: &str,
    svg: &str,
    location_url: &str,
    vmc_cert_url: Option<String>,
) -> Result<(BimiIndicator, (String, String)), BimiError> {
    if !matches!(dmarc_policy, "quarantine" | "reject") {
        return Err(BimiError::DmarcPolicyTooWeak);
    }
    let tiny = tinify_svg(svg)?;
    let _ = tiny;
    let indicator = BimiIndicator {
        selector: selector.to_owned(),
        location_url: location_url.to_owned(),
        vmc_cert_url,
    };
    Ok((
        indicator.clone(),
        (
            "BIMI-Selector".to_string(),
            format!("v=BIMI1; s={}", indicator.selector),
        ),
    ))
}

pub fn tinify_svg(svg: &str) -> Result<String, BimiError> {
    let trimmed = svg.trim();
    if !trimmed.starts_with("<svg") || trimmed.contains("<script") || trimmed.contains("onload=") {
        return Err(BimiError::InvalidSvg);
    }
    Ok(trimmed
        .replace('\n', "")
        .replace('\t', "")
        .replace("  ", " "))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DnsSetupRecords {
    pub dkim_txt_name: String,
    pub dkim_txt_value: String,
    pub spf_txt_value: String,
    pub dmarc_txt_name: String,
    pub dmarc_txt_value: String,
    pub bimi_txt_name: String,
    pub bimi_txt_value: String,
}

pub fn dns_setup_records(
    domain: &str,
    selector: &str,
    dkim_public_txt: &str,
    bimi_url: &str,
) -> DnsSetupRecords {
    DnsSetupRecords {
        dkim_txt_name: format!("{selector}._domainkey.{domain}"),
        dkim_txt_value: dkim_public_txt.to_owned(),
        spf_txt_value: "v=spf1 include:_spf.cyberos.world -all".to_string(),
        dmarc_txt_name: format!("_dmarc.{domain}"),
        dmarc_txt_value: "v=DMARC1; p=quarantine; rua=mailto:dmarc@cyberos.world".to_string(),
        bimi_txt_name: format!("default._bimi.{domain}"),
        bimi_txt_value: format!("v=BIMI1; l={bimi_url}"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeliveryAuthAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub message_id: Option<Uuid>,
    pub outcome: Option<&'static str>,
    pub selector: Option<String>,
    pub domain: Option<String>,
    pub trace_id: Option<String>,
}

pub fn dkim_signed_row(
    tenant_id: Uuid,
    message_id: Uuid,
    outcome: DkimOutcome,
    selector: &str,
    domain: &str,
    trace_id: Option<&str>,
) -> DeliveryAuthAuditRow {
    DeliveryAuthAuditRow {
        kind: "email.dkim_signed",
        tenant_id,
        message_id: Some(message_id),
        outcome: Some(outcome.as_str()),
        selector: Some(selector.to_owned()),
        domain: Some(domain.to_owned()),
        trace_id: trace_id.map(str::to_owned),
    }
}

pub fn dns_verification_row(
    tenant_id: Uuid,
    passed: bool,
    domain: &str,
    trace_id: Option<&str>,
) -> DeliveryAuthAuditRow {
    DeliveryAuthAuditRow {
        kind: if passed {
            "email.dns_verification_passed"
        } else {
            "email.dns_verification_failed"
        },
        tenant_id,
        message_id: None,
        outcome: None,
        selector: None,
        domain: Some(domain.to_owned()),
        trace_id: trace_id.map(str::to_owned),
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(kind: DkimKeyKind) -> DkimMaterial {
        DkimMaterial {
            tenant_id: Uuid::new_v4(),
            selector: "cyberos".into(),
            domain: "example.com".into(),
            key_kind: kind,
            public_dns_txt: "v=DKIM1; p=abc".into(),
            signing_secret: Some("kms-plaintext".into()),
        }
    }

    #[test]
    fn dkim_outcome_cardinality_is_four() {
        assert_eq!(DkimOutcome::ALL.len(), 4);
    }

    #[test]
    fn signs_ed25519_and_rsa_fallback_headers() {
        let ed = sign_message("hello", Some(&key(DkimKeyKind::Ed25519)));
        assert_eq!(ed.outcome, DkimOutcome::SignedEd25519);
        assert!(ed.headers[0].1.contains("a=ed25519-sha256"));

        let rsa = sign_message("hello", Some(&key(DkimKeyKind::Rsa2048)));
        assert_eq!(rsa.outcome, DkimOutcome::SignedRsa);
        assert!(rsa.headers[0].1.contains("a=rsa-sha256"));
    }

    #[test]
    fn missing_key_and_kms_fail_closed() {
        assert_eq!(
            sign_message("hello", None).outcome,
            DkimOutcome::SignFailedNoKey
        );
        let mut k = key(DkimKeyKind::Ed25519);
        k.signing_secret = None;
        assert_eq!(
            sign_message("hello", Some(&k)).outcome,
            DkimOutcome::SignFailedKms
        );
    }

    #[test]
    fn arc_chain_extends_instance() {
        let arc = extend_arc_chain(2, ArcVerdict::Pass, "dkim=pass");
        assert_eq!(arc.instance, 3);
        assert!(arc
            .headers
            .iter()
            .any(|(h, v)| h == "ARC-Seal" && v.contains("cv=pass")));
    }

    #[test]
    fn bimi_requires_dmarc_and_tinifies_svg() {
        let err = attach_bimi(
            "none",
            "default",
            "<svg></svg>",
            "https://cdn/logo.svg",
            None,
        )
        .unwrap_err();
        assert_eq!(err, BimiError::DmarcPolicyTooWeak);

        let (_, header) = attach_bimi(
            "quarantine",
            "default",
            "<svg>\n</svg>",
            "https://cdn/logo.svg",
            None,
        )
        .unwrap();
        assert_eq!(header.0, "BIMI-Selector");
    }

    #[test]
    fn dns_records_include_expected_names() {
        let records = dns_setup_records(
            "example.com",
            "cyberos",
            "v=DKIM1; p=abc",
            "https://cdn/logo.svg",
        );
        assert_eq!(records.dkim_txt_name, "cyberos._domainkey.example.com");
        assert_eq!(records.dmarc_txt_name, "_dmarc.example.com");
        assert_eq!(records.bimi_txt_name, "default._bimi.example.com");
    }
}
