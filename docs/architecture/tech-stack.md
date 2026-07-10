---
title: Tech Stack — CyberOS
source: website/docs/architecture/tech-stack.html
migrated: FR-DOCS-002
---

0

## Eight tiers, one supergraph

CyberOS has more than 8 distinct concerns (we enumerate 16 below), but they cluster cleanly into 8 architectural _tiers_. Each tier is a separate deployment unit, scales independently, and exposes a stable contract to the tier above. 

flowchart TB subgraph T1 ["T1 · Persona / Agent layer"] LANG["LangGraph supervisor (StateGraph + interrupt)"] SKILLS["Anthropic Skills format · 47 C-suite persona workflows hot-reload"] LITELLM_T1["LiteLLM client (routing core)"] end subgraph T2 ["T2/T3 · Frontend layer"] HOST["Host shell · Vite + React 19 + Tauri (desktop)"] REMOTES["Module remotes · Webpack 5 + Module Federation"] end subgraph T3 ["T4/T5/T6 · API + Agent surface"] APOLLO["Apollo Router · GraphQL Federation v2.5+"] MCPGW["MCP Gateway · Streamable HTTP · 2025-11-25"] AIGW["AI Gateway · LiteLLM router · Bedrock primary"] end subgraph T4 ["T7 · Backend services"] SUBGRAPHS["22 subgraphs · TypeScript (Yoga) or Rust (async-graphql)"] MCPSERVERS["22 MCP servers · per-module · TS SDK or mcp-rs"] end subgraph T5 ["T8/T9 · Data + search"] PG["PostgreSQL 17 + pgvector HNSW + Apache AGE 1.5 + PGroonga"] EMBED["BGE-M3 embedder + BGE-rerank-v2-m3 (self-hosted)"] end subgraph T6 ["T10/T11 · Infrastructure"] NATS_T["NATS JetStream (event spine)"] S3["S3 / R2 / MinIO (object storage)"] end subgraph T7 ["T12/T13/T14 · Cryptography + sync"] YJS["Yjs / Automerge (CRDTs for realtime)"] CRYPTO["Ed25519 + scrypt key wrap + MMR + STH"] LEDGER["msgspec canonical JSON · binlog framing"] end subgraph T8 ["T15/T16 · Compliance + UX"] OPA["OPA + Conftest (policy)"] TRUST["Trust Center (cert hosting)"] BVP["Be Vietnam Pro · CyberSkill design system"] end T1 --> T2 T2 --> T3 T3 --> T4 T4 --> T5 T4 --> T6 T4 --> T7 T4 --> T8 classDef t1 fill:#f9c64f,stroke:#9c750a classDef t2 fill:#e8d4c2,stroke:#45210e classDef t3 fill:#fef6e0,stroke:#9c750a classDef t4 fill:#f5ede6,stroke:#45210e classDef t5 fill:#cba88a,stroke:#45210e classDef t6 fill:#fde7b3,stroke:#9c750a classDef t7 fill:#fee2e2,stroke:#b91c1c classDef t8 fill:#f0eee9,stroke:#475569 class LANG,SKILLS,LITELLM_T1 t1 class HOST,REMOTES t2 class APOLLO,MCPGW,AIGW t3 class SUBGRAPHS,MCPSERVERS t4 class PG,EMBED t5 class NATS_T,S3 t6 class YJS,CRYPTO,LEDGER t7 class OPA,TRUST,BVP t8 

### Three design constraints driving every pick

  1. **1\. Vietnamese data sovereignty** — no SaaS dependency where Vietnamese-origin personal data must travel through a US-based vendor's servers. AWS Bedrock is acceptable because of the ap-southeast-1 region; OpenAI direct is not (no Singapore endpoint).
  2. **2\. Cost ceiling at scale** — ≤ $150/mo LLM + $230/mo infra at 10-Member internal; ≤ $4/active user/mo LLM + $2,200/mo infra at 50-tenant (N(FR pending), N(FR pending)). Anything that doesn't fit that envelope is rejected.
  3. **3\. Migration door always open** — every pick has a documented escape hatch. Storage is S3-compatible (DEC-005), so R2 ↔ MinIO ↔ AWS S3 is a config flip. SQL is portable, audit chain is exportable, MCP servers are spec-conforming.



1

## Tier 1 — Persona / Agent layer

