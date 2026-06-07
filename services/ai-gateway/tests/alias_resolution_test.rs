//! FR-AI-006 §5 — Integration tests for alias resolution.

use std::collections::HashMap;
use std::sync::{Arc, Once};

use cyberos_ai_gateway::alias::{self, AliasError, LatencyClass};
use cyberos_ai_gateway::cost_table;
use cyberos_ai_gateway::policy::*;
use proptest::prelude::*;

// ─── Test helpers ─────────────────────────────────────────────────────────────

static COST_TABLE_INIT: Once = Once::new();

fn init_cost_table_for_tests() {
    COST_TABLE_INIT.call_once(|| {
        let fixture_path = std::path::PathBuf::from("tests/fixtures/cost_table/valid_rates.yaml");
        futures::executor::block_on(cost_table::init_cost_table(&fixture_path))
            .expect("valid cost-table fixture should load");
    });
}

fn model_map(entries: &[(&str, &str)]) -> HashMap<String, String> {
    entries
        .iter()
        .map(|(alias, model)| ((*alias).to_string(), (*model).to_string()))
        .collect()
}

fn bedrock_provider(region: &str, entries: &[(&str, &str)]) -> Provider {
    Provider::Bedrock {
        region: region.to_string(),
        model_alias_map: model_map(entries),
    }
}

fn anthropic_provider(entries: &[(&str, &str)]) -> Provider {
    Provider::Anthropic {
        model_alias_map: model_map(entries),
    }
}

fn openai_provider(entries: &[(&str, &str)]) -> Provider {
    Provider::Openai {
        model_alias_map: model_map(entries),
    }
}

fn counter_value(name: &str, labels: &[(&str, &str)]) -> f64 {
    prometheus::gather()
        .into_iter()
        .filter(|family| family.get_name() == name)
        .flat_map(|family| family.get_metric().to_vec())
        .find(|metric| {
            labels.iter().all(|(key, value)| {
                metric
                    .get_label()
                    .iter()
                    .any(|label| label.get_name() == *key && label.get_value() == *value)
            })
        })
        .map(|metric| metric.get_counter().get_value())
        .unwrap_or(0.0)
}

fn counter_sum(name: &str) -> f64 {
    prometheus::gather()
        .into_iter()
        .filter(|family| family.get_name() == name)
        .flat_map(|family| family.get_metric().to_vec())
        .map(|metric| metric.get_counter().get_value())
        .sum()
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
            pii_allowlist: None,
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
    let policy = test_policy_with_bedrock_primary();
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

// ─── AC #5: ZDR enforcement ──────────────────────────────────────────────────

#[test]
fn zdr_required_but_provider_not_zdr_errors() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.zdr_required = true;
    policy.ai_policy.primary_provider = openai_provider(&[("chat.smart", "gpt-4o")]);

    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    match err {
        AliasError::ZdrViolation {
            resolved_provider,
            resolved_model,
            attestation,
        } => {
            assert_eq!(resolved_provider, ProviderKind::Openai);
            assert_eq!(resolved_model, "gpt-4o");
            assert!(attestation.is_some());
        }
        _ => panic!("expected ZdrViolation, got {:?}", err),
    }
}

// ─── AC #6: Residency enforcement ────────────────────────────────────────────

#[test]
fn residency_pin_mismatch_errors() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.primary_provider = bedrock_provider(
        "us-east-1",
        &[("chat.smart", "anthropic.claude-3-5-sonnet-20241022-v2:0")],
    );

    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    match err {
        AliasError::ResidencyViolation {
            resolved_region,
            policy_residency,
            attempted_alias,
            vn1_no_provider,
        } => {
            assert_eq!(resolved_region, Some("us-east-1".to_string()));
            assert_eq!(policy_residency, Residency::Sg1);
            assert_eq!(attempted_alias, "chat.smart");
            assert!(!vn1_no_provider);
        }
        _ => panic!("expected ResidencyViolation, got {:?}", err),
    }
}

// ─── AC #8: No provider has alias after trying fallback chain ────────────────

