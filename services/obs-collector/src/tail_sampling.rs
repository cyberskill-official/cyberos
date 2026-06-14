//! FR-OBS-006 — Tail-based sampling policy.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Metric emitted for every tail-sampling decision.
pub const SAMPLED_TRACES_TOTAL: &str = "obs_sampled_traces_total";
/// Gauge for the collector in-flight trace buffer depth.
pub const SAMPLING_BUFFER_DEPTH: &str = "obs_sampling_buffer_depth";

/// Tail-sampling decision reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SamplingReason {
    /// Span status.code was ERROR.
    Error,
    /// HTTP status was 5xx.
    Http5xx,
    /// Tenant is explicitly flagged for full trace capture.
    FlaggedTenant,
    /// End-to-end latency exceeded the route budget.
    Slow,
    /// Normal trace kept by deterministic 10% sampling.
    NormalSample,
    /// Trace dropped by normal sampling or buffer pressure.
    Dropped,
}

impl SamplingReason {
    /// Metric label value.
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Http5xx => "http_5xx",
            Self::FlaggedTenant => "flagged_tenant",
            Self::Slow => "slow",
            Self::NormalSample => "normal_sample",
            Self::Dropped => "dropped",
        }
    }
}

/// Tail-sampling decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplingDecision {
    /// Keep the trace.
    Keep {
        /// First-match sampling reason.
        reason: SamplingReason,
    },
    /// Drop the trace.
    Drop {
        /// Drop reason.
        reason: SamplingReason,
    },
}

impl SamplingDecision {
    /// Whether the decision keeps the trace.
    pub fn kept(self) -> bool {
        matches!(self, Self::Keep { .. })
    }

    /// Decision reason.
    pub fn reason(self) -> SamplingReason {
        match self {
            Self::Keep { reason } | Self::Drop { reason } => reason,
        }
    }
}

/// Tail-sampling policy with first-match semantics.
#[derive(Debug, Clone, PartialEq)]
pub struct TailSamplingPolicy {
    /// Normal trace sampling rate, where 0.10 is 10%.
    pub normal_rate: f64,
    /// Wait before deciding so full traces are available.
    pub decision_wait_seconds: u64,
    /// In-flight trace buffer cap.
    pub num_traces: usize,
    /// Default slow threshold.
    pub default_latency_budget_ms: u64,
    /// Route-specific slow thresholds keyed by `service:route`.
    pub route_latency_budgets_ms: BTreeMap<String, u64>,
    /// Tenants captured at 100% while flagged.
    pub flagged_tenants: BTreeSet<String>,
}

impl Default for TailSamplingPolicy {
    fn default() -> Self {
        Self {
            normal_rate: 0.10,
            decision_wait_seconds: 30,
            num_traces: 100_000,
            default_latency_budget_ms: 2_000,
            route_latency_budgets_ms: BTreeMap::from([
                ("ai-gateway:/v1/chat/completions".to_string(), 5_000),
                ("ai-gateway:/v1/embeddings".to_string(), 500),
                ("auth-service:/v1/auth/token".to_string(), 250),
            ]),
            flagged_tenants: BTreeSet::new(),
        }
    }
}

/// Minimal full-trace summary used by the policy evaluator and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceSummary {
    /// W3C trace id as hex.
    pub trace_id_hex: String,
    /// True when any span status.code is ERROR.
    pub status_error: bool,
    /// HTTP status from semantic-convention attributes.
    pub http_status_code: Option<u16>,
    /// Tenant id carried by resource/span attributes or baggage.
    pub tenant_id: Option<String>,
    /// Service name.
    pub service: String,
    /// Route name/path.
    pub route: String,
    /// End-to-end trace latency.
    pub duration_ms: u64,
}

