---
id: TASK-SKILL-202
title: Skill quality floor - NFR stubs, untrusted-content backport, pair parity
template: task@1
type: improvement
module: skill
status: reviewing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-SKILL-118, TASK-CUO-209, TASK-IMP-140]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 16
service: modules/skill
new_files:
  - tools/install/check-skill-floor.sh
  - tools/install/tests/test_skill_floor.sh
modified_files:
  - tools/install/build.sh
  - tools/install/chain-allowlist.txt
  - tools/install/check-pair-parity.sh
  - modules/cuo/chief-technology-officer/workflows/certify-nfrs.md
  - CHANGELOG.md
source_pages:
  - "modules/skill/nfr-certification-author/SKILL.md (22 lines), nfr-evaluator (20), nfr-regression-handler (22), nfr-test-runner (21) - four SKILL.md stubs with no contract sections, no HITL rules, no untrusted-content discipline, vendored into every consumer payload as if real"
  - "tools/install/build.sh:84-146 (VENDORED_SKILLS heredoc lists the vendored set including the four NFR names at lines 91-94; cp -R per name at :146) and tools/install/chain-allowlist.txt (UNPAIRED exemptions for the four, citing TASK-CUO-209)"
  - "measured 2026-07-23 against the built payload (dist/cyberos/cuo/skills, 56 skill dirs): 24 vendored skills lack BOTH `untrusted_inputs` frontmatter and references/UNTRUSTED_CONTENT.md - the four NFR stubs plus architectural-spike-audit/author, backlog-state-update-audit/author, coverage-gate-audit/author, debugging-cycle-audit/author, edge-case-matrix-audit/author, mock-contract-test-audit/author, observability-injection-audit/author, plan-audit/author, repo-context-map-audit/author, task-reconcile, workflow-improver. The audit report said 21; the measured count at authoring time is 24 (20 excluding the NFR stubs this task dispositions separately)"
  - "tools/install/check-pair-parity.sh:13-14 (SCOPE holds 11 pair names; 'grows as pairs are deepened' per TASK-CUO-209 AC 5); measured 2026-07-23: the vendored payload carries 25 author/audit pairs (plan said 24) - the 14 outside SCOPE are architecture-decision-record*, code-review, decommissioning, deployment-checklist, implementation-plan*, postmortem, product-requirements-document, release-notes, retrospective, runbook, software-design-document, software-requirements-specification, statement-of-work, test-strategy, threat-model (*already in SCOPE; net new = 14)"
  - "modules/cuo/chief-technology-officer/workflows/certify-nfrs.md (the workflow that routes to the four NFR skills - the consumer-visible surface that would improvise if the stubs stay)"
  - "modules/skill/repo-context-map-author + edge-case-matrix-author + backlog-state-update-author: the highest-exposure repo-readers in the missing set (they read arbitrary consumer-repo files as input); the well-formed pattern to copy exists in e.g. modules/skill/task-author/references/UNTRUSTED_CONTENT.md + its untrusted_inputs frontmatter block"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T5 'Skill quality' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit findings H7 + H8)."
  - "2026-07-23 authoring: NFR stub default disposition is DELIST from the vendored payload (drop the four names from build.sh's VENDORED_SKILLS + their chain-allowlist entries; source dirs stay in modules/skill as unvendored scaffolds) rather than implement - authoring four real skill contracts is its own project and must not gate this floor task. certify-nfrs.md gains an explicit not-yet-shipped notice instead of silently losing its skills. Recorded for the review gate."
  - "2026-07-23 authoring: plan said 21 skills missing injection discipline and 24 total pairs; measured 24 missing and 25 pairs. Specs are written to the measured numbers with the checker keyed to properties, not counts, so drift in either direction fails honestly."
---

# TASK-SKILL-202: Skill quality floor - NFR stub disposition, untrusted-content backport, full pair parity

## Summary

