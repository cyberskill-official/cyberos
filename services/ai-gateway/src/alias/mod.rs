//! TASK-AI-006 — Model-alias resolution.
//!
//! Maps closed-set logical aliases (`chat.smart`, `chat.fast`, etc.) to concrete
//! `(provider, model)` tuples via tenant policy. Supports per-tenant overrides,
//! fallback chain, cost-table validation, ZDR checks, and residency enforcement.
//!
//! See TASK-AI-006 for normative behaviour and acceptance criteria.

pub mod types;

pub use types::{AliasError, LatencyClass, ResolvedModel};

use std::time::Instant;

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram, CounterVec, Histogram};

use crate::cost_table;
use crate::policy::{ProviderKind, Residency, TenantPolicy};
use crate::residency;
use crate::zdr;

/// Closed set of supported aliases for slice 2.
pub const SUPPORTED_ALIASES: &[&str] = &[
    "chat.smart",
    "chat.fast",
    "chat.long",
    "embed.standard",
    "embed.code",
    "rerank.fast",
];

/// Return the closed set of supported aliases.
pub fn supported_aliases() -> &'static [&'static str] {
    SUPPORTED_ALIASES
}

// ─── Metrics ──────────────────────────────────────────────────────────────────

static ALIAS_RESOLUTIONS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_alias_resolutions_total",
        "Successful alias resolutions by alias, provider, and fallback position",
        &["alias", "resolved_provider", "fallback_position"]
    )
    .unwrap()
});

static ALIAS_FAILURES: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_alias_resolution_failures_total",
        "Failed alias resolutions by alias and failure reason",
        &["alias", "reason"]
    )
    .unwrap()
});

static ALIAS_LATENCY_NS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "ai_alias_resolution_latency_ns",
        "Latency of alias::resolve calls in nanoseconds",
        vec![100.0, 500.0, 1_000.0, 5_000.0, 10_000.0]
    )
    .unwrap()
});

// ─── Public API ───────────────────────────────────────────────────────────────

