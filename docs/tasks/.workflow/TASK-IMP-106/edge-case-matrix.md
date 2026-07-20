---
artefact: edge-case-matrix@1
task_id: TASK-IMP-106
total_rows: 16
created: 2026-07-18
verdict: pass (edge-case-matrix-audit: every category >=1 row; SECURITY rows point at a test or a
         justification; DEGRADATION rows carry detection+recovery; §1.4 rows enumerate the
         fail-closed surface path by path)
---
# Edge-case matrix - TASK-IMP-106

The summary is pure output over two inputs: what this run recorded as removed, and what is on disk when the run finishes. So the interesting edges are all *state of the tree*, and §1.4 - "a path that is not present MUST NOT be claimed as kept" - is the fail-closed rule the matrix has to attack from every side. Rows 4-9 are that attack: one row per kept path absent, plus the two ways the whole list can be wrong at once.

| # | category | trigger | expected behavior | covered by |
|---|----------|---------|-------------------|-----------|
| 1 | null/empty | no `.cyberos/` at all - uninstall on a repo that was never installed | exits at the existing `nothing to do` guard BEFORE the summary; prints no removed list and no kept list (§3 row 1: never claim a machine that was not there) | t21 arm 3 |
| 2 | null/empty | `.cyberos/` exists but nothing else does - no corpus, no page, no CHANGELOG, no BRAIN | `removed:` prints; the kept block is omitted ENTIRELY (no header, no `rm -rf` line with an empty argument list) | t21 arm 4 |
| 3 | null/empty | BRAIN dir present but EMPTY (`.cyberos/memory/store/` with no files) | still named as kept - existence is the test, not contents (§3 row 2: emptiness is not the uninstaller's judgment) | t20 arm (store seeded empty) |
| 4 | **§1.4 fail-closed** | `docs/status/` absent (page never rendered - the exact live defect) | `docs/status` MUST NOT appear anywhere in the output; the other three still do | **t21 arm 1** |
| 5 | **§1.4 fail-closed** | `.cyberos/memory` absent (`CYBEROS_UNINSTALL_KEEP_BRAIN=0` drops the store, so §5 never restores it) | `.cyberos/memory` MUST NOT be named kept; a second derivation arm on a different path, so "only docs/status is conditional, the rest hard-coded" cannot pass | **t21 arm 2** |
| 6 | **§1.4 fail-closed** | `docs/tasks/` absent (operator deleted the corpus before uninstalling) | not named; the summary does not resurrect it as a claim | t21 arm 4 (all-absent case subsumes) |
| 7 | **§1.4 fail-closed** | `CHANGELOG.md` absent | not named | t21 arm 4 (all-absent case subsumes) |
| 8 | **§1.4 fail-closed** | the derivation is replaced by a hard-coded list that happens to be right on a full install | t20 still passes (it is a full install) - so t20 alone CANNOT prove §1.4. t21 is the load-bearing arm and MUST fail on a hard-coded list. Proven by breaking it: see coverage-gate / this task's report. | **t21 (all arms)** |
| 9 | **§1.4 fail-closed** | the summary block silently prints nothing (a `set -e` abort, a bad guard) - every "MUST NOT name X" assertion passes vacuously | every negative assertion in t21 is paired with a POSITIVE one (`docs/tasks/` IS still named), so a summary that vanished fails the arm instead of passing it | **t21 arms 1-2** |
| 10 | bounds | `.cyberos/memory` exists but `.cyberos/` itself is otherwise gone (the normal KEEP_BRAIN path - §5 recreates exactly `.cyberos/memory/store`) | named kept; the `rm -rf` line names `.cyberos/memory`, not `.cyberos` (which would be a lie - the machine is already gone) | t20 |
| 11 | bounds | partially-removed machine from an interrupted earlier uninstall (e.g. hook already gone, `.cyberos/` still there) | `removed:` lists what THIS run removed, not what it wished it had (§3 row 4). The list is appended at each removal site, so an un-taken branch contributes nothing by construction | t20 (hook-absent fixture) + code shape |
| 12 | malformed | `docs/tasks` exists as a FILE, not a directory | the probe is `[ -e "$root/docs/tasks/" ]` - trailing slash - which is FALSE for a regular file, so a file named `docs/tasks` is not claimed as the corpus directory. Verified: `[ -e realfile/ ]` -> false, `[ -e realdir/ ]` -> true, on GNU bash 5.1 (the CI runner's shell; ubuntu-latest). This is `stat(2)` path resolution (ENOTDIR on a trailing slash), not a bash-version behavior, so it carries to macOS bash 3.2 - but that claim is REASONED, not measured, and is recorded as such. | code shape (documented in-line) |
| 13 | concurrency | a live install holds `.cyberos/.install.lock` (TASK-IMP-103's branch) | uninstall exits 1 at line 146 BEFORE the summary - no summary on a refused run, so nothing is claimed about a tree we did not touch | existing test_install_lock.sh::t05_uninstall_lock_ownership (unchanged by this task) |
| 14 | concurrency | a path appears or vanishes between the probe and the print | not defensible and not worth defending: the summary is a report about an instant, and the instant it reports is the probe. No lock, no retry. Recorded as accepted, not fixed. | ADR-free accepted risk (documented in-line) |
| 15 | SECURITY | a path name is interpolated into the printed `rm -rf` command | the four names are STRING LITERALS in the source, never read from the tree or from `$1`; the probe only decides *whether* a literal prints, never *what* prints. `$root` appears only in the surrounding prose ("run from <root>"), matching the existing `re-install:` line (uninstall.sh:165). Nothing shell-quotes back into a command the script runs - the script NEVER executes the printed command (§3 security row) | t20 (asserts the exact literal command) + code shape |
| 16 | DEGRADATION | the summary block itself fails (unbound var, bad probe) and takes the whole uninstall down under `set -e`, after the machine is already removed | detection: the block runs LAST - every filesystem mutation is already complete before the first summary line, so a summary bug can never leave a half-removed machine. recovery: re-running uninstall hits the `nothing to do` guard and exits 0. The summary is the only section with nothing after it, deliberately. | t22 (proves the block mutates nothing) |

## Category coverage (audit gate)

- null/empty: rows 1, 2, 3
- bounds: rows 10, 11
- malformed: row 12
- concurrency: rows 13, 14
- SECURITY: row 15 -> paired to t20 + an in-line justification; no ADR needed (no new attack surface: the block adds no input, no interpolation, and no execution)
- DEGRADATION: row 16 -> detection + recovery both named
- **§1.4 fail-closed: rows 4-9** - the reason this matrix is 16 rows and not 8

## The row that matters most

Row 8. A hard-coded kept list satisfies §1.1, §1.2 and §1.3 completely and violates §1.4 invisibly - on a full install the output is byte-identical to the correct implementation. t20 cannot see the difference and never will; that is not a flaw in t20, it is the shape of the defect. Only an arm that makes a claimed path ABSENT can tell a derivation from a recital. That arm is t21, and it is the one this task lives or dies on.
</parameter>
</invoke>
