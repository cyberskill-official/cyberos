//! TASK-OBS-004 — exported payloads must not contain raw PII patterns (AC6).

use cyberos_ai_gateway::langsmith::{build_payload, LangSmithMetadata, RedactedPrompt, RedactedResponse};
use regex::Regex;

fn meta() -> LangSmithMetadata {
    LangSmithMetadata {
        model_alias: "chat.smart".into(),
        resolved_model: "claude-3-5-sonnet".into(),
        provider: "anthropic".into(),
        temperature: None,
        max_tokens: None,
        latency_ms: 1,
        cost_usd: 0.0,
        persona_handle: String::new(),
        tenant_id: "t1".into(),
        trace_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        tool_calls: vec![],
    }
}

#[test]
fn exported_payload_contains_no_pii_patterns() {
    let prompt = RedactedPrompt("hello <VN_CCCD_1> world <VN_PHONE_1>".into());
    let response = RedactedResponse("ok for <EMAIL_1>".into());
    let payload = build_payload(prompt, response, meta());
    let json = serde_json::to_string(&payload).unwrap();

    let cccd_re = Regex::new(r"\b\d{12}\b").unwrap();
    let phone_re = Regex::new(r"\b0\d{9}\b").unwrap();
    let email_re =
        Regex::new(r"\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b").unwrap();
    assert!(!cccd_re.is_match(&json), "raw CCCD leaked: {json}");
    assert!(!phone_re.is_match(&json), "raw phone leaked: {json}");
    assert!(!email_re.is_match(&json), "raw email leaked: {json}");
    assert!(json.contains("<VN_CCCD_1>"));
    assert!(json.contains("<VN_PHONE_1>"));
    assert!(json.contains("<EMAIL_1>"));
}
