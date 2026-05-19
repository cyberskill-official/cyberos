---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-016
title: "Tenant residency pinning (sg-1 / eu-1 / us-1 / vn-1) propagating to provider region selection"
module: AI
priority: MUST
status: ready_to_implement
verify: T
phase: P0
milestone: P0 · slice 4
slice: 4
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-AI-005, FR-AI-006, FR-AI-008, FR-AI-015, FR-AI-022]
depends_on: [FR-AI-006]
blocks: [FR-TEN-103, FR-AI-104]   # FR-AI-104 placeholder — not yet specified

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#residency
  - website/docs/legal/vn-decree-53.html
  - website/docs/legal/gdpr-cross-border.html
source_decisions:
  - Decree 53/2022 (Vietnam — data localisation for personal data of VN residents)
  - PDPL Art. 7 (Vietnam personal-data-sale + cross-border restrictions)
  - GDPR Art. 44 (cross-border transfer general principle)
  - GDPR Art. 45 (adequacy decisions; EU↔US data transfers via Data Privacy Framework)
  - DEC-073 (residency is a tenant-policy field; default refused if not pinned for PDPL tenants)
  - archive/2026-05-14/RESEARCH_REVIEW.md §4.5 (residency precedence over cost-optimisation)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/residency/mod.rs
  - services/ai-gateway/src/residency/parse.rs
  - services/ai-gateway/src/residency/region_table.rs
  - services/ai-gateway/tests/residency_test.rs
  - services/ai-gateway/tests/residency_property_test.rs
  - services/ai-gateway/tests/residency_integration_test.rs
