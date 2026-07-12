---
workflow_id: chief-transformation-officer/per-program-change-management
workflow_version: 1.0.0
purpose: Author the change-management plan for a transformation program — ADKAR awareness/desire/knowledge/ability/reinforcement.
persona: cuo/chief-transformation-officer
cadence: per-event
status: shipped

inputs:
  - { name: program_charter,       source: cuo/chief-transformation-officer/per-program-charter, format: program-charter@1 }
  - { name: stakeholder_map,       source: change-impact analysis, format: markdown }
  - { name: prior_change_plans,    source: similar prior change-mgmt-plan@1, format: change-management-plan@1 (multiple) }

outputs:
  - { name: change_mgmt_plan,      format: change-mgmt-plan@1, recipient: cuo/chief-transformation-officer + program owner + cuo/chro + cuo/cco-communications }

skill_chain:
  - { step: 1, skill: change-management-plan-author, inputs_from: { program_charter: program_charter, stakeholder_map: stakeholder_map, prior_change_plans: prior_change_plans }, outputs_to: plan_draft }
  - { step: 2, skill: change-management-plan-audit,  inputs_from: plan_draft, outputs_to: change_mgmt_plan }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "change-impact analysis surfaces risk of >10% attrition in affected functions" }

consults:
  - { persona: cuo/chief-human-resources-officer,           when: "training + support delivery framework needed" }
  - { persona: cuo/chief-communications-officer, when: "internal-comms campaign needed" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with change_mgmt_plan hash + stakeholder count + ADKAR coverage
  - HITL pause at step 2 on QA-ADKAR-001 (ADKAR phase missing) or QA-STAKEHOLDER-001 (affected stakeholder missed)
---

# Per program change management — `chief-transformation-officer/per-program-change-management`

Chief Transformation Officer's per-program change-management plan per Prosci ADKAR (Awareness / Desire / Knowledge / Ability / Reinforcement) + Kotter 8-step + Bridges Transition Model. Required for every transformation program with stakeholder impact >50 people OR multi-function reorg.

## When to invoke

- "Build change-mgmt plan for [program]"
- "Change management for [transformation]"
- "Stakeholder change plan"

## How to invoke

```bash
cyberos-cuo run cuo/chief-transformation-officer/per-program-change-management \
  --input program_charter=./transformation/programs/2026-erp-rollout/charter.md \
  --input stakeholder_map=./transformation/programs/2026-erp-rollout/stakeholders.md \
  --input prior_change_plans=./transformation/prior/ \
  --output-dir ./transformation/programs/2026-erp-rollout/change-plan/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2-4 weeks for stakeholder consultation
- **Worst case:** high-attrition risk triggers CEO review + plan modification

## Skill chain

- **Step 1 `change-management-plan-author`** — drafts per Prosci ADKAR + Kotter + Bridges.
- **Step 2 `change-management-plan-audit`** — validates per `change_mgmt_plan_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ADKAR-001 | ADKAR phase missing | Operator extends |
| 2 | QA-STAKEHOLDER-001 | Stakeholder missed | Operator extends |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Transformation Officer role profile
- `./per-program-charter.md` — upstream feeder
- `../../../skill/change-mgmt-plan-{author,audit}/SKILL.md`
