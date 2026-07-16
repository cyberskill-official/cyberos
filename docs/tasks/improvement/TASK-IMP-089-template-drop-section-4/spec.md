---
id: TASK-IMP-089
title: task@1 template drops the duplicate out-of-scope section 4
template: task@1
type: improvement
module: improvement
status: implementing
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T17:25:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-208]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 1
service: tools/install/templates
new_files: []
modified_files:
  - tools/install/templates/TASK-TEMPLATE.md
  - scripts/tests/test_template_schema.sh
source_pages:
  - "tools/install/templates/TASK-TEMPLATE.md sections 4 and 5 (the engineering-half duplicate of Scope > Out of scope)"
  - "operator decision 2026-07-16 (batch-2 PLAN gate, IMP-07): drop section 4; Scope > Out of scope is the single home"
  - "TASK-WEB-001 audit ISS-005 and every shipped spec since: the pointer-line idiom reconciling the two sections"
source_decisions:
  - "2026-07-16 Stephen: IMP-07 drop-section-4 chosen; batch 3 PLAN approved."
---

# TASK-IMP-089: task@1 template drops the duplicate out-of-scope section 4

## Summary

Every conforming spec has carried two out-of-scope homes - the PRD half's `## Scope > ### Out of scope / Non-Goals` (the rubric's home, SEC-006/QA-006) and the template's `## 4. Out of scope / non-goals` - reconciled by a pointer line in all nine specs shipped this run. Per the recorded IMP-07 decision, the template drops section 4 and renumbers Protected invariants from 5 to 4; the rubric needs no change because it never required section 4.

## Problem

Mandated duplication is authoring tax and drift risk: two homes for one statement invite divergence, and every audit this run logged the reconciliation as a wontfix finding.

## Proposed Solution

Remove the `## 4. Out of scope / non-goals` block from TASK-TEMPLATE.md, renumber `## 5. Protected invariants...` to `## 4.`, and update the template schema test to assert the new shape (single out-of-scope home, invariants at 4). Existing specs are untouched - the rubric accepts both shapes since section 4 was never a rule.

## Alternatives Considered

- Make section 4 optional with guidance. Rejected by the decision: an optional duplicate still invites divergence.
- Keep both and document the idiom. Rejected: nine specs of pointer lines is the evidence against it.

## Success Metrics

- Primary: a spec authored from the new template carries exactly one out-of-scope home and lints clean (schema test asserts the shape on every run). Baseline: two homes plus a pointer line in 9 of 9 specs this run. Deadline: final acceptance.
- Guardrail: the payload-vendored template matches source byte-for-byte after rebuild (existing template gates keep covering it).

## Scope

In scope: the template block removal, renumbering, schema-test update.

### Out of scope / Non-Goals

- Rewriting existing specs (both shapes stay valid to the rubric).
- The per-type body templates (feature/bug/chore carry the PRD half only - no section 4 exists there).

## Dependencies

- None; cone is the template file plus its schema test (disjoint from all batch members).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the recorded IMP-07 decision; implementation under ship-tasks supervision.
- **Human review:** batch-3 PLAN approved 2026-07-16; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 TASK-TEMPLATE.md MUST NOT contain a `## 4. Out of scope / non-goals` section; `## Scope > ### Out of scope / Non-Goals` remains the single home.
- 1.2 The Protected-invariants section MUST be renumbered to `## 4.` with its content unchanged.
- 1.3 test_template_schema.sh MUST assert the new shape: no duplicate out-of-scope heading in TASK-TEMPLATE.md, invariants present at section 4.
- 1.4 The rebuilt payload MUST carry the updated template (covered by the existing vendor gates; asserted once in the schema test against a scratch build).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - template shape: single out-of-scope home, invariants at 4 - test: `scripts/tests/test_template_schema.sh::t08_single_out_of_scope_home`
- [ ] AC 2 (traces_to: #1.3) - schema test fails on a reintroduced duplicate (fixture) - test: `scripts/tests/test_template_schema.sh::t08_duplicate_reintroduction_fails`
- [ ] AC 3 (traces_to: #1.4) - scratch payload carries the new shape - test: `scripts/tests/test_template_schema.sh::t08_payload_carries_shape`

## 3. Edge cases

- Existing specs with the old shape: untouched and still rubric-valid; the schema test targets the TEMPLATE, never the corpus (AC 1).
- Downstream prose citing "section 5 invariants": grep at implement time and update only template-adjacent references, not historical specs.
- Security-class: none - prompt-text template edit gated by tests.
