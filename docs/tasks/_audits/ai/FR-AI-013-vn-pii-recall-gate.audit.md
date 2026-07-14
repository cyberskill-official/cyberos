---
task_id: TASK-AI-013
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (238 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-012 depth (~780 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

TASK-AI-013 was expanded from 238 lines (compressed first-pass) to ~780 lines matching TASK-AI-001 / TASK-AI-012 depth.

The expansion added 6 §1 normative clauses (#7 structured failure assertion contents, #8 workflow self-gate via `paths:` self-inclusion, #10 recognizer-version pin via `fixture_manifest.yaml`, #11 offline-only execution, #12 JSON artefact emission, #14 pre-commit real-PII hook, #15 fixture-format validation, #16 trend-chart hook), 7 substantive §2 rationale paragraphs (precision-vs-recall Bayesian trade-off, quarterly regen mechanism, version-pin lock-step, pre-commit-hook fail-closed-in-CI / warn-only-locally pattern, negative-sample collection rationale, runtime-budget forcing function, JSON-artefact audit primitive, workflow self-gate principle), full YAML fixture format + fixture_manifest.yaml in §3, recall-gate CI-output template in §3, expanded §4 from 8 to 18 acceptance criteria, full pytest bodies in §5 for `test_recall_gate.py` (per-recognizer + aggregate + version-pin + counts + negative-samples), `test_precision_warning.py` (no-gate; baseline-delta warning), `test_fixture_invariants.py` (schema + span offsets + no-duplicate-ids), pre-commit hook shell + Python implementation, `test_recall_gate_runtime_budget.py` subprocess-based budget guard, full `vn-pii-recall.yml` + `vn-pii-quarterly-refresh.yml` workflow files, `Makefile` additions, `regen_fixture.py` skeleton, expanded §7 (code/concept/operational dependency split), 5 example payloads in §8 (success artefact, failure artefact, precision warning, positive + negative samples, quarterly issue body), 24 failure modes in §10 (vs. 4 in the first pass), 11 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — Fixture path/format diverges from TASK-AI-012 (FR coupling drift)

- **severity:** error
- **rule_id:** consistency / cross-FR coupling
- **location:** frontmatter `new_files`, §3 fixture format, §5 test bodies
- **status:** resolved

#### Description

The compressed first-pass declared the fixture at `services/ai-gateway/pii/tests/vn_pii_corpus.jsonl` (JSONL, in the `tests/` directory). TASK-AI-012 (the upstream this FR depends_on) places the fixture at `services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml` (YAML, in `fixtures/`), and TASK-AI-012's `test_recall_floor.py` reads from that path.

A consumer (TASK-AI-013's CI gate) and a producer (TASK-AI-012's fixture-curation tooling) disagreeing on the fixture path is the cleanest possible recipe for "the gate ran against a non-existent file and silently passed because pytest collection emitted 0 tests."

This is the same coupling-drift pattern as TASK-AI-006 ISS-001 (province-code list duplicated across recognizers).

#### Suggested fix

Adopt TASK-AI-012's path. Update:

- frontmatter `new_files` → `fixtures/vn_pii_200_samples.yaml` + `fixtures/vn_pii_200_samples_README.md` + `fixtures/fixture_manifest.yaml`.
- §3 fixture format → YAML matching TASK-AI-012 §6's `expected_entities` / `expected_count` shape, with `expected_spans` added for span-offset validation.
- §5 tests → read YAML, not JSONL; align constants with TASK-AI-012's `recognizers/confidence.py` exports.

### ISS-002 — Per-recognizer recall floor declared but not tested

- **severity:** error
- **rule_id:** test-coverage
- **location:** §1 #5 (per-recognizer), §5 (verification)
- **status:** resolved

#### Description

The compressed first-pass had §1 #5 saying *"recall ≥ 99% per recognizer, not aggregate"* — but the §5 test (`test_recall_per_recognizer`) computed `counts_by_entity[...]["tp"] / (tp + fn)` and asserted only that recall per type ≥ 0.99. It missed three things:

1. The aggregate floor (TASK-AI-012 §1 #1 post-ISS-004 requires BOTH per-type AND aggregate).
2. A structured failure message identifying which recognizer failed and which sample IDs were missed.
3. A guard against zero-denominator (a recognizer with zero samples in a fixture).

A code-gen agent reading the FR has no template for the failure-message shape; the gate fails with `AssertionError: ` and no payload.

#### Suggested fix

Rewrite §5's recall test to (a) compute per-recognizer recall, (b) compute aggregate, (c) collect failures into a list with full context (entity, recall value, missed sample IDs), (d) assert with formatted message.

This is the same pattern as TASK-AI-012 §5's `test_recall_at_least_99_percent_per_type` after TASK-AI-012's ISS-004 fix. Reuse the structure verbatim and extend with aggregate + version-pin checks.

### ISS-003 — No recognizer-version pin; fixture and recognizers can drift undetected

- **severity:** error
- **rule_id:** correctness / lock-step versioning
- **location:** §1 (no clause), §3 (no manifest), §5 (no version-match test)
- **status:** resolved

#### Description

TASK-AI-012 §1 #14 exposes `GET /recognizers/version` returning per-entity semver strings precisely so downstream tests can pin against them. The compressed first-pass of TASK-AI-013 doesn't reference the endpoint and doesn't pin versions.

Failure mode: TASK-AI-012's `VnCccdRecognizer` bumps from 1.0.0 → 1.1.0 (e.g., a new province-code added). The fixture is curated against 1.0.0 behaviour. The recall gate runs against 1.1.0 and reports 99.5% — but the 99.5% includes 1.1.0 behaviour on samples that 1.0.0 wouldn't have caught either. The fixture is no longer the calibration target it was created for; recall numbers are unreliable.

Without a version pin, "we shipped a recognizer upgrade and the gate still passed" is indistinguishable from "we shipped a recognizer upgrade and broke the calibration target without anyone noticing."

#### Suggested fix

Add `fixture_manifest.yaml` (per-recognizer version pins + recall floors + precision baselines + runtime budget). Add `test_recognizer_versions_match_manifest` in §5 that calls `TestClient(app).get("/recognizers/version")` and asserts each version matches the manifest. On mismatch, fail with `fixture_version_mismatch` — forcing the operator to either bump the fixture (regen for the new recognizer behaviour) OR revert the recognizer change.

Add §1 clause (#10) documenting the pin requirement.

### ISS-004 — Pre-commit "no real PII" hook claimed in §10 but not specified in §1 or §5

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** §10 row (mentioned), §1 (no clause), §5 (no implementation)
- **status:** resolved

#### Description

The first-pass §10 had a row:

> *"Real PII committed to corpus | Pre-commit hook (regex scan) | Commit rejected | Engineer uses synthetic equivalent."*

But: §1 doesn't mandate the hook, §3 doesn't define the hook contract, §5 doesn't show how it works, and `new_files` doesn't list it. A reader treats "pre-commit hook (regex scan)" as a recommendation rather than a normative requirement, and a code-gen agent has no template.

The hook is load-bearing: real PII in the corpus is a worse incident than a single live leak (a *systematic* leak with a git audit trail). Promising it in §10 without implementing it is a fail-open default — the hook ships only if some future engineer notices the §10 row and reads it as instruction.

#### Suggested fix

1. Add §1 #14 — normative requirement for the hook.
2. Add `.pre-commit-hooks/no-real-pii-in-corpus.sh` and `services/ai-gateway/pii/scripts/check_no_real_pii.py` to `new_files`.
3. Show the script bodies in §5 and §6.
4. Document the "fail-closed in CI, warn-only locally" semantic (devs without ops-credentials get WARN at commit; CI runner has the decrypted pattern table and gates merge).
5. Add §11 implementation note explaining the rotation/encryption of the pattern table.

### ISS-005 — Workflow file not listed in `paths:` → loosen-the-gate PRs bypass the gate

- **severity:** error
- **rule_id:** anti-tamper / self-gate
- **location:** §5 (workflow YAML), §1 (no clause)
- **status:** resolved

#### Description

The compressed first-pass's workflow had `paths: ['services/ai-gateway/pii/**']`. A PR that ONLY edits `.github/workflows/vn-pii-recall.yml` (e.g., changes 0.99 to 0.95, or comments out the assertion) would not trigger the gate — because the workflow file's path doesn't match `pii/**`.

Result: an attacker (or an absent-minded engineer) can loosen the gate in a PR that the gate itself never inspects. This is the classic "you can't relax your own enforcement unilaterally" anti-pattern.

#### Suggested fix

Extend `paths:`:

```yaml
on:
  pull_request:
    paths:
      - 'services/ai-gateway/pii/**'
      - '.github/workflows/vn-pii-recall.yml'
```

Add §1 clause (#8) documenting the self-gate requirement and the rationale.

### ISS-006 — Math error in §11 about "at most 2 missed per recognizer"

- **severity:** warning
- **rule_id:** correctness / arithmetic claim
- **location:** §11 notes
- **status:** resolved

#### Description

The compressed first-pass claimed:

> *"For 99% recall on 200 samples, at most 2 can be missed per recognizer (50 samples × 1% = 0.5, rounded up to 1; some recognizers have more samples but the floor is per-recognizer)."*

This is wrong on two counts. (1) The 200-sample total isn't the divisor — per-recognizer recall divides by the per-type count, not 200. (2) For 20-sample recognizers (VN_NDD, VN_BANK_ACCOUNT), `20 × 0.99 = 19.8`, which rounds-up to 20 — i.e., **zero misses tolerated** at the 99% floor, not "at most 1."

A reader internalises "I can miss 2 per recognizer" and shrugs off a single regression on VN_NDD — but the actual gate fails on the first miss for that recognizer (`19/20 = 0.95 < 0.99`).

#### Suggested fix

Replace the §11 paragraph with the correct math:

> *"Every recognizer must hit 100% on its current sample subset to clear the 99% floor with margin; one miss on a 20-sample recognizer drops it to 95%, which fails. For 50-sample types (CCCD, MST=30, ADDRESS=40, PHONE=40), the math is identical — `n × 0.99` ranges from 19.8 to 49.5, all rounding up. The 99% number is a per-recognizer minimum of 100% in practice for slice 3."*

## §3 — Strengths preserved through expansion

- §3 introduces `fixture_manifest.yaml` as the single source of truth for per-recognizer floors, recognizer version pins, precision baselines, and runtime budget. A calibration change is a single-PR diff to that file; no test-code mutation required for routine tuning.
- §5 separates the recall test (gating) from the precision test (warning-only) by file boundary. Each test file has a single responsibility; failure messages can't be confused between gates and warnings.
- §5's `test_no_real_pii_in_corpus` runs in <1s pre-commit; it's the cheapest possible insurance against a class of incident that has no recovery path other than full rotation of any exposed customer.
- §10 inventory grew from 4 rows to 24 — including the off-by-one path (sample-count drift), the silent-skip path (pytest skip), the self-gate path (workflow file edited), the network-egress path (accidental `requests.get`), and the GitHub LFS truncation path (corpus file < expected size). Each row has an unambiguous detection mechanism.
- §1 #16 declares the trend chart as a downstream consumer (TASK-AI-022) without scope-creeping into the chart impl itself; the artefact's JSON shape is the contract.
- §11 explicitly documents the `regen_fixture.py` anonymisation rule ("preserve format validity, destroy identity linkage") which is the operational invariant for keeping the fixture honest quarter-to-quarter.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the FR itself:

- **ISS-001 RESOLVED**: Fixture path realigned to `services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml` (matching TASK-AI-012 §6); format converted from JSONL to YAML with the `expected_entities` / `expected_count` / `expected_spans` shape; tests updated to use `yaml.safe_load`.

- **ISS-002 RESOLVED**: §5 now has `test_recall_per_recognizer_and_aggregate` that (a) computes per-recognizer recall, (b) computes aggregate, (c) collects failures with full context (entity, recall value, missed sample IDs), and (d) writes the structured JSON artefact even on failure. Pattern reuses TASK-AI-012 §5 post-ISS-004 verbatim.

- **ISS-003 RESOLVED**: §1 #10 added; `fixture_manifest.yaml` introduced in §3 with `recognizer_versions` field; `test_recognizer_versions_match_manifest` in §5 calls `GET /recognizers/version` and asserts the pin. Mismatch fails with `fixture_version_mismatch`.

- **ISS-004 RESOLVED**: §1 #14 added; `.pre-commit-hooks/no-real-pii-in-corpus.sh` + `scripts/check_no_real_pii.py` listed in `new_files` and bodied in §5/§6; `.pre-commit-config.yaml` registration shown in §6; §11 implementation note added documenting the "fail-closed in CI, warn-only locally" pattern and the encrypted-pattern-table rotation.

- **ISS-005 RESOLVED**: §1 #8 added explicitly requiring `.github/workflows/vn-pii-recall.yml` in `paths:`; §5 workflow YAML shows the self-inclusion; §10 has a row for the workflow-edit attack path with the self-gate as detection.

- **ISS-006 RESOLVED**: §11 first paragraph rewritten with the correct math — "every recognizer must hit 100% on its current sample subset to clear the 99% floor with margin; one miss on a 20-sample recognizer drops it to 95%, which fails." Removed the false "at most 2 missed" claim.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-013 audit (final). Status: PASS at 10/10.*
