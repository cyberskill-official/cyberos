---
title: "BRAIN Layer 3 — archival corpus of raw conversations indexed for retrieval"
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
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship Layer 3 of BRAIN: a per-tenant archival corpus that retains the **raw, unsummarised conversation text** (CHAT messages, EMAIL bodies, meeting transcripts, KB page revisions, decision-ledger comment threads, CUO interaction histories) indexed for retrieval but kept structurally separate from Layer 2's distilled facts. Layer 3 is the cold tier in the three-layer architecture (PRD §5.2 / §5.5); it is the source-of-record from which Layer 2 facts were extracted, and the substrate that GraphRAG community summaries, the Auto Dream nightly consolidation, and the regulator-ready right-to-erasure (RTBE) export draw from. Storage is Postgres (hot rolling 90 days) plus S3-compatible object storage with Object Lock in `Compliance` mode (cold archive, 7-year default retention, 10-year for `rew.*` / `esop.*` / `hr.contract.*` scopes). Hybrid retrieval against Layer 3 is gated to specific read paths (CUO Review-mode long-form drafts, KB Q&A, audit reconstruction, DSAR responses) — never streamed at high volume to consumers because Layer 2 is the canonical retrieval surface.

## Problem

Layer 1 (Markdown filesystem; FR-BRAIN-001) and Layer 2 (vector + graph fact memory; FR-BRAIN-002) together answer "what does the platform know" but neither answers "where did that fact come from in the original conversation?" The PRD's no-hallucination posture requires that every fact be traceable back to its raw source text — for citation correctness, for audit reconstruction, for regulator response, and for the eventual DSAR / RTBE flows in P3+. A Layer-2-only architecture loses provenance the moment the extractor distills a 200-message thread into one fact; a Layer-1-only architecture cannot store the firehose of CHAT and EMAIL because Markdown files were not designed for high-velocity append-only volume.

Three product properties depend on Layer 3 being right:

- **Citation provenance.** When CUO answers "Acme is on a 90-day payment cycle" with a Layer 2 citation, the citation must be a clickable link back to the originating CHAT message or email — not the Layer 1 file (which is a human-curated synthesis, not the canonical source).
- **Right-to-erasure.** PDPL Decree 13 plus GDPR Article 17 (P3+) require that on a verified erasure request the platform deletes *all* personal data of the data subject, including raw conversations referencing them. Without Layer 3 we cannot answer "what raw text mentions this person" at the granularity the regulator expects.
- **Auto Dream consolidation.** The nightly job that recomputes GraphRAG community summaries (FR-BRAIN-002) needs the raw corpus to re-extract whenever a denylist update or an extractor-prompt revision changes how facts are produced; without Layer 3 a re-extract is impossible because the original text has been thrown away.

## Proposed Solution

The shape of the answer is a Layer 3 subgraph on the BRAIN module plus the hot/cold tiering pipeline plus a tightly-scoped retrieval surface.

**Schema.**

```sql
CREATE TABLE brain.l3_doc (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  source_kind     TEXT NOT NULL,    -- "chat.message", "email.body", "meeting.transcript",
                                    -- "kb.revision", "decision.comment", "cuo.interaction"
  source_ref      TEXT NOT NULL,    -- canonical pointer (e.g. "chat://channel-id/msg-id")
  source_url      TEXT,             -- deep-link the user can click
  authors         UUID[] NOT NULL,  -- Member IDs referenced or authored
  occurred_at     TIMESTAMPTZ NOT NULL,
  raw_text        TEXT NOT NULL,    -- the canonical raw content, redacted only at the denylist level
  raw_text_pgrn   TSVECTOR_TYPE NOT NULL,  -- PGroonga full-text index
  raw_text_embed  vector(1024),     -- bge-m3 paragraph embedding (cap 4 KB; longer docs chunked)
  metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
  layer2_fact_ids UUID[],           -- the facts derived from this doc; updated by the extractor
  retention_class TEXT NOT NULL,    -- "default-7y", "rew-10y", "esop-10y", "contract-10y"
  archived_at     TIMESTAMPTZ,      -- when migrated to S3 cold tier
  s3_object_key   TEXT,             -- populated when archived
  s3_bucket       TEXT,
  pii_pseudonymisation_pending BOOLEAN NOT NULL DEFAULT false,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (occurred_at);

CREATE INDEX l3_doc_tenant_kind_idx ON brain.l3_doc (tenant_id, source_kind, occurred_at DESC);
CREATE INDEX l3_doc_pgrn_idx        ON brain.l3_doc USING pgroonga (raw_text);
CREATE INDEX l3_doc_embed_idx       ON brain.l3_doc USING hnsw (raw_text_embed vector_cosine_ops);
CREATE INDEX l3_doc_authors_idx     ON brain.l3_doc USING gin (authors);
```

