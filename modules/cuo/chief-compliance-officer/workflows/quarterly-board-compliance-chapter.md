---
workflow_id: chief-compliance-officer/quarterly-board-compliance-chapter
workflow_version: 1.0.0
purpose: Author the quarterly compliance chapter of the board deck — program health, testing results, regulatory exposure, escalations.
persona: cuo/chief-compliance-officer
cadence: quarterly
status: shipped

inputs:
  - { name: compliance_program,    source: cuo/chief-compliance-officer/annual-compliance-program, format: compliance-program@1 }
  - { name: control_testing,       source: cuo/chief-compliance-officer/quarterly-control-testing, format: compliance-program@1 (testing chapter) }
  - { name: filings_summary,       source: cuo/chief-compliance-officer/per-regulatory-filing set, format: regulatory-filing@1 (set) }
  - { name: incident_summary,      source: compliance incidents this quarter, format: markdown }

outputs:
  - { name: board_compliance_chapter, format: compliance-program@1 (board chapter), recipient: cuo/ceo (for inclusion in quarterly-board-update) + Board }

skill_chain:
  - { step: 1, skill: compliance-program-author, inputs_from: { compliance_program: compliance_program, control_testing: control_testing, filings_summary: filings_summary, incident_summary: incident_summary }, outputs_to: chapter_draft }
  - { step: 2, skill: compliance-program-audit,  inputs_from: chapter_draft, outputs_to: board_compliance_chapter }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "chapter surfaces material weakness or regulatory action requiring disclosure" }

consults:
  - { persona: cuo/chief-legal-officer,      when: "disclosure obligations" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with board_compliance_chapter hash + posture summary + escalation count
  - HITL pause at step 2 on QA-DISCLOSURE-001 (material weakness without disclosure plan)
---

# Quarterly board compliance chapter — `chief-compliance-officer/quarterly-board-compliance-chapter`

CCO-Compliance's contribution to the quarterly board deck. Combines program + testing + filings + incidents into compliance-chapter view. Feeds `chief-executive-officer/quarterly-board-update`.

## When to invoke

- "Write the compliance chapter for Q<n> board"
- "Board compliance update"
- "CCO-Compliance board contribution"

## How to invoke

```bash
cyberos-cuo run cuo/chief-compliance-officer/quarterly-board-compliance-chapter \
  --input compliance_program=./compliance/2026/program.md \
  --input control_testing=./compliance/2026-Q1/testing.md \
  --input filings_summary=./compliance/2026-Q1/filings/ \
  --input incident_summary=./compliance/2026-Q1/incidents.md \
  --output-dir ./board/2026-Q1/compliance-chapter/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 business day operator review
- **Worst case:** material weakness adds 1-2 weeks of legal coordination

## Skill chain

- **Step 1 `compliance-program-author`** — drafts board-chapter view.
- **Step 2 `compliance-program-audit`** — validates per `compliance_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-DISCLOSURE-001 | Material weakness no disclosure plan | Escalate to CEO + CLO-Legal |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — CCO-Compliance role profile
- `../../chief-executive-officer/workflows/quarterly-board-update.md` — board-deck consumer
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
