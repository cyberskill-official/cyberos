---
workflow_id: chief-data-officer/quarterly-data-governance-review
workflow_version: 1.0.0
purpose: Review data governance state — quality scorecards, access reviews, lineage completeness, MDM hygiene, policy adherence.
persona: cuo/chief-data-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_governance,      source: last quarter's data-governance@1, format: data-governance@1 }
  - { name: quality_scorecards,    source: catalog tool data-quality metrics, format: csv }
  - { name: access_audit,          source: IAM / RBAC review per data product, format: csv }
  - { name: incident_log,          source: data-incident postmortem set, format: postmortem@1 (multiple) }

outputs:
  - { name: data_governance,       format: data-governance@1, recipient: cuo/cdo-data + cuo/cpo-privacy + cuo/cco-compliance }

skill_chain:
  - { step: 1, skill: data-governance-author, inputs_from: { prior_governance: prior_governance, quality_scorecards: quality_scorecards, access_audit: access_audit, incident_log: incident_log }, outputs_to: governance_draft }
  - { step: 2, skill: data-governance-audit,  inputs_from: governance_draft, outputs_to: data_governance }

escalates_to:
  - { persona: cuo/chief-privacy-officer,    when: "access audit surfaces over-broad permissions on personal-data products" }
  - { persona: cuo/chief-compliance-officer, when: "policy adherence < 90% trips compliance threshold" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "access controls intersect security boundaries" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with data_governance hash + quality-score-mean + access-violation-count
  - HITL pause at step 2 on QA-ACCESS-001 (over-broad access) or QA-INCIDENT-001 (incidents trend up)
---

# Quarterly data governance review — `chief-data-officer/quarterly-data-governance-review`

CDO-Data's quarterly governance review. Per DAMA-DMBOK governance chapter + Data Mesh federated-governance patterns + ISO/IEC 38505 data governance principles.

## When to invoke

- "Run the Q<n> data governance review"
- "Data quality + access audit"
- "Governance state check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-data-officer/quarterly-data-governance-review \
  --input prior_governance=./data/2026-Q1/governance.md \
  --input quality_scorecards=./data/2026-Q1/quality.csv \
  --input access_audit=./data/2026-Q1/access-audit.csv \
  --input incident_log=./data/2026-Q1/incidents/ \
  --output-dir ./data/2026-Q1/governance/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for owner round-trip
- **Worst case:** policy violations may trigger 1-month remediation program

## Skill chain

- **Step 1 `data-governance-author`** — drafts per DAMA-DMBOK + Data Mesh + ISO/IEC 38505.
- **Step 2 `data-governance-audit`** — validates per `data_governance_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ACCESS-001 | Over-broad access | Escalate to CPO-Privacy |
| 2 | QA-INCIDENT-001 | Incident trend up | Operator drives RCA |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3 — CDO-Data role profile
- `../../cpo-privacy/README.md` — privacy-governance peer
- `../../../skill/data-governance-{author,audit}/SKILL.md`
