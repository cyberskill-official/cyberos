---
workflow_id: chief-of-staff/quarterly-okr-cascade
workflow_version: 1.0.0
purpose: Govern the quarterly OKR cascade for the CEO — collect function drafts, ensure measurability, manage round-trips, publish the final set.
persona: cuo/chief-of-staff
cadence: quarterly
status: shipped

inputs:
  - { name: ceo_vision_brief,   source: cuo/ceo (verbal or written brief),                       format: markdown brief }
  - { name: prior_okrs,         source: last quarter's okr-set@1,                                format: objectives-and-key-results-set@1 }
  - { name: function_drafts,    source: each function head (CTO / CFO / CPO / CRO / etc.),       format: markdown (one per function) }

outputs:
  - { name: company_okrs,       format: okr-set@1, recipient: entire C-suite + all-hands }
  - { name: decision_log_entry, format: decision-log@1 entry, recipient: cuo/chief-of-staff + cuo/ceo (audit trail) }

skill_chain:
  - { step: 1, skill: objectives-and-key-results-set-author, inputs_from: { ceo_vision_brief: ceo_vision_brief, prior_okrs: prior_okrs, function_drafts: function_drafts }, outputs_to: okrs_draft }
  - { step: 2, skill: objectives-and-key-results-set-audit,  inputs_from: okrs_draft, outputs_to: company_okrs }
  - { step: 3, skill: decision-log-author, inputs_from: { cascade_summary: company_okrs }, outputs_to: log_entry_draft }
  - { step: 4, skill: decision-log-audit,  inputs_from: log_entry_draft, outputs_to: decision_log_entry }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "function drafts conflict at the company-OKR level; CEO arbitrates" }

consults:
  - { persona: cuo/chief-financial-officer,         when: "function OKRs imply >25% budget reallocation" }
  - { persona: cuo/chief-product-officer, when: "product OKRs need PRD alignment check" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with company_okrs hash + decision_log_entry hash
  - HITL pause at step 2 on QA-MEASURE-001 (KR not measurable) and step 4 on QA-OWNER-001 (decision lacks owner)
---

# Quarterly OKR cascade — `chief-of-staff/quarterly-okr-cascade`

CoS-owned governance for the quarterly OKR cascade. Pairs the `okr-set` skill (authoring) with the `decision-log` skill (audit trail) so every cascade is captured as a board-discoverable decision. Sister workflow to `chief-executive-officer/okr-cascade` — CoS runs the operational machine; CEO owns the strategic content.

## When to invoke

- "Run the Q<n> OKR cascade governance"
- "Collect function OKR drafts for Q<n>"
- "Publish the final OKR set with decision log"

## How to invoke

```bash
cyberos-cuo run cuo/chief-of-staff/quarterly-okr-cascade \
  --input ceo_vision_brief=./engagements/2026-Q2/vision.md \
  --input prior_okrs=./engagements/2026-Q1/okrs/final.md \
  --input function_drafts=./engagements/2026-Q2/function-drafts/ \
  --output-dir ./engagements/2026-Q2/okrs/
```

## Expected duration

- **Happy path:** 30-60 min runtime + 1-2 weeks for function-head round-trip
- **Worst case:** conflict-resolution may require live exec-staff meeting

## Skill chain

- **Step 1 `objectives-and-key-results-set-author`** — drafts the company OKR set with function cascades.
- **Step 2 `objectives-and-key-results-set-audit`** — validates per `okr_set_rubric@1.0`.
- **Step 3 `decision-log-author`** — records the cascade as a decision-log entry.
- **Step 4 `decision-log-audit`** — validates per `decision_log_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-MEASURE-001 | KR not measurable | Operator rewrites with metric |
| 2 | QA-CONFLICT-001 | Function OKRs conflict | Escalate to CEO |
| 4 | QA-OWNER-001 | Decision lacks named owner | Operator assigns |

## Cross-references
- `../README.md` §5 (Strategic) — "OKR-cascade governance"
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../chief-executive-officer/workflows/okr-cascade.md` — peer workflow CEO owns
- `../../../skill/{okr-set,decision-log}-{author,audit}/SKILL.md`