Three quality gaps ship in every consumer payload: four ~20-line NFR skill stubs are vendored as if they were real skills (no contract, no HITL rules - an agent routed to them will improvise "NFR certification" with false confidence); 24 of the 56 vendored skills - including the highest-exposure repo-readers - carry neither the `untrusted_inputs` frontmatter contract nor a `references/UNTRUSTED_CONTENT.md`, so prompt-injection discipline is absent exactly where arbitrary consumer-repo text is read; and the pair-parity checker holds only 11 of the 25 vendored author/audit pairs to its file-class floor. This task delists the stubs, backports the injection discipline from the exemplary skills to the 20 non-stub gaps, expands parity SCOPE to all pairs, and adds a skill-floor lint so undersized skills can never be silently vendored again.

## Problem

Audit findings H7 + H8, verified first-hand 2026-07-23:

1. **Stub skills vendored as real (H7).** `build.sh:91-94` vendors `nfr-certification-author`, `nfr-evaluator`, `nfr-test-runner`, `nfr-regression-handler` - each a 20-22 line SKILL.md with a description and one paragraph, no envelopes, no invariants, no acceptance material. `certify-nfrs.md` routes to them. The chain-coverage checker exempts them as "unpaired by design", which answers the pairing question while leaving the deeper one - they are not *skills*, they are name reservations shipped as product.
2. **Injection discipline missing on 24/56 (H8).** The repo's own best skills (task-author, task-audit) carry a two-part discipline: `untrusted_inputs` frontmatter (wrap-marker, injection scan, surface-to-human) and a `references/UNTRUSTED_CONTENT.md` procedure. 24 vendored skills carry neither - including `repo-context-map-author` and `edge-case-matrix-author`, which exist specifically to read arbitrary consumer-repo files, the canonical injection vector.
3. **Parity floor covers 11 of 25 pairs.** `check-pair-parity.sh` SCOPE (`:13-14`) names 11 pairs; the payload carries 25. The 14 outside SCOPE can ship without envelopes, rubrics, or failure-mode references and nothing fails.

## Proposed Solution

Delist the four NFR stubs from `build.sh`'s `VENDORED_SKILLS` and remove their `chain-allowlist.txt` exemptions (the allowlist's own rot-warning rule then keeps the file honest); the source dirs remain in `modules/skill/` as unvendored scaffolds. Give `certify-nfrs.md` an explicit "NFR skills not yet shipped - this workflow requires their full implementation" notice at its routing step so the workflow degrades loudly, not improvisationally. Backport the injection discipline to the 20 remaining gap skills by instantiating the task-author pattern: an `untrusted_inputs` frontmatter block (wrap_in_marker `untrusted_content`, injection_scan required, on_marker_hit surface_to_human) plus a `references/UNTRUSTED_CONTENT.md` adapted to each skill's actual input surface (repo files for the repo-readers; artefact bodies for the audits). Expand `check-pair-parity.sh` SCOPE to all 25 vendored pairs, authoring the missing file classes for the 14 newly-scoped pairs (envelopes, PIPELINE/INVARIANTS or RUBRIC/AUDIT_LOOP/REPORT_FORMAT, FAILURE_MODES, acceptance/README) at parity with the existing deepened pairs. Add `tools/install/check-skill-floor.sh` - a vendoring-time lint asserting every vendored SKILL.md meets a minimum floor (>= 60 body lines below frontmatter AND required sections present: a contract/usage heading, an operating-principles or MUST/MUST-NOT block; threshold chosen to be far below any real skill, far above any stub) - wired into `build.sh` so an undersized skill fails the build, with a test suite covering positive/negative paths.

## Alternatives Considered

- **Implement the four NFR skills instead of delisting.** Rejected for this task: each needs a real contract (envelopes, rubric or pipeline, HITL points, acceptance tests) - four skill-authoring projects. Shipping the floor must not wait on them; the delisting leaves named scaffolds and a loud workflow notice, and re-vendoring is one build.sh line when they are real.
- **Delete the NFR source dirs entirely.** Rejected: the name reservations and descriptions are genuine design intent (TASK-CUO-209 vendored them deliberately); deletion loses that record. Delisting removes the false product claim while keeping the intent discoverable.
- **One shared UNTRUSTED_CONTENT.md included by reference.** Rejected: the discipline's value is per-skill specificity (WHAT input is untrusted and WHERE the wrap happens differs between a repo-reader and an artefact-audit); a generic include invites cargo-cult compliance. The frontmatter block is uniform; the reference doc is per-skill.
- **Grow SCOPE incrementally as pairs deepen (status quo).** Rejected: that is the current design and it produced 14 unheld pairs in production; "grows as pairs are deepened" has no forcing function. Inverting it (all pairs in SCOPE; exemptions must be argued) makes shallowness the visible state.
- **Line-count-only stub lint.** Rejected: trivially gamed by padding and misses the real question (does a contract exist); the floor pairs a line minimum with required-section presence, both mechanical.

