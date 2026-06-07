//! FR-AI-006 §5 — Integration tests for alias resolution.

use std::collections::HashMap;

use cyberos_ai_gateway::alias::{self, AliasError, LatencyClass, ResolvedModel};
use cyberos_ai_gateway::cost_table;
use cyberos_ai_gateway::policy::*;

// ─── Test helpers ─────────────────────────────────────────────────────────────

fn init_cost_table_for_tests() {
    let fixture_path = std::path::PathBuf::from("tests/fixtures/cost_table/valid_rates.yaml");
    // Ignore AlreadyInitialised errors (other tests may have init'd first)
    let _ = futures::executor::block_on(cost_table::init_cost_table(&fixture_path));
}

fn test_policy_with_bedrock_primary() -> TenantPolicy {
    init_cost_table_for_tests();

    let mut model_alias_map = HashMap::new();
    model_alias_map.insert(
        "chat.smart".into(),
        "anthropic.claude-3-5-sonnet-20241022-v2:0".into(),
    );
    model_alias_map.insert(
        "chat.fast".into(),
        "anthropic.claude-3-haiku-20240307-v1:0".into(),
    );
    model_alias_map.insert(
        "embed.standard".into(),
        "amazon.titan-embed-text-v2:0".into(),
    );

    TenantPolicy {
        tenant_id: "org:test".into(),
        ai_policy: AiPolicy {
            monthly_cap_usd: rust_decimal_macros::dec!(100.00),
            warn_threshold: 0.80,
            hard_stop: true,
            primary_provider: Provider::Bedrock {
                region: "ap-southeast-1".into(),
                model_alias_map,
            },
            fallback_chain: vec![],
            call_timeout_seconds: 60,
            residency: Residency::Sg1,
            zdr_required: false,
            emergency_override: EmergencyOverride::default(),
            allowed_personas: None,
            alias_overrides: None,
            residency_requires_regional_provider: None,
            pii_redaction_extra: None,
        },
    }
}

// ─── AC #1: Happy path — primary ──────────────────────────────────────────────

#[test]
fn resolves_chat_smart_to_bedrock_via_primary() {
    let policy = test_policy_with_bedrock_primary();
    let r = alias::resolve("chat.smart", &policy).unwrap();
    assert_eq!(r.provider_kind, ProviderKind::Bedrock);
    assert_eq!(r.model, "anthropic.claude-3-5-sonnet-20241022-v2:0");
    assert_eq!(r.region, Some("ap-southeast-1".to_string()));
    assert_eq!(r.fallback_position, 0);
    assert!(r.is_zdr);
    assert_eq!(r.latency_class, LatencyClass::Standard);
}

// ─── AC #2: Fallback resolution ───────────────────────────────────────────────

#[test]
fn falls_through_to_anthropic_for_chat_long() {
    let mut policy = test_policy_with_bedrock_primary();
    // Add Anthropic fallback with chat.long
    let mut anthropic_map = HashMap::new();
    anthropic_map.insert("chat.long".into(), "claude-3-5-sonnet-20241022".into());
    policy.ai_policy.fallback_chain.push(Provider::Anthropic {
        model_alias_map: anthropic_map,
    });

    let r = alias::resolve("chat.long", &policy).unwrap();
    assert_eq!(r.provider_kind, ProviderKind::Anthropic);
    assert_eq!(r.model, "claude-3-5-sonnet-20241022");
    assert_eq!(r.fallback_position, 1);
    assert_eq!(r.latency_class, LatencyClass::Slow);
}

// ─── AC #3: Override beats primary ────────────────────────────────────────────

