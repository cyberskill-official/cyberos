---
artefact: edge-case-matrix@1
task_id: TASK-IMP-084
total_rows: 12
created: 2026-07-16
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions, SECURITY rows point at code+test, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-084

All test functions live in tools/install/tests/test_task_lint.sh.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | spec with zero numbered `- 1.N` clauses under `## 1. Description` (pure PRD shape) | `info TRACE-001` note ("clause traceability left to the model audit"), exit stays 0 - the template allows judgment there | t05_trace_family (zero-clause arm) |
| 2 | null/empty | empty inline list `new_files: []` plus verify:-only ACs | parses to an empty list; TRACE-003 has nothing to resolve; fully green fixture exits 0 with empty output | t01_cli_and_determinism (green arm) + t05_trace_family (verify arm) |
| 3 | null/empty | `template` key absent entirely | FM-004 `template_ambiguous` at error severity and the file STOPS - no downstream SEC/COND/TRACE findings ride on a template the lint never detected (proven by the exactly-one-error assert) | t02_fm_family (FM-004 arm) |
| 4 | bounds | title of 80 code points (limit is 72 after trim, counted in code points - a display bound, not a byte bound, per spec §3) | FM-101 error naming the count; 68-71-62-point green-corpus titles pass untouched | t02_fm_family (FM-101 arm) + t06_green_corpus |
| 5 | bounds | author handle regex bounds `^@[A-Za-z0-9_.-]{1,38}$` - a bare `tester` without the `@` | FM-102 error quoting the offending value | t02_fm_family (FM-102 arm) |
| 6 | malformed | exotic YAML the strict subset refuses (anchor `&keep` on a value) | FM-001 error NAMING THE LINE (asserted against a grep -n of the fixture); parse continues so one cause yields one finding, never a silent skip | t02_fm_family (FM-001 arm) |
| 7 | malformed | `priority: p9`, own-line `# UNREVIEWED` surviving in frontmatter, `severity` on a non-bug | exactly FM-105 / FM-112 / FM-114 respectively, one error each, exit 2 - the guardrail metric's "each fixture produces exactly its rule_id" | t02_fm_family |
| 8 | concurrency/order | two consecutive runs on identical input, text and `--json` | byte-identical stdout (cmp) - findings bytewise sorted, traversal sorted, no clock/env/random text anywhere in the output | t01_cli_and_determinism |
| 9 | concurrency/order | directory input holding multiple `*/spec.md`, green and red mixed | recursion finds every spec.md in bytewise path order; cross-file findings interleave stably across runs | t01_cli_and_determinism (dir arm, run twice) |
| 10 | SECURITY | lint consumes hostile spec bytes: it reads files and never executes content - imports are node:fs + node:path only, no child_process, no network, no eval; `--json` output is data | inspection (task-lint.mjs import block) + t06_green_corpus (runs over real specs carrying `<untrusted_content>` blocks); NOTE per spec §3: SAFE-003 injection-marker scanning stays with the model audit - nobody may assume the lint covers it |
| 11 | SECURITY | quoted foreign bytes trying to alter verdicts: a `# H1` heading or AC-shaped line INSIDE `<untrusted_content>` or a code fence | shadowed - untrusted/fenced interiors count as body content (SEC-008) but never as structure (headings/clauses/ACs), so quoted content cannot add or mask SEC/COND/TRACE findings | t06_green_corpus (TASK-IMP-084's own spec embeds a `# ...` H1 inside its untrusted block and lints clean with zero warnings) |
| 12 | DEGRADATION | unreadable input (missing path); lint dropped from the payload; auditor forgets the lint exists | detection: missing path -> `template_ambiguous` error + exit 2, never a guess or a crash (t01); t07 gates payload presence, `.cyberos/docs-tools/` lay-down AND a live run of the installed copy; t08 gates the lint-first wiring in all three SKILL.md copies. recovery: build.sh guarded copy is a one-line re-vendor; re-install lays the tool back down | t01_cli_and_determinism (missing-path arm), t07_payload_and_install, t08_skill_wiring_present |

Documented-by-design (spec §3): CRLF and BOM are normalized for parsing and heading matching, reported nowhere - content bytes are the corpus's business. FM-113 resolution is the single piece of cross-file state and scans docs/tasks/** names only (no file reads); exercised in code, kept out of the fixture set because no batch-1 spec is a `duplicate` - the model audit still owns the semantic question of whether a duplicate link is the RIGHT link.
