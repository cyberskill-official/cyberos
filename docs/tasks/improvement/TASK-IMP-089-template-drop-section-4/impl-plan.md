---
artefact: implementation-plan@1
task_id: TASK-IMP-089
created: 2026-07-17
estimate_pts: 1
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 1)
---
# Implementation plan - TASK-IMP-089

Slices (each maps to spec §1 clauses and edge-case-matrix rows):
1. Template surgery - delete the `## 4. Out of scope / non-goals` heading + body up to the next heading (4 lines: heading, blank, `- ...`, blank) and renumber `## 5. Protected invariants this task must not weaken` to `## 4.` with the two body lines byte-untouched. Single anchored replace asserted to match exactly once, so a drifted template fails the edit instead of mangling it (§1.1, §1.2; rows 2, 4).
2. Shape oracle - `shape_why()` in test_template_schema.sh: any out-of-scope H2 (numbered at ANY number or unnumbered, case-insensitive; H3 exempt by construction since `^## ` cannot match `###`), exactly-one exact-literal invariants H2 at 4, no stray `## 5.` heading. One rule set shared by all three arms (rows 3, 4, 6).
3. Three t08 arms + harness line - `t08_single_out_of_scope_home` (live template through the oracle + invariants-body probe; §1.1/§1.2, AC 1, rows 1-2), `t08_duplicate_reintroduction_fails` (awk fixture re-adds the retired block above the invariants, arm demands the `duplicate-out-of-scope-H2` token specifically; §1.3, AC 2, row 5), `t08_payload_carries_shape` (scratch `build.sh "$TMP/payload"` then oracle + `cmp -s` byte-parity on payload/cuo/templates/TASK-TEMPLATE.md; §1.4, AC 3, rows 8-10); plus one `mktemp -d`/trap line after the counters (row 7).
4. Reference sweep - grep tools/, modules/, scripts/, README.md for "Protected invariants", numbered section-4/5 headings and out-of-scope H2s; update template-adjacent hits only. Result: zero exist outside TASK-TEMPLATE.md itself (near-misses adjudicated in context-map.md - historical-spec §5 citations in test headers and README channel numbering stay untouched per spec §3).

Pattern conformance (context-map): suite keeps `set -uo pipefail`, shared PASS/FAIL counters, ok/fail labels equal to the AC test names, WHY-block comments in the suite's own voice; scratch build never touches dist/ (payload-sync doctrine - the batch parent rebuilds before commit). Out of scope honored: no per-type template edits, no rubric change, no corpus rewrites, no pointer line added to the template.

Estimate: 1 pt (~1 h) - matches spec effort_hours: 1. Actual landed surface: 2 modified files, +62/-5 (TASK-TEMPLATE.md +1/-5; test_template_schema.sh +61/-0), 0 new files; suite 10/10 in ~1.2 s including the scratch payload build.
