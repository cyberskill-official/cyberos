---
memory_id: mem_019e1968-d80d-7bba-b98a-34ba31c42fb4
scope: memories/facts
classification: public
authority: human-confirmed
version: 1
created_at: 2026-05-11T23:15:49+07:00
created_by: subject:stephen-cheng
last_updated_at: 2026-05-11T23:15:49+07:00
updated_by: subject:stephen-cheng
provenance: {source: doc, source_ref: docs/CyberOS-PRD.docx §5.2, confidence: 1.0}
consent: {has_consent: true, consent_event: null, consent_scope: [fact]}
tags: [brain, architecture, three-layer, layer-1, layer-2, layer-3]
relationships: []
retention: {rule: indefinite, earliest_delete: null}
embedding: {model: null, version: null, vector_id: null}
sync_class: shared
source_freshness_tier: 10
---

# FACT-002 BRAIN has three layers

## Claim
BRAIN architecture: Layer 1 (filesystem .cyberos-memory/, today), Layer 2 (vector + graph, P0+), Layer 3 (archival corpus, P0+). Layer 1 is authoritative; 2-3 are derived.

## Source
PRD §5.2 The three layers.

## Evidence
- Layer 1 = working notebook (filesystem, Markdown+YAML)
- Layer 2 = personalisation memory (pgvector + AGE + PGroonga + bge-m3)
- Layer 3 = archival corpus (S3 cold tier, full history for compliance)

## Freshness
- Confidence: 1.0
