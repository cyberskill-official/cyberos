//! In-app content reporting (TASK-CHAT-267). A member reports a message, an attachment, or a person; the
//! service records it and does nothing else. It does not hide, delete, or flag the content — deciding what
//! happens is TASK-CHAT-269's job and a human's decision (§1 #13).
//!
//! Two properties carry the whole design:
//!
//! * **The snapshot is the evidence (§1 #4).** The reported content is copied into `chat_reports` inside the
//!   same transaction that inserts the report, under a `FOR SHARE` lock on the source row. The sender can
//!   edit or soft-delete the message afterwards; the snapshot does not move. Without it the obvious abuse is
//!   post-abuse, wait-for-report, delete — and the moderation queue shows an empty row.
//!
//! * **The endpoint is not an oracle (§1 #5, #6).** A second report against the same target by the same
//!   reporter returns `200` with the first report's id, not `409` — a distinct response for "already
//!   reported" would let anyone probe whether a message has an outstanding report. For the same reason the
//!   rate-limit check runs BEFORE any target lookup (§1 #7): otherwise a rate-limited caller could tell a
//!   real message id (404) from a fake one.

use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

/// §1 #7. Far above any honest reporter's rate; the point is to make report-spam-as-harassment uneconomic,
/// not to ration legitimate use.
const REPORT_RATE_LIMIT_PER_HOUR: i64 = 20;
/// §1 #2.
const DETAIL_MAX_CHARS: usize = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Message,
    Attachment,
    Subject,
}

impl TargetKind {
    /// Explicit wire label. Never Debug-format an enum into a label: Debug output is not a stable wire
    /// format, and this string lands in the DB CHECK constraint and the audit row.
    pub fn as_label(&self) -> &'static str {
        match self {
            TargetKind::Message => "message",
            TargetKind::Attachment => "attachment",
            TargetKind::Subject => "subject",
        }
    }
}

/// Closed set (§1 #2). Closed because TASK-CHAT-269 groups and prioritises the queue by reason, and free text
/// cannot be grouped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Spam,
    Harassment,
    Hate,
    Sexual,
    Violence,
    SelfHarm,
    Illegal,
    Other,
}

