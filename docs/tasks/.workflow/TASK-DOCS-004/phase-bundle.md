# TASK-DOCS-004 phase bundle (batch-mode ship)

## implementation
Yaml corpus repair FIRST (42 files, 63 lines quoted - iterative minimal-quoting on exactly the line
yaml trips on; zero manual): corpus now 100% strict-yaml. Migrator scripts/migrate_fr_layout.py:
491 FR specs + audits moved to <module>/<STEM>/{spec.md,audit.md}, idempotent, sweep rewrites
repo-root + one-level-relative + same-dir citations across live trees (archives/.workflow excluded);
2 write-protected memory test files chmod'd + swept. Tooling on the new layout: regen (folder glob,
stem-from-folder, LOUD read_fm - unparseable files listed on stderr), roadmap walker, data-extract
walker (frs.json for the catalog), doc-anchors extended to the FR tree with corpus-wide planned-files
rule (any spec's new_files legitimizes a citation - TRACE-003's own semantics) + status-aware severity
(done/closed/on_hold spec refs warn as historical; active FRs must resolve). Fixtures folderized.

## field findings folded in
- TASK-PLUGIN-003 (draft) + TASK-TEN-002 (ready_to_implement) cited planned files absent from their own
  new_files - repaired (top-level new_files entries; TEN-002's wave-1 nested shape noted).
- 28 module READMEs are stale hand-written indexes (chat/ lists the RETIRED pre-101 generation) -
  exempted with reasons; the status hub (TASK-DOCS-006) becomes the living index.

## verification
test_fr_layout.sh 6/6 (AC 1-6: zero flat files, 491 folders==491 specs, idempotent, regen loud +
backlog==roadmap==491, corpus strict-yaml, anchors green 386 refs). Full regression: 8 repo suites +
ship_manifest 8/8 + site build + payload build all green.

## HITL record
Gates covered by the operator's standing batch verdict (2026-07-12 in-chat PLAN approval:
"approve PLAN + start shipping... ship in batch, non-stop") - recorded per CyberOS doctrine as the
human approval for reviewing->ready_to_test and testing->done of this batch's FRs.
