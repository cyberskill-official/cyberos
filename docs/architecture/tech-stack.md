---
title: Tech stack
source: website/docs/architecture/tech-stack.html
migrated: FR-DOCS-002
---

## Eight tiers, one supergraph

CyberOS has more than 8 distinct concerns (16 are enumerated below), but they cluster cleanly into 8 architectural tiers. Each tier is a separate deployment unit, scales independently, and exposes a stable contract to the tier above.

| Cluster | Concerns | Components |
|---|---|---|
| Persona / agent layer | T1 | LangGraph supervisor (StateGraph + interrupt); Anthropic Skills format (47 C-suite persona workflows, hot-reload); LiteLLM client (routing core) |
| Frontend layer | T2, T3 | Host shell (Vite + React 19 + Tauri desktop); module remotes (Webpack 5 + Module Federation) |
| API + agent surface | T4, T5, T6 | Apollo Router (GraphQL Federation v2.5+); MCP Gateway (Streamable HTTP, 2025-11-25); AI Gateway (LiteLLM router, Bedrock primary) |
| Backend services | T7 | 22 subgraphs (TypeScript Yoga or Rust async-graphql); 22 MCP servers (per-module, TS SDK or mcp-rs) |
| Data + search | T8, T9 | PostgreSQL 17 + pgvector HNSW + Apache AGE 1.5 + PGroonga; BGE-M3 embedder + BGE-rerank-v2-m3 (self-hosted) |
| Infrastructure | T10, T11 | NATS JetStream (event spine); S3 / R2 / MinIO (object storage) |
| Cryptography + sync | T12, T13, T14 | Yjs / Automerge (CRDTs for realtime); Ed25519 + scrypt key wrap + MMR + STH; msgspec canonical JSON + binlog framing |
| Compliance + UX | T15, T16 | OPA + Conftest (policy); Trust Center (cert hosting); Be Vietnam Pro + CyberSkill design system |

### Three design constraints driving every pick

1. **Vietnamese data sovereignty** - no SaaS dependency where Vietnamese-origin personal data must travel through a US-based vendor's servers. AWS Bedrock is acceptable because of the ap-southeast-1 region; OpenAI direct is not (no Singapore endpoint).
2. **Cost ceiling at scale** - <= $150/mo LLM + $230/mo infra at 10-Member internal; <= $4/active user/mo LLM + $2,200/mo infra at 50-tenant (N(FR pending), N(FR pending)). Anything that does not fit that envelope is rejected.
3. **Migration door always open** - every pick has a documented escape hatch. Storage is S3-compatible (DEC-005), so R2 <-> MinIO <-> AWS S3 is a config flip. SQL is portable, the audit chain is exportable, MCP servers are spec-conforming.

## Tier 1 - Persona / agent layer

Pick: LangGraph (state graph) + LiteLLM (routing) + Anthropic Skills format (47 C-suite persona workflows, hot-reloadable).

LangGraph is the agentic-supervisor framework. Its `StateGraph` primitive models CUO as a graph of nodes (router, skill-load, tool-call, HITL-confirm, answer-compose) with first-class `interrupt` support for human-in-the-loop gates on destructive tools. LiteLLM is the model-routing core that owns provider failover and prompt caching. The Anthropic Skills format (`SKILL.md` + `scripts/` + `references/`) keeps each C-level skill as a hot-reloadable directory.

#### Why these picks

- LangGraph, not DSPy / native LangChain: StateGraph models CUO's interrupt/resume semantics natively; DSPy is optimisation-first (signature -> optimiser pipeline) and not a fit for a long-lived router; native LangChain agents are imperative and harder to audit.
- LiteLLM: MIT-licensed, OpenAI-shaped surface, 100+ providers behind a single API, drop-in OTel hooks.
- Anthropic Skills format: open standard (Anthropic + VS Code + Cursor + Codex compat); hot-reload via `watchexec`; metadata-first dispatch keeps the context window lean.

#### Trade-offs

- LangGraph cost: Python-only (no Rust); requires an LLM API call per supervisor decision (mitigated by Haiku-tier routing).
- Skill format: not yet a published RFC; depends on Anthropic continuing to evolve it. Mitigation: schema-pinned to the 2025-04 spec; conformance tests in CI.
- LiteLLM fork risk: CyberOS-specific middleware (Presidio redaction, VN identifier rules, persona stamping) lives as a fork branch; rebases monthly.