Monthly partitions; `pg_partman` migrates partitions older than 90 days to the cold tier (S3 object per partition, manifest-indexed) and detaches the partition. Documents are never deleted from the cold tier except by an explicit RTBE (FR-CP-002 in this batch); the Object Lock in `Compliance` mode prevents accidental deletion even by a privileged operator.

**Ingestion.** Layer 3 listens on the same NATS subjects as Layer 2 (FR-BRAIN-002 §"Ingestion pipeline") plus dedicated subjects for high-volume sources (`cyberos.{tenant}.chat.message.posted`, `cyberos.{tenant}.email.message.received`). The ingestion path:

1. **Denylist filter (same as Layer 2).** Compensation, equity, government IDs, bank accounts, special-category health data — never enter the corpus. Matched documents are dropped; an audit row is written in scope `brain.denylist.{tenant}`. This is the same regex set FR-BRAIN-002 §"Denylist filter" uses; both layers share the implementation library.
2. **Chunking.** Docs ≤ 4 KB are stored whole; larger docs (long emails, meeting transcripts) are chunked at paragraph boundaries with 256-token overlap; each chunk is its own `l3_doc` row with a `metadata.parent_doc_id` pointing to a synthetic parent.
3. **Embedding.** `bge-m3` paragraph embedding via the AI Gateway (FR-AI-001) `/v1/embed`. Self-hosted; no external provider call.
4. **PGroonga indexing.** Vietnamese-aware tokenisation; same configuration as CHAT search and BRAIN Layer 2.
5. **Provenance link.** When Layer 2 ingestion produces a fact derived from this document, the fact's `provenance.source_doc_id` references the Layer 3 row, and the Layer 3 row's `layer2_fact_ids[]` is updated. The bidirectional link is what makes the citation back-trace work.

**Retrieval surface.** Layer 3 is *not* the default retrieval target. CUO's hybrid retrieval pipeline (FR-BRAIN-002) hits Layer 2 first; Layer 3 is consulted in five specific cases:

1. **Citation hydration.** When CUO returns a Layer 2 fact, the UI needs the raw quote — Layer 3 returns the source paragraph for the citation chip.
2. **Review-mode long-form.** CUO's Review-mode drafts (FR-GENIE-002 in this batch) cite raw passages from Layer 3 to ground the draft.
3. **KB "ask this page" deep search.** When a user is on a KB page and asks a question whose answer is in a comment thread, Layer 3 surfaces the thread.
4. **Audit reconstruction.** An auditor reconstructing what a Member said about a topic on a date queries Layer 3 directly (RBAC-gated; `Auditor`, `DPO`, and the Member-themselves only).
5. **DSAR / RTBE.** A data-subject-access-request response or an erasure-request execution enumerates Layer 3 by `authors[]` containing the subject ID.

Retrieval API:

```graphql
type Query {
  brainL3Search(query: String!, sourceKind: SourceKind, since: DateTime, until: DateTime,
                authorMemberId: ID, first: Int = 20): BrainL3DocConnection!
  brainL3Doc(id: ID!): BrainL3Doc
  brainL3DocsForFact(factId: ID!): [BrainL3Doc!]!
  brainL3DocsForMember(memberId: ID!, first: Int = 100, after: String): BrainL3DocConnection!
}
```

`brainL3DocsForMember` is the DSAR helper; access requires the requesting Member to be the subject themselves, the DPO, or an Auditor with an open case. Access is audited in scope `brain.l3.dsar.{tenant}`.

