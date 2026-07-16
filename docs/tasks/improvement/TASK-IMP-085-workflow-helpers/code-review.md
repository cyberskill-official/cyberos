# TASK-IMP-085 — code review packet

Files under review: new `tools/install/docs-tools/ship-manifest.mjs` (418 lines),
`tools/install/docs-tools/backlog-mutate.mjs` (394 lines),
`tools/install/tests/test_workflow_helpers.sh` (455 lines, executable); modified
`tools/install/build.sh` (+3, guarded vendor copies) and
`modules/cuo/chief-technology-officer/workflows/ship-tasks.md` (+9/−1, two pointer
passages + the workflow_version bump disclosed below). Suite state at review:
test_workflow_helpers 9/9, 0 failed (~2.5 s including payload build + scratch
install); test_task_lint re-run 8/8 (build.sh was touched; its t07/t08 payload
gates stay green). Other dirt in the same working tree (`docs/tasks/BACKLOG.md`,
`docs/release/`, TASK-IMP-086/087 artefacts) belongs to batch siblings and is
covered by their own packets.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | five commands (init/record/verify/resume-line/delete) on `docs/tasks/.workflow/<task-id>.ship.json` | `t01_manifest_lifecycle` — init creates the manifest at the mandated path (asserted literally), record writes steps, verify walks, resume-line echoes, delete removes + is idempotent; re-init without --force refuses (exit 2); verify on an unknown id is a loud exit 2. Path shape is code-pinned (`manifestPathFor`); task-id is a filename component by regex (ID_RE), never a path |
| 1.2 | two-phase atomic writes (`.tmp.<nonce>` + rename); `task_sha256` + `workflow_version` pinned at init; step entries `{index, skill, status, artefact_path, artefact_sha256, verdict, completed_at}` hashed at record time | `t02_two_phase_atomic` — planted garbage `.tmp.deadbeef` never corrupts reads or writes, is never consumed, and successful writes leave no tmp litter; `task_sha256` equals an independently computed sha256 of the spec, `workflow_version` pinned verbatim; a missing artefact at record time is exit 2 ("unreadable at record time"), never a null hash for a claimed file. `t01` asserts the full seven-field entry shape, the record-time hash, and the pinned-clock `completed_at` |
| 1.3 | verify exits 3 (version) / 4 (task hash) / 5 (earliest stale artefact, step named) / 0 (intact, first non-done step); resume-line echoes the mandated format | `t03_verify_staleness_exits` — 2.0.0-vs-1.0.0 → exit 3 with "needs_human"; edited spec → exit 4 with "task_sha256 mismatch"; BOTH artefacts tampered → exit 5 naming step 1 and NOT step 2 (earliest wins); restored → exit 0 naming step 3/31; resume-line inherits exit 5 on staleness and, intact, equals the mandated string byte-for-byte (whole-line `=` compare in t01 and t03, including `(K artefacts, hashes OK)` and `routed_back_count=R`; `--routed-back` bumps R in t01). `<skill>` resolution: recorded step > workflow-doc skill_chain parse > "unknown" (both non-recorded branches exercised: t01 via wf.md chain, t03 via the "unknown" fallback) |
| 1.4 | flip: locate by stem, verify status cell + full old line bytes, exit 6 on drift/missing/duplicate, rewrite exactly that line, update counted headers | `t04_flip_and_drift_refusal` — green flips rewrite only the cell (line count unchanged, ` (improvement)` suffix bytes preserved because the rewrite splices the cell prefix, never re-renders the row); wrong cell → exit 6 "drifted"; wrong `--old-line` → exit 6 "byte-for-byte"; matching `--old-line` passes; missing row → exit 6; duplicated stem → exit 6 naming both line numbers. Counts: `t06_counts_maintained` |
| 1.5 | insert: uniqueness anywhere in the file (exit 7), regenerator-identical grammar, stem-ascending placement in the contiguous block, counts updated | `t05_insert_uniqueness_and_grammar` — the whole rendered row is asserted with `^...$` anchors (grammar byte-exact vs regen_backlog()); TASK-ALPHA-002 lands between 001 and 003 (awk order proof); a second insert of the same id with a DIFFERENT slug into a DIFFERENT section is exit 7 (uniqueness is file-wide, id-keyed); unicode title places by stem only; placeholder `- (nothing remaining)` is replaced as the first row; the bare `## gamma` header is untouched; 0-candidate auto-detect refuses with exit 2 naming `--section`. Counts: `t06` |
| 1.6 | stdlib-only, deterministic, `--json` envelopes, exit codes in `--help`, whole-file discipline (1 row + ≤1 header) | `t07_json_and_determinism` — both `--help`s grep-carry the exit-code tables (3/4/5 and 6/7) + the CYBEROS_NOW doc; envelopes parse and carry ok/exit_code/old_line/new_line/new_header (error envelopes too, exit 6 arm); manifests, verify envelopes, flip envelopes, and mutated backlogs are cmp-identical across reruns; the whole-file diff proof: flip = exactly 1 changed row + 1 changed header, insert = 1 added row + 1 changed header, no-counts flip = 1 changed row only; CRLF round-trips with the inserted row inheriting the section's ending. Stdlib: import blocks are node:fs/node:path/node:crypto only — no child_process, no network, no eval (both files) |
| 1.7 | build.sh vendors both tools via guarded copies; a payload without them fails t08 | `t08_payload_and_install` — payload `docs-tools/` carries both files non-empty AND cmp-identical to source; scratch install lays both into `.cyberos/docs-tools/`; the INSTALLED copies run (--help) and complete an init+verify lifecycle inside the scratch repo. build.sh:175-177 uses the block's exact `[ -f ] && cp` idiom next to the task-lint copy |
| 1.8 | two pointer sentences in ship-tasks.md: Resume semantics names ship-manifest.mjs alongside ship_manifest.py; backlog-layout names backlog-mutate.mjs as the byte-discipline executor | `t09_doctrine_wiring` — section-scoped greps (awk-extracted "Resume semantics" and "Backlog layout" spans) find `ship-manifest.mjs` + the retained `ship_manifest.py` reference, and `backlog-mutate.mjs` + the literal "byte-discipline executor", in the modules/ source AND both payload copies (cuo/ship-tasks.md, plugin/skills/ship-tasks/cuo/ship-tasks.md); `workflow_version: 2.6.2` asserted in all three |
| 1.9 | suite at `tools/install/tests/test_workflow_helpers.sh` covering lifecycle, atomicity, staleness exits, drift, uniqueness, counts, determinism, payload+install, doctrine | the suite itself — t01..t09 with the spec's exact AC function names; discovered by scripts/tests/run_all.sh:43's `tools/install/tests/test_*.sh` glob with zero wiring (AC 10's ops check: the glob names the dir and the file matches `test_*.sh`; run_all itself is the batch parent's gate run per swarm doctrine) |

