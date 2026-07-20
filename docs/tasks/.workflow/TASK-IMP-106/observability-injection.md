---
artefact: observability-injection@1
task_id: TASK-IMP-106
created: 2026-07-18
verdict: pass (observability-injection-audit)
branch_coverage: 8/8 branches carry an output point (100%, threshold 80%)
---
# Observability injection - TASK-IMP-106

## The honest framing

This task's deliverable **is** observability. The change adds no state machine, no external IO, and no error branch - it makes an existing silent decision (what uninstall keeps, and why) legible at the one moment it surprises someone. So "inject observability into the new code" collapses into "the new code is the observability", and the audit below measures whether every branch of it actually says something.

There is no structured-log/trace/counter surface to hook: uninstall.sh is a POSIX-shell script an operator runs by hand and reads on a terminal. Its log format is `cyberos uninstall: <section>` / `  <action>`, established across all 20 pre-existing echo lines (repo-context-map). Emitting JSON lines or opening a trace span here would be observability for a consumer that does not exist. The consumer is a human, and the terminal is the sink.

`tenant_id` / `subject_id` are N/A: no tenant model, no request context, no personal data. The one identifier in scope is `$root`, and it is already printed on line 18 (`target=$root`).

## Branch table - every branch of the new code has an output point

| # | branch | condition | what it emits | silent? |
|---|--------|-----------|---------------|---------|
| 1 | removed list printed | `_removed_list` non-empty | `  removed:` + one indented line per recorded removal | no |
| 2 | removed list skipped | `_removed_list` empty (unreachable in practice - `rm -rf "$CY"` always records) | nothing, deliberately: an empty `removed:` heading asserts a run that removed nothing, which is false by construction here | intentional |
| 3 | `_keep` hit | `[ -e "$root/<path>" ]` true | the path + its one-line reason, and the path joins the `rm -rf` line | no |
| 4 | `_keep` miss | probe false | nothing - this IS §1.4. Emitting "docs/status/: absent" would be noise about a thing the operator never had | intentional |
| 5 | kept block printed | `_kept_paths` non-empty | heading + lines + the `run this from <root>` prose + the `rm -rf` command | no |
| 6 | kept block skipped | nothing kept survives | nothing - no heading, no bare `rm -rf` (edge row 2) | intentional |
| 7 | never-installed exit | `[ ! -d "$CY" ]` (pre-existing, line ~30) | `cyberos uninstall: nothing to do (no .cyberos/)` and exits 0 before the summary | no |
| 8 | refusal under a live install | TASK-IMP-103's lock branch | the refusal on **stderr** + exit 1, before the summary | no |

branches_with_output_point: 8/8 (100%) — threshold is 80%.

The four "intentional silence" rows are the point of the task, not a gap: each one is a case where saying something would be saying something false. §1.4 is exactly the rule that silence about an absent path beats a claim about it.

## Error branches / counters

None added. The block introduces no new exit path and no failure mode of its own:

- It runs LAST. Every filesystem mutation is complete before its first line, so a bug here cannot leave a half-removed machine (edge row 16).
- The `while read` loop uses `[ -n "$_r" ] || continue` rather than `&& echo`, because an `&&` list that short-circuits leaves a non-zero status as the loop body's last command and `set -e` would abort the run *after* the machine is gone. The summary must never fail the run it reports on.
- `_keep` returns 0 on both paths explicitly.

Counter increments for error branches: N/A - there are no error branches and no metrics sink in a hand-run shell script. Recorded as N/A rather than fabricated as 0.

## What a reader of the output can now reconstruct

Before: that the machine was removed. Nothing about the corpus, which is the thing they were worried about. After: which entries this run removed, which of their four work-paths survived and why, the exact command to finish the job by hand, and - by omission - which of those paths were never there.
</parameter>
</invoke>