#[test]
fn no_provider_has_alias_counts_primary_and_fallbacks() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.primary_provider = bedrock_provider(
        "ap-southeast-1",
        &[("chat.fast", "anthropic.claude-3-haiku-20240307-v1:0")],
    );
    policy.ai_policy.fallback_chain = vec![
        anthropic_provider(&[("chat.fast", "claude-3-haiku-20240307")]),
        openai_provider(&[("chat.fast", "gpt-4o")]),
    ];

    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(
        err,
        AliasError::NoProviderHasAlias {
            alias,
            providers_tried: 3
        } if alias == "chat.smart"
    ));
}

// ─── AC #9/#10/#10a: Override failures are terminal ─────────────────────────

#[test]
fn override_cost_table_miss_does_not_fall_through_to_primary() {
    let mut policy = test_policy_with_bedrock_primary();
    let mut overrides = HashMap::new();
    overrides.insert(
        "chat.smart".to_string(),
        OverrideTarget {
            provider: bedrock_provider("ap-southeast-1", &[("chat.smart", "missing-model")]),
        },
    );
    policy.ai_policy.alias_overrides = Some(overrides);

    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(
        err,
        AliasError::ResolvedModelMissingCostEntry {
            provider: ProviderKind::Bedrock,
            model
        } if model == "missing-model"
    ));
}

#[test]
fn override_zdr_violation_does_not_fall_through_to_primary() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.zdr_required = true;
    let mut overrides = HashMap::new();
    overrides.insert(
        "chat.smart".to_string(),
        OverrideTarget {
            provider: openai_provider(&[("chat.smart", "gpt-4o")]),
        },
    );
    policy.ai_policy.alias_overrides = Some(overrides);

    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(
        err,
        AliasError::ZdrViolation {
            resolved_provider: ProviderKind::Openai,
            resolved_model,
            ..
        } if resolved_model == "gpt-4o"
    ));
}

#[test]
fn override_residency_violation_does_not_fall_through_to_primary() {
    let mut policy = test_policy_with_bedrock_primary();
    let mut overrides = HashMap::new();
    overrides.insert(
        "chat.smart".to_string(),
        OverrideTarget {
            provider: bedrock_provider(
                "us-east-1",
                &[("chat.smart", "anthropic.claude-3-5-sonnet-20241022-v2:0")],
            ),
        },
    );
    policy.ai_policy.alias_overrides = Some(overrides);

    let err = alias::resolve("chat.smart", &policy).unwrap_err();
    assert!(matches!(
        err,
        AliasError::ResidencyViolation {
            resolved_region: Some(region),
            policy_residency: Residency::Sg1,
            attempted_alias,
            ..
        } if region == "us-east-1" && attempted_alias == "chat.smart"
    ));
}

// ─── AC #11: Determinism ─────────────────────────────────────────────────────

fn alias_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        proptest::sample::select(
            alias::supported_aliases()
                .iter()
                .map(|alias| (*alias).to_string())
                .collect::<Vec<_>>(),
        ),
        Just("chat.tiny".to_string()),
        Just("embed.experimental".to_string()),
    ]
}

fn provider_strategy() -> impl Strategy<Value = Provider> {
    prop_oneof![
        Just(bedrock_provider(
            "ap-southeast-1",
            &[
                ("chat.smart", "anthropic.claude-3-5-sonnet-20241022-v2:0"),
                ("chat.fast", "anthropic.claude-3-haiku-20240307-v1:0"),
                ("embed.standard", "amazon.titan-embed-text-v2:0"),
            ],
        )),
        Just(bedrock_provider(
            "us-east-1",
            &[("chat.smart", "anthropic.claude-3-5-sonnet-20241022-v2:0")],
        )),
        Just(anthropic_provider(&[
            ("chat.smart", "claude-3-5-sonnet-20241022"),
            ("chat.fast", "claude-3-haiku-20240307"),
            ("chat.long", "claude-3-5-sonnet-20241022"),
        ])),
        Just(openai_provider(&[
            ("chat.smart", "gpt-4o"),
            ("chat.fast", "gpt-4o"),
        ])),
    ]
}

fn residency_strategy() -> impl Strategy<Value = Residency> {
    prop_oneof![
        Just(Residency::Sg1),
        Just(Residency::Us1),
        Just(Residency::Eu1),
        Just(Residency::Vn1),
    ]
}

