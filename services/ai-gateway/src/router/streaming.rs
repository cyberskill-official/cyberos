//! FR-AI-010 — Provider SSE normalization.

use futures::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::{FinishReason, ProviderStreamResponse, RouterError};
use crate::policy::ProviderKind;
use crate::streaming::{ProviderStreamEvent, ProviderStreamUsage};

const PROVIDER_EVENT_CHANNEL_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy)]
pub(crate) enum StreamDialect {
    OpenAi,
    Anthropic,
    Bedrock,
}

#[derive(Debug, Default)]
struct ParserState {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    cached_input_tokens: u32,
    usage_emitted: bool,
    done_emitted: bool,
}

#[derive(Debug)]
struct SseFrame {
    event: Option<String>,
    data: String,
}

pub(crate) fn response_to_provider_stream(
    response: reqwest::Response,
    provider: ProviderKind,
    dialect: StreamDialect,
) -> ProviderStreamResponse {
    let (tx, rx) =
        mpsc::channel::<Result<ProviderStreamEvent, RouterError>>(PROVIDER_EVENT_CHANNEL_CAPACITY);

    tokio::spawn(async move {
        let mut bytes = response.bytes_stream();
        let mut pending = String::new();
        let mut state = ParserState::default();

        while let Some(next) = bytes.next().await {
            match next {
                Ok(chunk) => {
                    let chunk = match std::str::from_utf8(&chunk) {
                        Ok(chunk) => chunk,
                        Err(err) => {
                            let _ = tx
                                .send(Err(RouterError::InvalidResponse {
                                    reason: format!(
                                        "{} streaming chunk was not utf-8: {err}",
                                        provider.as_metric_label()
                                    ),
                                }))
                                .await;
                            return;
                        }
                    };
                    pending.push_str(chunk);

                    while let Some((raw_frame, consumed)) = take_frame(&pending) {
                        let raw_frame = raw_frame.to_string();
                        pending.drain(..consumed);
                        if let Some(frame) = parse_sse_frame(&raw_frame) {
                            match parse_frame(&mut state, dialect, &frame) {
                                Ok(events) => {
                                    for event in events {
                                        if tx.send(Ok(event)).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                                Err(err) => {
                                    let _ = tx.send(Err(err)).await;
                                    return;
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    let _ = tx
                        .send(Err(RouterError::TerminalProviderError {
                            provider,
                            status: 503,
                            message: format!("provider streaming body error: {err}"),
                            retry_after_secs: None,
                        }))
                        .await;
                    return;
                }
            }
        }

        if !pending.trim().is_empty() {
            if let Some(frame) = parse_sse_frame(pending.trim_end()) {
                match parse_frame(&mut state, dialect, &frame) {
                    Ok(events) => {
                        for event in events {
                            if tx.send(Ok(event)).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err)).await;
                    }
                }
            }
        }
    });

    ProviderStreamResponse::new(ReceiverStream::new(rx))
}

fn take_frame(input: &str) -> Option<(&str, usize)> {
    let lf = input.find("\n\n").map(|idx| (idx, 2));
    let crlf = input.find("\r\n\r\n").map(|idx| (idx, 4));
    match (lf, crlf) {
        (Some((lf_idx, lf_len)), Some((crlf_idx, crlf_len))) => {
            if lf_idx < crlf_idx {
                Some((&input[..lf_idx], lf_idx + lf_len))
            } else {
                Some((&input[..crlf_idx], crlf_idx + crlf_len))
            }
        }
        (Some((idx, len)), None) | (None, Some((idx, len))) => Some((&input[..idx], idx + len)),
        (None, None) => None,
    }
}

fn parse_sse_frame(raw: &str) -> Option<SseFrame> {
    let mut event = None;
    let mut data_lines = Vec::new();

    for line in raw.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        if let Some(value) = line.strip_prefix("event:") {
            event = Some(value.trim_start().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.trim_start().to_string());
        }
    }

    if data_lines.is_empty() {
        return None;
    }

    Some(SseFrame {
        event,
        data: data_lines.join("\n"),
    })
}

fn parse_frame(
    state: &mut ParserState,
    dialect: StreamDialect,
    frame: &SseFrame,
) -> Result<Vec<ProviderStreamEvent>, RouterError> {
    if frame.data.trim() == "[DONE]" {
        return Ok(done_once(state, FinishReason::Stop));
    }

    let value: Value =
        serde_json::from_str(&frame.data).map_err(|err| RouterError::InvalidResponse {
            reason: format!("streaming SSE JSON parse failed: {err}"),
        })?;

    match dialect {
        StreamDialect::OpenAi => parse_openai_frame(state, &value),
        StreamDialect::Anthropic | StreamDialect::Bedrock => {
            parse_anthropic_family_frame(state, frame.event.as_deref(), &value)
        }
    }
}

fn parse_openai_frame(
    state: &mut ParserState,
    value: &Value,
) -> Result<Vec<ProviderStreamEvent>, RouterError> {
    let mut events = Vec::new();

    if let Some(usage) = value.get("usage").filter(|usage| !usage.is_null()) {
        state.prompt_tokens = usage_u32(usage, "prompt_tokens");
        state.completion_tokens = usage_u32(usage, "completion_tokens");
        state.cached_input_tokens = usage
            .get("prompt_tokens_details")
            .and_then(|details| usage_u32(details, "cached_tokens"))
            .or_else(|| usage_u32(usage, "cached_input_tokens"))
            .unwrap_or(0);
        emit_usage_once(state, &mut events);
    }

    if let Some(choices) = value.get("choices").and_then(Value::as_array) {
        for choice in choices {
            if let Some(text) = choice
                .get("delta")
                .and_then(|delta| delta.get("content"))
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
            {
                events.push(ProviderStreamEvent::Token {
                    text: text.to_string(),
                });
            }
            if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
                events.extend(done_once(
                    state,
                    super::normalize::finish_reason_from_provider(Some(reason)),
                ));
            }
        }
    }

    Ok(events)
}

fn parse_anthropic_family_frame(
    state: &mut ParserState,
    event_name: Option<&str>,
    value: &Value,
) -> Result<Vec<ProviderStreamEvent>, RouterError> {
    let mut events = Vec::new();
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .or(event_name)
        .unwrap_or_default();

    match kind {
        "message_start" => {
            if let Some(usage) = value
                .get("message")
                .and_then(|message| message.get("usage"))
                .or_else(|| value.get("usage"))
            {
                state.prompt_tokens = usage_u32(usage, "input_tokens");
                state.cached_input_tokens =
                    usage_u32(usage, "cache_read_input_tokens").unwrap_or(0);
            }
        }
        "content_block_delta" => {
            if let Some(text) = value
                .get("delta")
                .and_then(|delta| delta.get("text"))
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
                .or_else(|| value.get("text").and_then(Value::as_str))
            {
                events.push(ProviderStreamEvent::Token {
                    text: text.to_string(),
                });
            }
        }
        "message_delta" => {
            if let Some(usage) = value.get("usage") {
                state.completion_tokens = usage_u32(usage, "output_tokens");
                emit_usage_once(state, &mut events);
            }
            if let Some(reason) = value
                .get("delta")
                .and_then(|delta| delta.get("stop_reason"))
                .and_then(Value::as_str)
            {
                events.extend(done_once(
                    state,
                    super::normalize::finish_reason_from_provider(Some(reason)),
                ));
            }
        }
        "message_stop" => {
            emit_usage_once(state, &mut events);
            events.extend(done_once(state, FinishReason::Stop));
        }
        _ => {
            if let Some(text) = value
                .get("delta")
                .and_then(|delta| delta.get("text"))
                .and_then(Value::as_str)
                .or_else(|| value.get("text").and_then(Value::as_str))
                .filter(|text| !text.is_empty())
            {
                events.push(ProviderStreamEvent::Token {
                    text: text.to_string(),
                });
            }
        }
    }

    Ok(events)
}

fn emit_usage_once(state: &mut ParserState, events: &mut Vec<ProviderStreamEvent>) {
    if state.usage_emitted {
        return;
    }
    let Some(prompt_tokens) = state.prompt_tokens else {
        return;
    };
    let Some(completion_tokens) = state.completion_tokens else {
        return;
    };
    state.usage_emitted = true;
    events.push(ProviderStreamEvent::Usage(ProviderStreamUsage {
        prompt_tokens,
        completion_tokens,
        cached_input_tokens: state.cached_input_tokens,
    }));
}

fn done_once(state: &mut ParserState, reason: FinishReason) -> Vec<ProviderStreamEvent> {
    if state.done_emitted {
        Vec::new()
    } else {
        state.done_emitted = true;
        vec![ProviderStreamEvent::Done(reason)]
    }
}

fn usage_u32(value: &Value, field: &str) -> Option<u32> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|raw| u32::try_from(raw).ok())
}
