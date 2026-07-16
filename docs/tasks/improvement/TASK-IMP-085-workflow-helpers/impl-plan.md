---
artefact: implementation-plan@1
task_id: TASK-IMP-085
created: 2026-07-16
estimate_pts: 4
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 6)
---
# Implementation plan - TASK-IMP-085

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. ship-manifest.mjs shell + manifest IO - five commands on
   `docs/tasks/.workflow/<task-id>.ship.json` under a `--root` walk (docs/tasks or
   .git, cwd fallback); stable-stringify (sorted keys) so identical state is
   identical bytes; two-phase `atomicWrite` (`.tmp.<nonce>` + fsync + rename)
   mirroring ship_manifest.py's `write_atomic`; readers open the exact path and
   ignore tmp by construction; `init` pins `task_sha256` + `workflow_version` and
   scaffolds `.workflow/.gitignore`; `record` writes the seven-field step entry
   hashing the artefact AT RECORD TIME (missing artefact = loud exit 2) with
   `--routed-back` carrying the count; injectable clock `CYBEROS_NOW`/`--now`
   (§1.1, §1.2; rows 1, 5, 9, 11, 13).
2. The staleness walk + resume line - `walk()` implements the workflow's order with
   distinct exits: current-version source is `--workflow-version` > `--workflow-doc`
   frontmatter > auto-discovered doc (modules/ or .cyberos/ path), else a loud
   skip note; version mismatch exit 3 (needs_human), task re-hash mismatch exit 4
   (all stale, history retained), ascending artefact re-hash exit 5 naming the
   EARLIEST stale step, intact exit 0 naming the first non-done step
   (skipped-conditional resolves, per resume_plan); `resume-line` reuses the walk
   (it never claims hashes OK it did not prove) and echoes the mandated string
   with `<skill>` from the recorded step, else the doc's skill_chain parse, else
   `unknown` (§1.3; rows 3, 7).
3. backlog-mutate.mjs flip - CR-preserving line model over a raw `\n` split; row
   located by stem (`<id>` or `<id>-` prefix on the `- [status] stem - title`
   grammar); refusals exit 6: zero rows, 2+ rows naming both lines, status cell
   != `<from>`, `--old-line` byte drift; the rewrite replaces ONLY the cell prefix
   (`- [from]` -> `- [to]`) so title/tags/comments/CR bytes survive; nearest `## `
   header above gets counts updated only when it parses the regen counts grammar
   exactly (from -1, to +1, zeros dropped, STATUS_ORDER order; uncovered counts =
   header untouched, never a lie) (§1.4, §1.6; rows 4, 6, 8, 12).
4. backlog-mutate.mjs insert - whole-file uniqueness gate first (any row for the
   id or exact stem anywhere = exit 7 naming the line); target section by
   `--section` exact name or auto-detected as the unique section already holding
   the `TASK-<MOD>-` prefix (0/2+ candidates = exit 2, never a guess); the row
   renders regen-identical (+` (improvement)` under `--class`), placed
   stem-ascending in the contiguous block, placeholder replaced as the first row;
   guards: stem is one whitespace-free token, titles reject newline bytes
   (row-injection), no section creation (regen owns it, exit 2)
   (§1.5, §1.6; rows 2, 10, 11).
5. Gating suite tools/install/tests/test_workflow_helpers.sh (t01-t09 per the
   spec's AC names, task-lint harness shape: want-filter, mktemp+trap, ok/fail
   counters, cached ensure_payload) + the two guarded build.sh vendor lines in the
   docs-tools block's own idiom + the two ship-tasks.md pointer passages (Resume
   semantics -> ship-manifest.mjs alongside ship_manifest.py; Backlog layout ->
   backlog-mutate.mjs as the byte-discipline executor) + the workflow_version
   2.6.1 -> 2.6.2 bump the pointers justify (§1.7, §1.8, §1.9; rows 9, 13).

Pattern conformance (context-map): node stdlib only (node:fs, node:path,
node:crypto), ESM single files, loud refusals over guesses, `--json` as data;
determinism kept absolute by making the one contract-required clock injectable
instead of weakening the byte-identity rule; grammar authority stays with
regen_backlog() - the tool encodes, never redefines (spec §5). Out of scope
honored: no contract semantics changes, no BRAIN emission (IMP-05), no coverage
scoping (IMP-14), no skill-prose rewiring beyond the two sentences.

Estimate: 4 pts (~6 h) - matches spec effort_hours: 6. Actual landed surface:
3 new files (ship-manifest.mjs 418 lines, backlog-mutate.mjs 394 lines,
test_workflow_helpers.sh 455 lines, all executable), 2 modified (build.sh +3,
ship-tasks.md +9/-1), suite 9/9 in ~2.5 s including the payload build and
scratch install; test_task_lint.sh stays 8/8 after the build.sh touch.
