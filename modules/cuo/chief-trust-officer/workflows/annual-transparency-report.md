---
workflow_id: chief-trust-officer/annual-transparency-report
workflow_version: 1.0.0
purpose: Author the annual transparency report — government-data-requests, content-moderation actions, model decisions, abuse-handling metrics.
persona: cuo/chief-trust-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_report,          source: last year's transparency-report@1, format: transparency-report@1 }
  - { name: govt_requests,         source: cuo/clo-legal (government-data-request log), format: csv }
  - { name: moderation_log,        source: trust + safety team, format: csv }
  - { name: abuse_metrics,         source: trust + safety platform, format: csv }

outputs:
  - { name: transparency_report,   format: transparency-report@1, recipient: cuo/chief-trust-officer + public + cuo/clo-legal + Board (annual transparency chapter) }

skill_chain:
  - { step: 1, skill: transparency-report-author, inputs_from: { prior_report: prior_report, govt_requests: govt_requests, moderation_log: moderation_log, abuse_metrics: abuse_metrics }, outputs_to: report_draft }
  - { step: 2, skill: transparency-report-audit,  inputs_from: report_draft, outputs_to: transparency_report }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "report includes data subject to gag orders or national-security letters" }

consults:
  - { persona: cuo/chief-communications-officer, when: "report launch needs coordinated comms" }
  - { persona: cuo/chief-ethics-officer, when: "model-decision section needs ethics sign-off" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with transparency_report hash + govt-request count + moderation-action count
  - HITL pause at step 2 on QA-GAG-001 (gag-ordered data included) or QA-METHODOLOGY-001 (metric methodology not documented)
---

# Annual transparency report — `chief-trust-officer/annual-transparency-report`

Chief Trust Officer's annual transparency report per Santa Clara Principles + EFF transparency-report standards + Lumen Database conventions. Public-facing accountability document.

## When to invoke

- "Build the 2026 transparency report"
- "Annual transparency disclosure"
- "Trust + safety annual report"

## How to invoke

```bash
cyberos-cuo run cuo/chief-trust-officer/annual-transparency-report \
  --input prior_report=./trust/2025/transparency.md \
  --input govt_requests=./legal/2025/govt-requests.csv \
  --input moderation_log=./trust/2025/moderation.csv \
  --input abuse_metrics=./trust/2025/abuse.csv \
  --output-dir ./trust/2026/transparency/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 4-8 weeks for legal review + public launch
- **Worst case:** gag-order conflict requires removal + legal coordination

## Skill chain

- **Step 1 `transparency-report-author`** — drafts per Santa Clara Principles + EFF + Lumen.
- **Step 2 `transparency-report-audit`** — validates per `transparency_report_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-GAG-001 | Gag-ordered data included | Escalate to CLO-Legal |
| 2 | QA-METHODOLOGY-001 | Methodology missing | Operator documents |

## Cross-references
- `../../../../modules/cuo/README.md` §5.6 — Chief Trust Officer role profile
- `./quarterly-trust-portal-update.md` — peer
- `../../../skill/transparency-report-{author,audit}/SKILL.md`
