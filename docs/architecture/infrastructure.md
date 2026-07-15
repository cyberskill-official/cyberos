---
title: Infrastructure
source: website/docs/architecture/infrastructure.html
migrated: TASK-DOCS-002
---

## The six-pillar plane

The platform contract locks the high-level shape: every module is an independently deployable Apollo Federation subgraph and a Module-Federation frontend remote, sitting on top of six cross-cutting pillars. The pillars are infrastructure modules - they are owned, versioned, and on-call exactly like the functional modules - but they are special in that every other module depends on them. Removing any one of the six breaks the platform.

The six pillars:

- **AUTH** - OAuth 2.1 + JWT (RS256); per-tenant authorization server
- **AI Gateway** - LiteLLM router; Bedrock, Anthropic, OpenAI
- **MCP Gateway** - 2025-11-25 spec; per-module servers
- **OBS** - LGTM stack (Loki, Grafana, Tempo, Mimir)
- **Apollo Router** - GraphQL Federation v2.5+; persisted queries
- **NATS JetStream** - subjects `cyberos.{tenant}.{module}.{entity}.{verb}`

Request shape: Members (browser / Tauri) enter through the Apollo Router; agents (Claude / Codex / Cursor, via Skills + MCP) enter through the MCP Gateway. The 22 functional modules - each a subgraph + MCP server + UI remote - call the AI Gateway for LLM work, persist to per-module PostgreSQL 17 schemas (RLS enforced) and S3 / R2 / MinIO object storage, emit events to NATS, and send traces to OBS. The gateways trace to OBS as well.

### Why a separate plane, not per-module?

- **Audit unity.** Compliance regulators need one audit chain, not 22. NATS subjects, Merkle-chained audit rows, and OAuth issuance all share one canonical surface.
- **Cost shape.** Per-tenant LLM cost is the largest variable expense at scale; a single gateway lets the CFO put a ceiling on it (<= $150/mo internal, <= $4/active user/mo at 50-tenant).
- **Provider failover.** One gateway means primary Bedrock -> Anthropic ZDR -> OpenAI ZDR happens once, not 22 times in 22 different ways.
- **Agent parity.** Strategic bet #1. A human's request and an agent's request hit the same gateway, the same RBAC, the same audit row.

### The contracts each pillar exposes

- AUTH: JWT validation header, audience binding, RBAC predicate evaluator
- AI Gateway: `POST /v1/chat/completions`, `/v1/embeddings`, `/v1/rerank` (OpenAI-shaped)
- MCP Gateway: Streamable HTTP + `/.well-known/mcp` + per-tenant OAuth-PRM
- OBS: OTel collector at `otel:4317` (gRPC) + `:4318` (HTTP)
- GraphQL: composed supergraph at `https://{tenant}.cyberos.world/graphql`
- NATS: subject `cyberos.{tenant}.{module}.{entity}.{verb}`

## AUTH

The identity backbone. AUTH owns who you are; every other module trusts AUTH. The minimal interface is "give me an RS256-signed JWT for this Member" - the rest is implementation.

### Why a separate layer

If each module re-implemented identity, the platform would suffer the classic distributed-monolith pathology: 22 places where a security advisory needs to be applied, 22 places where MFA enforcement can drift. AUTH centralises the surface that absolutely cannot drift.

