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
time over the payload and written into `manifest.yaml`; three components compare it against an install, and none of them re-hashes that install. But no
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
| `audit-fleet.sh:13, :19, :37` | `_rs()` greps the manifest; compares each fleet repo's token to an expected one |

A grep for any re-hash of the installed tree (`find`/`shasum` over `$CY`) across all of them returns
**empty**. Nothing recomputes. (`check-version-sync.sh` reads `rules_sha` too, at `:56`, but is NOT a
drift comparator and is out of scope: `grep -c '\.cyberos'` on it returns **0** - it compares payload
stamps to the root `VERSION` and has no installed side at all.) `update-check.sh:76` states the
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
`audit-fleet.sh`; a suite. `check-version-sync.sh` is deliberately NOT in scope: it has no installed side (`grep -c '\.cyberos'` = 0).

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

- 1.1 Every component that reports rule drift between an installed machine and a payload (`version.sh`, `lib/update-check.sh`, `audit-fleet.sh`) MUST derive the installed side by RECOMPUTING the fingerprint over the installed tree. No component may derive the installed side by reading a stored token out of the installed manifest.
- 1.2 The cone is a single explicit list, declared ONCE in a shared file that `build.sh` and every comparator read. It is a maintained list, not a derivation: `install.sh` vendors from three separate places (`:185-198` and `:432`), conditionally (`[ -d ]` / `[ -f ]` guards), inside a loop (`:432` iterates `AGENTS.md memory.schema.json memory.invariants.yaml`), and under environment control (`memory/` only when `CYBEROS_NO_MEMORY != 1`). No static read of that file yields the set. (Operator decision, 2026-07-18: a maintained list reconciled by a build check, rather than a bash analyser.)
- 1.3 The cone MUST cover the vendored trees `cuo`, `plugin`, `mcp`, `lib`, `docs-tools` and the vendored files under `memory/` (`:185-198`, `:432`), and MUST exclude four classes: (a) install-generated or operator-owned paths never copied - `memory/store/`, `gates.env`, `config.yaml`, `.update-check-cache`, `AGENT-ENTRY.md`, `gates.env.bak.*`, `.install.lock`; (b) `manifest.yaml`, which is COPIED (`:188`) but MUST be excluded because it CONTAINS `rules_sha` - `build.sh:354` computes the value and `:357` writes the file holding it, so covering it makes the value depend on itself; (c) `VERSION`, which is COPIED (`:189`) but is TASK-IMP-104's axis, not this one; (d) the six vendored root scripts (`:190-195`), which are the machine's own installer surface and change on every payload edit.
- 1.4 A build check MUST reconcile the declared list against `install.sh`: a path `install.sh` vendors that is neither in the cone NOR in §1.3's four exclusion classes MUST fail the build. The exclusions are declared alongside the cone, so `manifest.yaml`, `VERSION` and the root scripts are vendored-and-excluded BY NAME rather than by omission - a vendored path that nobody has classified is the failure this catches. The check reconciles a maintained list against reality; it does not derive the list, and MUST NOT be written so that it compares the list to itself.
- 1.5 A recomputed installed fingerprint differing from the payload's MUST be reported as drift, INCLUDING when both VERSION strings and both stored tokens are equal.
- 1.6 Path-naming is decided per INVOCATION, never per component, because reachability is a property of how a tool is called: `version.sh` run as `.cyberos/version.sh` has `$here == $CY` and NO payload tree, while `bash <payload>/version.sh <repo>` has one; `lib/update-check.sh` sourced from `.cyberos/lib/` has `self_root == $CY` and no tree in its PRIMARY mode; `audit-fleet.sh` resolves a REAL tree at `dist/cyberos/` by default (`:16-18`) and has only a token under `CYBEROS_EXPECT_RULES_SHA`. Where a payload tree IS reachable, the report MUST name every differing path. Where only a token is reachable, the report MUST state that it cannot name paths rather than implying a complete report.
- 1.7 An installed tree byte-identical to the payload ACROSS THE CONE MUST be reported as current. "Byte-identical across the cone" is the whole test: paths outside the cone (`ci/`, `cli/`, `template/`, which ship and are never installed; `memory/store/`, which installs and never ships) MUST NOT affect the verdict.
- 1.8 Recomputation MUST use `build.sh`'s own `_rsha()` (`sha256sum` on Linux, `shasum -a 256` on macOS) and its `LC_ALL=C` sort, so a byte-identical tree yields a byte-identical fingerprint on both platforms.
- 1.9 A comparison that cannot be computed (manifest absent or unreadable on either side, cone unreadable) MUST yield the verdict `unknown`. No component may yield a current verdict on a comparison it could not perform.
- 1.10 Exit contracts: `lib/update-check.sh` remains soft-by-default (`:18`) and MUST NOT change its exit semantics. `audit-fleet.sh` MUST NOT retain its present fail-open behaviour (`:19`, "rule-drift check DISABLED"); with no expected token it MUST yield `unknown` per 1.9 rather than silently passing.
- 1.11 No component's output may describe the fingerprint as detecting tampering, since anything able to rewrite a vendored file can rewrite the manifest beside it.
- 1.12 A comparison MUST NOT modify the installed machine. `lib/update-check.sh` alone MAY continue writing its existing `$cy/.update-check-cache` (`:99`); no other component may write anything.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - for EACH of the three comparators, mutating one byte of a vendored file while leaving BOTH stored manifest tokens untouched is reported as drift; the test MUST exercise all three by name and MUST FAIL if any one still answers from the stored token - test: `tools/install/tests/test_rules_sha_recompute.sh::t01_all_three_recompute_not_recall`
- [ ] AC 2 (traces_to: #1.2) - the cone is read by BOTH `build.sh` and every comparator from ONE declared file; the test MUST FAIL if any of them carries its own copy of the list, and MUST FAIL against today's `build.sh:354`, which inlines `find cuo plugin mcp cli memory` - test: `tools/install/tests/test_rules_sha_recompute.sh::t02_one_shared_cone_definition`
- [ ] AC 3 (traces_to: #1.3) - the cone covers `cuo plugin mcp lib docs-tools` and the vendored `memory/` files (mutating one file in EACH yields drift; the test MUST FAIL on today's cone, which omits `lib` and `docs-tools`); and each of the four exclusion classes is exercised - a generated path (`gates.env`), the CIRCULAR one (mutating `manifest.yaml`'s own `rules_sha:` line), `VERSION`, and a root script - each yields NO drift - test: `tools/install/tests/test_rules_sha_recompute.sh::t03_cone_covers_vendored_and_excludes_four_classes`
- [ ] AC 4 (traces_to: #1.4) - adding a `cp` of a NEW path into `install.sh` that is in neither the cone nor an exclusion class fails the build; and the check is not self-comparing - the test MUST FAIL if the reconciler is rewritten to read its expected set from the same list it validates - test: `tools/install/tests/test_rules_sha_recompute.sh::t04_reconciler_catches_unclassified_vendored_path`
- [ ] AC 5 (traces_to: #1.5) - drift is reported with both VERSION strings AND both stored tokens equal - the exact 2026-07-18 shape; the test MUST FAIL if any component reports current - test: `tools/install/tests/test_rules_sha_recompute.sh::t05_equal_tokens_still_drift`
- [ ] AC 6 (traces_to: #1.6) - per INVOCATION, not per component: `bash <payload>/version.sh <repo>` (tree reachable) names all THREE mutated files and MUST FAIL if only the first is named; `bash .cyberos/version.sh` (no tree) states it cannot name paths; `audit-fleet.sh` on its DEFAULT (tree at `dist/cyberos/`) names them, and under `CYBEROS_EXPECT_RULES_SHA` states it cannot - test: `tools/install/tests/test_rules_sha_recompute.sh::t06_naming_follows_invocation_not_component`
- [ ] AC 7 (traces_to: #1.7) - a freshly installed unmodified machine reports current from every component; the test MUST FAIL on today's cone, which yields `ae756045` over the install vs `66bb0459` over the payload on a byte-identical tree - caused INDEPENDENTLY by `cli/` (coned, never installed) AND by `memory/store/` (install-generated inside a coned dir, 3 payload files vs 8 installed); pruning either alone still mismatches, and the AC MUST exercise both - test: `tools/install/tests/test_rules_sha_recompute.sh::t07_clean_install_is_current`
- [ ] AC 8 (traces_to: #1.8) - the recomputed fingerprint equals `build.sh`'s for the same tree, and a fixture forcing the `shasum`/`sha256sum` branch yields the same digest as the other; the test MUST FAIL if either platform branch diverges - test: `tools/install/tests/test_rules_sha_recompute.sh::t08_cross_platform_digest_stable`
- [ ] AC 9 (traces_to: #1.9) - with the payload manifest removed, separately with it unreadable, separately with the INSTALLED manifest removed, and separately with the cone unreadable, every component emits the verdict `unknown` and none emits a current verdict; the assertion is on the emitted VERDICT (1.9's verb) - test: `tools/install/tests/test_rules_sha_recompute.sh::t09_uncomputable_is_unknown_not_current`
- [ ] AC 10 (traces_to: #1.10) - `update-check.sh` keeps its exit semantics in EVERY mode it binds (`soft` exits 0 on drift, `strict` exits non-zero, `always`, `off`) - a regression guardrail, stated as such; and `audit-fleet.sh` with no expected token emits `unknown` and does NOT pass, which MUST FAIL against today's `:19` "rule-drift check DISABLED" warning-and-continue - test: `tools/install/tests/test_rules_sha_recompute.sh::t10_exit_contracts`
- [ ] AC 11 (traces_to: #1.11) - no component's output matches tamper/integrity/authenticity wording on a drift run, an `unknown` run, OR a current run - test: `tools/install/tests/test_rules_sha_recompute.sh::t11_no_tamper_claims`
- [ ] AC 12 (traces_to: #1.12) - the installed tree is byte-identical before and after a comparison from EACH component; `.update-check-cache` is excluded for `lib/update-check.sh` ONLY, and the test MUST FAIL if `version.sh` or `audit-fleet.sh` writes it - test: `tools/install/tests/test_rules_sha_recompute.sh::t12_check_is_read_only`

## 3. Edge cases

- Payload NEWER than installed with VERSION differing: TASK-IMP-104's ordering guard owns the refusal; this check reports drift and does not duplicate it.
- Operator hand-edited a vendored file: reported as drift and named. The report says what differs, never who differed it (§1.11 covers the wording).
- A vendored file that is legitimately empty: hashed like any other. Empty is a content.
- First install, no installed tree: §1.9's `unknown`, not drift and not current.
- **A vendor step that silently drops a file** (`build.sh:198`'s `[ -f ... ] && cp` - the `workflow-improve.mjs` case): the file is absent from the payload, so absent from the payload fingerprint, so a matching install is NOT drift. **Nothing in this task catches that.** §1.4's cone check is directory-and-path level against `install.sh`'s copy list; a conditional `cp` whose guard is false still LISTS the path, so the cone agrees and the file is simply missing from both sides. Recorded as an uncovered gap, not handed to a clause that cannot honour it. It is the sibling defect and wants its own task.
- `audit-fleet.sh` with a reachable payload rather than a bare token: it MAY name paths per §1.6's first arm; the clause requires the disclaimer only when no tree is reachable.
- macOS vs Linux: §1.8 pins `_rsha()` and `LC_ALL=C`. This is the `stat -c/-f` class of defect TASK-IMP-103 already paid for once.
- Security-class: the fingerprint is a staleness signal, not an integrity guarantee. §1.11 makes that normative and AC 11 tests it across all three verdicts.
