---
id: TASK-IMP-122
title: rules_sha must be recomputed, not recalled
template: task@1
type: improvement
module: improvement
status: draft
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-18T04:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-074, TASK-IMP-104, TASK-IMP-082]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-18
memory_chain_hash: null
effort_hours: 6
service: tools/install
new_files:
  - tools/install/tests/test_rules_sha_recompute.sh
modified_files:
  - tools/install/build.sh
  - tools/install/version.sh
  - tools/install/lib/update-check.sh
  - tools/install/check-version-sync.sh
  - tools/install/audit-fleet.sh
source_pages:
  - "docs/tasks/_audits/2026-07-18-phase-corpus-measurement.md - method note: why this spec's counts are measured, not recalled"
  - "tools/install/build.sh:354 - rules_sha computed at BUILD over $out, cone = `find cuo plugin mcp cli memory`"
  - "tools/install/version.sh:61-65 - _rs() greps ^rules_sha: from a manifest; :89-93 verdict=rules_drift"
  - "tools/install/lib/update-check.sh:8-11 _cyberos_rules_sha() greps the manifest; :76-96 RULE DRIFT branch; :18 mode soft by default; :99 writes $cy/.update-check-cache on every run"
  - "tools/install/check-version-sync.sh:56 and tools/install/audit-fleet.sh:13,19,37 - two more stored-token readers"
  - "measured 2026-07-18: grep for any re-hash of the installed tree (find/shasum over $CY) in version.sh + update-check.sh returns EMPTY - no component recomputes"
  - "measured 2026-07-18: docs-tools/ and lib/ EXIST in dist/cyberos/ and are ABSENT from the :354 cone"
source_decisions:
  - "2026-07-18 Stephen: PLAN gate - author as its own task, distinct from TASK-IMP-104 (ordering)."
  - "2026-07-18 audit FAIL 4/10 -> rewrite. The first draft claimed 'nothing compares content', which is false: TASK-IMP-074 ships rules_sha. REPAIR it; do not add a second comparator (TASK-IMP-104 §1.2)."
---

# TASK-IMP-122: rules_sha must be recomputed, not recalled

## Summary

CyberOS already ships a rule-content fingerprint. `rules_sha` (TASK-IMP-074) is computed at build
time over the payload and written into `manifest.yaml`; four components compare it. But no
component ever re-hashes the installed tree - each greps the stored token out of
`.cyberos/manifest.yaml` and compares it to the token in the payload's manifest. Two tokens from
one build always agree, whatever happens to the files afterward. So the check answers "did these
manifests come from the same build?" and is read as "do these bytes still match?".

## Problem

Measured on this repo, 2026-07-18.

**Defect 1 - the fingerprint is recalled, not recomputed.** `build.sh:354` computes `rules_sha`
over the assembled output. `install.sh` vendors the manifest containing it. Then:

| component | what it does |
|---|---|
| `version.sh:61-65, :89-93` | `_rs()` greps `^rules_sha:` from each manifest; `verdict="rules_drift"` on mismatch |
| `lib/update-check.sh:8-11, :76-96` | `_cyberos_rules_sha()` greps the manifest; emits `RULE DRIFT ... (same version, different rules)` |
| `check-version-sync.sh:56` | greps `^rules_sha:` from the payload manifest |
| `audit-fleet.sh:13, :19, :37` | `_rs()` greps the manifest; compares each fleet repo's token to an expected one |

A grep for any re-hash of the installed tree (`find`/`shasum` over `$CY`) in `version.sh` or
`lib/update-check.sh` returns **empty**. Nothing recomputes. `update-check.sh:76` states the
correct thesis - *"VERSION is a promise; rules_sha is the evidence"* - and then greps a token that
is a promise for exactly the same reason VERSION is: it was written once, at build, by the thing
being checked.

The consequence is not theoretical. On 2026-07-18 this repo's own `.cyberos` differed from `dist/`
in six vendored artefacts while `VERSION` read `1.0.0` and `rules_sha` matched on both sides. The
installed `batch-select.mjs` was the pre-PR#53 build carrying the undeclared-cone bug, and returned
a different batch than source. Every check reported current.

**Defect 2 - the cone excludes shipped trees.** `build.sh:354` hashes
`find cuo plugin mcp cli memory`. Measured: `docs-tools/` and `lib/` are present in
`dist/cyberos/` and absent from that list. Four of the six artefacts that actually drifted
(`batch-select.mjs`, `render-status-hub.mjs`, `verify-goals.mjs`, `workflow-improve.mjs`) live in
`docs-tools/`. So even a recomputing check would miss them until the cone is widened. The two
defects are independent and both must close, or the fix reports current on the evidence that
motivated it.

