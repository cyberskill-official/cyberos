---
task_id: TASK-AI-015
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (234 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-012 depth (~870 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

TASK-AI-015 was expanded from 234 lines to ~870 lines matching TASK-AI-012 / TASK-AI-014 depth.

The expansion added 7 §1 normative clauses (#4 mandatory fields validated at parse, #6 `ai.zdr_violation` memory row builder, #9 two-tier soft-stale/hard-stale with auto-override, #10 HTTPS-only source-URL validation, #11 attested-by domain allow-list, #13 HTTP 403 response shape with notes-scrubbing, #15 reload INFO log), 6 substantive §2 rationale paragraphs (provenance-as-evidence frame, HTTPS-only MITM argument, two-tier staleness SOC 2 alignment, revocation-as-operational-signal rationale, attestor-vs-git-blame argument, enforcement-at-alias-resolution-not-precheck rationale, dedicated-audit-row evidence frame), full Rust type definitions in §3 (`ZdrAttestation`, `LoaderInitError` variants, `AliasError::ZdrViolation` extension), full parser with all four required-field checks + URL validation + attestor validation in §3, `staleness.rs` module in §3 with soft/hard checks, `is_zdr` implementation with hard-stale override in §3, full `.github/workflows/zdr-staleness-check.yml` workflow in §3, expanded §4 from 7 to 19 acceptance criteria, full Rust test bodies in §5 (happy + fail-closed + alias integration + audit-row + HTTPS-rejection + bare-attestor-rejection + missing-field-rejection + revocation-with-metric + soft-stale + hard-stale-override), `watch.rs` skeleton with diff-detect revocation in §6, `canonical::zdr_violation` builder in §6, expanded §7 with code/concept/operational dep split, 8 example payloads in §8 (caller, audit row, HTTP refusal, attestation lookup, INFO + WARN + ERROR logs, CI output), 22 failure modes in §10 (vs. 5 in first pass), 9 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — `(ProviderKind, String)` HashMap key uses `*provider` deref → assumes ProviderKind: Copy

- **severity:** error
- **rule_id:** correctness / type-trait assumption
- **location:** §3 (HashMap key), §6 (is_zdr skeleton)
- **status:** resolved

#### Description

The first-pass §6 had:

```rust
ATTESTATIONS.get()
    .and_then(|s| s.load().get(&(*provider, model.to_string())).map(|a| a.is_zdr))
```

The `*provider` deref assumes `ProviderKind: Copy`. TASK-AI-008's `ProviderKind` enum may not derive `Copy` (it currently does, but a future variant carrying a `String` field — e.g., `Custom(String)` — would break the deref). The fragile assumption is invisible in the spec; a code-gen agent that adds a `Custom(String)` variant later breaks `is_zdr` with no spec-level signal.

#### Suggested fix

Either:
1. Document that `ProviderKind: Copy` is a load-bearing invariant in §7 (concept dependencies); add a `#[derive(Copy, Clone)]` requirement to TASK-AI-008's enum spec; OR
2. Change the key to `(ProviderKind, String)` with `Clone`-based access via reference-friendly lookup.

We adopt (2) for forward-compatibility: the HashMap key is built fresh from `(provider.clone(), model.to_string())` at lookup. The micro-cost (one enum clone per lookup, ~10ns) is negligible vs. the fragility cost.

### ISS-002 — AC #6 "revocation warns" lacks a test body in §5

- **severity:** error
- **rule_id:** test-coverage
- **location:** §4 AC #6, §5 (verification)
- **status:** resolved

#### Description

First-pass §5 had stubbed test placeholders:

```rust
#[tokio::test]
async fn revocation_warns() { /* AC #6 */ }
```

No mechanism shown for: (a) how the test mutates the YAML, (b) how the test waits for the hot-reload, (c) how the test asserts `tracing::warn!` fired (capture vs. metric), (d) how it asserts the metric increment.

This is the same shape as TASK-AI-007 ISS-001, TASK-AI-012 ISS-001, TASK-AI-013 ISS-002 — ACs reference behaviors without matching test bodies. A code-gen agent has no template.

#### Suggested fix

Replace the stub with a complete `tokio::test` body that (a) writes a revoked-bool version of the YAML to disk, (b) sleeps 500ms for hot-reload, (c) asserts `is_zdr` flipped to false, (d) asserts the OTel counter `ai_zdr_attestations_revoked_total{provider, model}` increment. Pattern matches TASK-AI-014's `test_tamper_detection_fires_with_metric`.

### ISS-003 — Staleness CI lint claimed in §10/§11 but no implementation shown

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** §10 row + §11 note (claim), §3/§5/§6 (no implementation)
- **status:** resolved

#### Description

The first-pass said in §10:

> *"Stale attestation (verified_at > 90 days old) | CI lint flag | PR warning (not blocking) | Operator refreshes attestation quarterly."*

And §11:

> *"The 90-day staleness CI lint matches industry SOC 2 cadence for vendor reassessment."*

But: §1 doesn't define the threshold, §3 doesn't define the check function, §5 doesn't test it, §6 doesn't implement it, and `new_files` doesn't include the workflow file. The lint is a recommendation; nothing in the FR makes it real.

Worse, the "PR warning (not blocking)" model is the wrong default: a 91-day-old attestation might still be true (vendor policy unchanged), but it might not (vendor policy changed and no one noticed). Soft warning at 90d is fine; the spec is missing the hard-stop at some longer threshold (matching SOC 2's annual cadence).

#### Suggested fix

1. Add §1 #9 two-tier staleness (soft 90d → CI warn; hard 365d → `is_zdr` forced to false).
2. Add `staleness.rs` module in §3 + §6 with `is_soft_stale`, `is_hard_stale`.
3. Modify `is_zdr` in §3/§6 to apply the hard-stale override.
4. Add `.github/workflows/zdr-staleness-check.yml` weekly cron in `new_files` + §3.
5. Add AC #10 (soft-stale flagged in CI) + AC #11 (hard-stale forces false) in §4.
6. Add §5 tests for both thresholds.
7. Add §10 rows for both staleness paths.

### ISS-004 — source_url not validated; any string accepted

- **severity:** warning
- **rule_id:** input validation / security boundary
- **location:** §1 #4 (claim "informational"), §3 (no validator), §6 (no validator)
- **status:** resolved

#### Description

The first-pass §1 #4 said the row carries `source_url` "as informational." But the source_url IS the audit primitive — the URL is the evidence-of-attestation. Accepting any string means a malicious or careless commit could ship an entry citing `http://example.com/forged-page` or `not-a-url` and the gate would still allow ZDR-attested routing through it.

A regulator reviewing the audit trail expects URLs to (a) resolve to publisher-controlled pages, (b) use HTTPS so a MITM can't forge the page. Neither is enforced.

#### Suggested fix

1. Add §1 #10 normative requirement: HTTPS-only source URLs; bare paths and HTTP URLs rejected at parse.
2. Add `validate_source_url` in §3 + §6 using the `url` crate.
3. Add `url@2` to Cargo.toml in `modified_files`.
4. Add AC #12 (HTTP source_url rejected at parse) in §4.
5. Add §5 test asserting HTTP rejection.
6. Add §2 rationale paragraph explaining the MITM-floor argument.

### ISS-005 — `attested_by` not validated; "alice" is accepted

- **severity:** warning
- **rule_id:** input validation / attribution discipline
- **location:** §1 #4 (claim), §3/§5/§6 (no validator)
- **status:** resolved

#### Description

The first-pass shipped attestations like `attested_by: stephen@cyberos.world` (with email format) but the spec didn't require email format. A future commit could ship `attested_by: alice` or `attested_by: ops-team` — both pass parse, neither tells an auditor who the attestor is. The "who do I email to question this" answer becomes "we don't know."

Plus: even with email format, allowing any domain (`attested_by: alice@gmail.com`) doesn't establish CyberSkill or approved-auditor identity. The attestor MUST be either CyberSkill staff or a recognised third-party auditor.

#### Suggested fix

1. Add §1 #11 normative requirement: `<localpart>@<approved-auditor-domain>` format.
2. Add `APPROVED_AUDITOR_DOMAINS` constant in `parse.rs` with `cyberos.world` + initial third-party SOC 2 firms.
3. Add `validate_attested_by` in §3 + §6.
4. Add ACs #13 (bare-string rejected) + #14 (out-of-domain rejected) in §4.
5. Add §5 tests for both rejections.
6. Add §2 rationale paragraph on attestor-vs-git-blame.

### ISS-006 — Hot-reload of malformed file behaviour undefined

- **severity:** warning
- **rule_id:** robustness / fail-safe semantics
- **location:** §1 #5 (hot-reload claim), §10 (no row)
- **status:** resolved

#### Description

The first-pass §1 #5 said "MUST be hot-reloadable via `notify`." But what happens if the new YAML has a parse error?

Three possible behaviours:
(a) Cache cleared (`is_zdr` returns false for everything) — loud failure but blocks ALL ZDR-required requests.
(b) Cache unchanged (continues serving old data) — silent failure; operator might not notice.
(c) Cache cleared AND gateway exits — loud + impactful.

The spec doesn't pick. TASK-AI-005 and TASK-AI-007's hot-reload patterns use behaviour (b) with a WARN log; this FR should too, but it's not explicit.

#### Suggested fix

1. Make §1 #7 explicit: parse error on hot-reload leaves cache unchanged; WARN log + metric increment.
2. Add §5 test asserting parse-error keeps cache.
3. Add §10 row: "YAML parse error at hot-reload | Reload fails; cache unchanged | INFO log 'reload failed'; metric `reload_failure_total` | Operator fixes YAML; next file-watch event triggers retry."
4. Match TASK-AI-014's `test_hot_reload_of_malformed_file_leaves_cache_unchanged` pattern.

## §3 — Strengths preserved through expansion

- §3 introduces `LoaderInitError` with distinct variants for `Schema`, `InvalidSourceUrl`, `InvalidAttestor`, `AlreadyInitialised` — each variant is a different remediation path, making error-handling code in the boot path explicit about which failure class it's handling.
- §1 #6 introduces a dedicated `ai.zdr_violation` memory row kind. This is the proof-of-refusal primitive: a regulator asking "did you ever route PDPL data to a non-ZDR provider for tenant X" gets a positive answer (rows showing the refusal) rather than an absence-of-evidence answer.
- §1 #9 introduces the two-tier staleness model (90d soft, 365d hard) with the hard-tier forcing `is_zdr=false` regardless of recorded value. This is the defence-in-depth that converts "we forgot to reverify" from a silent failure to a loud refusal.
- §1 #10 + #11 make `source_url` and `attested_by` validated at parse — accepting only HTTPS URLs and approved-domain attestors. Both validations are cheap (parser-level) but materially raise the audit-grade of the table.
- §3 specifies the memory audit row builder (`canonical::zdr_violation`) inside this FR, not punting to TASK-AI-003. The owning-FR-builds-the-builder pattern matches TASK-AI-014's `canonical::persona_loaded` fix.
- §10 inventory grew from 5 rows to 22 — including the field-validation paths (HTTPS, bare-attestor, missing field), the hot-reload-parse-error path, the audit-emit-fails path, the watcher-thread-panic path, and the notes-leak-in-response-body path. Each row has an unambiguous detection mechanism.
- §11 documents the Anthropic "Enterprise plan only" caveat AS A KNOWN GAP with a clear TASK-AI-022 follow-up. The honesty about scope is important — ops needs to manually verify Enterprise tier per tenant during onboarding until the runtime check ships.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the FR itself:

- **ISS-001 RESOLVED**: HashMap key access uses `(provider.clone(), model.to_string())` — no `Copy` dependency on `ProviderKind`. Skeleton in §6 updated; §7 documents the clone-per-lookup as a deliberate forward-compatibility choice.

- **ISS-002 RESOLVED**: §5 now has `revocation_warns_and_metricises` with the full pattern (write-revoked-YAML → sleep-for-hot-reload → assert is_zdr false → assert OTel counter increment → restore). Matches the TASK-AI-014 tamper-test structure.

- **ISS-003 RESOLVED**: §1 #9 added with two-tier 90d/365d staleness; `staleness.rs` shown in §3/§6 with `is_soft_stale`/`is_hard_stale`; `is_zdr` modified to apply hard-stale override; `.github/workflows/zdr-staleness-check.yml` weekly cron added to `new_files` and §3; ACs #10 + #11 added; §5 has tests for both thresholds; §10 has rows for both paths.

- **ISS-004 RESOLVED**: §1 #10 added; `validate_source_url` using the `url` crate shown in §3 + §6; `url@2` added to `modified_files` Cargo.toml; AC #12 added; §5 has `http_source_url_rejected` test; §2 has the MITM-floor rationale.

- **ISS-005 RESOLVED**: §1 #11 added; `APPROVED_AUDITOR_DOMAINS` constant + `validate_attested_by` shown in §3 + §6; ACs #13 + #14 added; §5 has `bare_string_attestor_rejected` and out-of-domain rejection tests; §2 has the attestor-vs-git-blame rationale.

- **ISS-006 RESOLVED**: §1 #7 explicit on hot-reload-parse-error keeping cache unchanged; §6 `reload_with_diff` shows the WARN-and-skip pattern; §10 row added; §5 (via the watch test framework) covers the path. TASK-AI-014's hot-reload-malformed pattern reused.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-015 audit (final). Status: PASS at 10/10.*
