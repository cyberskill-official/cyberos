---
id: TASK-IMP-145
title: "TRACE citation truth — SKILL-202 and gates G7/G8"
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-25T10:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-SKILL-202, TASK-IMP-140, TASK-CUO-305]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.5.1"
owner: Stephen Cheng (CTO)
created: 2026-07-25
effort_hours: 3
service: scripts/tests + docs
new_files: []
modified_files:
  - scripts/tests/test_skill_stub_lint.sh
  - docs/tasks/skill/TASK-SKILL-202-skill-quality-floor/spec.md
  - docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/spec.md
  - docs/verification/benchmark-gates.md
  - CHANGELOG.md
source_pages:
  - "docs/tasks/skill/TASK-SKILL-202-skill-quality-floor/spec.md:29 (new_files names tools/install/tests/test_skill_floor.sh) and :113-119 (all seven ACs cite test_skill_floor.sh::tNN) - neither path exists at HEAD"
  - "docs/tasks/skill/TASK-SKILL-202-skill-quality-floor/testing-evidence.md 'Cited suite (TRACE-004) - path residual': the floor landed in scripts/tests/test_skill_stub_lint.sh, 7/7 green; gate-1 accepted the deviation and asked gate-2 to confirm"
  - "docs/verification/benchmark-gates.md G7/G8 Owner + Checked-files rows and the same rows embedded in docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/spec.md (the doc's content contract)"
  - "scripts/tests/test_skill_stub_lint.sh t01-t07 (the as-built floor detector, injection-discipline scan and parity-SCOPE completeness check)"
  - "tools/install/docs-tools/verify-goals.mjs:98 - 'the acceptance cites a test that no longer exists, which IS the finding'"
source_decisions:
  - "2026-07-25 session operator: post-1.2.0 plan Wave 2 residual - align SKILL-202 AC citations to scripts/tests/test_skill_stub_lint.sh (TASK-CUO-305 folded the other four batch/8 friction items and did not carry this one)."
---

# TASK-IMP-145: TRACE citation truth for the skill quality floor

## Summary

`TASK-SKILL-202` is `done` and every one of its seven acceptance criteria cites
`tools/install/tests/test_skill_floor.sh` — a file that was never created. The floor it
describes shipped as `scripts/tests/test_skill_stub_lint.sh` and is green. This task
repoints the citations at the suite that exists, adds the three small checks needed so no
criterion cites nothing, records the one half that stayed deferred, and fixes the same
stale paths where benchmark gates G7 and G8 inherited them.

## Problem

`TASK-SKILL-202`'s own `testing-evidence.md` names the divergence under the heading
"Cited suite (TRACE-004) — path residual": the implementation put the floor in
`scripts/tests/test_skill_stub_lint.sh` (7 tests, green, auto-registered by `run_all.sh`'s
glob), while the spec's `new_files:` and all seven AC lines kept naming
`tools/install/tests/test_skill_floor.sh` plus a `tools/install/check-skill-floor.sh`
checker whose build wiring was the optional F4 half and never landed. Gate-1 accepted the
deviation and asked gate-2 to confirm; gate-2 accepted the task and the citations stayed.

Two other surfaces inherited the same paths: `docs/verification/benchmark-gates.md` (G7
and G8 Owner + Checked-files rows) and the embedded G7/G8 definitions inside
`TASK-IMP-140`'s spec, which is that doc's content contract.