**Why REPAIR and not a second mechanism.** TASK-IMP-104 §1.2 forbids install carrying a second
implementation, and 104 explicitly rejected content hashing for ITS question: *"Compare a manifest
hash rather than a version. Rejected: overkill for a monotonic version line, and it answers
'different' rather than 'older', which is not the question."* 104 guards ORDER and is correct as
shipped; "different" is precisely this task's question. `rules_sha` is the right mechanism, in the
right place, doing the wrong thing. This task repairs it.

## Proposed Solution

Recompute the fingerprint over the installed tree at comparison time, using the same function and
the same cone as the build, and compare that against the payload's manifest token. Widen the cone
to every directory the payload ships. Report the differing paths, not just a verdict. Keep each
component's existing exit contract.

## Alternatives Considered

- Add a per-file manifest beside `rules_sha`. Rejected: a second comparator for the same question,
  which TASK-IMP-104 §1.2 forbids by name, and the first draft of this spec proposed it only
  because it had not found `rules_sha`.
- Bump VERSION on every payload change. Rejected: conflates release identity with build identity,
  forces a bump for a comment fix, and still cannot see a vendor step that silently drops a file.
- Compare mtimes. Rejected: not content; survives no copy or checkout faithfully; would have
  reported the 2026-07-18 drift as fine.
- Re-vendor on every `.cyberos` use. Rejected: install is not free, and a guard that silently
  repairs drift teaches nobody that the channel leaked.
- Widen the cone only, leaving the token stored. Rejected: it fixes which files are covered and
  leaves the check comparing two copies of one build-time answer. Defect 1 survives untouched.

## Success Metrics

- Primary: a vendored file mutated by one byte after install, with both VERSIONs and both stored
  tokens equal, is reported as drift and named. Baseline: reported `up_to_date` - reproduced
  synthetically 2026-07-18; the original six-artefact instance was repaired by `e2504cf3` and is
  no longer live evidence.
- Guardrail: a freshly installed, unmodified machine reports current - no false drift, or the
  check gets ignored.
- Guardrail: every directory present in the payload is inside the cone, enforced at build.

## Scope

In scope: the cone at `build.sh:354`; recomputation in `version.sh`, `lib/update-check.sh`,
`check-version-sync.sh`, `audit-fleet.sh`; a suite.

### Out of scope / Non-Goals

- TASK-IMP-104's ordering guard. Correct as shipped; untouched.
- Install-generated and operator-owned paths (`gates.env`, `config.yaml`, `.update-check-cache`,
  `memory/store/`). Normatively excluded by §1.3, not merely noted.
- Auto-repairing drift. Reporting is the deliverable; `install.sh` already re-vendors.
- Tamper-evidence. §1.8 makes the prohibition normative rather than a hope in prose.

## Dependencies

None. TASK-IMP-104's guard runs before the lock and is untouched.

## AI Authorship Disclosure

- **Tools used:** Claude (Opus 4.8) running the CyberOS task-author skill inside Cowork.
- **Scope:** rewritten 2026-07-18 after an independent audit failed the first draft at 4/10 for
  asserting "nothing compares content". Every claim here was re-measured against source that day;
  the four consumers and the cone exclusions are grep output, not recall.
- **Human review:** scope approved at the 2026-07-18 PLAN gate; both HITL gates are recorded
  human verdicts.

## 1. Description (normative)

- 1.1 Every component that reports rule drift (`version.sh`, `lib/update-check.sh`,
  `check-version-sync.sh`, `audit-fleet.sh`) MUST derive the installed side by RECOMPUTING the
  fingerprint over the installed tree. No component may derive the installed side by reading a
  stored token out of the installed manifest.
- 1.2 Recomputation MUST use the same function and the same cone as `build.sh`, so that a
  byte-identical tree yields a byte-identical fingerprint across macOS and Linux.
- 1.3 The cone MUST cover every directory the payload ships, and MUST exclude install-generated
  and operator-owned paths (`gates.env`, `config.yaml`, `.update-check-cache`, `memory/store/`).
  A payload directory outside the cone MUST fail the build.
- 1.4 A recomputed installed fingerprint differing from the payload's MUST be reported as drift,
  INCLUDING when both VERSION strings and both stored tokens are equal.
- 1.5 A drift report MUST name every differing path, not only the first.
- 1.6 A byte-identical installed tree MUST be reported as current.
- 1.7 A comparison that cannot be computed (manifest absent or unreadable on either side, cone
  unreadable) MUST yield the verdict `unknown`. No component may yield a current verdict on a
  comparison it could not perform. Exit contracts are unchanged: `lib/update-check.sh` remains
  soft-by-default (`:18`) and MUST NOT change its exit semantics; `audit-fleet.sh` MUST NOT keep
  its present fail-open behaviour of disabling the check with a warning (`:19`).