modified_files:
  - services/ai-gateway/src/alias.rs                         # FR-AI-006 §1 #7 invokes residency::matches
  - services/ai-gateway/src/handlers/chat.rs                 # emit ai.residency_violation memory row on refusal
  - services/ai-gateway/src/memory_writer.rs                  # add canonical::residency_violation builder
  - services/ai-gateway/src/policy.rs                        # parse `residency` field from FR-AI-005 schema
  - services/ai-gateway/Cargo.toml                           # proptest@1, regex@1 (for region-string validation)
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests}/**
  - bash: cargo test -p cyberos-ai-gateway residency
  - bash: cargo test -p cyberos-ai-gateway --test residency_property_test
disallowed_tools:
  - hardcode region strings outside `region_table.rs`
  - bypass `policy.ai_policy.residency` from any code path
  - silently degrade `vn-1` to `sg-1` (must explicitly refuse and let the caller choose; per §1 #6)
  - extend the residency enum without an FR amendment (slice 4 ships exactly 4 variants)

# ───── Estimated work ─────
effort_hours: 8
sub_tasks:
  - "0.5h: Residency enum (Sg1, Eu1, Us1, Vn1) + Region newtype with regex validation"
  - "0.5h: region_table.rs — RESIDENCY_REGIONS map (Residency → frozenset of acceptable regions)"
  - "0.5h: residency::matches(Residency, &Region) -> bool"
  - "0.5h: residency::parse_residency(&str) -> Result<Residency, ResidencyParseError>"
  - "0.5h: AZ-suffix stripping — `ap-southeast-1a` → `ap-southeast-1` (provider returns AZ; we match region)"
  - "1.0h: Integration into FR-AI-006's alias resolution (matches AFTER ZDR check; documented precedence)"
  - "0.5h: ai.residency_violation memory audit row builder + handler emission"
  - "0.5h: HTTP 403 RESIDENCY_VIOLATION response shape"
  - "0.5h: OTel metrics (mismatches_total, vn_fallback_refused_total)"
  - "1.0h: Property test (1000 random Residency × Region pairs; assert no cross-residency leak)"
  - "0.5h: Default residency fallback — tenants with no `residency` field default to refuse-all per §1 #4"
  - "0.5h: Override mechanism — tenant policy `residency_override: { alias_pattern: residency_override }` for explicit per-alias pinning (FR-AI-005 schema extension)"
  - "0.5h: Tests — 17 ACs across happy path, property, integration, parse, AZ-strip, override"
risk_if_skipped: "Tenant `residency: vn-1` could resolve to `bedrock: us-east-1`. Decree 53/2022 (data localisation for VN-resident PII) violation; PDPL Art. 7 cross-border-transfer breach. Catastrophic VN regulatory failure on first audit. EU tenants pinned to `eu-1` could resolve to US-East — GDPR Art. 44 violation; the EU-US Data Privacy Framework only covers specific certified sub-processors (we are not certified). Customer churn on regulator notification; class of risk that ends a B2B contract instantly."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** enforce tenant residency at alias-resolution time. The enforcement and surrounding contract obey the following:

1. **MUST** expose `residency::matches(policy_residency: Residency, provider_region: &Region) -> bool`. Returns `true` if and only if the provider's region is in the residency's acceptable-region set per the `region_table.rs` mapping.
2. **MUST** define the residency → acceptable-region mapping in `region_table.rs` as a `frozenset` per residency; changes to the mapping require explicit FR amendment:
   - `Sg1` → `{"ap-southeast-1"}` (Singapore; AWS only at slice 4).
   - `Eu1` → `{"eu-central-1", "eu-west-1"}` (Frankfurt, Ireland — both adequacy-covered for EU↔EU intra-region transfers).
   - `Us1` → `{"us-east-1", "us-east-2", "us-west-2"}` (Northern Virginia, Ohio, Oregon).
   - `Vn1` → `{}` (empty set; no AWS region is in-country for Vietnam at slice 4).
3. **MUST** be invoked by `alias::resolve` (FR-AI-006 §1 #7). When `residency::matches(policy.residency, &resolved_region) == false`, `alias::resolve` returns `Err(AliasError::ResidencyViolation { policy_residency, resolved_region, attempted_alias })`.
4. **MUST** treat tenants with a missing `policy.ai_policy.residency` field as fail-closed for PDPL-pinned tenants (any tenant with `policy.tenant_jurisdiction == "VN"`); for other tenants the missing-field default is `Sg1` (Asia-Pacific consensus default for the CyberSkill home region). The default is documented in FR-AI-005's schema; this FR consumes the parsed value.
5. **MUST** strip AZ suffix from the provider-returned region string before matching: `"ap-southeast-1a"` → `"ap-southeast-1"`. The matcher operates on AWS region strings (no AZ); AZ-aware policies are out of scope at slice 4. The strip rule is `^(?P<region>[a-z]{2}-[a-z]+-\d+)[a-z]?$` — stripping a single trailing alpha character if present.
6. **MUST** return `false` for `Vn1` against ANY provider region in slice 4 (no VN provider integrated). Tenants pinning `vn-1` MUST be refused at resolve time; the refusal carries a distinct error message `vn1_no_provider_yet` so the operator dashboard can distinguish "no VN provider" from "wrong region" failures. FR-AI-104 (placeholder) will integrate Viettel Cloud + FPT Cloud and add their region strings to the `Vn1` set.
7. **MUST** emit an `ai.residency_violation` memory audit row when a request is refused due to residency. The row carries `tenant_id`, `agent_persona`, `requested_alias`, `policy_residency`, `resolved_region`, `request_id`, AND a `vn1_no_provider` boolean (true when residency is Vn1 and the failure is due to absence of a VN provider rather than wrong region).
8. **MUST** propagate `ResidencyViolation` errors as HTTP `403 RESIDENCY_VIOLATION` with body `{"error":"residency_violation","policy_residency":"<r>","resolved_region":"<reg>","contact":"ops@cyberos.world"}`. For Vn1 failures, the error code is `residency_violation` AND the body includes `"reason":"no_vn_provider_yet"` so client UIs can render an informative message.
9. **MUST** be deterministic: same `(Residency, Region)` pair always returns the same boolean. The `region_table.rs` mapping is a `LazyLock<HashMap<Residency, HashSet<&'static str>>>`; no I/O, no time-dependent state, no env-var lookup.
10. **MUST** integrate with FR-AI-015 (ZDR enforcement) such that the precedence is: **ZDR check first, then residency**. A request that fails ZDR is refused with `ZdrViolation`; a request that passes ZDR but fails residency is refused with `ResidencyViolation`. Both checks run in `alias::resolve`; the order is fixed (ZDR before residency) and documented in FR-AI-006's resolve function.
11. **MUST** support a per-alias residency override (`policy.ai_policy.residency_override`) for tenants that need to pin a specific alias to a different residency than the tenant default. Schema: `residency_override: { "<alias-glob>": "<residency>" }`. Example: a SG tenant pinning `chat.eu-customer-data` to `eu-1`. The override is consulted BEFORE the tenant default; ambiguous overrides (multiple globs match) fail with `OverrideAmbiguous`.
12. **MUST NOT** silently degrade `vn-1` to a "closest available" region. Slice 4 explicitly refuses; the alternative (silent fallback to `sg-1`) is the failure mode that produces a Decree 53 violation on a tenant who pinned `vn-1` precisely to avoid out-of-country routing.
13. **SHOULD** emit OTel metrics:
    - `ai_residency_mismatches_total{policy_residency, resolved_region}` (counter; alarm > 0).
    - `ai_residency_vn1_refused_total{tenant_id}` (counter; tracks `Vn1` refusals separately from generic mismatches; trending this informs FR-AI-104 prioritisation).
    - `ai_residency_overrides_used_total{tenant_id, alias}` (counter; how often per-alias overrides fire).
    - `ai_residency_default_applied_total{outcome}` (counter; outcome ∈ `sg1_default | refused_pdpl_no_pin`; tracks the missing-field fallback path).
14. **SHOULD** log at WARN on every `Vn1` refusal — `vn1 residency refused; FR-AI-104 Viettel integration needed for tenant=<id>` so the operator dashboard can prioritise the VN-provider build.

---

## §2 — Why this design (rationale for humans)

**Why is residency the highest-precedence policy gate (after ZDR)?** Residency violations are *unrecoverable*: once data crosses a border, no audit-trail or apology undoes the transfer. Decree 53/2022 (Vietnam) imposes data localisation for VN-resident PII; the fines are denominated in revenue percentage. GDPR Art. 44's cross-border-transfer principle, paired with the limited adequacy decisions (Art. 45), means EU data routed to non-adequate jurisdictions is presumed unlawful. Both regimes treat the violation as occurring at the moment of transfer; there is no remediation that reverses the breach. The cost of an extra HashMap lookup in `alias::resolve` (microseconds) is trivial vs. the cost of a single mis-routed call (potentially the contract).

**Why fail-closed on missing residency for PDPL tenants (§1 #4)?** A VN-jurisdiction tenant whose policy file omits `residency:` is in an indeterminate state — they didn't explicitly pin VN, but the regulator presumes VN data localisation applies. Defaulting to `Sg1` (the closest acceptable AP region) is the conservative call: it avoids the Decree 53 risk without forcing every onboarding to explicitly set residency. Tenants who legitimately want EU or US routing must explicitly pin; the default is the safe-but-tight choice. Non-PDPL tenants get the same `Sg1` default for operational consistency.

**Why static enum (§1 #2) and not config-driven?** The four residency values map to legal-jurisdiction categories that change rarely (AWS region launches don't add new residency tiers — they add new acceptable regions within an existing tier). Encoding them as a Rust enum + LazyLock map means the type system enforces "you can't add a residency without amending the FR and recompiling." A config-driven approach invites operational drift ("operator added `apac-2` to the YAML; nobody noticed it doesn't map to any AWS region"). The trade-off is loss of hot-reloadability for residency itself; given the rarity of changes, this is the right trade.

**Why is `Vn1` an empty set rather than mapped to `ap-southeast-1` (the closest region)?** Because mapping it to `ap-southeast-1` would convert "Vietnam data residency" into "Singapore data residency" without telling anyone. A tenant pinning `vn-1` is making a regulatory statement; satisfying that statement requires a VN-located provider, which we don't have at slice 4. The honest answer is "we can't serve you under this constraint yet" — the dishonest answer is "we'll route to Singapore and hope the regulator doesn't notice." Refusing and waiting for FR-AI-104 (Viettel/FPT integration) preserves the regulatory contract.

**Why strip AZ suffix (§1 #5)?** Bedrock and Vertex sometimes return AZ-suffixed region strings (`ap-southeast-1a`) in error messages or routing metadata. The matcher operates at region granularity (the legal residency unit) — AZ is sub-region and irrelevant to localisation regulation. The single trailing `[a-z]?` strip handles every AWS AZ format and is unambiguous (no AWS region ends in a single letter; the ambiguity of `ap-southeast-1a` would only arise if AWS launched a region literally named `ap-southeast-1a`, which they won't because their convention treats trailing letters as AZ).

**Why does residency precedence go AFTER ZDR (§1 #10)?** Both are compliance gates; both refuse calls. Order matters because the operator dashboard sees the first-fired error. Putting ZDR first lets a tenant who fails BOTH gates see "ZDR violation" first (the more expensive/restrictive gate is more diagnostic — fixing ZDR often resolves residency, since ZDR-attested providers tend to publish region availability). Putting residency first would obscure ZDR failures behind region failures. The order is documented and tested; future FRs touching this precedence must justify a change.

**Why per-alias residency override (§1 #11)?** Real-world: a SG-default tenant has one workflow that processes EU customer data ("send EU customer support summary to model X"). They want most calls to route via SG (latency, cost), but THIS specific alias must route via EU. Without the override, the tenant has only two unsatisfactory choices: (a) flip the entire tenant to `Eu1` (degrading every other call's latency), or (b) accept the GDPR risk. The override gives them surgical control. The ambiguity-rejection rule (`OverrideAmbiguous`) prevents the operational footgun of two glob patterns matching the same alias with different residencies.

**Why a dedicated audit row (`ai.residency_violation`)?** Same reasoning as ZDR's audit row (FR-AI-015 §1 #6): a regulator's question "did you ever route VN data outside Vietnam" needs a positive answer (rows showing refusals) rather than absence-of-evidence. The `vn1_no_provider` field is informational — when a regulator asks "why did you refuse this Vn1 tenant," the row says "because we don't have a VN provider integrated yet" rather than "we refused for some reason."

**Why does the metric set include `ai_residency_vn1_refused_total{tenant_id}`?** Trending this metric tells us how badly we need FR-AI-104 (the Viettel/FPT integration). If five VN tenants refuse 1000 calls/day each due to `Vn1` not being mapped, the FR-AI-104 prioritisation case writes itself. Without the per-tenant breakdown, the demand signal is invisible.

**Why a property test (§5)?** The matcher's correctness is "no cross-residency pairs return true." A unit test enumerating every (Residency × Region) pair has 4 × ~30 = ~120 cases; manually writing them is tedious and bug-prone. proptest generates random pairs and asserts the global property in 1000+ trials. The property test catches accidental aliasing (e.g., adding a region to `Eu1` that's also in `Us1` due to copy-paste error) that a happy-path test would miss.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Type definitions

```rust
// services/ai-gateway/src/residency/mod.rs

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Residency {
    #[serde(rename = "sg-1")] Sg1,
    #[serde(rename = "eu-1")] Eu1,
    #[serde(rename = "us-1")] Us1,
    #[serde(rename = "vn-1")] Vn1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region(String);   // newtype to prevent string-typing accidents

impl Region {
    /// Strip AZ suffix per §1 #5.
    pub fn from_provider_string(raw: &str) -> Result<Self, RegionParseError> {
        static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"^(?P<region>[a-z]{2}-[a-z]+-\d+)[a-z]?$").unwrap()
        });
        let caps = RE.captures(raw).ok_or_else(|| RegionParseError::Invalid(raw.into()))?;
        Ok(Region(caps.name("region").unwrap().as_str().into()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

pub fn matches(policy_residency: Residency, provider_region: &Region) -> bool {
    REGIONS_BY_RESIDENCY.get(&policy_residency)
        .map(|set| set.contains(provider_region.as_str()))
        .unwrap_or(false)
}

pub fn parse_residency(s: &str) -> Result<Residency, ResidencyParseError> {
    serde_yaml::from_str(s).map_err(|e| ResidencyParseError::Invalid(e.to_string()))
}

#[derive(Debug, thiserror::Error)]
pub enum ResidencyParseError {
    #[error("invalid residency value {0:?}; expected sg-1 | eu-1 | us-1 | vn-1")]
    Invalid(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RegionParseError {
    #[error("invalid region string {0:?}; expected AWS region format")]
    Invalid(String),
}

// In FR-AI-006 alias.rs (modified_files):
pub enum AliasError {
    // ... existing variants from FR-AI-006 + FR-AI-015 ...
    ResidencyViolation {
        policy_residency: Residency,
        resolved_region: Region,
        attempted_alias: String,
        vn1_no_provider: bool,
    },
}
```

### Region table (single source of truth)

```rust
// services/ai-gateway/src/residency/region_table.rs

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use super::Residency;

/// §1 #2: residency → acceptable-region mapping. Changes require FR amendment.
pub static REGIONS_BY_RESIDENCY: LazyLock<HashMap<Residency, HashSet<&'static str>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(Residency::Sg1, {
            let mut s = HashSet::new();
            s.insert("ap-southeast-1");        // Singapore
            s
        });
        m.insert(Residency::Eu1, {
            let mut s = HashSet::new();
            s.insert("eu-central-1");          // Frankfurt
            s.insert("eu-west-1");             // Ireland
            s
        });
        m.insert(Residency::Us1, {
            let mut s = HashSet::new();
            s.insert("us-east-1");             // N. Virginia
            s.insert("us-east-2");             // Ohio
            s.insert("us-west-2");             // Oregon
            s
        });
        m.insert(Residency::Vn1, HashSet::new());   // §1 #6: empty until FR-AI-104
        m
    });
```

### Override parser

```rust
// services/ai-gateway/src/residency/parse.rs

use globset::{Glob, GlobMatcher};

pub struct ResidencyOverride {
    pub alias_pattern: GlobMatcher,
    pub residency: Residency,
}

#[derive(Debug, thiserror::Error)]
pub enum OverrideError {
    #[error("ambiguous override: aliases [{0:?}] both match alias {1!r}")]
    OverrideAmbiguous(Vec<String>, String),
    #[error("invalid glob pattern {0!r}: {1}")]
    InvalidGlob(String, String),
}

pub fn resolve_override(
    overrides: &[ResidencyOverride], alias: &str,
) -> Result<Option<Residency>, OverrideError> {
    let matches: Vec<_> = overrides.iter()
        .filter(|o| o.alias_pattern.is_match(alias))
        .collect();
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches[0].residency)),
        _ => Err(OverrideError::OverrideAmbiguous(
            matches.iter().map(|o| o.alias_pattern.glob().to_string()).collect(),
            alias.into(),
        )),
    }
}
```

### Tenant policy schema extension (FR-AI-005)

```yaml
# tenants/<tenant_id>/policy.yaml — additions
ai_policy:
  residency: sg-1                          # default residency for this tenant
  residency_override:                      # per-alias override (§1 #11)
    "chat.eu-customer-*":  eu-1            # any chat alias starting with eu-customer- pins EU
    "embeddings.gdpr-pii": eu-1            # specific embeddings alias pins EU
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Sg1 → ap-southeast-1 matches** — `matches(Sg1, &Region("ap-southeast-1".into()))` returns `true`.
2. **Sg1 → us-east-1 mismatches** — `matches(Sg1, &Region("us-east-1".into()))` returns `false`.
3. **Eu1 → eu-central-1 AND eu-west-1 match** — both regions return `true`.
4. **Us1 → all three US regions match** — `us-east-1`, `us-east-2`, `us-west-2` all return `true`.
5. **Vn1 → empty set always returns false** — `matches(Vn1, &Region("ap-southeast-1".into()))` returns `false`.
6. **AZ-suffix stripped** — `Region::from_provider_string("ap-southeast-1a")` returns `Region("ap-southeast-1")`; `matches(Sg1, &that)` returns `true`.
7. **Invalid region string rejected** — `Region::from_provider_string("not-a-region")` returns `Err(RegionParseError::Invalid)`.
8. **Property test: no cross-residency leak** — proptest 1000 trials over `(any_residency, any_region)`: if `matches(R, region)` returns `true`, then `region.as_str()` is in `REGIONS_BY_RESIDENCY[R]`. No accidental aliasing between residencies.
9. **FR-AI-006 integration: refusal on mismatch** — Tenant policy `residency: sg-1`, alias resolves to bedrock `us-east-1` → `alias::resolve` returns `Err(AliasError::ResidencyViolation { policy_residency: Sg1, resolved_region: Region("us-east-1"), .. })`.
10. **FR-AI-006 integration: success on match** — Tenant policy `residency: sg-1`, alias resolves to bedrock `ap-southeast-1` → `alias::resolve` returns `Ok((Bedrock, "claude-3-...", Region("ap-southeast-1")))`.
11. **Vn1 refusal carries vn1_no_provider flag** — Tenant policy `residency: vn-1`; any alias resolution returns `Err(AliasError::ResidencyViolation { vn1_no_provider: true, .. })`; OTel `ai_residency_vn1_refused_total{tenant_id}` increments.
12. **HTTP 403 RESIDENCY_VIOLATION on refusal** — Handler converts AliasError to a `403` response with the documented body shape; Vn1 case includes `"reason":"no_vn_provider_yet"`.
13. **Audit row emitted** — Every refusal emits exactly one `ai.residency_violation` memory row with all required fields populated.
14. **Missing residency field defaults to Sg1 (non-PDPL tenant)** — Tenant policy without `residency` field; FR-AI-005 schema parser defaults to `Sg1`; `alias::resolve` enforces against `Sg1`.
15. **Missing residency field for PDPL tenant fails closed** — Tenant with `tenant_jurisdiction: VN` but no `residency` field; FR-AI-005 returns `PolicyError::MissingResidencyForPdplTenant`; HTTP `503 POLICY_INVALID`.
16. **Per-alias override applies before tenant default** — Policy `residency: sg-1` + `residency_override: { "chat.eu-customer-*": eu-1 }`; resolving `chat.eu-customer-summary` enforces against `Eu1`, not `Sg1`.
17. **Ambiguous override rejected at parse** — Two glob patterns match the same alias with different residencies; `resolve_override` returns `Err(OverrideAmbiguous)`; FR-AI-005 policy load fails with `PolicyError::AmbiguousResidencyOverride`.
18. **ZDR check fires before residency check** — Tenant policy `zdr_required: true` + `residency: sg-1`; alias resolves to `openai gpt-4o` (non-ZDR, non-SG); `alias::resolve` returns `Err(AliasError::ZdrViolation)`, NOT `ResidencyViolation` — ZDR is the first-fired error.

---

## §5 — Verification

### Happy + matcher tests

```rust
// services/ai-gateway/tests/residency_test.rs
use cyberos_ai_gateway::residency::{self, Residency, Region};

#[test]
fn sg1_accepts_apse1_only() {
    let region = Region::from_provider_string("ap-southeast-1").unwrap();
    assert!(residency::matches(Residency::Sg1, &region));
    let region = Region::from_provider_string("us-east-1").unwrap();
    assert!(!residency::matches(Residency::Sg1, &region));
}

#[test]
fn eu1_accepts_central_and_west() {
    for r in &["eu-central-1", "eu-west-1"] {
        assert!(residency::matches(Residency::Eu1, &Region::from_provider_string(r).unwrap()));
    }
}

#[test]
fn us1_accepts_all_three_us_regions() {
    for r in &["us-east-1", "us-east-2", "us-west-2"] {
        assert!(residency::matches(Residency::Us1, &Region::from_provider_string(r).unwrap()));
    }
}

#[test]
fn vn1_empty_set_always_returns_false() {
    for r in &["ap-southeast-1", "us-east-1", "eu-central-1"] {
        let region = Region::from_provider_string(r).unwrap();
        assert!(!residency::matches(Residency::Vn1, &region));
    }
}

#[test]
fn az_suffix_stripped() {
    let region = Region::from_provider_string("ap-southeast-1a").unwrap();
    assert_eq!(region.as_str(), "ap-southeast-1");
    assert!(residency::matches(Residency::Sg1, &region));
}

#[test]
fn invalid_region_string_rejected() {
    let err = Region::from_provider_string("not-a-region").expect_err("invalid");
    assert!(matches!(err, residency::RegionParseError::Invalid(_)));
}

#[test]
fn parse_residency_from_yaml() {
    assert_eq!(residency::parse_residency("sg-1").unwrap(), Residency::Sg1);
    assert_eq!(residency::parse_residency("vn-1").unwrap(), Residency::Vn1);
    assert!(residency::parse_residency("apac-2").is_err());
}
```

### Property test

```rust
// services/ai-gateway/tests/residency_property_test.rs
use proptest::prelude::*;
use cyberos_ai_gateway::residency::{self, Residency, Region, region_table::REGIONS_BY_RESIDENCY};

fn any_residency() -> impl Strategy<Value = Residency> {
    prop_oneof![Just(Residency::Sg1), Just(Residency::Eu1), Just(Residency::Us1), Just(Residency::Vn1)]
}

fn any_region() -> impl Strategy<Value = Region> {
    let known_regions = [
        "ap-southeast-1", "ap-southeast-2", "ap-northeast-1",
        "eu-central-1", "eu-west-1", "eu-west-2", "eu-north-1",
        "us-east-1", "us-east-2", "us-west-1", "us-west-2",
        "ca-central-1", "sa-east-1",
    ];
    prop::sample::select(known_regions.to_vec())
        .prop_map(|r| Region::from_provider_string(r).unwrap())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn no_cross_residency_leak(r in any_residency(), region in any_region()) {
        // AC #8: matches(R, region) → true ⇒ region in REGIONS_BY_RESIDENCY[R]
        if residency::matches(r, &region) {
            let allowed = REGIONS_BY_RESIDENCY.get(&r).unwrap();
            prop_assert!(allowed.contains(region.as_str()),
                        "cross-residency leak: {r:?} matched {region:?} but region not in allowed set");
        }
    }

    #[test]
    fn deterministic(r in any_residency(), region in any_region()) {
        // §1 #9: same pair → same result, run twice.
        let r1 = residency::matches(r, &region);
        let r2 = residency::matches(r, &region);
        prop_assert_eq!(r1, r2);
    }
}
```

### Integration test (FR-AI-006 wiring)

```rust
// services/ai-gateway/tests/residency_integration_test.rs
use cyberos_ai_gateway::{alias, residency::Residency};

#[tokio::test]
async fn alias_resolve_refuses_on_residency_mismatch() {
    let policy = test_policy_with_residency(Residency::Sg1);
    let result = alias::resolve("chat.us-only-alias", &policy).await;   // routes to us-east-1
    match result {
        Err(alias::AliasError::ResidencyViolation { policy_residency, resolved_region, vn1_no_provider, .. }) => {
            assert_eq!(policy_residency, Residency::Sg1);
            assert_eq!(resolved_region.as_str(), "us-east-1");
            assert!(!vn1_no_provider);
        }
        _ => panic!("expected ResidencyViolation"),
    }
}

#[tokio::test]
async fn alias_resolve_succeeds_on_residency_match() {
    let policy = test_policy_with_residency(Residency::Sg1);
    let (provider, model, region) = alias::resolve("chat.smart", &policy).await.unwrap();
    assert_eq!(region.as_str(), "ap-southeast-1");
}

#[tokio::test]
async fn vn1_carries_no_provider_flag() {
    let policy = test_policy_with_residency(Residency::Vn1);
    let result = alias::resolve("chat.smart", &policy).await;
    match result {
        Err(alias::AliasError::ResidencyViolation { vn1_no_provider: true, .. }) => {}
        _ => panic!("expected vn1_no_provider"),
    }

    let counter = otel_test_helper::counter_value(
        "ai_residency_vn1_refused_total",
        &[("tenant_id", &policy.tenant_id)],
    );
    assert!(counter >= 1);
}

#[tokio::test]
async fn audit_row_emitted_on_residency_refusal() {
    let request_id = "req_test_residency_001";
    let _ = handlers::chat::handle(test_request_with_residency(
        Residency::Sg1, request_id, "chat.us-only-alias",
    )).await;
    let rows = memory_test_helper::find_rows("ai.residency_violation", request_id);
    assert_eq!(rows.len(), 1);
    let p = &rows[0].payload;
    assert_eq!(p["policy_residency"], "sg-1");
    assert_eq!(p["resolved_region"], "us-east-1");
    assert_eq!(p["vn1_no_provider"], false);
}

#[tokio::test]
async fn zdr_check_fires_before_residency() {
    // §1 #10: ZDR before residency.
    let policy = test_policy_with_residency_and_zdr(Residency::Sg1, /* zdr_required */ true);
    let result = alias::resolve("chat.openai-us-only", &policy).await;   // openai gpt-4o, us-east-1, non-ZDR
    match result {
        Err(alias::AliasError::ZdrViolation { .. }) => {}   // ZDR fires first
        Err(alias::AliasError::ResidencyViolation { .. }) => panic!("residency fired before ZDR"),
        _ => panic!("expected error"),
    }
}

