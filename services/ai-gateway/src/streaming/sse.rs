//! FR-AI-010 §3 — SSE event serialization.
//!
//! Converts `StreamEvent` into axum `sse::Event` with the canonical wire format:
//!   event: token / usage / done / error / heartbeat
//!   data: {json}

use axum::response::sse::Event as SseEvent;

use super::StreamEvent;

impl StreamEvent {
    /// Convert to an axum SSE event.
    pub fn to_sse_event(&self) -> SseEvent {
        match self {
            StreamEvent::Token {
                text,
                model,
                index,
            } => SseEvent::default()
                .event("token")
                .json_data(serde_json::json!({
                    "text": text,
                    "model": model,
                    "index": index,
                }))
                .unwrap_or_else(|_| SseEvent::default().event("token").data("{}")),

            StreamEvent::Usage {
                prompt_tokens,
                completion_tokens,
                cached_input_tokens,
            } => SseEvent::default()
                .event("usage")
                .json_data(serde_json::json!({
                    "prompt_tokens": prompt_tokens,
                    "completion_tokens": completion_tokens,
                    "cached_input_tokens": cached_input_tokens,
                }))
                .unwrap_or_else(|_| SseEvent::default().event("usage").data("{}")),

            StreamEvent::Done { finish_reason } => SseEvent::default()
                .event("done")
                .json_data(serde_json::json!({
                    "finish_reason": finish_reason_label(finish_reason),
                }))
                .unwrap_or_else(|_| SseEvent::default().event("done").data("{}")),

            StreamEvent::Error { code, message } => SseEvent::default()
                .event("error")
                .json_data(serde_json::json!({
                    "code": code.as_metric_label(),
                    "message": message,
                }))
                .unwrap_or_else(|_| SseEvent::default().event("error").data("{}")),

            StreamEvent::Heartbeat => {
                SseEvent::default().event("heartbeat").data("{}")
            }
        }
    }
}

fn finish_reason_label(reason: &crate::router::FinishReason) -> &'static str {
    match reason {
        crate::router::FinishReason::Stop => "stop",
        crate::router::FinishReason::Length => "length",
        crate::router::FinishReason::ToolCalls => "tool_calls",
        crate::router::FinishReason::ContentFilter => "content_filter",
        crate::router::FinishReason::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_event_sse_does_not_panic() {
        let ev = StreamEvent::Token {
            text: "hello".into(),
            model: "test-model".into(),
            index: 0,
        };
        let _sse = ev.to_sse_event();
    }

    #[test]
    fn heartbeat_event_sse_does_not_panic() {
        let _sse = StreamEvent::Heartbeat.to_sse_event();
    }

    #[test]
    fn error_event_sse_does_not_panic() {
        let _sse = StreamEvent::Error {
            code: super::super::ErrorCode::ProviderDisconnect,
            message: "upstream closed".into(),
        }
        .to_sse_event();
    }
}