**MCP tools.**
- `cyberos.brain.l3_search(query, source_kind?, since?, until?, author?)` — read; `read_only: true`.
- `cyberos.brain.l3_get_doc(id)` — read.
- `cyberos.brain.l3_docs_for_fact(fact_id)` — read; the citation-hydration path.

There is **no** write tool for Layer 3 directly; ingestion is event-driven, not call-driven, by design. Erasure happens through the RTBE flow (FR-CP-002), not through an MCP tool.

**Hot/cold tiering pipeline.** The `pg_partman` job runs nightly at 03:30 ICT (after Auto Dream's 03:00 community-summary job finishes). For each partition older than 90 days:

1. Stream the partition's rows to a JSONL file with one row per line.
2. Compress with `zstd` level 19.
3. Upload to `s3://cyberos-brain-l3-archive-{region}/{tenant_id}/{yyyy-mm}/partition.zst.jsonl` with Object Lock in `Compliance` mode and the retention end-date set to the partition's `retention_class` floor.
4. Verify by re-reading the manifest and checksum-matching against the source.
5. Update the rows with `archived_at` + `s3_object_key` + `s3_bucket`.
6. Detach (not drop) the partition; the metadata rows remain queryable.

When a query needs to read an archived partition, the gateway reads the JSONL from S3, materialises a temporary table, runs the query, and discards the materialisation. For low-frequency historical queries (auditor, DSAR) the latency penalty is acceptable; for hot-path retrievals the 90-day rolling window is the floor.

**Re-extraction support.** A founder-only operation `cyberos brain reextract --since YYYY-MM-DD --extractor-version v0.5.2` walks Layer 3 (hot + cold) and re-runs the Layer 2 extractor. Used when the extractor prompt is materially revised or the denylist is tightened. Re-extraction produces an audit log diff: removed facts, updated facts, new facts. The diff is reviewed by the founder before being applied.

## Alternatives Considered

- **Skip Layer 3; keep only Layer 2.** Rejected: citation provenance fails, RTBE cannot be executed at the regulator's expected granularity, Auto Dream cannot re-extract.
- **Use Mattermost's native message store as the archival corpus for CHAT and skip a separate L3 table.** Rejected: cross-source search (CHAT + EMAIL + KB + meeting transcripts together) would require federation across heterogeneous schemas; one canonical Layer 3 schema makes the retrieval surface uniform.
- **Cold archive on the same Postgres cluster with cheap storage, no S3.** Rejected: regulatory Object Lock requires a backend that supports retention modes; Postgres cannot.
- **Encrypt every Layer 3 row with a per-row tenant-derived key.** Deferred: per-row crypto adds operational complexity without a marginal compliance gain over Postgres TDE plus S3 Object Lock plus per-tenant residency. Revisit at P3.
- **Mirror Layer 3 to the SIEM.** Deferred to P3 alongside the audit-log SIEM forwarding.

## Success Metrics

- **Primary metric.** S0-3+ demo passes: (1) every CHAT message and KB page revision produced during the synthetic test scenario lands in `brain.l3_doc` with the correct `source_kind`, (2) a Layer 2 fact returned by CUO surfaces its source paragraph from Layer 3 in the citation chip in ≤ 200 ms p95, (3) a synthetic DSAR for `Member-X` enumerates 100% of Layer 3 docs whose `authors[]` contains the Member, (4) the cold-tier migration job moves a 91-day-old synthetic partition to S3 with verified Object Lock retention and successful read-back materialisation.
- **Guardrail metric.** Zero unauthorised Layer 3 reads across the lifetime of P0. Authorised reads are: Auditor + DPO + author-self + automated citation-hydration. Any other read attempt is logged + alerted.
- **Performance NFR.** L3 hot-tier search p95 ≤ 800 ms over a 1M-document synthetic corpus.

## Scope

**In-scope (S0-3 base + S0-6 cold-tier).**
- `brain.l3_doc` table, partitioning, hot indexes (PGroonga + HNSW + GIN authors).
- Ingestion path on canonical NATS subjects with denylist + chunking + embedding.
- Bidirectional Layer 2 ↔ Layer 3 link (fact provenance + doc-derived facts).
- Retrieval API with the four use-case-gated paths.
- MCP tools (read-only).
- `pg_partman` cold-tier migration job to S3 with Object Lock in `Compliance` mode.
- Read-back path that materialises archived partitions on demand.
- Re-extraction CLI (founder-only).
- Audit integration in scope `brain.l3.{tenant}`; DSAR-class reads in scope `brain.l3.dsar.{tenant}`.

**Out-of-scope (deferred).**
- Per-row crypto with tenant-derived keys (P3).
- SIEM mirror of Layer 3 (P3).
- L3 → KB promote-to-canonical UI (P1).
- L3 streaming consumer for downstream pipelines beyond what Layer 2 needs (P2 — surfaces in REW close-cycle anomaly detection).
- Cross-tenant federated L3 search (forbidden by design — never).

## Dependencies

- FR-INFRA-001 (Postgres + S3 access).
- FR-AUTH-001, FR-AUTH-002 (identity + audit).
- FR-AI-001 (embedding endpoint).
- FR-MCP-001 (MCP gateway for the read tools).
- FR-BRAIN-001 (Layer 1 source events).
- FR-BRAIN-002 (Layer 2 fact provenance link).
- FR-CHAT-001 (CHAT message events).
- FR-OBS-001 (cold-tier migration metrics + alerts).
- S3-compatible object store with Object Lock in `Compliance` mode (Hetzner Object Storage at present; AWS S3 if Hetzner support is insufficient — DEC-053+).
- Compliance: PDPL Decree 13 (raw conversation content is personal data; the per-tenant residency + Object Lock retention floor is the control). EU AI Act Article 12 (logging) — Layer 3 is the deep evidence stream alongside the audit log.
- Locked decisions referenced: DEC-032 (BRAIN three-layer architecture), DEC-054 (Object Lock Compliance mode for cold archive), DEC-055 (90-day hot retention floor).

## AI Risk Assessment

Layer 3 is the substrate for AI-grounded retrieval; while the layer itself is storage, its retrieval surface materially shapes AI-derived behaviour. EU AI Act risk class: `limited`.

### Data Sources

Layer 3 stores per-tenant raw conversation content, written by the tenant's own Members and the external counterparties they correspond with (email senders, customer DMs the Member shares). No third-party training data is ingested. The denylist filter prevents compensation, equity, government-ID, bank-account, and special-category health content from entering. Re-extraction is bounded to per-tenant, per-extractor-version reruns over the same corpus.

### Human Oversight

Layer 3 reads are RBAC-gated and audited; high-impact reads (DSAR, RTBE, re-extraction) require a privileged role and write a high-prominence audit row that the Compliance Cockpit (FR-OBS-001) surfaces. The DPO has end-to-end visibility on every L3 read in DSAR scope. Re-extraction diffs are reviewed by the founder before application.

### Failure Modes

- **Cold-tier migration corruption.** Manifest checksum mismatch fails the migration; the partition stays hot until resolved; alert fired.
- **S3 Object Lock retention misconfiguration.** Sev-0; tenant data could be deleted prematurely. Mitigation: per-bucket policy enforced at provisioning; nightly verification job re-asserts policy parity.
- **Denylist evasion through obfuscation.** A new pattern slips through. Mitigation: nightly sweep over `raw_text` plus a `pii_pseudonymisation_pending` workflow that lets the DPO redact without dropping the row (preserving citation provenance for the surrounding facts).
- **Read-back from cold tier exceeds latency budget.** Caller surface ("Genie, what did Acme say about pricing in March 2025?") shows a "fetching from archive" indicator; query returns within 30 s p95.
- **Bidirectional Layer 2 ↔ Layer 3 link drift.** Reconciler runs hourly; orphan facts (no source doc) are flagged and reviewed.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, ingestion path, hot/cold tiering pipeline, re-extraction flow, failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; the S3 Object Lock interaction with Hetzner Object Storage to be re-verified by the Engineering Lead at PR-review.
