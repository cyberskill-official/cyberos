# KB module — task index

_Generated 2026-05-17 — 9 FRs, 49 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-KB-001](TASK-KB-001-document-schema/spec.md) | MUST | 1 | 6 | KB Document schema — slug + markdown body + YAML frontmatter + closed category enum + 3-tier ACL + i |
| [TASK-KB-002](TASK-KB-002-server-side-renderer/spec.md) | MUST | 4 | 5 | KB server-side renderer — markdown → sanitised HTML (ammonia) + sanitised plaintext for memory ingest |
| [TASK-KB-003](TASK-KB-003-permission-tiers/spec.md) | MUST | 4 | 5 | KB 3 permission tiers — public / org-only / role-restricted with share-link tokens for time-bounded  |
| [TASK-KB-004](TASK-KB-004-fts5-pgroonga-lexical/spec.md) | MUST | 5 | 6 | KB FTS5 + PGroonga lexical search — VN bigram tokenisation + English stemming + per-tenant index wit |
| [TASK-KB-005](TASK-KB-005-bge-m3-semantic/spec.md) | MUST | 5 | 6 | KB BGE-M3 semantic search — memory Layer 2 vector ingest + dense embedding query with chunk-level ret |
| [TASK-KB-006](TASK-KB-006-bge-rerank/spec.md) | MUST | 5 | 4 | KB BGE-rerank-v2-m3 cross-encoder — reranks top-K results from TASK-KB-004 lexical + TASK-KB-005 semanti |
| [TASK-KB-007](TASK-KB-007-ask-this-page-qa/spec.md) | MUST | 5 | 8 | KB Ask-this-page Q&A — CUO-grounded answer over current + linked docs with span-level citations and  |
| [TASK-KB-008](TASK-KB-008-runbook-tags/spec.md) | MUST | 5 | 5 | KB runbook category — applicability tags (provider / region / severity) for OBS triage with FR-OBS-0 |
| [TASK-KB-009](TASK-KB-009-translation-of-link/spec.md) | SHOULD | 5 | 4 | KB dual-language `translation_of` link — vi/en pairing with locale-aware reader display and translat |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-KB-005→TASK-AI-019, TASK-KB-006→TASK-AI-020
- **AUTH**: TASK-KB-001→TASK-AUTH-003, TASK-KB-001→TASK-AUTH-101
- **memory**: TASK-KB-007→TASK-MEMORY-108
- **CUO**: TASK-KB-007→TASK-CUO-101
- **OBS**: TASK-KB-008→TASK-OBS-007

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._