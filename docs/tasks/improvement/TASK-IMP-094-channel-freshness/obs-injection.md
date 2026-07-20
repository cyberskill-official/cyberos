# TASK-IMP-094 observability injection

The deliverable is install-time surface (symlinks, pointer files, gitignore lines) plus its uninstall inverse - nothing of it executes at consumer runtime, so there is no state transition to log, no external IO to span, and no error branch to count once install exits. Recording that honestly beats inventing telemetry for `ln -s`.

What stands in for observability:
- **install.sh emits its own trace**: every new path lands in the summary the operator reads (`pointer files:` gains the two rules pointers; `native skills:` gains `.claude/skills/task-author .claude/skills/task-audit .agents/skills/*` - captured in the gate log), and a re-vendor that adds them is visible in the consumer's `git status`.
- **uninstall narrates each removal**: one `  removed ...` line per stripped entry (`managed entry` vs `installer copy` says WHICH ownership test fired) - gate log E5.
- **The fallback is self-evidencing**: a copy where a symlink was expected is exactly what t_shared_skills_resolve prints on regression, and the `[ -e ]` post-check means a dangling link cannot survive an install even silently.
- **The suites are the monitor**: three channel scenarios + the t01 round-trip run on every suite invocation - the only recurring signal this change class can produce.

Branch coverage of the new install code: link-created (default), copy-fallback engaged (counterpart absent / ln failure - exercised structurally via the exclusion arm's want_agent short-circuit and asserted as a class by t_shared_skills_resolve), entry-exists skip (idempotence), family-filtered skip (exclusion arm). Uninstall: symlink-match, copy-match, foreign-lookalike no-match (reviewed), dir-prune both outcomes (t01 arm + platform repo keeps .agents/rules).
