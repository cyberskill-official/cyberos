---
id: NFR-AI-009
title: "AI Gateway residency pinning enforcement — provider region matches tenant policy 100%"
module: AI
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of upstream calls route to a provider region matching the tenant residency policy"
owner: CSO
created: 2026-05-18
related_frs: [FR-AI-016]
---

## §1 — Statement (BCP-14 normative)

1. Every tenant **MUST** carry a residency policy field `residency_region` ∈ {`apac`, `eu`, `us`, `global`} on the tenants table. The AI Gateway reads this from the JWT or from a cached lookup.
2. For tenants with `residency_region != global`, the gateway **MUST** route ONLY to provider regions matching that residency. Cross-region routing **MUST** be impossible by construction — not by post-hoc filter.
3. The provider registry (`services/ai-gateway/src/providers/registry.toml`) tags each provider entry with a `region` field; the selection algorithm filters by `region == tenant_residency_region` before scoring.
4. If no provider in the matching region is available (all circuit-open per NFR-AI-003), the gateway **MUST** return HTTP 503 with body `{error: "no_provider_in_residency_region", region: "<tenant_region>"}` — it **MUST NOT** fall over to a non-matching region.
5. Every upstream call **MUST** emit a structured trace `ai_gateway.upstream.region_match` with `{tenant_residency, provider_region, match: true}`. Any row with `match: false` is sev-0.

## §2 — Why this constraint

Data residency is a hard contractual claim for EU tenants (GDPR Art. 44+) and an emerging one for Vietnamese tenants (Decree 53/2022 data localisation). A single cross-region call is sufficient to violate the contract — even if no PII was in the prompt, the metadata leaks (tenant identity, traffic volume, request timing). The "no fallback" rule is intentional — we prefer a temporary 503 over a permanent compliance breach. The construct-not-filter implementation guards against future code changes accidentally re-enabling fallback.

## §3 — Measurement

- Counter `ai_gateway_region_match_total{tenant_region, provider_region, match}` per call.
- Sev-0 alarm on `match=false` rate > 0 (any cross-region routing is immediate page).
- Gauge `ai_gateway_residency_region_503_rate{tenant_region}` — surfaces when all matching providers are down; informs capacity-planning.

## §4 — Verification

- Property test `services/ai-gateway/tests/residency_pinning_test.rs` (T) — generates 1000 random (tenant_region, available_providers) scenarios; asserts no cross-region call ever fires.
- Integration test (T) — simulates `eu` tenant with only `us` providers available; asserts HTTP 503, no upstream call.
- Quarterly audit (I) — CSO inspects 50 random recent traces; every `upstream.region_match` event must have `match=true`.

## §5 — Failure handling

- Any `match=false` event → sev-0; halt all routing globally until root cause identified; emergency CSO + CTO call.
- 503 rate > 5% for a tenant_region for 10 minutes → sev-2; capacity team adds a provider in that region.
- Tenant requests residency change → must be processed via TEN module migration flow, not by JWT mutation (FR-TEN-001 contract).

---

*End of NFR-AI-009.*
