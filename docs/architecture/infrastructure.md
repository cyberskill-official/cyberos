---
title: Infrastructure — CyberOS
source: website/docs/architecture/infrastructure.html
migrated: FR-DOCS-002
---

0

## The six-pillar plane

The platform contract locks the high-level shape: every module is an independently deployable Apollo Federation subgraph and a Module-Federation frontend remote, sitting on top of six cross-cutting pillars. The pillars are _infrastructure modules_ — they are owned, versioned, and on-call exactly like the functional modules — but they are special in that every other module depends on them. Removing any one of the six breaks the platform. 

flowchart TB subgraph TOP ["End users · agents"] USER[("Member  
browser / Tauri")] AGENT[("Claude / Codex / Cursor  
via Skills + MCP")] end subgraph PLANE ["Cross-cutting infrastructure plane (P0)"] direction LR AUTH["🔐 AUTH  
OAuth 2.1 + JWT (RS256)  
per-tenant authz server"] AIGW["⚡ AI Gateway  
LiteLLM router  
Bedrock · Anthropic · OpenAI"] MCPGW["🔌 MCP Gateway  
2025-11-25 spec  
per-module servers"] OBS["👁 OBS  
LGTM stack  
Loki · Grafana · Tempo · Mimir"] GQL["🌐 Apollo Router  
Federation v2.5+  
persisted queries"] NATS_BUS["📬 NATS JetStream  
cyberos.{tenant}.{module}.{entity}.{verb}"] end subgraph MODS ["22 functional modules — subgraph + MCP + UI remote"] memory["🧠 memory"] CHAT["💬 CHAT"] PROJ["📋 PROJ"] OTHERS["…19 more"] end subgraph DATA ["Per-module data layer"] PG[("PostgreSQL 17  
per-module schema  
RLS enforced")] S3[("S3 / R2 / MinIO  
object store")] end USER --> GQL AGENT --> MCPGW GQL -- "auth context" --> AUTH GQL --> memory GQL --> CHAT GQL --> PROJ GQL --> OTHERS MCPGW --> memory MCPGW --> CHAT MCPGW --> PROJ memory --> AIGW CHAT --> AIGW PROJ --> AIGW memory --> PG CHAT --> PG PROJ --> PG memory --> S3 memory -. "emits".-> NATS_BUS CHAT -. "emits".-> NATS_BUS PROJ -. "emits".-> NATS_BUS memory -. "traces".-> OBS CHAT -. "traces".-> OBS PROJ -. "traces".-> OBS AIGW -. "traces".-> OBS MCPGW -. "traces".-> OBS GQL -. "traces".-> OBS classDef plane fill:#fef6e0,stroke:#9c750a,stroke-width:2px classDef mod fill:#e8d4c2,stroke:#45210e classDef ext fill:#f9c64f,stroke:#9c750a classDef data fill:#cba88a,stroke:#45210e class AUTH,AIGW,MCPGW,OBS,GQL,NATS_BUS plane class memory,CHAT,PROJ,OTHERS mod class USER,AGENT ext class PG,S3 data 

🪨

### Why a separate plane, not per-module?

  * **Audit unity.** Compliance regulators need one audit chain, not 22. NATS subjects, Merkle-chained audit rows, OAuth issuance all share one canonical surface.
  * **Cost shape.** Per-tenant LLM cost is the largest variable expense at scale; a single gateway lets the CFO put a ceiling on it (: ≤ $150/mo internal, ≤ $4/active user/mo at 50-tenant).
  * **Provider failover.** One gateway means primary Bedrock → Anthropic ZDR → OpenAI ZDR happens once, not 22 times in 22 different ways.
  * **Agent parity.** Strategic bet #1. A human's request and an agent's request hit the _same_ gateway, the _same_ RBAC, the _same_ audit row.



📜

### The contracts each pillar exposes

  * **AUTH** → JWT validation header, audience binding, RBAC predicate evaluator
  * **AI Gateway** → `POST /v1/chat/completions`, `/v1/embeddings`, `/v1/rerank` (OpenAI-shaped)
  * **MCP Gateway** → Streamable HTTP + `/.well-known/mcp` \+ per-tenant OAuth-PRM
  * **OBS** → OTel collector at `otel:4317` (gRPC) + `:4318` (HTTP)
  * **GraphQL** → composed supergraph at `https://{tenant}.cyberos.world/graphql`
  * **NATS** → subject `cyberos.{tenant}.{module}.{entity}.{verb}`



1

## AUTH

P0 · planned

🔐 The identity backbone. AUTH owns who you are; every other module trusts AUTH. The minimal interface is "give me an RS256-signed JWT for this Member" — the rest is implementation. 

### Why a separate layer

If each module re-implemented identity, the platform would suffer the classic distributed-monolith pathology: 22 places where a security advisory needs to be applied, 22 places where MFA enforcement can drift. AUTH centralises the surface that absolutely cannot drift.

More structural: mandates that **AI agents authenticate as Members** — there is no "service account with broad permissions" pattern. Agent parity ( bet #1) requires that a Claude session run on the same identity contract as a human session. One AUTH module is the only sensible place to enforce that.

#### Tech stack

  * **Library** · Hono + jose (TypeScript subgraph) or axum + jsonwebtoken (Rust subgraph)
  * **Token** · JWT RS256 with rotating signing keys (90-day rotation, 30-day grace)
  * **Session refresh** · opaque token, HttpOnly+SameSite=Lax cookie, 30-day max
  * **MFA** · TOTP (RFC 6238) minimum + WebAuthn / passkey for elevated roles
  * **Magic link** · onboarding only, single-use, 15-minute TTL
  * **OAuth 2.1 + PKCE** · per-tenant authorisation server (S256 challenge only)
  * **RBAC store** · Postgres with RLS-enforced `app.tenant_id` session GUC



#### Why these picks

  * **RS256, not HS256** · only AUTH needs the private key; every subgraph validates with the public key without round-tripping AUTH.
  * **Per-tenant authz server**`acme.cyberos.world/.well-known/oauth-authorization-server` means a leaked Acme token cannot replay against Beta's gateway (audience binding from §8.4.1).
  * **Postgres RLS** · not application-layer ACL. Even a bug in a subgraph cannot cross tenant boundaries because the database itself refuses to return another tenant's rows.
  * **WebAuthn for Founder/CEO** · phishing-resistant by spec; mandatory per.



AUTH internal architecture

flowchart TB subgraph AUTH_INT ["AUTH module"] direction TB OAUTH["OAuth 2.1 server  
per-tenant authz server"] JWTISSUER["JWT issuer  
RS256 · 24h expiry"] REFRESH["Refresh token store  
opaque tokens · cookie"] MFAVERIFY["MFA verifier  
TOTP + WebAuthn"] RBAC["RBAC predicate engine  
role × resource × action"] KEYS["Signing key rotation  
JWKS endpoint"] SESSIONDB[("Postgres  
sessions · refresh · MFA enrolment")] end USER[("Member browser")] --> OAUTH OAUTH --> MFAVERIFY MFAVERIFY --> JWTISSUER JWTISSUER --> KEYS JWTISSUER --> SESSIONDB REFRESH --> SESSIONDB OAUTH -.-> REFRESH SUBG[("Any subgraph")] -. "JWKS fetch · cache".-> KEYS SUBG -. "validates JWT  
checks RBAC predicate".-> RBAC classDef int fill:#e8d4c2,stroke:#45210e classDef ext fill:#f9c64f,stroke:#9c750a classDef db fill:#fde7b3,stroke:#9c750a class OAUTH,JWTISSUER,REFRESH,MFAVERIFY,RBAC,KEYS int class USER,SUBG ext class SESSIONDB db 

Flow · Member login with passkey + MFA-elevated step-up

sequenceDiagram autonumber actor M as Member participant H as Host shell participant AU as AUTH participant DB as Sessions DB participant SG as Any subgraph M->>H: Open acme.cyberos.world H->>AU: GET /.well-known/oauth-authorization-server AU-->>H: { issuer, authz_endpoint, token_endpoint, S256 only } H->>AU: GET /authorize?response_type=code&code;_challenge=... AU->>M: Passkey prompt (WebAuthn) M-->>AU: Signed assertion AU->>DB: Verify enrolment · increment session counter AU-->>H: 302 + code H->>AU: POST /token { code, code_verifier } AU-->>H: { jwt (RS256, 24h), refresh_token (cookie) } H->>SG: GraphQL query + Bearer jwt SG->>SG: Validate via cached JWKS · check RBAC predicate SG-->>H: 200 { data } Note over M,SG: Later — Founder triggers a destructive op H->>AU: POST /step-up { scope: "rew:write" } AU->>M: TOTP prompt M-->>AU: 6-digit code AU-->>H: { jwt with mfa_elevated=true, 15m TTL } 

### Key contracts
    
    
    # GraphQL contract — abbreviated
    type Query {
     me: Member!
     myRoles: [Role!]!
     jwks: JwksDocument! # public keys for downstream subgraphs
    }
    
    type Mutation {
     exchangeCodeForToken(code: ID!, verifier: String!): TokenBundle!
     refresh(token: String!): TokenBundle!
     enrollMfa(method: MfaMethod!): MfaEnrollment!
     stepUp(scope: String!, totp: String): TokenBundle!
     revokeSession(sessionId: ID!): Boolean!
    }
    
    type TokenBundle {
     jwt: String! # RS256, 24h expiry
     expiresAt: DateTime!
     refreshCookie: String # Set-Cookie header sent server-side
    }
    
    # MCP tool surface
    cyberos.auth.whoami # readOnly=true
    cyberos.auth.list_roles # readOnly=true
    cyberos.auth.audit_login # readOnly=true (own sessions only)

### Role catalogue 

Role| Reads| Writes| Sign / transfer| MFA  
---|---|---|---|---  
**Founder/CEO**|  all (own tenant)| all (own tenant)| all| passkey · mandatory  
**Engineering Lead**|  all (own tenant)| all except REW/ESOP/HR-comp| —| passkey · mandatory  
**HR/Ops Lead**|  HR / REW / LEARN| HR / REW / LEARN| HR docs| TOTP min  
**Account Manager**|  CRM / PROJ / TIME / INV / PORTAL| CRM / PROJ / TIME / INV / PORTAL| INV / DOC| TOTP min  
**Member**|  own + assigned + public| own + assigned| own time entries| recommended  
**Board Member**|  governance scope| limited sign-offs| SP valuation| passkey · mandatory  
**External Client (P4)**|  PORTAL scope| PORTAL scope| own docs only| recommended  
**Tenant Admin (P4)**|  tenant config + audit| tenant config| tenant agreements| passkey · mandatory  
**AI Agent (Member)**|  = Member| = Member| no auto-sign| inherited  
  
JWT verify p95

< 5 ms

JWKS cached in-subgraph

Login flow p95

< 600 ms

with passkey · excludes user time

Availability

≥ 99.95%

AUTH outage = platform outage

#### Status

P0 planned · P0 phase build window

  * OAuth 2.1 PRM compatible with §8.4.3 MCP discovery
  * JWKS endpoint live before any subgraph deploys
  * Per-tenant audz server provisioning automated at TEN-module setup (P4)



#### References

  * Internal spec — Authentication & RBAC
  * Internal spec — Role catalogue (technical detail)
  * Internal spec — OAuth 2.1 for MCP (audience binding)
  * RFC 7519 (JWT), RFC 7636 (PKCE), W3C WebAuthn L3



2

## AI Gateway

P0 · planned

⚡ One door for every LLM call. Routing, caching, redaction, persona stamping, cost accounting, residency enforcement, circuit breaking — all here, once. 

### Why a separate layer

The temptation in a 23-module platform is to let each module call the LLM SDK directly — "just `import anthropic` in the subgraph." That fails three ways. First, **cost** : without one place to set per-tenant budgets, the bill is unobservable until the credit-card statement arrives. Second, **residency** : a Vietnam-resident tenant must hit the Bedrock Singapore endpoint, never the US; the rule is too easy to bypass when 23 subgraphs each make their own choice. Third, **safety** : PII redaction, persona-version stamping, and the OWASP Gen-AI Top-10 mitigations cannot be 23-times-correct; they need one chokepoint.

makes the gateway non-optional: every LLM call from every module flows through it. The cost target — **≤ $150/mo internal, ≤ $4/active user/mo at 50-tenant scale** — only works because we can see and cap every token at the gateway.

#### Tech stack

  * **Routing core**[LiteLLM](<https://github.com/BerriAI/litellm>) (MIT) — 100+ provider unified API
  * **Providers** · primary AWS Bedrock (Claude Sonnet 4.6 / Haiku 4.5) → fallback Anthropic API ZDR → fallback OpenAI ZDR
  * **Embeddings** · self-hosted BAAI/bge-m3 on a shared GPU node
  * **Rerank** · self-hosted BAAI/bge-reranker-v2-m3
  * **Cache** · Redis (semantic + exact-match prompt cache)
  * **Redaction** · Microsoft Presidio + custom CyberSkill rules for VN identifiers (CCCD, MST)
  * **Cost ledger** · Postgres + per-tenant rolling counter (resets daily UTC)
  * **Tracing** · OTel spans for every model call; LangSmith for CUO sessions



#### Why these picks

  * **LiteLLM** · MIT-licensed, single API, the entire CyberOS extension surface is middleware. Cheap to fork if needed.
  * **Bedrock primary** · ZDR by default, residency by region, Anthropic models without contract overhead, regional Singapore endpoint for VN tenants.
  * **Self-hosted embeddings** · embedding cost dominates at scale; BGE-M3 (one of the highest MIRACL scores) runs cheaply on one shared GPU.
  * **Presidio** · open-source NER for PII, plus custom rules for Vietnamese-specific identifiers (MST, CCCD, bank account regex).
  * **Semantic cache** · CUO answers many similar questions; 30-50% hit rate at internal scale per pilot.



AI Gateway internal architecture

flowchart TB CALLER[("Subgraph or CUO")] --> INGRESS["Ingress · JWT + tenant validation"] INGRESS --> COSTGATE{"Cost budget check  
per-tenant ceiling"} COSTGATE -- "over budget" --> REJECT[/"429 quota exceeded"/] COSTGATE -- "ok" --> REDACT["Presidio redaction  
\+ VN identifier rules"] REDACT --> PERSONA["Persona-version stamp  
prepend system prompt"] PERSONA --> CACHE{"Semantic + exact cache"} CACHE -- "hit" --> CACHEHIT["Return cached response  
fresh persona stamp"] CACHE -- "miss" --> ROUTER["LiteLLM router  
model selection by capability"] ROUTER --> RESIDENCY{"Tenant residency"} RESIDENCY -- "vn-shard" --> BEDROCK_SG["Bedrock  
ap-southeast-1"] RESIDENCY -- "eu-shard" --> BEDROCK_EU["Bedrock  
eu-central-1"] RESIDENCY -- "us-shard" --> BEDROCK_US["Bedrock  
us-east-2"] BEDROCK_SG --> RESPONSE["Stream response"] BEDROCK_EU --> RESPONSE BEDROCK_US --> RESPONSE BEDROCK_SG -. "failover".-> ANTHROPIC["Anthropic ZDR"] ANTHROPIC -. "failover".-> OPENAI["OpenAI ZDR"] ANTHROPIC --> RESPONSE OPENAI --> RESPONSE RESPONSE --> LEDGER["Cost ledger update"] RESPONSE --> AUDIT["Audit row to NATS"] LEDGER --> CALLER CACHEHIT --> CALLER classDef gate fill:#fef6e0,stroke:#9c750a,stroke-width:2px classDef prov fill:#e8d4c2,stroke:#45210e classDef block fill:#fecaca,stroke:#b91c1c class INGRESS,COSTGATE,REDACT,PERSONA,CACHE,ROUTER,RESIDENCY,RESPONSE,LEDGER,AUDIT,CACHEHIT gate class BEDROCK_SG,BEDROCK_EU,BEDROCK_US,ANTHROPIC,OPENAI prov class REJECT block 

Flow · CUO answer with provider failover

sequenceDiagram autonumber participant CUO as CUO supervisor participant GW as AI Gateway participant CA as Semantic cache participant LL as LiteLLM router participant BR as Bedrock (primary) participant AN as Anthropic ZDR (fallback) CUO->>GW: chat-completion { msgs, persona=CFO-v3 } GW->>GW: Cost check (tenant=acme: 60% of $150 cap) GW->>GW: Redact PII (Presidio + VN rules) GW->>GW: Inject CFO-v3 system prompt GW->>CA: Lookup hash + embedding CA-->>GW: miss GW->>LL: route(model=claude-sonnet-4-6, region=ap-southeast-1) LL->>BR: invoke_model BR--xLL: 5xx after 4s Note over LL,BR: Circuit breaker opens for 30s LL->>AN: messages.create AN-->>LL: stream chunks LL-->>GW: stream GW->>CA: Store (response, hash, embedding) GW->>GW: Log cost (412 tokens × $0.003) GW-->>CUO: stream GW->>GW: Audit row to NATS cyberos.acme.ai.invoke.completed 

### Key contracts
    
    
    # OpenAI-shaped surface (LiteLLM convention)
    POST /v1/chat/completions # streaming + non-streaming
    POST /v1/embeddings # BGE-M3 self-hosted
    POST /v1/rerank # BGE-reranker self-hosted
    POST /v1/messages # Anthropic-shaped passthrough
    GET /v1/usage?tenant=acme # rolling cost view
    GET /v1/models # capability-classified list
    
    # Required headers
    Authorization: Bearer <subgraph JWT>
    X-Cyberos-Tenant: acme
    X-Cyberos-Persona: cfo-v3
    X-Cyberos-Module: rew # for cost attribution
    X-Cyberos-Trace-Id: 01HXY... # OTel trace propagation
    
    # Response headers
    X-Cyberos-Cost-Cents: 13
    X-Cyberos-Cache: hit | miss | bypass
    X-Cyberos-Provider: bedrock | anthropic | openai
    X-Cyberos-Persona-Version: cfo-v3.2.1

### Latency budgets 

Path| p50| p95| Notes  
---|---|---|---  
Chat completion (Haiku)| < 600 ms| < 1.4 s| CHAT message-suggest; uses prompt cache  
Chat completion (Sonnet)| < 1.5 s| < 3.0 s| CUO answers; complex reasoning  
Embedding (BGE-M3)| < 30 ms| < 80 ms| self-hosted; batch of 32  
Reranker (BGE-rerank-v2-m3)| < 80 ms| < 200 ms| self-hosted; top-150 → top-20  
memory search end-to-end| < 120 ms| < 250 ms| embed + retrieve + rerank  
MCP tool call (read-only)| < 200 ms| < 500 ms| via Apollo Router; cached  
MCP tool call (write)| < 400 ms| < 1.0 s| includes audit + NATS emit  
  
Internal cost

≤ $150/mo

LLM portion at 10 Members

50-tenant cost

≤ $4/user/mo

LLM portion per active user

Cache hit rate

≥ 30%

Semantic + exact

Failover MTTR

< 5 s

Bedrock → Anthropic ZDR

#### Status

P0 planned · P0 · slice 1 → P0 · exit

  * LiteLLM forked at `cyberos/litellm-cyberos` with middleware overlay
  * Presidio rule pack `cyberos-vn-rules` covers MST, CCCD, VietQR
  * Bedrock allowlist · Sonnet 4.6, Haiku 4.5, Titan embed (fallback)
  * Cost ledger primed with $0.003/$0.015 Sonnet rate, $0.0008/$0.004 Haiku



#### References

  * Internal spec — AI Gateway
  * Internal spec — Latency budgets
  * OWASP Gen AI Top 10 (2025-04 revision)
  * NIST AI 600-1 (GenAI Risk Profile)



3

## MCP Gateway

P0 · planned

🔌 The agent-operability surface. Every module owns its MCP server; the Gateway federates them into one discovery endpoint. CyberOS targets the **2025-11-25 spec** — the production-stable line as of May 2026. 

### Why a separate layer

The MCP "gateway" is not a single monolithic server — it is a **federation router**. Each module owns its own MCP server, runs side-by-side with its subgraph, shares the database connection pool, and uses the same RBAC predicates. The gateway's job is two things: (1) one discovery endpoint at `/.well-known/mcp` so a Claude Desktop or Cursor session can auto-detect the catalog, and (2) cross-cutting policy enforcement — tool annotations (destructive / readOnly / idempotent / openWorld), OAuth audience binding, audit row emission.

Naming convention is the moat against tool-name collisions: `cyberos.{module}.{verb}_{noun}`. Examples: `cyberos.proj.create_task`, `cyberos.memory.search`, `cyberos.rew.payslip_explain` (read-only narrative; never compute). Collisions are rejected at registration time.

### 2025-11-25 spec features adopted

Spec change| What it gives us| Implementation  
---|---|---  
**Tasks** (long-running ops)| "draft the board pack" without blocking chat session| Tasks subgraph stores state; webhooks callback on complete  
**Sampling-with-Tools**|  Servers can delegate sub-tasks back to host LLM with tool access| Nested decomposition; rate-limited per session  
**SEP-986 well-known**|  Single `.well-known/mcp` replaces hand-coded server lists| Gateway publishes discovery; clients auto-detect  
**Tool annotations**| `destructive=true` tools auto-route through human-in-the-loop| Validated at registration; runtime check on call  
**Streamable HTTP**|  Single endpoint, mid-stream resumability, HTTP/2 + HTTP/3| Default transport; SSE deprecated for new servers  
**Elicitation**|  Server can ask user "which workspace?" mid-call| Implemented as prompt back-channel; Gateway proxies safely  
**Resource embedding**|  Tool returns can embed Resources for LLM to read| "Show me the policy doc" + tool-call combined patterns  
**Title fields**|  Human-friendly names separate from technical IDs| Genie panel display; technical name = audit identifier  
  
#### Tech stack

  * **Transport** · Streamable HTTP (default) + WebSocket upgrade
  * **Per-module server** · TypeScript SDK (`@modelcontextprotocol/sdk`) or Rust (`mcp-rs` · CyberSkill-published)
  * **Federation router** · Hono + custom MCP-aware reverse proxy
  * **Discovery**`/.well-known/mcp` \+ `/.well-known/oauth-protected-resource`
  * **OAuth** · OAuth 2.1 + PKCE (S256); audience-bound tokens
  * **Registry** · Postgres table; tool annotations validated on registration
  * **HITL** · LangGraph `interrupt` gate for destructive tools



#### Tool annotations enforced

  * `readOnlyHint=true` · executes without confirmation prompt
  * `destructiveHint=true` · HITL confirm UI; LangGraph interrupt
  * `idempotentHint=true` · safe to retry; no double-execution
  * `openWorldHint=true` · may communicate externally; annotated in audit
  * `title="..."` · human-friendly label in Genie panel



MCP Gateway architecture · federation, not monolith

flowchart TB CLIENT[("Claude Desktop / Codex / Cursor  
12+ AI clients · all MCP 2025-11-25-compatible")] CLIENT --> DISCOVERY["/.well-known/mcp  
SEP-986 discovery"] DISCOVERY --> PRM["/.well-known/oauth-protected-resource  
per-tenant"] PRM --> AUTHSRV["OAuth authorisation server  
(AUTH module)"] CLIENT --> ROUTER["MCP Gateway router  
Streamable HTTP"] ROUTER --> REGISTRY[("Tool registry  
annotation-validated")] ROUTER --> HITL{"destructive?"} HITL -- "yes" --> CONFIRM["LangGraph interrupt  
user confirm in Genie panel"] HITL -- "no" --> DISPATCH["dispatch to module server"] CONFIRM --> DISPATCH DISPATCH --> MEMORY_MCP["memory MCP server  
cyberos.memory.*"] DISPATCH --> CHAT_MCP["CHAT MCP server  
cyberos.chat.*"] DISPATCH --> PROJ_MCP["PROJ MCP server  
cyberos.proj.*"] DISPATCH --> REW_MCP["REW MCP server  
cyberos.rew.* (read-only)"] DISPATCH --> OTHER["…19 more"] MEMORY_MCP --> AUDITNATS["NATS audit row"] CHAT_MCP --> AUDITNATS PROJ_MCP --> AUDITNATS classDef gw fill:#fef6e0,stroke:#9c750a,stroke-width:2px classDef mod fill:#e8d4c2,stroke:#45210e classDef ext fill:#f9c64f,stroke:#9c750a classDef block fill:#fecaca,stroke:#b91c1c class DISCOVERY,PRM,ROUTER,REGISTRY,DISPATCH,CONFIRM gw class MEMORY_MCP,CHAT_MCP,PROJ_MCP,REW_MCP,OTHER mod class CLIENT,AUTHSRV,AUDITNATS ext class HITL block 

Flow · Claude Desktop invokes a destructive tool

sequenceDiagram autonumber actor M as Member participant CD as Claude Desktop participant GW as MCP Gateway participant AU as AUTH participant PR as PROJ MCP server participant LG as LangGraph (CUO) participant DB as PROJ Postgres participant NA as NATS CD->>GW: GET /.well-known/mcp GW-->>CD: { servers, authz_endpoint, scopes } CD->>AU: OAuth 2.1 + PKCE flow (audience=acme.cyberos.world) AU-->>CD: access_token (audience-bound) CD->>GW: tools/call { name: "cyberos.proj.delete_task", id: "T-42" } GW->>GW: Lookup annotation · destructiveHint=true GW->>LG: interrupt(decision="delete_task T-42 — confirm?") LG->>M: Genie panel · "Confirm delete?" M-->>LG: Approve LG-->>GW: resume GW->>PR: forward call (with member JWT) PR->>DB: DELETE FROM tasks WHERE id=$1 AND tenant_id=current_setting('app.tenant_id') DB-->>PR: 1 row affected PR->>NA: publish cyberos.acme.proj.task.deleted { id, actor, prev } PR-->>GW: { ok: true } GW-->>CD: { content: [{type:"text", text:"Task T-42 deleted"}] } 

### OAuth-Protected Resource Metadata 
    
    
    # GET /.well-known/oauth-protected-resource (per-tenant)
    {
     "resource": "https://acme-tenant.cyberos.world",
     "authorization_servers": [
     "https://acme-tenant.cyberos.world/oauth"
     ],
     "scopes_supported": [
     "cyberos.read", "cyberos.write",
     "cyberos.memory.read", "cyberos.memory.write",
     "cyberos.proj.read", "cyberos.proj.write",
     "cyberos.chat.read", "cyberos.chat.write",
     "cyberos.rew.read"
     ],
     "bearer_methods_supported": ["header"]
    }
    
    # GET /.well-known/oauth-authorization-server
    {
     "issuer": "https://acme-tenant.cyberos.world/oauth",
     "authorization_endpoint": ".../authorize",
     "token_endpoint": ".../token",
     "code_challenge_methods_supported": ["S256"],
     "response_types_supported": ["code"],
     "grant_types_supported": ["authorization_code", "refresh_token"]
    }

**Note:** _Resource-server-as-OAuth-client_ behaviour is forbidden — the prior pattern that the security community flagged in 2025 ("token shadow handoff" via `X-Forwarded-Authorization`) is rejected at gateway level. Audience binding ensures an Acme tenant's token cannot be replayed at the Beta tenant's gateway even if leaked.

#### Status

P0 planned · P0 · slice 1 → P0 · exit

  * 2025-11-25 spec targeted (Tasks, Streamable HTTP, Elicitation, PRM)
  * Per-module servers ship alongside subgraphs; reuse RBAC/DB pool
  * Tool annotations validated at registration; runtime check on call
  * 12+ AI client compatibility tested (Claude products, Cursor, Cline, Codex); all MCP 2025-11-25-compatible



#### References

  * Internal spec — MCP Gateway & 2025-11-25 spec
  * Internal spec — Authentication & authorisation
  * Internal spec — Tool registry & per-module servers
  * Internal spec — OAuth-Protected Resource & PRM flow
  * [MCP 2025-11-25 spec](<https://modelcontextprotocol.io/specification/2025-11-25>)



4

## OBS — Observability

P0 · planned

👁 Logs, metrics, traces — for every module, every gateway, every agent action. OBS is also the surface CUO's CTO skill reports against. 

### Why a separate layer

In a 23-module platform with agent traffic on top of human traffic, you cannot answer "why is REW slow?" without tying together: a Member's request through Apollo Router, the REW subgraph's DB query, a call out to the AI Gateway for a payslip narrative, an audit row to NATS, and a downstream LEARN subscription. OBS is the single trace tree that makes that legible.

OBS also feeds CUO's CTO skill: weekly OBS dashboard digests, security advisory pipelines, and model registry summaries all read from OBS ( AI matrix).

#### Tech stack — LGTM (DEC-021)

  * **L** oki · logs (open-source, self-hostable, S3-compatible storage)
  * **G** rafana · dashboards (open-source visualisation)
  * **T** empo · distributed traces (OTel-native)
  * **M** imir · metrics (Prometheus long-term storage)
  * **Collector** · OpenTelemetry Collector (OTel) at `:4317` gRPC
  * **Alerting** · Grafana Alertmanager + PagerDuty webhook
  * **Agent traces** · LangSmith for CUO session timelines
  * **Synthetic monitoring** · Grafana k6 scripts in CI



#### Why these picks

  * **LGTM, not Datadog** · cost predictability is the founder's first constraint; Datadog at 23-module ingest scales linearly with bill.
  * **OTel-native** · every subgraph + gateway speaks one wire format; trade providers later without rewriting instrumentation.
  * **S3 backend** · Loki/Tempo/Mimir all store on the same R2/MinIO bucket as memory's archival layer; one cost model.
  * **LangSmith for agent runs** · CUO's persona-version stamps, tool calls, and HITL gates need agent-aware UX; LangSmith is the lowest-friction tool.



OBS pipeline · OTel → LGTM

flowchart LR subgraph SRC ["Emitters (every CyberOS component)"] M1[Subgraphs] --> OTEL M2[Gateways] --> OTEL M3[CUO supervisor] --> OTEL M4[Front-end host] --> OTEL M5[NATS · S3 · Postgres] --> OTEL end OTEL[("OpenTelemetry Collector  
gRPC:4317 · HTTP:4318")] OTEL --> LOKI[("Loki  
logs · S3 backend")] OTEL --> TEMPO[("Tempo  
traces · S3 backend")] OTEL --> MIMIR[("Mimir  
metrics · S3 backend")] LOKI --> GRAFANA[("Grafana  
dashboards · LGTM unified")] TEMPO --> GRAFANA MIMIR --> GRAFANA GRAFANA --> ALERT[("Alertmanager  
PagerDuty webhook")] GRAFANA --> CUO_CTO[("CUO · CTO skill  
weekly digest")] classDef src fill:#e8d4c2,stroke:#45210e classDef pipe fill:#fef6e0,stroke:#9c750a,stroke-width:2px classDef store fill:#cba88a,stroke:#45210e classDef vis fill:#f5ede6,stroke:#45210e class M1,M2,M3,M4,M5 src class OTEL pipe class LOKI,TEMPO,MIMIR store class GRAFANA,ALERT,CUO_CTO vis 

Trace sampling

10%

100% for errors + LLM calls

Log retention

30 d

90d audit · 7y compensation

Alert MTTR

< 5 min

PagerDuty page → Founder

OBS cost

≤ $80/mo

internal scale

#### Status

P0 planned · P0 · slice 2 → P0 · exit

  * LGTM stack via Grafana Cloud free tier in P0, self-host at P1
  * OTel instrumentation in module-template; every new subgraph wired by default
  * RED dashboard per subgraph (rate, errors, duration)
  * CUO CTO-skill digest job emits weekly summary to Founder



#### References

  * DEC-021 — LGTM observability stack
  * OpenTelemetry semantic conventions 1.27+
  * N(FR pending) — p99 latency degradation budget (CI gate)



5

## GraphQL Federation

P0 · planned

🌐 Apollo Federation v2.5+. One supergraph, 22 subgraphs, one persisted-query budget. The agent surface and the human surface read the same schema. 

### Why a separate layer (and why Apollo Federation specifically)

CyberOS rejects three alternative API postures. **REST per module** means the front-end host shell does N round-trips per page. **BFF (backend-for-frontend) per module** means N+M maintenance burdens. **Single monolithic GraphQL** means schema-merge conflicts every time a module ships. Apollo Federation v2.5+ solves all three: each module owns its subgraph SDL, the Router composes the supergraph at deploy time, and one HTTP roundtrip can pull from 5 modules in parallel.

makes **persisted queries mandatory** for production traffic. Query hashes are pre-registered at deploy; any unregistered query is rejected with a 400. This caps abuse, enables CDN caching, and lets each subgraph publish a query budget.

#### Tech stack

  * **Router** · Apollo Router (Rust, OSS, MIT) v1.50+ — Federation v2.5+ compliant
  * **Subgraph servers** · GraphQL Yoga (TypeScript) or async-graphql (Rust)
  * **Composition**`rover supergraph compose` in CI
  * **Persisted query store** · GCS / R2 bucket (CDN-cached)
  * **Directives used**`@key`, `@external`, `@requires`, `@provides`, `@shareable`, `@inaccessible`, `@tag`
  * **Auth context** · JWT validated at Router; `tenant_id` \+ `actor` propagated to subgraphs as headers
  * **Caching** · Apollo Router edge cache + Cloudflare CDN



#### Why these picks

  * **Apollo Router, not GraphQL Mesh** · production-grade Rust runtime; query plan cache; Federation v2.5 reference.
  * **Persisted queries mandatory** · zero query injection surface; CDN-cacheable; rate-shaped.
  * **Federation v2.5**`@interfaceObject` (P3 module hierarchies), `@progressive @override` (zero-downtime schema moves).
  * **Schema deprecation discipline** · removal requires ≥ 1 phase notice (N(FR pending)); breaks no client mid-phase.



Federation composition · 22 subgraphs → one supergraph

flowchart TB HOST[("Host shell · Module Federation  
Vite · React 19 · Tauri")] HOST --> ROUTER["Apollo Router v1.50+  
query plan · auth context · cache"] ROUTER --> COMPOSITION{"Persisted query  
hash lookup"} COMPOSITION -- "miss" --> REJECT[/"400 unregistered"/] COMPOSITION -- "hit" --> PLAN["Query plan  
parallel subgraph fanout"] PLAN --> SG_memory["memory subgraph"] PLAN --> SG_CHAT["CHAT subgraph"] PLAN --> SG_PROJ["PROJ subgraph"] PLAN --> SG_AUTH["AUTH subgraph"] PLAN --> SG_DOTS["…18 more"] SG_memory --> PG_memory[("memory schema  
RLS-enforced")] SG_CHAT --> PG_CHAT[("CHAT schema  
RLS-enforced")] SG_PROJ --> PG_PROJ[("PROJ schema  
RLS-enforced")] SG_AUTH --> PG_AUTH[("AUTH schema")] classDef gw fill:#fef6e0,stroke:#9c750a,stroke-width:2px classDef mod fill:#e8d4c2,stroke:#45210e classDef ext fill:#f9c64f,stroke:#9c750a classDef block fill:#fecaca,stroke:#b91c1c classDef db fill:#fde7b3,stroke:#9c750a class ROUTER,PLAN gw class SG_memory,SG_CHAT,SG_PROJ,SG_AUTH,SG_DOTS mod class HOST ext class COMPOSITION block class PG_memory,PG_CHAT,PG_PROJ,PG_AUTH db 

Flow · "open my dashboard" pulls from 5 subgraphs in parallel

sequenceDiagram autonumber actor M as Member participant H as Host shell participant R as Apollo Router participant CHAT as CHAT subgraph participant PROJ as PROJ subgraph participant CRM as CRM subgraph participant REW as REW subgraph participant BR as memory subgraph M->>H: open /home H->>R: POST /graphql { hash: "0xabc...", vars } R->>R: Persisted query lookup → DashboardQuery R->>R: Validate JWT · extract tenant_id, actor par parallel fanout R->>CHAT: { unreadCount } and R->>PROJ: { myTasks(limit:5) } and R->>CRM: { myDeals(stage:OPEN) } and R->>REW: { myPayslipStub } and R->>BR: { memorySearch(query:"today") } end CHAT-->>R: { unreadCount: 3 } PROJ-->>R: { myTasks: [...] } CRM-->>R: { myDeals: [...] } REW-->>R: { myPayslipStub: {...} } BR-->>R: { memorySearch: {...} } R-->>H: { data: {...merged} } H->>M: render 

GraphQL p95

≤ 400 ms

N(FR pending)

Cache hit rate

≥ 70%

N(FR pending) persisted-query

Subgraph deploy

≤ 10 min

N(FR pending) module CI

Composition check

pass

N(FR pending) per release

#### Status

P0 planned · P0 · start → P0 · slice 1

  * Apollo Router scaffold + composition CI live at P0 · start
  * memory, AUTH subgraphs first to integrate at P0 · slice 1
  * Persisted query registration auto-bound to host-shell build
  * Schema deprecation policy in CONTRIBUTING.md



#### References

  * DEC-002 — Apollo Federation v2.5+
  * [Apollo Federation v2 docs](<https://www.apollographql.com/docs/federation/>)



6

## NATS JetStream

P0 · planned

📬 Every state-changing action emits an event. NATS JetStream is the spine. Durable consumers, tenant-scoped subjects, audit-grade retention. 

### Why a separate layer

A 23-module platform needs to decouple write paths from downstream effects. When REW publishes a payslip, six things need to happen: memory ingests the narrative, LEARN updates the career-level snapshot, OBS emits a metric, CUO queues a "review your payslip" Notify, the Compliance audit row is hashed, and the Member's mobile gets a push. All six are _events_ on the canonical subject `cyberos.acme.rew.payslip.published`.

locks the convention: `cyberos.{tenant}.{module}.{entity}.{verb}`. Subjects are tenant-scoped so subscribers cannot accidentally cross tenant boundaries. Streams retain 30 days by default, 90 days for compensation/ESOP. CUO's ambient-trigger consumers subscribe through durable consumers so a restart never loses pending nudges.

#### Tech stack

  * **Broker** · NATS Server v2.10+ with JetStream
  * **Client libs**`nats.go`, `@nats-io/nats.js`, `async-nats` (Rust)
  * **Schemas** · CloudEvents 1.0 envelope + JSON Schema body per subject
  * **Schema registry** · self-hosted; refs in subgraph CI
  * **Durable consumers** · CUO ambient-trigger, OBS rollups, memory ingestion
  * **Replication** · 3-node JetStream cluster per region
  * **DLQ** · failed messages routed to `cyberos.{tenant}.dlq.{module}.{entity}.{verb}`



#### Why NATS, not Kafka / Redpanda

  * **Subject hierarchy native** · Kafka has flat topics; NATS subjects (`cyberos.acme.proj.*`) match CyberOS conventions one-to-one.
  * **Latency** · sub-millisecond pub/sub; Kafka adds tens of ms per consumer group.
  * **Footprint** · single 50 MB binary; no Zookeeper/Kraft cluster to operate at 10-Member scale.
  * **JetStream** · adds Kafka-style durability without giving up subject wildcards.
  * **Cost** · runs on a single $20/mo VM in P0; clusters at P3.



### Canonical subjects 
    
    
    # Format
    cyberos.{tenant}.{module}.{entity}.{verb}
    
    # Examples
    cyberos.acme.proj.task.created
    cyberos.acme.proj.task.assigned
    cyberos.acme.proj.task.completed
    cyberos.acme.rew.payslip.published # 90-day retention
    cyberos.acme.rew.bp_balance.updated
    cyberos.acme.memory.fact.added
    cyberos.acme.memory.fact.conflict_detected
    cyberos.acme.crm.deal.stage_changed
    cyberos.acme.chat.message.posted
    cyberos.acme.audit.event.recorded # Merkle-chained; 7y retention
    cyberos.acme.ai.invoke.completed # cost ledger
    cyberos.acme.mcp.tool.invoked
    
    # Durable consumers
    - cuo-ambient → subscribes to *.task.* + *.deal.* + *.payslip.*
    - memory-ingest → subscribes to all non-compensation events
    - obs-rollup → subscribes to *.>
    - compliance-audit → subscribes to *.audit.>
    - learn-snapshot → subscribes to rew.payslip.* + hr.level.*

Flow · REW publishes payslip · six downstream consumers

sequenceDiagram autonumber participant HR as HR/Ops Lead participant REW as REW subgraph participant DB as REW Postgres participant NA as NATS JetStream participant BR as memory ingest participant LE as LEARN snapshot participant CO as Compliance audit participant CUO as CUO ambient participant PUSH as Mobile push participant OB as OBS rollup HR->>REW: publish payslip { memberId, monthEnd } REW->>DB: INSERT payslip · INSERT audit_event (Merkle hash) REW->>NA: publish cyberos.acme.rew.payslip.published par fan-out NA->>BR: ingest narrative · (compensation excluded by denylist) and NA->>LE: snapshot career-level + tenure and NA->>CO: append to per-scope Merkle chain and NA->>CUO: nudge member "review your payslip" and NA->>PUSH: push notification and NA->>OB: increment payslips_published_total{tenant=acme} end 

Pub latency p95

< 5 ms

in-cluster

Default retention

30 d

90d comp/ESOP · 7y audit

Replication

R=3

per-region JetStream cluster

DLQ replay

CLI

`cyberos dlq replay`

#### Status

P0 planned · P0 · start → P0 · slice 1

  * Single-node NATS at P0 · start; 3-node cluster at P1 · exit
  * Module-template includes typed publisher/consumer helpers
  * Schema registry CI-validated at subgraph PR-time
  * Per-tenant subject ACLs at NATS-level (defense in depth alongside RLS)



#### References

  * DEC-004 — NATS JetStream events
  * [NATS JetStream docs](<https://docs.nats.io/nats-concepts/jetstream>)



7

## End-to-end · "a module makes a request"

The six pillars are useful in isolation; they are _load-bearing_ when composed. Below is a single end-to-end trace of one user action — Trinh, a Member, asks Genie "what should I work on today?" — passing through every pillar exactly once. 

sequenceDiagram autonumber actor T as Trinh (Member) participant H as Host shell participant R as Apollo Router participant AU as AUTH participant CUO as CUO supervisor participant MCP as MCP Gateway participant PROJ as PROJ MCP server participant BR as memory MCP server participant AI as AI Gateway participant LL as LiteLLM → Bedrock participant NA as NATS participant OB as OBS · OTel T->>H: "@genie what should I work on today?" H->>R: POST /graphql { hash, vars } R->>AU: validate JWT (JWKS cached) AU-->>R: { tenant=cyberskill, actor=member:trinh } R->>CUO: cuoAsk { prompt, threadId } CUO->>OB: span "cuo.session.start" CUO->>MCP: tools/list (refresh registry) MCP-->>CUO: { proj.list_my_tasks, memory.search,... } par retrieve context CUO->>MCP: cyberos.proj.list_my_tasks(member=trinh, due_today=true) MCP->>PROJ: forward (with Trinh's JWT) PROJ-->>MCP: 4 tasks MCP-->>CUO: [4 tasks] and CUO->>MCP: cyberos.memory.search("today priorities trinh") MCP->>BR: forward BR->>AI: POST /v1/embeddings { input } AI->>LL: BGE-M3 embed (self-hosted) LL-->>AI: vector AI-->>BR: vector BR-->>MCP: [5 relevant facts] MCP-->>CUO: [5 facts] end CUO->>AI: chat.completions { msgs, persona=cuo-v3, model=sonnet } AI->>AI: cost-check · redact · cache miss AI->>LL: bedrock invoke LL-->>AI: stream AI-->>CUO: stream CUO-->>R: { answer, citations, persona_version } R-->>H: { data } H-->>T: render Genie response CUO->>NA: publish cyberos.cyberskill.cuo.session.completed CUO->>OB: span "cuo.session.end" (latency=2.3s, tokens=412, cost=$0.011) 

### The six pillars, one trace

**AUTH** · JWT validated at Router; agent identity = Trinh's identity

**GraphQL** · persisted-query lookup at Router; auth context propagated to subgraphs

**MCP Gateway** · tool discovery; per-module dispatch with Trinh's JWT

**AI Gateway** · embedding + chat completion; cost ledger; persona stamping

**NATS** · CUO session lifecycle published for downstream consumers

**OBS** · one trace tree across the whole 11-step sequence

∞

## References

#### CyberOS source documents

  * Internal spec — The high-level system
  * Internal spec — GraphQL Federation
  * Internal spec — Module Federation (frontend)
  * Internal spec — MCP Gateway and the 2025-11-25 spec
  * Internal spec — Authentication and authorisation
  * Internal spec — Tool registry and per-module servers
  * Internal spec — OAuth-protected resource and PRM flow
  * Internal spec — AI Gateway
  * DEC-001..DEC-066 — locked decisions



#### External standards & specs

  * [MCP 2025-11-25 specification](<https://modelcontextprotocol.io/specification/2025-11-25>)
  * [Apollo Federation v2 docs](<https://www.apollographql.com/docs/federation/>)
  * RFC 7519 — JSON Web Token (JWT)
  * RFC 7636 — PKCE for OAuth 2.0
  * RFC 6749 — OAuth 2.0 / draft OAuth 2.1
  * RFC 6238 — TOTP
  * W3C WebAuthn Level 3
  * RFC 6532 — Internationalized email (UTF-8 throughout)
  * [NATS JetStream documentation](<https://docs.nats.io/>)
  * OpenTelemetry semantic conventions 1.27+
  * OWASP Generative AI Top 10 (2025-04)
  * NIST AI 600-1 — Generative AI Risk Profile



[ Back to home ](<../index.html>) [ Next · Compliance plan  ](<compliance.html>)
