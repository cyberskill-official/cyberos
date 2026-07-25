---
audit_template_version: "audit_rubric@2.0"
audited_file: "docs/tasks/improvement/TASK-IMP-022-ban-defensive-asserts/spec.md"
audited_file_sha256_prefix: "8be4d61e6854d3cd"
audited_body_sha256_prefix: "42e3d96eab783952"
rubric_version: "audit_rubric@2.0"
skill_id: "task-audit"
skill_version: "1.0.0"
last_audit_at: "2026-07-25T22:30:00+07:00"
overall_status: "pass"
score: 10/10
iterations: 2
issue_counts: { total: 7, open: 0, needs_human: 0, fixed: 6, wontfix: 1 }
machine_floor: "task-lint.mjs run FIRST — 0 errors, 1 info (TRACE-001 heading shape)"
trace_id: "batch-12g-imp-022-2026-07-25"
---

# Audit — TASK-IMP-022 (ban defensive asserts)

Machine floor first per TASK-IMP-084: `node tools/install/docs-tools/task-lint.mjs` → 0 errors, 1 info. The info is TRACE-001's heading-shape rule (`## 1. Clauses` rather than `## 1. Description`), identical to TASK-IMP-118 and TASK-IMP-117; traceability itself is discharged in the TRACE section below.

This spec was authored from an unauthored draft stub (34 lines, "author the normative clauses when this task is picked up") whose only content was the R13 reference. Every claim in it is therefore ORIGINATED, which puts the whole document under TRACE-007's harshest reading — see ISS-002.

## Findings

**ISS-001 (fixed, iteration 1) — the lint was specified before the corpus was measured.**
The first pass proposed four detectors including shell polarity inference (`if <A||B>; then ok` weak, `… then fail` strong). Measuring first showed the corpus holds four live `if [ ! -f A ] || [ ! -f B ]; then fail` sites — all correct — and zero weak ones. A rule whose only live matches would be false positives is uninstallable. Resolved: polarity inference moved to Alternatives with the measurement as the rejection reason, and to Scope → Out of scope; TRACE-008's judgment half named as what covers it. §Alternatives, §Scope, ECM row 13.

**ISS-002 (fixed, iteration 1) — R13 was treated as an input rather than a claim.**
R13 says "grep-audit for or-conditions inside asserts". The first pass implemented exactly that and inherited both of a grep's failure modes on this corpus. Under TRACE-007 the author must re-derive what they originate, and "a grep suffices here" is an originated claim. Resolved: the derivation is on disk — a grep flags `assert sources == {"dropbox-or-gdrive", "syncthing"}` (`modules/memory/tests/core/test_sync_conflicts.py:97`) and misses the backslash-continued disjunction at `test_store_acl.py:239`. Both directions are pinned as tests, not asserted in prose: `t02_da001_ignores_or_inside_a_string`, `t03_da001_catches_multiline_disjunction`. Recorded as **re-derived and CORRECTED** in the disclosure. §1.2, AC2, AC3.

**ISS-003 (fixed, iteration 1) — the lint flagged its own negative fixtures.**
`test_assert_lint.sh` must contain `grep -q needle "$f" || true` to prove DA-003 fires, so the first clean run of the live corpus came back with the suite's own heredoc bodies as findings. Waiving them was rejected: the waiver would then live inside the fixture and travel into the scratch file, defeating the arm it was placed to protect. Resolved on the semantics instead — a heredoc body is *data*, not shell the file executes, so DA-003/DA-004 (claims about executed shell) do not apply to it. Skipping heredoc bodies is correctness, not an exemption, and it is what lets an anti-defensive-assertion lint carry the fixtures that prove it fires. The consequence — assertions inside an embedded `python3 - <<'PY'` block are outside the floor — is stated as a bound in the lint header, in TRACE-008, and in §Scope rather than left to be discovered. §1.4, ECM row 10.