So the corpus carries a shipped task whose acceptance resolves nowhere and two benchmark
gates naming a checker nobody can run. `verify-goals.mjs` refuses exactly this shape for
goal predicates ("the acceptance cites a test that no longer exists, which IS the
finding"); spec-level citations have no such guard, which is why this one survived a
`done` flip.

## Proposed Solution

Repoint each SKILL-202 criterion at the real function that proves it and correct the
`new_files:` list to what shipped. Three clauses have no matching test today — §1.1
(delist + allowlist cleanliness), §1.2 (the `certify-nfrs.md` degradation notice) and
§1.7 (the CHANGELOG record) — so rather than let those ACs cite nothing, add
`t08_delist_and_allowlist_clean`, `t09_workflow_degrades_loud` and
`t10_changelog_records_floor` to the existing suite: three greps over `build.sh`,
`chain-allowlist.txt`, `certify-nfrs.md` and `CHANGELOG.md`, in the suite's existing
counter/exit style. Rewrite SKILL-202 §1.5/§1.6 to describe the as-built mechanism (the
detector is the suite's `floor_check`; the standalone checker and its build wiring stayed
deferred) and record that deviation in `source_decisions`, so the normative text stops
promising a file the repo does not have. Finally, correct the G7/G8 Owner and
Checked-files rows in `benchmark-gates.md` and in TASK-IMP-140's embedded definitions,
leaving severities and tiers untouched so `test_benchmark_gates.sh::t01`'s spec↔doc
cross-check keeps passing.

## Alternatives Considered

- **Create `tools/install/tests/test_skill_floor.sh` as a shim that calls the real
  suite.** Rejected: a second registration for one behaviour is the failure mode
  `run_all.sh`'s glob header argues against, and the shim would make the stale name
  permanent rather than retired.
- **Build the missing `check-skill-floor.sh` + build wiring so the citations become true
  as written.** Rejected for this task: that is real product work (a build-blocking gate),
  it was consciously deferred at ship time, and it does not need to gate the correction of
  a citation. Recording the deferral honestly is the smaller true statement.
- **Leave the spec alone; the residual is documented in the evidence file.** Rejected: the
  documentation is one file deep and the corpus surface (spec ACs, `new_files:`, two
  benchmark-gate rows) still reads as if the checker exists. A note that contradicts the
  artefact it annotates loses to the artefact.
- **Repoint the citations and skip the three new tests.** Rejected: three of the seven ACs
  would then cite nothing at all, which is worse than citing a wrong path — at least a
  wrong path is greppable.

## Success Metrics

- Primary: zero live references to `test_skill_floor.sh` / `check-skill-floor.sh` outside
  the historical `docs/batches/` ship notes, every SKILL-202 AC resolving to a function
  that exists, and `scripts/tests/test_skill_stub_lint.sh` green at 10/10.
- Guardrail: `scripts/tests/test_benchmark_gates.sh` and the full `run_all.sh` stay green;
  no vendored skill, no build behaviour and no gate severity/tier changes.

## Scope

In scope: the three new suite functions, SKILL-202's frontmatter + §1.5/§1.6 + AC block,
the G7/G8 rows in `benchmark-gates.md` and TASK-IMP-140's spec, CHANGELOG.

### Out of scope / Non-Goals

- Building `tools/install/check-skill-floor.sh` or wiring a floor check into `build.sh`
  (the deferred F4 half stays deferred — now recorded, not implied done).
- Re-litigating any SKILL-202 outcome: the delisting, the 20 injection-discipline
  backports and the SCOPE expansion all shipped and are re-verified by the suite this task
  cites.
- The historical `docs/batches/batch-8b-*` notes, which correctly record what was true on
  the day they were written.

## Dependencies

None. `TASK-SKILL-202` and `TASK-IMP-140` are both `done`; this task edits their
citation surfaces only, changing no clause's meaning.

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) auditing the post-1.2.0 plan's Wave 2 residuals
  against live `main`.
- **Scope:** the audit's own figures were treated as claims to verify, not as inputs.
  **re-derived and CONFIRMED:** `scripts/tests/test_skill_stub_lint.sh` passes 7/7 at HEAD,
  and its t01–t07 function names are as `testing-evidence.md` records them.
  **re-derived and CORRECTED:** the plan and the batch-8b notes describe the residual as a
  SKILL-202 spec matter only; a repo-wide grep at HEAD found the same stale paths in
  `docs/verification/benchmark-gates.md` and in TASK-IMP-140's embedded G7/G8 definitions,
  so the scope is four files, not one.
  **measured and ADDED:** three SKILL-202 clauses (§1.1, §1.2, §1.7) have no covering test
  in the as-built suite — measured by mapping each clause to the suite's functions — which
  is why this task adds t08–t10 rather than only repointing citations. No unscoped
  attestation is made about clauses outside that mapping.
- **Human review:** session operator instruction (2026-07-25) to close genuine plan gaps;
  HITL verdicts recorded with evidence at both gates.

## 1. Description

