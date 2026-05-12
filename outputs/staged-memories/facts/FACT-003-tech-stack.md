---
memory_id: mem_019e1968-d80b-7877-97ca-db659e5b3e1f
scope: memories/facts
classification: public
authority: human-confirmed
version: 1
created_at: 2026-05-11T23:15:49+07:00
created_by: subject:stephen-cheng
last_updated_at: 2026-05-11T23:15:49+07:00
updated_by: subject:stephen-cheng
provenance: {source: doc, source_ref: docs/CyberOS-PRD.docx §1.1, confidence: 1.0}
consent: {has_consent: true, consent_event: null, consent_scope: [fact]}
tags: [tech-stack, apollo-federation, mcp, postgres, pgvector, module-federation]
relationships: []
retention: {rule: indefinite, earliest_delete: null}
embedding: {model: null, version: null, vector_id: null}
sync_class: shared
source_freshness_tier: 10
---

# FACT-003 CyberOS tech stack

## Claim
Apollo Federation v2 (subgraphs per module), Module Federation (frontend), MCP 2025-11-25 spec (agent ops), PostgreSQL 17 + pgvector (HNSW) + Apache AGE (graph) + PGroonga (multilingual FTS), bge-m3 embeddings, bge-reranker-v2-m3 reranker.

## Source
PRD §1.1 + §5.4.

## Evidence
"Built on Apollo Federation v2 (subgraphs per module), Module Federation for the frontend, the Model Context Protocol 2025-11-25 spec for agent operability, PostgreSQL 17 with pgvector ..."

## Freshness
- Confidence: 1.0
