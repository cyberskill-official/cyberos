# TASK-IMP-083 — implementation plan (as executed)

Contract: spec §1 (1.1–1.7) + ACs 1–8. Constraint that shaped every edit: when `core.hooksPath` is unset, step 6b's written bytes and summary words must be EXACTLY today's (§1.4) — so the change is a resolver in front of an untouched state machine, not a rewrite of it.

## Step 1 — install.sh step 6b: resolve, don't restructure

- Inside the existing `else` (git checkout present), ahead of the state machine: `hooks_path="$(git -C "$root" config core.hooksPath 2>/dev/null || true)"`; empty (unset OR set to "") → `hooks_dir="$root/.git/hooks"`; `case`: absolute (`/*`) used as is, anything else anchored `"$root/${hooks_path%/}"`.
- `hk="$hooks_dir/pre-commit"`; `mkdir -p "$hooks_dir"`. With hooksPath unset both expand to the exact strings the old lines produced (`$root/.git/hooks[/pre-commit]`).
- `hook_at=""` normally; `" at ${hooks_path%/}/pre-commit"` when hooksPath is set. The five `HOOK_SET` strings gained `${hook_at}` at the natural point ("installed at …", "…existing pre-commit hook at …") — empty expansion keeps every word identical (§1.3/§1.4).
- State machine, both hook heredocs, `chmod +x`, the non-git skip branch, and the `CYBEROS_NO_HOOK` gate: untouched.

## Step 2 — uninstall.sh: same resolver + exact ownership

- Same four-line resolver (comment credits step 6b) so uninstall operates on `<hooks_dir>/pre-commit` and never touches `.git/hooks/pre-commit` when hooksPath points elsewhere (§1.5).
- Ownership test replaced: `head -5 "$hk" | grep -q cyberos-status-hook` → `_cyberos_owns_hook` copied verbatim from install.sh (line 2 equals the managed standalone header AND is not the appended `>>>` form). The comment names the bug: a foreign hook shorter than five lines carrying our appended block put the marker inside `head -5`, matched the heuristic, and was deleted whole. Ours-outright → `rm -f`; foreign-with-block → the existing sed marker-range strip, foreign body preserved.
- Enabling fix found while wiring the tests: uninstall.sh line 11's `root=` resolution was mis-grouped — `cd && git rev-parse … || cd … && pwd` parses as `((A && B) || C) && D`, so after a successful rev-parse `pwd` ALSO ran and `$root` captured two newline-joined paths; `"$root/.cyberos"` never existed and every uninstall on a git repo exited "nothing to do". Grouped explicitly: `( (cd && rev-parse) || (cd && pwd) )`. Without this, the hook section (and ACs 4/8) is unreachable dead code. Same file, minimal edit, commented with the task id.

## Step 3 — hygiene suite: t05_hookspath_* block

Seven scenarios + `_t05_install` helper (installs with `CYBEROS_NO_MIGRATE/NO_MEMORY/ NO_MCP=1` — the hook path under test needs none of it, and t05_hookspath_standalone deliberately leaves the page unrendered so the COMMIT proves the hook fired). Functions called at the foot after the existing t01–t06, shared counters:

1. `t05_hookspath_standalone` — hook lands executable at `.githooks/pre-commit`, v2 header on line 2, `.git/hooks/pre-commit` NOT created; node-gated: backlog commit fires the hook from the configured dir (docs/status/ in the commit tree).
2. `t05_hookspath_foreign_append` — 3-line foreign hook exiting 7: first line byte-preserved, markers exactly once, re-install does not duplicate, exit 7 re-raised.
3. `t05_no_hookspath_regression` — hook at `.git/hooks/pre-commit`, header line 2, exact `grep -qF` on today's auto-sync summary line, no hooksPath wording or path suffix.
4. `t05_hookspath_uninstall` — vendored uninstall removes from `.githooks/`, `.git/hooks` untouched.
5. `t05_short_foreign_uninstall_preserved` — the head-5 regression: strip-only, foreign bytes `cmp`-equal, exit-7 kept; construction self-check asserts the marker IS inside `head -5` so the scenario keeps failing the old heuristic by construction.
6. `t05_summary_names_path` — install output contains " at .githooks/pre-commit" (asserts on the standalone scenario's captured output; same install, one fewer run).
7. `t05_non_git_skip` — plain dir, summary carries "skipped (not a git checkout)".

## Verification loop (workspace, Linux, git 2.34, node 22)

- Baseline before edits: 6/6 green, 7.8 s.
- Run 1 after edits: 11 passed / 2 failed — both uninstall scenarios; root-caused to the pre-existing uninstall root-resolution bug above (uninstall never reached its hook section). Fixed (step 2, third bullet).
- Run 2: 13/13 green, ~10.5 s. Re-run after the re-entry assert: 13/13, ~8.5 s.
- Old-heuristic proof: `git show HEAD:tools/install/uninstall.sh` with ONLY the root line patched (to make its hook section reachable) run against a 3-line foreign hook + real appended block → prints "removed managed pre-commit hook", file deleted whole.
- Regression proof beyond the suite: no-hooksPath install compared against the pre-change committed payload (`dist/cyberos/install.sh`, zero `hooksPath` matches): hook bytes `cmp`-identical, auto-sync summary line `diff`-identical.
- `bash -n` clean on all three edited files. `dist/` untouched; rebuild + version-sync + full `run_all.sh` belong to the batch parent per payload-sync doctrine.
