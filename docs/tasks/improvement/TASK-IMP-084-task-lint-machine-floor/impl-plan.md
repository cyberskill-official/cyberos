---
artefact: implementation-plan@1
task_id: TASK-IMP-084
created: 2026-07-16
estimate_pts: 3
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 5)
---
# Implementation plan - TASK-IMP-084

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. Strict frontmatter reader + FM family in tools/install/docs-tools/task-lint.mjs -
   `readFrontmatter`/`parseScalar`/`splitInline` accept exactly the task@1 subset (flat
   scalars, quoted strings with escapes, inline + block lists, own-line comments) and emit
   FM-001 naming the line for everything else; `checkFrontmatterFields` carries the RUBRIC §2
   enums verbatim (FM-101..114, FM-109 rejecting `unacceptable`, FM-111 demanding the literal
   boolean via a quoted-ness flag, FM-113 resolving against docs/tasks/** names, FM-114 iff
   type bug); FM-112 is a raw-region scan so a marker fires even when parsing already failed
   (§1.2; rows 3, 4, 5, 6, 7).
2. Body model + SEC/COND - fenced-code and `<untrusted_content>` interiors are shadowed
   (data, not structure) before heading collection; SEC-001..007 exact-H2 presence, SEC-008
   non-empty-before-next-heading, SEC-009 warning (one H1, no level jumps); COND-001..004
   triggered off the parsed frontmatter with H3-order and bullet-label checks (§1.3, §1.4;
   rows 10, 11).
3. TRACE structural halves - clauses harvested from `## 1. Description` (`- 1.N` lines),
   ACs from `/^- \[[ x]\] AC /`; TRACE-001 citation via `#1.N`, `§1 #N`, or a traces_to
   token, `(deferred to slice` exempt, zero clauses -> info note; TRACE-002 test:/verify:
   presence; TRACE-003 backticked `path::name` in test: entries against frontmatter
   new_files or disk relative to the walked-up repo root (§1.5; rows 1, 2).
4. CLI shell - `[--json]`, dir recursion to `*/spec.md` in bytewise order, findings
   formatted `severity rule_id file:line message`, bytewise-sorted, exit 0/2, unreadable
   or non-task@1 input -> `template_ambiguous` error and per-file stop (§1.1, §1.6, §1.7;
   rows 8, 9, 12).
5. Gating suite tools/install/tests/test_task_lint.sh (t01-t08 per the spec's AC names,
   rubrics-vendored harness shape, single green fixture + one-mutation red fixtures so each
   scenario isolates exactly one rule) + the two-line build.sh vendor copy + the two-paragraph
   SKILL.md machine-floor wiring (§1.8, §1.9; rows 3-12 "covered by", t06/t07/t08).

Pattern conformance (context-map): node stdlib only (node:fs, node:path - no new imports
class), ESM single file, loud failures; suite mirrors test_rubrics_vendored.sh counters/
summary/exit and its build-then-scratch-install pattern; run_all.sh:43 discovers the suite
with zero wiring. Out of scope honored: no QA/SAFE/XCHAIN/STALE checks, no auto-fixing, no
engineering-spec profile, no run-gates.sh wiring.

Estimate: 3 pts (~5 h) - matches spec effort_hours: 5. Actual landed surface: 2 new files
(task-lint.mjs 588 lines, test_task_lint.sh 289 lines), 2 modified (+4 SKILL.md, +2 build.sh,
zero deletions), suite 8/8 in ~2 s including the payload build and scratch install.
