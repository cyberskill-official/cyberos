---
workflow_id: chief-knowledge-officer/annual-knowledge-taxonomy
workflow_version: 1.0.0
purpose: Refresh the annual knowledge taxonomy — vocabulary, hierarchies, faceted classification, multi-language considerations.
persona: cuo/chief-knowledge-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_taxonomy,        source: last year's knowledge-taxonomy@1, format: knowledge-taxonomy@1 }
  - { name: usage_patterns,        source: search-log analytics, format: csv (top queries + zero-result queries) }
  - { name: corpus_inventory,      source: full asset inventory, format: csv }
  - { name: market_terminology,    source: industry term evolution sources (Gartner / Forrester), format: markdown }

outputs:
  - { name: knowledge_taxonomy,    format: knowledge-taxonomy@1, recipient: cuo/chief-knowledge-officer + cuo/cdo-data (master vocab alignment) + practice leads }

skill_chain:
  - { step: 1, skill: knowledge-taxonomy-author, inputs_from: { prior_taxonomy: prior_taxonomy, usage_patterns: usage_patterns, corpus_inventory: corpus_inventory, market_terminology: market_terminology }, outputs_to: taxonomy_draft }
  - { step: 2, skill: knowledge-taxonomy-audit,  inputs_from: taxonomy_draft, outputs_to: knowledge_taxonomy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "taxonomy changes warrant brand / positioning realignment" }

consults:
  - { persona: cuo/chief-data-officer,       when: "knowledge taxonomy intersects data master vocabulary" }
  - { persona: cuo/chief-marketing-officer,            when: "external terms need market-positioning alignment" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with knowledge_taxonomy hash + node count + multilang flag
  - HITL pause at step 2 on QA-ORPHAN-001 (assets unmappable to new taxonomy) or QA-DRIFT-001 (vocab drift from market)
---

# Annual knowledge taxonomy — `chief-knowledge-officer/annual-knowledge-taxonomy`

Chief Knowledge Officer's annual taxonomy refresh. Per ANSI Z39.19 controlled-vocabulary guidelines + ISO 25964 thesauri standards + SKOS (Simple Knowledge Organization System) for structured taxonomies. Multi-language consideration is essential for CyberSkill (VN + EN markets).

## When to invoke

- "Refresh the 2026 knowledge taxonomy"
- "Annual vocabulary review"
- "Taxonomy + thesaurus refresh"

## How to invoke

```bash
cyberos-cuo run cuo/chief-knowledge-officer/annual-knowledge-taxonomy \
  --input prior_taxonomy=./knowledge/2025/taxonomy.md \
  --input usage_patterns=./knowledge/2025/search-logs.csv \
  --input corpus_inventory=./knowledge/2026/inventory.csv \
  --input market_terminology=./market/2026/terms.md \
  --output-dir ./knowledge/2026/taxonomy/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-8 weeks for cross-practice input + remapping
- **Worst case:** wholesale rename + recategorization may span 1 quarter

## Skill chain

- **Step 1 `knowledge-taxonomy-author`** — drafts per ANSI Z39.19 + ISO 25964 + SKOS.
- **Step 2 `knowledge-taxonomy-audit`** — validates per `knowledge_taxonomy_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ORPHAN-001 | Unmappable assets | Operator remaps |
| 2 | QA-DRIFT-001 | Vocab drift from market | Operator extends synonyms |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Knowledge Officer role profile
- `./quarterly-knowledge-pipeline.md` — peer (taxonomy guides pipeline tagging)
- `../../../skill/knowledge-taxonomy-{author,audit}/SKILL.md`
