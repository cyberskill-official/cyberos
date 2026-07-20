---
artefact: repo-context-map@1
task_id: TASK-IMP-099
created: 2026-07-17
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated, undeclared-pin discovery recorded)
---
# Repo context map - TASK-IMP-099

## Baseline patterns the new code must follow
- the rule under edit: "Queue selection (no task id given)" in the Resume semantics block of modules/cuo/chief-technology-officer/workflows/ship-tasks.md (line 311 pre-batch; 312 after TASK-IMP-097's one-line insertion above it) - the sentence's tail ("then `created` ascending, then id ascending. Echo the ...") and the echo-line format are load-bearing and must survive byte-intact
- the authority being aligned to: FM-105 in modules/skill/task-audit/RUBRIC.md:26 (priority required, one of p0/p1/p2/p3, error severity) with the migration note at modules/skill/task-audit/RUBRIC.md:37; contract row modules/skill/contracts/task/CONTRACT.md:51
- rank code already bilingual: modules/cuo/cuo/ship_manifest.py `_PRIORITY_RANK` maps p0..p3 AND legacy MUST/SHOULD/COULD/WONT (comment at lines 22-26: both accepted so mid-migration repos sort deterministically) - out of scope to change, in scope to describe via the parenthetical
- version discipline: workflow_version in ship-tasks.md frontmatter moves on any normative wording change; suite pins are EXACT (`^workflow_version: 2\.6\.3$`) so every bump is a deliberate two-sided edit - the pin discipline is the feature, preserved not loosened
- pin inventory (discovery): the exact pin exists in TWO scenarios of tools/install/tests/test_workflow_helpers.sh - t12 (spec-declared) AND t09 (same string, line 456 pre-change, undeclared in the spec's source_pages). A bump that moves only t12's pin ships a red t09; both move together, disclosed in code-review.md
- suite harness shape: ensure_payload builds one scratch payload into $TMP/payload on first use and is a no-op after (test_workflow_helpers.sh); scenarios are want-gated functions with ok/fail counters and distinct failure messages; header comment block documents every scenario
- payload propagation: tools/install/build.sh copies ship-tasks.md to payload cuo/ and plugin/skills/ship-tasks/cuo/ - editing the source is the only plumbing; t09 checks all three copies, t12/t13 check source + payload cuo/

## Schemas / interfaces in scope
- reworded rule (one physical line): "order by priority: `p0` before `p1` before `p2` before `p3` (legacy MoSCoW values map per FM-105), then `created` ascending, then id ascending. Echo the"
- frontmatter: workflow_version 2.6.3 -> 2.6.4
- t13_queue_rule_p0_p3: for source AND scratch payload cuo/ship-tasks.md - positive grep for the p0-p3 ordering phrase, positive grep for the legacy-mapping parenthetical, negative grep for the MoSCoW rule SHAPE (`(MUST|SHOULD|COULD|WON.?T)[[:space:]]+before[[:space:]]+(MUST|SHOULD|COULD|WON.?T)`, case-insensitive - both sides must be MoSCoW values, so prose like "MUST be passed ... BEFORE this workflow" at skill_chain step 27 and "A batch SHOULD be shipped" in §11a can never trip it, while the parenthetical carries no "X before Y" pairing at all); payload version pinned 2.6.4
- t09/t12 pins: `^workflow_version: 2\.6\.4$` with failure messages updated to say 2.6.4; suite header comments for t09/t12 corrected and a t13 entry added (a comment asserting 2.6.3 above a test asserting 2.6.4 would be exactly the lying-header shape this suite exists to prevent)

## Files outside the immediate domain (modules/cuo/)
1. tools/install/tests/test_workflow_helpers.sh (modified - spec-declared in `modified_files`)

files_outside_immediate_domain: 1 (<= 3 -> no ADR trigger; the file is spec-declared).

## Blast radius
file_count: 2 modified, 0 new (ship-tasks.md 2 lines rewritten: rule + version; test_workflow_helpers.sh +~40/-10: two pins, three comment blocks, t13 + run line) | module_count: 2 (modules/cuo workflow, tools/install tests) | cross_module_edges: t13 -> build.sh scratch payload (read-only); prose -> ship_manifest.py rank order (described, not changed) module_placement_warning: null (spec declares `service: modules/cuo`; every touched file is spec-declared) Behavioral radius: doc-driven agents reading the vendored workflow now rank by the scale the linter enforces (FM-105) instead of the retired one; ship_manifest.py ordering is unchanged (it already ranked both scales identically), so no task-selection behavior changes for tool-driven runs - the prose stops contradicting the machine floor. Consumer repos inherit the reworded rule at 2.6.4 through the payload; any future bump without both pin moves, or pin move without bump, fails the suite loudly.
