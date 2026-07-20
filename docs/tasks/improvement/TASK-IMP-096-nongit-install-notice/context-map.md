# TASK-IMP-096 repo context map

## Cone
- `tools/install/install.sh` summary block, step 7 (one guarded line after the closing heredoc - lines 832-838)
- `tools/install/tests/test_install_hygiene.sh` (new t09_nongit_summary_line)

## Patterns the change must follow
- **Root-detection parity**: install resolves `root` via `git rev-parse --show-toplevel` (install.sh:30), NOT a `-d .git` probe. The new check MUST speak the same truth (`git -C "$root" rev-parse`), so a worktree/submodule where `.git` is a FILE counts as a checkout and a stale `.git` remnant does not. (Contrast: step 6b's hook block deliberately probes `-d .git` for hook *placement* - different question, not this task's to change.)
- **Install never refuses**: doc-only and evaluation installs on plain directories are legitimate; the line informs, it never gates (the spec's rejected alternative).
- **No commands run on the consumer's behalf**: the remedy is named verbatim, not executed.
- **Summary is the surface**: existing non-git hint lives in the hook line's aside ("skipped (not a git checkout)"); the new line is distinct prose so tests can tell them apart.

## Blast radius
- Files: 2 modified. One `if ! git rev-parse` + one echo at the summary's tail; git installs produce byte-identical output (t05_no_hookspath_regression's exact-line pin still green).
- Consumer impact: a non-git install now states what ship-tasks needs and the exact init command at install time - not at the first phase commit.
- Cross-module edges: install.sh + hygiene suite shared with TASK-IMP-094/095 - one agent, serial (this landed third).

## Module placement
Correct. `improvement` - installer guidance, not a product surface.
