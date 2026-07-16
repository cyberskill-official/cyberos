# TASK-IMP-102 repo context map

## Cone
- `modules/skill/task-audit/SKILL.md` (payload_hash_field, fixity_notes, re_entrancy, new §12)
- `tools/install/docs-tools/task-reconcile.mjs` (R1 preference order)
- `tools/install/tests/test_task_reconcile.sh` (t06)

## Patterns the change must follow
- **The skill states only what holds.** task-audit's own frontmatter claimed idempotence and fixity keyed on `audited_file_sha256` - a key that changes at every phase flip. Fixing the field without fixing those claims would leave the document lying in a quieter way.
- **§ numbering continues the file's own history** (§11 Rework Mode was added 2026-05-20; this lands as §12 with its date) - the skill reads as a changelog of decisions, not a flat spec.
- **Legacy is read, not rewritten**: historical audits describe historical specs; the reader marks them legacy and moves on (the same instinct as accepting both artefact homes).
- **The normalizer's field list is single-sourced** in task-reconcile and mirrored in the contract prose - the § names the list so the next author knows where to extend it.

## Blast radius
- Files: 3 modified. Modules: 2 (skill contract, docs-tools).
- Reach: every future audit gains a verifiable binding; every reconcile run prefers it. No existing audit becomes invalid.

## Module placement
Correct - governance-contract hardening, discovered by the batch's own instrument.
