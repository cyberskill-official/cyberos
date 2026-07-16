# TASK-IMP-096 implementation plan

1. **The line** (clauses 1.1, 1.2) - install.sh:832-838, directly after the summary heredoc's
   `EOF` (so it is the summary's last word, the position a first-time consumer actually sees):
   `if ! git -C "$root" rev-parse --show-toplevel >/dev/null 2>&1` -> echo the spec's exact
   line. Git checkouts take the else-nothing path - zero new output.
2. **Detection semantics** (spec edge 1) - `git rev-parse` against `$root`, matching the root
   resolution at install.sh:30; never `-d .git` (a worktree's `.git` FILE must stay silent,
   a stale remnant must not).
3. **Coverage** (clause 1.3) - hygiene t09_nongit_summary_line: non-git arm (count=1 +
   verbatim remedy), stale-.git-remnant arm (rev-parse semantics pinned), git arm (zero).
   CYBEROS_NO_MIGRATE speed path per the clause.
4. **Gates** - hygiene 19/19; live non-git scratch capture in the gate log.

Order: last of the install.sh trio (after TASK-IMP-094/095, same file, same serial agent).