## Success Metrics

- Primary: by the next CyberOS release, a fresh payload build vendors zero SKILL.md files below the floor (checker green in build), all 25 pairs pass `check-pair-parity.sh` with SCOPE = the full pair set, and every vendored skill whose inputs include repo or artefact text carries both halves of the injection discipline. Baselines today: 4 stubs vendored, 11/25 pairs scoped, 24 skills missing both halves.
- Guardrail: no vendored skill's existing behavior contract changes (the backport adds frontmatter + a reference doc; it rewrites no skill body logic), and payload build time stays within its current budget (+ the checker's ~1s).

## Scope

In scope: build.sh delisting + floor-checker wiring, chain-allowlist cleanup, certify-nfrs.md notice, 20 injection-discipline backports, 14 pair deepenings + SCOPE expansion, the new checker + test suite, CHANGELOG.

### Out of scope / Non-Goals

- Implementing the four NFR skills (future work; unblocked and unowned by this task).
- Rewriting any skill's body prose beyond adding the discipline block/reference and the parity file classes.
- The G7/G8 benchmark-gate CI meta-definitions - TASK-IMP-140 adopts this task's checkers as its G7/G8 mechanisms; soft forward reference, no cycle.
- Skill *quality* judgment beyond the mechanical floor (SKB-family rubric work stays with the skill-bundle tasks TASK-SKILL-111..115).

## Dependencies

None blocking. TASK-SKILL-118 (done) established the pair file-class policy and the parity checker this task expands; TASK-CUO-209 (done) vendored the NFR stubs this task delists - both are context, neither needs reopening; their decisions are superseded knowingly and the delisting names TASK-CUO-209 in its CHANGELOG line. TASK-IMP-140's G7/G8 gates run this task's checkers in CI - forward reference only.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** the stub line counts, the 24-of-56 measurement (both halves grepped per skill dir against the built payload), the 25-pair census, and the SCOPE list were all measured first-hand at HEAD; the plan's 21 and 24 figures are corrected to the measured 24 and 25 with the discrepancy recorded in `source_decisions`.
- **Human review:** the hardening plan was operator-approved 2026-07-23; the delist-not-implement default is a recorded decision for the review acceptance gate.

## 1. Description (normative)

