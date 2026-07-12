---
workflow_id: chief-knowledge-officer/quarterly-knowledge-pipeline
workflow_version: 1.0.0
purpose: Run the quarterly knowledge-asset codification pipeline — harvest candidates, manage queue, publish, measure reuse.
persona: cuo/chief-knowledge-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_pipeline,        source: last quarter's knowledge-pipeline@1, format: knowledge-pipeline@1 }
  - { name: engagement_register,   source: PMO engagement list, format: csv }
  - { name: asset_inventory,       source: knowledge repository (Notion / Confluence / Guru / internal), format: csv }
  - { name: usage_metrics,         source: KB platform analytics, format: csv }

outputs:
  - { name: knowledge_pipeline,    format: knowledge-pipeline@1, recipient: cuo/chief-knowledge-officer + cuo/coo + cuo/cco-customer + practice leads }

skill_chain:
  - { step: 1, skill: knowledge-pipeline-author, inputs_from: { prior_pipeline: prior_pipeline, engagement_register: engagement_register, asset_inventory: asset_inventory, usage_metrics: usage_metrics }, outputs_to: pipeline_draft }
  - { step: 2, skill: knowledge-pipeline-audit,  inputs_from: pipeline_draft, outputs_to: knowledge_pipeline }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "codification throughput drops > 30% QoQ — IP-moat risk" }
  - { persona: cuo/chief-operating-officer,            when: "engagement-end → codification time exceeds 6 weeks consistently" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "engineering-asset codification needs tooling integration (e.g. snippet libraries)" }
  - { persona: cuo/chief-customer-officer,   when: "case-study assets need customer approval" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with knowledge_pipeline hash + asset count by stage + reuse-rate
  - HITL pause at step 2 on QA-TAXONOMY-001 (asset uncategorized) or QA-REUSE-001 (reuse-metric missing for published assets)
---

# Quarterly knowledge pipeline — `chief-knowledge-officer/quarterly-knowledge-pipeline`

Chief Knowledge Officer's quarterly codification pipeline. Per Nonaka SECI + Wenger CoP + Davenport + ANSI Z39.19 + McKinsey KM. Critical for consulting firms (CyberSkill commercial baseline §7 high-ROI) where IP/asset codification IS the moat.

## When to invoke

- "Run the Q<n> knowledge pipeline"
- "Codification review"
- "Knowledge-asset queue refresh"

## How to invoke

```bash
cyberos-cuo run cuo/chief-knowledge-officer/quarterly-knowledge-pipeline \
  --input prior_pipeline=./knowledge/2026-Q1/pipeline.md \
  --input engagement_register=./pmo/2026-Q1/engagements.csv \
  --input asset_inventory=./knowledge/inventory.csv \
  --input usage_metrics=./knowledge/2026-Q1/usage.csv \
  --output-dir ./knowledge/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for practice-lead validation
- **Worst case:** taxonomy drift triggers 1-quarter cleanup project

## Skill chain

- **Step 1 `knowledge-pipeline-author`** — drafts per Nonaka + Davenport + ANSI Z39.19 + McKinsey KM.
- **Step 2 `knowledge-pipeline-audit`** — validates per `knowledge_pipeline_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-TAXONOMY-001 | Uncategorized asset | Operator tags |
| 2 | QA-REUSE-001 | Reuse-metric missing | Operator instruments |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Knowledge Officer role profile
- `./annual-knowledge-taxonomy.md` — peer (taxonomy refresh)
- `../../../skill/knowledge-pipeline-{author,audit}/SKILL.md`