- 1.8 No component's output may describe the fingerprint as detecting tampering, since anything
  able to rewrite a vendored file can rewrite the manifest beside it.
- 1.9 A comparison MUST NOT modify the installed machine, except that `lib/update-check.sh` MAY
  continue writing its existing `$cy/.update-check-cache` (`:99`).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - for EACH of the four components, mutating one byte of a vendored file while leaving BOTH stored manifest tokens untouched is reported as drift; the test MUST exercise all four by name and MUST FAIL if any one still answers from the stored token - test: `tools/install/tests/test_rules_sha_recompute.sh::t01_all_four_recompute_not_recall`
- [ ] AC 2 (traces_to: #1.2) - a byte-identical tree yields the same fingerprint as `build.sh` produced for it, asserted by comparing the recomputed value to the payload manifest's value on an unmodified install - test: `tools/install/tests/test_rules_sha_recompute.sh::t02_recompute_matches_build`
- [ ] AC 3 (traces_to: #1.3) - a directory present in `dist/cyberos/` and absent from the cone fails the build; and mutating a file under `docs-tools/` (outside today's cone, and where 4 of the 6 measured drifts lived) is reported as drift - test: `tools/install/tests/test_rules_sha_recompute.sh::t03_cone_covers_every_shipped_dir`
- [ ] AC 4 (traces_to: #1.3) - install-generated paths are excluded: writing `gates.env`, `config.yaml`, `.update-check-cache` and a `memory/store/` entry after install does NOT produce drift; the test MUST FAIL if a fresh install reports drift against itself - test: `tools/install/tests/test_rules_sha_recompute.sh::t04_generated_paths_excluded`
- [ ] AC 5 (traces_to: #1.4) - drift is reported with both VERSION strings AND both stored tokens equal - the exact 2026-07-18 shape; the test MUST FAIL if any component reports current - test: `tools/install/tests/test_rules_sha_recompute.sh::t05_equal_tokens_still_drift`
- [ ] AC 6 (traces_to: #1.5) - with THREE files mutated, the report names all three; the test MUST FAIL if only the first is named - test: `tools/install/tests/test_rules_sha_recompute.sh::t06_names_every_differing_path`
- [ ] AC 7 (traces_to: #1.6) - a freshly installed unmodified machine reports current from every component - test: `tools/install/tests/test_rules_sha_recompute.sh::t07_clean_install_is_current`
- [ ] AC 8 (traces_to: #1.7) - with the payload manifest removed, and separately with it made unreadable, every component yields `unknown` and none yields a current verdict; the assertion is on the emitted VERDICT (1.7's verb), and separately that `update-check.sh` still exits 0 under its soft default and `audit-fleet.sh` no longer disables the check - test: `tools/install/tests/test_rules_sha_recompute.sh::t08_uncomputable_is_unknown_not_current`
- [ ] AC 9 (traces_to: #1.8) - no component's output matches tamper/integrity/authenticity wording on a drift run - test: `tools/install/tests/test_rules_sha_recompute.sh::t09_no_tamper_claims`
- [ ] AC 10 (traces_to: #1.9) - the installed tree is byte-identical before and after a comparison from each component, excluding only `.update-check-cache`; the test MUST FAIL if any other path changes - test: `tools/install/tests/test_rules_sha_recompute.sh::t10_check_is_read_only`

## 3. Edge cases

- Payload NEWER than installed with VERSION differing: TASK-IMP-104's ordering guard owns the
  refusal; this check reports drift and does not duplicate it.
- Operator hand-edited a vendored file: reported as drift and named. The report says what differs,
  never who differed it (§1.8 covers the wording).
- A vendored file that is legitimately empty: hashed like any other. Empty is a content.
- First install, no installed tree to compare: §1.7's `unknown`, not drift and not current.
- A vendor step that silently drops a file (`build.sh:198`'s `[ -f ... ] && cp`): the file is
  absent from the payload, so absent from the payload fingerprint, so a matching install is NOT
  drift. This check CANNOT catch a vendor-step omission - the omission is upstream of the
  fingerprint - and §1.3's build-time cone check is what catches it instead. Recorded because the
  first draft claimed the manifest would catch it, and it cannot.
- macOS vs Linux: `_rsha()` already selects `sha256sum` or `shasum -a 256`; recomputation MUST use
  that same function, or every cross-platform install reports drift against itself. This is the
  `stat -c/-f` class of defect that TASK-IMP-103 already paid for once.
- Security-class: the fingerprint is a staleness signal, not an integrity guarantee. §1.8 makes
  that normative and AC 9 tests it, rather than leaving it as prose in a Non-Goal.