More structural: AI agents authenticate as Members - there is no "service account with broad permissions" pattern. Agent parity (strategic bet #1) requires that a Claude session run on the same identity contract as a human session. One AUTH module is the only sensible place to enforce that.

#### Tech stack

- Library: Hono + jose (TypeScript subgraph) or axum + jsonwebtoken (Rust subgraph)
- Token: JWT RS256 with rotating signing keys (90-day rotation, 30-day grace)
- Session refresh: opaque token, HttpOnly + SameSite=Lax cookie, 30-day max
- MFA: TOTP (RFC 6238) minimum + WebAuthn / passkey for elevated roles
- Magic link: onboarding only, single-use, 15-minute TTL
- OAuth 2.1 + PKCE: per-tenant authorisation server (S256 challenge only)
- RBAC store: Postgres with RLS-enforced `app.tenant_id` session GUC

#### Why these picks

- RS256, not HS256: only AUTH needs the private key; every subgraph validates with the public key without round-tripping AUTH.
- Per-tenant authz server: `acme.cyberos.world/.well-known/oauth-authorization-server` means a leaked Acme token cannot replay against Beta's gateway (audience binding, section 8.4.1).
- Postgres RLS, not application-layer ACL: even a bug in a subgraph cannot cross tenant boundaries, because the database itself refuses to return another tenant's rows.
- WebAuthn for Founder/CEO: phishing-resistant by spec; mandatory.

#### Internal architecture

AUTH comprises an OAuth 2.1 server (the per-tenant authz server), a JWT issuer (RS256, 24 h expiry), a refresh-token store (opaque tokens, cookie), an MFA verifier (TOTP + WebAuthn), an RBAC predicate engine (role x resource x action), and a signing-key rotation service with a JWKS endpoint - backed by a sessions Postgres (sessions, refresh, MFA enrolment). Subgraphs fetch and cache the JWKS, then validate JWTs and check RBAC predicates locally.

#### Flow: Member login with passkey, then MFA-elevated step-up

1. The Member opens `acme.cyberos.world`; the host shell fetches `/.well-known/oauth-authorization-server` (issuer, authz endpoint, token endpoint, S256 only).
2. The host shell calls `GET /authorize?response_type=code&code_challenge=...`; AUTH prompts for a passkey (WebAuthn).
3. The Member signs the assertion; AUTH verifies enrolment against the sessions DB, increments the session counter, and returns a 302 with the authorization code.
4. The host shell exchanges the code and verifier at `POST /token`, receiving a JWT (RS256, 24 h) plus a refresh-token cookie.
5. GraphQL queries carry the Bearer JWT; each subgraph validates it via cached JWKS and checks the RBAC predicate before returning data.
6. Later, when the Founder triggers a destructive op, the host shell calls `POST /step-up { scope: "rew:write" }`; AUTH prompts for a 6-digit TOTP code and returns a JWT with `mfa_elevated=true` and a 15-minute TTL.

### Key contracts

```
# GraphQL contract - abbreviated
type Query {
  me: Member!
  myRoles: [Role!]!
  jwks: JwksDocument!   # public keys for downstream subgraphs
}

type Mutation {
  exchangeCodeForToken(code: ID!, verifier: String!): TokenBundle!
  refresh(token: String!): TokenBundle!
  enrollMfa(method: MfaMethod!): MfaEnrollment!
  stepUp(scope: String!, totp: String): TokenBundle!
  revokeSession(sessionId: ID!): Boolean!
}

type TokenBundle {
  jwt: String!          # RS256, 24h expiry
  expiresAt: DateTime!
  refreshCookie: String # Set-Cookie header sent server-side
}

# MCP tool surface
cyberos.auth.whoami       # readOnly=true
cyberos.auth.list_roles   # readOnly=true
cyberos.auth.audit_login  # readOnly=true (own sessions only)
```

### Role catalogue

| Role | Reads | Writes | Sign / transfer | MFA |
|---|---|---|---|---|
| Founder/CEO | all (own tenant) | all (own tenant) | all | passkey, mandatory |
| Engineering Lead | all (own tenant) | all except REW/ESOP/HR-comp | - | passkey, mandatory |
| HR/Ops Lead | HR / REW / LEARN | HR / REW / LEARN | HR docs | TOTP min |
| Account Manager | CRM / PROJ / TIME / INV / PORTAL | CRM / PROJ / TIME / INV / PORTAL | INV / DOC | TOTP min |
| Member | own + assigned + public | own + assigned | own time entries | recommended |
| Board Member | governance scope | limited sign-offs | SP valuation | passkey, mandatory |
| External Client (P4) | PORTAL scope | PORTAL scope | own docs only | recommended |
| Tenant Admin (P4) | tenant config + audit | tenant config | tenant agreements | passkey, mandatory |
| AI Agent (Member) | = Member | = Member | no auto-sign | inherited |

### Targets

| Metric | Target | Notes |
|---|---|---|
| JWT verify p95 | < 5 ms | JWKS cached in-subgraph |
| Login flow p95 | < 600 ms | with passkey; excludes user time |
| Availability | >= 99.95% | AUTH outage = platform outage |

#### Status

P0 planned; build window: P0 phase.

- OAuth 2.1 PRM compatible with section 8.4.3 MCP discovery
- JWKS endpoint live before any subgraph deploys
- Per-tenant authz server provisioning automated at TEN-module setup (P4)

#### References

- Internal spec - Authentication and RBAC
- Internal spec - Role catalogue (technical detail)
- Internal spec - OAuth 2.1 for MCP (audience binding)
- RFC 7519 (JWT), RFC 7636 (PKCE), W3C WebAuthn L3

## AI Gateway

One door for every LLM call. Routing, caching, redaction, persona stamping, cost accounting, residency enforcement, circuit breaking - all here, once.

### Why a separate layer

The temptation in a 23-module platform is to let each module call the LLM SDK directly - "just `import anthropic` in the subgraph." That fails three ways. First, cost: without one place to set per-tenant budgets, the bill is unobservable until the credit-card statement arrives. Second, residency: a Vietnam-resident tenant must hit the Bedrock Singapore endpoint, never the US; the rule is too easy to bypass when 23 subgraphs each make their own choice. Third, safety: PII redaction, persona-version stamping, and the OWASP Gen-AI Top-10 mitigations cannot be 23-times-correct; they need one chokepoint.

The gateway is non-optional: every LLM call from every module flows through it. The cost target - <= $150/mo internal, <= $4/active user/mo at 50-tenant scale - only works because every token is seen and capped at the gateway.

#### Tech stack

- Routing core: [LiteLLM](https://github.com/BerriAI/litellm) (MIT) - 100+ provider unified API
- Providers: primary AWS Bedrock (Claude Sonnet 4.6 / Haiku 4.5) -> fallback Anthropic API ZDR -> fallback OpenAI ZDR
- Embeddings: self-hosted BAAI/bge-m3 on a shared GPU node
- Rerank: self-hosted BAAI/bge-reranker-v2-m3
- Cache: Redis (semantic + exact-match prompt cache)
- Redaction: Microsoft Presidio + custom CyberSkill rules for VN identifiers (CCCD, MST)
- Cost ledger: Postgres + per-tenant rolling counter (resets daily UTC)
- Tracing: OTel spans for every model call; LangSmith for CUO sessions

#### Why these picks

- LiteLLM: MIT-licensed, single API; the entire CyberOS extension surface is middleware. Cheap to fork if needed.
- Bedrock primary: ZDR by default, residency by region, Anthropic models without contract overhead, regional Singapore endpoint for VN tenants.
- Self-hosted embeddings: embedding cost dominates at scale; BGE-M3 (one of the highest MIRACL scores) runs cheaply on one shared GPU.
- Presidio: open-source NER for PII, plus custom rules for Vietnamese-specific identifiers (MST, CCCD, bank account regex).
- Semantic cache: CUO answers many similar questions; 30-50% hit rate at internal scale per pilot.

#### Request pipeline

Every call passes through, in order: ingress (JWT + tenant validation); per-tenant cost budget check (over budget returns 429 quota exceeded); Presidio redaction plus VN identifier rules; persona-version stamp (system prompt prepended); semantic + exact cache lookup (a hit returns the cached response with a fresh persona stamp); LiteLLM router (model selection by capability); residency routing (vn-shard -> Bedrock ap-southeast-1, eu-shard -> Bedrock eu-central-1, us-shard -> Bedrock us-east-2) with failover Bedrock -> Anthropic ZDR -> OpenAI ZDR; streamed response; then cost ledger update and an audit row to NATS.

#### Flow: CUO answer with provider failover

1. CUO sends a chat completion (persona=CFO-v3); the gateway checks the tenant's budget (for example acme at 60% of the $150 cap), redacts PII (Presidio + VN rules), injects the CFO-v3 system prompt, and misses the semantic cache.
2. LiteLLM routes to Bedrock (claude-sonnet-4-6, ap-southeast-1); Bedrock fails with a 5xx after 4 s and the circuit breaker opens for 30 s.
3. LiteLLM fails over to Anthropic ZDR, which streams the response back through the gateway to CUO.
4. The gateway stores the response in the cache (hash + embedding), logs the cost (412 tokens x $0.003), and emits an audit row to NATS (`cyberos.acme.ai.invoke.completed`).

### Key contracts

```
# OpenAI-shaped surface (LiteLLM convention)
POST /v1/chat/completions   # streaming + non-streaming
POST /v1/embeddings         # BGE-M3 self-hosted
POST /v1/rerank             # BGE-reranker self-hosted
POST /v1/messages           # Anthropic-shaped passthrough
GET  /v1/usage?tenant=acme  # rolling cost view
GET  /v1/models             # capability-classified list

# Required headers
Authorization: Bearer <subgraph JWT>
X-Cyberos-Tenant: acme
X-Cyberos-Persona: cfo-v3
X-Cyberos-Module: rew           # for cost attribution
X-Cyberos-Trace-Id: 01HXY...    # OTel trace propagation

# Response headers
X-Cyberos-Cost-Cents: 13
X-Cyberos-Cache: hit | miss | bypass
X-Cyberos-Provider: bedrock | anthropic | openai
X-Cyberos-Persona-Version: cfo-v3.2.1
```

### Latency budgets

| Path | p50 | p95 | Notes |
|---|---|---|---|
| Chat completion (Haiku) | < 600 ms | < 1.4 s | CHAT message-suggest; uses prompt cache |
| Chat completion (Sonnet) | < 1.5 s | < 3.0 s | CUO answers; complex reasoning |
| Embedding (BGE-M3) | < 30 ms | < 80 ms | self-hosted; batch of 32 |
| Reranker (BGE-rerank-v2-m3) | < 80 ms | < 200 ms | self-hosted; top-150 -> top-20 |
| memory search end-to-end | < 120 ms | < 250 ms | embed + retrieve + rerank |
| MCP tool call (read-only) | < 200 ms | < 500 ms | via Apollo Router; cached |
| MCP tool call (write) | < 400 ms | < 1.0 s | includes audit + NATS emit |

### Targets

| Metric | Target | Notes |
|---|---|---|
| Internal cost | <= $150/mo | LLM portion at 10 Members |
| 50-tenant cost | <= $4/user/mo | LLM portion per active user |
| Cache hit rate | >= 30% | semantic + exact |
| Failover MTTR | < 5 s | Bedrock -> Anthropic ZDR |

#### Status

P0 planned; build window: P0 slice 1 -> P0 exit.

- LiteLLM forked at `cyberos/litellm-cyberos` with middleware overlay
- Presidio rule pack `cyberos-vn-rules` covers MST, CCCD, VietQR
- Bedrock allowlist: Sonnet 4.6, Haiku 4.5, Titan embed (fallback)
- Cost ledger primed with $0.003/$0.015 Sonnet rates, $0.0008/$0.004 Haiku

#### References

- Internal spec - AI Gateway
- Internal spec - Latency budgets
- OWASP Gen AI Top 10 (2025-04 revision)
- NIST AI 600-1 (GenAI Risk Profile)

## MCP Gateway

The agent-operability surface. Every module owns its MCP server; the Gateway federates them into one discovery endpoint. CyberOS targets the 2025-11-25 spec - the production-stable line as of May 2026.

### Why a separate layer

The MCP "gateway" is not a single monolithic server - it is a federation router. Each module owns its own MCP server, runs side-by-side with its subgraph, shares the database connection pool, and uses the same RBAC predicates. The gateway's job is two things: (1) one discovery endpoint at `/.well-known/mcp` so a Claude Desktop or Cursor session can auto-detect the catalog, and (2) cross-cutting policy enforcement - tool annotations (destructive / readOnly / idempotent / openWorld), OAuth audience binding, audit row emission.

The naming convention is the moat against tool-name collisions: `cyberos.{module}.{verb}_{noun}`. Examples: `cyberos.proj.create_task`, `cyberos.memory.search`, `cyberos.rew.payslip_explain` (read-only narrative; never compute). Collisions are rejected at registration time.

### 2025-11-25 spec features adopted

| Spec change | What it gives us | Implementation |
|---|---|---|
| Tasks (long-running ops) | "Draft the board pack" without blocking the chat session | Tasks subgraph stores state; webhooks call back on complete |
| Sampling-with-Tools | Servers can delegate sub-tasks back to the host LLM with tool access | Nested decomposition; rate-limited per session |
| SEP-986 well-known | Single `/.well-known/mcp` replaces hand-coded server lists | Gateway publishes discovery; clients auto-detect |
| Tool annotations | `destructive=true` tools auto-route through human-in-the-loop | Validated at registration; runtime check on call |
| Streamable HTTP | Single endpoint, mid-stream resumability, HTTP/2 + HTTP/3 | Default transport; SSE deprecated for new servers |
| Elicitation | Server can ask the user "which workspace?" mid-call | Implemented as a prompt back-channel; Gateway proxies safely |
| Resource embedding | Tool returns can embed Resources for the LLM to read | "Show me the policy doc" + tool-call combined patterns |
| Title fields | Human-friendly names separate from technical IDs | Genie panel display; technical name = audit identifier |

#### Tech stack

- Transport: Streamable HTTP (default) + WebSocket upgrade
- Per-module server: TypeScript SDK (`@modelcontextprotocol/sdk`) or Rust (`mcp-rs`, CyberSkill-published)
- Federation router: Hono + custom MCP-aware reverse proxy
- Discovery: `/.well-known/mcp` + `/.well-known/oauth-protected-resource`
- OAuth: OAuth 2.1 + PKCE (S256); audience-bound tokens
- Registry: Postgres table; tool annotations validated on registration
- HITL: LangGraph `interrupt` gate for destructive tools

#### Tool annotations enforced

- `readOnlyHint=true`: executes without a confirmation prompt
- `destructiveHint=true`: HITL confirm UI; LangGraph interrupt
- `idempotentHint=true`: safe to retry; no double-execution
- `openWorldHint=true`: may communicate externally; annotated in audit
- `title="..."`: human-friendly label in the Genie panel

#### Architecture: federation, not monolith

MCP clients (Claude Desktop / Codex / Cursor - 12+ AI clients, all MCP 2025-11-25-compatible) discover the catalog at `/.well-known/mcp` (SEP-986), resolve the per-tenant `/.well-known/oauth-protected-resource`, and authenticate against the OAuth authorisation server (the AUTH module). Calls enter the gateway router over Streamable HTTP, hit the annotation-validated tool registry, and pass a destructive-tool check - destructive calls go through a LangGraph interrupt with user confirmation in the Genie panel - before being dispatched to the owning module's MCP server (`cyberos.memory.*`, `cyberos.chat.*`, `cyberos.proj.*`, `cyberos.rew.*` read-only, and the other 19). Module servers emit audit rows to NATS.

#### Flow: Claude Desktop invokes a destructive tool

1. Claude Desktop fetches `/.well-known/mcp` (servers, authz endpoint, scopes) and completes the OAuth 2.1 + PKCE flow against AUTH (audience = acme.cyberos.world), receiving an audience-bound access token.
2. It calls `tools/call { name: "cyberos.proj.delete_task", id: "T-42" }`.
3. The gateway looks up the annotation, sees `destructiveHint=true`, and interrupts through LangGraph (CUO): the Genie panel asks the Member "Confirm delete?".
4. On approval, the gateway forwards the call - with the Member's JWT - to the PROJ MCP server.
5. PROJ deletes the row tenant-scoped (`DELETE FROM tasks WHERE id=$1 AND tenant_id=current_setting('app.tenant_id')`), publishes `cyberos.acme.proj.task.deleted { id, actor, prev }` to NATS, and returns `{ ok: true }`.
6. The gateway returns the result ("Task T-42 deleted") to Claude Desktop.

### OAuth-Protected Resource Metadata

```
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
```

Note: resource-server-as-OAuth-client behaviour is forbidden - the prior pattern the security community flagged in 2025 ("token shadow handoff" via `X-Forwarded-Authorization`) is rejected at gateway level. Audience binding ensures an Acme tenant's token cannot be replayed at the Beta tenant's gateway even if leaked.

#### Status

P0 planned; build window: P0 slice 1 -> P0 exit.

- 2025-11-25 spec targeted (Tasks, Streamable HTTP, Elicitation, PRM)
- Per-module servers ship alongside subgraphs; reuse RBAC and the DB pool
- Tool annotations validated at registration; runtime check on call
- 12+ AI client compatibility tested (Claude products, Cursor, Cline, Codex); all MCP 2025-11-25-compatible

#### References

- Internal spec - MCP Gateway and the 2025-11-25 spec
- Internal spec - Authentication and authorisation
- Internal spec - Tool registry and per-module servers
- Internal spec - OAuth-Protected Resource and PRM flow
- [MCP 2025-11-25 spec](https://modelcontextprotocol.io/specification/2025-11-25)

## OBS - observability

Logs, metrics, traces - for every module, every gateway, every agent action. OBS is also the surface CUO's CTO skill reports against.

### Why a separate layer

In a 23-module platform with agent traffic on top of human traffic, you cannot answer "why is REW slow?" without tying together: a Member's request through the Apollo Router, the REW subgraph's DB query, a call out to the AI Gateway for a payslip narrative, an audit row to NATS, and a downstream LEARN subscription. OBS is the single trace tree that makes that legible.

OBS also feeds CUO's CTO skill: weekly OBS dashboard digests, security advisory pipelines, and model registry summaries all read from OBS (see the AI matrix).

#### Tech stack - LGTM (DEC-021)

- Loki: logs (open-source, self-hostable, S3-compatible storage)
- Grafana: dashboards (open-source visualisation)
- Tempo: distributed traces (OTel-native)
- Mimir: metrics (Prometheus long-term storage)
- Collector: OpenTelemetry Collector (OTel) at `:4317` gRPC
- Alerting: Grafana Alertmanager + PagerDuty webhook
- Agent traces: LangSmith for CUO session timelines
- Synthetic monitoring: Grafana k6 scripts in CI

#### Why these picks

- LGTM, not Datadog: cost predictability is the founder's first constraint; Datadog at 23-module ingest scales linearly with the bill.
- OTel-native: every subgraph + gateway speaks one wire format; trade providers later without rewriting instrumentation.
- S3 backend: Loki/Tempo/Mimir all store on the same R2/MinIO bucket as memory's archival layer; one cost model.
- LangSmith for agent runs: CUO's persona-version stamps, tool calls, and HITL gates need agent-aware UX; LangSmith is the lowest-friction tool.

#### Pipeline

Every CyberOS component - subgraphs, gateways, the CUO supervisor, the front-end host, and NATS / S3 / Postgres - emits to the OpenTelemetry Collector (gRPC :4317, HTTP :4318). The collector feeds Loki (logs), Tempo (traces), and Mimir (metrics), all on the S3 backend. Grafana unifies the three for dashboards and drives Alertmanager (PagerDuty webhook) plus the CUO CTO-skill weekly digest.

### Targets

| Metric | Target | Notes |
|---|---|---|
| Trace sampling | 10% | 100% for errors + LLM calls |
| Log retention | 30 d | 90 d audit; 7 y compensation |
| Alert MTTR | < 5 min | PagerDuty page -> Founder |
| OBS cost | <= $80/mo | internal scale |

#### Status

P0 planned; build window: P0 slice 2 -> P0 exit.

- LGTM stack via Grafana Cloud free tier in P0, self-host at P1
- OTel instrumentation in the module template; every new subgraph wired by default
- RED dashboard per subgraph (rate, errors, duration)
- CUO CTO-skill digest job emits a weekly summary to the Founder

#### References

- DEC-021 - LGTM observability stack
- OpenTelemetry semantic conventions 1.27+
- N(task pending) - p99 latency degradation budget (CI gate)

## GraphQL federation

Apollo Federation v2.5+. One supergraph, 22 subgraphs, one persisted-query budget. The agent surface and the human surface read the same schema.

### Why a separate layer (and why Apollo Federation specifically)

CyberOS rejects three alternative API postures. REST per module means the front-end host shell does N round-trips per page. BFF (backend-for-frontend) per module means N+M maintenance burdens. A single monolithic GraphQL means schema-merge conflicts every time a module ships. Apollo Federation v2.5+ solves all three: each module owns its subgraph SDL, the Router composes the supergraph at deploy time, and one HTTP roundtrip can pull from 5 modules in parallel.

Persisted queries are mandatory for production traffic. Query hashes are pre-registered at deploy; any unregistered query is rejected with a 400. This caps abuse, enables CDN caching, and lets each subgraph publish a query budget.

#### Tech stack

- Router: Apollo Router (Rust, OSS, MIT) v1.50+ - Federation v2.5+ compliant
- Subgraph servers: GraphQL Yoga (TypeScript) or async-graphql (Rust)
- Composition: `rover supergraph compose` in CI
- Persisted query store: GCS / R2 bucket (CDN-cached)
- Directives used: `@key`, `@external`, `@requires`, `@provides`, `@shareable`, `@inaccessible`, `@tag`
- Auth context: JWT validated at the Router; `tenant_id` + `actor` propagated to subgraphs as headers
- Caching: Apollo Router edge cache + Cloudflare CDN

#### Why these picks

- Apollo Router, not GraphQL Mesh: production-grade Rust runtime; query plan cache; the Federation v2.5 reference.
- Persisted queries mandatory: zero query injection surface; CDN-cacheable; rate-shaped.
- Federation v2.5: `@interfaceObject` (P3 module hierarchies), `@progressive @override` (zero-downtime schema moves).
- Schema deprecation discipline: removal requires >= 1 phase notice (N(task pending)); breaks no client mid-phase.

#### Composition and request path

The host shell (Module Federation; Vite, React 19, Tauri) posts persisted-query hashes to the Apollo Router (query plan, auth context, cache). An unregistered hash is rejected with a 400; a registered hash produces a query plan that fans out to subgraphs in parallel - memory, CHAT, PROJ, AUTH, and the other 18 - each backed by its own RLS-enforced Postgres schema.

#### Flow: "open my dashboard" pulls from 5 subgraphs in parallel

1. The Member opens /home; the host shell posts `{ hash: "0xabc...", vars }` to the Router.
2. The Router resolves the hash to DashboardQuery, validates the JWT, and extracts tenant_id + actor.
3. Parallel fanout: CHAT `{ unreadCount }`, PROJ `{ myTasks(limit:5) }`, CRM `{ myDeals(stage:OPEN) }`, REW `{ myPayslipStub }`, memory `{ memorySearch(query:"today") }`.
4. The Router merges the five responses into one `{ data }` payload; the host shell renders.

### Targets

| Metric | Target | Notes |
|---|---|---|
| GraphQL p95 | <= 400 ms | N(task pending) |
| Cache hit rate | >= 70% | N(task pending) persisted-query |
| Subgraph deploy | <= 10 min | N(task pending) module CI |
| Composition check | pass | N(task pending) per release |

#### Status

P0 planned; build window: P0 start -> P0 slice 1.

- Apollo Router scaffold + composition CI live at P0 start
- memory and AUTH subgraphs first to integrate at P0 slice 1
- Persisted query registration auto-bound to the host-shell build
- Schema deprecation policy in CONTRIBUTING.md

#### References

- DEC-002 - Apollo Federation v2.5+
- [Apollo Federation v2 docs](https://www.apollographql.com/docs/federation/)

## NATS JetStream

Every state-changing action emits an event. NATS JetStream is the spine. Durable consumers, tenant-scoped subjects, audit-grade retention.

### Why a separate layer

A 23-module platform needs to decouple write paths from downstream effects. When REW publishes a payslip, six things need to happen: memory ingests the narrative, LEARN updates the career-level snapshot, OBS emits a metric, CUO queues a "review your payslip" Notify, the Compliance audit row is hashed, and the Member's mobile gets a push. All six are events on the canonical subject `cyberos.acme.rew.payslip.published`.

The convention is locked: `cyberos.{tenant}.{module}.{entity}.{verb}`. Subjects are tenant-scoped so subscribers cannot accidentally cross tenant boundaries. Streams retain 30 days by default, 90 days for compensation/ESOP. CUO's ambient-trigger consumers subscribe through durable consumers so a restart never loses pending nudges.

#### Tech stack

- Broker: NATS Server v2.10+ with JetStream
- Client libs: `nats.go`, `@nats-io/nats.js`, `async-nats` (Rust)
- Schemas: CloudEvents 1.0 envelope + JSON Schema body per subject
- Schema registry: self-hosted; refs in subgraph CI
- Durable consumers: CUO ambient-trigger, OBS rollups, memory ingestion
- Replication: 3-node JetStream cluster per region
- DLQ: failed messages routed to `cyberos.{tenant}.dlq.{module}.{entity}.{verb}`

#### Why NATS, not Kafka / Redpanda

- Subject hierarchy native: Kafka has flat topics; NATS subjects (`cyberos.acme.proj.*`) match CyberOS conventions one-to-one.
- Latency: sub-millisecond pub/sub; Kafka adds tens of ms per consumer group.
- Footprint: a single 50 MB binary; no Zookeeper/Kraft cluster to operate at 10-Member scale.
- JetStream: adds Kafka-style durability without giving up subject wildcards.
- Cost: runs on a single $20/mo VM in P0; clusters at P3.

### Canonical subjects

```
# Format
cyberos.{tenant}.{module}.{entity}.{verb}

# Examples
cyberos.acme.proj.task.created
cyberos.acme.proj.task.assigned
cyberos.acme.proj.task.completed
cyberos.acme.rew.payslip.published    # 90-day retention
cyberos.acme.rew.bp_balance.updated
cyberos.acme.memory.fact.added
cyberos.acme.memory.fact.conflict_detected
cyberos.acme.crm.deal.stage_changed
cyberos.acme.chat.message.posted
cyberos.acme.audit.event.recorded     # Merkle-chained; 7y retention
cyberos.acme.ai.invoke.completed      # cost ledger
cyberos.acme.mcp.tool.invoked

# Durable consumers
- cuo-ambient      -> subscribes to *.task.* + *.deal.* + *.payslip.*
- memory-ingest    -> subscribes to all non-compensation events
- obs-rollup       -> subscribes to *.>
- compliance-audit -> subscribes to *.audit.>
- learn-snapshot   -> subscribes to rew.payslip.* + hr.level.*
```

#### Flow: REW publishes a payslip; six downstream consumers

1. The HR/Ops Lead publishes a payslip `{ memberId, monthEnd }`; REW inserts the payslip row and an audit_event (Merkle hash) in its Postgres.
2. REW publishes `cyberos.acme.rew.payslip.published` to NATS JetStream.
3. Fan-out, in parallel: memory ingests the narrative (compensation excluded by the denylist); LEARN snapshots career level + tenure; the compliance audit consumer appends to the per-scope Merkle chain; CUO nudges the Member to review the payslip; the mobile push notification goes out; OBS increments `payslips_published_total{tenant=acme}`.

### Targets

| Metric | Target | Notes |
|---|---|---|
| Pub latency p95 | < 5 ms | in-cluster |
| Default retention | 30 d | 90 d comp/ESOP; 7 y audit |
| Replication | R=3 | per-region JetStream cluster |
| DLQ replay | CLI | `cyberos dlq replay` |

#### Status

P0 planned; build window: P0 start -> P0 slice 1.

- Single-node NATS at P0 start; 3-node cluster at P1 exit
- Module template includes typed publisher/consumer helpers
- Schema registry CI-validated at subgraph PR-time
- Per-tenant subject ACLs at NATS level (defense in depth alongside RLS)

#### References

- DEC-004 - NATS JetStream events
- [NATS JetStream docs](https://docs.nats.io/nats-concepts/jetstream)

## End-to-end: a module makes a request

The six pillars are useful in isolation; they are load-bearing when composed. Below is a single end-to-end trace of one user action - Trinh, a Member, asks Genie "what should I work on today?" - passing through every pillar exactly once.

1. Trinh types "@genie what should I work on today?"; the host shell posts the persisted-query hash to the Apollo Router.
2. The Router validates the JWT against AUTH (JWKS cached) and resolves `{ tenant=cyberskill, actor=member:trinh }`.
3. The Router invokes `cuoAsk { prompt, threadId }`; CUO opens a "cuo.session.start" span in OBS and refreshes the tool registry from the MCP Gateway (`proj.list_my_tasks`, `memory.search`, ...).
4. Context retrieval runs in parallel through the MCP Gateway, with Trinh's JWT: `cyberos.proj.list_my_tasks(member=trinh, due_today=true)` returns 4 tasks from PROJ; `cyberos.memory.search("today priorities trinh")` has memory call the AI Gateway for a BGE-M3 embedding (self-hosted) and returns 5 relevant facts.
5. CUO sends the chat completion (persona=cuo-v3, model=sonnet) to the AI Gateway, which cost-checks, redacts, misses the cache, and invokes Bedrock via LiteLLM; the stream flows back to CUO.
6. CUO returns `{ answer, citations, persona_version }` through the Router; the host shell renders the Genie response.
7. CUO publishes `cyberos.cyberskill.cuo.session.completed` to NATS and closes the "cuo.session.end" span in OBS (latency=2.3 s, tokens=412, cost=$0.011).

### The six pillars, one trace

- AUTH: JWT validated at the Router; agent identity = Trinh's identity
- GraphQL: persisted-query lookup at the Router; auth context propagated to subgraphs
- MCP Gateway: tool discovery; per-module dispatch with Trinh's JWT
- AI Gateway: embedding + chat completion; cost ledger; persona stamping
- NATS: CUO session lifecycle published for downstream consumers
- OBS: one trace tree across the whole sequence

## References

#### CyberOS source documents

- Internal spec - The high-level system
- Internal spec - GraphQL Federation
- Internal spec - Module Federation (frontend)
- Internal spec - MCP Gateway and the 2025-11-25 spec
- Internal spec - Authentication and authorisation
- Internal spec - Tool registry and per-module servers
- Internal spec - OAuth-protected resource and PRM flow
- Internal spec - AI Gateway
- DEC-001..DEC-066 - locked decisions

#### External standards and specs

- [MCP 2025-11-25 specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [Apollo Federation v2 docs](https://www.apollographql.com/docs/federation/)
- RFC 7519 - JSON Web Token (JWT)
- RFC 7636 - PKCE for OAuth 2.0
- RFC 6749 - OAuth 2.0 / draft OAuth 2.1
- RFC 6238 - TOTP
- W3C WebAuthn Level 3
- RFC 6532 - Internationalized email (UTF-8 throughout)
- [NATS JetStream documentation](https://docs.nats.io/)
- OpenTelemetry semantic conventions 1.27+
- OWASP Generative AI Top 10 (2025-04)
- NIST AI 600-1 - Generative AI Risk Profile

## Changelog

History lives in the [changelog](../reference/changelog.html); this page describes only the current state.
