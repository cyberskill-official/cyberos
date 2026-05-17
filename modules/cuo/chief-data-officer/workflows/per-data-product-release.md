---
workflow_id: chief-data-officer/per-data-product-release
workflow_version: 1.0.0
purpose: Release a new (or updated) data product — schema, SLA, lineage, consumer onboarding, deprecation policy.
persona: cuo/chief-data-officer
cadence: per-event
status: shipped

inputs:
  - { name: product_brief,         source: data-product owner, format: markdown }
  - { name: schema_draft,          source: engineering team, format: schema definitions (JSON/Avro/Proto) }
  - { name: lineage_map,           source: catalog tool (DataHub / Atlan / Collibra), format: markdown / json }
  - { name: consumer_list,         source: data-platform team, format: csv (downstream consumers + intended use) }

outputs:
  - { name: data_product,          format: data-product@1, recipient: cuo/cdo-data + data product consumers + cuo/cpo-privacy (governance) }

skill_chain:
  - { step: 1, skill: data-product-author, inputs_from: { product_brief: product_brief, schema_draft: schema_draft, lineage_map: lineage_map, consumer_list: consumer_list }, outputs_to: product_draft }
  - { step: 2, skill: data-product-audit,  inputs_from: product_draft, outputs_to: data_product }

escalates_to:
  - { persona: cuo/chief-privacy-officer,    when: "data product contains personal data not previously approved (DPIA / PIA needed)" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "schema changes propagate to upstream services" }
  - { persona: cuo/chief-ai-officer,           when: "data product feeds ML model training (need versioning lineage)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with data_product hash + schema-version + consumer-count + SLA targets
  - HITL pause at step 2 on QA-SLA-001 (SLA not specified) or QA-LINEAGE-001 (lineage incomplete)
---

# Per-data-product release — `chief-data-officer/per-data-product-release`

CDO-Data's per-data-product release workflow. Per Data Mesh + Open Data Product Specification. Each release includes schema + SLA + lineage + consumer onboarding + deprecation policy. Triggered per major release (semver-major) or new product launch.

## When to invoke

- "Release the [data product] update"
- "Publish the [dataset] as data product"
- "Data product release workflow"

## How to invoke

```bash
cyberos-cuo run cuo/chief-data-officer/per-data-product-release \
  --input product_brief=./data/products/2026-customer-360/brief.md \
  --input schema_draft=./data/products/2026-customer-360/schema.json \
  --input lineage_map=./data/products/2026-customer-360/lineage.md \
  --input consumer_list=./data/products/2026-customer-360/consumers.csv \
  --output-dir ./data/products/2026-customer-360/release/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 1-2 weeks for cross-team validation
- **Worst case:** privacy escalation may add 1 month

## Skill chain

- **Step 1 `data-product-author`** — drafts per Open Data Product Specification + Data Mesh patterns.
- **Step 2 `data-product-audit`** — validates per `data_product_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-SLA-001 | SLA not specified | Operator defines |
| 2 | QA-LINEAGE-001 | Lineage incomplete | Operator extends |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.3 — CDO-Data role profile
- `../../chief-privacy-officer/workflows/privacy-impact-assessment.md` — peer (PIA for personal-data products)
- `../../../skill/data-product-{author,audit}/SKILL.md`