## Acceptance criteria

AC 1 `t01_manifest_lifecycle` ok · AC 2 `t02_two_phase_atomic` ok ·
AC 3 `t03_verify_staleness_exits` ok · AC 4 `t04_flip_and_drift_refusal` ok ·
AC 5 `t05_insert_uniqueness_and_grammar` ok · AC 6 `t06_counts_maintained` ok ·
AC 7 `t07_json_and_determinism` ok · AC 8 `t08_payload_and_install` ok ·
AC 9 `t09_doctrine_wiring` ok. Suite 9/9. AC 10: path + glob discovery verified
as the ops check above (run_all.sh execution stays with the batch parent —
sub-agents do not run whole-workspace gates mid-batch).

## workflow_version bump (disclosure)

The task's pointer prose alone would not require a version bump, but the two
passages are NORMATIVE pointers (they tell doc-driven agents which executable
implements the section they are reading), so `workflow_version` was bumped
2.6.1 → 2.6.2. Consequence, by the contract's own design: any ship-manifest
pinned at 2.6.1 verifies as exit 3 (needs_human) on next resume — the pin doing
its job, not a regression; manifests are gitignored session state, none are
committed. t09 asserts 2.6.2 in source and both payload copies.

## Deviations / implementation decisions (disclosure)

- The manifest carries one field beyond SHIP-MANIFEST.md's table: `task_file`
  (root-relative spec path, recorded at init). Without it the spec's own §1 #1.1
  signature `verify <task-id>` cannot re-hash the spec. The python `validate()`
  checks required fields and tolerates extras; `--task-file` on verify overrides.
- `record --routed-back` (a flag, not a new command) carries `routed_back_count`
  across route-backs — the workflow's terminal rule needs a writer for the count.
- verify's current-version source is `--workflow-version` > `--workflow-doc` >
  auto-discovery (modules/ or .cyberos/ doc); with NO source it skips the check
  with a loud printed note rather than failing (a repo with no doc cannot have a
  mixed-version run) — documented in --help.
- `insert` refuses to CREATE sections (exit 2). SKILL.md §2b sketches section
  creation for the regenerator; doing it here would exceed the spec's own §1 #1.6
  discipline (1 row + ≤1 header) and duplicate regen conventions (Totals, module
  ordering). Spec §1.5 presumes an existing section; the refusal names the fix.
- A counted header whose counts do not cover the from-status is left untouched
  (never decremented negative, never rewritten into a lie); the row still flips.
- `--old-line` comparison excludes the line terminator (the recorded pre-image in
  a backlog-state-update artefact never carries CR/LF); all other bytes exact.
- Row-injection guard beyond the spec text: titles carrying newline bytes are
  exit 2 (a title could otherwise smuggle a second row through the one-row
  contract); stems must be single whitespace-free tokens. Proven in t05.
- backlog-mutate validates statuses against the 12-value STATUS-REFERENCE enum
  (regen's 10-value STATUS_ORDER first, `cannot_reproduce`/`duplicate` appended)
  so a legal frontmatter status is never refused; headers render in that order.

## Diff size

Three new files (1,267 lines total, all executable), two modified: build.sh +3/−0
(guarded copies in the docs-tools block's idiom), ship-tasks.md +9/−1 (version
bump + the two pointer passages). No dependency added anywhere. `dist/` untouched
here — rebuild, version-sync and full suite before commit are the batch parent's
step per payload-sync doctrine.

## Verdict

| Check | State |
|---|---|
| §1 clauses 1.1–1.9 | each proven above by a named test or pinned line |
| Primary metric (both contracts machine-executed: manifest reproduces the resume line; drifted flip refused non-zero) | pass (t01/t03 exact-line compare; t04 exit-6 arms) |
| Guardrail metric (atomicity + drift fixtures on every gate run) | pass (t02/t04 in the run_all-discovered suite) |
| Determinism contract (byte-identical, injectable clock documented) | pass (t07: manifests, envelopes, mutated files) |
| Whole-file discipline (diff = 1 row + ≤1 header, CRLF preserved) | pass (t07 footprint + CRLF arms) |
| ship-tasks.md ripple | disclosed; 2 pointer passages + sanctioned 2.6.2 bump, gated by t09 |
| Invariants (§5: frontmatter is truth, grammar authority unchanged, payload doctrine, HITL) | intact (tools encode contracts; regen stays byte-authority; t08/t09 gate the payload; no done-flip anywhere) |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
