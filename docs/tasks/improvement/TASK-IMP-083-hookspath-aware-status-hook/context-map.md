# TASK-IMP-083 — repo context map

Scope: teach install/uninstall to land and remove the status-sync pre-commit hook where `core.hooksPath` points. Everything below is what the implementation had to know about.

## Files touched (owned by this task)

| File | Role | What changed |
|---|---|---|
| `tools/install/install.sh` (step 6b, ~lines 533–716) | writes the managed pre-commit hook via an ownership state machine (absent/ours → standalone v2; foreign → marked append; v1 append → upgrade; v2 → keep) | hooks-directory resolver added ahead of the UNCHANGED state machine; `hk` now derives from `hooks_dir`; the five `HOOK_SET` strings gained `${hook_at}` (empty when hooksPath unset — byte/word regression contract) |
| `tools/install/uninstall.sh` (hook section, formerly lines 23–35) | removes the managed hook or strips the appended block | same resolver; ownership test replaced: `head -5 \| grep -q` heuristic → exact line-2 `_cyberos_owns_hook` copied from install.sh step 6b. Enabling fix: the `root=` resolution on line 11 was mis-grouped (`(A && B \|\| C) && pwd`) so `$root` captured two newline-joined paths and EVERY uninstall on a git repo exited "nothing to do" — grouped explicitly so the hook section is reachable at all |
| `tools/install/tests/test_install_hygiene.sh` | install hygiene suite (t01–t06 existing) | new `t05_hookspath_*` block: 7 scenarios + `_t05_install` helper, called at the foot after the existing calls, shared PASS/FAIL counters |

## Upstream / adjacent (read, not modified)

- `tools/install/build.sh` — copies `install.sh`/`uninstall.sh` verbatim into the payload (lines 157–158); the test suite builds a scratch payload from the working tree, so the suite always exercises the edited scripts. `dist/cyberos/` is the committed payload and is rebuilt by the batch parent (payload-sync doctrine) — deliberately not written here.
- `install.sh` step 1 — vendors `uninstall.sh` to `.cyberos/uninstall.sh`; the uninstall scenarios invoke that vendored copy, matching real operator flow.
- Hook bodies (standalone v2 heredoc, POSIX append block) — byte-identical, out of scope.
- `run-gates.sh` / docs-site sync — untouched (spec non-goals).
- The cyberos repo itself sets `core.hooksPath=.githooks` — the live affected repo that motivated the task; nothing here depends on that setting (tests build scratch repos).

## Key mechanics relied on

- git ≥ 2.9 executes hooks from `core.hooksPath` (relative → repo root, absolute → as is), falling back to `.git/hooks` only when unset; `git config` merges local/global scope, which is exactly the scope git itself uses to pick the hooks dir — so resolving via `git -C "$root" config core.hooksPath` is faithful by construction (worktrees included).
- The managed standalone hook always carries `# cyberos-status-hook v2 (managed by cyberos install)` on line 2; the appended form's marker line contains `>>>`. Line 2 + the `>>>` exclusion is the exact ownership separator at any file length (install.sh's own documented fix, now shared with uninstall).
- Suite conventions mimicked: `mkrepo`, per-scenario `$TMP` repos, `ok`/`fail` counters, node-gated assertions, and `CYBEROS_NO_MIGRATE/NO_MEMORY/NO_MCP` to keep installs fast.
