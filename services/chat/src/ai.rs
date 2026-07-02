//! AI-native chat actions (AI cluster): channel summarize ("catch me up"), action-item extraction, and
//! smart reply suggestions. Same posture as translate.rs: the caller must be a channel member, chat reads
//! the recent messages ITSELF and calls the ai-gateway server-side (the transcript only ever leaves chat to
//! the one configured gateway, never to the browser's network), and when the gateway is unset or down every
//! endpoint degrades to a clean 502 without ever blocking the chat path. Speaker labels come from a
//! client-supplied subject->name map (the client owns the directory; chat's DB has no names) and are used
//! only inside the prompt. Audit events carry counts, never content.

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::translate::{call_gateway, gateway_url, GwChatRequest, GwMessage};
use crate::{audit, auth, db, AppState};

#[derive(Debug, Deserialize)]
pub struct AiRequest {
    /// How many recent messages to consider (summarize/actions; replies uses its own small window).
    #[serde(default)]
    pub limit: Option<i64>,
    /// subject_id -> display name, used only to label transcript speakers. Unknown senders fall back to a
    /// short id, so a stale map degrades gracefully.
    #[serde(default)]
    pub names: HashMap<Uuid, String>,
}

#[derive(Debug, Serialize)]
pub struct AiTextResponse {
    /// Markdown (bullet) text, rendered by the client's rich-text view.
    pub text: String,
    /// How many messages went into the prompt (shown as "based on the last N messages").
    pub message_count: usize,
}

#[derive(Debug, Serialize)]
pub struct AiRepliesResponse {
    pub suggestions: Vec<String>,
}

/// Keep prompts bounded for the local models behind the gateway: newest lines win.
const MAX_TRANSCRIPT_BYTES: usize = 24 * 1024;
const UNAVAILABLE: &str = "AI is unavailable right now";

/// Label a speaker from the client-supplied map (newlines and colons stripped so a crafted display name
/// cannot fake extra transcript lines), falling back to a short id.
fn speaker(names: &HashMap<Uuid, String>, id: Uuid) -> String {
    match names.get(&id) {
        Some(n) => {
            let clean: String = n
                .replace(['\n', '\r', ':'], " ")
                .trim()
                .chars()
                .take(40)
                .collect();
            if clean.is_empty() {
                id.to_string()[..8].to_string()
            } else {
                clean
            }
        }
        None => id.to_string()[..8].to_string(),
    }
}

/// Oldest-first "Name: body" lines, capped at MAX_TRANSCRIPT_BYTES by dropping the OLDEST lines. Pure, so
/// the shape is unit-testable.
pub fn format_transcript(rows: &[(Uuid, String)], names: &HashMap<Uuid, String>) -> String {
    let lines: Vec<String> = rows
        .iter()
        .map(|(sender, body)| format!("{}: {}", speaker(names, *sender), body.trim()))
        .collect();
    let mut start = 0;
    let mut total: usize = lines.iter().map(|l| l.len() + 1).sum();
    while total > MAX_TRANSCRIPT_BYTES && start < lines.len() {
        total -= lines[start].len() + 1;
        start += 1;
    }
    lines[start..].join("\n")
}

/// The last `limit` non-empty messages of a channel (member-gated), oldest first.
async fn load_rows(
    st: &AppState,
    headers: &HeaderMap,
    channel: Uuid,
    limit: i64,
) -> Result<(Uuid, Uuid, Vec<(Uuid, String)>), (StatusCode, String)> {
    let claims = auth::authenticate(st, headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    if db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT sender_subject_id, body FROM chat_messages
         WHERE channel_id = $1 AND deleted_at IS NULL AND body <> ''
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind(channel)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;
    Ok((tenant, subject, rows.into_iter().rev().collect()))
}

fn no_messages() -> (StatusCode, String) {
    (
        StatusCode::BAD_REQUEST,
        "nothing to work with yet - this conversation has no messages".to_string(),
    )
}

/// POST /v1/chat/channels/{id}/ai/summarize - "catch me up": a short bullet summary of the recent
/// conversation, in the transcript's dominant language (this team is bilingual VN/EN).
pub async fn summarize(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<AiRequest>,
) -> Result<Json<AiTextResponse>, (StatusCode, String)> {
    let limit = body.limit.unwrap_or(100).clamp(10, 200);
    let (tenant, subject, rows) = load_rows(&st, &headers, channel, limit).await?;
    if rows.is_empty() {
        return Err(no_messages());
    }
    let transcript = format_transcript(&rows, &body.names);
    let system = "You summarize a team chat conversation for a bilingual Vietnamese/English software team. \
                  Write 3-8 short markdown bullet points ('- ') covering the decisions made, open questions, \
                  and important updates. Answer in the transcript's dominant language. Output only the \
                  bullets, no preamble.";
    let url = gateway_url(UNAVAILABLE)?;
    let req = GwChatRequest {
        alias: "chat.smart".to_string(),
        messages: vec![
            GwMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            GwMessage {
                role: "user".to_string(),
                content: transcript,
            },
        ],
    };
    let text = call_gateway(&url, &tenant, &req, UNAVAILABLE).await?;
    audit::emit(
        &st,
        tenant,
        subject,
        "chat.ai_summarize",
        serde_json::json!({"channel_id": channel, "messages": rows.len()}),
    )
    .await;
    Ok(Json(AiTextResponse {
        text,
        message_count: rows.len(),
    }))
}

/// POST /v1/chat/channels/{id}/ai/actions - extract concrete action items from the recent conversation.
pub async fn actions(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<AiRequest>,
) -> Result<Json<AiTextResponse>, (StatusCode, String)> {
    let limit = body.limit.unwrap_or(100).clamp(10, 200);
    let (tenant, subject, rows) = load_rows(&st, &headers, channel, limit).await?;
    if rows.is_empty() {
        return Err(no_messages());
    }
    let transcript = format_transcript(&rows, &body.names);
    let system = "Extract the concrete action items from this team chat transcript. Output one markdown \
                  bullet per item ('- '), naming the owner when the conversation states one ('- Name: task') \
                  and keeping each item under 20 words, in the transcript's dominant language (Vietnamese or \
                  English). If there are no action items, output exactly: (none)";
    let url = gateway_url(UNAVAILABLE)?;
    let req = GwChatRequest {
        alias: "chat.smart".to_string(),
        messages: vec![
            GwMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            GwMessage {
                role: "user".to_string(),
                content: transcript,
            },
        ],
    };
    let text = call_gateway(&url, &tenant, &req, UNAVAILABLE).await?;
    audit::emit(
        &st,
        tenant,
        subject,
        "chat.ai_actions",
        serde_json::json!({"channel_id": channel, "messages": rows.len()}),
    )
    .await;
    Ok(Json(AiTextResponse {
        text,
        message_count: rows.len(),
    }))
}

/// Split the model's reply-suggestion output into up to 3 clean suggestions (one per line; leading list
/// markers and wrapping quotes stripped). Pure, unit-tested.
pub fn parse_suggestions(raw: &str) -> Vec<String> {
    raw.lines()
        .map(|l| {
            l.trim()
                .trim_start_matches(['-', '*', '•'])
                .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ')')
                .trim()
                .trim_matches('"')
                .to_string()
        })
        .filter(|l| !l.is_empty())
        .take(3)
        .collect()
}