**Pick:** LangGraph (state graph) + LiteLLM (routing) + Anthropic Skills format (47 C-suite persona workflows · hot-reloadable).

LangGraph is the agentic-supervisor framework. Its `StateGraph` primitive models CUO as a graph of nodes (router, skill-load, tool-call, HITL-confirm, answer-compose) with first-class `interrupt` support for human-in-the-loop gates on destructive tools. LiteLLM is the model-routing core that owns provider failover and prompt caching. Anthropic Skills format (`SKILL.md` \+ `scripts/` \+ `references/`) keeps each C-level skill as a hot-reloadable directory. 

#### Why these picks

  * **LangGraph, not DSPy / native LangChain** — StateGraph models CUO's interrupt/resume semantics natively; DSPy is optimisation-first (signature → optimiser pipeline) and not a fit for a long-lived router; native LangChain agents are imperative and harder to audit.
  * **LiteLLM** — MIT-licensed, OpenAI-shaped surface, 100+ providers behind a single API, drop-in OTel hooks.
  * **Anthropic Skills format** — open standard (Anthropic + VS Code + Cursor + Codex compat); hot-reload via `watchexec`; metadata-first dispatch keeps context window lean.



#### Trade-offs

  * **LangGraph cost:** Python-only (no Rust); requires an LLM API per supervisor decision (mitigated by Haiku-tier routing).
  * **Skill format:** not yet a published RFC; depends on Anthropic continuing to evolve it. Mitigation: schema-pinned to 2025-04 spec; conformance tests in CI.
  * **LiteLLM fork risk:** CyberOS-specific middleware (Presidio redaction, VN identifier rules, persona stamping) lives as a fork branch; rebases monthly.



#### Production cost

≤ $0 host cost (runs inside the CUO service Pod). LLM cost flows through Tier 6 AI Gateway.

2

## Tier 2 — Frontend host shell

**Pick:** Vite + React 19 + Tauri (desktop bundling).

The host shell is the thin orchestrator that hosts Module-Federation remotes. Vite (Rollup-based) for dev-server speed; React 19 for the new `use` hook and React Compiler's automatic memoisation; Tauri for the desktop bundling story. 

#### Why these picks

  * **Vite** — sub-second HMR; ESM-native; Rollup-based prod build; Module Federation plugin (`@module-federation/vite`) stable in late 2025.
  * **React 19** — React Compiler removes most `useMemo`/`useCallback` ceremony; `use` hook simplifies suspense; concurrent rendering is GA.
  * **Tauri 2 (Rust)** — 3-10 MB bundle vs Electron's 100+ MB; uses OS webview; passes Apple notarisation by default.



#### Trade-offs

  * **Webview consistency:** Tauri uses OS-native webviews (Edge WebView2, WKWebView, WebKitGTK) → CSS feature parity needs CI matrix testing.
  * **Module Federation + Vite:** the official plugin is _newer_ than Webpack's; some plugins lag behind. Mitigation: host shell built around the official plugin's stable subset only.
  * **React 19 ecosystem:** some libraries (charts, drag-drop) still pinning React 18; monitor and pin as needed.



3

## Tier 3 — Frontend remotes (per module)

**Pick:** Webpack 5 + Module Federation v2 per module.

Each module ships as an MF remote bundle. The host shell lazy-loads on route entry. CSS scoped via CSS Modules to prevent cross-module collisions. Design tokens come from one published package (`@cyberskill/tokens`). 

#### Why Webpack for remotes, Vite for host?

  * **Module Federation maturity** — Webpack's MF plugin has 4+ years of production miles; remote bundling, runtime versioning, and shared-deps resolution are battle-tested.
  * **Bidirectional remotes** — Webpack MF supports remote-as-host (a module can host its own sub-remotes); useful for SKILL → CUO interactions.
  * **Host can be Vite** — the Vite MF plugin can consume Webpack remotes; the inverse is not always true.



#### NFR ceilings

  * **N(FR pending)** — module first-paint ≤ 1.5s on cold load
  * **N(FR pending)** — initial module bundle ≤ 50 KB gzipped JS
  * **N(FR pending)** — module-rebuild time on token change ≤ 30 min including tests



4

## Tier 4 — API gateway (GraphQL Federation)

**Pick:** Apollo Router (Rust) — Federation v2.5+ compliant.

