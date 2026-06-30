//! Inline translation (FR-CHAT-101): POST /v1/chat/translate {text, target_lang}. The caller is authenticated
//! and the request is rate-bounded by the usual auth path; chat then calls the ai-gateway SERVER-SIDE (so the
//! message text only ever leaves chat to the one configured gateway, never to the browser's network) with a
//! translation prompt and the `chat.fast` alias. The gateway is not in the prod stack yet, so this MUST fail
//! gracefully: if AI_GATEWAY_URL is unset or the call fails, return a clean 502 with a short message - it must
//! never crash the service and never block the chat path. Emits an additive `chat.message_translated` audit.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, AppState};

#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub target_lang: String,
}

#[derive(Debug, Serialize)]
pub struct TranslateResponse {
    pub translated: String,
}

/// A {role, content} pair, the shape the ai-gateway `/v1/chat` body expects under `messages`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GwMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct GwChatRequest {
    alias: String,
    messages: Vec<GwMessage>,
}

#[derive(Debug, Deserialize)]
struct GwChatResponse {
    content: String,
}

/// The longest text we will translate in one call - a chat message, not a document. Keeps the gateway prompt
/// bounded and the cost predictable.
const MAX_TEXT_BYTES: usize = 8 * 1024;

/// Build the gateway chat messages for a translation: a system instruction to translate into `target_lang`
/// preserving meaning, then the user's text. Pure (no I/O) so the prompt shape is unit-testable.
pub fn build_messages(text: &str, target_lang: &str) -> Vec<GwMessage> {
    let system = format!(
        "You are a translation engine. Translate the user's message into {target_lang}, preserving meaning, \
         tone, names, and formatting. Output only the translation, with no quotes, labels, or commentary."
    );
    vec![
        GwMessage {
            role: "system".to_string(),
            content: system,
        },
        GwMessage {
            role: "user".to_string(),
            content: text.to_string(),
        },
    ]
}

pub async fn translate(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<TranslateRequest>,
) -> Result<Json<TranslateResponse>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let text = body.text.trim().to_string();
    if text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text is required".to_string()));
    }
    if text.len() > MAX_TEXT_BYTES {
        return Err((
            StatusCode::BAD_REQUEST,
            "text is too long to translate".to_string(),
        ));
    }
    let target = body.target_lang.trim().to_string();
    if target.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "target_lang is required".to_string(),
        ));
    }

    // The gateway is optional infrastructure: when AI_GATEWAY_URL is unset/blank, translation is simply
    // unavailable (a clean 502), never a crash. An empty value (compose default) is treated as unset.
    let base = std::env::var("AI_GATEWAY_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
        .ok_or((
            StatusCode::BAD_GATEWAY,
            "translation is unavailable (ai-gateway not configured)".to_string(),
        ))?;
    let root = base.trim_end_matches('/');
    let url = format!("{root}/v1/chat");
    let req = GwChatRequest {
        alias: "chat.fast".to_string(),
        messages: build_messages(&text, &target),
    };
    let translated = call_gateway(&url, &tenant, &req).await?;

    audit::emit(
        &st,
        tenant,
        subject,
        "chat.message_translated",
        serde_json::json!({"target_lang": target, "chars": text.chars().count()}),
    )
    .await;
    Ok(Json(TranslateResponse { translated }))
}

/// Do the gateway round-trip, mapping every failure (transport, non-2xx, malformed body, empty content) to a
/// clean 502 so the handler never panics and the chat path is never blocked.
async fn call_gateway(
    url: &str,
    tenant: &Uuid,
    req: &GwChatRequest,
) -> Result<String, (StatusCode, String)> {
    let unavailable = || {
        (
            StatusCode::BAD_GATEWAY,
            "translation is unavailable right now".to_string(),
        )
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("x-tenant-id", tenant.to_string())
        .json(req)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(target: "cyberos_chat::translate", error = %e, "ai-gateway request failed");
            unavailable()
        })?;
    if !resp.status().is_success() {
        tracing::warn!(target: "cyberos_chat::translate", status = %resp.status(), "ai-gateway returned non-success");
        return Err(unavailable());
    }
    let parsed: GwChatResponse = resp.json().await.map_err(|e| {
        tracing::warn!(target: "cyberos_chat::translate", error = %e, "ai-gateway response was not the expected shape");
        unavailable()
    })?;
    let translated = parsed.content.trim().to_string();
    if translated.is_empty() {
        return Err(unavailable());
    }
    Ok(translated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_messages_has_system_then_user_and_names_target() {
        let msgs = build_messages("Xin chào", "English");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "system");
        assert!(
            msgs[0].content.contains("English"),
            "system prompt names the target language"
        );
        assert!(
            msgs[0].content.contains("only the translation"),
            "system prompt forbids commentary"
        );
        assert_eq!(msgs[1].role, "user");
        assert_eq!(msgs[1].content, "Xin chào", "user content is the raw text");
    }

    #[test]
    fn build_messages_passes_text_through_verbatim() {
        // The text must reach the model unmodified (no trimming/escaping here; the handler trims once).
        let msgs = build_messages("  spaced  ", "Vietnamese");
        assert_eq!(msgs[1].content, "  spaced  ");
    }
}