**ISS-004 (fixed, iteration 1) — "clean corpus" was asserted before it was true.**
The draft claimed the corpus scanned clean while six `DA-001` sites were live. The temptation at that point is the defect itself: widen the detector, or waive the six, and the sentence becomes true. Resolved by replacement — each of the six was probed by running the code under test and reading the single result (`error` is `yaml_parse: …`; the channel is `warnings`, `issues` is empty; both composition strings present; `details == "no binlog segments"`; `binlog.exists()` is False; `allowed=False, mode='read'`), then asserted exactly. Zero waivers. §1.7, AC8; the six replacements carry a one-line `TASK-IMP-022 DA-001` note naming what the old disjunction let through.

**ISS-005 (fixed, iteration 2) — AC8's pass counts were unbound numbers.**
`276` and `521` are ORIGINATED NUMERIC claims (TRACE-007). Prose provenance does not discharge them. Resolved: both are reproducible — `cd modules/cuo && python3 -m pytest tests/ -q` and `cd modules/memory && python3 -m pytest tests/ -q` — and the derivation is recorded in `testing-evidence.md` §3 with the pre-change and post-change runs side by side, including the one PRE-EXISTING red (`test_schema_single_source.py`, red on `bb161013` before this branch existed) so the guardrail is not quietly credited with a failure it did not cause.

**ISS-006 (fixed, iteration 2) — the corpus bound was implied, not stated.**
"The gated test corpus" reads as "the tests", and `services/**` holds far more assertions than `modules/**` does. A green run that silently means "not looked at" is the placebo R13 names at line 125. Resolved: the bound is stated three times where a reader could rely on the green — the lint's own header, TRACE-008's **Bound** paragraph, and §Scope → Out of scope — each naming `docs/batches/batch-10e-imp-stub-wont-do.md` as the authority for the 1.x payload boundary. ECM row 13 carries no test by design and says so.

**ISS-007 (wontfix-info) — the waiver mechanism ships with zero users.**
`# defensive-assert-ok:` has no live call site: all six findings were fixed, not waived. Deleting it and adding it on first need was considered and rejected — a lint with no escape hatch gets deleted wholesale at the first legitimate disjunction, which is worse than one unused affordance. It is bounded rather than unbounded: the reason is mandatory (`DA-005` otherwise), every active waiver prints on every run, and `t08` pins the corpus at zero waivers so the first one has to be argued in review. Accepted as info.

## Rubric families

- **FM:** clean. `type: improvement`, `priority: p1` (not MoSCoW), `template: task@1`, both `# UNREVIEWED` markers cleared under the recorded operator confirmation of `ai_authorship` / `eu_ai_act_risk_class` — FM-112 satisfied by a human verdict, not by deletion.
- **SEC:** all seven required sections present and substantive.
- **COND:** three-bullet AI Authorship Disclosure with the TRACE-007 partition labels; `re-derived and CORRECTED` carries the grep-vs-AST measurement, `measured and ADDED` carries the honest zero-bite finding for the shell half. COND-004's shape check is green, and per its own message a green shape check is not a verified disclosure — the partitions are tested above.
- **QA:** §3 carries 13 rows across all six categories, with 2 SECURITY and 2 DEGRADATION. Above the 8-row floor. Rows 9 and 10 are the sharpest: a waiver used to silence a real defect (bounded, not prevented — and the spec says *bounded*), and a heredoc fixture that the floor deliberately cannot see.
- **SAFE:** adds one read-only scanner. It executes nothing from the corpus it reads; `ast.parse` never evaluates, and the shell side never invokes a line it scans. No network, no writes.
- **TRACE:** 1.1–1.10 each cite a test; AC1–AC10 each cite a clause. TRACE-006 applied to this spec's own clauses: 1.2's verb is *flag / not flag*, discharged by t02 and t03 asserting exit code AND rule id AND file:line, not by the presence of a rule name; 1.7's verb is *replace*, discharged by t08 (zero findings, zero waivers) plus both pytest suites holding — not by the lint alone, which a waiver would also satisfy. TRACE-008 applied to itself: every one of t01–t10 has a fixture or a corpus state that makes it red, and t01–t07 exist for no other reason.

## Verdict

**pass — 10/10.** The task closes R13 with both halves it asked for and refuses the version of itself that would have been easier to ship: no grep, no baseline file, no waived six. The one place it is weaker than it looks — `services/**`, and assertions inside embedded heredoc interpreters — is stated in three places rather than left for a reader to infer from a green run.

*End of TASK-IMP-022 audit.*