- 1.1 `scripts/tests/test_skill_stub_lint.sh` MUST gain `t08_delist_and_allowlist_clean`:
  `tools/install/build.sh`'s `VENDORED_SKILLS` list contains no `nfr-` entry and
  `tools/install/chain-allowlist.txt` contains no `nfr-` line.
- 1.2 The suite MUST gain `t09_workflow_degrades_loud`: `certify-nfrs.md` (source and
  vendored copies that exist) states at its routing step that the NFR skills are not yet
  shipped.
- 1.3 The suite MUST gain `t10_changelog_records_floor`: `CHANGELOG.md` records the
  delisting naming `TASK-CUO-209`, the backport count, and the parity SCOPE expansion.
- 1.4 Every acceptance criterion in `TASK-SKILL-202`'s spec MUST cite a function that
  exists in `scripts/tests/test_skill_stub_lint.sh`, and its `new_files:` MUST name what
  shipped rather than paths that were never created.
- 1.5 `TASK-SKILL-202` §1.5 and §1.6 MUST describe the as-built mechanism (floor detector
  inside the suite) and MUST record the deferred standalone-checker/build-wiring half as a
  named deviation rather than an unmet MUST.
- 1.6 `docs/verification/benchmark-gates.md` G7 and G8 MUST name
  `scripts/tests/test_skill_stub_lint.sh` in their Owner and Checked-files rows, and
  `TASK-IMP-140`'s embedded G7/G8 definitions MUST match — severities and tiers unchanged.
- 1.7 `CHANGELOG.md` Unreleased MUST record the citation repair.
- 1.8 The suite MUST gain `t11_skill_202_citations_resolve`, which parses every
  `test: <path>::<fn>` citation out of `TASK-SKILL-202`'s acceptance block and fails when
  the path is absent or the function is not defined in it — so this class of rot cannot
  survive another `done` flip silently.

## Acceptance Criteria

- [ ] AC 1 (traces_to: #1.1) — `build.sh`'s vendored list and `chain-allowlist.txt` carry no `nfr-` entry, and the check fails on a fixture that re-adds one - test: `scripts/tests/test_skill_stub_lint.sh::t08_delist_and_allowlist_clean`
- [ ] AC 2 (traces_to: #1.2) — every `certify-nfrs.md` copy states the NFR skills are not yet shipped at its routing step - test: `scripts/tests/test_skill_stub_lint.sh::t09_workflow_degrades_loud`
- [ ] AC 3 (traces_to: #1.3, #1.7) — `CHANGELOG.md` records the delisting naming TASK-CUO-209, the backport count and the SCOPE expansion, plus this task's citation repair - test: `scripts/tests/test_skill_stub_lint.sh::t10_changelog_records_floor`
- [ ] AC 4 (traces_to: #1.4, #1.5, #1.8) — every SKILL-202 acceptance citation resolves to a function defined in the cited file, and the deferred standalone-checker half is named in the spec rather than left as an unmet MUST - test: `scripts/tests/test_skill_stub_lint.sh::t11_skill_202_citations_resolve`
- [ ] AC 5 (traces_to: #1.6) — G7/G8 name the real suite in their Owner and Checked-files rows while the spec↔doc severity/tier cross-check stays green - test: `scripts/tests/test_benchmark_gates.sh::t01_doc_complete_and_consistent`

## Test plan

1. `bash scripts/tests/test_skill_stub_lint.sh`
2. `bash scripts/tests/test_benchmark_gates.sh`
3. `bash scripts/tests/run_all.sh`
4. `bash .cyberos/cuo/gates/run-gates.sh`

## 3. Edge cases

- **`certify-nfrs.md` exists in more than one place (source + built payload).** t09 checks
  every copy it finds and fails naming the copy that lost the notice, rather than passing
  on the first hit.
- **A future release rewrites the CHANGELOG's Unreleased section.** t10 keys on the
  delisting record anywhere in the file, not on the top entry, so a release rollup does
  not turn a true statement into a red test.
- **The NFR skills are genuinely implemented later and re-vendored.** t08 then fails
  loudly, which is correct: re-vendoring is a reviewed change that must revisit SKILL-202's
  delisting record in the same commit.
- **Security-class:** documentation and test-only; the new functions read and pattern-match
  repo files, execute nothing, and add no network or secret surface.
