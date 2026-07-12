---
workflow_id: chief-people-officer/synonym-pointer-readme
workflow_version: 1.0.0
purpose: Pointer documentation — CPO-People is synonym of CHRO at firms that prefer "Chief People Officer" nomenclature. Use CHRO workflows.
persona: cuo/chief-people-officer
cadence: on-demand
status: shipped

inputs:
  - { name: routing_request,       source: CUO supervisor, format: markdown }

outputs:
  - { name: pointer_response,      format: decision-log@1, recipient: requester }

skill_chain:
  - { step: 1, skill: decision-log-author, inputs_from: { routing_request: routing_request }, outputs_to: pointer_draft }
  - { step: 2, skill: decision-log-audit,  inputs_from: pointer_draft, outputs_to: pointer_response }

audit_hooks:
  - workflow_complete row on PASS with pointer_response hash
---

# Synonym pointer — `chief-people-officer/synonym-pointer-readme`

**CPO-People (Chief People Officer) is a synonym of CHRO (Chief Human Resources Officer) at firms that prefer "People" nomenclature.** This workflow exists to document the routing decision: CPO-People requests should be served by CHRO workflows.

Per memory note: `chro` is default; `cpo-people` only when firm prefers CPO nomenclature. The two folders point at the same role profile.

## When to invoke

- When CUO routing surfaces CPO-People — respond with this pointer.
- For all actual HR work, route to `cuo/chief-human-resources-officer/workflows/`.

## Cross-references

- `../../../../modules/cuo/docs/module.md` §5.5 — both CHRO and CPO-People per the §2 acronym disambiguation
- `../../chro/README.md` — canonical implementation
- `../../chief-human-resources-officer/workflows/` — all 5 shipped workflows (annual-comp-cycle, quarterly-workforce-plan, new-hire-onboarding, quarterly-enps-pulse, quarterly-dei-program-review)