impl Reason {
    pub fn as_label(&self) -> &'static str {
        match self {
            Reason::Spam => "spam",
            Reason::Harassment => "harassment",
            Reason::Hate => "hate",
            Reason::Sexual => "sexual",
            Reason::Violence => "violence",
            Reason::SelfHarm => "self_harm",
            Reason::Illegal => "illegal",
            Reason::Other => "other",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateReport {
    pub target_kind: TargetKind,
    #[serde(default)]
    pub target_message_id: Option<Uuid>,
    #[serde(default)]
    pub target_attachment_id: Option<Uuid>,
    #[serde(default)]
    pub target_subject_id: Option<Uuid>,
    pub reason: Reason,
    #[serde(default)]
    pub detail: Option<String>,
}

/// The ONLY thing the reporter gets back (§1 #5, #14). No status, no history, no count: the response must
/// not become an oracle for prior reports, and there is no user-facing report history in this slice.
#[derive(Debug, Serialize)]
pub struct ReportAccepted {
    pub id: Uuid,
}

/// What we copied out of the target at report time. For a subject there is nothing to snapshot: the target
/// IS the person.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Snapshot {
    channel_id: Option<Uuid>,
    body: Option<String>,
    filename: Option<String>,
    content_type: Option<String>,
    size_bytes: Option<i64>,
    sender_id: Option<Uuid>,
}

// ── Pure helpers. Kept free of the DB so they are unit-testable without Postgres (see `mod tests`). ──

/// §1 #2 / AC 14. Counted in chars, not bytes: a 1000-char Vietnamese detail is well over 1000 bytes and is
/// not what the cap is for.
fn validate_detail(detail: Option<&str>) -> Result<(), (StatusCode, String)> {
    if let Some(d) = detail {
        if d.chars().count() > DETAIL_MAX_CHARS {
            return Err((StatusCode::BAD_REQUEST, "detail is too long".to_string()));
        }
    }
    Ok(())
}

/// §1 (target shape) / AC 6, 13. The DB CHECK `chat_reports_target_shape` is the real guard; this is the
/// same rule in the handler so a confused client gets a 400 with a sentence rather than a constraint
/// violation. Returns the id of the one populated target column.
fn target_id(req: &CreateReport) -> Result<Uuid, (StatusCode, String)> {
    let shape_err = || {
        (
            StatusCode::BAD_REQUEST,
            "target shape does not match target_kind".to_string(),
        )
    };
    match req.target_kind {
        TargetKind::Message => {
            if req.target_attachment_id.is_some() || req.target_subject_id.is_some() {
                return Err(shape_err());
            }
            req.target_message_id.ok_or_else(shape_err)
        }
        TargetKind::Attachment => {
            if req.target_message_id.is_some() || req.target_subject_id.is_some() {
                return Err(shape_err());
            }
            req.target_attachment_id.ok_or_else(shape_err)
        }
        TargetKind::Subject => {
            if req.target_message_id.is_some() || req.target_attachment_id.is_some() {
                return Err(shape_err());
            }
            req.target_subject_id.ok_or_else(shape_err)
        }
    }
}

/// AC 6. A reporter cannot report themselves. Also enforced by `chat_reports_not_self` in the DB.
fn reject_self_report(req: &CreateReport, reporter: Uuid) -> Result<(), (StatusCode, String)> {
    if req.target_kind == TargetKind::Subject && req.target_subject_id == Some(reporter) {
        return Err((
            StatusCode::BAD_REQUEST,
            "cannot report yourself".to_string(),
        ));
    }
    Ok(())
}

// ── Handler ──

/// POST /v1/chat/reports — record a report. 201 on a new report, 200 when this reporter already has an open
/// report against this target (§1 #6). Never notifies the reported person (§1 #5).
pub async fn create(
    State(st): State<AppState>,
    headers: HeaderMap,
    // Taken as a Result so a body that fails to deserialize — an unknown `reason`, a malformed target id —
    // surfaces as 400 (AC 13). Axum's default `Json` rejection is 422, which the contract does not allow.
    body: Result<Json<CreateReport>, JsonRejection>,
) -> Result<(StatusCode, Json<ReportAccepted>), (StatusCode, String)> {
    // Authenticate BEFORE looking at the body, so an unauthenticated caller cannot use body validation as a
    // probe. `?` here yields the 401.
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let reporter = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let Json(req) = body.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "invalid report body: unknown reason, or malformed target".to_string(),
        )
    })?;

    validate_detail(req.detail.as_deref())?;
    reject_self_report(&req, reporter)?;
    let _ = target_id(&req)?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;

    // §1 #7 / AC 9 — the rate limit is the FIRST statement in the transaction, before any target lookup, so
    // a rate-limited caller cannot use the endpoint's 403/404 to probe which message ids exist. Counting
    // rows (one index scan over at most 20, on chat_reports_rate_idx) rather than an in-process token bucket:
    // the chat service is stateless and horizontally scaled, so a per-replica bucket would be N times more
    // permissive than advertised.
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM chat_reports
          WHERE reporter_subject_id = $1 AND created_at > now() - interval '1 hour'",
    )
    .bind(reporter)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    if recent >= REPORT_RATE_LIMIT_PER_HOUR {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "too many reports".to_string(),
        ));
    }

    // Resolve the target, enforce membership (§1 #3), and take the snapshot (§1 #4) inside the SAME
    // transaction that inserts, so the snapshot cannot race an edit or a delete.
    let snap = snapshot_target(&mut tx, tenant, reporter, &req).await?;

    // ON CONFLICT DO NOTHING, then read the existing row — deliberately NOT `DO UPDATE ... RETURNING`. The
    // upsert form would let a duplicate submission overwrite the original snapshot with a fresh one, which
    // re-opens exactly the edit-then-report race the snapshot exists to close. Do-nothing preserves the
    // first snapshot, which is the one that matters.
    let inserted: Option<(Uuid,)> = sqlx::query_as(
        "INSERT INTO chat_reports
            (tenant_id, reporter_subject_id, target_kind, target_message_id, target_attachment_id,
             target_subject_id, channel_id, reason, detail, snapshot_body, snapshot_filename,
             snapshot_content_type, snapshot_size_bytes, snapshot_sender_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
         ON CONFLICT DO NOTHING
         RETURNING id",
    )
    .bind(tenant)
    .bind(reporter)
    .bind(req.target_kind.as_label())
    .bind(req.target_message_id)
    .bind(req.target_attachment_id)
    .bind(req.target_subject_id)
    .bind(snap.channel_id)
    .bind(req.reason.as_label())
    .bind(req.detail.as_deref())
    .bind(snap.body.as_deref())
    .bind(snap.filename.as_deref())
    .bind(snap.content_type.as_deref())
    .bind(snap.size_bytes)
    .bind(snap.sender_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;

    // §1 #6 / AC 7 — the partial unique index fired: an open report by this reporter against this target
    // already exists. Return it with 200, NOT 409. Same shape either way; the caller learns nothing it did
    // not already know, and a double-tap is not an error.
    let (id, code) = match inserted {
        Some((id,)) => (id, StatusCode::CREATED),
        None => (
            existing_open_report(&mut tx, tenant, reporter, &req).await?,
            StatusCode::OK,
        ),
    };

    tx.commit().await.map_err(crate::internal)?;

    // §1 #8 / AC 10 — after commit, and WITHOUT the snapshot body or the detail. The audit chain is
    // hash-chained and replicated into the memory module: durable and hard to rewrite, which is exactly
    // right for "who did what when" and exactly wrong for a copy of content someone has asked us to
    // consider removing. The audit row records that a report happened; the report row holds the evidence,
    // under RLS, deletable when the report is resolved.
    audit::emit(
        &st,
        tenant,
        reporter,
        "chat.report_created",
        json!({
            "report_id":   id,
            "target_kind": req.target_kind.as_label(),
            "reason":      req.reason.as_label(),
            "channel_id":  snap.channel_id,
        }),
    )
    .await;

    // §1 #13 — no automated action on the content. We recorded. That is all.
    Ok((code, Json(ReportAccepted { id })))
}

/// Resolve the target, enforce membership, and copy the evidence out — all on the caller's transaction, so
/// the `FOR SHARE` locks below are held until the insert commits.
async fn snapshot_target(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant: Uuid,
    reporter: Uuid,
    req: &CreateReport,
) -> Result<Snapshot, (StatusCode, String)> {
    match req.target_kind {
        TargetKind::Message => {
            let id = target_id(req)?;
            // FOR SHARE holds the message row for the life of the tx, so a concurrent edit or delete cannot
            // land between this read and the insert. Not FOR UPDATE: we only read chat_messages, and
            // FOR UPDATE would serialise unrelated readers of a hot message row.
            //
            // Deliberately no `deleted_at IS NULL` filter: an already-soft-deleted message is still
            // reportable — the recipient has already read it, and that is precisely when evidence matters.
            let row: Option<(Uuid, Uuid, String)> = sqlx::query_as(
                "SELECT channel_id, sender_subject_id, body FROM chat_messages
                  WHERE id = $1 AND tenant_id = $2 FOR SHARE",
            )
            .bind(id)
            .bind(tenant)
            .fetch_optional(&mut **tx)
            .await
            .map_err(crate::internal)?;
            let (channel_id, sender, body) =
                row.ok_or((StatusCode::NOT_FOUND, "no such message".to_string()))?;
            require_member(tx, channel_id, reporter).await?;
            Ok(Snapshot {
                channel_id: Some(channel_id),
                body: Some(body),
                sender_id: Some(sender),
                ..Default::default()
            })
        }
        TargetKind::Attachment => {
            let id = target_id(req)?;
            let row: Option<(Uuid, Uuid, String, String, i64)> = sqlx::query_as(
                "SELECT channel_id, uploader_subject_id, filename, content_type, size_bytes
                   FROM chat_attachments
                  WHERE id = $1 AND tenant_id = $2 FOR SHARE",
            )
            .bind(id)
            .bind(tenant)
            .fetch_optional(&mut **tx)
            .await
            .map_err(crate::internal)?;
            let (channel_id, uploader, filename, content_type, size_bytes) =
                row.ok_or((StatusCode::NOT_FOUND, "no such attachment".to_string()))?;
            require_member(tx, channel_id, reporter).await?;
            // The bytes are NOT snapshotted — only the metadata. Copying the payload of a reported file into
            // a second table doubles the blast radius of the very content someone asked us to remove; the
            // attachment row itself is the evidence and TASK-CHAT-269 renders from this metadata.
            Ok(Snapshot {
                channel_id: Some(channel_id),
                filename: Some(filename),
                content_type: Some(content_type),
                size_bytes: Some(size_bytes),
                sender_id: Some(uploader),
                ..Default::default()
            })
        }
        TargetKind::Subject => {
            // §1 #3 — NO membership check, by design. The DM path exists: chat_channels.kind = 'direct'
            // lets any workspace member open a DM with any other. If reporting a person required
            // co-membership of a group channel, the one place harassment is most likely — a DM from someone
            // you do not work with — would be the one place you could not report it.
            //
            // No snapshot: the target IS the person, and people are not content.
            let _ = target_id(req)?;
            Ok(Snapshot::default())
        }
    }
}

/// §1 #3 → 403. A person who cannot see the content cannot report it.
async fn require_member(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    channel: Uuid,
    subject: Uuid,
) -> Result<(), (StatusCode, String)> {
    if db::role_in_channel(tx, channel, subject)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    Ok(())
}

/// The open report this reporter already has against this target — the row the partial unique index just
/// refused to duplicate. Matched on the same triple the index is built over.
async fn existing_open_report(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant: Uuid,
    reporter: Uuid,
    req: &CreateReport,
) -> Result<Uuid, (StatusCode, String)> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM chat_reports
          WHERE tenant_id = $1
            AND reporter_subject_id = $2
            AND target_kind = $3
            AND status = 'open'
            AND target_message_id    IS NOT DISTINCT FROM $4
            AND target_attachment_id IS NOT DISTINCT FROM $5
            AND target_subject_id    IS NOT DISTINCT FROM $6",
    )
    .bind(tenant)
    .bind(reporter)
    .bind(req.target_kind.as_label())
    .bind(req.target_message_id)
    .bind(req.target_attachment_id)
    .bind(req.target_subject_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(crate::internal)?;

    // If ON CONFLICT DO NOTHING suppressed the insert, the conflicting row is by definition there. Reaching
    // this arm without finding it means the insert was refused by something other than the open-report
    // index, which is a bug, not a duplicate — surface it rather than inventing a 200.
    row.map(|r| r.0).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "report insert conflicted but no open report found".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    fn req(kind: TargetKind) -> CreateReport {
        CreateReport {
            target_kind: kind,
            target_message_id: None,
            target_attachment_id: None,
            target_subject_id: None,
            reason: Reason::Spam,
            detail: None,
        }
    }

    #[test]
    fn reason_labels_are_the_wire_format_not_debug() {
        // These strings are pinned by the DB CHECK constraint and by the audit row. If one of them changes,
        // the constraint rejects the insert — so pin them here, where the failure is legible.
        assert_eq!(Reason::Spam.as_label(), "spam");
        assert_eq!(Reason::SelfHarm.as_label(), "self_harm"); // NOT "selfharm", NOT "SelfHarm"
        assert_eq!(Reason::Harassment.as_label(), "harassment");
        assert_eq!(Reason::Other.as_label(), "other");
        assert_eq!(TargetKind::Message.as_label(), "message");
        assert_eq!(TargetKind::Attachment.as_label(), "attachment");
        assert_eq!(TargetKind::Subject.as_label(), "subject");
    }

    #[test]
    fn reason_deserializes_from_snake_case_and_rejects_free_text() {
        let r: Reason = serde_json::from_str("\"self_harm\"").unwrap();
        assert_eq!(r, Reason::SelfHarm);
        // AC 13: the set is closed. "because" is not a reason.
        assert!(serde_json::from_str::<Reason>("\"because\"").is_err());
    }

    #[test]
    fn detail_cap_counts_chars_not_bytes() {
        assert!(validate_detail(None).is_ok());
        assert!(validate_detail(Some(&"x".repeat(DETAIL_MAX_CHARS))).is_ok());
        // AC 14: 1001 chars is a 400.
        assert!(validate_detail(Some(&"x".repeat(DETAIL_MAX_CHARS + 1))).is_err());
        // 1000 Vietnamese chars is >1000 bytes but is still 1000 characters — and must pass. A byte cap
        // here would silently make the field a third the size for a Vietnamese reporter.
        let vi = "ạ".repeat(DETAIL_MAX_CHARS);
        assert!(vi.len() > DETAIL_MAX_CHARS, "precondition: multi-byte");
        assert!(validate_detail(Some(&vi)).is_ok());
    }

    #[test]
    fn target_shape_must_match_target_kind() {
        let msg = uid(10);

        let mut r = req(TargetKind::Message);
        r.target_message_id = Some(msg);
        assert_eq!(target_id(&r).unwrap(), msg);

        // Right kind, no id.
        let r = req(TargetKind::Message);
        assert_eq!(target_id(&r).unwrap_err().0, StatusCode::BAD_REQUEST);

        // Kind says message, but a subject id came along too — AC 13 / the DB target_shape CHECK.
        let mut r = req(TargetKind::Message);
        r.target_message_id = Some(msg);
        r.target_subject_id = Some(uid(2));
        assert_eq!(target_id(&r).unwrap_err().0, StatusCode::BAD_REQUEST);

        // Kind says subject, but a message id came along.
        let mut r = req(TargetKind::Subject);
        r.target_subject_id = Some(uid(2));
        r.target_message_id = Some(msg);
        assert_eq!(target_id(&r).unwrap_err().0, StatusCode::BAD_REQUEST);

        let mut r = req(TargetKind::Attachment);
        r.target_attachment_id = Some(uid(7));
        assert_eq!(target_id(&r).unwrap(), uid(7));
    }

    #[test]
    fn self_report_is_refused() {
        let me = uid(1);
        // AC 6.
        let mut r = req(TargetKind::Subject);
        r.target_subject_id = Some(me);
        assert_eq!(
            reject_self_report(&r, me).unwrap_err().0,
            StatusCode::BAD_REQUEST
        );

        // Reporting someone else is fine.
        let mut r = req(TargetKind::Subject);
        r.target_subject_id = Some(uid(2));
        assert!(reject_self_report(&r, me).is_ok());

        // Reporting your own *message* is not a self-report — you may well want to flag something you
        // posted by mistake, and the not-self CHECK only covers the subject arm.
        let mut r = req(TargetKind::Message);
        r.target_message_id = Some(uid(10));
        assert!(reject_self_report(&r, me).is_ok());
    }

    #[test]
    fn accepted_response_carries_only_the_id() {
        // §1 #5 / #14: no status, no count, no reporter field. If someone adds one, this fails.
        let v = serde_json::to_value(ReportAccepted { id: uid(9) }).unwrap();
        let obj = v.as_object().expect("object");
        assert_eq!(obj.len(), 1, "response must carry exactly one field");
        assert!(obj.contains_key("id"));
    }
}
