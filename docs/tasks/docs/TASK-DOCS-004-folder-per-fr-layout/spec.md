---
id: TASK-DOCS-004
title: "Folder-per-task layout - <module>/<STEM>/{spec.md, audit.md, assets/} + loud regen + the 42 yaml-invalid tasks repaired"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: docs
priority: p0
status: done
verify: T
phase: Wave D - visual deliverables
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-TPL-001, TASK-DOCS-005, TASK-SKILL-120, TASK-DOCS-003]
depends_on: []
blocks: [TASK-DOCS-005, TASK-SKILL-120]
source_pages:
  - scripts/migrate_improvement_to_task.py
  - tools/docs-site/render-roadmap.mjs
  - scripts/check_doc_anchors.sh
source_decisions:
  - "2026-07-12 operator decision: each task gets its own folder with assets/ (images, videos) so specs can be visual-rich."
  - "2026-07-12 field finding (TASK-DOCS-003 ship): regen_backlog's read_fm silently skips 42 yaml-invalid tasks - BACKLOG says 444 where frontmatter says 486. The skip becomes loud and the files get repaired here."
language: python + bash + markdown
service: docs/tasks/
new_files:
  - scripts/migrate_task_layout.py
  - scripts/tests/test_task_layout.sh
modified_files:
  - scripts/migrate_improvement_to_task.py
  - tools/docs-site/render-roadmap.mjs
  - scripts/check_doc_anchors.sh
  - tools/cyberos-init/plugin/commands/create-tasks.md
---

# TASK-DOCS-004: Folder-per-task layout

## §1 - Description

Every task becomes a folder that can carry its own media, and the corpus becomes fully machine-readable while we are touching every file anyway.

Normative clauses:

1. Layout MUST become `docs/tasks/<module>/<STEM>/spec.md` + `<STEM>/audit.md` (when an audit exists) where STEM is the current file stem (e.g. `TASK-AUTH-102-totp-webauthn-mfa`). An `assets/` subfolder is created ON DEMAND (first asset), not pre-created empty. `.workflow/` and `_audits/` trees are untouched.
2. A one-shot script `scripts/migrate_task_layout.py` MUST perform the move for every task + sibling audit via `git mv` semantics (history-preserving), idempotent (re-run = no-op), and MUST print a per-module summary. Legacy flat files MUST NOT remain.
3. Every tool that globs the old layout MUST be updated in the same change: `migrate_improvement_to_task.py` (regen + status flips), `render-roadmap.mjs`, `check_doc_anchors.sh` scan roots, the plugin command doc's path examples, and any test fixture that builds task trees. Discovery grammar becomes `<module>/*/spec.md` with `id` from frontmatter (stem from the folder name).
4. `read_fm` MUST become loud: files whose frontmatter fails strict YAML are listed on stderr with the parse error (one line each) and counted in the regen summary; the silent `continue` is removed. The regen totals line MUST equal the roadmap's frontmatter-derived totals over the same corpus.
5. The 42 currently yaml-invalid task files MUST be repaired (minimal edits: quoting/structure only, no semantic changes) so the whole corpus parses strict-YAML; the repair list ships in the migration commit message or an appendix file.
6. Relative references INSIDE moved files (e.g. `../improvement/TASK-IMP-068...`) MUST keep resolving: the migrator rewrites one directory level (`../<mod>/<file>.md` -> `../../<mod>/<STEM>/spec.md`) for repo-relative and sibling citations it can resolve; unresolvable relatives are listed for manual review, and `check_doc_anchors.sh` (extended to scan `docs/tasks/**/spec.md`) MUST exit 0 after migration.

## §2 - Why this design

Folder-per-task is the only layout that gives each spec a private asset namespace without a global assets dump. Doing the yaml repair in the same pass means the corpus goes from 91% to 100% machine-readable exactly once, and the backlog/roadmap split-brain (444 vs 486) dies at the root.

## §3 - Contract

Discovery: `docs/tasks/<module>/<STEM>/spec.md`; audit sibling `audit.md`; assets under `<STEM>/assets/**`, referenced relatively from spec.md (`assets/<file>`).

## §4 - Acceptance criteria

1. **Complete move** (§1 #1, #2) - after migration: zero `<module>/task-*.md` flat files; N folders == N pre-move tasks; audits paired; git log --follow works on a sampled spec.
2. **Idempotent** (§1 #2) - second run exits 0 reporting nothing to do.
3. **Tooling green on new layout** (§1 #3) - regen, roadmap, doc-anchors, and all repo suites pass post-move.
4. **Loud regen + reconciled totals** (§1 #4, #5) - zero skipped files; BACKLOG totals == roadmap totals == 486 (или the true count at migration time).
5. **Repairs are minimal** (§1 #5) - the yaml diff on the 42 files touches only frontmatter formatting (no value semantics changed - review-verified).
6. **Internal references resolve** (§1 #6) - check_doc_anchors extended to the task tree exits 0.

## §5 - Verification

`scripts/tests/test_task_layout.sh`: t01_no_flat_files, t02_folder_count_matches, t03_idempotent_rerun, t04_regen_loud_and_reconciled, t05_repairs_minimal (diff-scope), t06_anchors_green. (AC 1-6.)

## §6 - Implementation skeleton

Migrator: walk modules, `git mv` file->folder/spec.md + audit->audit.md, rewrite one-level relatives, emit summary. Regen: glob change + loud read_fm. Roadmap: walk change (folder stem = id source fallback). Doc-anchors: add task tree root + spec.md grammar.

## §7 - Dependencies

None upstream (foundation of Wave D). Blocks TASK-DOCS-005 (renderer walks folders) and TASK-SKILL-120 (skills scaffold folders).

## §8 - Example payloads

`docs/tasks/auth/TASK-AUTH-102-totp-webauthn-mfa/spec.md` + `audit.md`; regen stderr: `read_fm: 0 unparseable (was 42)`.

## §9 - Open questions

None blocking. Whether `.workflow/` bundles move INTO task folders is deliberately out of scope (session artefacts vs spec artefacts).

## §10 - Failure modes inventory

1. Half-completed move (crash mid-run) - migrator is per-file atomic (mv then next) and idempotent; re-run completes the remainder.
2. Tool missed by #3 - the full-suite gate in AC 3 is the net; grep inventory for `task-*.md` globs ships in the migration notes.
3. Sandbox/CI path assumptions in tests - fixtures build the NEW layout only; legacy fixture helpers deleted same commit.
4. yaml repair changes meaning - AC 5 restricts to formatting; reviewer diff-scope check.
5. External links into old paths (workflow bundles, memory rows) - historical artefacts keep old paths as history; only LIVE contracts are swept (same doctrine as TASK-SKILL-119).

## §11 - Implementation notes

Run the migrator FIRST in its commit, tooling updates in the same commit, so no commit exists where tools and layout disagree.

*End of TASK-DOCS-004.*
