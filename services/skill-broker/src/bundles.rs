//! TASK-SKILL-105..110 — built-in bundle helper contracts.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryCapture {
    pub row_kind: &'static str,
    pub memory_kind: String,
    pub body: String,
    pub sync_class: String,
}

pub fn memory_capture(memory_kind: impl Into<String>, body: impl Into<String>) -> MemoryCapture {
    MemoryCapture {
        row_kind: "skill.memory_capture.requested",
        memory_kind: memory_kind.into(),
        body: body.into(),
        sync_class: "private".into(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySyncRequest {
    pub row_kind: &'static str,
    pub direction: String,
    pub dry_run: bool,
}

pub fn memory_sync(direction: impl Into<String>, dry_run: bool) -> MemorySyncRequest {
    MemorySyncRequest {
        row_kind: "skill.memory_sync.requested",
        direction: direction.into(),
        dry_run,
    }
}

pub fn synthesis_author(clusters: &[&str]) -> String {
    let joined = clusters.join("; ");
    format!(
        "Derived synthesis from {} cluster(s): {joined}",
        clusters.len()
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MstValidation {
    pub normalized: String,
    pub valid_shape: bool,
}

pub fn validate_vietnam_mst(input: &str) -> MstValidation {
    let normalized = input
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    MstValidation {
        valid_shape: matches!(normalized.len(), 10 | 13),
        normalized,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VietQrPayload {
    pub bank_bin: String,
    pub account: String,
    pub amount_vnd: u64,
    pub memo: String,
    pub payload: String,
}

pub fn vietqr_payload(
    bank_bin: impl Into<String>,
    account: impl Into<String>,
    amount_vnd: u64,
    memo: impl Into<String>,
) -> VietQrPayload {
    let bank_bin = bank_bin.into();
    let account = account.into();
    let memo = memo.into();
    let payload = format!("VQR|{bank_bin}|{account}|{amount_vnd}|{memo}");
    VietQrPayload {
        bank_bin,
        account,
        amount_vnd,
        memo,
        payload,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VatInvoice {
    pub seller_mst: String,
    pub buyer_mst: String,
    pub invoice_no: String,
    pub total_vnd: u64,
}

pub fn vat_invoice_xml(invoice: &VatInvoice) -> String {
    format!(
        "<Invoice><SellerMST>{}</SellerMST><BuyerMST>{}</BuyerMST><No>{}</No><TotalVND>{}</TotalVND></Invoice>",
        escape(&invoice.seller_mst),
        escape(&invoice.buyer_mst),
        escape(&invoice.invoice_no),
        invoice.total_vnd
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingaporeCompanyCheck {
    pub uen: String,
    pub valid_shape: bool,
}

pub fn singapore_uen_check(input: &str) -> SingaporeCompanyCheck {
    let uen = input.trim().to_ascii_uppercase();
    let len = uen.chars().count();
    let valid_shape = (9..=10).contains(&len) && uen.chars().all(|ch| ch.is_ascii_alphanumeric());
    SingaporeCompanyCheck { uen, valid_shape }
}

pub fn singapore_gst_invoice_ref(company_uen: &str, invoice_no: &str) -> String {
    format!(
        "SG-GST-{}-{}",
        company_uen.trim().to_ascii_uppercase(),
        invoice_no.trim()
    )
}

pub fn singapore_cpf_estimate(monthly_wage_cents: u64, employer_rate_bps: u64) -> u64 {
    monthly_wage_cents.saturating_mul(employer_rate_bps) / 10_000
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndonesiaTaxCheck {
    pub npwp: String,
    pub valid_shape: bool,
}

pub fn indonesia_npwp_check(input: &str) -> IndonesiaTaxCheck {
    let npwp = input
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    IndonesiaTaxCheck {
        valid_shape: matches!(npwp.len(), 15 | 16),
        npwp,
    }
}

pub fn indonesia_efaktur_xml(npwp: &str, invoice_no: &str, amount_idr: u64) -> String {
    format!(
        "<EFaktur><NPWP>{}</NPWP><InvoiceNo>{}</InvoiceNo><AmountIDR>{}</AmountIDR></EFaktur>",
        escape(npwp),
        escape(invoice_no),
        amount_idr
    )
}

fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
