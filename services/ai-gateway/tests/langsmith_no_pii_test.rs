//! FR-OBS-004 — exported LangSmith payloads must contain redacted text only.

use cyberos_ai_gateway::langsmith::{
    build_payload, LangSmithMetadata, RedactedPrompt, RedactedResponse,
};

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";

#[test]
fn exported_payload_contains_no_raw_pii_patterns() {
    let payload = build_payload(
        TRACE_ID,
        RedactedPrompt(
            "email <EMAIL_ADDRESS_1> phone <PHONE_NUMBER_1> cccd <VN_CCCD_1>".to_string(),
        ),
        RedactedResponse("ok".to_string()),
        metadata(),
    );
    let encoded = serde_json::to_string(&payload).unwrap();

    assert!(!encoded.contains("alice@example.com"));
    assert!(!encoded.contains("0901234567"));
    assert!(!encoded.contains("031234567678"));
    assert!(encoded.contains("<EMAIL_ADDRESS_1>"));
    assert!(encoded.contains("<PHONE_NUMBER_1>"));
    assert!(encoded.contains("<VN_CCCD_1>"));
}

fn metadata() -> LangSmithMetadata {
    LangSmithMetadata {
        model_alias: "chat.smart".to_string(),
        resolved_model: "claude-3-5-sonnet".to_string(),
        provider: "anthropic".to_string(),
        temperature: Some(0.2),
        max_tokens: Some(100),
        latency_ms: 42,
        cost_usd: 0.0078,
        persona_handle: "cuo-cpo@0.4.1".to_string(),
        tenant_id: "org:test".to_string(),
        trace_id: TRACE_ID.to_string(),
        tool_calls: vec![],
    }
}
