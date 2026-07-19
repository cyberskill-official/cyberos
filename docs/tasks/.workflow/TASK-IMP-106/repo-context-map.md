---
artefact: repo-context-map@1
task_id: TASK-IMP-106
created: 2026-07-18
verdict: pass (repo-context-map-audit)
---
# Repo context map - TASK-IMP-106

Scope: `tools/install/uninstall.sh` (the summary block) + `tools/install/tests/test_install_hygiene.sh`
(three new arms). Everything below was read out of the files named, not recalled.

## Baseline patterns the new code MUST follow

- **error_convention**: `set -euo pipefail` at the top; informational lines to stdout; a refusal
  goes to stderr and exits non-zero (`echo "cyberos uninstall: an install is running ..." >&2;
  exit 1`, uninstall.sh:146-147). The summary is informational -> stdout, no new exit paths.
  *pinned_in*: tools/install/uninstall.sh:8,146-147
- **echo/reporting style** (uninstall.sh, all 20 echo lines read):
  - section-level: `cyberos uninstall: <text>` at column 0 (lines 18, 23, 146, 163)
  - per-action: two-space indent, past tense, verb first - `  removed .cyberos/`,
    `  stripped cyberos block from pre-commit`, `  removed .agents/skills/<x> (managed entry)`
    (lines 65, 70, 81, 99, 102, 114, 152)
  - continuation of a per-action line: seven-space indent (line 108)
  - a parenthetical rider carries the reason: `(managed entry)`, `(installer copy)`,
    `(unmarked skill dir - not an installer copy we can prove; ...)` (lines 99, 102, 107-108)
  -> the new summary uses the same two-space/indent ladder and the same reason-in-line habit.
    It does NOT invent a box, a banner, or colour.
- **section headers**: numbered comments `# 1. pre-commit`, `# 2. managed .gitignore block`,
  `# 2b. shared .agents/skills entries`, `# 3. BRAIN store`, `# 4. remove machine`,
  `# 5. optional restore brain`, `# 6. skill symlinks ...` (lines 27, 74, 85, 121, 133, 154, 162).
  The summary lands as `# 6.` - the file's own numbering, not a new convention.
- **comment habit**: each non-obvious branch carries a paragraph naming the bug it fixed and why
  the code is shaped that way (lines 11-14, 28-31, 43-51, 86-93, 134-137). The new block explains
  the hard-coded-list defect it replaces, in that voice.
- **test_framework**: standalone bash, one function per case, no framework. `ok`/`fail` counters,
  `local all=1` + `[ "$all" -eq 1 ] && ok tNN` for multi-assert arms, fixed-string `grep -qF`.
  *pinned_in*: tools/install/tests/test_install_hygiene.sh:34-35,42-91
- **test: literals ARE the test**. t01's header (lines 47-49) warns that the stale-wording needles
  must not be swept to current wording, and t05_no_hookspath_regression:252 greps today's summary
  line as a fixed string. Encoding expected output verbatim, with a comment saying so, is the
  house style for "unchanged vs today".
  *pinned_in*: test_install_hygiene.sh:47-49, 252-253
- **test: construction checks**. An arm that constructs a precondition asserts the precondition
  actually holds, so the arm cannot silently stop testing anything
  (`"construction broken: marker outside head -5, old bug not exercised"`).
  *pinned_in*: test_install_hygiene.sh:288-289; test_install_lock.sh:79
- **test: exercise the REAL code, never a copy**. The sibling suite slices live blocks out of
  install.sh/uninstall.sh with `sed -n '/anchor/,/anchor/p'` and runs them, guarding with
  `[ -n "$blk" ] || no ... "block not found"`. t22 follows this shape (strip-and-compare rather
  than copy-and-compare).
  *pinned_in*: tools/install/tests/test_install_lock.sh:12-18, 78-79, 125
- **test: speed flags**. Arms that do not need migrate/memory/MCP install with
  `CYBEROS_NO_MIGRATE=1 CYBEROS_NO_MEMORY=1 CYBEROS_NO_MCP=1` (`_t05_install`/`_t06_install`).
  *pinned_in*: test_install_hygiene.sh:194, 322
- **suite registration**: arms are declared as functions, then called in a flat list at the bottom
  (lines 442-460), then the counter line. New arms append to that list.

## Schemas / interfaces in scope

- uninstall.sh's own state machine, sections 1-5: hook removal/strip, .gitignore block, the
  .agents/.claude skill entries, the BRAIN stash/restore round-trip, `rm -rf "$CY"`. TASK-IMP-103's
  lock-ownership branch (lines 138-150) is present and merged - the dependency is satisfied on disk,
  not merely in the backlog.
- Kept-path surface per spec §1.2: `docs/tasks/`, `docs/status/`, `CHANGELOG.md`, `.cyberos/memory`.
  `.cyberos/memory` exists after a run ONLY via section 5's restore (line 155-160) - i.e. only when
  KEEP_BRAIN is on AND a store existed. That is precisely why §1.4 is a derivation, not a list.
- `build.sh:160` is a verbatim `cp "$here/uninstall.sh" "$out/uninstall.sh"` - the vendored payload
  copy is byte-identical to source, so an assertion against the vendored copy is an assertion
  against source. (Checked: `cmp` in the implementation plan's verification.)

## Files outside the immediate domain

None. Both touched files are inside the declared `service: tools/install`:

1. tools/install/uninstall.sh (declared in `modified_files`)
2. tools/install/tests/test_install_hygiene.sh (declared in `modified_files`)

files_outside_immediate_domain: 0 (<= 3 -> no ADR; steps 3-4 skip by condition)

## Blast radius

file_count: 2 | module_count: 1 (tools/install) | cross_module_edges: none
dist/ is NOT rebuilt by this task (build.sh copies uninstall.sh verbatim; the payload refresh is
the operator's pre-push step per ship-tasks "Pre-push / pre-install re-verification").
module_placement_warning: null (module `improvement` = cross-cutting; correct per ship-tasks §1a)

## Recorded decision (no ADR - not architectural)

Today's line 164 reads `kept: docs/tasks/, docs/status/, CHANGELOG.md, AGENTS.md / pointer files`.
Spec §1.2 enumerates four kept paths and `AGENTS.md / pointer files` is not among them; §1.3 pairs
the kept list with a `rm -rf` command. Naming the agent pointer files inside a list whose companion
command deletes that list would tell an operator to delete machine surface under the heading "your
work". The new block names exactly the spec's four. The pointer files are still kept on disk
(unchanged - §1.5); they are simply no longer described as the operator's corpus.
This drops the substring `AGENTS.md / pointer files` from uninstall's output. Nothing greps it
(verified: `grep -rn "AGENTS.md / pointer" tools/ scripts/ .github/` -> only uninstall.sh:164).
</content>
</invoke>
