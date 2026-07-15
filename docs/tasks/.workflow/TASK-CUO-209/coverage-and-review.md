---
artefacts: repo-context-map@1 + edge-case-matrix@1 + implementation-plan@1 + observability-injection@1 + coverage-gate@1 + code-review@1 (bundled)
task_id: TASK-CUO-209
tests_failed: 0
tests_passed: 8
files_below_90pct: []
ecm_rows_uncovered: []
created: 2026-07-12
verdicts: all pass - human verdict pending at HITL gate 1
---
# Ship artefact bundle - TASK-CUO-209

## Context map / plan (condensed - single-tool task)
Domain: tools/cyberos-init only (build.sh set block, sizes; GUIDE source docs/index.md; README;
allowlist; test). files_outside_immediate_domain: 0 -> no ADR. Edge rows folded into the suite:
52-skill matrix (t01), data-not-string regression trap (t02), computed counts full vs reduced
(t03 + t07), 14-row map totality + invoker enum (t04), sibling checks (t05), 2 MB budget (t06),
reduced floor (t07), frozen workflows (t08).

## Coverage
Suite t01-t08 green; ALL five cyberos-init suites green as regression (10+7+9+8+8 = 42 cases).
Live build: profile=full skills=52 payload=8499200 plugin_zip=1029894 (49% of the 2 MB budget);
chain OK: 24 referenced, 52 vendored, 6 allowlisted.

## §1 clause -> evidence
| #1 full catalog vendored (52) | t01 | passed |
| #2 set is reviewable data | t02 | passed |
| #3 counts computed, drive profile | t03 (+ t07 reduced=0) | passed |
| #4 lifecycle map 14 rows, no TBD, invoker enum | t04 | passed |
| #5 sibling checks green | t05 (chain-coverage; pair-parity SKIP pending TASK-SKILL-118, visible) | passed |
| #6 sizes + 2 MB budget assert | t06 | passed |
| #7 reduced floor intact | t07 (skill-less fixture -> profile=reduced, stamps ok) | passed |
| #8 workflows untouched | t08 (git diff scoped, clean) | passed |

## Field finding folded upstream (recorded on TASK-SKILL-116)
t07 surfaced that the chain-coverage check broke the reduced floor (zero vendored = every
reference "missing"). Amended: zero-vendored payloads print `chain SKIP: reduced profile` and
exit 0; partial vendoring still fails. TASK-SKILL-116 §1 #5 + audit §11 updated; its suite gained
t07_reduced_profile_skips (now 7/7).

## Reviewer attention points
1. Plugin zip grew 320 KB -> ~1.0 MB (52 skills). Budget headroom 51%; the trim fallback
   (payload keeps all, plugin trims) stays documented in the task §10 if skill-selection quality
   ever regresses.
2. GUIDE map marks contract level per pair (full/thin) - expectation-setting for the thin
   upstream pairs until TASK-SKILL-118 deepens them.
3. The four NFR allowlist entries document the UNPAIRED exemption reasons inline.

## Verdict requested
Review acceptance (HITL gate 1): approve to advance, or reject with findings.