#[test]
fn override_beats_primary() {
    let mut policy = test_policy_with_bedrock_primary();
    // Primary has chat.fast → claude-3-haiku
    // Override pins chat.fast → claude-3-5-sonnet (Anthropic native)
    let mut override_map = HashMap::new();
    let mut anthropic_map = HashMap::new();
    anthropic_map.insert("chat.fast".into(), "claude-3-5-sonnet-20241022".into());
    override_map.insert(
        "chat.fast".into(),
        OverrideTarget {
            provider: Provider::Anthropic {
                model_alias_map: anthropic_map,
            },
        },
    );
    policy.ai_policy.alias_overrides = Some(override_map);

    let r = alias::resolve("chat.fast", &policy).unwrap();
    assert_eq!(r.provider_kind, ProviderKind::Anthropic);
    assert_eq!(r.model, "claude-3-5-sonnet-20241022");
    assert_eq!(r.fallback_position, 0); // override always reports as position 0
}

// ─── AC #4: Unknown alias ─────────────────────────────────────────────────────

#[test]
fn unknown_alias_errors_with_supported_list() {
    let policy = test_policy_with_bedrock_primary();
    let err = alias::resolve("chat.tiny", &policy).unwrap_err();
    match err {
        AliasError::UnknownAlias { alias, supported } => {
            assert_eq!(alias, "chat.tiny");
            assert_eq!(supported.len(), 6);
            assert!(supported.contains(&"chat.smart".to_string()));
        }
        _ => panic!("expected UnknownAlias, got {:?}", err),
    }
}

// ─── AC #7: Cost-table validation ─────────────────────────────────────────────

#[test]
fn cost_table_missing_errors() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.primary_provider = Provider::Bedrock {
        region: "ap-southeast-1".into(),
        model_alias_map: {
            let mut m = HashMap::new();
            m.insert("chat.smart".into(), "fake-model-not-in-cost-table".into());
            m
        },
    };

    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(
        err,
        AliasError::ResolvedModelMissingCostEntry { .. }
    ));
}

// ─── AC #8: No provider has alias ─────────────────────────────────────────────

#[test]
fn empty_fallback_returns_no_provider() {
    let mut policy = test_policy_with_bedrock_primary();
    // Remove chat.long from primary (it's not there by default)
    // And don't add any fallbacks
    let err = alias::resolve("chat.long", &policy).unwrap_err();
    assert!(matches!(
        err,
        AliasError::NoProviderHasAlias {
            providers_tried: 1,
            ..
        }
    ));
}

// ─── AC #15: Empty fallback chain ─────────────────────────────────────────────

#[test]
fn empty_fallback_chain_with_primary_miss() {
    let mut policy = test_policy_with_bedrock_primary();
    // Primary doesn't have embed.code
    policy.ai_policy.fallback_chain.clear();

    let err = alias::resolve("embed.code", &policy).unwrap_err();
    assert!(matches!(
        err,
        AliasError::NoProviderHasAlias {
            providers_tried: 1,
            ..
        }
    ));
}

// ─── Latency class mapping ────────────────────────────────────────────────────

#[test]
fn latency_classes_correct() {
    let policy = test_policy_with_bedrock_primary();

    // chat.smart → Standard
    let r = alias::resolve("chat.smart", &policy).unwrap();
    assert_eq!(r.latency_class, LatencyClass::Standard);

    // chat.fast → Fast
    let r = alias::resolve("chat.fast", &policy).unwrap();
    assert_eq!(r.latency_class, LatencyClass::Fast);

    // embed.standard → Fast
    let r = alias::resolve("embed.standard", &policy).unwrap();
    assert_eq!(r.latency_class, LatencyClass::Fast);
}

// ─── supported_aliases ────────────────────────────────────────────────────────

#[test]
fn supported_aliases_returns_all_six() {
    let aliases = alias::supported_aliases();
    assert_eq!(aliases.len(), 6);
    assert!(aliases.contains(&"chat.smart"));
    assert!(aliases.contains(&"chat.fast"));
    assert!(aliases.contains(&"chat.long"));
    assert!(aliases.contains(&"embed.standard"));
    assert!(aliases.contains(&"embed.code"));
    assert!(aliases.contains(&"rerank.fast"));
}
