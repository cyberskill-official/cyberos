---
artefact: implementation-plan@1
task_id: TASK-IMP-106
created: 2026-07-18
verdict: pass (implementation-plan-audit: every edge-case row addressed; existing patterns
         respected per repo-context-map; estimate within capacity)
estimate_pts: 2
---
# Implementation plan - TASK-IMP-106

## Slice 1 - record removals at the moment they happen (§1.1)

`_note_removed <text>` appends to a newline-delimited `_removed_list`, called beside each
existing `echo "  removed ..."`. Defined near the top, ABOVE section 1, because sections 1, 2,
2b, 4 all call it.

- Newline-delimited string, NOT a bash array: `set -u` + an empty array is an error on bash < 4.4,
  and the operator's macOS shell may be bash 3.2. The file's other accumulators
  (`KEEP_BRAIN_STASH`) are plain strings too.
- Appending at the removal site is what makes §1.1 and edge row 11 true *by construction*: a
  branch that did not run cannot contribute a line. There is no second list to keep in sync.

Call sites (each keeps its existing echo byte-for-byte - the live per-action log is unchanged):
| uninstall.sh | existing echo | recorded as |
|---|---|---|
| :65 | `removed managed pre-commit hook` | `<hooks_dir>/pre-commit (managed hook)` |
| :70 | `stripped cyberos block from pre-commit` | `<hooks_dir>/pre-commit (our block only - the file is not ours)` |
| :81 | `removed managed .gitignore block` | `.gitignore (managed block only)` |
| :99 | `removed .agents/skills/<x> (managed entry)` | `.agents/skills/<x> (managed entry)` |
| :102 | `removed .agents/skills/<x> (installer copy)` | `.agents/skills/<x> (installer copy)` |
| :114 | `removed .claude/skills/<x> (managed entry)` | `.claude/skills/<x> (managed entry)` |
| :152 | `removed .cyberos/` | `.cyberos/ (the vendored machine)` |

## Slice 2 - derive the kept list from disk (§1.2, §1.4)

`_keep <path> <reason>`: probe `[ -e "$root/<path>" ]`; on miss `return 0` and contribute
nothing. On hit, append a formatted line and the bare path to `_kept_paths`.

- The four paths are STRING LITERALS at the call sites (edge row 15): the probe decides
  *whether* a literal prints, never *what* prints. No tree data reaches the output.
- `docs/tasks/`, `docs/status/`, `.cyberos/memory/` are probed WITH a trailing slash so a
  regular file of that name is not claimed as the directory (edge row 12).
- Display spelling follows spec §1.2 exactly: `docs/tasks/`, `docs/status/`, `CHANGELOG.md`,
  `.cyberos/memory`.
- The kept block (header + lines + `rm -rf`) prints only when `_kept_paths` is non-empty, so a
  bare repo gets no empty-argument `rm -rf` (edge row 2).

## Slice 3 - the manual-removal command (§1.3)

One line: `    rm -rf <the derived paths>`, preceded by prose naming `$root` as the working
directory. `$root` stays OUT of the command (edge row 15); the existing `re-install:` line
(uninstall.sh:165) is the precedent for `$root` in prose.

## Slice 4 - placement (§1.5, edge row 16)

The block replaces today's section 6 (lines 162-165) and stays LAST. Every filesystem mutation
completes before the first summary line, so a summary bug cannot leave a half-removed machine.
It adds no `rm`, no `mkdir`, no redirect into a file, and no `exit`.

Section header stays in the file's own numbering (`# 6.`) - and `# 6.` to EOF is the anchor
t22 strips on, so the header is load-bearing, not decoration.

## Slice 5 - the suite (3 arms)

| arm | AC | clause | shape |
|---|---|---|---|
| `t20_uninstall_summary_names_kept` | AC 1 | §1.1, §1.2, §1.3 | full fixture: all four kept paths seeded explicitly (NOT relying on install to render a page - a node-less runner would otherwise silently weaken the arm). Asserts: removed names `.cyberos/`; each of the four kept paths appears WITH its reason on the same line; the `rm -rf` command is asserted as one exact fixed string. |
| `t21_uninstall_summary_derived_not_hardcoded` | AC 2 | §1.4 | 4 arms: (1) `docs/status/` absent -> unnamed, `docs/tasks/` still named; (2) `CYBEROS_UNINSTALL_KEEP_BRAIN=0` -> `.cyberos/memory` unnamed, others still named; (3) never-installed repo -> no kept block at all; (4) `.cyberos/` only, nothing else -> no kept block, no bare `rm -rf`. |
| `t22_uninstall_behavior_unchanged` | AC 3 | §1.5 | two identical fixtures; A runs the real vendored uninstall, B runs the SAME vendored script with `# 6.`-to-EOF stripped. Post-run file trees must be byte-identical. Construction checks: the anchor exists, and the stripped copy really lost the summary. |

**Why every t21 negative is paired with a positive.** A summary that failed to print satisfies
every "MUST NOT name X" assertion. Each arm therefore also asserts a path that MUST still be
named. This is the TASK-IMP-118 defect class (an assertion that does not fail when its clause is
violated), and it is the single most likely way this task ships broken.

**Why t22 strips rather than diffs against git.** Pinning a baseline SHA rots: a later task that
legitimately changes what uninstall removes would fail t22 for a reason that has nothing to do
with §1.5. Stripping the block compares the state machine against itself, so both sides move
together and the arm keeps measuring exactly one thing - the summary's side effects. Slicing live
code out of the script with `sed` is the sibling suite's own idiom (test_install_lock.sh:78, 125),
construction check included.

## Verification (before any commit)

1. `bash -n tools/install/uninstall.sh` - parses. NOT sufficient on its own: a fix inside a
   heredoc passes `bash -n`. There is no heredoc here; the block is plain `echo`/`printf`.
2. `bash tools/install/tests/test_install_hygiene.sh` - the whole suite, not just the new arms
   (t01 and t05_hookspath_uninstall already drive uninstall and must stay green).
3. `bash tools/install/tests/test_install_lock.sh` - the sibling suite slices uninstall.sh by
   anchor (`/^_ul=.../,/^fi$/`); my edits must not move those anchors.
4. Break each new arm, watch it fail, restore. Recorded in the report.
5. Read the file, not just the output: `sed -n` the final block and check it against the spec.

## Non-goals (spec §Scope)

No `--purge` flag. No prompt. No change to the state machine. No `dist/` rebuild. No install.sh.
</parameter>
</invoke>
