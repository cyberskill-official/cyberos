//! HTTP-facing orchestration for FR-EMAIL-004.

use crate::delivery_auth::{
    attach_bimi, dns_setup_records, mark_dns_verification, upsert_dns_setup, BimiIndicator,
    DeliveryAuthEventRow, DnsSetupRecords, TenantDnsSetupRow,
};
use crate::errors::EmailResult;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct DnsSetupRequest {
    pub domain: String,
    pub selector: Option<String>,
    pub dkim_public_txt: String,
    pub bimi_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsSetupResponse {
    pub setup: TenantDnsSetupRow,
    pub records: DnsSetupRecords,
}

pub async fn dns_setup(
    db: &PgPool,
    tenant_id: Uuid,
    req: DnsSetupRequest,
) -> EmailResult<DnsSetupResponse> {
    let selector = req.selector.as_deref().unwrap_or("cyberos");
    let bimi_url = req
        .bimi_url
        .as_deref()
        .unwrap_or("https://assets.cyberos.world/bimi/default.svg");
    let records = dns_setup_records(&req.domain, selector, &req.dkim_public_txt, bimi_url);
    let setup = upsert_dns_setup(db, tenant_id, &req.domain, selector, &records).await?;
    Ok(DnsSetupResponse { setup, records })
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsVerifyRequest {
    pub domain: String,
    pub selector: Option<String>,
    pub passed: bool,
    pub trace_id: Option<String>,
}

pub async fn dns_verify(
    db: &PgPool,
    tenant_id: Uuid,
    req: DnsVerifyRequest,
) -> EmailResult<DeliveryAuthEventRow> {
    mark_dns_verification(
        db,
        tenant_id,
        &req.domain,
        req.selector.as_deref().unwrap_or("cyberos"),
        req.passed,
        req.trace_id.as_deref(),
    )
    .await
    .map_err(Into::into)
}

#[derive(Debug, Clone, Deserialize)]
pub struct BimiEnableRequest {
    pub dmarc_policy: String,
    pub selector: Option<String>,
    pub svg: String,
    pub location_url: String,
    pub vmc_cert_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BimiEnableResponse {
    pub indicator: BimiIndicator,
    pub header_name: String,
    pub header_value: String,
}

pub async fn bimi_enable(req: BimiEnableRequest) -> EmailResult<BimiEnableResponse> {
    let selector = req.selector.as_deref().unwrap_or("default");
    let (indicator, (header_name, header_value)) = attach_bimi(
        &req.dmarc_policy,
        selector,
        &req.svg,
        &req.location_url,
        req.vmc_cert_url,
    )
    .map_err(|e| crate::EmailError::Other(e.to_string()))?;
    Ok(BimiEnableResponse {
        indicator,
        header_name,
        header_value,
    })
}
