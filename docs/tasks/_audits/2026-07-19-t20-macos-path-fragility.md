# Pre-existing: t20 uninstall-summary arm fails on macOS (path canonicalization) - 2026-07-19

Found while running `tools/install/tests/test_install_hygiene.sh` on the macOS host (bash
3.2) during IMP-126's coverage gate. Recording it as a SEPARATE finding: it predates
IMP-126 and is not caused by it.

## The failure

`t20_uninstall_summary_names_kept` fails on macOS with "removal command does not name the
directory to run it from" (line 487: `grep -qF "run this from $d"`). The suite is green on
Linux (CI) and red only on macOS.

## Root cause - version-independent

`_t2x_fixture` calls `mkrepo`, so the fixture is a git repo. `uninstall.sh:15` resolves the
repo root as `git rev-parse --show-toplevel`, which on macOS returns the PHYSICAL path
(`/private/var/folders/...`) because `/var` is a symlink to `/private/var`. The test builds
its expected string from the LOGICAL `$TMP` (`/var/folders/...`) that `mktemp -d` returned,
so the fixed-string grep misses. On Linux `/tmp` is not a symlink, so logical == physical
and the arm passes.

Proven version-independent: running BOTH the pre-IMP-126 `uninstall.sh` (commit 4ac2fd12)
and the patched one in a git-repo fixture on macOS, each emits the physical path and each
misses the logical-path grep. The `run this from $root` echo is byte-identical across the
two versions. IMP-126 did not touch root resolution.

## Not IMP-126's

IMP-126's own four arms (t_mcp_registration_removed, t_no_dangling_skill_links,
t_hook_strip_byte_identical_across_cycles, t_operator_files_preserved) pass on macOS bash
3.2 and on Linux bash 5. The suite is 26/0 on Linux and 25/1 on macOS, the 1 being this
pre-existing t20 fragility.

## Suggested fix (separate task)

Make the test canonicalize its expected root the same way the uninstaller does, e.g.
compute `root="$(cd "$d" && git rev-parse --show-toplevel 2>/dev/null || (cd "$d" && pwd))"`
and grep `"run this from $root"`. Audit sibling arms in the same file for the same
logical-vs-physical assumption (any arm that greps a `$TMP`/`$d` path against the
uninstaller's echoed root). Cone would be `tools/install/tests/test_install_hygiene.sh`
only. This is small and self-contained; a good candidate to fold into a "test-suite macOS
portability" task alongside the bash-3.2 lesson from IMP-126.

No code change was made here. This is a recorded finding.
