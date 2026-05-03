---
title: "BRAIN Layer 2 — vector + graph fact memory with hybrid retrieval, GraphRAG community summaries, ingestion denylist"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: limited
target_release: "P0 / 2026-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship Layer 2 of BRAIN: a fact-level memory built on PostgreSQL with `pgvector` for embeddings, Apache AGE for the graph, and PGroonga for Vietnamese-aware lexical search. Layer 2 ingests every Layer 1 file (FR-BRAIN-001) and every event from connected modules (CHAT messages, KB pages, decisions ledger entries, meeting transcripts) through an extractor that produces atomic *facts*, runs a **hybrid retrieval pipeline** (lexical BM25 via PGroonga + vector via pgvector + graph traversal via AGE + cross-encoder reranking via `bge-reranker-v2-m3`), maintains **GraphRAG community summaries** via Microsoft GraphRAG-style hierarchical clustering, and supports **four operations** on facts (ADD, UPDATE, DELETE, NOOP — Mem0-style). The ingestion path enforces a hard denylist (compensation values, equity values, government IDs, bank accounts, special-category health data) so those classes of data never enter Layer 2 even if they pass through Layer 1 by mistake. Citations resolve to the originating Layer 1 file + line range. Every retrieval is auditable.

## Problem

Layer 1 is human-readable but not query-friendly: a question like "what's the typical hold-up at proposal stage across recent deals?" requires aggregating signal across dozens of files. Layer 2 is the substrate that turns Layer 1 into a queryable graph + vector index without losing the citation back to Layer 1.

The PRD's "no answer without a citation" property (PRD §4.3 anti-metric) demands that every retrieval returns the originating sources, that those sources be human-inspectable in Layer 1, and that the retrieval pipeline never hallucinate a citation. The Mem0 + GraphRAG + Letta comparative analysis (PRD §5.1) settled on a hybrid retrieval pipeline and the four-operation contract (ADD / UPDATE / DELETE / NOOP) as the productive shape.

The denylist is a hard requirement: PDPL Decree 13 plus the company's social-contract (P1-protection invariant: "evaluation never reduces base salary in cash") plus EU AI Act high-risk obligations on compensation modules (in P2+) require that compensation values, equity values, government IDs, and bank accounts never traffic through a vector index that an LLM can retrieve from. The denylist is enforced at *ingestion* — bytes never land in `pgvector` to begin with.

S0-3 sprint exit (PRD §17.3) requires the demo: "What is Acme's payment cycle?" returns the answer with citation; BRAIN search p95 ≤ 600 ms.

## Proposed Solution

The shape of the answer is the BRAIN module's Layer 2 subgraph + MCP server + ingestion pipeline + nightly community-summary job. It runs on the same Postgres cluster as everything else.

**Schema.**

```sql
CREATE SCHEMA brain;

CREATE TABLE brain.fact (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  layer1_file_id  UUID,                    -- FK to brain.layer1_file (FR-BRAIN-001); null if from a non-Layer-1 source
  source_kind     TEXT NOT NULL,           -- "layer1", "chat", "kb", "meeting", "decision", "email_summary"
  source_ref      TEXT NOT NULL,           -- canonical pointer back to the source (e.g. layer1 path + line range)
  text            TEXT NOT NULL,           -- the atomic fact in natural language
  text_embed_m3   vector(1024),            -- bge-m3 embedding (1024-dim)
  text_pgrn       TSVECTOR_TYPE NOT NULL,  -- PGroonga-indexed Vietnamese-aware tokens
  subject_uri     TEXT,                    -- entity URI (e.g. "client:acme-corp")
  predicate       TEXT,                    -- relation (e.g. "has_payment_cycle")
  object_uri      TEXT,                    -- entity URI for the object, or NULL
  object_literal  TEXT,                    -- literal value when not an entity reference
  confidence      REAL,                    -- 0..1 from the extractor
  provenance      JSONB,                   -- extractor name + version + author + timestamp + raw span
  status          TEXT NOT NULL DEFAULT 'active',  -- 'active', 'superseded', 'disputed'
  superseded_by   UUID REFERENCES brain.fact(id),
  disputed_with   UUID[],
  community_id    UUID,                    -- Leiden community membership, populated nightly
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX fact_text_embed_m3_idx ON brain.fact USING hnsw (text_embed_m3 vector_cosine_ops);
CREATE INDEX fact_text_pgrn_idx     ON brain.fact USING pgroonga (text);
CREATE INDEX fact_tenant_subject_idx ON brain.fact (tenant_id, subject_uri, status);
CREATE INDEX fact_tenant_community_idx ON brain.fact (tenant_id, community_id, status);
```

