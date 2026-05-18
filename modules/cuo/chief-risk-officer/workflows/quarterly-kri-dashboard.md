---
workflow_id: chief-risk-officer/quarterly-kri-dashboard
workflow_version: 1.0.0
purpose: Refresh the quarterly Key Risk Indicator dashboard — KRI thresholds, breach analysis, escalations, trend.
persona: cuo/chief-risk-officer
cadence: quarterly
status: shipped

inputs:
  - { name: erm_framework,         source: cuo/chief-risk-officer/annual-erm-framework, format: enterprise-risk-framework@1 }
  - { name: prior_dashboard,       source: last quarter's kri-dashboard@1, format: key-risk-indicator-dashboard@1 }
  - { name: source_metrics,        source: per-KRI data feeds (security tooling, financial systems, ops, HR), format: csv }

outputs:
  - { name: kri_dashboard,         format: kri-dashboard@1, recipient: cuo/cro-risk + cuo/ceo + Board (quarterly risk chapter) }

skill_chain:
  - { step: 1, skill: key-risk-indicator-dashboard-author, inputs_from: { erm_framework: erm_framework, prior_dashboard: prior_dashboard, source_metrics: source_metrics }, outputs_to: dashboard_draft }
  - { step: 2, skill: key-risk-indicator-dashboard-audit,  inputs_from: dashboard_draft, outputs_to: kri_dashboard }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "any tier-1 KRI breaches threshold (e.g. cash runway, audit finding, material litigation)" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "financial KRI explanation needed" }
  - { persona: cuo/chief-information-security-officer,           when: "cyber KRI explanation needed" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with kri_dashboard hash + KRI count + breach count
  - HITL pause at step 2 on QA-THRESHOLD-001 (threshold missing) or QA-BREACH-001 (breach without narrative)
---

# Quarterly KRI dashboard — `chief-risk-officer/quarterly-kri-dashboard`

CRO-Risk's quarterly KRI dashboard refresh per RIMS RMM + COSO ERM. Each KRI tied to a risk class in the ERM framework. Threshold breaches trigger escalation; trends inform appetite recalibration.

## When to invoke

- "Refresh the Q<n> KRI dashboard"
- "Risk metrics dashboard"
- "Quarterly KRI review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-risk-officer/quarterly-kri-dashboard \
  --input erm_framework=./risk/2026/erm/framework.md \
  --input prior_dashboard=./risk/2026-Q1/kri.md \
  --input source_metrics=./risk/2026-Q1/metrics/ \
  --output-dir ./risk/2026-Q1/kri/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for owner round-trip
- **Worst case:** breach escalation may trigger same-quarter remediation

## Skill chain

- **Step 1 `key-risk-indicator-dashboard-author`** — drafts per RIMS + COSO ERM.
- **Step 2 `key-risk-indicator-dashboard-audit`** — validates per `kri_dashboard_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-THRESHOLD-001 | Threshold missing | Operator anchors to ERM appetite |
| 2 | QA-BREACH-001 | Breach no narrative | Operator drafts |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — CRO-Risk role profile
- `./annual-erm-framework.md` — upstream parent
- `../../../skill/kri-dashboard-{author,audit}/SKILL.md`