/// Deterministic sampling rule kept for older tests and callers.
pub fn decide(status_code: u16, trace_id_hex: &str, normal_rate: f64) -> SamplingDecision {
    let trace = TraceSummary {
        trace_id_hex: trace_id_hex.to_string(),
        status_error: false,
        http_status_code: Some(status_code),
        tenant_id: None,
        service: "unknown".to_string(),
        route: "unknown".to_string(),
        duration_ms: 0,
    };
    let policy = TailSamplingPolicy {
        normal_rate,
        ..TailSamplingPolicy::default()
    };
    decide_trace(&policy, &trace)
}

/// Decide whether to keep a trace using FR-OBS-006 precedence.
pub fn decide_trace(policy: &TailSamplingPolicy, trace: &TraceSummary) -> SamplingDecision {
    if trace.status_error {
        return keep(SamplingReason::Error);
    }
    if trace.http_status_code.is_some_and(|status| status >= 500) {
        return keep(SamplingReason::Http5xx);
    }
    if trace
        .tenant_id
        .as_ref()
        .is_some_and(|tenant| policy.flagged_tenants.contains(tenant))
    {
        return keep(SamplingReason::FlaggedTenant);
    }
    if trace.duration_ms > latency_budget(policy, trace) {
        return keep(SamplingReason::Slow);
    }
    if deterministic_sample(&trace.trace_id_hex, policy.normal_rate) {
        keep(SamplingReason::NormalSample)
    } else {
        SamplingDecision::Drop {
            reason: SamplingReason::Dropped,
        }
    }
}

/// Validate the canonical FR policy.
pub fn validate_policy(policy: &TailSamplingPolicy) -> Result<(), String> {
    if policy.decision_wait_seconds < 20 {
        return Err("decision_wait_must_be_at_least_20s".into());
    }
    if policy.num_traces < 100_000 {
        return Err("num_traces_must_be_at_least_100000".into());
    }
    if (policy.normal_rate - 0.10).abs() > f64::EPSILON {
        return Err("normal_must_sample_at_10_percent".into());
    }
    Ok(())
}

/// Load one tenant id per line, allowing comments and YAML list syntax.
pub fn load_flagged_tenants_yaml(raw: &str) -> Result<BTreeSet<String>, String> {
    let mut tenants = BTreeSet::new();
    for (index, line) in raw.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let tenant = line.strip_prefix('-').unwrap_or(line).trim();
        if tenant.is_empty() {
            return Err(format!("line {}: empty tenant id", index + 1));
        }
        tenants.insert(tenant.to_string());
    }
    Ok(tenants)
}

/// Load route latency budgets from YAML.
pub fn load_route_latency_budgets_yaml(raw: &str) -> Result<(u64, BTreeMap<String, u64>), String> {
    let parsed: RouteLatencyBudgetsYaml =
        serde_yaml::from_str(raw).map_err(|err| err.to_string())?;
    let default_ms = parsed.default_ms.unwrap_or(2_000);
    let mut routes = BTreeMap::new();
    for route in parsed.routes {
        if route.service.trim().is_empty() || route.route.trim().is_empty() {
            return Err("route budget requires service and route".into());
        }
        if route.threshold_ms == 0 {
            return Err("route threshold_ms must be positive".into());
        }
        routes.insert(
            format!("{}:{}", route.service, route.route),
            route.threshold_ms,
        );
    }
    Ok((default_ms, routes))
}

fn keep(reason: SamplingReason) -> SamplingDecision {
    SamplingDecision::Keep { reason }
}

fn latency_budget(policy: &TailSamplingPolicy, trace: &TraceSummary) -> u64 {
    let key = format!("{}:{}", trace.service, trace.route);
    policy
        .route_latency_budgets_ms
        .get(&key)
        .copied()
        .unwrap_or(policy.default_latency_budget_ms)
}

fn deterministic_sample(trace_id_hex: &str, normal_rate: f64) -> bool {
    let rate = normal_rate.clamp(0.0, 1.0);
    let prefix = trace_id_hex.get(..8).unwrap_or("0");
    let value = u32::from_str_radix(prefix, 16).unwrap_or(0) as f64 / u32::MAX as f64;
    value < rate
}

#[derive(Debug, Deserialize)]
struct RouteLatencyBudgetsYaml {
    default_ms: Option<u64>,
    #[serde(default)]
    routes: Vec<RouteLatencyBudgetYaml>,
}