/// POST /v1/chat/channels/{id}/ai/replies - up to 3 short reply suggestions for the current conversation,
/// matched to its language and tone. Uses a small window and the fast alias so it feels instant.
pub async fn replies(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<AiRequest>,
) -> Result<Json<AiRepliesResponse>, (StatusCode, String)> {
    let (tenant, subject, rows) = load_rows(&st, &headers, channel, 12).await?;
    if rows.is_empty() {
        return Err(no_messages());
    }
    let transcript = format_transcript(&rows, &body.names);
    let my_name = speaker(&body.names, subject);
    let system = "Suggest 3 short, natural replies the user could send next in this team chat. Match the \
                  conversation's language (Vietnamese or English) and tone, keep each reply under 15 words, \
                  and make the three replies meaningfully different. Output exactly 3 lines, one reply per \
                  line, with no numbering, bullets, or quotes.";
    let url = gateway_url(UNAVAILABLE)?;
    let req = GwChatRequest {
        alias: "chat.fast".to_string(),
        messages: vec![
            GwMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            GwMessage {
                role: "user".to_string(),
                content: format!("{transcript}\n\n(The person replying is: {my_name})"),
            },
        ],
    };
    let raw = call_gateway(&url, &tenant, &req, UNAVAILABLE).await?;
    let suggestions = parse_suggestions(&raw);
    if suggestions.is_empty() {
        return Err((StatusCode::BAD_GATEWAY, UNAVAILABLE.to_string()));
    }
    audit::emit(
        &st,
        tenant,
        subject,
        "chat.ai_replies",
        serde_json::json!({"channel_id": channel, "messages": rows.len(), "suggestions": suggestions.len()}),
    )
    .await;
    Ok(Json(AiRepliesResponse { suggestions }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(map: &[(Uuid, &str)]) -> HashMap<Uuid, String> {
        map.iter().map(|(k, v)| (*k, v.to_string())).collect()
    }

    #[test]
    fn transcript_labels_known_speakers_and_falls_back_to_short_ids() {
        let a = Uuid::from_bytes([1; 16]);
        let b = Uuid::from_bytes([2; 16]);
        let names = n(&[(a, "Anna Vu")]);
        let t = format_transcript(
            &[(a, "chao em".to_string()), (b, "hi anh".to_string())],
            &names,
        );
        assert_eq!(
            t,
            format!("Anna Vu: chao em\n{}: hi anh", &b.to_string()[..8])
        );
    }

    #[test]
    fn transcript_strips_newlines_and_colons_from_names() {
        let a = Uuid::from_bytes([3; 16]);
        let names = n(&[(a, "Evil:\nName: x")]);
        let t = format_transcript(&[(a, "hello".to_string())], &names);
        assert!(t.starts_with("Evil  Name  x: hello"), "got: {t}");
        assert_eq!(t.lines().count(), 1, "a crafted name cannot add lines");
    }

    #[test]
    fn transcript_caps_bytes_by_dropping_oldest() {
        let a = Uuid::from_bytes([4; 16]);
        let long = "x".repeat(1000);
        let rows: Vec<(Uuid, String)> = (0..40).map(|i| (a, format!("{i} {long}"))).collect();
        let t = format_transcript(&rows, &HashMap::new());
        assert!(t.len() <= MAX_TRANSCRIPT_BYTES);
        assert!(t.contains("39 "), "the newest line survives");
        assert!(!t.contains("\n0 "), "the oldest lines are dropped");
    }

    #[test]
    fn suggestions_parse_and_strip_markers() {
        let raw = "1. Ok, em lam ngay\n- \"Sounds good!\"\n\n* Để mai họp nhé\n4th line ignored? no - capped";
        let s = parse_suggestions(raw);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], "Ok, em lam ngay");
        assert_eq!(s[1], "Sounds good!");
        assert_eq!(s[2], "Để mai họp nhé");
    }

    #[test]
    fn suggestions_empty_output_yields_none() {
        assert!(parse_suggestions("  \n \n").is_empty());
    }
}
