---
artefact: implementation-plan@1
task_id: TASK-IMP-088
created: 2026-07-16
estimate_pts: 1
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 2)
---
# Implementation plan - TASK-IMP-088

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. Hoist `is_platform_repo()` above step 3b - definition moved verbatim (marker file test
   unchanged) with a comment explaining WHY it sits there (step 3b runs ~190 lines before
   the AGENTS.md handling, its other caller); the old site keeps a one-line pointer comment
   so a reader landing there is redirected (§1 #1.2; rows 3, 4).
2. Per-shape template line inside the create-once guard - `cfg_tmpl_line="task_template: task@1"`,
   then `is_platform_repo && cfg_tmpl_line="# task_template: engineering-spec@1"` (the
   file's own short-circuit idiom, safe under set -e in non-final && position), and the
   heredoc's commented line replaced by `$cfg_tmpl_line`. Every other heredoc line
   byte-identical to today (§1 #1.1, #1.2; rows 1, 2, 6).
3. Create-once untouched - no change to the `[ ! -f "$cfg_file" ]` guard; §1 #1.3 is
   satisfied by NOT writing code (row 5 proves it).
4. Hygiene scenarios t06_consumer_template_default / t06_platform_keeps_comment /
   t06_existing_config_untouched - harness house style, `_t06_install` speed-flag helper
   mirroring `_t05_install` (config scaffolding does not depend on migrate/memory/MCP),
   platform marker faked with a 0-byte file, existing-config arm cmp'd against a
   pre-install byte copy (§1 #1.4; all rows' covered-by).

Pattern conformance (context-map): create-once heredoc shape kept; short-circuit assignment
idiom precedented at install.sh:44/:137-141; suite discovered by run_all.sh's existing glob.
Out of scope honored: resolution chain, task-author prose, engineering-spec@1 profile,
existing-repo migration - none touched.

Estimate: 1 pt (~2 h) - matches spec effort_hours: 2. Actual landed surface: 2 modified
files (install.sh +16/-5 including the hoist and comments; test_install_hygiene.sh +50
including header doc lines), suite 17/17 in ~15 s.
