//! HTTP-facing orchestration for TASK-EMAIL-011.

use crate::dsar::{
    aggregate_jsonl, complete_export_job, enqueue_export_job, get_export_job,
    load_subject_messages, DsarExportJobRow, DsarExportSummary,
};
use crate::errors::EmailResult;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct DsarExportRequest {
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
    pub requested_by: Option<Uuid>,
    pub idempotency_key: String,
    pub output_jsonl_s3_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DsarExportResponse {
    pub job: DsarExportJobRow,
    pub summary: Option<DsarExportSummary>,
}

pub async fn export(db: &PgPool, req: DsarExportRequest) -> EmailResult<DsarExportResponse> {
    let job = enqueue_export_job(
        db,
        req.tenant_id,
        req.subject_id,
        req.requested_by,
        &req.idempotency_key,
    )
    .await?;

    let messages = load_subject_messages(db, req.tenant_id, req.subject_id).await?;
    let summary = aggregate_jsonl(req.tenant_id, req.subject_id, &messages)
        .map_err(|e| crate::EmailError::Other(e.to_string()))?;
    let output_key = req.output_jsonl_s3_key.unwrap_or_else(|| {
        format!(
            "s3://cyberos-dsar/{}/{}/{}.jsonl",
            req.tenant_id, req.subject_id, job.id
        )
    });
    let output_sha256 = sha256_hex(&summary.jsonl);
    let job = complete_export_job(
        db,
        req.tenant_id,
        job.id,
        &output_key,
        &output_sha256,
        &summary,
    )
    .await?;
    Ok(DsarExportResponse {
        job,
        summary: Some(summary),
    })
}

pub async fn get_job(
    db: &PgPool,
    tenant_id: Uuid,
    job_id: Uuid,
) -> EmailResult<Option<DsarExportJobRow>> {
    get_export_job(db, tenant_id, job_id)
        .await
        .map_err(Into::into)
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}