#### Production cost

<= $0 host cost (runs inside the CUO service Pod). LLM cost flows through the Tier 6 AI Gateway.

## Tier 2 - Frontend host shell

Pick: Vite + React 19 + Tauri (desktop bundling).

The host shell is the thin orchestrator that hosts Module-Federation remotes. Vite (Rollup-based) for dev-server speed; React 19 for the new `use` hook and React Compiler's automatic memoisation; Tauri for the desktop bundling story.

#### Why these picks

- Vite: sub-second HMR; ESM-native; Rollup-based prod build; the Module Federation plugin (`@module-federation/vite`) stable in late 2025.
- React 19: React Compiler removes most `useMemo`/`useCallback` ceremony; the `use` hook simplifies suspense; concurrent rendering is GA.
- Tauri 2 (Rust): 3-10 MB bundle vs Electron's 100+ MB; uses the OS webview; passes Apple notarisation by default.

#### Trade-offs

- Webview consistency: Tauri uses OS-native webviews (Edge WebView2, WKWebView, WebKitGTK), so CSS feature parity needs CI matrix testing.
- Module Federation + Vite: the official plugin is newer than Webpack's; some plugins lag behind. Mitigation: the host shell is built around the official plugin's stable subset only.
- React 19 ecosystem: some libraries (charts, drag-drop) still pin React 18; monitor and pin as needed.

## Tier 3 - Frontend remotes (per module)

Pick: Webpack 5 + Module Federation v2 per module.

Each module ships as an MF remote bundle. The host shell lazy-loads on route entry. CSS is scoped via CSS Modules to prevent cross-module collisions. Design tokens come from one published package (`@cyberskill/tokens`).

#### Why Webpack for remotes, Vite for the host?

- Module Federation maturity: Webpack's MF plugin has 4+ years of production miles; remote bundling, runtime versioning, and shared-deps resolution are battle-tested.
- Bidirectional remotes: Webpack MF supports remote-as-host (a module can host its own sub-remotes); useful for SKILL -> CUO interactions.
- The host can be Vite: the Vite MF plugin can consume Webpack remotes; the inverse is not always true.

#### NFR ceilings

- N(FR pending) - module first-paint <= 1.5 s on cold load
- N(FR pending) - initial module bundle <= 50 KB gzipped JS
- N(FR pending) - module-rebuild time on token change <= 30 min including tests

## Tier 4 - API gateway (GraphQL Federation)

Pick: Apollo Router (Rust) - Federation v2.5+ compliant.

