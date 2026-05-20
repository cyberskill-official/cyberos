---
workflow_id: chief-of-staff/decision-log-keeping
workflow_version: 1.0.0
purpose: Capture a single decision as a structured log entry — context, options, decision, owner, due date, follow-ups.
persona: cuo/chief-of-staff
cadence: on-demand
status: shipped

inputs:
  - { name: decision_brief,     source: workflow-caller (or meeting-notes capture), format: markdown (free-form decision context) }
  - { name: meeting_context,    source: meeting notes or transcript,                format: markdown }

outputs:
  - { name: decision_entry,     format: decision-log@1, recipient: cuo/chief-of-staff + all decision stakeholders }

skill_chain:
  - { step: 1, skill: decision-log-author, inputs_from: { decision_brief: decision_brief, meeting_context: meeting_context }, outputs_to: entry_draft }
  - { step: 2, skill: decision-log-audit,  inputs_from: entry_draft, outputs_to: decision_entry }

escalates_to:
  - { persona: cuo/chief-executive-officer, when: "decision-log-audit fires QA-AUTHORITY-001 — decision exceeds documented authority of stated owner" }

consults:
  - { persona: cuo/chief-legal-officer, when: "decision has legal / contractual implications" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with decision_entry hash + decision-type tag
  - HITL pause at step 2 on QA-OWNER-001 (no owner) or QA-DUE-001 (no follow-up due date)
---

# Decision log keeping — `chief-of-staff/decision-log-keeping`

CoS-owned single-decision capture. Used after any meeting where a non-trivial decision is made. Audited for owner / due-date / options-considered so future audits can reconstruct WHY a decision was made. Per Amazon "6-page memo" + GitLab "transparent decision log" pattern.

## When to invoke

- "Log the decision from [meeting]"
- "Capture this decision"
- "Add to the decision log"

## How to invoke

```bash
cyberos-cuo run cuo/chief-of-staff/decision-log-keeping \
  --input decision_brief=./decisions/2026-05-17-vendor-X.md \
  --input meeting_context=./meetings/2026-05-17-vendor-review.md \
  --output-dir ./decisions/2026/
```

## Expected duration

- **Happy path:** 5-15 min runtime + same-day operator review
- **Worst case:** authority-escalation may require CEO 1:1

## Skill chain

- **Step 1 `decision-log-author`** — drafts per Amazon 6-pager template: context / options / decision / owner / due-date / follow-ups.
- **Step 2 `decision-log-audit`** — validates per `decision_log_rubric@1.0` (FM + SEC + QA-OWNER-001 + QA-DUE-001 + QA-AUTHORITY-001 + QA-OPTIONS-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-OWNER-001 | No owner | Operator assigns |
| 2 | QA-DUE-001 | No follow-up date | Operator sets due-date |
| 2 | QA-AUTHORITY-001 | Decision exceeds owner authority | Escalate to CEO |

## Cross-references
- `../README.md` §5 (Operational) — "decision log"
- `../../../../modules/cuo/README.md` §5.7
- `../../../skill/decision-log-{author,audit}/SKILL.md`