The graph layer uses Apache AGE: `subject_uri --[predicate]--> object_uri` is materialised as a Cypher edge in the `brain_graph` graph; nodes carry their fact IDs as properties; the AGE graph is traversable from Cypher and joinable to relational tables.

**Ingestion pipeline.**

1. **Source listener.** A NATS consumer subscribes to `cyberos.{tenant}.brain.l1.>`, `cyberos.{tenant}.chat.message.created`, `cyberos.{tenant}.kb.page.published`, `cyberos.{tenant}.proj.task.commented`, etc. Each event triggers an ingestion job.
2. **Denylist filter.** The raw event content is passed through the denylist regex set:
   - `VN_CCCD` (12-digit citizen IDs)
   - `VN_TAX_CODE`, `VN_BANK_ACCOUNT`, `VN_SI_NUMBER`
   - Compensation/equity numeric patterns when adjacent to currency or share-count tokens
   - Health/medical condition keywords (PDPL special-category data)
   - Cryptographic key material (`-----BEGIN PRIVATE KEY-----`, JWTs)
   If a match is found, the event is **not ingested**; an audit row is written in scope `brain.denylist.{tenant}` recording the source, the masked match, and the chosen action (`drop` or `redact-and-ingest`). The default action is `drop`. An exception list lets specific source kinds (the rare KB page about a public reference rate) opt into `redact-and-ingest`.
3. **Fact extractor.** Sanitised content is passed to the AI Gateway (FR-AI-001) with a prompt that returns a list of structured facts in JSON. The extractor uses Haiku 4.5 for short content (CHAT messages, KB sections) and Sonnet 4.6 for long content (meeting transcripts, decision rationales). Each returned fact is `{ subject_uri, predicate, object_uri | object_literal, text, confidence, raw_span }`. The extractor is prompt-engineered to refuse to invent subjects or predicates not present in the input.
4. **Operation classifier (Mem0 four-operation pattern).** For each candidate fact, the classifier decides:
   - `ADD` — a new fact; insert.
   - `UPDATE` — supersedes an existing fact (matched by subject + predicate); the old fact is marked `superseded_by` the new fact's ID; the old fact remains for audit but is excluded from default retrieval.
   - `DELETE` — explicit forgetting (triggered by the natural-language CRUD path "forget that"); the fact is marked `status: 'archived'` and excluded from retrieval; the underlying Layer 1 file is updated by FR-BRAIN-001.
   - `NOOP` — duplicate or trivial restatement; not inserted; an audit row records the dedup.
5. **Embedding + indexing.** ADD/UPDATE writes embed the fact's `text` via `bge-m3` (1024-dim) through the AI Gateway's embedding endpoint and write to `brain.fact.text_embed_m3`. PGroonga indexes the text for Vietnamese-aware lexical search. The AGE graph edge is created or updated.
6. **Conflict detection.** If a new fact conflicts with an existing fact (same subject + same predicate, different object, both `active`), it is marked `status: 'disputed'` and added to the existing fact's `disputed_with` array. The conflict-resolution UI (PRD §5.6, FR-BRAIN-CONFLICT-001 in batch-02) surfaces both for the human to choose.

**Hybrid retrieval pipeline.** Given a query string and a tenant context, the retrieval pipeline:

1. **Query-time embedding.** Embed the query via `bge-m3`.
2. **Three parallel candidate fetches.**
   - **Vector**: top-50 facts by cosine similarity in `pgvector`.
   - **Lexical (PGroonga)**: top-50 facts by BM25, with Vietnamese tokenisation.
   - **Graph**: starting from the entities mentioned in the query (resolved by a small entity-linker), walk the AGE graph to depth 2 and collect facts attached to traversed nodes; cap at 50.
3. **Union and dedupe** by fact ID.
4. **Cross-encoder rerank** via `bge-reranker-v2-m3` through the AI Gateway. Top-K (default K=8, configurable per consumer) facts returned with their scores.
5. **Citation hydration.** For each returned fact, include the canonical citation (Layer 1 path + line range, or non-Layer-1 source ref) so the consumer can cite back.
6. **Community context.** If the query straddles multiple entities, the GraphRAG community summary for the relevant communities is included as auxiliary context.