The Rust-based Apollo Router executes the composed supergraph plan. It validates JWTs, attaches tenant + actor context, runs the persisted-query lookup, and dispatches the subgraph fanout in parallel. Detailed on the [Infrastructure page](infrastructure.html#graphql-federation).

#### Why Apollo Router, not Mesh / Hasura / Yoga?

- Apollo Router: production-grade Rust runtime; the reference implementation for Federation v2.5; query plan cache; the persisted-query story is first-class.
- GraphQL Mesh: flexible but the wrong abstraction for federation (REST/SOAP/SQL adapters; not a router).
- Hasura: Postgres-first; does not model multi-subgraph federation; vendor lock-in concerns.
- GraphQL Yoga (as router): Yoga is great as a subgraph server, not as a federation router.

#### Trade-offs

- Elastic License (v1.2): non-OSI but production-friendly; review legal before a commercial offering.
- Single binary: Rust-only; tweaks require Rust expertise (or YAML config + Rhai script).
- Telemetry surface: needs the OTel collector + Grafana to be useful (already in OBS).

## Tier 5 - MCP Gateway (per-module servers)

Pick: per-module MCP servers + federation router; 2025-11-25 spec.

Each module owns its MCP server. The gateway is a federation router, not a monolith. Streamable HTTP, OAuth-PRM, well-known discovery, tool annotations. Detailed on the [Infrastructure page](infrastructure.html#mcp-gateway).

#### SDK choice

- TypeScript: official `@modelcontextprotocol/sdk` (1.20+) - used by 19 of 23 modules
- Rust: CyberSkill-published `mcp-rs` - used by memory and Skill (the two Rust-first modules)
- Tool registration: annotation-validated at startup; CI conformance test in the module template

#### Spec version policy

- Current: 2025-11-25 (production-stable as of May 2026)
- Previous: 2025-06-18 (one phase grace period after a spec bump)
- Tested clients: Claude Desktop, Claude Code, Cursor, Cline, Codex; all MCP 2025-11-25-compatible (~12+ clients in the ecosystem)

## Tier 6 - AI Gateway (LiteLLM router)

Pick: LiteLLM with a CyberOS middleware overlay.

One gateway, one cost ledger, one residency policy. Routes between Bedrock primary (Sonnet 4.6, Haiku 4.5), Anthropic ZDR fallback, OpenAI ZDR fallback. Detailed on the [Infrastructure page](infrastructure.html#ai-gateway).

#### Why these providers

- Bedrock primary: ZDR by default, regional Singapore endpoint for VN tenants, Anthropic Sonnet/Haiku available without contract overhead.
- Anthropic ZDR fallback: direct API at Anthropic's Zero Data Retention contract tier; bypasses Bedrock when AWS has regional issues.
- OpenAI ZDR fallback: third tier; explicitly enabled per-tenant; covers a Bedrock + Anthropic dual outage.
- No Gemini for now: no Singapore ZDR endpoint at a suitable contract; revisit when GCP Singapore Gemini Enterprise ships.

#### Cost ceiling

- N(FR pending) - <= $150/mo internal LLM
- N(FR pending) - <= $4/active user/mo at 50-tenant
- Semantic + exact-prompt cache >= 30% hit rate
- Self-hosted BGE-M3 embeddings on a shared GPU node (~$80/mo for GPU)

## Tier 7 - Backend services (per-module subgraphs)

Pick: TypeScript (GraphQL Yoga) or Rust (async-graphql) per subgraph; the choice is owned by the module owner.

Most subgraphs are TypeScript for developer ergonomics. The two performance-critical modules - memory (memory writer hot path) and Skill (Wasmtime runtime, capability broker) - are Rust. Module owners pick per module; the contract (Apollo Federation SDL) is identical regardless.

#### TypeScript stack (default)

- Runtime: Bun 1.2+ (faster startup, native TS/JS)
- GraphQL server: GraphQL Yoga (urql-team-maintained, federation-aware)
- ORM: Prisma 5 (the audit-event schema is a Prisma model)
- Validation: Zod + GraphQL codegen
- Testing: Vitest + Playwright (component + e2e)

#### Rust stack (perf-critical)

- Runtime: Tokio + axum
- GraphQL server: async-graphql (federation v2 supported)
- ORM: SQLx (compile-time-checked queries)
- Validation: serde + msgspec for canonical-JSON ledger entries
- Testing: cargo test + criterion benchmarks

## Tier 8 - Data layer (Postgres + extensions)

Pick: PostgreSQL 17 + pgvector HNSW + Apache AGE 1.5 + PGroonga.

One Postgres database per region, with extensions stacked: pgvector for vector search (HNSW index), Apache AGE for graph traversal (OpenCypher dialect), PGroonga for Vietnamese-tokenised lexical search. Per-module schema isolation; RLS on every tenant-keyed table.

#### Why Postgres + pgvector vs a separate vector DB?

- Joins. CUO queries routinely combine a vector hit with a structured filter (tenant_id, scope, classification). Vector-DB-only solutions (Pinecone, Weaviate) make this two roundtrips.
- Transactional. Embeddings must be written atomically with the source row. Two-system writes need 2PC; one Postgres avoids it.
- Cost. Pinecone scales linearly with vectors; Postgres scales with hardware. At 1M chunks the cost flip favours Postgres.
- HNSW maturity. pgvector 0.7+ HNSW now matches Pinecone recall on MIRACL benchmarks for VN content.

#### RLS posture

- Every tenant-keyed table has `RLS ENABLED` + `FORCE ROW LEVEL SECURITY`.
- Session GUC: `SET LOCAL app.tenant_id = $1` at session start.
- Bypass: only the migration runner has `BYPASSRLS`; the standard app role does not.
- Audit: RLS policy violations are blocked at DB level (logged + alerted).

## Tier 9 - Search and embeddings (self-hosted)

Pick: BAAI/bge-m3 (embedder) + BAAI/bge-reranker-v2-m3 (reranker), self-hosted on one shared GPU node.

BGE-M3 produces 1024-dim dense + sparse + multi-vector embeddings in one pass. Multilingual native (top MIRACL Vietnamese scores). The reranker re-orders the top-150 hits to a top-20 using cross-encoder scoring. Both run on a single shared GPU node (Hetzner CCX23 + RTX A4000 or similar; ~$80/mo).

#### Why self-hosted vs OpenAI text-embedding-3-large?

- Cost. OpenAI embeddings at scale (50 tenants x 100k chunks/tenant x 30 reembeds/year) dominate the LLM bill. A self-hosted GPU is a fixed $80/mo.
- Latency. Sub-30 ms p50 vs ~150 ms via OpenAI (network + queue).
- Vietnamese quality. BGE-M3 outscores OpenAI on MIRACL-VI (Vietnamese subset) by ~8 points.
- Residency. No data leaves CyberOS-controlled hardware. The compliance Q&A answers itself.

#### NFR ceilings

- N(FR pending) - memory search <= 250 ms p95 on 1M chunks
- Embed p95 <= 80 ms; rerank p95 <= 200 ms
- End-to-end retrieve: embed + pgvector + rerank <= 250 ms p95

## Tier 10 - Event bus (NATS JetStream)

Pick: NATS Server 2.10+ with JetStream durable consumers.

Detailed on the [Infrastructure page](infrastructure.html#nats-jetstream). The choice is driven by the subject-hierarchy-native fit (`cyberos.{tenant}.{module}.{entity}.{verb}`), sub-millisecond latency, and the single-binary operational footprint.

#### Why NATS, not Kafka / Redpanda?

See the alternatives tables below for the full comparison.

## Tier 11 - Object storage (S3-compatible)

Pick: S3-compatible - Cloudflare R2 (zero-egress) or MinIO (self-host) per environment.

DEC-005 locks the choice to the S3-compatible protocol, not a specific vendor. Production internal uses Cloudflare R2 (zero egress fee, global CDN). Self-hosted demo / on-prem tenants use MinIO. Migration is a config flip.

- R2 cost: $0 egress; $0.015/GB-month storage
- MinIO: self-host; Apache-2 licensed; single binary
- Use cases (5+): memory archival, OBS logs, INV PDFs, ESOP docs, DOC signed documents

## Tier 12 - Realtime sync (CRDTs)

Pick: Yjs (CHAT, collaborative docs) + Automerge (offline-first complex models).

Yjs is the production-grade CRDT lib for text + lists (rich-text CHAT messages, KB docs). Automerge owns the offline-first model surface for clients that need to edit while disconnected (Tauri desktop <-> web). Both speak similar BinaryDoc formats; conversion when needed.

#### Where they are used

- CHAT - rich-text message editing, threaded reply collaboration (Yjs)
- KB - collaborative document editing (Yjs + Tiptap editor)
- PROJ - task description rich-text + checklists (Yjs)
- Desktop offline - Tauri-based offline edits sync via Automerge when reconnected
- memory cross-tenant import - CRDT-style merge for `cyberos import` (memory module, section 14.2)

## Tier 13 - Cryptography

Pick: Ed25519 signatures + scrypt key-wrap + Merkle Mountain Range (MMR) + Signed Tree Heads (STH).

memory's audit ledger uses an MMR for additive inclusion proofs. Each consolidation cycle signs a Tree Head with Ed25519. Signing keys are passphrase-wrapped via scrypt (P2 Stage 2). Detailed on the [memory module page](../modules/memory/index.html).

#### Primitive choice

- Ed25519, not RSA-4096: 32-byte keys, deterministic, FIPS 186-5 approved.
- SHA-256, not BLAKE2: universally available, audit-time discoverable.
- scrypt for key wrap: memory-hard; deliberately expensive at unwrap time.
- MMR over a Merkle tree: supports unbounded appends without re-balancing; matches the Certificate Transparency Log convention.

#### Where used

- memory audit chain - every memory operation appends a leaf; STH signed per consolidation.
- AuditEvent - per-scope Merkle chain; prevHash chained.
- DEC-019 - Merkle-chained audit log invariant.
- N(FR pending) - memory signed-zip portability: Ed25519 sig + Merkle proof.

## Tier 14 - Audit ledger encoding

Pick: msgspec canonical-JSON + binlog framing (length + CRC32C + seq + ts + payload).

`msgspec` (Python; mirrored in Rust via custom serde) produces deterministic JSON (sorted keys, UTF-8 NFC, no insignificant whitespace), meeting RFC 8785 JCS. The binary frame header makes the ledger durable under partial-write conditions.

#### Format spec (Memory AGENTS.md section 6.2)

```
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
  "chain": "sha256:def..."  # SHA-256(canonical(record_minus_chain) || prev_chain)
}
```

## Tier 15 - Compliance tooling

Pick: OPA (Open Policy Agent) + Conftest + Trust Center (static site).

OPA enforces Rego policies across Kubernetes manifests, GraphQL operation directives, and IAM transitions. Conftest runs OPA in CI for declarative-file validation. The Trust Center is a static site (Astro + MDX) hosting VPAT, SOC 2, ISO 27001, and CSA STAR docs.

#### What OPA enforces

- K8s admission - block deploys missing tenant labels, resource limits, network policies
- GraphQL directives - every @sensitive field must have a classification
- IAM transitions - Founder role grants require a dual-approval workflow
- MCP tool annotations - destructive tool registration validation

#### Trust Center stack

- Astro - static site generator with MDX support
- Auth - NDA click-wrap before downloading reports (AUTH module integration)
- Signed URLs - 24-hour TTL via R2 presigned URLs
- Audit - every download logged for N(FR pending) compliance

## Tier 16 - Typography and design tokens

Pick: Be Vietnam Pro (UI) + JetBrains Mono (code) + CyberSkill Global Design System v1.0.0.

Be Vietnam Pro is the diacritic-aware, Vietnamese-first typeface; Design System Part 5 specifies stack-fidelity (N(FR pending)). Tokens are exported in W3C DTCG format (2025.10) for cross-platform consumption (Style Dictionary, Tailwind via a PostCSS plugin, iOS/Android, Figma).

#### Token surface

- Anchors - Umber #45210E, Ochre #F4BA17, sub-brand accents
- Typography - Be Vietnam Pro (UI), JetBrains Mono (code)
- Spacing rhythm - 4 px base; powers-of-two scale
- Genie token set - dedicated panel/chip/mode-indicator tokens; versioned alongside the CUO persona

## "What calls what" - dependency graph

The tiers compose left-to-right: every request from a user or agent flows through this graph. Cycles are forbidden by design.

One request traverses every tier:

1. The user opens a route; the T2 host shell lazy-loads the T3 module remote.
2. The remote posts a persisted-query hash to the T4 Apollo Router, which builds the federated query plan and dispatches it to the T7 subgraph (Bun/Tokio).
3. The subgraph SELECTs from T8 Postgres (RLS-scoped), calls the T6 AI Gateway (`POST /v1/embeddings`), gets the vector from T9 BGE-M3 (GPU), and runs the pgvector SELECT for hits.
4. The subgraph publishes an event to T10 NATS, writes any attachment to T11 R2, and appends an audit row (msgspec canonical) to the T14 ledger.
5. The response composes back through the Router to the remote; the host shell paints.

The same graph in prose: users and agents reach the host shell (Vite + React 19), which loads module remotes (Webpack 5 + MF); remotes call the Apollo Router, which validates against AUTH JWKS and fans out to subgraphs (TS/Rust); subgraphs use Postgres 17 + extensions, the AI Gateway (LiteLLM -> AWS Bedrock / Anthropic ZDR / OpenAI ZDR / self-hosted BGE-M3), NATS JetStream, and R2 / MinIO. MCP clients (Claude / Cursor) reach the same subgraphs through the MCP Gateway, which also authenticates via AUTH. Subgraphs, the Router, the AI Gateway, and the MCP Gateway all trace to OBS (OTel).

## Cost-vs-tier model

Two reference scales - 10 Members internal (P0-P2) and 50 tenants (P4 GA). Each tier's contribution maps to a hard NFR ceiling.

Where the dollar goes at internal scale (the $535/mo N(FR pending) envelope):

| Share | Line item | $/mo |
|---|---|---|
| 28% | LLM (primarily Sonnet + Haiku via Bedrock) | $150 |
| 17% | K8s compute (22 subgraphs + gateways) | $90 |
| 15% | Postgres (primary + read replica) | $80 |
| 15% | OBS (LGTM: Loki + Tempo + Mimir + Grafana) | $80 |
| 15% | GPU embed (shared BGE-M3 node) | $80 |
| 5% | Object storage (R2 zero-egress) | $25 |
| 4% | NATS (single-node JetStream) | $20 |
| 1% | CDN + auth | $10 |

Internal scale (10 Members): total <= $530/mo against the N(FR pending) budget of $530/mo ($150 LLM + $380 infra). 50-tenant scale: LLM $800, Postgres (3 regions) $600, compute (k8s) $500, storage $200, OBS $200, NATS (cluster) $100, AUTH $50, GPU embed $200 - total <= $2,650/mo against the N(FR pending) budget of $2,200/mo + $4/user/mo LLM.

### Per-tier production cost (P1 exit, P4 mid projections)

| Tier | Pick | Internal (10 Members) | 50-tenant scale | Migration door |
|---|---|---|---|---|
| T1 Persona / agent | LangGraph + LiteLLM | $0 host | $0 host | Replace the LangGraph supervisor |
| T2 Host shell | Vite + React 19 + Tauri | $5/mo CDN | $50/mo CDN | Switch host to Next.js |
| T3 Module remotes | Webpack 5 + MF | included | included | Pin the MF v2 spec |
| T4 Apollo Router | Apollo Router | $0 (OSS binary) | $50/mo VM cluster | Elastic License v1.2 review |
| T5 MCP Gateway | Custom router + per-module servers | $0 (in-cluster) | $30/mo | MCP spec preserves portability |
| T6 AI Gateway | LiteLLM + Bedrock primary | $150/mo | $800/mo (+ per-user) | Provider mix via config |
| T7 Backend | Bun / Tokio; 22 subgraphs | $90/mo k8s | $500/mo k8s | Containers, portable |
| T8 Data | Postgres 17 + pgvector + AGE + PGroonga | $80/mo | $600/mo (3 regions) | SQL portable |
| T9 Embeddings | BGE-M3 + reranker (GPU) | $80/mo | $200/mo (multi-GPU) | Switch to OpenAI text-embed |
| T10 Event bus | NATS JetStream | $20/mo VM | $100/mo cluster | NATS subjects -> Kafka topics |
| T11 Object storage | R2 / MinIO | $25/mo | $200/mo | S3-compatible config flip |
| T12 CRDT sync | Yjs / Automerge | $0 (libs) | $0 (libs) | Doc-format-portable |
| T13-14 Cryptography + ledger | Ed25519 + MMR + msgspec | $0 | $0 | Schema-portable |
| T15 Compliance | OPA + Trust Center | $5/mo static host | $20/mo | OPA Rego portable |
| OBS (LGTM) | Grafana / Loki / Tempo / Mimir | $80/mo | $200/mo | OTel-native; switch backend |
| Total | - | <= $535/mo | ~$2,750/mo | - |

## Alternatives considered (per major pick)

Five major architectural picks deserve an explicit alternatives table. Each rejected option has a documented rejection rationale and a "would reconsider when..." trigger.

### Postgres + pgvector - vs a separate vector DB

| Option | Pros | Cons | Status |
|---|---|---|---|
| Postgres + pgvector HNSW | One DB; transactional embed-writes; structural joins; cheap; VN tokenisation via PGroonga | Operational complexity (more extensions); ~10% slower than a dedicated vector DB at very large scale | SELECTED |
| Pinecone | Best-in-class recall; managed; horizontal scaling | Vendor lock; egress fees; no Singapore region; not transactional with the source of truth | Rejected - sovereignty + cost |
| Weaviate | OSS; multi-modal; GraphQL native | Memory-heavy; Janus runtime; embedded mode not prod-grade for 1M+ chunks | Rejected - operational cost |
| Qdrant | Rust-native; fast; OSS | Two-system writes still required; smaller community | Reconsider if pgvector p95 fails N(FR pending) |
| Milvus / Zilliz | Scales to billions; cloud-native | K8s-heavy ops; same two-system issue | Out of scope for 10-50 tenant scale |

### Apollo Federation v2 - vs REST / gRPC / single GraphQL

| Option | Pros | Cons | Status |
|---|---|---|---|
| Apollo Federation v2.5+ | Per-module subgraph ownership; single agent surface; persisted query budget; query plan cache | Apollo Router Elastic License (non-OSI); Rust expertise to extend | SELECTED |
| REST per module | Universal; cacheable | N round-trips per page; no agent-friendly introspection; N OpenAPIs to maintain | Rejected - agent ergonomics |
| gRPC + ConnectRPC | Strongly typed; fast; protobuf schema | Browser story still weak; no native cross-subgraph composition; agents don't speak gRPC natively | Rejected - frontend friction |
| Single monolithic GraphQL | Single schema | Merge conflicts every PR; team coupling; deploy coupling | Rejected - team scale |
| tRPC | Excellent DX; TS end-to-end | TS-only; no agent-facing surface; per-module schemas don't compose | Rejected - language lock |

### LangGraph - vs DSPy / native LangChain / Semantic Kernel

| Option | Pros | Cons | Status |
|---|---|---|---|
| LangGraph | StateGraph native; `interrupt` HITL; checkpointer for resumability; LangSmith tracing | Python-only; ties to the LangChain ecosystem | SELECTED |
| DSPy | Optimisation-first; auto-prompting | Not a router framework; long-lived agent loop awkward | Reconsider for batch evals only |
| Native LangChain agents | Mature; vast tool catalog | Imperative loop; hard to audit; HITL via callbacks is brittle | Rejected - auditability |
| Semantic Kernel | C# / Python native; Microsoft-backed | Smaller community; MS ecosystem bias | Rejected - community size |
| CrewAI | Multi-agent ergonomics | Less mature; HITL gates not first-class | Watching |

### NATS JetStream - vs Kafka / Redpanda

| Option | Pros | Cons | Status |
|---|---|---|---|
| NATS JetStream | Subject hierarchy native; sub-ms latency; single 50 MB binary; runs on a $20/mo VM at internal scale | Smaller community than Kafka; less tooling around DLQ replay | SELECTED |
| Apache Kafka | Industry standard; massive tooling; Confluent SaaS available | JVM ops complexity; flat topics; Zookeeper/Kraft cluster mandatory; expensive at 10-Member scale | Rejected - footprint |
| Redpanda | Kafka-protocol-compatible; Rust; lower ops cost | Same flat-topic model; less mature than Kafka | Reconsider if Kafka tooling needed |
| AWS SQS / EventBridge | Managed; pay-per-message | Vendor lock; no subject hierarchy; latency 30-100 ms typical | Rejected - sovereignty |
| Apache Pulsar | Multi-tenant native; geo-replication | Operational complexity (BookKeeper); overkill at this scale | Out of scope |

### Tauri - vs Electron / Wails / native

| Option | Pros | Cons | Status |
|---|---|---|---|
| Tauri 2 | 3-10 MB bundle (vs Electron's 100+); Rust backend; OS webview; passes Apple notarisation by default | OS-native webview means a CSS testing matrix; a Rust IPC layer to learn | SELECTED |
| Electron | Mature; Chrome consistency; vast plugin ecosystem | 100+ MB bundle; memory-hungry; security-update treadmill | Rejected - bundle size |
| Wails (Go backend) | Go single-binary feel; webview-based | Smaller community; v3 still maturing | Reconsider |
| Native (Swift / WinUI / GTK) | Best UX; smallest bundles | 3x the code; can't reuse React components | Rejected - scope |
| Web-only (PWA) | No native ship | No file-system access; no native notification UX | Reconsider at the P3 mobile evaluation |

## Changelog

History lives in the [changelog](../reference/changelog.html); this page describes only the current state.
