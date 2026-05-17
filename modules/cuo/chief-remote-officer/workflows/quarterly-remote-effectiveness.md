---
workflow_id: chief-remote-officer/quarterly-remote-effectiveness
workflow_version: 1.0.0
purpose: Review remote-team effectiveness — async/sync mix, meeting load, doc-as-default adoption, collaboration health.
persona: cuo/chief-remote-officer
cadence: quarterly
status: shipped

inputs:
  - { name: collaboration_telemetry, source: Slack/Zoom/Notion/etc. telemetry, format: csv }
  - { name: prior_review,          source: last quarter's review, format: remote-policy@1 (effectiveness chapter) }
  - { name: employee_pulse,        source: cuo/chief-human-resources-officer/quarterly-enps-pulse remote slice, format: enps-program@1 }

outputs:
  - { name: remote_effectiveness,  format: remote-policy@1 (quarterly chapter), recipient: cuo/chief-remote-officer + cuo/chro + cuo/coo }

skill_chain:
  - { step: 1, skill: remote-policy-author, inputs_from: { collaboration_telemetry: collaboration_telemetry, prior_review: prior_review, employee_pulse: employee_pulse }, outputs_to: review_draft }
  - { step: 2, skill: remote-policy-audit,  inputs_from: review_draft, outputs_to: remote_effectiveness }

escalates_to:
  - { persona: cuo/chief-human-resources-officer,           when: "meeting-load > 50% calendar OR async-default adoption < 60%" }

audit_hooks:
  - workflow_complete row on PASS with remote_effectiveness hash
  - HITL pause at step 2 on QA-METRIC-001
---

# Quarterly remote effectiveness — `chief-remote-officer/quarterly-remote-effectiveness`

Chief Remote Officer's quarterly effectiveness review per GitLab handbook + Doist Async-First framework + Atlassian Team Health Monitor.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7
- `./annual-remote-policy.md` — upstream parent
- `../../../skill/remote-policy-{author,audit}/SKILL.md`