- 1.1 `build.sh` MUST NOT vendor `nfr-certification-author`, `nfr-evaluator`, `nfr-test-runner`, or `nfr-regression-handler` (removed from `VENDORED_SKILLS`), and `chain-allowlist.txt` MUST drop their four UNPAIRED exemption lines in the same change (an allowlist entry whose name no payload dir matches is rot by the file's own rule).
- 1.2 `certify-nfrs.md` MUST state at its skill-routing step that the four NFR skills are not yet shipped and the workflow requires their full implementation before it can run - a loud degradation replacing today's silent improvisation surface.
- 1.3 Every remaining vendored skill that reads repo files or artefact bodies as input - the 20 measured gap skills, enumerated in source_pages - MUST carry BOTH an `untrusted_inputs` frontmatter block (wrap_in_marker `untrusted_content`, injection_scan required, on_marker_hit surface_to_human, matching the task-author pattern) AND a `references/UNTRUSTED_CONTENT.md` adapted to that skill's specific input surface.
- 1.4 `check-pair-parity.sh` SCOPE MUST enumerate every author/audit pair present in the vendored payload (25 at authoring time), and the 14 newly-scoped pairs MUST carry the AUTHOR_CLASSES / AUDIT_CLASSES file sets so the expanded check passes - authored at parity with the existing deepened pairs, not as empty placeholder files.
- 1.5 A new checker `tools/install/check-skill-floor.sh <skills-dir>` MUST fail (distinct non-zero exit) when any vendored SKILL.md has fewer than 60 non-frontmatter lines OR lacks the required floor sections, and `build.sh` MUST run it against the assembled payload so an undersized skill fails the build. Placeholder-syntax detection stays with the existing sweep tooling; this floor is size + structure.
- 1.6 The new suite `tools/install/tests/test_skill_floor.sh` MUST cover: floor-checker pass on the real payload, fail on a synthetic stub fixture, fail on a missing-section fixture, delisting (payload contains no nfr-* skill dir), allowlist cleanliness, and parity-SCOPE completeness (SCOPE equals the payload's measured pair set - a pair vendored but unscoped fails).
- 1.7 `CHANGELOG.md` MUST record the delisting (naming TASK-CUO-209 as the superseded vendoring decision), the injection-discipline backport count, and the parity expansion.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - a scratch payload build contains no `cuo/skills/nfr-*` directory and `chain-allowlist.txt` contains no `nfr-` line; the chain-coverage checker stays green (no rot warnings) - test: `tools/install/tests/test_skill_floor.sh::t01_nfr_delisted_clean`
- [ ] AC 2 (traces_to: #1.2) - `certify-nfrs.md` contains the not-yet-shipped notice at its routing step - test: `tools/install/tests/test_skill_floor.sh::t02_workflow_degrades_loud`
- [ ] AC 3 (traces_to: #1.3) - for each of the 20 enumerated gap skills: SKILL.md frontmatter parses and contains the `untrusted_inputs` block with the three required keys, and `references/UNTRUSTED_CONTENT.md` exists non-empty and names that skill's own input surface (not a byte-copy of another skill's file) - test: `tools/install/tests/test_skill_floor.sh::t03_injection_discipline_backported`
- [ ] AC 4 (traces_to: #1.4) - `check-pair-parity.sh` against the built payload exits 0 with SCOPE size equal to the payload's pair count, and deleting one required class file from a newly-scoped pair in a scratch copy makes it exit 10 naming the file - test: `tools/install/tests/test_skill_floor.sh::t04_full_scope_parity`
- [ ] AC 5 (traces_to: #1.5) - `check-skill-floor.sh` exits 0 on the real payload, non-zero on a fixture stub (< 60 lines) and on a fixture missing the required sections, and a build with an injected stub fixture fails - test: `tools/install/tests/test_skill_floor.sh::t05_floor_checker_blocks_build`
- [ ] AC 6 (traces_to: #1.6) - the suite runs green under `bash scripts/tests/run_all.sh` discovery (glob `tools/install/tests/test_*.sh`) - test: `tools/install/tests/test_skill_floor.sh::t06_registered_by_glob`
- [ ] AC 7 (traces_to: #1.7) - CHANGELOG's top entry names the delisting with TASK-CUO-209, a backport count of 20, and the SCOPE expansion - test: `tools/install/tests/test_skill_floor.sh::t07_changelog_records_floor`

## 3. Edge cases

- **A consumer repo already installed with the NFR stubs:** uninstall/reinstall replaces `.cyberos/cuo/skills/` wholesale, so the stubs disappear on next update; an install that never updates keeps them - acceptable, since the loud certify-nfrs notice ships in the same payload that removes the skills.
- **The 60-line floor vs legitimately small future skills:** a real single-purpose skill below 60 lines would fail the build - by design, loudly, at vendoring time, where the author can either meet the floor or argue an explicit exemption in the checker (the exemption list starts empty; adding to it is a reviewed change).
- **Byte-copied UNTRUSTED_CONTENT.md across skills:** AC 3 requires each reference doc to name its own skill's input surface; the test compares docs pairwise for full-content identity to catch copy-paste compliance (identical general sections are fine; full-file identity is not).
- **SCOPE drift after this task:** a future pair vendored without SCOPE membership fails t04's completeness half (SCOPE must equal the measured pair set), inverting the old grow-when-remembered design.
- **`task-reconcile` and `workflow-improver` (repo-readers, not pairs):** both are in the 20-skill backport set via 1.3 (which keys on input surface, not pairing); neither is touched by SCOPE (parity is a pair property).
- **Security-class:** this task ADDS trust-boundary discipline and removes improvisation surface; the checkers execute no vendored content (they read and pattern-match only). The backported frontmatter changes no tool grants for any skill.