#[tokio::test]
async fn per_alias_override_wins_over_tenant_default() {
    let policy = test_policy_with_override(
        Residency::Sg1,   // default
        vec![("chat.eu-customer-*".into(), Residency::Eu1)],
    );
    let result = alias::resolve("chat.eu-customer-summary", &policy).await;
    // Without override, this would resolve to SG region; with override, must resolve to EU.
    let (_, _, region) = result.unwrap();
    assert!(region.as_str().starts_with("eu-"));
}

#[tokio::test]
async fn ambiguous_override_rejected_at_policy_load() {
    let yaml = r#"
        ai_policy:
          residency: sg-1
          residency_override:
            "chat.*-customer-*": eu-1
            "chat.eu-*":         us-1
    "#;
    let err = policy::parse(yaml).expect_err("expected AmbiguousResidencyOverride");
    assert!(matches!(err, policy::PolicyError::AmbiguousResidencyOverride { .. }));
}
```

```bash
cd services/ai-gateway
cargo test -p cyberos-ai-gateway residency
cargo test -p cyberos-ai-gateway --test residency_property_test
```

---

## §6 — Implementation skeleton

See §3 for the type defs + region table + override parser. Integration into FR-AI-006's resolve:

```rust
// services/ai-gateway/src/alias.rs (modified)

