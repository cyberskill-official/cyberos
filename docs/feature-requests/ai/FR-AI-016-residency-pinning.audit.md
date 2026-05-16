---
fr_id: FR-AI-016
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.0/10        # the first-pass compressed version (162 lines)
score_post_expansion: 9.0/10      # after expanding to FR-AI-012 / FR-AI-015 depth (~890 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

FR-AI-016 was expanded from 162 lines (the smallest first-pass in slice 4) to ~890 lines matching FR-AI-012 / FR-AI-015 depth.

The expansion added 8 §1 normative clauses (#4 default-residency fallback for missing field with PDPL-tenant special case, #5 AZ-suffix stripping with regex, #7 `ai.residency_violation` BRAIN row, #8 HTTP 403 response shape with Vn1-specific reason field, #10 explicit ZDR→residency precedence, #11 per-alias residency_override mechanism with ambiguity rejection, #12 explicit no-silent-Vn1-degrade rule, #14 WARN log on Vn1 refusals for FR-AI-104 prioritisation), 7 substantive §2 rationale paragraphs (residency-as-unrecoverable framing with regulator-specific cost analysis, fail-closed-PDPL conservative-default rationale, static-enum-vs-config trade-off, Vn1-empty-set honesty principle, AZ-strip rule applied to legal jurisdiction granularity, ZDR-before-residency precedence diagnostic argument, per-alias override surgical-control rationale, audit-row positive-evidence frame, vn1-metric demand-signal frame, property-test global-correctness rationale), full Rust type system in §3 (Residency enum with serde rename, Region newtype with regex AZ-strip, ResidencyParseError, RegionParseError, AliasError::ResidencyViolation extension, REGIONS_BY_RESIDENCY LazyLock, ResidencyOverride + OverrideError + resolve_override), expanded §4 from 8 to 18 acceptance criteria, full Rust test bodies in §5 (happy + AZ-strip + invalid-region + parse + property-test 1000-trial + integration with override + Vn1 metric assertion + audit-row + ZDR-precedence + ambiguous-override-rejected), full integration skeleton in §6 (alias::resolve modified to invoke ZDR-then-residency, canonical::residency_violation builder, handler refusal path with metric + WARN + body assembly), expanded §7 with code/concept/operational dep split, 7 example payloads in §8 (tenant policy with override, alias caller, audit row, two HTTP refusal variants, override-applied trace, ambiguous-override load failure), 19 failure modes in §10 (vs. 3 in first pass), 9 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — `AliasError::ResidencyViolation` referenced in AC #8 but not defined in §3 type signatures

- **severity:** error
- **rule_id:** spec-completeness / type-vs-test mismatch
- **location:** §3 (no enum extension shown), §4 AC #8 (uses the variant)
- **status:** resolved

#### Description

The first-pass §4 AC #8 said: *"Integration test: Policy `residency: sg-1`; resolve `chat.smart` → primary is `bedrock us-east-1` → `Err(ResidencyViolation)`."* But the §3 API contract listed only `pub fn matches(...)` and the `Residency` enum — no `AliasError::ResidencyViolation` variant, no fields, no error type definition.

A code-gen agent reading the spec has no way to know what fields the variant carries or what error chain it joins. The handler-side conversion (this AliasError variant → HTTP 403) is also undefined.

#### Suggested fix

Add to §3 the explicit `AliasError::ResidencyViolation { policy_residency, resolved_region, attempted_alias, vn1_no_provider }` variant as an extension to FR-AI-006's enum. Document the fields. Show how the handler maps it to HTTP 403.

### ISS-002 — Override mechanism mentioned in AC #8 ("Override to bedrock ap-southeast-1 → Ok") but not specified anywhere

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** §4 AC #8 (mentions "override"), §1 + §3 + §5 + §6 (no override mechanism defined)
- **status:** resolved

#### Description

The first-pass AC #8 wrote: *"Override to `bedrock ap-southeast-1` → `Ok`."* But the FR doesn't define what "override" means — is it a per-tenant config, a per-call request field, a different alias name? Without a spec, this AC is untestable; a code-gen agent cannot implement what isn't specified.

The pattern is real (a tenant occasionally needs one alias to behave differently from the tenant default), so the override mechanism is genuinely useful — but it has to be a first-class normative feature, not an aside in an AC.

#### Suggested fix

Add §1 #11 normative requirement: per-alias `residency_override` map in tenant policy (`{"<glob>": "<residency>"}`). Add `parse::resolve_override` in §3 + §6. Add ambiguity rejection (`OverrideAmbiguous`). Add ACs #16 (override applies before default) and #17 (ambiguous override rejected at policy load). Add §5 tests for both. Add §10 rows for the override-related failure modes.

### ISS-003 — No `ai.residency_violation` audit row; FR-AI-015 has the analogue but FR-AI-016 lacks it

- **severity:** error
- **rule_id:** spec-completeness / cross-FR consistency
- **location:** §1 (no clause), §3 (no builder), §6 (no emission)
- **status:** resolved

#### Description

FR-AI-015 §1 #6 mandates an `ai.zdr_violation` BRAIN row on every ZDR refusal — the proof-of-refusal primitive. The first-pass FR-AI-016 has no equivalent for residency violations.

A regulator asking "did you ever route VN PII outside Vietnam for tenant X" needs a positive-evidence answer (rows showing the refusal proves we caught it) rather than absence-of-evidence (we didn't see any data flow, but how do we prove we tried?). Without the row, residency refusals are invisible to the audit chain.

The two compliance gates (ZDR + residency) MUST have parallel audit primitives.

#### Suggested fix

Add §1 #7 normative requirement: emit `ai.residency_violation` BRAIN row on every refusal. Add `canonical::residency_violation` builder in §3 + §6. Add AC #13 asserting emission. Add §5 test asserting the row's payload. Add §10 row for "Audit row emit fails (BRAIN bridge down)."

### ISS-004 — Region string format unspecified; AZ suffix handling undefined

- **severity:** error
- **rule_id:** correctness / input handling
- **location:** §1 (no format clause), §3 (no Region type)
- **status:** resolved

#### Description

The first-pass §3 used `provider_region: &str` everywhere. No specification of:
1. What format the string takes (`ap-southeast-1` vs. `ap-southeast-1a` vs. `arn:aws:bedrock:ap-southeast-1:...`).
2. Whether AZ suffix is acceptable.
3. What happens for unknown formats.

Provider SDKs (Bedrock, Vertex) inconsistently surface region — sometimes region-only, sometimes AZ-suffixed, sometimes ARN-prefixed. A bare `&str` matcher would silently mismatch on AZ-suffixed regions ("ap-southeast-1a" ≠ "ap-southeast-1") and produce false ResidencyViolation refusals.

This is the same string-typing-accident class that ISS-001 fixed via the `PersonaHandle` newtype in FR-AI-014.

#### Suggested fix

1. Introduce `Region` newtype in §3 with `from_provider_string` constructor.
2. Add §1 #5 normative requirement: AZ suffix stripped via regex `^(?P<region>[a-z]{2}-[a-z]+-\d+)[a-z]?$`.
3. Add `RegionParseError::Invalid` for unknown formats.
4. Update `matches()` signature to take `&Region`, not `&str`.
5. Add ACs #6 (AZ-strip works) + #7 (invalid format rejected) in §4.
6. Add §5 test for both cases.

### ISS-005 — ZDR + residency precedence undefined; either could fire first

- **severity:** error
- **rule_id:** correctness / cross-gate ordering
- **location:** §1 (no precedence clause), §6 (no integration shown)
- **status:** resolved

#### Description

The first-pass FR doesn't specify whether ZDR or residency runs first in `alias::resolve`. Both are compliance gates; both refuse calls. Order matters because:
1. The operator dashboard sees the first-fired error.
2. Audit rows are emitted for whichever fires.
3. Tests asserting "ZDR refusal" vs. "residency refusal" can't be written without knowing the order.

Worse, undefined precedence means two engineers implementing this concurrently could pick different orders, leading to test failures depending on which test runs first.

#### Suggested fix

Add §1 #10: ZDR fires before residency. Document the diagnostic rationale in §2 (ZDR is the more diagnostic/restrictive gate; fixing ZDR often resolves residency too). Add AC #18 asserting ZDR fires first when both fail. Add §5 test for the ordering. Show the order explicitly in §6's `alias::resolve` skeleton.

### ISS-006 — Property test (§5) just says "ensure pairs from different families don't match"; not concretely assertable

- **severity:** warning
- **rule_id:** test-coverage / proptest specification
- **location:** §5 property test stub
- **status:** resolved

#### Description

First-pass §5 had:

```rust
#[test]
fn property_no_cross_residency() {
    proptest!(|(r in any_residency(), reg in any_region())| {
        // ensure pairs from different families don't match
    });
}
```

Three problems:
1. The body is a comment, not an assertion.
2. "Pairs from different families" is undefined — no formal property.
3. `any_residency()` and `any_region()` strategies aren't shown.

A code-gen agent has nothing to implement.

#### Suggested fix

Replace the stub with a complete proptest body asserting the formal property: `if matches(R, region) is true, then region.as_str() must be in REGIONS_BY_RESIDENCY[R]`. This catches the "accidental aliasing" class (e.g., adding `eu-central-1` to both Eu1 and Us1 by copy-paste).

Show the strategy definitions (`any_residency`, `any_region`). Set `ProptestConfig::with_cases(1000)`. Add a second proptest property for determinism (`matches(R, region) twice → same result`).

## §3 — Strengths preserved through expansion

- §3 introduces `Residency` (enum) and `Region` (newtype) as distinct, parseable types — the type-system separation prevents string-typing accidents (e.g., comparing `"ap-southeast-1a"` against `"ap-southeast-1"` and silently mismatching).
- §1 #2 commits to the `LazyLock<HashMap<Residency, HashSet<&'static str>>>` static map. The "changes require FR amendment" discipline keeps the residency mapping stable across deploys; operators can't quietly add regions and break compliance assumptions.
- §1 #4 makes the missing-residency-field default explicit per tenant jurisdiction: PDPL tenants fail closed (no implicit fallback), non-PDPL tenants default to `Sg1`. The tier discrimination prevents accidentally relaxing PDPL constraints.
- §1 #6 + §1 #12 explicitly forbid silent `Vn1 → Sg1` degradation. The honesty principle: a tenant pinning `vn-1` is making a regulatory statement, and we cannot satisfy that statement without VN infrastructure. Refusing is correct; silently fallback is wrong.
- §1 #11 introduces per-alias residency_override with `OverrideAmbiguous` rejection. Surgical control without the operational footgun of competing globs.
- §10 inventory grew from 3 rows to 19 — including the AZ-strip path, the missing-field-PDPL-fail-closed path, the override-glob-syntax-error path, the audit-emit-fails path, the FR-AI-104-lands-but-Vn1-still-empty regression-test path, and the metric-cardinality-explosion path. Each row has an unambiguous detection mechanism.
- §11 documents the static-enum-vs-config-driven trade-off explicitly: residency tiers map to legal jurisdictions (slow-changing) → static enum is right; ZDR attestations drift quarterly → YAML config is right. Future engineers don't second-guess the asymmetry.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the FR itself:

- **ISS-001 RESOLVED**: §3 now defines `AliasError::ResidencyViolation { policy_residency, resolved_region, attempted_alias, vn1_no_provider }` as an extension to FR-AI-006's enum; §6 shows the handler-side conversion to HTTP 403; §8 has the response payload example.

- **ISS-002 RESOLVED**: §1 #11 added with the `residency_override: { "<glob>": "<residency>" }` map; `parse::resolve_override` in §3 + §6; `OverrideAmbiguous` rejection; ACs #16 + #17 added; §5 has `per_alias_override_wins_over_tenant_default` and `ambiguous_override_rejected_at_policy_load` tests; §8 example shows the policy YAML.

- **ISS-003 RESOLVED**: §1 #7 added; `canonical::residency_violation` builder in §3 + §6; AC #13 asserts emission; §5 has `audit_row_emitted_on_residency_refusal` test; §10 has the "Audit row emit fails" row.

- **ISS-004 RESOLVED**: `Region` newtype in §3 with regex AZ-strip; §1 #5 normative; ACs #6 + #7 added; §5 has `az_suffix_stripped` and `invalid_region_string_rejected` tests; matcher signature updated to take `&Region`.

- **ISS-005 RESOLVED**: §1 #10 explicit (ZDR before residency); §2 has the diagnostic rationale paragraph; AC #18 asserts the ordering; §5 has `zdr_check_fires_before_residency` test; §6's `alias::resolve` skeleton shows ZDR check FIRST, then override resolution, then residency match.

- **ISS-006 RESOLVED**: §5 property test now has a complete body asserting `matches(R, region) is true → region in REGIONS_BY_RESIDENCY[R]`; strategies (`any_residency`, `any_region`) shown; `ProptestConfig::with_cases(1000)` configured; second property (`deterministic`) added.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of FR-AI-016 audit (final). Status: PASS at 10/10.*
