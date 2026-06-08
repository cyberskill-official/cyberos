use cyberos_ai_gateway::cli::json_schemas;
use serde_json::json;

#[test]
fn usage_output_validates_against_v1_schema() {
    let value = json!({
        "schema_version": "v1",
        "tenant": "org:test",
        "month": "2026-05",
        "cap_usd": 200.0,
        "spent_usd": 12.5,
        "spent_pct": 6.25,
        "calls": 17,
        "top_models_by_spend": [
            {"model": "claude-3-5-sonnet", "spend_usd": 12.5, "calls": 17}
        ]
    });
    json_schemas::validate_output("usage", "v1", &value).unwrap();
}

#[test]
fn representative_cli_outputs_validate_against_v1_schemas() {
    json_schemas::validate_output(
        "models",
        "v1",
        &json!({
            "schema_version": "v1",
            "models": [{"provider": "bedrock", "model": "claude", "alias": "chat.smart"}]
        }),
    )
    .unwrap();
    json_schemas::validate_output(
        "pricing",
        "v1",
        &json!({
            "schema_version": "v1",
            "pricing": [{"provider": "bedrock", "model": "claude", "input_per_1k": 0.003, "output_per_1k": 0.015}]
        }),
    )
    .unwrap();
    json_schemas::validate_output(
        "breaker",
        "v1",
        &json!({
            "schema_version": "v1",
            "breakers": [{"provider": "bedrock", "model": "claude", "state": "Closed", "failures": 0, "next_half_open": ""}]
        }),
    )
    .unwrap();
    json_schemas::validate_output(
        "policy-diff",
        "v1",
        &json!({
            "schema_version": "v1",
            "tenant": "org:test",
            "changes": [{"field": "cap_usd", "before": "150", "after": "200"}]
        }),
    )
    .unwrap();
    json_schemas::validate_output(
        "invoice",
        "v1",
        &json!({
            "schema_version": "v1",
            "tenant": "org:test",
            "period": "2026-05",
            "total_usd": 12.5,
            "rows": [{"date": "2026-05-01", "model": "claude", "calls": 3, "cost_usd": 12.5}]
        }),
    )
    .unwrap();
    json_schemas::validate_output(
        "expiry-status",
        "v1",
        &json!({
            "schema_version": "v1",
            "pending_holds": 1,
            "stale_expired": 0
        }),
    )
    .unwrap();
    json_schemas::validate_output(
        "memory-audit-trail",
        "v1",
        &json!({
            "schema_version": "v1",
            "rows": [{"seq": 1, "timestamp": "2026-05-01T00:00:00Z", "kind": "ai.precheck", "payload_brief": "tenant_id=org:test"}]
        }),
    )
    .unwrap();
}
