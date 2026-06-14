//! LangSmith payload construction for FR-OBS-004.

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::policy::ProviderKind;
use crate::router::{ChatCompleteRequest, Choice, ProviderResponse, ProviderUsage};

pub const MAX_REDACTED_BYTES: usize = 100 * 1024;
pub const TRUNCATION_MARKER: &str = "...[truncated by FR-OBS-004]";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RedactedPrompt(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RedactedResponse(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LangSmithMetadata {
    pub model_alias: String,
    pub resolved_model: String,
    pub provider: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub latency_ms: u32,
    pub cost_usd: f64,
    pub persona_handle: String,
    pub tenant_id: String,
    pub trace_id: String,
    pub tool_calls: Vec<ToolCallTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCallTrace {
    pub tool_name: String,
    pub redacted_args: RedactedPrompt,
    pub outcome: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LangSmithPayload {
    pub trace_id: String,
    pub prompt: String,
    pub response: String,
    pub metadata: LangSmithMetadata,
}

pub fn build_payload(
    trace_id: &str,
    redacted_prompt: RedactedPrompt,
    redacted_response: RedactedResponse,
    mut metadata: LangSmithMetadata,
) -> LangSmithPayload {
    let trace_id = trace_id.to_ascii_lowercase();
    metadata.trace_id = trace_id.clone();
    LangSmithPayload {
        trace_id,
        prompt: truncate_redacted(redacted_prompt.0),
        response: truncate_redacted(redacted_response.0),
        metadata,
    }
}

pub fn prompt_from_messages(messages: &[crate::router::Message]) -> RedactedPrompt {
    let mut out = String::new();
    for message in messages {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&message.role);
        out.push_str(": ");
        out.push_str(&message.content);
    }
    RedactedPrompt(out)
}

pub fn response_from_choices(choices: &[Choice]) -> RedactedResponse {
    let mut out = String::new();
    for choice in choices {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&choice.content);
    }
    RedactedResponse(out)
}

pub fn tool_calls_from_response(response: &ProviderResponse) -> Vec<ToolCallTrace> {
    response
        .choices
        .iter()
        .flat_map(|choice| choice.tool_calls.iter())
        .map(|call| ToolCallTrace {
            tool_name: call.name.clone(),
            redacted_args: RedactedPrompt(call.arguments.clone()),
            outcome: "success".to_string(),
        })
        .collect()
}

pub fn cost_usd_for_response(provider: ProviderKind, model: &str, usage: ProviderUsage) -> f64 {
    let Some(rate) = crate::cost_table::lookup(&provider, model) else {
        return 0.0;
    };
    let per_1k = Decimal::from(1000u32);
    let prompt = (Decimal::from(usage.prompt_tokens) / per_1k) * rate.input_per_1k_usd;
    let completion = (Decimal::from(usage.completion_tokens) / per_1k) * rate.output_per_1k_usd;
    (prompt + completion).to_f64().unwrap_or(0.0)
}

pub fn metadata_from_router_response(
    trace_id: &str,
    req: &ChatCompleteRequest,
    response: &ProviderResponse,
    provider: ProviderKind,
    model: &str,
    tenant_id: &str,
) -> LangSmithMetadata {
    LangSmithMetadata {
        model_alias: req.alias.clone(),
        resolved_model: model.to_string(),
        provider: provider.as_metric_label().to_string(),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        latency_ms: response.latency_ms,
        cost_usd: cost_usd_for_response(provider, model, response.usage),
        persona_handle: req
            .agent_persona
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        tenant_id: tenant_id.to_string(),
        trace_id: trace_id.to_ascii_lowercase(),
        tool_calls: tool_calls_from_response(response),
    }
}

pub fn is_w3c_trace_id(value: &str) -> bool {
    value.len() == 32
        && value.chars().all(|ch| ch.is_ascii_hexdigit())
        && !value.chars().all(|ch| ch == '0')
}

pub fn truncate_redacted(value: String) -> String {
    if value.len() <= MAX_REDACTED_BYTES {
        return value;
    }
    let mut end = 0;
    for (idx, ch) in value.char_indices() {
        let next = idx + ch.len_utf8();
        if next > MAX_REDACTED_BYTES {
            break;
        }
        end = next;
    }
    format!("{}{}", &value[..end], TRUNCATION_MARKER)
}
