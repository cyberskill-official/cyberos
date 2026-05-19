//! FR-EMAIL-001 §1 #7 + §1 #8 — message + thread metadata writers.

use crate::errors::EmailResult;
use crate::types::{EmailMessage, EmailThread, MessageDirection, MessageStatus};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Upsert a thread row + bump the running count. Idempotent on `thread_id`.
pub async fn upsert_thread(
    db: &PgPool,
    tenant_id: Uuid,
    thread_id: &str,
    subject_normalised: Option<&str>,
    last_message_at: DateTime<Utc>,
    new_participants: &[String],
) -> EmailResult<()> {
    sqlx::query(
        "INSERT INTO thread_metadata
            (thread_id, tenant_id, subject_normalised, last_message_at, message_count, participant_addresses)
         VALUES ($1, $2, $3, $4, 1, $5)
         ON CONFLICT (thread_id) DO UPDATE
         SET last_message_at = EXCLUDED.last_message_at,
             message_count   = thread_metadata.message_count + 1,
             participant_addresses = (
                 SELECT ARRAY(SELECT DISTINCT unnest(thread_metadata.participant_addresses || EXCLUDED.participant_addresses))
             )",
    )
    .bind(thread_id)
    .bind(tenant_id)
    .bind(subject_normalised)
    .bind(last_message_at)
    .bind(new_participants)
    .execute(db)
    .await?;
    Ok(())
}

/// Insert a metadata row. Append-only: each status transition writes a
/// new row carrying `prior_message_id`.
#[allow(clippy::too_many_arguments)]
pub async fn insert_message(
    db: &PgPool,
    msg: &NewMessage<'_>,
) -> EmailResult<EmailMessage> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let row: EmailMessage = sqlx::query_as(
        "INSERT INTO message_metadata
            (id, tenant_id, stalwart_message_id, thread_id, direction, from_address,
             to_addresses, cc_addresses, bcc_addresses, subject, received_at,
             s3_body_key, s3_body_kms_key_id, body_sha256_hex, byte_size, status,
             prior_message_id, spam_score, dkim_pass, spf_pass, dmarc_pass, bimi_present,
             created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                 $17, $18, $19, $20, $21, $22, $23)
         RETURNING *",
    )
    .bind(id)
    .bind(msg.tenant_id)
    .bind(msg.stalwart_message_id)
    .bind(msg.thread_id)
    .bind(msg.direction)
    .bind(msg.from_address)
    .bind(msg.to_addresses)
    .bind(msg.cc_addresses)
    .bind(msg.bcc_addresses)
    .bind(msg.subject)
    .bind(msg.received_at)
    .bind(msg.s3_body_key)
    .bind(msg.s3_body_kms_key_id)
    .bind(msg.body_sha256_hex)
    .bind(msg.byte_size)
    .bind(msg.status)
    .bind(msg.prior_message_id)
    .bind(msg.spam_score)
    .bind(msg.dkim_pass)
    .bind(msg.spf_pass)
    .bind(msg.dmarc_pass)
    .bind(msg.bimi_present)
    .bind(now)
    .fetch_one(db)
    .await?;
    Ok(row)
}

pub struct NewMessage<'a> {
    pub tenant_id: Uuid,
    pub stalwart_message_id: i64,
    pub thread_id: &'a str,
    pub direction: MessageDirection,
    pub from_address: &'a str,
    pub to_addresses: &'a [String],
    pub cc_addresses: &'a [String],
    pub bcc_addresses: &'a [String],
    pub subject: Option<&'a str>,
    pub received_at: DateTime<Utc>,
    pub s3_body_key: &'a str,
    pub s3_body_kms_key_id: &'a str,
    pub body_sha256_hex: &'a str,
    pub byte_size: i64,
    pub status: MessageStatus,
    pub prior_message_id: Option<Uuid>,
    pub spam_score: Option<f32>,
    pub dkim_pass: Option<bool>,
    pub spf_pass: Option<bool>,
    pub dmarc_pass: Option<bool>,
    pub bimi_present: Option<bool>,
}

/// FR-EMAIL-001 §1 #19 list handler — cursored list by (tenant, received_at).
pub async fn list_messages(
    db: &PgPool,
    tenant_id: Uuid,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: i64,
) -> EmailResult<Vec<EmailMessage>> {
    let rows: Vec<EmailMessage> = sqlx::query_as(
        "SELECT * FROM message_metadata
         WHERE tenant_id = $1
           AND ($2::timestamptz IS NULL OR received_at >= $2)
           AND ($3::timestamptz IS NULL OR received_at <= $3)
         ORDER BY received_at DESC
         LIMIT $4",
    )
    .bind(tenant_id)
    .bind(from)
    .bind(to)
    .bind(limit.clamp(1, 1000))
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// Normalise a subject per RFC 5322 — strip leading `Re: ` / `Fwd: ` /
/// `Fw: ` (case-insensitive) and collapse whitespace.
pub fn normalise_subject(subject: Option<&str>) -> Option<String> {
    let s = subject?.trim();
    if s.is_empty() {
        return None;
    }
    let mut work = s.to_owned();
    loop {
        let lower = work.to_lowercase();
        if lower.starts_with("re: ") || lower.starts_with("fw: ") {
            work = work[4..].trim_start().to_owned();
        } else if lower.starts_with("fwd: ") {
            work = work[5..].trim_start().to_owned();
        } else {
            break;
        }
    }
    // Edge case: an input like "Re: " was trimmed to "Re:" before the loop
    // ran, so the bare prefix never matched. Treat any residual bare-
    // prefix tail (with no following content) as an empty subject.
    let lower = work.to_lowercase();
    if matches!(lower.as_str(), "re:" | "fw:" | "fwd:") {
        return None;
    }
    let collapsed: String = work
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if collapsed.is_empty() { None } else { Some(collapsed) }
}

/// Fetch a single thread row.
pub async fn get_thread(db: &PgPool, thread_id: &str) -> EmailResult<Option<EmailThread>> {
    let row = sqlx::query_as::<_, EmailThread>(
        "SELECT * FROM thread_metadata WHERE thread_id = $1",
    )
    .bind(thread_id)
    .fetch_optional(db)
    .await?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::normalise_subject;

    #[test]
    fn normalises_re_fwd_prefixes() {
        assert_eq!(normalise_subject(Some("Re: foo")).unwrap(), "foo");
        assert_eq!(normalise_subject(Some("Re: Re: bar")).unwrap(), "bar");
        assert_eq!(normalise_subject(Some("Fwd: baz")).unwrap(), "baz");
        assert_eq!(normalise_subject(Some("FW: qux")).unwrap(), "qux");
    }

    #[test]
    fn collapses_whitespace() {
        assert_eq!(normalise_subject(Some("  foo    bar  ")).unwrap(), "foo bar");
    }

    #[test]
    fn empty_returns_none() {
        assert_eq!(normalise_subject(None), None);
        assert_eq!(normalise_subject(Some("")), None);
        assert_eq!(normalise_subject(Some("Re: ")), None);
    }
}
