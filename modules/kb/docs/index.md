---
title: KB — RAG corpus · memory companion · Auto-runbook source · CyberOS
source: website/docs/modules/kb/index.html
migrated: FR-DOCS-002
---

KB is CyberOS's **documentation surface and the canonical source for AI-grounded retrieval**. The data model is simple: a `Document` has a slug, a markdown body, YAML frontmatter, a category, a permission tier, and a chain of `Version`s. Every save produces a new immutable version with a chained audit row in memory. The renderer produces sanitised HTML (server-side) for human reading and a clean plaintext stream for memory ingestion. Search is a three-layer stack: FTS5 / PGroonga (Vietnamese bigram tokenisation) for lexical, BGE-M3 embeddings (memory Layer 2) for semantic, and BGE-rerank-v2-m3 cross-encoder for re-ranking. "Ask this page" produces an answer grounded only in the current doc + explicitly linked docs, with span-level citations. Permissions: public · org-only · role-restricted (with share-link tokens for time-bound external access). Dual-language: a doc has a `language` field and an optional `translation_of` link to its counterpart. 

Status

Planned

P1 · design phase

Source

Markdown

\+ YAML frontmatter

Versioning

Immutable

every save = new version

Search stack

3 layers

FTS5 + semantic + reranker

AI Q&A

Grounded

span-level citations

Languages

vi + en

translation_of linkage

memory ingest

≤ 5 s p95

(FR pending)

Est. LoC

~6,500

Rust + TS editor

0

## The bigger picture — three strategic roles

The naive read of KB: "it's our wiki." The real read: KB is what makes Genie a useful agent and OBS a useful operations console. Strip KB out and CUO has nothing to cite, OBS's auto-runbook router has nothing to consult, and "ask the docs" becomes "make stuff up." KB is the structured-knowledge half of the memory protocol — long-form versioned, ACL-aware, server-rendered docs — that the audit-chained memory store doesn't replace but complements. 

Role 1 · RAG corpus

Three-layer retrieval · span-level citations

Markdown source + immutable versions. Triple-layer retrieval: FTS5 (PGroonga for VN) + BGE-M3 semantic embeddings + BGE-rerank-v2-m3 cross-encoder. "Ask this page" answers ground in current + linked docs with span citations. AI Q&A across all of KB respects ACL at retrieval time, not after. Span-level citations make hallucination detectable. 

Role 2 · memory companion

Versioned long-form alongside chain-anchored memories

memory is the audit-chained personal/tenant memory store; KB is the curated long-form companion. PROJ Issues, CRM Deals, OBS alerts can cite KB docs as authority (high-trust source) AND memory memories (chain-anchored decision). Semantic search spans both. "Promote to canonical" elevates a doc to high-authority memory source used by Lumi cross-tenant synthesis (if sync_class permits). 

Role 3 · Runbook catalogue for OBS

Auto-runbook router consults KB for alert remediation

OBS's auto-runbook router (§2.6 in obs.html) calls KB to find runbooks matching an alert's signature. Runbooks have an applicability tag (provider, region, severity), a step-by-step body, and a confidence-correlation field that grows as triage results feed back. KB is the catalogue; OBS is the consumer; CUO is the ranker. The growth loop is built-in: novel incidents become new runbooks; old runbooks gain confidence. 

### KB in the platform — who reads, who consults

