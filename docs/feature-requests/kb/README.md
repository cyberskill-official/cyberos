# KB module — feature request index

_Generated 2026-05-17 — 9 FRs, 49 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-KB-001](FR-KB-001-document-schema/spec.md) | MUST | 1 | 6 | KB Document schema — slug + markdown body + YAML frontmatter + closed category enum + 3-tier ACL + i |
| [FR-KB-002](FR-KB-002-server-side-renderer/spec.md) | MUST | 4 | 5 | KB server-side renderer — markdown → sanitised HTML (ammonia) + sanitised plaintext for memory ingest |
| [FR-KB-003](FR-KB-003-permission-tiers/spec.md) | MUST | 4 | 5 | KB 3 permission tiers — public / org-only / role-restricted with share-link tokens for time-bounded  |
| [FR-KB-004](FR-KB-004-fts5-pgroonga-lexical/spec.md) | MUST | 5 | 6 | KB FTS5 + PGroonga lexical search — VN bigram tokenisation + English stemming + per-tenant index wit |
| [FR-KB-005](FR-KB-005-bge-m3-semantic/spec.md) | MUST | 5 | 6 | KB BGE-M3 semantic search — memory Layer 2 vector ingest + dense embedding query with chunk-level ret |
| [FR-KB-006](FR-KB-006-bge-rerank/spec.md) | MUST | 5 | 4 | KB BGE-rerank-v2-m3 cross-encoder — reranks top-K results from FR-KB-004 lexical + FR-KB-005 semanti |
| [FR-KB-007](FR-KB-007-ask-this-page-qa/spec.md) | MUST | 5 | 8 | KB Ask-this-page Q&A — CUO-grounded answer over current + linked docs with span-level citations and  |
| [FR-KB-008](FR-KB-008-runbook-tags/spec.md) | MUST | 5 | 5 | KB runbook category — applicability tags (provider / region / severity) for OBS triage with FR-OBS-0 |
| [FR-KB-009](FR-KB-009-translation-of-link/spec.md) | SHOULD | 5 | 4 | KB dual-language `translation_of` link — vi/en pairing with locale-aware reader display and translat |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-KB-005→FR-AI-019, FR-KB-006→FR-AI-020
- **AUTH**: FR-KB-001→FR-AUTH-003, FR-KB-001→FR-AUTH-101
- **memory**: FR-KB-007→FR-MEMORY-108
- **CUO**: FR-KB-007→FR-CUO-101
- **OBS**: FR-KB-008→FR-OBS-007

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._