fn policy_strategy() -> impl Strategy<Value = TenantPolicy> {
    (
        provider_strategy(),
        proptest::collection::vec(provider_strategy(), 0..3),
        residency_strategy(),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(
            |(
                primary_provider,
                fallback_chain,
                residency,
                zdr_required,
                residency_requires_regional_provider,
            )| TenantPolicy {
                tenant_id: "org:prop".into(),
                ai_policy: AiPolicy {
                    monthly_cap_usd: rust_decimal_macros::dec!(100.00),
                    warn_threshold: 0.80,
                    hard_stop: true,
                    primary_provider,
                    fallback_chain,
                    call_timeout_seconds: 60,
                    residency,
                    zdr_required,
                    emergency_override: EmergencyOverride::default(),
                    allowed_personas: None,
                    alias_overrides: None,
                    residency_requires_regional_provider: Some(
                        residency_requires_regional_provider,
                    ),
                    pii_redaction_extra: None,
                    pii_allowlist: None,
                },
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10_000,
        .. ProptestConfig::default()
    })]

    #[test]
    fn determinism_property(alias in alias_strategy(), policy in policy_strategy()) {
        init_cost_table_for_tests();
        let expected = format!("{:?}", alias::resolve(&alias, &policy));
        for _ in 0..100 {
            prop_assert_eq!(format!("{:?}", alias::resolve(&alias, &policy)), expected.clone());
        }
    }
}

// ─── AC #13: Concurrent safety ───────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_resolve_matches_baseline() {
    let policy = Arc::new(test_policy_with_bedrock_primary());
    let expected = format!("{:?}", alias::resolve("chat.smart", &policy).unwrap());

    let mut handles = Vec::new();
    for _ in 0..16 {
        let policy = Arc::clone(&policy);
        let expected = expected.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..10_000 {
                assert_eq!(
                    format!("{:?}", alias::resolve("chat.smart", &policy).unwrap()),
                    expected
                );
            }
        }));
    }

    for handle in handles {
        handle.await.expect("resolve task should not panic");
    }
}

// ─── AC #14: OBS metrics emitted ─────────────────────────────────────────────

#[test]
fn success_metrics_increment_by_fallback_position_without_failures() {
    let mut policy = test_policy_with_bedrock_primary();
    policy.ai_policy.fallback_chain.push(anthropic_provider(&[(
        "chat.long",
        "claude-3-5-sonnet-20241022",
    )]));

    let primary_before = counter_value(
        "ai_alias_resolutions_total",
        &[
            ("alias", "chat.smart"),
            ("resolved_provider", "bedrock"),
            ("fallback_position", "0"),
        ],
    );
    let fallback_before = counter_value(
        "ai_alias_resolutions_total",
        &[
            ("alias", "chat.long"),
            ("resolved_provider", "anthropic"),
            ("fallback_position", "1"),
        ],
    );
    let failures_before = counter_sum("ai_alias_resolution_failures_total");

    for _ in 0..90 {
        alias::resolve("chat.smart", &policy).unwrap();
    }
    for _ in 0..10 {
        alias::resolve("chat.long", &policy).unwrap();
    }

    let primary_after = counter_value(
        "ai_alias_resolutions_total",
        &[
            ("alias", "chat.smart"),
            ("resolved_provider", "bedrock"),
            ("fallback_position", "0"),
        ],
    );
    let fallback_after = counter_value(
        "ai_alias_resolutions_total",
        &[
            ("alias", "chat.long"),
            ("resolved_provider", "anthropic"),
            ("fallback_position", "1"),
        ],
    );
    let failures_after = counter_sum("ai_alias_resolution_failures_total");

    assert_eq!(primary_after - primary_before, 90.0);
    assert_eq!(fallback_after - fallback_before, 10.0);
    assert_eq!(failures_after - failures_before, 0.0);
}

// ─── AC #16: Malformed override rejected by FR-AI-005 schema ────────────────

#[test]
fn malformed_override_missing_provider_rejected_by_policy_schema() {
    let yaml = r#"
tenant_id: org:test
ai_policy:
  monthly_cap_usd: "100.00"
  primary_provider:
    kind: bedrock
    region: ap-southeast-1
    model_alias_map:
      chat.smart: anthropic.claude-3-5-sonnet-20241022-v2:0
  residency: sg-1
  alias_overrides:
    chat.smart: {}
"#;

    let err = serde_yaml::from_str::<TenantPolicy>(yaml).unwrap_err();
    assert!(
        err.to_string().contains("missing field `provider`"),
        "unexpected error: {err}"
    );
}