flowchart TB KB["📚 KB  
markdown · versioned · ACL'd"] MEMBER["👤 Member  
reader + author"] GENIE["✨ Genie / CUO  
span-cited Q&A"] OBS["👁 OBS auto-runbook router  
consult catalogue"] PROJ["📋 PROJ  
Issues cite KB docs"] CRM["🤝 CRM  
account context cites KB"] memory["🧠 memory  
citation graph"] LUMI["☁ Lumi's memory (P3+)  
canonical-source cross-tenant"] PORTAL["🚪 PORTAL  
public-readable docs"] MEMBER -- "read · write" --> KB KB --> GENIE KB --> OBS PROJ -- "cite" --> KB CRM -- "cite" --> KB KB --> memory KB -. "promote to canonical (sync_class permits)" .-> LUMI KB --> PORTAL classDef hub fill:#fef3c7,stroke:#92400e,stroke-width:3px,color:#451a03 classDef mod fill:#e0e7ff,stroke:#3730a3 classDef memory fill:#fef6e0,stroke:#9c750a class KB hub class MEMBER,GENIE,OBS,PROJ,CRM,PORTAL mod class memory,LUMI memory 

### Auto vs human-in-loop operations matrix

Operation| How it happens| Why this split  
---|---|---  
Doc save| **Manual** Member action| Each save = new immutable version; never edit-in-place.  
Re-ingest to memory Layer 2| **Auto** on every save| Embedding + chunk-tracking; downstream RAG never stale.  
Translation linkage| **Manual** \+ auto-suggest| Author confirms `translation_of` after first VN/EN translation.  
"Ask this page" Q&A| **Auto** ground in current + linked docs| Bounded surface; ACL-aware; span citations always attached.  
Cross-KB Q&A| **Auto** \+ ACL filter pre-retrieval| Filter at retrieval time, never after; "no result" is OK.  
Promote to canonical| **Manual** CDO action| Elevation grants high authority weight in memory/Lumi synthesis; intentional.  
Runbook publish (for OBS)| **Manual** after post-incident review| Runbook is a deliberate authoring act; on-call authors after resolving.  
ACL change| **Manual** with audit row| Permission changes are high-impact; every change chains in memory.  
Share-link generation (time-bound)| **Manual** , **auto-expire**|  External access requires explicit token; auto-expires per policy.  
Stale-doc flag| **Auto** nightly scan| Docs > 6 months without view OR with dropped citation count flagged for review; CDO triages.  
  
1

## Why KB exists

Three jobs in one module: (a) **the docs surface** a team needs for how-tos, runbooks, decision logs, and policies; (b) **the canonical source for grounded AI retrieval** — if you do not control where the AI answer comes from, you do not control what it says; (c) **a per-doc permissioned publishing layer** for Trust Center pages and client-shared documents. Off-the-shelf wikis (Notion, Confluence) handle (a) but treat (b) and (c) as afterthoughts. CyberOS treats (b) as the central design force: every KB document is a first-class memory Layer 2 citation source, every Q&A answer cites span-level back to KB, and every "promote to canonical" elevates a doc to a high-authority memory source. That property is only credible when KB owns its versioning, ACL, and ingestion path. 

📌

Citations or it didn't happen

Every AI answer grounded in KB carries span-level citations. Hover the citation → see the exact source paragraph. Bad answers are auditable.

🌐

Dual-language native

vi and en docs are linked via `translation_of`; reader sees the language matching their JWT locale; the AI grounds in both when relevant.

🪪

ACL-aware retrieval

A user asking memory a question only gets citations from KB docs they are allowed to read. ACL is enforced at retrieval time, not after.

The bet is that the docs surface and the AI retrieval surface are the same surface. The cost is that KB is more constrained than a free-form wiki — every doc has a category, every save is versioned, every ingestion respects ACL. The benefit is that "ask the KB" is a credible AI feature, not a vibes-shaped hallucination machine, because every answer cites a specific span of a specific doc. 

2

## What it does — 5W1H2C5M

A structured decomposition of KB's scope. Every cell traces back to and §19.7.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is KB?| A markdown-source, server-rendered HTML, versioned, ACL'd documentation system that ingests into memory Layer 2 for AI-grounded retrieval. Three-layer search (FTS5 + semantic + reranker). "Ask this page" with span-level citations. Dual-language vi + en.  
**5W · Who**|  Who uses it?| **Members:** read + write docs daily. **CDO seat:** owns the surface; reviews "promote to canonical" requests. **Members in a category role:** can edit docs in their category. **Trust Center readers:** public-readable docs for opted-in tenants. **Agents:** KB is the primary grounded-retrieval target.  
**5W · When**|  When does it run?| Continuous: SPA editor + reader. On every save: render → memory ingest p95 ≤ 5 s. Nightly: dead-link detection; semantic index refresh on changed embeddings.  
**5W · Where**|  Where does it run?| P1: single region (SG-1) with VN-residency RDS. P3+: multi-region read replicas. Source markdown is RDS + S3 (S3 for attachments and large binaries).  
**5W · Why**|  Why a separate module?| Off-the-shelf wikis do not treat AI-grounded retrieval as a first-class concern; folding KB into memory corrupts the memory ingestion ledger; folding it into PROJ ties it to engagement-scoped lifetimes. Standalone module with tight memory integration is the right shape.  
**1H · How**|  How does it work?| Editor writes markdown + frontmatter; on save, server validates, renders HTML (sanitised), computes diff vs prior version, creates a Version row, queues memory ingest. Search: FTS5 / PGroonga produces top-100 lexical, BGE-M3 reranks, BGE-rerank-v2-m3 picks top-10. Q&A: pull top spans, format prompt, call AI Gateway with citation-required system instruction.  
**2C · Cost**|  Cost budget?| P1: ~$55 / month single-tenant pilot (Fargate + RDS + Redis + S3). Embedding cost ~$0.0001 / doc-version; reranker ~$0.0005 / query. 50-tenant: ~$220 / month.  
**2C · Constraints**|  Constraints?| (a) memory ingest p95 ≤ 5 s ((FR pending)). (b) Q&A must cite ((FR pending)). (c) Permissions enforced at retrieval time — ACL leak via Q&A is a sev-0 bug. (d) Trust Center pages are public-readable only when explicitly opted in ((FR pending)). (e) Vietnamese-quality search ≥ 90% recall on a fixed evaluation corpus.  
**5M · Materials**|  Stack?| Rust 1.81 · axum · sqlx · PostgreSQL 16 + PGroonga · pulldown-cmark for markdown · ammonia for HTML sanitisation · Redis 7 · S3 + KMS · BGE-M3 embedder + BGE-rerank-v2-m3 reranker (memory-shared) · TipTap or CodeMirror for the editor · OpenTelemetry SDK.  
**5M · Methods**|  Method choices?| Markdown source of truth (not block-based proprietary). Immutable versioning (no in-place edit). Server-side render (no client-side trust). FTS5 / PGroonga + semantic + reranker triple-layer (not just one). ACL at retrieval time (not at display time).  
**5M · Machines**|  Deployment?| Fargate axum service. RDS Postgres Multi-AZ. PGroonga compiled into the RDS image. Redis hot cache. Embedding + reranker GPU node shared with memory.  
**5M · Manpower**|  Who maintains?| 0.4 FTE (CDO seat) at P1 launch + 0.1 FTE (CCO for Trust Center pages). CTO owns the engine.  
**5M · Measurement**|  How measured?| Search p95 ≤ 350 ms, Q&A p95 ≤ 4 s end-to-end, citation accuracy ≥ 95% (claim → source span), memory ingest lag p95 ≤ 5 s, Vietnamese-query recall ≥ 90%.  
  
3

## Architecture

KB is one axum service. Four surfaces (GraphQL subgraph, REST admin, public-readable HTML for Trust Center pages, MCP tool catalogue). Three stores (PostgreSQL canonical + PGroonga + FTS5, Redis hot cache, S3 for attachments). The renderer and the memory ingester are separate concerns: the renderer produces HTML for humans; the memory ingester produces a sanitised plaintext + chunking stream for vectorisation. 

graph TB subgraph CLIENT ["Clients"] SPA["SPA editor + reader"] PUB["Public reader (Trust Center)"] AGENT["🎯 CUO via MCP"] end subgraph EDGE ["Edge"] GQL["GraphQL subgraph"] REST["REST admin + render"] HTMLPUB["Public HTML"] MCP["MCP tools"] end subgraph CORE ["KB service (Rust)"] DOC["Document CRUD"] VER["Version archiver"] RENDER["Server-side renderer  
pulldown-cmark + ammonia"] DIFF["Diff engine"] ACL["ACL gate  
public · org · role"] MEMORY_ING["memory ingester  
chunk + sanitise"] SEARCH["Search engine  
FTS5 + semantic + rerank"] QA["Q&A; grounded composer"] XLATE["Translation linker"] BACKLINK["Backlink computer"] end subgraph EMBED ["memory-shared"] EMB["BGE-M3 embedder"] RANK["BGE-rerank-v2-m3 reranker"] end subgraph STORES ["Stores"] PG[("PostgreSQL + PGroonga  
document · version · category  
RLS by tenant_id")] RED[("Redis 7  
rendered HTML cache · search cache")] S3[("S3 + KMS  
attachments")] end subgraph SINKS ["Sinks"] memory["🧠 memory  
Layer 2 ingestion · audit"] AI["⚡ AI Gateway"] OBS["👁 OBS"] end SPA --> GQL SPA --> REST PUB --> HTMLPUB AGENT --> MCP GQL --> DOC REST --> DOC REST --> RENDER REST --> SEARCH REST --> QA MCP --> SEARCH MCP --> QA HTMLPUB --> ACL DOC --> VER DOC --> RENDER DOC --> DIFF DOC --> MEMORY_ING DOC --> ACL MEMORY_ING --> memory MEMORY_ING --> EMB SEARCH --> EMB SEARCH --> RANK QA --> AI QA --> SEARCH DOC --> XLATE DOC --> BACKLINK DOC --> PG RENDER --> RED DOC --> S3 DOC --> OBS classDef planned fill:#fef6e0,stroke:#92400e classDef store fill:#f5f3ff,stroke:#7c3aed classDef sink fill:#f5ede6,stroke:#45210e class SPA,PUB,AGENT,GQL,REST,HTMLPUB,MCP,DOC,VER,RENDER,DIFF,ACL,MEMORY_ING,SEARCH,QA,XLATE,BACKLINK,EMB,RANK planned class PG,RED,S3 store class memory,AI,OBS sink 

### Document categories (closed)

how-to

Step-by-step instructions: "How to file a leave request", "How to onboard a Member".

reference

Stable facts: API reference, role catalogue, rate cards, compliance citations.

decision-log

Why we did X. Mirrors a memory `memories/decisions/` entry but human-readable.

policy

Company policy: leave, compensation, security, code of conduct.

runbook

Incident-response playbooks: AUTH key compromise, EMAIL Stalwart CVE, payroll outage.

trust-center

Public-readable on opt-in tenants. DPA, sub-processor list, security overview, DMARC status.

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`document.rs`| services/kb/src/document.rs| Document CRUD. Slug uniqueness per tenant. Frontmatter validation (kind, category, language, permission tier).  
`version.rs`| services/kb/src/version.rs| Version archiver. Every save → new immutable row. Retains markdown + rendered HTML hash.  
`renderer.rs`| services/kb/src/renderer.rs| Server-side markdown → HTML. Uses pulldown-cmark + ammonia (sanitise). No client-side JS execution.  
`diff.rs`| services/kb/src/diff.rs| Unified diff between versions. Powers the version-history UI.  
`memory_ingest.rs`| services/kb/src/memory_ingest.rs| On every version save: strip markdown, chunk at semantic boundaries, write memory Layer 2 rows + embeddings. p95 ≤ 5 s ((FR pending)).  
`search.rs`| services/kb/src/search.rs| Triple-layer search. FTS5 / PGroonga → top-100; BGE-M3 cosine top-30; BGE-rerank-v2-m3 → top-10.  
`qa.rs`| services/kb/src/qa.rs| Q&A composer. Pull top spans → format prompt with citation-required system instruction → call AI Gateway → parse cited answer.  
`acl.rs`| services/kb/src/acl.rs| ACL gate at retrieval time. Filters spans before they reach the QA composer.  
`share_link.rs`| services/kb/src/share_link.rs| Time-bound share-link tokens for external readers ((FR pending)).  
`translation.rs`| services/kb/src/translation.rs| Translation-of linkage. Reader sees doc in JWT-locale; AI grounds across language pairs.  
`backlink.rs`| services/kb/src/backlink.rs| Backlink graph: "what links here" query.  
`promote.rs`| services/kb/src/promote.rs| "Promote to canonical" — elevates a doc to a high-authority memory source ((FR pending)). Requires CDO approval.  
`notion_import.rs`| services/kb/src/notion_import.rs| Notion-export ZIP import ((FR pending)). Preserves links + categories.  
`export.rs`| services/kb/src/export.rs| Per-page or per-tree markdown export.  
`trust_center.rs`| services/kb/src/trust_center.rs| Public-readable Trust Center pages. Opt-in per tenant ((FR pending)).  
`migrations/`| services/kb/migrations/| sqlx migrations + PGroonga index DDL. RLS on every table.  
  
4

## Data model

Documents have a slug, current version pointer, category, permission tier, language, and optional translation_of link. Versions are immutable; the document row's `current_version_id` points at the latest. Permissions cascade by category (tenant default) and can be tightened per doc. 

erDiagram TENANT ||--o{ DOCUMENT: "owns" DOCUMENT ||--o{ VERSION: "has" DOCUMENT ||--o| DOCUMENT: "translation_of" DOCUMENT }o--|| CATEGORY: "in" DOCUMENT ||--o{ PERMISSION: "ACL" DOCUMENT ||--o{ TAG: "tagged" DOCUMENT ||--o{ ATTACHMENT: "has" DOCUMENT ||--o{ BACKLINK: "linked from" DOCUMENT ||--o| MEMORY_INGEST_STATE: "ingested" DOCUMENT ||--o{ SHARE_LINK: "shared via" VERSION ||--o| CITATION_SPAN: "indexed" DOCUMENT ||--o| PROMOTION: "promoted" TENANT { uuid id PK string slug } CATEGORY { string code PK "how-to | reference | decision-log | policy | runbook | trust-center" uuid tenant_id FK string display_name_vi string display_name_en string default_permission "public | org | role" } DOCUMENT { uuid id PK uuid tenant_id FK string slug string category_code FK string permission "public | org | role-restricted" string allowed_role_codes "csv (when role-restricted)" string language "vi | en" uuid translation_of FK "nullable" uuid current_version_id FK string title timestamp created_at timestamp updated_at uuid created_by FK bool trust_center_published } VERSION { uuid id PK uuid document_id FK int version_num string markdown_source string rendered_html_hash string body_text_sha256 int word_count timestamp saved_at uuid saved_by FK string change_summary string memory_chain } PERMISSION { uuid document_id FK uuid subject_id FK "explicit grant" string role_code "or role" string access "read | write" } TAG { uuid document_id FK string tag } ATTACHMENT { uuid id PK uuid document_id FK string filename string s3_key bigint size_bytes string mime_type bool scanned "clean" } BACKLINK { uuid from_document_id FK uuid to_document_id FK int link_count "occurrences" } MEMORY_INGEST_STATE { uuid document_id PK uuid latest_ingested_version_id FK int chunks_emitted timestamp ingested_at int latency_ms } SHARE_LINK { uuid id PK uuid document_id FK string token_sha256 timestamp valid_from timestamp valid_to int max_views "nullable" int views uuid created_by FK } CITATION_SPAN { uuid id PK uuid version_id FK int chunk_index int char_start int char_end string embedding_id "memory Layer 2 ref" } PROMOTION { uuid document_id PK string memory_canonical_path "memories/…" timestamp promoted_at uuid promoted_by FK } 

### Permission tiers

Tier| Visible to| Used for  
---|---|---  
`public`| Anyone with the URL (incl. anonymous if trust_center_published)| Trust Center pages, public marketing docs.  
`org`| Any authenticated subject in the tenant| How-to, reference, decision-log (default).  
`role-restricted`| Subjects holding one of `allowed_role_codes`| Policy (HR), runbook (CSO), compensation references.  
`explicit`| Subjects in `PERMISSION` table| Per-doc carve-outs (e.g. specific Member can read role-restricted).  
`share-link`| Anyone with the token, until expiry| Time-bound external sharing (client review of a doc).  
  
5

## API surface

Four surfaces: a federated GraphQL subgraph; a REST surface for editor + reader (with rendered HTML caching at the edge); a public HTML endpoint for Trust Center pages; and an MCP tool catalogue (search + ask) for CUO. 

### GraphQL subgraph
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@requiresScopes"])
    
    type Document @key(fields: "id") {
     id: ID!
     slug: String!
     title: String!
     category: Category!
     permission: PermissionTier!
     language: Language!
     translationOf: Document
     translations: [Document!]!
     currentVersion: Version!
     versions(limit: Int = 20): [Version!]!
     renderedHtml: String!
     tags: [String!]!
     attachments: [Attachment!]!
     backlinks: [Document!]!
     trustCenterPublished: Boolean!
     memoryIngestState: MemoryIngestState
     promotion: Promotion
    }
    
    type Version @key(fields: "id") {
     id: ID!
     documentId: ID!
     versionNum: Int!
     savedAt: DateTime!
     savedBy: Subject!
     changeSummary: String
     wordCount: Int!
     diffFrom(version: ID!): String!
    }
    
    type SearchResult {
     document: Document!
     snippet: String!
     score: Float!
    }
    
    type QAResult {
     question: String!
     answer: String!
     citations: [Citation!]!
     confidence: Float!
    }
    
    type Citation {
     documentId: ID!
     documentTitle: String!
     versionId: ID!
     charStart: Int!
     charEnd: Int!
     snippet: String!
    }
    
    enum Category { HOW_TO REFERENCE DECISION_LOG POLICY RUNBOOK TRUST_CENTER }
    enum PermissionTier { PUBLIC ORG ROLE_RESTRICTED EXPLICIT SHARE_LINK }
    enum Language { VI EN }
    
    type Query {
     document(id: ID, slug: String): Document
     searchDocuments(query: String!, category: Category, limit: Int = 10): [SearchResult!]!
     askPage(documentId: ID!, question: String!): QAResult!
     askKb(question: String!, scope: AskKbScope): QAResult! @requiresScopes(scopes: [["kb.ask"]])
    }
    
    type Mutation {
     createDocument(input: CreateDocumentInput!): Document!
     @requiresScopes(scopes: [["kb.write"]])
     saveDocument(id: ID!, markdown: String!, changeSummary: String!): Version!
     setPermission(id: ID!, tier: PermissionTier!, allowedRoleCodes: [String!]): Document!
     @requiresScopes(scopes: [["kb.permission"]])
     promoteToCanonical(id: ID!, memoryCanonicalPath: String!): Promotion!
     @requiresScopes(scopes: [["kb.promote"]])
     createShareLink(id: ID!, validUntil: DateTime!, maxViews: Int): ShareLinkResult!
     importNotionZip(zipS3Key: String!): NotionImportJob!
     @requiresScopes(scopes: [["kb.import"]])
    }

### REST surface

Method| Path| Purpose  
---|---|---  
GET| `/kb/{slug}`| Render document as HTML (ACL-gated).  
GET| `/kb/{slug}.md`| Markdown source download.  
POST| `/kb/{slug}/save`| Save markdown + frontmatter.  
GET| `/kb/{slug}/versions/{n}`| Render a specific version.  
GET| `/kb/{slug}/diff?from={a}&to={b}`| Unified diff.  
GET| `/kb/search?q=…&cat=…`| Triple-layer search.  
POST| `/kb/ask-page`| Q&A grounded in this page + linked pages.  
POST| `/kb/ask`| Q&A across whole KB (ACL-filtered).  
GET| `/trust-center/{slug}`| Public read (opted-in tenants).  
GET| `/share/{token}`| Share-link access.  
POST| `/admin/import/notion`| Notion ZIP import.  
POST| `/admin/export/tree?root=…`| Per-tree markdown export.  
  
### MCP tool catalogue

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cyberos.kb.search`| query, category?, limit| SearchResult| readonly · scope=kb.read  
`cyberos.kb.get_document`| slug| Document| readonly · scope=kb.read  
`cyberos.kb.ask_page`| document_id, question| QAResult| readonly · scope=kb.read  
`cyberos.kb.ask`| question, scope?| QAResult| readonly · scope=kb.ask  
`cyberos.kb.list_versions`| document_id| Version| readonly · scope=kb.read  
`cyberos.kb.diff`| document_id, from, to| diff text| readonly · scope=kb.read  
`cyberos.kb.save`| document_id, markdown, change_summary| Version| scope=kb.write  
`cyberos.kb.create_share_link`| document_id, valid_until, max_views?| {token, url}| scope=kb.share  
`cyberos.kb.promote`| document_id, memory_canonical_path| Promotion| destructive · human-confirm · scope=kb.promote  
  
6

## Key flows

### Flow 1 — Create / edit a doc with memory re-ingest

sequenceDiagram autonumber participant U as Editor SPA participant API as KB GraphQL participant V as Version archiver participant R as Renderer participant BI as memory ingester participant EMB as BGE-M3 embedder participant BR as 🧠 memory Layer 2 participant B as memory audit U->>API: saveDocument(id, markdown, change_summary) API->>API: parse + validate frontmatter API->>R: render markdown → sanitised HTML R-->>API: html · body_text · hash API->>V: INSERT version (immutable) V-->>API: version_id API->>B: kb.version_saved (audit chain) API->>BI: enqueue ingest job API-->>U: 200 Version (≤ 150 ms) BI->>BI: strip markdown · semantic chunking BI->>EMB: embed chunks (batch) EMB-->>BI: vectors BI->>BR: upsert chunks + vectors BI->>B: kb.memory_ingested {chunks, ms} Note over BI,B: total p95 ≤ 5 s ((FR pending)) 

### Flow 2 — Triple-layer search

sequenceDiagram autonumber participant U as User SPA participant API as KB /kb/search participant FTS as PostgreSQL + PGroonga participant EMB as BGE-M3 embedder participant ACL as ACL gate participant RANK as BGE-rerank-v2-m3 participant B as memory audit U->>API: GET /kb/search?q="hóa đơn cấp" API->>FTS: tsquery + PGroonga bigram match → top-100 FTS-->>API: candidate doc_versions API->>EMB: embed query EMB-->>API: query vector API->>API: cosine top-30 over candidate chunks API->>ACL: filter chunks by subject ACL ACL-->>API: visible chunks API->>RANK: rerank top-30 → top-10 RANK-->>API: scored results API->>B: kb.search {q, results_count} API-->>U: 200 SearchResult 

ACL is applied _before_ reranking, never after. A doc the user cannot read never reaches the reranker, the QA composer, or the UI.

### Flow 3 — "Ask this page" with citations

sequenceDiagram autonumber participant U as Reader participant API as KB /kb/ask-page participant ACL as ACL gate participant PG as PostgreSQL participant EMB as BGE-M3 participant CHUNKS as chunk store participant QA as Q&A; composer participant AI as ⚡ AI Gateway participant B as memory audit U->>API: askPage(document_id, "When do I file VAT?") API->>ACL: check subject can read document ACL-->>API: allowed API->>PG: fetch document + linked documents API->>EMB: embed question EMB-->>API: vector API->>CHUNKS: cosine top-8 across this doc + linked docs CHUNKS-->>API: spans API->>QA: compose prompt (system: "cite every claim with span id") QA->>AI: chat.completions AI-->>QA: cited answer JSON QA->>QA: validate every claim has a span_id alt valid QA-->>API: QAResult API->>B: kb.qa_answered {citations_count, confidence} API-->>U: 200 answer + citations else hallucination QA->>B: kb.qa_hallucination_blocked QA-->>API: fallback "I don't know" end 

### Flow 4 — Promote to canonical

sequenceDiagram autonumber participant M as Member participant API as KB participant CDO as CDO seat participant BR as 🧠 memory canonical participant B as memory audit M->>API: promoteToCanonical(doc, memory_path="memories/policy/leave.md") API->>API: validate scope=kb.promote API->>CDO: request approval (CHAT Notify) CDO->>API: approve API->>BR: write canonical memory entry mirroring KB doc BR-->>API: memory_chain API->>B: kb.promoted {document, memory_path, approver} API-->>M: 200 Promotion Note over BR: doc is now a high-authority memory source  
(Layer-1 canonical, not just Layer-2 chunk) 

### Flow 5 — Notion import

sequenceDiagram autonumber participant U as CDO participant CLI as cyberos-kb participant S3 as S3 participant IMP as Notion importer participant API as KB participant BI as memory ingester participant B as memory audit U->>CLI: cyberos-kb import notion --zip notion-export.zip CLI->>S3: upload zip CLI->>API: importNotionZip(zipS3Key) API->>IMP: parse zip · walk pages loop each page IMP->>IMP: convert blocks → markdown IMP->>API: createDocument + saveDocument API->>BI: enqueue ingest API->>B: kb.imported {notion_id, slug} end IMP-->>CLI: report {created:N, errors:K} CLI-->>U: ✓ 247 docs imported · 3 errors 

7

## Document lifecycle

A document's status is implicit (it always has a current version). Versions are immutable; archive / restore moves the `current_version_id` pointer. Promotion is a one-way state transition that registers the doc as a memory canonical source. 

stateDiagram-v2 [*] --> Draft: create Draft --> Published: first save (version 1) Published --> Published: subsequent saves version 2 plus Published --> Promoted: CDO promotes to canonical Promoted --> Published: demotion (rare; CDO + audit row required) Published --> Archived: archive (current_version preserved) Archived --> Published: restore Archived --> Purged: DSAR or 90-d after tenant offboard Purged --> [*] Promoted --> Archived: archive (memory canonical retained) 

### Version retention

Category| Retention| Notes  
---|---|---  
`policy`| 10 years| Required by Vietnamese Decree 13 / labour law.  
`runbook`| 5 years| Incident-response audit support.  
`decision-log`| indefinite| Mirrors memory decisions retention.  
`reference`| indefinite| Foundational facts.  
`how-to`| 2 years| How-tos drift; old versions archived.  
`trust-center`| indefinite| External commitments — provenance retained.  
  
8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

9

## Non-Functional Requirements

NFRs from that KB must satisfy.

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Search p95| ≤ 350 ms| OBS histogram  
`N(FR pending)`| Q&A p95 (end-to-end)| ≤ 4 s| OBS + AI Gateway  
`N(FR pending)`| memory ingest p95| ≤ 5 s ((FR pending))| BI histogram  
`N(FR pending)`| Document render p95 (cache-cold)| ≤ 250 ms| OBS histogram  
`N(FR pending)`| Vietnamese-query recall (eval corpus)| ≥ 90%| quarterly review  
`N(FR pending)`| Citation accuracy| ≥ 95%| monthly human review of 50 Q&A pairs  
`N(FR pending)`| Q&A "I don't know" rate on out-of-corpus queries| ≥ 90%| red-team eval  
`N(FR pending)`| ACL leak via search / Q&A| = 0| CI test on every PR  
`N(FR pending)`| HTML rendering XSS| = 0| ammonia sanitisation + CSP  
`N(FR pending)`| Service availability| ≥ 99.9% (28-day)| OBS SLO  
`N(FR pending)`| Version durability| 0 lost saves under crash| chaos test  
`N(FR pending)`| Policy / runbook retention (10 / 5 years)| 100%| retention policy enforcement  
  
10

## Dependencies

KB depends on AUTH (RBAC + ACL), memory (Layer 2 ingestion target + audit), AI Gateway (Q&A composer), MCP (CUO tools), and OBS. It is depended on by CUO (grounded answers), Trust Center readers, and downstream agents that ask KB questions. 

graph LR subgraph upstream ["KB depends on"] AUTH["🔐 AUTH  
RBAC + ACL"] memory["🧠 memory  
Layer 2 + audit"] AI["⚡ AI Gateway  
Q&A; composer"] EMB["BGE-M3 + reranker  
(memory-shared)"] MCP["🔌 MCP"] OBS["👁 OBS"] end KB["📚 KB"] subgraph downstream ["KB is depended on by"] CUO["🎯 CUO  
grounded retrieval"] PORTAL["Portal · P2  
(client KB views)"] EMAIL["✉️ EMAIL  
digests"] CHAT["💬 CHAT  
link previews"] end AUTH --> KB memory --> KB AI --> KB EMB --> KB MCP --> KB OBS --> KB KB --> CUO KB --> PORTAL KB --> EMAIL KB --> CHAT classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class memory,EMB shipped class KB,AUTH,AI,MCP,OBS,CUO,PORTAL,EMAIL,CHAT planned 

11

## Compliance scope

KB holds policy and decision-log documents that are themselves compliance artefacts; it must satisfy retention, residency, and access-audit obligations.

Regulation / standard| Article / clause| KB feature that satisfies it  
---|---|---  
Vietnam PDPL (Law 91/2025)| Art. 14 — DSAR| DSAR export of every doc a subject authored or edited.  
Vietnam Decree 13/2023| Art. 17 — Processing log| Every save / view writes a memory audit row.  
Vietnam Decree 53/2022| Art. 26 — Residency| VN-tenant docs on hanoi-1 RDS + S3.  
GDPR (EU 2016/679)| Art. 15 — Right of access| DSAR export.  
GDPR| Art. 17 — Right to erasure| Document purge with audit row; KB Layer 2 chunk removal cascades to memory.  
ISO/IEC 27001:2022| A.5.10 — Acceptable use| Policy docs live in KB; acceptance audit via read receipts.  
ISO/IEC 27001:2022| A.8.5 — Secure authentication| ACL-gated retrieval; share-link tokens time-bound.  
SOC 2 Type II| CC2.2 — Internal communication| KB is the canonical doc surface for policy + runbook.  
SOC 2 Type II| CC6.1 — Logical access| RBAC + per-doc ACL; ACL applied at retrieval.  
OWASP Top-10 (web)| A03 — Injection (XSS)| ammonia HTML sanitisation; CSP headers.  
  
12

## Risk entries

KB-specific risks tracked in the [risk register](<../../reference/risk-register.html#kb>).

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-KB-001`| ACL leak via search / Q&A surface| Low| High| CSO| ACL applied _before_ reranking + LLM; CI test asserts a restricted doc cannot surface.  
`R-KB-002`| Q&A hallucinates a citation that does not match the cited span| Medium| Medium| CDO| QA composer validates every cited span_id; mismatched citations rejected; "I don't know" returned.  
`R-KB-003`| memory ingest backlog blinds retrieval after major doc rewrite| Medium| Medium| CDO| p95 ≤ 5 s SLO; backlog alarm at > 60 s pages CDO.  
`R-KB-004`| Vietnamese tokenisation regression on PGroonga upgrade| Low| Medium| CTO| 50-query VN eval corpus run on every PGroonga upgrade.  
`R-KB-005`| XSS via markdown embedding raw HTML| Low| High| CSO| ammonia sanitiser + strict CSP; fuzz tests on every PR.  
`R-KB-006`| Notion import truncates large pages| Medium| Low| CTO| Page-size guard; rejected pages reported in import summary.  
`R-KB-007`| Share-link token replay after expiry| Low| Medium| CSO| Token expiry enforced server-side; revocation propagates to Redis cache within 30 s.  
`R-KB-008`| Promoted-to-canonical doc later modified, memory canonical out of sync| Medium| Medium| CDO| Demotion required before edit on promoted doc; or auto re-promotion with audit row.  
`R-KB-009`| Translation drift between vi and en versions| Medium| Low| CCO| Drift detection (word-count + key-phrase diff); flagged in editor UI.  
`R-KB-010`| Public Trust Center page reveals private policy by misconfiguration| Low| High| CCO| Trust-center publish requires double-confirm + audit row; CI test asserts no role-restricted docs published.  
`R-KB-011`| **Runbook catalogue drift — runbook claims "increase Bedrock quota" but tenant uses Vertex**|  Medium| Medium| CDO| Runbooks tagged with applicability (provider, region, severity); CUO triage filters before suggestion; staleness scan flags > 6mo unverified.  
`R-KB-012`| OBS-runbook coupling tightens — KB outage breaks auto-runbook router| Low| High| CTO| OBS caches last-known-good runbook catalogue (1h TTL); on KB outage CUO triage falls back to static severity routing; alarm on KB unreachable.  
`R-KB-013`| Span-citation drift — doc edited, citation now points to wrong paragraph| Medium| Medium| CDO| Citations are version-pinned (doc_id × version × span_id); editing creates new version; old citations resolve to old version; nightly sweep flags broken cites.  
`R-KB-014`| Vertical-pack vendor uploads malicious markdown that escapes sanitiser| Low| High| CSO| Pack-uploaded docs run through extended sanitisation (ammonia + CSP); CSO review required before promote-to-canonical for vendor-authored docs.  
`R-KB-015`| Q&A "I don't know" rate too high → Members stop using Genie| Medium| Medium| CDO| Out-of-corpus questions trigger doc-gap-detector → suggests "no doc exists, write one?"; tracking shows topic-area gaps.  
  
13

## KPIs

KB rolls up 9 KPIs covering search quality, Q&A grounding, ingestion latency, and editorial health.

KPI| Formula| Source| Target  
---|---|---|---  
**Search p95**|  histogram| OBS| ≤ 350 ms  
**Q &A p95**| histogram| OBS + AI Gateway| ≤ 4 s  
**Citation accuracy**|  matching_spans / claims| monthly human review| ≥ 95%  
**memory ingest p95**|  histogram| BI| ≤ 5 s  
**VN-query recall**|  relevant_returned / relevant_total| quarterly eval| ≥ 90%  
**"I don't know" rate (out-of-corpus)**|  idk / total_questions| red-team| ≥ 90%  
**Docs per Member**|  active_docs / members| memory audit| tracked; baseline 8  
**Stale-doc rate**|  > 180 d untouched / total| memory| ≤ 25%  
**ACL-leak incidents**|  count| memory audit| = 0  
**Runbook applicability accuracy**|  OBS routings correctly matched / total runbook suggestions| OBS triage telemetry| ≥ 0.80  
**Span-citation integrity**|  citations resolving to valid (doc_id × version × span_id) / total citations| nightly sweep| = 1.0  
**Doc-gap-detector signal rate**|  topic gaps suggested / "I don't know" responses| Q&A telemetry| ≥ 0.30 (catch enough gaps)  
**Cross-tenant retrieval reject rate**|  ACL-filtered retrievals / total retrievals| retrieval logs| tracked; spike = active probing  
**Vendor-pack doc CSO-review rate**|  pack-authored canonical-promoted docs with CSO sign-off / total such promotions| memory audit| = 1.0  
  
14

## RACI matrix

KB is owned by CDO seat (interim CEO).

Activity| CEO| CDO| CTO| CSO| CCO| CHRO  
---|---|---|---|---|---|---  
Service design + spec| A| R| C| C| C| I  
Implementation| I| C| A/R| C| I| I  
Promote-to-canonical approval| C| A/R| I| I| I| I  
Trust Center publication| C| C| I| C| A/R| I  
Policy doc authorship| C| C| I| C| I| A/R  
ACL audit| C| C| R| A| I| I  
Vietnamese-quality review| I| A/R| C| I| C| I  
Notion / external import| I| A/R| C| I| I| I  
DSAR fulfilment| I| C| C| R| I| I  
  
**R** Responsible · **A** Accountable · **C** Consulted · **I** Informed.

15

## Planned CLI surface

`cyberos-kb` for tenant operators, bulk import / export, and CDO promotion review.

### 1\. Create a doc from markdown
    
    
    $ cyberos-kb create \
     --slug how-to-file-leave \
     --category how-to \
     --language vi \
     --file./leave.md
    
    [create] doc id: 01HZL1…
    [render] markdown → HTML (sanitised)
    [memory] enqueued ingest job
    [audit] memory seq=15401 chain=…

### 2\. Link a translation
    
    
    $ cyberos-kb link-translation \
     --vi how-to-file-leave \
     --en how-to-file-leave-en
    
    [link] translation_of pair created (vi ↔ en)
    [audit] memory seq=15402 chain=…

### 3\. Search
    
    
    $ cyberos-kb search "hóa đơn cấp khi nào" --limit 5
    
    rank slug score snippet
    1 how-to-issue-hoadon 0.94 "...cấp hóa đơn khi giao hàng hoặc..."
    2 policy-hoadon-issuance 0.89 "...Theo Circular 78/2021, hóa đơn phải cấp..."
    3 runbook-hoadon-failure 0.76 "...nếu cấp hóa đơn thất bại, kiểm tra..."
    4 decision-log-hoadon-migration 0.71 "...chúng tôi chọn vietnam-vat-invoice vì..."
    5 reference-hoadon-fields 0.68 "...trường mst, đơn vị tính, thuế suất..."

### 4\. Ask a page
    
    
    $ cyberos-kb ask-page \
     --slug how-to-issue-hoadon \
     "When must we issue the hóa đơn?"
    
    answer (confidence: 0.91):
     The hóa đơn must be issued at the time of delivery of goods or completion
     of service provision [1], with the exception of advance payment scenarios
     where it must be issued within 5 working days of receipt [2].
    
    citations:
     [1] how-to-issue-hoadon · v3 · spans 142-298
     [2] policy-hoadon-issuance · v7 · spans 1402-1490

### 5\. Promote to canonical
    
    
    $ cyberos-kb promote --slug policy-leave --canonical-path memories/policy/leave.md
    
    [validate] requesting CDO approval (CHAT Notify sent)
    [approved] by stephen@cyberskill.world at 2026-05-14T09:32Z
    [memory] canonical entry created at memories/policy/leave.md
    [audit] memory seq=15418 chain=…

### 6\. Notion import
    
    
    $ cyberos-kb import notion --zip notion-export.zip --map cat=how-to
    
    [parse] 247 pages found in zip
    [convert] blocks → markdown
    [create] ✓ 244 docs created · ✗ 3 (in errors.csv)
    [memory] all enqueued
    [audit] memory seq=15489 chain=…

### 7\. Export a tree
    
    
    $ cyberos-kb export --root policy --format markdown --output./policies/
    
    [export] 28 docs · 4 categories · written to./policies/
    [manifest] policies/INDEX.md generated

16

## Phase status & estimates

Status

Planned

P1 · design phase

Est. LoC

~6,500

Rust + TS editor

Planned tests

80+

incl. ACL fuzz + citation eval

External libs

~11

axum · sqlx · pulldown-cmark · ammonia

CLI subcommands

~16 planned

`cyberos-kb`

P1 budget

~$55/mo

Fargate + RDS + Redis + S3

Capability| Status  
---|---  
Markdown editor + frontmatter| planned · P1  
Immutable versioning + diff| planned · P1  
Per-page ACL + share-link tokens| planned · P1  
FTS5 / PGroonga + semantic + reranker| planned · P1  
"Ask this page" with citations| planned · P1  
"Ask the KB" (whole-corpus QA)| planned · P1  
memory ingest p95 ≤ 5 s| planned · P1  
Promote-to-canonical (CDO gate)| planned · P1  
Translation linkage (vi ↔ en)| planned · P1  
Backlink graph| planned · P1  
Notion import + markdown export| planned · P1  
Trust Center public-readable pages| planned · P1  
Attachment AV scan| planned · P1  
Confluence / GitBook import| planned · P2+  
Real-time collaborative editing (Yjs)| planned · P2+  
Translation auto-draft via AI| planned · P2+  
  
17

## References

  * **FR catalogue** — KB product FRs.
  * **NFR catalogue** — KB NFRs.
  * **Bigger picture (§0 above):** 3 strategic roles + KB-in-platform Mermaid + 10-row auto-vs-human matrix.
  * **Cross-module page links:** [memory.html](<../memory/index.html>) · [obs.html](<../obs/index.html>) · [cuo.html](<../cuo/index.html>) · [ai.html](<../ai/index.html>) · [proj.html](<../proj/index.html>) · [portal.html](<../portal/index.html>)
  * **OBS auto-runbook contract:** [OBS §2.6](<../obs/index.html#auto-runbook>) — KB is the runbook catalogue source for the triage router.
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §6](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — KB docs promoted to canonical become high-authority source for Lumi cross-tenant synthesis (sync_class permitting).
  * **Build-readiness audit:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) — KB at P1 · mid (P1, alongside PROJ).
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>).
  * **BAAI BGE-M3** — multilingual embedding model (used for semantic layer).
  * **BAAI BGE-rerank-v2-m3** — cross-encoder reranker.
  * **PGroonga** — Postgres full-text search with Vietnamese bigram tokenisation.
  * **pulldown-cmark** — CommonMark + GFM parser for Rust.
  * **ammonia** — HTML sanitiser for Rust.
  * **CommonMark + GFM** — markdown specifications.
  * **Vietnam Decree 13/2023/NĐ-CP** — Personal data processing.
  * **Vietnam Law 91/2025/QH15 (PDPL)**.
  * **Notion export format** — ZIP of markdown + assets.
  * **Architecture context:** [infrastructure.html#kb](<../../architecture/infrastructure.html#kb>).



★

## Personas & skill bundles that touch KB

KB is the curated-canonical-docs companion to memory's audit-chained memory. Of the 47 CUO personas, the knowledge-stewardship ones below feed KB the most.

Persona affinities (6 of 47)

  * [chief-knowledge-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-knowledge-officer/workflows>) · quarterly-knowledge-pipeline + annual-knowledge-taxonomy
  * [chief-learning-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-learning-officer/workflows>) · annual-learning-program + leadership-development
  * [chief-product-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-product-officer/workflows>) · feature-prd-intake (canonical PRDs)
  * [chief-technology-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-technology-officer/workflows>) · adr-quick-capture (canonical ADRs)
  * [chief-data-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-data-officer/workflows>) · quarterly-data-governance-review
  * [chief-of-staff](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-of-staff/workflows>) · decision-log-keeping (decision-log canonical home)



Skill-bundle reads & writes

  * [product-requirements-document-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/product-requirements-document-author>) \+ audit · PRDs land here
  * [architecture-decision-record-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/architecture-decision-record-author>) \+ audit · ADRs land here
  * [runbook-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/runbook-author>) \+ audit · ops runbooks
  * [software-design-document-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/software-design-document-author>) \+ audit · canonical SDDs
  * [knowledge-pipeline-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/knowledge-pipeline-author>) \+ audit · CKO recurring pipeline



KB documents marked `canonical` elevate to high-authority memory sources used by Lumi cross-tenant synthesis (when sync_class permits). See memory page §3.4.

[← Previous module: CRM](<../crm/index.html>) [All modules →](<../index.html#catalog>)

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