The Rust-based Apollo Router executes the composed supergraph plan. Validates JWTs, attaches tenant + actor context, runs persisted-query lookup, dispatches subgraph fanout in parallel. Detailed in the [Infrastructure page](<infrastructure.html#graphql-federation>). 

#### Why Apollo Router, not Mesh / Hasura / Yoga?

  * **Apollo Router** — production-grade Rust runtime; reference implementation for Federation v2.5; query plan cache; persisted-query story is first-class.
  * **GraphQL Mesh** — flexible but is the wrong abstraction for federation (REST/SOAP/SQL adapters; not a router).
  * **Hasura** — Postgres-first; doesn't model multi-subgraph federation; vendor lock-in concerns.
  * **GraphQL Yoga (as router)** — Yoga is great as a subgraph server; not as a federation router.



#### Trade-offs

  * **Elastic License (v1.2)** — non-OSI but production-friendly; review legal before commercial offering.
  * **Single binary** — Rust-only; tweaks require Rust expertise (or YAML config + Rhai script).
  * **Telemetry surface** — needs OTel collector + Grafana to be useful (already in OBS).



5

## Tier 5 — MCP Gateway (per-module servers)

**Pick:** Per-module MCP servers + federation router · 2025-11-25 spec.

Each module owns its MCP server. The gateway is a federation router, not a monolith. Streamable HTTP, OAuth-PRM, well-known discovery, tool annotations. Detailed in the [Infrastructure page](<infrastructure.html#mcp-gateway>). 

#### SDK choice

  * **TypeScript:** official `@modelcontextprotocol/sdk` (1.20+) — used by 19 of 23 modules
  * **Rust:** CyberSkill-published `mcp-rs` — used by memory, Skill (the two Rust-first modules)
  * **Tool registration** — annotation-validated at startup; CI conformance test in module-template



#### Spec version policy

  * **Current:** 2025-11-25 (production-stable as of May 2026)
  * **Previous:** 2025-06-18 (one phase grace period after spec bump)
  * **Tested clients:** Claude Desktop, Claude Code, Cursor, Cline, Codex; all MCP 2025-11-25-compatible (~12+ clients in ecosystem)



6

## Tier 6 — AI Gateway (LiteLLM router)

**Pick:** LiteLLM with CyberOS middleware overlay.

One gateway, one cost ledger, one residency policy. Routes between Bedrock primary (Sonnet 4.6, Haiku 4.5), Anthropic ZDR fallback, OpenAI ZDR fallback. Detailed in the [Infrastructure page](<infrastructure.html#ai-gateway>). 

#### Why these providers

  * **Bedrock primary** — ZDR by default, regional Singapore endpoint for VN tenants, Anthropic Sonnet/Haiku available without contract overhead.
  * **Anthropic ZDR fallback** — direct API at Anthropic's _Zero Data Retention_ contract tier; bypasses Bedrock when AWS has regional issues.
  * **OpenAI ZDR fallback** — third tier; explicitly enabled per-tenant; covers Bedrock + Anthropic dual outage.
  * **No Gemini for now** — no Singapore ZDR endpoint at suitable contract; revisit when GCP Singapore Gemini Enterprise ships.



#### Cost ceiling

  * N(FR pending) — ≤ $150/mo internal LLM
  * N(FR pending) — ≤ $4/active user/mo at 50-tenant
  * Semantic + exact-prompt cache ≥ 30% hit rate
  * Self-hosted BGE-M3 embeddings on shared GPU node (~$80/mo for GPU)



7

## Tier 7 — Backend services (per-module subgraphs)

**Pick:** TypeScript (GraphQL Yoga) or Rust (async-graphql) per subgraph, choice owned by module owner.

Most subgraphs are TypeScript for developer ergonomics. The two performance-critical modules — memory (memory writer hot path) and Skill (Wasmtime runtime, capability broker) — are Rust. Module owners pick per module; the contract (Apollo Federation SDL) is identical regardless. 

#### TypeScript stack (default)

  * **Runtime:** Bun 1.2+ (faster startup, native TS/JS)
  * **GraphQL server:** GraphQL Yoga (urql-team-maintained, federation-aware)
  * **ORM:** Prisma 5 (the audit-event schema in is a Prisma model)
  * **Validation:** Zod + GraphQL codegen
  * **Testing:** Vitest + Playwright (component + e2e)



#### Rust stack (perf-critical)

  * **Runtime:** Tokio + axum
  * **GraphQL server:** async-graphql (federation v2 supported)
  * **ORM:** SQLx (compile-time-checked queries)
  * **Validation:** serde + msgspec for canonical-JSON ledger entries
  * **Testing:** cargo test + criterion benchmarks



8

## Tier 8 — Data layer (Postgres + extensions)

**Pick:** PostgreSQL 17 + pgvector HNSW + Apache AGE 1.5 + PGroonga.

One Postgres database per region, with extensions stacked: pgvector for vector search (HNSW index), Apache AGE for graph traversal (OpenCypher dialect), PGroonga for Vietnamese-tokenised lexical search. Per-module schema isolation; RLS on every tenant-keyed table. 

#### Why Postgres + pgvector vs separate vector DB?

  * **Joins.** CUO queries routinely combine a vector hit with structured filter (tenant_id, scope, classification). Vector-DB-only solutions (Pinecone, Weaviate) make this two-roundtrip.
  * **Transactional.** Embeddings must be written atomically with the source row. Two-system writes need 2PC; one Postgres avoids it.
  * **Cost.** Pinecone scales linearly with vectors; Postgres scales with hardware. At 1M chunks the cost flip favours Postgres.
  * **HNSW maturity.** pgvector 0.7+ HNSW index now matches Pinecone recall on MIRACL benchmarks for VN content.



#### RLS posture

  * **Every tenant-keyed table** has `RLS ENABLED` \+ `FORCE ROW LEVEL SECURITY`.
  * **Session GUC:** `SET LOCAL app.tenant_id = $1` at session start.
  * **Bypass:** only the migration runner has `BYPASSRLS`; standard app role does not.
  * **Audit:** RLS policy violations are blocked at DB level (logged + alerted).



9

## Tier 9 — Search & embeddings (self-hosted)

**Pick:** BAAI/bge-m3 (embedder) + BAAI/bge-reranker-v2-m3 (reranker), self-hosted on one shared GPU node.

BGE-M3 produces 1024-dim dense + sparse + multi-vector embeddings in one pass. Multilingual native (top MIRACL Vietnamese scores). The reranker re-orders top-150 hits to top-20 using cross-encoder scoring. Both run on a single shared GPU node (Hetzner CCX23 + RTX A4000 or similar; ~$80/mo). 

#### Why self-hosted vs OpenAI text-embedding-3-large?

  * **Cost.** OpenAI embeddings at scale (50-tenant × 100k chunks/tenant × 30 reembeds/year) dominate the LLM bill. Self-hosted GPU is a fixed $80/mo.
  * **Latency.** Sub-30ms p50 vs ~150ms via OpenAI (network + queue).
  * **Vietnamese quality.** BGE-M3 outscores OpenAI on MIRACL-VI (Vietnamese subset) by ~8 points.
  * **Residency.** No data leaves CyberOS-controlled hardware. Compliance Q&A answers itself.



#### NFR ceilings

  * N(FR pending) — memory search ≤ 250ms p95 on 1M chunks
  * Embed p95 ≤ 80ms; rerank p95 ≤ 200ms
  * End-to-end retrieve: embed + pgvector + rerank ≤ 250ms p95



10

## Tier 10 — Event bus (NATS JetStream)

**Pick:** NATS Server 2.10+ with JetStream durable consumers.

Detailed in the [Infrastructure page](<infrastructure.html#nats-jetstream>). Choice driven by subject-hierarchy native fit (`cyberos.{tenant}.{module}.{entity}.{verb}`), sub-millisecond latency, and single-binary operational footprint. 

#### Why NATS, not Kafka / Redpanda?

See the alternatives table at §Alternatives considered for the full comparison.

11

## Tier 11 — Object storage (S3-compatible)

**Pick:** S3-compatible — Cloudflare R2 (zero-egress) or MinIO (self-host) per environment.

DEC-005 locks the choice to S3-compatible _protocol_ , not specific vendor. Production internal uses Cloudflare R2 (zero egress fee, global CDN). Self-hosted demo / on-prem tenant uses MinIO. Migration is a config flip. 

R2 cost

$0 egress

$0.015/GB-month storage

MinIO

self-host

Apache-2; single binary

Use cases

5+

memory archival, OBS logs, INV PDFs, ESOP docs, DOC signed

12

## Tier 12 — Realtime sync (CRDTs)

**Pick:** Yjs (CHAT, collaborative docs) + Automerge (offline-first complex models).

Yjs is the production-grade CRDT lib for text + lists (rich-text CHAT messages, KB docs). Automerge owns the offline-first model surface for clients that need to edit while disconnected (Tauri desktop ⇄ web). Both speak similar BinaryDoc formats; conversion when needed. 

#### Where they're used

  * **CHAT** — rich-text message editing, threaded reply collaboration (Yjs)
  * **KB** — collaborative document editing (Yjs + Tiptap editor)
  * **PROJ** — task description rich-text + checklist (Yjs)
  * **Desktop offline** — Tauri-based offline edits sync via Automerge when reconnected
  * **memory cross-tenant import** — CRDT-style merge for `cyberos import` (memory module §14.2)



13

## Tier 13 — Cryptography

**Pick:** Ed25519 signatures + scrypt key-wrap + Merkle Mountain Range (MMR) + Signed Tree Heads (STH).

memory's audit ledger uses MMR for additive inclusion proofs. Each consolidation cycle signs a Tree Head with Ed25519. Signing keys are passphrase-wrapped via scrypt (P2 Stage 2). Detailed in [memory module page](<../modules/memory/index.html>). 

#### Primitive choice

  * **Ed25519, not RSA-4096** — 32-byte keys, deterministic, FIPS 186-5 approved.
  * **SHA-256, not BLAKE2** — universally available, audit-time discoverable.
  * **scrypt for key wrap** — memory-hard; deliberately expensive at unwrap time.
  * **MMR over Merkle tree** — supports unbounded appends without re-balancing; matches Certificate Transparency Log convention.



#### Where used

  * **memory audit chain** — every memory operation appends a leaf; STH signed per consolidation.
  * **AuditEvent** — per-scope Merkle chain; prevHash chained.
  * **DEC-019** — Merkle-chained audit log invariant.
  * **N(FR pending)** — memory signed-zip portability: Ed25519 sig + Merkle proof.



14

## Tier 14 — Audit ledger encoding

**Pick:** msgspec canonical-JSON + binlog framing (length + CRC32C + seq + ts + payload).

`msgspec` (Python; mirrored in Rust via custom serde) produces deterministic JSON (sorted keys, UTF-8 NFC, no insignificant whitespace) → meets RFC 8785 JCS. The binary frame header makes the ledger durable under partial-write conditions. 

#### Format spec (Memory AGENTS.md §6.2)
    
    
    # Each ledger record frame:
    [u32 length BE][u32 crc32c BE][u64 seq BE][u64 ts_ns BE][payload]
    
    # Payload = msgspec canonical JSON of:
    {
     "seq": 12345,
     "ts_ns": 1715683200000000000,
     "tenant": "cyberskill",
     "actor": "member:trinh",
     "op": "put",
     "path": "memories/decisions/...",
     "body_hash": "sha256:abc...",
     "prev_chain": "sha256:xyz...",
     "chain": "sha256:def..." # SHA-256(canonical(record_minus_chain) || prev_chain)
    }

15

## Tier 15 — Compliance tooling

**Pick:** OPA (Open Policy Agent) + Conftest + Trust Center (static site).

OPA enforces Rego policies across Kubernetes manifests, GraphQL operation directives, and IAM transitions. Conftest runs OPA in CI for declarative-file validation. Trust Center is a static site (Astro + MDX) hosting VPAT, SOC 2, ISO 27001, CSA STAR docs. 

#### What OPA enforces

  * **K8s admission** — block deploys missing tenant labels, resource limits, network policies
  * **GraphQL directives** — every @sensitive field must have classification
  * **IAM transitions** — Founder role grants require dual-approval workflow
  * **MCP tool annotations** — destructive tool registration validation



#### Trust Center stack

  * **Astro** — static site generator with MDX support
  * **Auth** — NDA click-wrap before downloading reports (AUTH module integration)
  * **Signed URLs** — 24-hour TTL via R2 presigned URLs
  * **Audit** — every download logged for N(FR pending) compliance



16

## Tier 16 — Typography & design tokens

**Pick:** Be Vietnam Pro (UI) + JetMemorys Mono (code) + CyberSkill Global Design System v1.0.0.

Be Vietnam Pro is the diacritic-aware Vietnamese-first typeface; the Design System Part 5 specifies stack-fidelity (N(FR pending)). Tokens are exported in W3C DTCG format (2025.10) for cross-platform consumption (Style Dictionary, Tailwind via PostCSS plugin, iOS/Android, Figma). 

#### Token surface

  * **Anchors** — Umber #45210E, Ochre #F4BA17, sub-brand accents
  * **Typography** — Be Vietnam Pro (UI), JetMemorys Mono (code)
  * **Spacing rhythm** — 4px base; powers-of-two scale
  * **Genie token set** — dedicated panel/chip/mode-indicator tokens; versioned alongside CUO persona



17

## "What calls what" — dependency graph

The tiers compose left-to-right: every request from a user or agent flows through this graph. Cycles are forbidden by design. 

One request traverses every tier — sequence view

sequenceDiagram autonumber actor U as User participant T2 as T2 Host shell participant T3 as T3 Module remote participant T4 as T4 Apollo Router participant T7 as T7 Subgraph (Bun/Tokio) participant T8 as T8 Postgres + ext participant T6 as T6 AI Gateway participant T9 as T9 BGE-M3 (GPU) participant T10 as T10 NATS participant T11 as T11 R2 participant T14 as T14 Audit ledger U->>T2: open route T2->>T3: lazy-load remote T3->>T4: persisted query hash T4->>T7: federated query plan T7->>T8: SELECT (RLS-scoped) T8-->>T7: rows T7->>T6: POST /v1/embeddings T6->>T9: BGE-M3 self-hosted T9-->>T6: vector T6-->>T7: embed T7->>T8: SELECT pgvector T8-->>T7: hits T7->>T10: publish event T7->>T11: write attachment (if any) T7->>T14: append audit row (msgspec canonical) T7-->>T4: response T4-->>T3: composed result T3-->>T2: render T2-->>U: paint 

flowchart LR USER[("User · Agent")] --> HOST["Host shell  
Vite + React 19"] HOST --> REMOTE["Module remote  
Webpack 5 + MF"] REMOTE --> APOLLO["Apollo Router"] APOLLO --> AUTH["AUTH JWKS"] APOLLO --> SUBG["Subgraph (TS/Rust)"] SUBG --> PG[("Postgres 17 + ext.")] SUBG --> AIGW["AI Gateway"] SUBG --> NATS_DEP[("NATS JetStream")] SUBG --> S3_DEP[("R2 / MinIO")] AIGW --> LL["LiteLLM"] LL --> BEDROCK["AWS Bedrock"] LL --> ANT["Anthropic ZDR"] LL --> OAI["OpenAI ZDR"] LL --> BGE["BGE-M3 (self-hosted GPU)"] USER --> MCPCLT[("MCP client  
Claude / Cursor")] MCPCLT --> MCPGW["MCP Gateway"] MCPGW --> AUTH MCPGW --> SUBG SUBG -. trace.-> OBS["OBS · OTel"] APOLLO -. trace.-> OBS AIGW -. trace.-> OBS MCPGW -. trace.-> OBS classDef u fill:#fef6e0,stroke:#9c750a classDef fe fill:#e8d4c2,stroke:#45210e classDef gw fill:#f9c64f,stroke:#9c750a classDef be fill:#f5ede6,stroke:#45210e classDef data fill:#cba88a,stroke:#45210e classDef ext fill:#fde7b3,stroke:#9c750a class USER,MCPCLT u class HOST,REMOTE fe class APOLLO,MCPGW,AIGW,AUTH gw class SUBG be class PG,NATS_DEP,S3_DEP data class LL,BEDROCK,ANT,OAI,BGE,OBS ext 

18

## Cost-vs-tier model

Two reference scales — 10 Members internal (P0–P2) and 50 tenants (P4 GA). Each tier's contribution maps to a hard NFR ceiling. 

Cost flow — where the dollar goes at internal scale

flowchart LR BUDGET[("$535/mo  
N(FR pending) envelope")] --> LLM["28% · LLM  
$150 · primarily Sonnet + Haiku via Bedrock"] BUDGET --> COMPUTE["17% · K8s compute  
$90 · 22 subgraphs + gateways"] BUDGET --> PG["15% · Postgres  
$80 · primary + read replica"] BUDGET --> OBS_C["15% · OBS (LGTM)  
$80 · Loki + Tempo + Mimir + Grafana"] BUDGET --> GPU["15% · GPU embed  
$80 · shared BGE-M3 node"] BUDGET --> STORE["5% · object storage  
$25 · R2 zero-egress"] BUDGET --> NATS_C["4% · NATS  
$20 · single-node JetStream"] BUDGET --> EDGE["1% · CDN + auth  
$10"] classDef envelope fill:#fef6e0,stroke:#9c750a,stroke-width:2px classDef llm fill:#f9c64f,stroke:#9c750a classDef compute fill:#f5ede6,stroke:#45210e classDef data fill:#e8d4c2,stroke:#45210e classDef obs fill:#fde7b3,stroke:#9c750a classDef ext fill:#cba88a,stroke:#45210e class BUDGET envelope class LLM,GPU llm class COMPUTE compute class PG,STORE data class OBS_C obs class NATS_C,EDGE ext 

xychart-beta title "Monthly cost per tier ($USD)" x-axis ["LLM (T1+T6)", "Postgres (T8)", "Compute (T7)", "Storage (T11)", "OBS (LGTM)", "NATS (T10)", "AUTH (T4-side)", "Embeddings GPU (T9)"] y-axis "USD per month" 0 --> 600 bar [150, 80, 90, 25, 80, 20, 5, 80] 

Internal scale (10 Members) — total ≤ $530/mo against N(FR pending) budget of $530/mo ($150 LLM + $380 infra)

xychart-beta title "Cost shape at 50-tenant scale ($USD/month)" x-axis ["LLM", "Postgres (3 regions)", "Compute (k8s)", "Storage", "OBS", "NATS (cluster)", "AUTH", "GPU embed"] y-axis "USD per month" 0 --> 1400 bar [800, 600, 500, 200, 200, 100, 50, 200] 

50-tenant scale — total ≤ $2,650/mo against N(FR pending) budget of $2,200 + $4/user/mo LLM

### Per-tier production cost (P1 · exit, P4 · mid projections)

Tier| Pick| Internal (10 Members)| 50-tenant scale| Migration door  
---|---|---|---|---  
T1 Persona / Agent| LangGraph + LiteLLM| $0 host| $0 host| Replace LangGraph supervisor  
T2 Host shell| Vite + React 19 + Tauri| $5/mo CDN| $50/mo CDN| Switch host to Next.js  
T3 Module remotes| Webpack 5 + MF| included| included| Pin MF v2 spec  
T4 Apollo Router| Apollo Router| $0 (OSS binary)| $50/mo VM cluster| Elastic License v1.2 review  
T5 MCP Gateway| Custom router + per-module servers| $0 (in-cluster)| $30/mo| MCP spec preserves portability  
T6 AI Gateway| LiteLLM + Bedrock primary| $150/mo| $800/mo (+ per-user)| Provider mix via config  
T7 Backend| Bun / Tokio · 22 subgraphs| $90/mo k8s| $500/mo k8s| Containers, portable  
T8 Data| Postgres 17 + pgvector + AGE + PGroonga| $80/mo| $600/mo (3 regions)| SQL portable  
T9 Embeddings| BGE-M3 + reranker (GPU)| $80/mo| $200/mo (multi-GPU)| Switch to OpenAI text-embed  
T10 Event bus| NATS JetStream| $20/mo VM| $100/mo cluster| NATS subjects → Kafka topics  
T11 Object storage| R2 / MinIO| $25/mo| $200/mo| S3-compatible config flip  
T12 CRDT sync| Yjs / Automerge| $0 (libs)| $0 (libs)| Doc-format-portable  
T13–14 Cryptography + Ledger| Ed25519 + MMR + msgspec| $0| $0| Schema-portable  
T15 Compliance| OPA + Trust Center| $5/mo static host| $20/mo| OPA Rego portable  
OBS (LGTM)| Grafana / Loki / Tempo / Mimir| $80/mo| $200/mo| OTel-native; switch backend  
Total| —| ≤ $535/mo| ~$2,750/mo| —  
  
19

## Alternatives considered (per major pick)

Five major architectural picks deserve an explicit alternatives table. Each rejected option has a documented rejection rationale and a "would reconsider when..." trigger. 

### Postgres + pgvector — vs separate vector DB

Option| Pros| Cons| Status  
---|---|---|---  
**Postgres + pgvector HNSW**|  One DB; transactional embed-writes; structural joins; cheap; VN-tokenisation via PGroonga| Operational complexity (more extensions); ~10% slower than dedicated vector DB at very large scale| SELECTED  
Pinecone| Best-in-class recall; managed; horizontal scaling| Vendor lock; egress fees; no Singapore region; not transactional w/ source-of-truth| Rejected — sovereignty + cost  
Weaviate| OSS; multi-modal; GraphQL native| Memory-heavy; Janus runtime; embedded-mode not prod-grade for 1M+ chunks| Rejected — operational cost  
Qdrant| Rust-native; fast; OSS| Two-system writes still required; smaller community| Reconsider if pgvector p95 fails N(FR pending)  
Milvus / Zilliz| Scales to billions; cloud-native| K8s-heavy ops; same two-system issue| Out of scope for 10–50 tenant scale  
  
### Apollo Federation v2 — vs REST / gRPC / single GraphQL

Option| Pros| Cons| Status  
---|---|---|---  
**Apollo Federation v2.5+**|  Per-module subgraph ownership; single agent surface; persisted query budget; query plan cache| Apollo Router Elastic License (non-OSI); Rust expertise to extend| SELECTED  
REST per module| Universal; cacheable| N round-trips per page; no agent-friendly introspection; N OpenAPIs to maintain| Rejected — agent ergonomics  
gRPC + ConnectRPC| Strongly typed; fast; protobuf schema| Browser story still weak; no native cross-subgraph composition; agents don't speak gRPC natively| Rejected — frontend friction  
Single monolithic GraphQL| Single schema| Merge conflicts every PR; team coupling; deploy-coupling| Rejected — team scale  
tRPC| Excellent DX; TS-end-to-end| TS-only; no agent-facing surface; per-module schemas don't compose| Rejected — language lock  
  
### LangGraph — vs DSPy / native LangChain / Semantic Kernel

Option| Pros| Cons| Status  
---|---|---|---  
**LangGraph**|  StateGraph native; `interrupt` HITL; checkpointer for resumability; LangSmith tracing| Python-only; ties to LangChain ecosystem| SELECTED  
DSPy| Optimisation-first; auto-prompting| Not a router framework; long-lived agent loop awkward| Reconsider for batch evals only  
Native LangChain agents| Mature; vast tool catalog| Imperative loop; hard to audit; HITL via callbacks is brittle| Rejected — auditability  
Semantic Kernel| C# / Python native; Microsoft-backed| Smaller community; MS ecosystem bias| Rejected — community size  
CrewAI| Multi-agent ergonomics| Less mature; HITL gates not first-class| Watching  
  
### NATS JetStream — vs Kafka / Redpanda

Option| Pros| Cons| Status  
---|---|---|---  
**NATS JetStream**|  Subject hierarchy native; sub-ms latency; single 50MB binary; runs on $20/mo VM at internal scale| Smaller community than Kafka; less tooling around DLQ replay| SELECTED  
Apache Kafka| Industry standard; massive tooling; Confluent SaaS available| JVM ops complexity; flat topics; Zookeeper/Kraft cluster mandatory; expensive at 10-Member scale| Rejected — footprint  
Redpanda| Kafka-protocol-compatible; Rust; lower ops cost| Same flat-topic model; less mature than Kafka| Reconsider if Kafka-tooling needed  
AWS SQS / EventBridge| Managed; pay-per-message| Vendor lock; no subject hierarchy; latency 30-100ms typical| Rejected — sovereignty  
Apache Pulsar| Multi-tenant native; geo-replication| Operational complexity (BookKeeper); overkill at our scale| Out of scope  
  
### Tauri — vs Electron / Wails / Native

Option| Pros| Cons| Status  
---|---|---|---  
**Tauri 2**|  3-10MB bundle (vs Electron's 100+); Rust backend; OS webview; passes Apple notarisation by default| OS-native webview means CSS testing matrix; Rust IPC layer to learn| SELECTED  
Electron| Mature; Chrome consistency; vast plugin ecosystem| 100+ MB bundle; memory-hungry; security-update treadmill| Rejected — bundle size  
Wails (Go backend)| Go single-binary feel; webview-based| Smaller community; v3 still maturing| Reconsider  
Native (Swift / WinUI / GTK)| Best UX; smallest bundles| 3× the code; can't reuse React components| Rejected — scope  
Web-only (PWA)| No native ship| No file-system access; no native notification UX| Reconsider at P3 mobile evaluation  
  
[ Back to home ](<../index.html>) [ Next · Milestones  ](<milestones.html>)
