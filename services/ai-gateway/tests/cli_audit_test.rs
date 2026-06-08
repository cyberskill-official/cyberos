use cyberos_ai_gateway::memory_writer::{canonical, AiInvocationKind, MemoryEmit};
use serde_json::{json, Value};

#[test]
fn cli_audit_row_kinds_are_canonicalised_with_required_metadata() {
    let cases = [
        (AiInvocationKind::CliPolicyUpdated, "ai.cli_policy_updated"),
        (AiInvocationKind::CliFailoverDrill, "ai.cli_failover_drill"),
        (
            AiInvocationKind::CliInvoiceExported,
            "ai.cli_invoice_exported",
        ),
        (AiInvocationKind::CliBreakerReset, "ai.cli_breaker_reset"),
        (
            AiInvocationKind::CliExpiryRepaired,
            "ai.cli_expiry_repaired",
        ),
        (AiInvocationKind::CliMemoryEmitted, "ai.cli_memory_emitted"),
    ];

    for (kind, tag) in cases {
        let req = MemoryEmit {
            kind,
            path: format!(
                "memories/decisions/ai-cli/test/{}.md",
                tag.replace('.', "-")
            ),
            extra: json!({
                "operator_id": "ops@cyberos.world",
                "command": "test command",
                "args": {"tenant": "org:test"},
                "request_id": "018f7d27-3db9-7db8-9c72-4f7a7f2c0000",
                "command_sha256": "a".repeat(64),
                "outcome": "confirmed"
            }),
        };

        let serialised = canonical::serialise(&req).unwrap();
        let parsed: Value = serde_json::from_str(&serialised).unwrap();
        assert_eq!(parsed["meta"]["kind"], tag);
        assert_eq!(parsed["meta"]["extra"]["operator_id"], "ops@cyberos.world");
        assert_eq!(parsed["meta"]["extra"]["command"], "test command");
        assert_eq!(
            parsed["meta"]["extra"]["request_id"],
            "018f7d27-3db9-7db8-9c72-4f7a7f2c0000"
        );
        assert_eq!(
            parsed["meta"]["extra"]["command_sha256"]
                .as_str()
                .unwrap()
                .len(),
            64
        );
        assert!(parsed["body"].as_str().unwrap().contains(tag));
    }
}