#[derive(Debug, Deserialize)]
struct RouteLatencyBudgetYaml {
    service: String,
    route: String,
    threshold_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace() -> TraceSummary {
        TraceSummary {
            trace_id_hex: "19999999000000000000000000000000".to_string(),
            status_error: false,
            http_status_code: Some(200),
            tenant_id: Some("tenant-a".to_string()),
            service: "auth-service".to_string(),
            route: "/v1/auth/token".to_string(),
            duration_ms: 20,
        }
    }

    #[test]
    fn errors_and_5xx_are_kept_before_other_policies() {
        let mut policy = TailSamplingPolicy::default();
        policy.flagged_tenants.insert("tenant-a".to_string());
        let mut slow_error = trace();
        slow_error.status_error = true;
        slow_error.http_status_code = Some(503);
        slow_error.duration_ms = 10_000;

        let decision = decide_trace(&policy, &slow_error);
        assert!(decision.kept());
        assert_eq!(decision.reason(), SamplingReason::Error);

        slow_error.status_error = false;
        let decision = decide_trace(&policy, &slow_error);
        assert_eq!(decision.reason(), SamplingReason::Http5xx);
    }

    #[test]
    fn flagged_tenant_beats_slow_and_normal_sampling() {
        let mut policy = TailSamplingPolicy::default();
        policy.flagged_tenants.insert("tenant-a".to_string());
        let mut sample = trace();
        sample.duration_ms = 10_000;

        let decision = decide_trace(&policy, &sample);
        assert!(decision.kept());
        assert_eq!(decision.reason(), SamplingReason::FlaggedTenant);
    }

    #[test]
    fn per_route_budget_detects_slow_auth_trace() {
        let policy = TailSamplingPolicy::default();
        let mut sample = trace();
        sample.duration_ms = 300;

        let decision = decide_trace(&policy, &sample);
        assert!(decision.kept());
        assert_eq!(decision.reason(), SamplingReason::Slow);
    }

    #[test]
    fn normal_traces_are_sampled_by_deterministic_trace_hash() {
        let policy = TailSamplingPolicy::default();
        let mut kept = trace();
        kept.trace_id_hex = "00000001000000000000000000000000".to_string();
        let mut dropped = trace();
        dropped.trace_id_hex = "ffffffff000000000000000000000000".to_string();

        assert_eq!(
            decide_trace(&policy, &kept).reason(),
            SamplingReason::NormalSample
        );
        assert_eq!(
            decide_trace(&policy, &dropped).reason(),
            SamplingReason::Dropped
        );
    }

    #[test]
    fn load_flagged_tenants_accepts_comments_and_yaml_list() {
        let tenants = load_flagged_tenants_yaml(
            r#"
# incident tenant
- tenant-a
tenant-b # inline comment
"#,
        )
        .unwrap();
        assert!(tenants.contains("tenant-a"));
        assert!(tenants.contains("tenant-b"));
    }

    #[test]
    fn load_route_latency_budgets_validates_shape() {
        let (default_ms, routes) = load_route_latency_budgets_yaml(
            r#"
default_ms: 2000
routes:
  - service: ai-gateway
    route: /v1/chat/completions
    threshold_ms: 5000
"#,
        )
        .unwrap();

        assert_eq!(default_ms, 2_000);
        assert_eq!(routes.get("ai-gateway:/v1/chat/completions"), Some(&5_000));
    }

    #[test]
    fn policy_validation_enforces_canonical_values() {
        validate_policy(&TailSamplingPolicy::default()).unwrap();

        let too_short = TailSamplingPolicy {
            decision_wait_seconds: 10,
            ..TailSamplingPolicy::default()
        };
        assert!(validate_policy(&too_short).is_err());

        let wrong_rate = TailSamplingPolicy {
            normal_rate: 0.5,
            ..TailSamplingPolicy::default()
        };
        assert!(validate_policy(&wrong_rate).is_err());
    }
}