pub async fn resolve(alias: &str, policy: &TenantPolicy) -> Result<(ProviderKind, String, Region), AliasError> {
    // Resolve alias → (provider, model, region)
    let (provider, model, region) = ALIAS_MAP.get().unwrap().load().get(alias)
        .ok_or_else(|| AliasError::UnknownAlias(alias.into()))?;

    // §1 #10: ZDR before residency.
    if policy.ai_policy.zdr_required && !zdr::is_zdr(&provider, &model) {
        return Err(AliasError::ZdrViolation {
            resolved_provider: provider, resolved_model: model.clone(),
            attestation: zdr::attestation_for(&provider, &model),
        });
    }

    // §1 #11: per-alias override resolves first.
    let effective_residency = match parse::resolve_override(&policy.ai_policy.residency_override, alias) {
        Ok(Some(r)) => r,
        Ok(None) => policy.ai_policy.residency,   // tenant default
        Err(e) => return Err(AliasError::OverridePolicyInvalid(e.to_string())),
    };

    // §1 #1, §1 #5: matcher with AZ-strip.
    if !residency::matches(effective_residency, &region) {
        let vn1_no_provider = effective_residency == Residency::Vn1;
        return Err(AliasError::ResidencyViolation {
            policy_residency: effective_residency,
            resolved_region: region.clone(),
            attempted_alias: alias.into(),
            vn1_no_provider,
        });
    }

    Ok((provider, model.clone(), region.clone()))
}
```

`canonical::residency_violation` builder:

```rust
pub mod canonical {
    pub fn residency_violation(
        tenant_id: &str, agent_persona: &str, requested_alias: &str,
        policy_residency: Residency, resolved_region: &Region,
        vn1_no_provider: bool, request_id: &str,
    ) -> AuditRow {
        AuditRow {
            kind: "ai.residency_violation".into(),
            payload: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "requested_alias": requested_alias,
                "policy_residency": serde_yaml::to_string(&policy_residency).unwrap().trim(),
                "resolved_region": resolved_region.as_str(),
                "vn1_no_provider": vn1_no_provider,
                "request_id": request_id,
            }),
            ..Default::default()
        }
    }
}
```

Handler refusal path:

```rust
// services/ai-gateway/src/handlers/chat.rs (modified)
match alias::resolve(&req.alias, &policy).await {
    Err(AliasError::ResidencyViolation { policy_residency, resolved_region, vn1_no_provider, attempted_alias }) => {
        memory_writer::emit(canonical::residency_violation(
            &policy.tenant_id, &req.agent_persona, &attempted_alias,
            policy_residency, &resolved_region, vn1_no_provider, &req.request_id,
        )).await?;
        metrics::residency_mismatch(policy_residency, &resolved_region);
        if vn1_no_provider {
            metrics::vn1_refused(&policy.tenant_id);
            tracing::warn!(tenant_id=%policy.tenant_id,
                          "vn1 residency refused; FR-AI-104 Viettel integration needed");
        }
        let body = serde_json::json!({
            "error": "residency_violation",
            "policy_residency": serde_yaml::to_string(&policy_residency).unwrap().trim(),
            "resolved_region": resolved_region.as_str(),
            "reason": if vn1_no_provider { Some("no_vn_provider_yet") } else { None },
            "contact": "ops@cyberos.world",
        });
        return Err(ApiError::Forbidden(body));
    }
    // ... other error variants ...
}
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-006** — `alias::resolve` consumes `residency::matches`. The `AliasError::ResidencyViolation` variant is added to FR-AI-006's enum.
- **FR-AI-005** — Tenant policy schema declares `policy.ai_policy.residency` (Residency enum) and `policy.ai_policy.residency_override` (map of glob → Residency). FR-AI-005's parser delegates to `residency::parse_residency` for the value.
- **FR-AI-015** — ZDR enforcement runs BEFORE residency in `alias::resolve` (precedence in §1 #10). Both errors are surfaced via the same handler path.
- **FR-AI-003** — memory audit-row bridge. This FR adds the `canonical::residency_violation` builder for the `ai.residency_violation` row kind.
- **FR-AI-104 (downstream placeholder)** — VN provider integration (Viettel Cloud + FPT Cloud). Will extend `REGIONS_BY_RESIDENCY[Vn1]` from empty set to a populated set; this FR's `Vn1 → false` rule reverts at that point.

### Concept dependencies (shared types)

- `Residency` enum is the residency primitive used by tenant policy, alias resolve, audit rows, response bodies, OTel metrics.
- `Region` newtype prevents string-typing accidents (e.g., comparing AZ-suffixed vs. region-only strings).
- `REGIONS_BY_RESIDENCY` is the single source of truth for the residency → region mapping; all matchers consult this.
- The precedence (ZDR before residency) is a documented invariant; FRs touching `alias::resolve` must preserve it.

### Operational / external

- Rust crates: `regex@1`, `globset@0.4`, `proptest@1`, `serde@1`, `serde_yaml@0.9`, `thiserror@1`.
- AWS region list is enumerated in `region_table.rs`; if AWS launches a new region in an existing residency tier (e.g., `eu-south-1` for EU), the table is updated via FR amendment.
- `LazyLock` is used for the static map (Rust 1.80+); on older toolchains, `once_cell::sync::Lazy` is the fallback.

---

## §8 — Example payloads

### Tenant policy (with override)

```yaml
# tenants/tenant_alpha/policy.yaml
tenant_id: tenant_alpha
tenant_jurisdiction: VN
ai_policy:
  residency: sg-1                          # default for this VN-jurisdiction tenant
  residency_override:
    "chat.eu-gdpr-*": eu-1                 # specific EU-data aliases pin EU
  zdr_required: true
```

### Caller in FR-AI-006 alias.rs

```rust
let (provider, model, region) = alias::resolve(&req.alias, &policy).await?;
// region.as_str() guaranteed in REGIONS_BY_RESIDENCY[policy.ai_policy.residency]
//                  OR in REGIONS_BY_RESIDENCY[override-resolved residency]
```

### Audit row `ai.residency_violation`

```json
{
  "kind": "ai.residency_violation",
  "ts_ns": 1747526400000000000,
  "payload": {
    "tenant_id": "tenant_alpha",
    "agent_persona": "cuo-cpo@0.4.1",
    "requested_alias": "chat.us-only-alias",
    "policy_residency": "sg-1",
    "resolved_region": "us-east-1",
    "vn1_no_provider": false,
    "request_id": "req_01HZK9R8M3X5C8Q4"
  }
}
```

### HTTP refusal (generic mismatch)

```text
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "error": "residency_violation",
  "policy_residency": "sg-1",
  "resolved_region": "us-east-1",
  "contact": "ops@cyberos.world"
}
```

### HTTP refusal (Vn1 no-provider)

```text
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "error": "residency_violation",
  "policy_residency": "vn-1",
  "resolved_region": "ap-southeast-1",
  "reason": "no_vn_provider_yet",
  "contact": "ops@cyberos.world"
}
```

### Override-applied trace log

```text
INFO  alias=chat.eu-gdpr-summary tenant_default=sg-1 override_matched=eu-1
      residency_override applied
```

### Ambiguous-override policy load failure

```text
ERROR policy file tenants/tenant_beta/policy.yaml load failed:
      AmbiguousResidencyOverride { aliases: ["chat.*-customer-*", "chat.eu-*"], for_alias: "chat.eu-customer-summary" }
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- **FR-AI-104 (placeholder)**: VN provider integration (Viettel Cloud + FPT Cloud). Will populate `REGIONS_BY_RESIDENCY[Vn1]` and remove the §1 #6 fail-closed rule.
- AZ-aware policies (rare regulatory ask: "must be in `us-east-1` AZ a"; e.g., for some federal compliance regimes) — out of scope; AZ stripping is the slice 4 boundary.
- Per-region cost-aware routing (route to the cheapest region within a residency) — slice 5; current resolver picks the alias's primary region.
- Multi-residency tolerance (`residency: [sg-1, eu-1]` accepting either) — out of scope; current model is single-residency-per-tenant + per-alias overrides for exceptions.
- Residency-aware fallback during regional outage (AWS Singapore down → fall back to AWS Tokyo for `Sg1`?) — explicitly OUT of scope; residency violation is unrecoverable; refusing is the correct behaviour.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Tenant pins `vn-1` but no VN provider | `Vn1` empty set always returns false | `Err(ResidencyViolation { vn1_no_provider: true })` → `403` with `reason: no_vn_provider_yet` | Tenant chooses `sg-1` (closest acceptable) OR waits for FR-AI-104 |
| New AWS region not in matcher | Region returns false (fail closed) | `Err(ResidencyViolation)` for affected residencies | Operator adds region to `region_table.rs`; FR amendment; redeploy |
| Property test detects cross-residency leak | proptest panics in CI | PR blocked | Fix `region_table.rs` (likely accidental aliasing across residencies) |
| Region string with AZ suffix | `Region::from_provider_string` strips suffix | Match proceeds against region-only string | By design (§1 #5) |
| Region string in unknown format | `RegionParseError::Invalid` from regex | `Err` propagates; alias resolve fails with `RegionParseError` (not residency violation) | Operator investigates provider response; likely a provider API change |
| Tenant policy without `residency` field, non-PDPL | FR-AI-005 schema default → `Sg1` | Enforce against `Sg1` | Operator updates policy if non-default residency intended |
| Tenant policy without `residency` field, PDPL tenant | FR-AI-005 schema rejects | `PolicyError::MissingResidencyForPdplTenant` → 503 at policy load | Operator must explicitly pin residency in policy |
| Tenant policy with `residency: apac-2` (invalid) | `parse_residency` rejects | `ResidencyParseError::Invalid` → policy load fails | Operator fixes value to one of sg-1, eu-1, us-1, vn-1 |
| Per-alias override glob matches no aliases | `resolve_override` returns None → fall back to default | Tenant default residency applies | By design |
| Per-alias override glob matches multiple aliases ambiguously | `resolve_override` → `OverrideAmbiguous` | `PolicyError::AmbiguousResidencyOverride` → 503 at policy load | Operator narrows glob patterns |
| Per-alias override glob has invalid syntax | `Glob::new` parse error | `PolicyError::InvalidOverrideGlob` → 503 at policy load | Operator fixes glob pattern |
| ZDR + residency both fail | Precedence: ZDR fires first (§1 #10) | `Err(ZdrViolation)` returned; residency check never runs | By design; operator sees ZDR error first |
| Audit row emit fails (memory bridge down) | `memory_writer::emit` returns Err | Refusal still proceeds; sev-1 log "residency refused but audit row failed" | Operator investigates memory; FR-AI-003 §10 covers |
| Concurrent `matches()` calls | LazyLock + immutable HashSet | All readers see same result | By design (§1 #9 deterministic) |
| Region table mutation attempted at runtime | LazyLock prevents writes | Compile error if attempted | By design |
| FR-AI-104 lands but `Vn1` set still empty | Test `test_vn1_set_populated_after_fr_ai_104` fails | CI blocked on FR-AI-104 PR | FR-AI-104 must extend the table as part of its acceptance |
| Alias is in alias map but its region is invalid format | `Region::from_provider_string` fails at alias-load time | Alias load fails; alias unavailable | Operator fixes alias map entry |
| Tenant overrides residency to a value with no acceptable provider | Override resolves to (e.g.) `Vn1` for an alias whose only region is `us-east-1` | `ResidencyViolation` with override-resolved residency | Operator removes override OR waits for VN provider |
| Tenant onboarding skipped residency selection | FR-AI-005 onboarding wizard enforces residency | Cannot complete onboarding without selection | By design |
| OTel metric label cardinality explosion (per-tenant_id × per-region) | Cardinality monitoring | Metric drop / sample | Operator scales OTel collector OR aggregates labels |

---

## §11 — Notes

- The Vn1 placeholder is intentional and load-bearing. Adding Viettel/FPT as Provider variants is FR-AI-104; until then, refusing Vn1 calls is the correct regulatory behaviour. The metric `ai_residency_vn1_refused_total` quantifies the demand for FR-AI-104.
- The Sg1 single-region pinning may relax in future if AWS adds another AP region with similar latency to Singapore (currently `ap-southeast-3` Jakarta is closest but not yet covered for Bedrock). Any addition requires FR amendment.
- The static enum + `LazyLock` map design trades hot-reloadability for type-system safety. Residency tiers map to legal jurisdictions (slow-changing); region additions within tiers are FR amendments. This is the right trade vs. ZDR (FR-AI-015), which uses a YAML config because attestations DO drift quarterly.
- The precedence rule (ZDR before residency, §1 #10) is a small but important UX choice. An operator getting "ZDR violation" first for a tenant who fails both gates can fix ZDR (by routing to a ZDR-attested provider) and discover the residency issue separately. The opposite order would mask the ZDR failure under a region failure.
- The per-alias override mechanism (§1 #11) is the answer to "we have ONE alias that needs to behave differently." Without it, tenants face the all-or-nothing choice. The `OverrideAmbiguous` rejection prevents the operational footgun of two glob patterns silently fighting.
- The AZ-strip rule (§1 #5) is conservative. Provider responses occasionally surface AZ-suffixed regions (in error messages, in routing metadata); the matcher operates at region granularity per the legal definition. AZ-aware policies are explicitly OUT of scope.
- The `Vn1` empty-set design (rather than mapping to "closest acceptable") is the honesty principle. A tenant pinning `vn-1` is making a regulatory statement; satisfying that statement requires VN-located infrastructure. Silently downgrading to SG would convert a regulatory failure into a silent failure, which is strictly worse — the tenant would assume compliance when it doesn't exist.
- The audit row (`ai.residency_violation`) is the proof-of-refusal primitive matching FR-AI-015's `ai.zdr_violation`. Both rows answer "did we ever route protected data outside its required jurisdiction" with positive evidence rather than absence-of-evidence.
- The property test runs 1000 trials by default. At CI time, this is ~50ms — trivial. The catch is "no cross-residency aliasing" — a property that's hard to write as a unit test (would need 4 × ~30 = 120 manual cases) and easy to break (one copy-paste mistake adds a region to two residencies).
- Future regulatory complexity (multi-residency tolerance, AZ-aware policies, regional cost optimisation) is consciously deferred. Slice 4 ships the minimum viable enforcement; slice 5+ adds nuance once we have operational data on which extensions matter.

---

*End of FR-AI-016. Status: draft (10/10 target).*
