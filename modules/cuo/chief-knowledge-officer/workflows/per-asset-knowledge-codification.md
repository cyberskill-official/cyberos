---
workflow_id: chief-knowledge-officer/per-asset-knowledge-codification
workflow_version: 1.0.0
purpose: Codify a single knowledge asset — outcomes, context, applicability, transferable patterns, anti-patterns, references.
persona: cuo/chief-knowledge-officer
cadence: per-event
status: shipped

inputs:
  - { name: source_engagement,     source: PMO engagement record, format: markdown brief }
  - { name: practitioner_interview, source: codification author/SME interview notes, format: markdown }
  - { name: artifacts,             source: actual deliverables produced (sanitized for confidentiality), format: directory tree }
  - { name: taxonomy,              source: current knowledge-taxonomy@1, format: knowledge-taxonomy@1 }

outputs:
  - { name: knowledge_asset,       format: knowledge-asset@1, recipient: cuo/chief-knowledge-officer + practice consumers + reuse-tracking system }

skill_chain:
  - { step: 1, skill: knowledge-asset-author, inputs_from: { source_engagement: source_engagement, practitioner_interview: practitioner_interview, artifacts: artifacts, taxonomy: taxonomy }, outputs_to: asset_draft }
  - { step: 2, skill: knowledge-asset-audit,  inputs_from: asset_draft, outputs_to: knowledge_asset }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "asset includes customer-confidential information requiring redaction approval" }

consults:
  - { persona: cuo/chief-marketing-officer,            when: "asset is publish-quality (case study / external thought leadership)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with knowledge_asset hash + taxonomy-tag + applicability scope
  - HITL pause at step 2 on QA-CONFIDENTIALITY-001 (un-sanitized confidential info) or QA-APPLICABILITY-001 (vague applicability)
---

# Per-asset knowledge codification — `chief-knowledge-officer/per-asset-knowledge-codification`

Chief Knowledge Officer's per-asset codification workflow. Per Nonaka SECI tacit→explicit conversion + Davenport Working Knowledge. Triggered at end of major engagement (per closure-author workflow output) or when a SME flags a transferable pattern.

## When to invoke

- "Codify the [engagement] knowledge"
- "Asset codification for [pattern]"
- "Document this engagement learning"

## How to invoke

```bash
cyberos-cuo run cuo/chief-knowledge-officer/per-asset-knowledge-codification \
  --input source_engagement=./engagements/2026-acme-platform/record.md \
  --input practitioner_interview=./knowledge/interviews/2026-acme.md \
  --input artifacts=./engagements/2026-acme-platform/deliverables/ \
  --input taxonomy=./knowledge/2026/taxonomy/knowledge-taxonomy.md \
  --output-dir ./knowledge/assets/2026/acme-platform-pattern/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 1-2 weeks for SME + legal review
- **Worst case:** confidentiality redaction may span 1 month

## Skill chain

- **Step 1 `knowledge-asset-author`** — drafts per Nonaka SECI + Davenport.
- **Step 2 `knowledge-asset-audit`** — validates per `knowledge_asset_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CONFIDENTIALITY-001 | Un-sanitized info | Escalate to CLO-Legal |
| 2 | QA-APPLICABILITY-001 | Vague scope | Operator tightens |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.7 — Chief Knowledge Officer role profile
- `./quarterly-knowledge-pipeline.md` — parent workflow
- `../../../skill/knowledge-asset-{author,audit}/SKILL.md`
