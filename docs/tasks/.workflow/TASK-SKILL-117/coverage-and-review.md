---
artefacts: coverage-gate@1 + code-review@1 (bundled)
task_id: TASK-SKILL-117
tests_failed: 0
tests_passed: 12
files_below_90pct: []
ecm_rows_uncovered: []
created: 2026-07-12
verdicts: coverage pass; code review pass - human verdict pending at HITL gate 1
---
# Coverage gate + code review packet - TASK-SKILL-117

## Verification method (contract work - acceptance-driven per task §5)
Both TRIGGER_TESTS.md files open with an executable pair-verification preamble; every line was run green: artefact id grammar + timebox HALT + evidence rule declared on the author side (AC 1, 3, 4); SPK-STRUCT/EVID/BOX/DISC families + the 10/10 bar on the audit side (AC 5); full-pair file layout present on both sides (AC 7). 12 preamble assertions total, 0 failures. Trigger suites: author 4 positive + 5 negative, audit 4 positive + 4 negative (AC 2, >= 3/3 each). ADR-author now names architectural-spike@1 with the lean fallback (AC 6). The audit fixture case table (CASE-01..06) pins the evidence rule, box arithmetic, unprobed-recommendation, confidence cross-check, and discard honesty (AC 4 + edge rows 1-7).

## Diff summary (15 files)
new: modules/skill/architectural-spike-author/{SKILL,PIPELINE,INVARIANTS}.md,
     envelopes/{input,output}.json, references/FAILURE_MODES.md, acceptance/TRIGGER_TESTS.md
new: modules/skill/architectural-spike-audit/{SKILL,RUBRIC,AUDIT_LOOP,REPORT_FORMAT}.md,
     envelopes/{input,output}.json, acceptance/TRIGGER_TESTS.md
mod: modules/skill/architecture-decision-record-author/SKILL.md (spike input + lean fallback)

## Conformance notes
- Frontmatter matches the catalog conventions byte-for-byte in structure (Identity / Scope contract / Inputs-outputs / Triggers-blockers blocks, verified against debugging-cycle-author and implementation-plan-author).
- Descriptions 793 and 747 chars - under the 1024 host limit (TASK-SKILL-111 lineage).
- NOT vendored: build.sh untouched per task §11 (TASK-CUO-209 owns the expansion); fresh build + chain-coverage stays green (24 referenced, 22 vendored, 2 allowlisted).
- ADR-author still carries the dead SDP anchor - deliberately untouched here; that is TASK-SKILL-119's sweep.

## Reviewer attention points
1. architectural-spike@1 is a NEW artefact contract - its shape is now normative; a future change means a version bump through the contracts CHANGELOG.
2. The confidence cross-check constant (high => >= 2 evidence per surviving option) is a judgment call encoded as HIGH_CONFIDENCE_MIN_EVIDENCE - easy to tune later.
3. audit stage metadata is "d" (pre-ADR) on both sides, matching ADR-author's stage.

## Verdict requested
Review acceptance (HITL gate 1): approve to advance, or reject with findings.
