//! FR-EMAIL-011 — DSAR JSONL export primitives.

use serde::{Deserialize, Serialize};
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
