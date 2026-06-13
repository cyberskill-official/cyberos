//! FR-AI-022 §5 — OTel module tests.

use cyberos_ai_gateway::otel::attributes;

#[test]
fn attribute_keys_are_stable() {
    // Ensure attribute key constants don't accidentally change.
    assert_eq!(attributes::TENANT_ID, "ai_gateway.tenant_id");
    assert_eq!(attributes::MODEL_ALIAS, "ai_gateway.model_alias");
    assert_eq!(attributes::AGENT_PERSONA, "ai_gateway.agent_persona");
    assert_eq!(attributes::OUTCOME, "ai_gateway.outcome");
    assert_eq!(attributes::CACHE_STATE, "ai_gateway.cache_state");
    assert_eq!(attributes::REQUEST_ID, "ai_gateway.request_id");
    assert_eq!(attributes::PROVIDER, "ai_gateway.provider");
    assert_eq!(attributes::MODEL, "ai_gateway.model");
}

#[test]
fn attribute_keys_are_pii_safe() {
    // Verify no PII-bearing keys exist in the approved set.
    let all_keys = [
        attributes::TENANT_ID,
        attributes::MODEL_ALIAS,
        attributes::AGENT_PERSONA,
        attributes::IDEMPOTENCY_KEY,
        attributes::STREAM,
        attributes::OUTCOME,
        attributes::PROVIDER,
        attributes::MODEL,
        attributes::ATTEMPT_NUM,
        attributes::FALLBACK_POSITION,
        attributes::STATUS_CODE,
        attributes::RETRIED,
        attributes::PROMPT_TOKENS,
        attributes::COMPLETION_TOKENS,
        attributes::ESTIMATED_USD,
        attributes::ACTUAL_USD,
        attributes::CACHE_STATE,
        attributes::CACHE_KEY_HASH16,
        attributes::REQUEST_ID,
        attributes::REGION,
        attributes::SERVICE_NAME,
        attributes::SERVICE_VERSION,
        attributes::RETRY_ATTEMPT,
        attributes::RETRY_BACKOFF_MS,
        attributes::RETRY_PRIOR_STATUS,
    ];
    // All keys should be ASCII-only (no unicode PII sneaking in).
    for key in &all_keys {
        assert!(key.is_ascii(), "attribute key contains non-ASCII: {key}");
    }
}

#[test]
fn otel_span_names_are_documented() {
    let doc = std::fs::read_to_string("docs/span-names.md").expect("span names doc");
    for span_name in [
        cyberos_ai_gateway::otel::spans::CHAT_COMPLETION_SPAN,
        cyberos_ai_gateway::otel::spans::EMBED_SPAN,
        cyberos_ai_gateway::otel::spans::RERANK_SPAN,
        cyberos_ai_gateway::otel::spans::PRECHECK_SPAN,
        cyberos_ai_gateway::otel::spans::ALIAS_RESOLVE_SPAN,
        cyberos_ai_gateway::otel::spans::PERSONA_LOAD_SPAN,
        cyberos_ai_gateway::otel::spans::ZDR_CHECK_SPAN,
        cyberos_ai_gateway::otel::spans::RESIDENCY_CHECK_SPAN,
        cyberos_ai_gateway::otel::spans::CACHE_LOOKUP_SPAN,
        cyberos_ai_gateway::otel::spans::REDACT_SPAN,
        cyberos_ai_gateway::otel::spans::PROVIDER_CALL_SPAN,
        cyberos_ai_gateway::otel::spans::RECONCILE_SPAN,
    ] {
        assert!(
            doc.contains(span_name),
            "{span_name} missing from docs/span-names.md"
        );
    }
}