/// Resolve a closed-set alias to a concrete (provider, model) tuple via tenant policy.
///
/// This is the only public entry point. Callers MUST NOT bypass this function.
pub fn resolve(alias: &str, policy: &TenantPolicy) -> Result<ResolvedModel, AliasError> {
    let started = Instant::now();

    // 1. Closed-set check — cheapest reject path
    if !SUPPORTED_ALIASES.contains(&alias) {
        ALIAS_FAILURES
            .with_label_values(&[alias, "unknown_alias"])
            .inc();
        return Err(AliasError::UnknownAlias {
            alias: alias.to_string(),
            supported: SUPPORTED_ALIASES.iter().map(|s| s.to_string()).collect(),
        });
    }

    // 2. Override path — beats primary + fallback
    if let Some(overrides) = &policy.ai_policy.alias_overrides {
        if let Some(override_target) = overrides.get(alias) {
            let r = check_and_build(alias, &override_target.provider, 0, policy)?;
            record_success(alias, &r, started);
            return Ok(r);
        }
    }

    // 3. Primary path
    let primary = &policy.ai_policy.primary_provider;
    if let Some(model) = primary.model_for_alias(alias) {
        let r = check_and_build_with_model(alias, primary, model, 0, policy)?;
        record_success(alias, &r, started);
        return Ok(r);
    }

    // 4. Fallback chain (in order)
    let mut providers_tried: u8 = 1; // primary counted
    for (idx, fb) in policy.ai_policy.fallback_chain.iter().enumerate() {
        providers_tried = providers_tried.saturating_add(1);
        if let Some(model) = fb.model_for_alias(alias) {
            let r = check_and_build_with_model(alias, fb, model, (idx + 1) as u8, policy)?;
            record_success(alias, &r, started);
            return Ok(r);
        }
    }

    ALIAS_FAILURES
        .with_label_values(&[alias, "no_provider_has_alias"])
        .inc();
    Err(AliasError::NoProviderHasAlias {
        alias: alias.to_string(),
        providers_tried,
    })
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn check_and_build(
    alias: &str,
    provider: &crate::policy::Provider,
    fallback_position: u8,
    policy: &TenantPolicy,
) -> Result<ResolvedModel, AliasError> {
    let model = provider.model_for_alias(alias).ok_or_else(|| {
        ALIAS_FAILURES
            .with_label_values(&[alias, "no_provider_has_alias"])
            .inc();
        AliasError::NoProviderHasAlias {
            alias: alias.to_string(),
            providers_tried: 1,
        }
    })?;
    check_and_build_with_model(alias, provider, model, fallback_position, policy)
}

fn check_and_build_with_model(
    alias: &str,
    provider: &crate::policy::Provider,
    model: &str,
    fallback_position: u8,
    policy: &TenantPolicy,
) -> Result<ResolvedModel, AliasError> {
    let kind = provider.kind();
    let region = provider.region();
    // TASK-AI-105: local providers (Ollama, LM Studio) run on the operator's own host, so they are
    // zero-cost (clause 7) and inherently zero-retention (clause 5). They are exempt from the cost-table
    // and ZDR gates that the cloud providers must pass. Residency still applies via the region check
    // below (local providers carry region None, so they pass unless the tenant requires a regional one).
    let is_local = matches!(kind, ProviderKind::Ollama | ProviderKind::LocalOpenai);

    // Cost-table check (TASK-AI-007) - skipped for local providers (zero cost).
    if !is_local && cost_table::lookup(&kind, model).is_none() {
        ALIAS_FAILURES
            .with_label_values(&[alias, "cost_missing"])
            .inc();
        return Err(AliasError::ResolvedModelMissingCostEntry {
            provider: kind,
            model: model.to_string(),
        });
    }

    // ZDR check (TASK-AI-015) - local providers are inherently ZDR.
    if !is_local && policy.ai_policy.zdr_required && !zdr::is_zdr(&kind, model) {
        ALIAS_FAILURES.with_label_values(&[alias, "zdr"]).inc();
        return Err(AliasError::ZdrViolation {
            resolved_provider: kind,
            resolved_model: model.to_string(),
            attestation: zdr::attestation_for(&kind, model),
        });
    }

    // Residency check (TASK-AI-016) — runs AFTER ZDR check (§1 #10)
    if let Some(region_str) = &region {
        let region = residency::Region::from_provider_string(region_str);
        match region {
            Ok(r) => {
                if !residency::matches(policy.ai_policy.residency, &r) {
                    ALIAS_FAILURES
                        .with_label_values(&[alias, "residency"])
                        .inc();
                    let vn1_no_provider = policy.ai_policy.residency == Residency::Vn1;
                    residency::record_mismatch(policy.ai_policy.residency, &r);
                    if vn1_no_provider {
                        residency::record_vn1_refused(&policy.tenant_id);
                    }
                    return Err(AliasError::ResidencyViolation {
                        resolved_region: Some(region_str.clone()),
                        policy_residency: policy.ai_policy.residency,
                        attempted_alias: alias.to_string(),
                        vn1_no_provider,
                    });
                }
            }
            Err(_e) => {
                // Region string in unknown format — propagate as residency violation
                ALIAS_FAILURES
                    .with_label_values(&[alias, "residency"])
                    .inc();
                return Err(AliasError::ResidencyViolation {
                    resolved_region: Some(region_str.clone()),
                    policy_residency: policy.ai_policy.residency,
                    attempted_alias: alias.to_string(),
                    vn1_no_provider: false,
                });
            }
        }
    } else if policy
        .ai_policy
        .residency_requires_regional_provider
        .unwrap_or(false)
    {
        return Err(AliasError::ResidencyViolation {
            resolved_region: None,
            policy_residency: policy.ai_policy.residency,
            attempted_alias: alias.to_string(),
            vn1_no_provider: false,
        });
    }

    Ok(ResolvedModel {
        provider_kind: kind,
        region,
        model: model.to_string(),
        fallback_position,
        is_zdr: is_local || zdr::is_zdr(&kind, model),
        latency_class: latency_class_for_alias(alias),
    })
}

fn latency_class_for_alias(alias: &str) -> LatencyClass {
    match alias {
        "chat.long" => LatencyClass::Slow,
        "chat.smart" => LatencyClass::Standard,
        "chat.fast" | "embed.standard" | "embed.code" | "rerank.fast" => LatencyClass::Fast,
        _ => unreachable!("alias already validated against SUPPORTED_ALIASES"),
    }
}

fn record_success(alias: &str, r: &ResolvedModel, started: Instant) {
    let elapsed_ns = started.elapsed().as_nanos() as f64;
    ALIAS_RESOLUTIONS
        .with_label_values(&[
            alias,
            r.provider_kind.as_metric_label(),
            &r.fallback_position.to_string(),
        ])
        .inc();
    ALIAS_LATENCY_NS.observe(elapsed_ns);
}
