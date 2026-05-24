//! FR-EMAIL-011 — DSAR JSONL export primitives.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentRef {
    pub filename: String,
    pub s3_key: String,
    pub sha256: String,
    pub size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DsarMessage {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub author_subject_id: Option<Uuid>,
    pub participant_subject_ids: Vec<Uuid>,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentRef>,
    pub memory_audit_chain_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DsarExportSummary {
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
    pub message_count: usize,
    pub attachment_count: usize,
    pub jsonl: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DsarExportJobRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
    pub requested_by: Option<Uuid>,
    pub status: String,
    pub idempotency_key: String,
    pub output_jsonl_s3_key: Option<String>,
    pub output_sha256: Option<String>,
    pub message_count: i64,
    pub attachment_count: i64,
    pub error_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
struct DsarMessageRow {
    id: Uuid,
    tenant_id: Uuid,
    author_subject_id: Option<Uuid>,
    participant_subject_ids: Vec<Uuid>,
    from_address: String,
    to_addresses: Vec<String>,
    cc_addresses: Vec<String>,
    subject: Option<String>,
    s3_body_key: String,
    body_sha256_hex: String,
    memory_audit_chain_hash: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct AttachmentRow {
    message_id: Uuid,
    filename: String,
    s3_key: String,
    sha256: String,
    size: i64,
}

pub async fn enqueue_export_job(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    requested_by: Option<Uuid>,
    idempotency_key: &str,
) -> Result<DsarExportJobRow, sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let row: DsarExportJobRow = sqlx::query_as(
        "INSERT INTO dsar_export_jobs (tenant_id, subject_id, requested_by, idempotency_key)
         VALUES ($1,$2,$3,$4)
         ON CONFLICT (tenant_id, subject_id, idempotency_key) DO UPDATE
            SET idempotency_key = EXCLUDED.idempotency_key
         RETURNING *",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(requested_by)
    .bind(idempotency_key)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn get_export_job(
    pool: &PgPool,
    tenant_id: Uuid,
    job_id: Uuid,
) -> Result<Option<DsarExportJobRow>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let row: Option<DsarExportJobRow> =
        sqlx::query_as("SELECT * FROM dsar_export_jobs WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(job_id)
            .fetch_optional(&mut *tx)
            .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn load_subject_messages(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> Result<Vec<DsarMessage>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let rows: Vec<DsarMessageRow> = sqlx::query_as(
        "SELECT
            m.id,
            m.tenant_id,
            (
                SELECT msr.subject_id
                FROM message_subject_refs msr
                WHERE msr.tenant_id = m.tenant_id
                  AND msr.message_id = m.id
                  AND msr.relation = 'author'
                LIMIT 1
            ) AS author_subject_id,
            COALESCE((
                SELECT array_agg(DISTINCT msr.subject_id)
                FROM message_subject_refs msr
                WHERE msr.tenant_id = m.tenant_id
                  AND msr.message_id = m.id
                  AND msr.relation IN ('recipient', 'cc')
            ), ARRAY[]::uuid[]) AS participant_subject_ids,
            m.from_address,
            m.to_addresses,
            m.cc_addresses,
            m.subject,
            m.s3_body_key,
            m.body_sha256_hex,
            (
                SELECT e.payload->>'memory_chain_hash'
                FROM outbound_delivery_events e
                WHERE e.tenant_id = m.tenant_id
                  AND e.message_id = m.id
                ORDER BY e.created_at DESC
                LIMIT 1
            ) AS memory_audit_chain_hash
         FROM message_metadata m
         WHERE m.tenant_id = $1
           AND EXISTS (
                SELECT 1 FROM message_subject_refs msr
                WHERE msr.tenant_id = m.tenant_id
                  AND msr.message_id = m.id
                  AND msr.subject_id = $2
           )
         ORDER BY m.received_at ASC",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .fetch_all(&mut *tx)
    .await?;

    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let attachments: Vec<AttachmentRow> = if ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as(
            "SELECT message_id, filename, s3_key, sha256, size
             FROM message_attachment_refs
             WHERE tenant_id = $1 AND message_id = ANY($2)
             ORDER BY message_id, ordinal",
        )
        .bind(tenant_id)
        .bind(&ids)
        .fetch_all(&mut *tx)
        .await?
    };
    tx.commit().await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let msg_attachments = attachments
                .iter()
                .filter(|att| att.message_id == row.id)
                .map(|att| AttachmentRef {
                    filename: att.filename.clone(),
                    s3_key: att.s3_key.clone(),
                    sha256: att.sha256.clone(),
                    size: att.size,
                })
                .collect();
            DsarMessage {
                id: row.id,
                tenant_id: row.tenant_id,
                author_subject_id: row.author_subject_id,
                participant_subject_ids: row.participant_subject_ids,
                from: row.from_address,
                to: row.to_addresses,
                cc: row.cc_addresses,
                subject: row.subject,
                body_text: None,
                body_html: None,
                attachments: msg_attachments,
                memory_audit_chain_hash: row.memory_audit_chain_hash.or(Some(row.body_sha256_hex)),
            }
        })
        .collect())
}

pub async fn complete_export_job(
    pool: &PgPool,
    tenant_id: Uuid,
    job_id: Uuid,
    output_jsonl_s3_key: &str,
    output_sha256: &str,
    summary: &DsarExportSummary,
) -> Result<DsarExportJobRow, sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let row: DsarExportJobRow = sqlx::query_as(
        "UPDATE dsar_export_jobs
         SET status = 'completed',
             output_jsonl_s3_key = $3,
             output_sha256 = $4,
             message_count = $5,
             attachment_count = $6,
             completed_at = now(),
             started_at = COALESCE(started_at, now())
         WHERE tenant_id = $1 AND id = $2
         RETURNING *",
    )
    .bind(tenant_id)
    .bind(job_id)
    .bind(output_jsonl_s3_key)
    .bind(output_sha256)
    .bind(summary.message_count as i64)
    .bind(summary.attachment_count as i64)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn fail_export_job(
    pool: &PgPool,
    tenant_id: Uuid,
    job_id: Uuid,
    error_code: &str,
) -> Result<DsarExportJobRow, sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let row: DsarExportJobRow = sqlx::query_as(
        "UPDATE dsar_export_jobs
         SET status = 'failed',
             error_code = $3,
             completed_at = now(),
             started_at = COALESCE(started_at, now())
         WHERE tenant_id = $1 AND id = $2
         RETURNING *",
    )
    .bind(tenant_id)
    .bind(job_id)
    .bind(error_code)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

async fn set_tenant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub fn aggregate_jsonl(
    tenant_id: Uuid,
    subject_id: Uuid,
    messages: &[DsarMessage],
) -> Result<DsarExportSummary, serde_json::Error> {
    let mut jsonl = String::new();
    let mut message_count = 0usize;
    let mut attachment_count = 0usize;

    for msg in messages.iter().filter(|m| {
        m.tenant_id == tenant_id
            && (m.author_subject_id == Some(subject_id)
                || m.participant_subject_ids.contains(&subject_id))
    }) {
        message_count += 1;
        attachment_count += msg.attachments.len();
        jsonl.push_str(&serde_json::to_string(msg)?);
        jsonl.push('\n');
    }

    Ok(DsarExportSummary {
        tenant_id,
        subject_id,
        message_count,
        attachment_count,
        jsonl,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct DsarAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
    pub message_count: Option<usize>,
    pub attachment_count: Option<usize>,
    pub trace_id: Option<String>,
}

pub fn audit_started(tenant_id: Uuid, subject_id: Uuid, trace_id: Option<&str>) -> DsarAuditRow {
    DsarAuditRow {
        kind: "email.dsar_export_started",
        tenant_id,
        subject_id,
        message_count: None,
        attachment_count: None,
        trace_id: trace_id.map(str::to_owned),
    }
}

pub fn audit_completed(summary: &DsarExportSummary, trace_id: Option<&str>) -> DsarAuditRow {
    DsarAuditRow {
        kind: "email.dsar_export_completed",
        tenant_id: summary.tenant_id,
        subject_id: summary.subject_id,
        message_count: Some(summary.message_count),
        attachment_count: Some(summary.attachment_count),
        trace_id: trace_id.map(str::to_owned),
    }
}

pub fn audit_failed(tenant_id: Uuid, subject_id: Uuid, trace_id: Option<&str>) -> DsarAuditRow {
    DsarAuditRow {
        kind: "email.dsar_export_failed",
        tenant_id,
        subject_id,
        message_count: None,
        attachment_count: None,
        trace_id: trace_id.map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(
        tenant_id: Uuid,
        author: Option<Uuid>,
        participants: Vec<Uuid>,
        attachments: usize,
    ) -> DsarMessage {
        DsarMessage {
            id: Uuid::new_v4(),
            tenant_id,
            author_subject_id: author,
            participant_subject_ids: participants,
            from: "a@example.com".into(),
            to: vec!["b@example.com".into()],
            cc: vec![],
            subject: Some("hello".into()),
            body_text: Some("body".into()),
            body_html: None,
            attachments: (0..attachments)
                .map(|i| AttachmentRef {
                    filename: format!("{i}.txt"),
                    s3_key: format!("s3://bucket/{i}.txt"),
                    sha256: "0".repeat(64),
                    size: 10,
                })
                .collect(),
            memory_audit_chain_hash: Some("a".repeat(64)),
        }
    }

    #[test]
    fn exports_authored_and_received_messages() {
        let tenant = Uuid::new_v4();
        let subject = Uuid::new_v4();
        let messages = vec![
            msg(tenant, Some(subject), vec![], 1),
            msg(tenant, None, vec![subject], 2),
            msg(tenant, None, vec![Uuid::new_v4()], 0),
        ];
        let summary = aggregate_jsonl(tenant, subject, &messages).unwrap();
        assert_eq!(summary.message_count, 2);
        assert_eq!(summary.attachment_count, 3);
        assert_eq!(summary.jsonl.lines().count(), 2);
    }

    #[test]
    fn cross_tenant_messages_are_excluded() {
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let subject = Uuid::new_v4();
        let messages = vec![
            msg(tenant_a, Some(subject), vec![], 0),
            msg(tenant_b, Some(subject), vec![], 0),
        ];
        let summary = aggregate_jsonl(tenant_a, subject, &messages).unwrap();
        assert_eq!(summary.message_count, 1);
    }

    #[test]
    fn attachments_are_references_not_inline_bytes() {
        let tenant = Uuid::new_v4();
        let subject = Uuid::new_v4();
        let summary =
            aggregate_jsonl(tenant, subject, &[msg(tenant, Some(subject), vec![], 1)]).unwrap();
        assert!(summary.jsonl.contains("s3://bucket/0.txt"));
        assert!(!summary.jsonl.contains("attachment_bytes"));
    }

    #[test]
    fn completed_audit_carries_counts_only() {
        let tenant = Uuid::new_v4();
        let subject = Uuid::new_v4();
        let summary =
            aggregate_jsonl(tenant, subject, &[msg(tenant, Some(subject), vec![], 1)]).unwrap();
        let row = audit_completed(&summary, Some("trace"));
        assert_eq!(row.kind, "email.dsar_export_completed");
        assert_eq!(row.message_count, Some(1));
        assert_eq!(row.attachment_count, Some(1));
    }
}
