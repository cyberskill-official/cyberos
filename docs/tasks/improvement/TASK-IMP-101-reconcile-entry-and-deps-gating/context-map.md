# TASK-IMP-101 repo context map

## Cone
- `modules/cuo/chief-technology-officer/workflows/ship-tasks.md` (frontmatter version, outputs list, skill_chain step 0, two new §§)
- `tools/install/tests/test_workflow_helpers.sh` (t12 + t09_doctrine_wiring version pins, new t14)

## Patterns the change must follow
- **Version discipline**: a normative change bumps `workflow_version`, and the exact pins in t09/t12 move with it - the pins are the feature (they force every edit to be deliberate and disclosed).
- **Chain entries are contracts**: naming `skill: task-reconcile` in skill_chain obliges the payload to carry that skill in both trees (`check-chain-coverage.sh`) - TASK-IMP-100 satisfies it; the two tasks are ordered for exactly this reason.
- **Doctrine states rules, not procedures**: the §§ name the trigger, the fork, and the rule; the runbook detail lives in the skill (TASK-IMP-097's precedent - link, do not duplicate).
- **Both artefact homes** (task folder and `docs/tasks/.workflow/<task-ID>/`) - the corpus convention the deps gate must honor or it false-blocks history.

## Blast radius
- Files: 2 modified. Modules: 2 (cuo workflow, install tests).
- Behavioral reach: every future ship-tasks run gains a conditional entry phase and a start-time dependency check. Existing in-flight runs are unaffected (valid manifests defer to resume semantics).

## Module placement
Correct - the workflow's own doctrine.
