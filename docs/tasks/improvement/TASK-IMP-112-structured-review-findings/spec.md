---
id: TASK-IMP-112
title: Structured review findings alongside the prose packet
template: task@1
type: improvement
module: improvement
status: ready_to_review
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-084]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 4
service: modules/skill
new_files:
  - modules/skill/code-review-author/envelopes/review-findings.schema.json
  - tools/install/tests/test_review_findings.sh
modified_files:
  - modules/skill/code-review-author/SKILL.md
  - modules/skill/code-review-audit/SKILL.md
source_pages:
  - "IMPROVEMENT_HANDOFF.md §8 IMP-22"
  - "modules/skill/code-review-author/SKILL.md:44,56 (envelopes/input.json + output.json schema_refs - the contract anticipates structure the artefact does not emit)"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-112: Structured review findings alongside the prose packet

## Summary

`code-review.md` is a prose packet: excellent for a human, opaque to a machine. Nothing downstream can count findings, route them, or act on them. Emit `review-findings@1` (JSON) beside the markdown - one record per finding with file, line, severity, clause reference, and suggested fix - keeping the prose as the human artefact.

## Problem

`code-review-author/SKILL.md` declares `envelopes/input.json` and `envelopes/output.json` schema_refs at lines 44 and 56: the contract already anticipates structured output. The artefact does not emit it. The consequence is that "how often is the reviewer corrected" - the metric the whole improvement loop depends on - is unmeasurable, because findings are sentences.

## Proposed Solution

Emit `review-findings.json` beside `code-review.md`, one record per finding: `file`, `line`, `severity` (severe | important | nit), `clause_ref`, `summary`, `suggested_fix`. The markdown stays exactly as it is and remains the artefact a human reads. The JSON is what a future CI step, the reconcile ladder, or the outer loop reads.

Severity uses the three-value taxonomy rather than our High/Medium/Low prose: a nit and a severe finding are different kinds of thing, and a reviewer who cannot say "this is a nit" says nothing instead.

## Alternatives Considered

- Replace the markdown with JSON. Rejected: the prose packet is what makes a review readable, and a human reading JSON reviews worse.
- Parse the markdown into JSON after the fact. Rejected: parsing prose a model wrote is a guess about a guess.
- Wait until a consumer needs it. Rejected: the consumers are specified (IMP-20, IMP-23) and the field is free to emit while the reviewer already knows the answer.

## Success Metrics

- Primary: every review emits a schema-valid findings file whose record count matches the markdown's finding count - suite-asserted. Baseline: no machine-readable output exists.
- Guardrail: `code-review.md` is unchanged in shape - this task adds an artefact, it does not reformat the human one.

## Scope

In scope: the `review-findings@1` schema, its emission, the audit's check that the two agree, suite arms.

### Out of scope / Non-Goals

- Posting findings anywhere (no CI, no PR integration).
- Changing what the reviewer judges or how it decides severity.
- Reformatting the markdown packet.

## Dependencies

None.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §8 IMP-22; verified against code-review-author/SKILL.md on merged main.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `code-review-author` MUST emit `review-findings.json` beside `code-review.md`, valid against `review-findings@1`.
- 1.2 Each record MUST carry `file`, `line`, `severity` (`severe | important | nit`), `clause_ref`, `summary`, and `suggested_fix`.
- 1.3 `clause_ref` MUST name the task §1 clause the finding bears on, or `null` when the finding is outside the spec's clauses (a real category, and pretending otherwise would fabricate a reference).
- 1.4 The JSON record count MUST equal the markdown's finding count - the two artefacts describe one review.
- 1.5 `code-review.md`'s shape MUST NOT change.
- 1.6 A review with zero findings MUST emit an empty array, not an absent file.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - a review emits schema-valid JSON with all six fields per record - test: `tools/install/tests/test_review_findings.sh::t01_schema_valid`
- [ ] AC 2 (traces_to: #1.4) - record count equals the markdown finding count; a mismatch reds at audit - test: `tools/install/tests/test_review_findings.sh::t02_counts_agree`
- [ ] AC 3 (traces_to: #1.3) - an out-of-spec finding carries `clause_ref: null` rather than a fabricated reference - test: `tools/install/tests/test_review_findings.sh::t03_null_clause_ref`
- [ ] AC 4 (traces_to: #1.6) - a clean review emits `[]` - test: `tools/install/tests/test_review_findings.sh::t04_empty_array_not_absent`
- [ ] AC 5 (traces_to: #1.5) - the markdown packet is byte-identical to today's for a fixture review - test: `tools/install/tests/test_review_findings.sh::t05_markdown_unchanged`

## 3. Edge cases

- A finding spanning a range of lines: `line` records the first; the range belongs in `summary`. A schema that models ranges before anyone needs them is the gold-plating the guide warns about.
- A finding about a deleted file: `file` names the path as it was in the diff; the record is about the diff, not the tree.
- Severity disagreement between the JSON and the prose: the audit reds it (1.4's shape) - one review cannot say two things.
- A file path containing characters that break naive JSON: the emitter MUST serialise properly rather than string-concatenate. Test with a quote and a backslash in a path.
- Security-class: the reviewer writes a JSON file describing a diff. Nothing reads it as a command. Paths recorded are data, never interpolated.
