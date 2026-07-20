---
artefact: implementation-plan@1
task_id: TASK-IMP-099
created: 2026-07-17
estimate_pts: 1
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected including the t09 pin discovery, estimate sane vs spec effort_hours 1)
---
# Implementation plan - TASK-IMP-099

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. Reword the queue-selection rule in modules/cuo/chief-technology-officer/workflows/ship-tasks.md (line 312 after TASK-IMP-097's insertion; the spec cites it pre-batch as 311). One physical line replaced: "order by priority (MUST before SHOULD before COULD), then" becomes "order by priority: `p0` before `p1` before `p2` before `p3` (legacy MoSCoW values map per FM-105), then" - the sentence tail (`created` ascending, id ascending, "Echo the") and the echo-line format keep their exact bytes. (§1 #1.1; rows 1, 2, 5.)
2. Bump frontmatter workflow_version 2.6.3 -> 2.6.4 in the same file - a normative selection rule changed wording, so the version moves with it. (§1 #1.2; row 3.)
3. Move BOTH exact version pins in tools/install/tests/test_workflow_helpers.sh to `^workflow_version: 2\.6\.4$`: t12 (spec-declared) and t09 (discovered - the identical pin at line 456 pre-change; the spec's source_pages named only t12, but leaving t09 would ship a red suite and violate AC 3). Failure messages updated to say 2.6.4; the file-header comment entries for t09 and t12 corrected so no comment asserts a version the test no longer pins. Disclosed in code-review.md. (§1 #1.2; rows 3, 4.)
4. Add t13_queue_rule_p0_p3 (and its header-comment entry + `want t13` run line): reuses the suite's existing ensure_payload helper (one scratch build, cached across t08/t09/t12/t13); for the SOURCE and the scratch payload's cuo/ship-tasks.md it greps the p0-p3 ordering phrase, greps the FM-105 legacy-mapping parenthetical, and negatively greps the MoSCoW rule SHAPE (`(MUST|SHOULD|COULD|WON.?T)[[:space:]]+before[[:space:]]+(MUST|SHOULD|COULD|WON.?T)`, case-insensitive; both sides must be MoSCoW values so single-sided prose and the parenthetical can never trip it), then pins the payload copy at 2.6.4. Pattern proven both ways by recorded probes: the retired wording is caught, the parenthetical is allowed. (§1 #1.3; rows 1, 2, 5, 6, 10.)

Pattern conformance (context-map): the rule edit is byte-minimal inside the Resume semantics block; rank code untouched (modules/cuo/cuo/ship_manifest.py `_PRIORITY_RANK` already ranks both scales - out of scope honored); the suite grows in its own idiom (want-gated function, distinct fail messages, ensure_payload reuse); the exact-pin discipline is preserved, not loosened - the alternative version-agnostic regex was spec-rejected.

Estimate: 1 pt (~1 h) - matches spec effort_hours: 1. Actual landed surface: 2 modified files, 0 new (ship-tasks.md 2 lines rewritten; test_workflow_helpers.sh +40/-10: two pins, header comments, t13 + run line); suite 13/13 in ~3 s, sibling payload suite 9/9 re-run green as the guardrail.
