# TASK-IMP-090 repo context map

## Cone
- `modules/skill/task-author/SKILL.md` (CONTRACT_ECHO manifest_path default, line 184)
- `tools/install/install.sh` (.workflow/.gitignore seed, lines 44-54)
- `tools/install/tests/test_install_hygiene.sh` (t07 scenarios)
- `docs/tasks/.workflow/.gitignore` + the three tracked batch manifests (index operation)
- new: `docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md`

## Patterns the change must follow
- **Seed idempotence**: install.sh is re-run on every re-vendor; every scaffold either creates once or appends once. The existing `wf_ignore` block is the pattern (create-if-absent, else grep -qxF before append).
- **Trailing-newline heal**: an operator-edited .gitignore may lack a final newline; `[ -z "$(tail -c 1 "$f")" ] || printf '\n'` before append is the convention used here so the pattern lands as its own line.
- **_audits is the durable record**: `docs/tasks/_audits/` already holds per-module audit records (28 module dirs). A batch record is a sibling document, not a new mechanism.
- **Session state is untracked**: TASK-CUO-206 established `*.ship.json` in this same seed; manifests are the same class of artefact and were simply missed.

## Blast radius
- Files: 5 modified + 1 new + 3 index removals. Modules: 3 (skill prose, install, docs).
- Cross-module edges: install.sh is shared with TASK-IMP-088 - serialized in one agent per the batch plan (§11a cone rule).
- Consumer impact: fresh installs seed both patterns; existing consumers gain the manifest pattern on their next re-vendor without losing operator lines.

## Module placement
Correct. `improvement` is the cross-cutting hardening module; this is payload + governance hygiene, not a product surface.
