//! Response normalization helpers for FR-AI-008 provider implementations.

use super::{Choice, FinishReason, ToolCall};

pub(crate) fn finish_reason_from_provider(value: Option<&str>) -> FinishReason {
    match value.unwrap_or_default() {
        "stop" | "end_turn" => FinishReason::Stop,
        "length" | "max_tokens" => FinishReason::Length,
        "tool_calls" | "tool_use" => FinishReason::ToolCalls,
        "content_filter" | "safety" => FinishReason::ContentFilter,
        _ => FinishReason::Other,
    }
}

pub(crate) fn text_choice(content: impl Into<String>, finish_reason: FinishReason) -> Choice {
    Choice {
        index: 0,
        content: content.into(),
        tool_calls: Vec::<ToolCall>::new(),
        finish_reason,
    }
}
