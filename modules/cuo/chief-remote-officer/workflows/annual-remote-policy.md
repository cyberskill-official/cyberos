---
workflow_id: chief-remote-officer/annual-remote-policy
workflow_version: 1.0.0
purpose: Author the annual remote-work policy — eligibility, locations/jurisdictions, equipment, communication norms, performance management.
persona: cuo/chief-remote-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_policy,          source: last year's remote-policy@1, format: remote-policy@1 }
  - { name: workforce_distribution, source: HRIS geography data, format: csv }
  - { name: employee_pulse,        source: cuo/chief-human-resources-officer/quarterly-enps-pulse (remote slices), format: enps-program@1 }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: remote_policy,         format: remote-policy@1, recipient: cuo/chief-remote-officer + cuo/chro + cuo/clo-legal + all employees }

skill_chain:
  - { step: 1, skill: remote-policy-author, inputs_from: { prior_policy: prior_policy, workforce_distribution: workforce_distribution, employee_pulse: employee_pulse, ceo_priorities: ceo_priorities }, outputs_to: policy_draft }
  - { step: 2, skill: remote-policy-audit,  inputs_from: policy_draft, outputs_to: remote_policy }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "new-jurisdiction employment law triggers required policy adjustment" }
  - { persona: cuo/chief-privacy-officer,    when: "cross-border data-handling implications" }

audit_hooks:
  - workflow_complete row on PASS with remote_policy hash + jurisdictions count
  - HITL pause at step 2 on QA-JURISDICTION-001 (employee jurisdiction not covered)
---

# Annual remote policy — `chief-remote-officer/annual-remote-policy`

Chief Remote Officer's annual remote-work policy per GitLab Remote Manifesto + Buffer State-of-Remote + Atlassian Distributed-Work Playbook.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7 — Chief Remote Officer role profile
- `../../chief-human-resources-officer/workflows/quarterly-enps-pulse.md` — peer (CHRO often absorbs CRO post-2022)
- `../../../skill/remote-policy-{author,audit}/SKILL.md`