The pipeline runs entirely in-cluster; no external service call beyond the AI Gateway's embed/rerank endpoints. Latency target: p95 ≤ 600 ms end-to-end at S0-3 demo (PRD §17.3 risk gate).

**GraphRAG community summaries.** Nightly at 03:00 ICT (the Auto Dream consolidation job from FR-BRAIN-001), a job runs Leiden community detection over the AGE graph, assigns each node a `community_id`, and asks the AI Gateway (Sonnet 4.6) to summarise each community's facts into a short narrative ("Acme Corp engagement summary: long-term retainer since 2024-09; primary contact Jane Doe (now CTO); current sprint 14 in delivery; payment cycle 90 days; no open risks."). The summary is stored in `brain.community_summary` with the same provenance + version pattern as facts. Retrieval pipelines include the summary for the highest-scoring community when the query is broad.

**Four operations exposed via MCP.**

- `cyberos.brain.search(query, top_k?, kind_filter?)` — read-only; returns facts with citations.
- `cyberos.brain.add_fact(subject, predicate, object, text)` — `destructive: false; idempotent: true` (the operation classifier may choose NOOP).
- `cyberos.brain.update_fact(fact_id, new_text, reason)` — `destructive: true; requires_confirmation: true`.
- `cyberos.brain.delete_fact(fact_id, reason)` — `destructive: true; requires_confirmation: true`.
- `cyberos.brain.get_community_summary(entity_uri)` — read-only.

The CUO persona's CEO/COO/CTO skills include `cyberos.brain.search` and `cyberos.brain.get_community_summary` (read paths) by default; ADD/UPDATE/DELETE are routed through the natural-language CRUD path (batch-02) which surfaces a confirm-on-write step in the Genie panel.

**Audit integration.** Every retrieval and every ADD/UPDATE/DELETE writes an audit row in scope `brain.l2.{tenant}`. Retrieval rows include `query_text` (redacted), top-K fact IDs returned, persona-version of the asking persona, and consumer module. UPDATE/DELETE rows include the prior fact's content for reconstructability.

## Alternatives Considered

- **Vector-only memory (no graph).** Rejected: graph traversal is what answers "what's the pattern across recent deals" — vector alone cannot follow entity relations.
- **Graph-only memory (no vector).** Rejected: lexical/semantic queries that do not name an entity ("what's our policy on weekend on-call?") cannot be answered with graph traversal alone.
- **External vector store (Pinecone, Qdrant, Weaviate).** Rejected: residency and per-tenant isolation are harder to verify; we already have pgvector inside the same Postgres cluster, joinable to the audit log without cross-system joins.
- **Single-pass retrieval (no rerank).** Rejected: cross-encoder reranking is the cheapest way to materially improve precision-at-K; the cost is one self-hosted model call per retrieval.
- **Mem0 hosted service.** Rejected: data residency cannot be enforced; the four-operation pattern Mem0 popularised is replicated here under our own infrastructure.
- **Simple LLM-as-extractor with no operation classifier.** Rejected: leads to monotonic fact growth and contradiction; the four-operation classifier is what keeps the memory coherent over time.

## Success Metrics

- **Primary metric.** S0-3 demo passes: (1) 1,000 synthetic facts ingested across the 10 employees' Layer 1 directories, (2) "What is Acme's payment cycle?" returns the answer with the correct Layer 1 citation, (3) p95 retrieval latency ≤ 600 ms, (4) zero denylist-bypass entries observed in `brain.fact` for synthetic prompts containing VN_CCCD or compensation values.
- **Guardrail metric.** Compensation / equity values in `brain.fact.text` or `brain.fact.text_embed_m3` = 0 over the lifetime of the platform (PRD §4.3 anti-metric). Detection runs nightly via a regex sweep over `brain.fact.text`; a single match is sev-0.
- **Retrieval quality metric.** "Citation correctness" — for every CUO answer that cites a BRAIN fact, the cited fact must contain a substring or paraphrase that supports the answer. Measured weekly via a sampled human review (founder + Engineering Lead). Drift > 2% in a week reopens the persona.

## Scope

**In-scope (S0-3).**
- `brain.fact` schema + `brain.community_summary` schema + AGE graph initialisation.
- Ingestion pipeline with denylist + extractor + four-operation classifier + dedup.
- Hybrid retrieval pipeline (vector + lexical + graph + rerank + citation hydration).
- GraphRAG community-summary nightly job (Leiden + summary by Sonnet 4.6).
- MCP tools: `search`, `add_fact`, `update_fact`, `delete_fact`, `get_community_summary`.
- Conflict-detection only at S0-3 (the resolution UI is FR-BRAIN-CONFLICT-001 in batch-02).
- Source listeners for Layer 1 events; CHAT/KB/PROJ sources land as those modules ship in S0-4 / P1.
- Nightly denylist sweep over `brain.fact.text`.
- Audit integration in scope `brain.l2.{tenant}`.

**Out-of-scope (deferred).**
- Conflict-resolution UI (FR-BRAIN-CONFLICT-001 in batch-02).
- Natural-language CRUD ("forget that") via LLM (FR-BRAIN-NLCRUD-001 in batch-02).
- Layer 3 archival corpus (batch-02 covers Layer 3).
- External datasource ingestion (web pages, public APIs) — P1 KB module is the controlled gateway.
- Per-Member private-fact partitions (P3) — for now, all facts are tenant-scoped and visible to roles whose RBAC allows; the special-category exclusions handle the privacy-critical classes.

## Dependencies

- FR-INFRA-001 (Postgres with pgvector, AGE, PGroonga).
- FR-AUTH-001 / FR-AUTH-002 (identity + audit).
- FR-AI-001 (embedding + rerank + extractor model calls).
- FR-MCP-001 (tool registration + destructive-confirmation gate).
- FR-BRAIN-001 (Layer 1 source data; FR-BRAIN-002 indexes it).
- Compliance: PDPL Decree 13 (special-category exclusion via denylist), EU AI Act Article 50 (transparency on AI-generated summaries; the community summary is AI-derived and renders the disclosure chip).
- Locked decisions referenced: DEC-032 (three-layer architecture), DEC-038 (four-operation pattern), DEC-039 (hybrid retrieval), DEC-040 (GraphRAG community summaries), DEC-036 (denylist).

## AI Risk Assessment

This feature both ingests AI-extracted facts and serves AI-grounded retrievals to natural persons via the CUO surface. EU AI Act risk class: `limited`.

### Data Sources

Layer 2 ingests only: (a) Layer 1 files written by the tenant's own Members, (b) module events from CHAT, KB, PROJ, etc. for the same tenant, (c) the AI Gateway's extractor output derived from those sources. No third-party data sets, no public scraping, no cross-tenant ingestion. The extractor model (Haiku 4.5 or Sonnet 4.6) runs through the AI Gateway and inherits its ZDR posture; per-tenant residency is preserved.

### Human Oversight

Every UPDATE and DELETE on a fact requires human confirmation through the MCP destructive-confirmation gate. The conflict-resolution UI (batch-02) is the surface for resolving disputed facts. The community-summary nightly job's outputs render with a `persona_version` and `ai_disclosure_id` chip in any UI that surfaces them. The DPO can audit the full ingestion+retrieval log in scope `brain.l2.{tenant}` and `brain.denylist.{tenant}`.

### Failure Modes

- **Denylist bypass.** A new pattern of personal-data leakage emerges (e.g. a new VN-tax-code format). Mitigation: the nightly sweep detects it and pages; the regex set is updated; the offending facts are batch-deleted with audit entries.
- **Extractor hallucinates a fact.** The extractor produces a `subject_uri` or `predicate` not grounded in the input. Mitigation: the prompt explicitly forbids invention; a regression test runs in CI on a curated corpus; sampled human review weekly.
- **Stale fact survives retrieval.** A `superseded_by` fact is still returned. Mitigation: the retrieval default filter is `status = 'active'`; the `disputed` status excludes the fact unless the consumer explicitly opts in.
- **GraphRAG community summary drifts from facts.** Nightly job rebuilds from current facts; older summaries are versioned and the latest is the default.
- **Latency budget breach.** Retrieval falls back to vector-only path (skipping rerank) under load; consumer surface still cites correctly.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted ingestion-pipeline section, retrieval-pipeline section, schema, denylist enforcement, and failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; PGroonga and AGE integration to be re-verified by the Engineering Lead at PR-review.
