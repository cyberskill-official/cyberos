# CyberOS — Software Requirements Specification (SRS)

**Project:** CyberOS — AI-Native Internal Operations Platform
**Owner:** CyberSkill Software Solutions Consultancy and Development Joint Stock Company (Vietnam, [cyberskill.world](https://cyberskill.world)) · *"Turn Your Will Into Real"*
**Document type:** Software Requirements Specification — official **v1.0** (single source of truth alongside PRD.md)
**Status:** Approved · 2026-04-28
**Doc ID:** `CYBEROS-SRS-1.0`
**Audience:** Module Owners, Engineering Contributors, Security Reviewers, External Auditors, Compliance Working Group, 3PAOs, future Tenant Engineering teams (P4)
**Companion document:** [PRD.md](./PRD.md)
**Legal source of truth:** Total Rewards & Career Path Appendix (referenced for REW / LEARN / ESOP modules)

This SRS is one of two documents that together govern CyberOS. There is no other documentation. Architectural Decision Records, Compliance Strategy, runbooks, and module READMEs are **folded into this document** as numbered sections. There are no separate ADR files; DEC-{NNN} entries live in §3.3.

---

## Table of Contents

0. Document Control & Distribution
1. Introduction
2. System Actors & External Interfaces
3. Architectural Drivers & Locked Decisions (37 DECs)
4. Per-Module Specifications (22 modules)
5. Cross-Module Federation & Data Boundaries
6. AI Integration Architecture
7. Security & Compliance NFRs
8. System-Wide Non-Functional Requirements
9. Verification & Acceptance
10. Engineering Delivery Organization
11. Compliance Strategy (full)
12. Appendices

---

## 0. Document Control & Distribution

### 0.1 Distribution Matrix (Scope Anchor)

| Component | Document | Rationale |
|---|---|---|
| Module value proposition & UX | [PRD](./PRD.md) | Iterative based on user reality |
| User flows | PRD | Behavior, copy, screen-state decisions |
| AI productivity features (product behavior) | PRD | Personalization changes with usage |
| Genie persona & mascot | PRD | Brand-strategic |
| Phase entry/exit criteria | Both | PRD owns "what's the gate"; SRS owns "what's verified" |
| Compliance tier model & cert sequence | PRD §10 + SRS §11 | Strategic in PRD; operational in SRS |
| Module API contracts (GraphQL SDL) | **SRS** | Federation interface — change-controlled |
| Data models (Prisma schema) | **SRS** | Migration risk; deploy-window discipline |
| Tenancy isolation & RLS policy | **SRS** | Security-critical; immutable boundary |
| AI integration architecture | **SRS** | Latency budgets, model contracts |
| Security & Compliance NFRs | **SRS** | Auditable; pen-test boundary |
| Locked Decisions (37 DECs) | **SRS §3.3** | Immutable architectural posture; replaces ADR files |

### 0.2 Tag Legend

- **[FIXED]** — durable principle; change-controlled
- **[DYNAMIC]** — operational state; updated freely

### 0.3 Priority Taxonomy (unified with PRD)

| MoSCoW | Engineering Priority | Phase Status | Definition |
|---|---|---|---|
| Must | Critical | In current phase | Phase-blocking |
| Should | High | In current phase | One-iteration slip tolerable |
| Could | Medium | Stretch | Ships if module team has capacity |
| Won't | Out of Phase Scope | Deferred | Excluded from current phase |

### 0.4 Verification Methods (RFC 2119 + IEEE 1233)

| Method | Code | Description |
|---|---|---|
| Test | T | Automated test in CI (unit, integration, e2e) |
| Demonstration | D | Manual demonstration in a phase-gate review |
| Inspection | I | Code/configuration review |
| Analysis | A | Static analysis, threat modeling, math/proof |

Each FR/NFR ID below carries a verification method tag like `[T]`, `[D]`, `[I]`, `[A]`.

### 0.5 ID Conventions [FIXED]

- Functional requirements: `FR-{MOD}-{NNN}` (e.g., `FR-PROJ-001`)
- Non-functional requirements: `NFR-{CAT}-{NNN}` using ISO/IEC 25010:2023 categories
- Use cases: `UC-{MOD}-{NNN}`
- Constraints: `CON-{NNN}`; Assumptions: `ASM-{NNN}`; Risks: `RSK-{NNN}`; Open Questions: `OQ-{NNN}`; Decisions: `DEC-{NNN}`
- Module codes: `AUTH, AI, MCP, OBS, CHAT, BRAIN, GENIE, PROJ, TIME, CRM, KB, HR, EMAIL, REW, LEARN, INV, ESOP, RES, OKR, DOC, CP, TEN`
- IDs are persistent; deletion sets status `Deprecated`, never reuses the number

---

## 1. Introduction

### 1.1 Purpose [FIXED]

This Software Requirements Specification (SRS) defines the technical contract for CyberOS — its architecture, module APIs, data models, security NFRs, AI integration, compliance posture, and verification criteria. It is the source of truth for engineering implementation across all phases (P0–P4). Together with the PRD, it is the only governing document for CyberOS — there are no separate ADR files, no separate compliance-strategy doc, no separate runbooks at v1.0.

### 1.2 Scope [FIXED]

CyberOS is a multi-tenant operational platform composed of **22 independently deployable modules**. Each module is:
- A backend subgraph (Apollo Server 5 + Express + `@apollo/subgraph`)
- A frontend remote (Module Federation)
- A Prisma-managed Postgres schema slice
- A set of MCP tools
- A NATS event producer + consumer

Modules are composed into a single user experience by:
- A **GraphOS Router** federating subgraphs
- A **Module Federation Shell** composing remotes
- A **Central MCP Server** registering each module's tools
- The **Genie** UI surface portable across every MFE remote

This SRS covers the contract between modules, the platform-provided cross-cutting services, and the verification criteria for each phase. CyberSkill is the only tenant through P3 (DEC-026); external tenant signup, billing, and Tenant Admin UX are P4 scope.

### 1.3 Reference Standards [FIXED]

| Standard | Use |
|---|---|
| IEEE 830-1998 SRS | Document structure |
| ISO/IEC/IEEE 29148:2018 | Requirements engineering |
| ISO/IEC 25010:2023 | NFR taxonomy |
| RFC 2119 | MUST/SHOULD/MAY semantics |
| RFC 6749, RFC 7636 | OAuth 2.1, PKCE |
| RFC 9728 | OAuth 2.0 Protected Resource Metadata |
| RFC 6238 | TOTP |
| RFC 3501 | IMAP4 (EMAIL module) |
| RFC 5321 | SMTP (EMAIL module) |
| RFC 5322 | Internet Message Format (EMAIL module) |
| RFC 5545 | iCalendar (EMAIL module calendar parse) |
| RFC 6455 | WebSocket (CHAT module) |
| W3C DID | Future tenant-portable identity (P4+) |
| Apollo Federation v2.7+ | Subgraph composition |
| MCP Spec 2025-11-25 | Agent surface |
| OWASP Top 10:2021 + ASVS 4.0 | Security baseline |
| WCAG 2.2 AA | Accessibility baseline |
| NIST CSF 2.0 | Security framework |
| Vietnamese PDPL 91/2025/QH15, Decree 356/2025/ND-CP | Data protection (CON-004 driver) |
| Vietnam AI Law 134/2025 | AI governance |
| EU AI Act (Regulation 2024/1689) Annex III §4 | High-risk for REW + LEARN |
| EU GDPR + UK GDPR | Data protection |
| EU NIS2 Directive 2022/2555 | Cybersecurity incident notification |
| EU Data Act Regulation 2023/2854 | Switching APIs |
| ISO 27001:2022 + 27017 + 27018 + 27701 + 42001 | Information security + cloud + AI |
| ISO 22301:2019 | Business continuity |
| C2PA Content Credentials v2.0 | AI provenance |

---

## 2. System Actors & External Interfaces [FIXED]

### 2.1 External Actors

| Actor | Channel | Auth | Notes |
|---|---|---|---|
| **Member (web)** | Vite shell + MFE remotes | OAuth 2.1 / password + mandatory TOTP | Primary daily user |
| **AI agent (Claude Desktop, internal Member)** | MCP via Streamable HTTP | OAuth 2.1 + PKCE | Same RBAC as Member |
| **AI agent (3rd-party, P4)** | MCP via Streamable HTTP | OAuth 2.1 + PKCE + tenant consent | Customer's GPT/Claude/Gemini |
| **Genie** (CyberOS internal) | Same MCP path as agents | Member's JWT (delegated) | Always confirm-step on writes |
| **Stripe** | REST + Webhooks | API key + webhook HMAC | INV (P2) |
| **Google Workspace (Gmail/Calendar)** | OAuth 2.1 | OAuth | EMAIL (P1) |
| **Microsoft 365 (Outlook/Calendar)** | OAuth 2.1 | OAuth | EMAIL (P1) |
| **Generic IMAP/SMTP** | IMAP/SMTP | App password (envelope-encrypted) | EMAIL (P1) |
| **AWS Bedrock** | REST | IAM | Primary LLM provider |
| **OpenAI / Anthropic** | REST | API key (per-tenant override) | LLM fallback via AI Gateway |
| **HIBP API** | REST | API key | Password breach checks |
| **Postmark / Amazon SES** | REST/SMTP | API key | Outbound transactional email |
| **DocuSign EU / Adobe Sign EU / Yousign / SES eIDAS QTSP** | OAuth + Webhooks | OAuth | DOC (P4) |
| **S3 / Cloudflare R2 / VN object storage** | REST | IAM role / token | Object storage (per-tenant residency) |
| **Zalo OA** | OAuth (P2 stretch) | OAuth | VN messaging notifications |
| **Board** | UI dual-sign workflow | OAuth + Board role | ESOP valuation publishing, SP grants |
| **GitHub** | OAuth + Webhook | OAuth | Source code, CI |
| **Sprinto / Drata / Vanta** | API | API key | Continuous compliance evidence (P1+; one of three by P1 exit) |
| **Lakera / Protect AI** | API | API key | Prompt injection defense (P2+) |
| **Fairlearn / Aequitas / AIF360** | Library | n/a | Bias testing (P2+) |
| **VN GDT e-invoice provider** | API | API key | INV Vietnamese e-invoice (P2; OQ-016) |

### 2.2 Internal Composition Boundary

```
                 ┌──────────────────────────────────────────────────┐
                 │  Vite Shell (Module Federation host)             │
                 │  + Genie portable component                      │
                 └────────────────────────────────────────────────┘
                              │           │              │
                              │ MFE       │ Genie ⌘+G    │ Slash
                              ▼           ▼              ▼
                 ┌──────────────────────────────────────────────────┐
                 │  Per-module MFE remotes (Vite + React)           │
                 └────────────────────────────────────────────────┘
                              │
                              │ HTTPS GraphQL
                              ▼
   ┌──────────────────────────────────────────────────────────────┐
   │  Apollo GraphOS Router (Rust)                                 │
   └──────────────────────────────────────────────────────────────┘
                              │
                              │ gRPC/HTTP per subgraph
                              ▼
   ┌──────────────────────────────────────────────────────────────┐
   │  Module Subgraphs (Apollo Server 5 + Express, 22 modules)     │
   └──────────────────────────────────────────────────────────────┘
                              │
                              │ Prisma
                              ▼
   ┌──────────────────────────────────────────────────────────────┐
   │  PostgreSQL 17 + pgvector + PGroonga + pg_jsonschema           │
   │  (per-tenant residency cluster: VN / EU / US / APAC)           │
   └──────────────────────────────────────────────────────────────┘
                              │
   ┌──────────────────────────┼───────────────────────────────────┐
   │ Redis (BullMQ jobs, presence, Socket.IO adapter, embedding cache) │
   │ NATS JetStream (domain events; BRAIN ingestion)               │
   │ S3/R2/VN-object (per-tenant residency)                        │
   │ Socket.IO sidecar (CHAT realtime)                             │
   │ IMAP/SMTP workers (EMAIL sync sidecar)                        │
   │ AI Gateway → Bedrock / OpenAI / Anthropic                     │
   │ MCP Server (Express + Apollo Server 5 thin wrapper)           │
   │ New Relic APM + AIM + Logs in Context                          │
   └──────────────────────────────────────────────────────────────┘

External clients via MCP Streamable HTTP:
   Claude Desktop, Internal Genie, 3rd-party agents (P4)
```

---

## 3. Architectural Drivers & Locked Decisions [FIXED]

### 3.1 Quality Attribute Drivers (Priority Ranked)

| Rank | Quality Attribute | Driver Statement | Influence on Architecture |
|---|---|---|---|
| 1 | Security (Tenant Isolation) | Tenant data leak = sev-0; commercialization-blocker | 3-layer enforcement (JWT/app/RLS); RLS as primary DB control |
| 2 | Compensation Fidelity | REW/ESOP math errors break legal social contract; sev-0 | Deterministic engine; LLM never in math path; parameter versioning; recompute-identical invariant |
| 3 | Independent Deployability | Each module owned end-to-end; teams must deploy without coordination | Federation v2 + Module Federation; per-module CI/CD |
| 4 | AI-Native Operability | Every action callable by an MCP tool with same RBAC; Genie omnipresent | MCP wraps GraphQL; no duplicate business logic; Genie as portable component |
| 5 | Knowledge Coverage | BRAIN p95 ingest ≤5s from source event | NATS event-driven; per-source consumer; pgvector HNSW |
| 6 | Modifiability | Module Owners must be able to evolve their domain | Schema-first SDL with progressive `@override`; Prisma migrations per module |
| 7 | Performance | p95 GraphQL ≤400ms; CHAT p95 deliver ≤200ms; Genie p95 ≤2s | DataLoader caching; pgvector HNSW; APQ; Socket.IO Redis adapter |
| 8 | Reliability | 99.5% MVP, 99.9% post-commercial | Graceful degradation per module; provider fallback for LLMs; IMAP IDLE supervised |
| 9 | Cost Discipline | ≤$380/mo internal infra at P2 | Free-tier Postgres alternatives; embedding cache; GraphOS Free |
| 10 | Internationalization | vi-VN default, en-US parity; **Vietnamese prevails on legal conflict** | ICU MessageFormat; i18n library per remote; comp surfaces VN-toggleable |
| 11 | Usability/Accessibility | WCAG 2.2 AA | shadcn + Radix primitives; keyboard-first design |
| 12 | Compliance-by-Construction | T1 Floor live at P0 exit; T3 conformity at P2 exit | Per-tenant residency; A05 filings; AI Act conformity pack; Trust Center |

### 3.2 Six Core Principles [FIXED]

1. **Modular monolith → federation hybrid.** In dev, modules can run together via local docker-compose. In prod, each module is its own container with its own subgraph.
2. **No cross-module DB reads.** Modules reference each other only via GraphQL `@key` entity refs or NATS events.
3. **Tenant isolation in 3 layers:** JWT claim → app middleware → PostgreSQL RLS policy.
4. **Schema-first SDL with codegen.** Federation directives are SDL-native; schemas reviewed in PR.
5. **MCP tools wrap GraphQL.** No duplicate business logic between MCP and GraphQL paths.
6. **AI-native by composition.** Every write tool emits an audit event; every read tool is RBAC-enforced and tenant-scoped. Genie + agents never escape this discipline.

### 3.3 Locked Technology Decisions [FIXED] (DEC-001..DEC-037)

This table replaces the prior ADR file collection. Each entry is a binding architectural decision with full context, rationale, and trade-offs. Once **Accepted**, an entry is immutable; superseding decisions get a new DEC number and reference the prior one.

| ID | Decision | Choice | Rationale & Trade-off |
|---|---|---|---|
| **DEC-001** | Backend per-module framework | **Apollo Server 5 + Express** (not NestJS) | Lean ownership; Express familiarity for new contributors; explicit DI per team; smaller cold start. Trade: more boilerplate per module than NestJS provides; mitigated by `_template/` scaffold. |
| **DEC-002** | Frontend module strategy | **Module Federation from day 1** | Per-module independent UI deployment matches backend independence. Trade: complexity in cross-module navigation; mitigated by shell routing. |
| **DEC-003** | Plan structure | **Phase-based** (P0..P4) | Reality-driven adjustment without breaking the plan. Trade: stakeholders must accept ambiguous timelines. |
| **DEC-004** | Database | **PostgreSQL 17 + pgvector + PGroonga + pg_jsonschema** | MVCC for invoicing; RLS for tenancy; pgvector at scale; PGroonga for VN search. Trade: single-engine eggs in one basket; mitigated by per-region cluster. |
| **DEC-005** | ORM | **Prisma 5.x** | Best TS types; mature migrations; Apollo-friendly. Trade: less raw SQL ergonomics; mitigated by `prisma.$queryRaw` escape hatch. |
| **DEC-006** | Hosting (regional) | **VN: Viettel IDC / FPT Smart Cloud / VNG Cloud / AWS Hanoi LZ; EU/US/APAC: Neon + Railway/Fly + Upstash + R2 + Cloudflare** | Vietnam Decree 53/2022 Art. 26.2 + Cybersecurity Law 24/2018 Art. 26.3 require VN-tenant data on VN-based infra. Railway/Neon US-only is non-compliant for VN-citizen data. |
| **DEC-007** | MCP transport | **Streamable HTTP, single endpoint, OAuth 2.1 + PKCE** | Production-grade; agent compatibility; per RFC 9728. |
| **DEC-008** | Module Federation runtime | **Rspack** (Webpack 5 compatible) | Faster build; same module-federation contract. |
| **DEC-009** | Inter-module event bus | **NATS JetStream** | Cost; simple ops; sufficient throughput at our scale. Trade: less feature-rich than Kafka; mitigated by reuse for BRAIN ingestion. |
| **DEC-010** | Async work queue | **BullMQ on Redis (Upstash)** | Lightweight; integrates with existing Redis. |
| **DEC-011** | Tenant residency model | **Per-tenant `residency` enum {VN, EU, US, APAC, OTHER}; routing layer dispatches reads/writes to the regional cluster** | VN sovereignty + GDPR data localization + customer choice for enterprise. |
| **DEC-012** | AI-derived data classification | **Treated as personal data** (embeddings, LLM logs, agent traces, BRAIN chunks) | Vietnam Decree 356/2025 Art. 30. DSAR erasure must extend to all of these. |
| **DEC-013** | Federal/defense path | **Declined** (CMMC, FedRAMP High, IL4+, ITAR, IRS Pub 1075) | VN ownership + VN-based personnel structurally incompatible without US sub + US-citizen ops. |
| **DEC-014** | Public-sector substitute path | **TX-RAMP → StateRAMP Cat 2 → FedRAMP 20x Moderate (no-sponsor)** | Realistic given DEC-013; reuses SOC 2 + ISO 27001 evidence. |
| **DEC-015** | Cert sequence | **SOC 2 Type II → ISO 27001:2022 → ISO 42001 within 18 months** | ~80% evidence reuse; phased de-risks audit findings. |
| **DEC-016** | Document signing approach | **eIDAS 2.0 QTSP integration** (DocuSign EU / Adobe Sign EU / Yousign / SES eIDAS) | Faster path to QES; regulated commercial unlocks (legal, banking). |
| **DEC-017** | Corporate structure | **Singapore HoldCo + VN OpCo flip at ARR $1.5–2M** | Standard VN-founder pattern; preserves 0% VAT export-of-services; avoids GILTI/Subpart F until US Tier-1 VC. |
| **DEC-018** | LLM provider stance | **AWS Bedrock primary; OpenAI + Anthropic via direct API + ZDR; geofence DeepSeek/CN-hosted models for non-CN customers** | ZDR + BAA availability; HIPAA via Bedrock+Anthropic; GDPR-incompatible providers excluded. |
| **DEC-019** | Subtask depth (PROJ) | **Unlimited** | Asana/Linear competitive parity at P1; UI flattens deep trees. |
| **DEC-020** | Boards + Sprints in PROJ | **Both ship in P1** | Dogfooding requires sprints; sprint entity reused by OKR module in P3. |
| **DEC-021** | Federation registry | **Apollo GraphOS Free for P0–P2; Hive Gateway escape hatch** | Free tier covers <100M ops/month; pricing-risk mitigation. |
| **DEC-022** | Vietnamese FT search | **tsvector + PGroonga + pgvector hybrid (RRF, k=60)** | Quality vs setup overhead; PGroonga handles VN tokenization. |
| **DEC-023** | Compensation/equity field encryption | **App-layer envelope + per-tenant KMS data keys; BYOK at T3** | DB compromise alone does not leak comp; BYOK upgrade for T3 enterprise. |
| **DEC-024** | MCP rate limits | **Free 60/min, Starter 300, Pro 1200, Enterprise custom; 2× burst over 10s** | Tier-based; ~5× lower than GraphQL API limits. |
| **DEC-025** | MFE framework | **Vite + React for remotes; Vite shell** | Faster builds, smaller bundles; remotes are auth'd internal tools (no SSR needed). |
| **DEC-026** | Internal-first scoping | **Multi-tenant arch retained; external sale deferred to P4** | Optimize P0–P3 for CyberSkill; preserve compliance investments; no refactor cost when external GA opens. |
| **DEC-027** | Communication scope | **CHAT full Slack-clone (P0) + EMAIL full IMAP/SMTP client (P1)** | Replace external comms tools immediately; AI agents need unified comms surface. |
| **DEC-028** | Total Rewards split | **REW + LEARN + ESOP as 3 modules; HR promoted from P2 to P1** | Distinct cycles + ownership; AI Act high-risk scoping; HR stays small. |
| **DEC-029** | Phantom Stock ledger | **Immutable append-only event sourcing** | Audit-grade; reproduces past valuations; M&A acceleration as event. |
| **DEC-030** | 3P payroll engine | **Deterministic; LLM never in math path; LLM narrates only via PII-safe `rew.payslip_explain`** | P1 protection invariant; AI Act conformity; auditable. |
| **DEC-031** | Parameter versioning | **First-class `parameter_version` per REW/LEARN/ESOP/GENIE** (immutable; UPDATE/DELETE blocked at DB-policy level) | Article 7a non-retroactive enforcement; reproducible recompute. |
| **DEC-032** | CHAT realtime stack | **Socket.IO + Redis presence + Postgres append-only with Merkle audit chain** | Stack-native; per-tenant residency; PGroonga search; audit-grade. |
| **DEC-033** | GENIE + BRAIN architecture | **Two separate P0 modules (GENIE for persona/UX; BRAIN for universal vector index)** | Module independence; reusable BRAIN by any AI consumer; clean AI Act scoping. |
| **DEC-034** | BRAIN ingestion model | **Event-driven auto-embed via NATS consumers; BullMQ-queued; per-source handler** | Realtime freshness; module independence preserved; idempotent on `(source_module, entity_id, version)`. |
| **DEC-035** | Genie persona | **Versioned (parameter-version pattern)** with explicit voice + behavior rules; Founder/CEO + Engineering Lead dual-sign to publish new versions | Brand consistency; AI Act transparency; reproducible historical conversations. |
| **DEC-036** | BRAIN data classification | **Explicit allowlist (CHAT, PROJ, CRM, KB, EMAIL summaries, HR non-comp, LEARN training/outcomes)** + **denylist (REW, ESOP, HR comp, special-category)** + **conditional opt-in (email body, DM, leave reason)** | Member trust; AI Act high-risk avoidance for Genie; DSAR cascade simplicity. |
| **DEC-037** | BRAIN vector index | **Postgres + pgvector HNSW + PGroonga + tsvector with RRF k=60** per-tenant residency cluster | Stack-native; reuses existing infrastructure; supports VN search. |

### 3.4 Decision Maintenance Process

Adding a new DEC: PR with the new entry against this table, full Rationale & Trade-off, signed by Engineering Lead + ≥1 other reviewer (CWG rep if compliance-tagged). Existing DECs are immutable; supersession adds a new DEC referencing the prior one.

---

## 4. Per-Module Specifications

Each module section includes: purpose, FRs, data model (Prisma sketch), GraphQL SDL highlights, MCP tools, NFRs, dependencies. Modules are listed in phase order.

### 4.1 Module: AUTH — Authentication & Tenancy [P0] [FIXED]

**Purpose:** Identity, tenant resolution, RBAC. All other modules depend on it.

**Functional requirements:**

- `FR-AUTH-001` [T] **MUST** support email + password sign-in. Passwords stored as Argon2id (memory=64MB, iterations=3, parallelism=4). HIBP breach check on password set/change.
- `FR-AUTH-002` [T] **MUST** support Google OAuth 2.1 OIDC sign-in. Microsoft 365 OAuth supported as P1 add for EMAIL provider linking.
- `FR-AUTH-003` [T] **MUST** issue RS256 JWT with claims `sub`, `tid`, `roles[]`, `scopes[]`, `residency`, `exp` ≤24h. Rotation with refresh-token replay detection.
- `FR-AUTH-004` [T] **MUST** support Member invitation via single-use token (7-day expiry).
- `FR-AUTH-005` [T] **MUST** support role assignment from `{owner, admin, member, viewer, client, board, hr_lead, account_manager, engineering_lead}`. Multi-role per Member supported.
- `FR-AUTH-006` [T] **MUST** require TOTP MFA enrollment for **all roles** before first non-MFA action (NYDFS Part 500 §11 alignment).
- `FR-AUTH-007` [T] **MUST** propagate `authorization: Bearer <jwt>` and `x-tenant-id` to every subgraph + MCP request.
- `FR-AUTH-008` [T] **MUST** enforce `app.tenant_id` Postgres session var per request before any data query.
- `FR-AUTH-009` [T] **MUST** reject sign-in attempts after 5 consecutive failures within 15 minutes (lockout with exponential backoff).
- `FR-AUTH-010` [T] **MUST** maintain a session revocation list reflected within 60 seconds across all subgraphs.
- `FR-AUTH-011` [T] [DEC-026] **MUST** allow only Founder/CEO role to create new tenants through P3; the Tenant Admin self-create flow is gated to P4.
- `FR-AUTH-012` [T] **MUST** support per-Member API tokens (limited scope, audited, rotatable, expiry configurable).
- `FR-AUTH-013` [T] **MUST** support fine-grained scope claims (`projects:read`, `projects:write`, `compensation:read`, `compensation:write`, `esop:read`, `esop:write`, `genie:invoke`, etc.).
- `FR-AUTH-014` [T] **MUST** support self-service password reset via email link (single-use, 1h expiry).
- `FR-AUTH-015` [T] [P2] **SHOULD** support WebAuthn/passkeys as MFA option.
- `FR-AUTH-016` [T] **MUST** apply tenant `residency` from the JWT to dispatch all queries to the regional cluster (DEC-011).

**Data model (Prisma sketch):**

```prisma
model Tenant {
  id          String   @id @default(uuid())
  slug        String   @unique
  name        String
  residency   Residency  // VN | EU | US | APAC | OTHER
  status      TenantStatus  // active | suspended | archived
  createdAt   DateTime @default(now())
}

enum Residency { VN EU US APAC OTHER }

model Member {
  id              String   @id @default(uuid())
  tenantId        String
  email           String
  emailVerified   Boolean  @default(false)
  passwordHash    String?
  totpSecretEncrypted Bytes?
  fullName        String
  status          MemberStatus  // active | invited | suspended | terminated
  roles           String[]   // owner, admin, member, ...
  scopes          String[]
  createdAt       DateTime @default(now())
  @@unique([tenantId, email])
}

model Session {
  id              String   @id @default(uuid())
  memberId        String
  tenantId        String
  refreshTokenHash String
  userAgent       String?
  ip              String?
  createdAt       DateTime @default(now())
  expiresAt       DateTime
  revokedAt       DateTime?
}

model Invitation {
  id        String   @id @default(uuid())
  tenantId  String
  email     String
  roles     String[]
  tokenHash String
  invitedBy String
  expiresAt DateTime
  acceptedAt DateTime?
}
```

**RLS:** every table has `tenant_id` and a policy `USING (tenant_id = current_setting('app.tenant_id')::text)`.

**MCP tools:** `auth.whoami`, `auth.list_members`, `auth.get_session`, `auth.list_sessions`, `auth.revoke_session`, `auth.invite_member`

### 4.2 Module: AI — AI Gateway [P0] [FIXED]

**Purpose:** Single internal abstraction for LLM calls; centralizes budget, redaction, telemetry, provider fallback, C2PA signing.

**Functional requirements:**

- `FR-AI-001` [T] **MUST** expose `aiGateway.complete({prompt, model, temperature, max_tokens, stream})` for chat completion.
- `FR-AI-002` [T] **MUST** expose `aiGateway.embed({text, model})` returning a vector.
- `FR-AI-003` [T] **MUST** expose `aiGateway.tool_call(...)` for function-calling support.
- `FR-AI-004` [T] **MUST** enforce per-tenant monthly USD budget cap; reject calls when exceeded with 402-equivalent error; alert via OBS at 80% threshold.
- `FR-AI-005` [T] **MUST** redact PII (emails, phone, ID numbers, comp values, ESOP balances, BP balances, special-category) by default; routes can opt out only with explicit `pii_safe_route` scope.
- `FR-AI-006` [T] **MUST** cache embeddings by `sha256(text)` in Redis with 30-day TTL.
- `FR-AI-007` [T] **MUST** auto-fallback Bedrock → OpenAI → Anthropic on provider 5xx within retry budget (default 3 retries with exponential backoff).
- `FR-AI-008` [T] **MUST** geofence requests originating from China-based tenants away from non-CN-hosted models (DEC-018).
- `FR-AI-009` [T] [DEC-030] **MUST** never be invoked from REW/ESOP/LEARN math paths. CI lint rule enforces (`@cyberos/eslint-config` rule `no-ai-in-math`).
- `FR-AI-010` [T] **MUST** log every LLM call to OBS with cost, latency, token counts; mark AI-derived outputs as PD per DEC-012.
- `FR-AI-011` [T] **MUST** sign every AI output destined for end-user consumption with a C2PA manifest (CN GB 45438, CA SB 942, VN AI Law Art. 10, EU AI Act Art. 50 conformity).
- `FR-AI-012` [T] **MUST** maintain a model registry (`model_registry` table) tracking provider, version, modality, training-data summary, eval results, deprecation date.
- `FR-AI-013` [T] **SHOULD** support per-tenant model preference override (tenant-level config).

**Data model (Prisma sketch):**

```prisma
model ModelRegistry {
  id            String   @id @default(uuid())
  provider      String   // bedrock | openai | anthropic
  modelId       String
  modality      String   // text | embed | vision | tool_call
  version       String
  trainingDataSummary String?
  evalResults   Json?
  cardUrl       String?  // KB doc reference
  deprecatedAt  DateTime?
  @@unique([provider, modelId, version])
}

model AiCallLog {
  id           String   @id @default(uuid())
  tenantId     String
  memberId     String?
  agentKind    String?  // genie | claude_desktop | 3rd_party
  modelUsed    String
  tokensIn     Int
  tokensOut    Int
  costUsd      Decimal
  latencyMs    Int
  c2paManifestUrl String?
  redactionApplied String[]
  status       String   // success | error
  errorClass   String?
  occurredAt   DateTime @default(now())
  @@index([tenantId, occurredAt(sort: Desc)])
}
```

**MCP tools:** `ai.complete`, `ai.embed`, `ai.list_models`, `ai.get_budget`, `ai.list_call_logs` (admin)

### 4.3 Module: MCP — MCP Server [P0] [FIXED]

**Purpose:** Agent-platform interface; single endpoint for any agent to drive CyberOS.

**Functional requirements:**

- `FR-MCP-001` [T] **MUST** expose Streamable HTTP at `https://mcp.cyberos.vn/mcp` (regional residency mirrors at `mcp-eu`, `mcp-us`, `mcp-apac`).
- `FR-MCP-002` [T] **MUST** authenticate via OAuth 2.1 + PKCE; per RFC 9728 publish protected-resource metadata.
- `FR-MCP-003` [T] **MUST** name tools `module.action` snake_case.
- `FR-MCP-004` [T] **MUST** wrap each tool call as a thin GraphQL mutation/query — no duplicate business logic.
- `FR-MCP-005` [T] **MUST** support the Tasks primitive for >10s long-running operations (PDF render, valuation re-compute, mailbox bulk import, BRAIN reindex).
- `FR-MCP-006` [T] **MUST** expose per-tenant resources via `cyberos://{module}/{path}` URIs (e.g., `cyberos://kb/{doc_id}`, `cyberos://chat/{channel_id}`, `cyberos://proj/{task_id}`, `cyberos://crm/contacts/{id}`).
- `FR-MCP-007` [T] **MUST** rate-limit per tier per DEC-024.
- `FR-MCP-008` [T] **MUST** emit audit log for every write tool with `actor_kind` (user|agent|system|genie), `actor_id`, `tool`, `params`, `before`, `after`, `tenant_id`, `occurred_at`.
- `FR-MCP-009` [T] **MUST** support OAuth scope-down flow so Member can delegate a 3rd-party agent with reduced scopes (P4).
- `FR-MCP-010` [T] **MUST** publish prompt templates ("Weekly OKR review", "Monthly payroll close", "New client kickoff", "Quarterly Board pack") as MCP Prompts.
- `FR-MCP-011` [T] **MUST** provide self-describing tool enumeration for any MCP client.

### 4.4 Module: OBS — Observability [P0] [FIXED]

**Purpose:** Operational visibility; SLO tracking; alerting; payroll-anomaly detection; BRAIN ingestion lag monitoring.

**Functional requirements:**

- `FR-OBS-001` [T] **MUST** install New Relic Node.js agent on every subgraph + MCP server + Socket.IO sidecar + EMAIL workers + BRAIN ingestion workers + Genie service.
- `FR-OBS-002` [T] **MUST** enable AI Monitoring (AIM) auto-tracing for Bedrock/OpenAI/Anthropic SDK calls.
- `FR-OBS-003` [T] **MUST** structured-log via pino with OTLP export; correlation ID per request propagated through subgraph + MCP + workers.
- `FR-OBS-004` [T] **MUST** define SLOs per module (latency, error rate, freshness) and track 28-day error budget.
- `FR-OBS-005` [T] **MUST** route NRQL alerts to CHAT `#cyberos-alerts`; sev-0 also pages PagerDuty.
- `FR-OBS-006` [T] [DEC-030] **MUST** run nightly compensation guardrail check: P1 reduction = 0; P3 cap respected on every payslip; BP ledger balance reconciles to events; ESOP event chain valid.
- `FR-OBS-007` [T] **MUST** monitor IMAP IDLE connection health per active mailbox; supervised restart on disconnect within 30s.
- `FR-OBS-008` [T] **MUST** monitor BRAIN ingestion lag p95; alert if >5s.
- `FR-OBS-009` [T] **MUST** monitor BRAIN denylist enforcement: any chunk with `source_module IN ('rew', 'esop')` is sev-0.
- `FR-OBS-010` [T] **MUST** monitor Genie answer-quality SLO: source-citation rate ≥98%; "I don't know" rate spike triggers alert.
- `FR-OBS-011` [T] **MUST** publish daily Merkle root of audit_log to Trust Center for tamper detection.
- `FR-OBS-012` [T] **MUST** retain logs ≥7 years for PDPL + financial records; legal hold capability.

**MCP tools:** `obs.list_alerts`, `obs.get_slo_status`, `obs.acknowledge_alert`

### 4.5 Module: CHAT — Internal Chat [P0] [FIXED]

**Purpose:** Realtime team communication; replaces Slack/Zalo internally. Full Slack-clone scope (DEC-027). Realtime via Socket.IO + Redis presence + Postgres append-only (DEC-032).

**Functional requirements:**

- `FR-CHAT-001` [T] **MUST** support workspaces (1 per tenant), public channels, private channels, DMs, group DMs (≤9 participants).
- `FR-CHAT-002` [T] **MUST** support threads — parent message + replies, with "also send to channel" option.
- `FR-CHAT-003` [T] **MUST** support `@user`, `@channel`, `@here`, `@team` mentions and route web-push notifications per Member preferences.
- `FR-CHAT-004` [T] **MUST** support emoji reactions, message edit (with `edited_at`), soft-delete (never hard-delete; tombstone + redacted body for compliance + retention).
- `FR-CHAT-005` [T] **MUST** support file uploads via S3-compatible per-tenant-residency-tagged bucket; signed URL to client; metadata in `chat_message.files` jsonb. Inline preview for image/PDF/video (≤50MB)/CSV/Excel.
- `FR-CHAT-006` [T] **MUST** support slash commands (built-in `/giphy`, `/shrug`, `/genie`, `/remind`; custom commands registered from MCP tool surface).
- `FR-CHAT-007` [T] **MUST** support full-text search via tsvector + Vietnamese PGroonga; semantic search via pgvector embeddings of message bodies (lazy-indexed, surfaced through BRAIN).
- `FR-CHAT-008` [T] **MUST** track read state per Member per channel via `chat_read_state` table; expose unread count.
- `FR-CHAT-009` [T] [DEC-032] **MUST** deliver realtime via Socket.IO over WebSocket with long-poll fallback; auth via JWT in handshake.
- `FR-CHAT-010` [T] **MUST** implement presence (online/away/offline/in-meeting/on-leave) via Redis with 15s heartbeat TTL; in-meeting + on-leave auto-pulled from HR.
- `FR-CHAT-011` [T] **MUST** persist messages append-only with `audit_hash = sha256(prev_audit_hash || canonical_json(payload))` per channel — Merkle chain.
- `FR-CHAT-012` [T] **MUST** emit `chat.message_sent` event to NATS for downstream consumers (BRAIN ingestion, search index, AI summarize triggers, audit ingestion).
- `FR-CHAT-013` [T] **MUST** support per-workspace retention configuration; default 7 years; legal hold capability for litigation.
- `FR-CHAT-014` [T] **MUST** apply tenant residency to message + file storage per DEC-011.
- `FR-CHAT-015` [T] **MUST** support DSAR erasure: tombstone `deleted_at` + body → "[redacted]"; full purge after retention; cascades to BRAIN chunks.
- `FR-CHAT-016` [T] **MUST** support web push via VAPID; mobile push deferred to P3+ when mobile app exists.
- `FR-CHAT-017` [D] **SHOULD** support thread/channel AI summarize via `chat.summarize_thread` and `chat.summarize_channel` MCP tools.
- `FR-CHAT-018` [T] **MUST** support pinned messages, bookmarks, saved-for-later per Member.
- `FR-CHAT-019` [T] **MUST** support reminders (`/remind me at 9am tomorrow about...`) via BullMQ scheduled jobs.
- `FR-CHAT-020` [T] **MUST** support DND schedule per Member.
- `FR-CHAT-021` [T] **MUST** support channel categories / sections, channel description, topic, pinned canvas (KB-doc embedded).
- `FR-CHAT-022` [T] **MUST** support channel notification preferences: `all` / `mentions` / `nothing`.
- `FR-CHAT-023` [T] **MUST** support export channel to PDF/Markdown.
- `FR-CHAT-024` [T] **SHOULD** provide channel analytics for admins.

**Data model (Prisma sketch):**

```prisma
model Workspace {
  id          String   @id @default(uuid())
  tenantId    String   @unique
  name        String
  retentionDays Int    @default(2555)  // 7 years
}

model ChatChannel {
  id          String   @id @default(uuid())
  tenantId    String
  workspaceId String
  name        String
  kind        ChatChannelKind  // PUBLIC | PRIVATE | DM | GROUP_DM
  topic       String?
  description String?
  category    String?
  archivedAt  DateTime?
  members     ChatChannelMember[]
  messages    ChatMessage[]
  @@index([tenantId, kind])
}

enum ChatChannelKind { PUBLIC PRIVATE DM GROUP_DM }

model ChatChannelMember {
  channelId       String
  memberId        String
  joinedAt        DateTime @default(now())
  role            String   // owner | moderator | member
  notificationPref String  @default("all")  // all | mentions | nothing
  @@id([channelId, memberId])
}

model ChatMessage {
  id              String    @id @default(uuid())
  tenantId        String
  channelId       String
  parentMessageId String?   // thread parent
  authorId        String
  body            String
  bodySearchTsv   Unsupported("tsvector")?
  files           Json?
  reactions       Json?
  pinnedAt        DateTime?
  editedAt        DateTime?
  deletedAt       DateTime?
  auditHash       String
  prevAuditHash   String?
  createdAt       DateTime  @default(now())
  @@index([tenantId, channelId, createdAt(sort: Desc)])
}

model ChatReadState {
  channelId         String
  memberId          String
  lastReadMessageId String?
  updatedAt         DateTime @updatedAt
  @@id([channelId, memberId])
}

model ChatReminder {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  channelId   String?
  body        String
  remindAt    DateTime
  delivered   Boolean  @default(false)
}
```

**MCP tools:** `chat.send_message`, `chat.list_channels`, `chat.list_messages`, `chat.search`, `chat.summarize_thread`, `chat.summarize_channel`, `chat.create_channel`, `chat.invite_to_channel`, `chat.upload_file`, `chat.set_status`, `chat.create_reminder`, `chat.export_channel`

### 4.6 Module: BRAIN — Universal Knowledge Layer [P0] [FIXED]

**Purpose:** Per-tenant Postgres + pgvector HNSW + PGroonga universal knowledge index; auto-ingests every eligible module write event; powers Genie + agent RAG (DEC-033, DEC-034, DEC-036, DEC-037).

**Functional requirements:**

- `FR-BRAIN-001` [T] **MUST** maintain `knowledge_chunk` table with the schema in §4.6.2.
- `FR-BRAIN-002` [T] **MUST** consume NATS events per source module:
  - `cyberos.chat.message_sent`
  - `cyberos.proj.task.{created,updated,completed}`
  - `cyberos.proj.task_comment.created`
  - `cyberos.crm.activity.{created,updated}`
  - `cyberos.crm.deal.stage_changed`
  - `cyberos.kb.document.{published,updated}`
  - `cyberos.email.thread.{received,sent}` (summary only by default)
  - `cyberos.hr.member.{updated_non_comp_field}`
  - `cyberos.learn.training.recorded`
  - `cyberos.learn.sabbatical_granted`
  - `cyberos.learn.promotion_approved` (outcome summary only)
  - `cyberos.obs.alert_anonymized`
- `FR-BRAIN-003` [T] [DEC-036] **MUST NOT** consume:
  - `cyberos.rew.*`
  - `cyberos.esop.*`
  - `cyberos.hr.compensation_changed`
  - any event with `pii_class='special_category'`
  - any compensation/equity payload
- `FR-BRAIN-004` [T] **MUST** apply ingestion policy per source — embed (possibly redacted) text, assign `pii_class`, set `dsar_marker = member_id` reference, set `retention_until` per source-module retention rules.
- `FR-BRAIN-005` [T] **MUST** queue ingestion jobs in BullMQ `brain-ingest` queue; concurrency 8 per pod; retry 3× exponential backoff; DLQ on permanent failure.
- `FR-BRAIN-006` [T] **MUST** call AI Gateway `embed` (cached); persist `body_embedding` (vector(1536)) + `body_search_tsv` + `body_pgroonga` in atomic transaction.
- `FR-BRAIN-007` [T] **MUST** be idempotent on `(tenant_id, source_module, source_entity_id, source_entity_version)`.
- `FR-BRAIN-008` [T] **MUST** re-embed on source row update; old version retained 24h then GC'd.
- `FR-BRAIN-009` [T] **MUST** delete chunks on source deletion event; tombstone left for audit.
- `FR-BRAIN-010` [T] **MUST** dispatch to regional cluster per `tenant.residency`; cross-region read forbidden.
- `FR-BRAIN-011` [T] **MUST** expose `brain.search(query, scope?, k=10)`: hybrid (BM25 + vector + PGroonga via RRF k=60); returns chunks with provenance link `cyberos://{module}/{path}`.
- `FR-BRAIN-012` [T] **MUST** expose `brain.ask(question, scope?)`: RAG endpoint via AI Gateway; force source-citation; "I don't know" if no relevant chunks.
- `FR-BRAIN-013` [T] **MUST** expose `brain.list_chunks_for_member(member_id)` for Member self-serve DSAR (own footprint).
- `FR-BRAIN-014` [T] **MUST** expose `brain.forget(member_id)` admin tool: locate every chunk where `dsar_marker = member_id` OR `metadata.author_id = member_id` OR `metadata.subject_id = member_id`; soft-delete (tombstone) immediately; hard-delete after 30-day legal hold; emit `brain.dsar_purged` audit event.
- `FR-BRAIN-015` [T] **MUST** run nightly retention purge `DELETE FROM knowledge_chunk WHERE retention_until < NOW()` in 1000-row batches.
- `FR-BRAIN-016` [T] **MUST** support `brain.reindex_source(source_module)` for parameter or schema changes.
- `FR-BRAIN-017` [T] **MUST** track ingestion latency per source; emit OBS metric for SLO.
- `FR-BRAIN-018` [T] **MUST** track per-tenant embedding spend; rolls up into AI Gateway budget cap.

**Data model (Prisma sketch):**

```prisma
model KnowledgeChunk {
  id                    String    @id @default(uuid())
  tenantId              String
  sourceModule          String    // 'chat' | 'proj' | 'proj_comment' | 'crm_activity' | 'crm_deal' | 'kb' | 'email_summary' | 'hr_non_comp' | 'learn_training' | 'learn_outcome_summary' | 'obs_alert'
  sourceEntityId        String
  sourceEntityKind      String
  sourceEntityVersion   Int       @default(1)
  body                  String
  bodySearchTsv         Unsupported("tsvector")?  // GENERATED
  bodyPgroonga          String?   // GENERATED
  bodyEmbedding         Unsupported("vector(1536)")
  piiClass              String    // 'public' | 'internal' | 'restricted'
  dsarMarker            String?   // member_id whose DSAR triggers deletion
  metadata              Json?     // {channel_id, author_id, subject_id, project_id, ...}
  retentionUntil        DateTime
  ingestedAt            DateTime  @default(now())
  auditHash             String
  @@unique([tenantId, sourceModule, sourceEntityId, sourceEntityVersion])
  @@index([tenantId, sourceModule, sourceEntityId])
  @@index([tenantId, dsarMarker])
  @@index([retentionUntil])
}

// HNSW index on body_embedding; GIN on body_search_tsv; pgroonga on body_pgroonga
```

**Ingestion handler pattern (TypeScript sketch):**

```typescript
// modules/brain/api/src/ingestion/chat-handler.ts
export async function handleChatMessageSent(event: ChatMessageSentEvent) {
  if (event.payload.is_dm) {
    await ingest({
      tenantId: event.tenant_id,
      sourceModule: 'chat',
      sourceEntityId: event.payload.message_id,
      sourceEntityKind: 'message_dm',
      body: event.payload.body,
      piiClass: 'restricted',  // DM in private namespace
      dsarMarker: event.payload.author_id,
      metadata: { channel_id: event.payload.channel_id, author_id: event.payload.author_id },
      retentionUntil: addYears(now(), 7),
    });
  } else {
    await ingest({ ...same, piiClass: 'internal', sourceEntityKind: 'message_public' });
  }
}
```

**MCP tools:** `brain.search`, `brain.ask`, `brain.list_sources`, `brain.list_chunks_for_member`, `brain.forget`, `brain.reindex_source`

### 4.7 Module: GENIE — Company Mascot AI Assistant [P0] [FIXED]

**Purpose:** Persistent omnipresent AI assistant with versioned persona; calls MCP tools on Member's behalf with confirm-step (DEC-033, DEC-035).

**Functional requirements:**

- `FR-GENIE-001` [T] **MUST** be invocable via floating button on every MFE remote; ⌘+G global keyboard shortcut; `/genie` slash command in CHAT.
- `FR-GENIE-002` [T] **MUST** maintain persistent per-Member conversation thread (last 90 days kept; older archived) with referenceable history.
- `FR-GENIE-003` [T] **MUST** load active persona version at conversation start; persona injected as system prompt on every LLM call.
- `FR-GENIE-004` [T] **MUST** maintain `genie_persona_version` table (immutable; UPDATE/DELETE blocked at DB-policy level) per DEC-031 pattern.
- `FR-GENIE-005` [T] **MUST** require Founder/CEO + Engineering Lead dual-sign workflow to publish new persona version.
- `FR-GENIE-006` [T] **MUST** call `brain.ask` for factual queries; force source citation; render citations as clickable `cyberos://{module}/{path}` chips.
- `FR-GENIE-007` [T] **MUST** invoke MCP tools for action requests using Member's JWT; **MUST** require explicit user confirmation (Yes/Edit/No chip) before executing any write tool.
- `FR-GENIE-008` [T] **MUST** display AI Act high-risk label ("AI-assisted; final decision by qualified human reviewer") on responses surfacing or reasoning about REW / LEARN promotion / comp data.
- `FR-GENIE-009` [T] **MUST** refuse to read REW / ESOP / HR-compensation data; respond with redirect ("I cannot access compensation or equity figures — please ask the HR/Ops Lead directly").
- `FR-GENIE-010` [T] **MUST** audit-log every conversation turn + tool invocation: actor, persona_version_id, prompt, response, citations[], tool_invoked?, before, after, occurred_at.
- `FR-GENIE-011` [T] **MUST** respect per-tenant model preference + residency (DEC-018, DEC-011).
- `FR-GENIE-012` [T] **MUST** support bilingual VN/EN; auto-detect Member language preference; explicit toggle; Vietnamese-prevailing on legal/comp surfaces (Total Rewards Appendix Article 7c).
- `FR-GENIE-013` [T] **MUST** render visual mascot states: idle, listening, thinking, speaking, error, succeeded — based on the CyberSkill logo (PRD §11).
- `FR-GENIE-014` [T] **MUST** expose persona test suite: 30+ canonical Member queries with expected behavior assertions; runs in CI on every Genie code change.
- `FR-GENIE-015` [T] **SHOULD** support voice input (P1 stretch via Whisper API or on-device — OQ-020).
- `FR-GENIE-016` [T] **MUST** support proactive nudges (digests, reminders, alerts) per Member preference; default cadence configurable.
- `FR-GENIE-017` [T] **MUST** include the Member-visible audit trail in their own conversation history.
- `FR-GENIE-018` [T] **MUST** export persona version JSON as part of EU AI Act Annex IV documentation.

**Data model (Prisma sketch):**

```prisma
model GeniePersonaVersion {
  id              String   @id @default(uuid())
  tenantId        String
  versionLabel    String
  voiceRules      Json     // tone, language, conciseness, etc.
  behaviorRules   Json     // citation, confirmation, deference rules
  scopeContract   Json     // does + does-not lists
  languages       Json     // ['vi-VN', 'en-US']
  visualMascotConfig Json  // states, sizes, colors
  effectiveDate   DateTime
  signedBy        Json
  signedAt        DateTime
  auditHash       String
  @@unique([tenantId, versionLabel])
}

model GenieConversation {
  id            String   @id @default(uuid())
  tenantId      String
  memberId      String
  startedAt     DateTime @default(now())
  lastActiveAt  DateTime @updatedAt
  personaVersionId String
}

model GenieMessage {
  id              String   @id @default(uuid())
  tenantId        String
  conversationId  String
  kind            String   // user | genie | system | tool_call | tool_result
  body            String?
  toolCallPayload Json?
  citations       Json?
  occurredAt      DateTime @default(now())
}

model GenieAction {
  id              String   @id @default(uuid())
  tenantId        String
  conversationId  String
  mcpTool         String
  params          Json
  result          Json?
  status          String   // pending_confirm | confirmed | executed | rejected | failed
  confirmedBy     String?
  confirmedAt     DateTime?
  auditHash       String
}
```

**MCP tools:** `genie.start_conversation`, `genie.send_message`, `genie.list_conversations`, `genie.get_conversation`, `genie.publish_persona_version`, `genie.list_persona_versions`, `genie.get_audit_log`

### 4.8 Module: PROJ — Projects & Tasks [P1] [FIXED]

**Purpose:** Project + task management replacing Notion/Asana for client and internal work.

**Functional requirements:**

- `FR-PROJ-001` [T] **MUST** support Projects with `code` (3–8 chars, [A-Z0-9]), name, description, client (CRM Company ref), startDate, endDate, status `ACTIVE/PAUSED/COMPLETED/CANCELLED`, projectType (`FIXED_PRICE/T_AND_M/INTERNAL`), defaultRate, currency.
- `FR-PROJ-002` [T] **MUST** support Tasks with status `BACKLOG/TODO/IN_PROGRESS/REVIEW/DONE/CANCELED`, priority `P0..P3`, assignee, watchers, due, estimate (hours), actual (rolled up from TIME entries), labels.
- `FR-PROJ-003` [T] [DEC-019] **MUST** support **unlimited subtask depth**.
- `FR-PROJ-004` [T] [DEC-020] **MUST** support Boards (Kanban) and Sprints (time-boxed).
- `FR-PROJ-005` [T] **MUST** support Labels (tenant-scoped, many-to-many) and Comments with `@mentions` cross-linked into CHAT thread.
- `FR-PROJ-006` [T] **MUST** support file attachments via pre-signed URLs (per-tenant residency).
- `FR-PROJ-007` [T] **MUST** emit `proj.task.created/updated/completed`, `proj.task_comment.created`, `proj.uat_signed_off`, `proj.warranty_period_ended`, `proj.client_bad_debt` events to NATS.
- `FR-PROJ-008` [T] **MUST** notify CHAT DM on assignment / mention / status change (configurable per Member).
- `FR-PROJ-009` [T] **MUST** soft-delete tasks with 30-day undo.
- `FR-PROJ-010` [T] **MUST** support task dependencies: `blocks`, `blocked_by`, `relates_to`.
- `FR-PROJ-011` [T] **MUST** support recurring tasks (daily / weekly / monthly cron).
- `FR-PROJ-012` [T] **MUST** support project templates and task templates.
- `FR-PROJ-013` [T] **MUST** support custom fields per project (text, number, select, date, member).
- `FR-PROJ-014` [T] **SHOULD** provide Gantt view (P1 stretch / P2 default).
- `FR-PROJ-015` [T] **MUST** integrate with TIME — one-click timer start per task.
- `FR-PROJ-016` [T] **MUST** support bulk operations (multi-select + bulk-edit).
- `FR-PROJ-017` [T] **MUST** support saved views and filters.
- `FR-PROJ-018` [T] **MUST** support project archive (read-only after closure).
- `FR-PROJ-019` [T] **MUST** support UAT sign-off workflow with Account Manager + External Client (P4) co-sign; emits `proj.uat_signed_off` event consumed by REW.
- `FR-PROJ-020` [T] **MUST** support warranty-period tracking with auto-close at end; emits `proj.warranty_period_ended` event.
- `FR-PROJ-021` [T] **MUST** support bad-debt flag (Account Manager + Founder/CEO approval); emits `proj.client_bad_debt` event consumed by REW for Management Risk Shield.

**MCP tools:** `projects.create_project`, `projects.update_project`, `projects.create_task`, `projects.update_task`, `projects.update_task_status`, `projects.list_tasks`, `projects.search`, `projects.create_sprint`, `projects.move_to_sprint`, `projects.add_dependency`, `projects.uat_sign_off`, `projects.flag_bad_debt`, `projects.archive_project`, `projects.create_template`, `projects.list_templates`

### 4.9 Module: TIME — Time Tracking [P1] [FIXED]

**Purpose:** Time entries + weekly timesheets feeding INV (P2), REW (workload), LEARN (productivity points), RES (P3 actuals).

**Functional requirements:**

- `FR-TIME-001` [T] **MUST** support timer-based entry (one-click start; auto-pause on idle 15min).
- `FR-TIME-002` [T] **MUST** support manual entry (single, bulk multi-day).
- `FR-TIME-003` [T] **MUST** record per entry: Member, task (PROJ ref), project, start, end, duration_minutes, billable, description, status `DRAFT/SUBMITTED/APPROVED/INVOICED/REJECTED`, currency.
- `FR-TIME-004` [T] **MUST** support weekly submission + manager approval workflow with comment.
- `FR-TIME-005` [T] **MUST** become immutable once linked to an invoice (status `INVOICED`).
- `FR-TIME-006` [T] **MUST** export CSV for finance; XLSX with per-project totals.
- `FR-TIME-007` [T] **MUST** detect overlapping entries and warn on submit.
- `FR-TIME-008` [T] **MUST** emit `time.week_approved` event with `{member_id, week_start, total_minutes, billable_minutes, project_breakdown[]}` consumed by REW + LEARN.
- `FR-TIME-009` [T] **MUST** provide calendar view (week / month) of own + (manager only) team time.
- `FR-TIME-010` [T] **MUST** support per-project rate override (Account Manager + manager set).
- `FR-TIME-011` [T] **MUST** integrate with Genie nudges on missing time logs; suggested entries from CHAT/EMAIL/calendar.
- `FR-TIME-012` [T] **MUST** be mobile-responsive for in-the-moment timer use.

**MCP tools:** `time.start_timer`, `time.stop_timer`, `time.log_manual`, `time.list_entries`, `time.submit_week`, `time.approve_week`, `time.reject_week`, `time.export_csv`, `time.list_pending_approvals`

### 4.10 Module: CRM [P1] [FIXED]

**Purpose:** Companies, Contacts, Leads, Deals, Activities, Pipelines, Forecasting.

**Functional requirements:**

- `FR-CRM-001` [T] **MUST** support Companies (industry, size, country, website, address, lifecycle stage, owner, custom fields).
- `FR-CRM-002` [T] **MUST** support Contacts (n:n with Companies; name, title, email, phone, LinkedIn, owner, notes, opt-out flags GDPR).
- `FR-CRM-003` [T] **MUST** support Leads (status `NEW/QUALIFIED/CONTACTED/UNQUALIFIED`, source, owner, conversion to Deal).
- `FR-CRM-004` [T] **MUST** support Deals (configurable pipeline; multi-pipeline per tenant; stages with probabilities; value multi-currency; expected close date; line items products/services).
- `FR-CRM-005` [T] **MUST** support Pipelines: configurable stages e.g. Discovery → Proposal → Negotiation → Closed-Won / Closed-Lost; multiple per tenant.
- `FR-CRM-006` [T] **MUST** support Activities (call/email/meeting/note/task) attached to Company/Contact/Lead/Deal.
- `FR-CRM-007` [T] **MUST** support inbound BCC: emailing `crm@{tenant}.cyberos.vn` auto-creates Activity, parses From/To, suggests Contact match.
- `FR-CRM-008` [T] **MUST** integrate with EMAIL — thread participants suggested as Contacts; one-click "Log thread to Deal" creates Activity.
- `FR-CRM-009` [T] **MUST** support hybrid search (BM25 via tsvector + pgvector via cosine, merged via RRF k=60).
- `FR-CRM-010` [T] **MUST** support forecasting: weighted pipeline by stage probability; commit / best-case / worst-case; per-period.
- `FR-CRM-011` [T] **MUST** support reports: pipeline by stage, deals by owner, activities by week, conversion rates.
- `FR-CRM-012` [T] **MUST** support soft-delete with 30-day undo.
- `FR-CRM-013` [T] **MUST** support GDPR contact erasure cascading to BRAIN.
- `FR-CRM-014` [T] **MUST** support tag/label system; custom fields per Deal/Company/Contact.
- `FR-CRM-015` [T] **MUST** support saved searches and views.
- `FR-CRM-016` [T] **MUST** support bulk import CSV / export CSV.
- `FR-CRM-017` [T] **MUST** emit `crm.activity.created`, `crm.deal.stage_changed` events consumed by BRAIN.

**MCP tools:** `crm.find_contact`, `crm.create_lead`, `crm.log_activity`, `crm.update_deal_stage`, `crm.list_pipeline`, `crm.search`, `crm.forecast`, `crm.find_company`, `crm.import_csv`, `crm.export_csv`, `crm.create_pipeline`, `crm.create_company`, `crm.create_contact`

### 4.11 Module: KB — Knowledge Base [P1] [FIXED]

**Purpose:** Markdown wiki with semantic search, RAG, version history. Houses Trust Center artifacts, runbooks, DEC summaries (authoritative in SRS §3.3; readable copies in KB), onboarding docs.

**Functional requirements:**

- `FR-KB-001` [T] **MUST** support Spaces (1 level) and Documents (Markdown + extensions: callouts, tabs, mermaid, code with syntax highlighting, KaTeX).
- `FR-KB-002` [T] **MUST** auto-chunk on save (500 tokens, 50 overlap); embed via AI Gateway; index in BRAIN with provenance.
- `FR-KB-003` [T] [DEC-022] **MUST** index for full-text via tsvector + PGroonga (Vietnamese) and vector via pgvector HNSW (cosine).
- `FR-KB-004` [T] **MUST** expose `kb.ask({question})` returning answer + cited chunk IDs (subset of `brain.ask` scoped to KB).
- `FR-KB-005` [T] **MUST** support hybrid search merged via RRF k=60.
- `FR-KB-006` [T] **MUST** generate AI summary on publish (cached) and version diff view.
- `FR-KB-007` [T] **MUST** apply DSAR erasure to embeddings + chunks per DEC-012.
- `FR-KB-008` [T] **MUST** support templates (runbook, post-mortem, onboarding checklist, KB-doc default).
- `FR-KB-009` [T] **MUST** support permissions per Space: read / write / admin.
- `FR-KB-010` [T] **MUST** support comments per doc with `@mentions`; reactions and bookmarks.
- `FR-KB-011` [T] **MUST** support embed in CHAT: `kb://{doc_id}` renders inline card.
- `FR-KB-012` [T] **MUST** support export Space to ZIP / Markdown bundle.
- `FR-KB-013` [T] **MUST** support Trust Center mode: Space marked "public-readable" exposes contents to `trust.cyberskill.world`.
- `FR-KB-014` [T] **SHOULD** support Mermaid + KaTeX rendering.

**MCP tools:** `kb.search`, `kb.read_document`, `kb.create_document`, `kb.update_document`, `kb.publish_document`, `kb.ask`, `kb.summarize_document`, `kb.create_space`, `kb.list_spaces`, `kb.create_template`, `kb.export_space`

### 4.12 Module: HR — Human Resources (full) [P1] [FIXED]

**Purpose:** Member identity-as-employee. Full HR system at v1.0 — not Lite. Source for REW base salary, LEARN career level, ESOP grant eligibility.

**Functional requirements:**

- `FR-HR-001` [T] **MUST** support Member profile: contact info, role, department, manager, hireDate, **continuousServiceStart**, employmentType (`FULL_TIME/PART_TIME/CONTRACTOR/INTERN`), location, country, **encrypted government IDs**, **encrypted bank account**, **encrypted home address**, emergency contact, profile photo, public bio.
- `FR-HR-002` [T] [DEC-023] **MUST** support encrypted compensation fields (P1 base salary, P2 allowance config) using app-layer envelope encryption with per-tenant KMS data keys; access restricted to roles `{owner, hr_lead}` and direct manager (audit-logged).
- `FR-HR-003` [T] **MUST** support continuousServiceStart (≠ hireDate when re-hired); LEARN sabbatical accrual depends on it.
- `FR-HR-004` [T] **MUST** support leave types `ANNUAL/SICK/UNPAID/PARENTAL/SABBATICAL/COMPASSIONATE/OTHER` with leave balance tracking and configurable accrual policy.
- `FR-HR-005` [T] **MUST** support leave request workflow: request → manager approve → balance deducted → CHAT DM notification → calendar update → REW informed (P3=0 for SABBATICAL months).
- `FR-HR-006` [T] **MUST** support sabbatical workflow: ≥3-month advance booking; non-encashable; HR/Ops Lead reviews resource implications; Founder/CEO approves; emits `learn.sabbatical_granted` event.
- `FR-HR-007` [T] **MUST** expose org chart query (manager hierarchy traversal) with photos and roles.
- `FR-HR-008` [T] **MUST** support onboarding workflows: per-role onboarding checklist (account setup, equipment, document signing, 30/60/90 day check-ins).
- `FR-HR-009` [T] **MUST** support per-Member document repository (contracts, NDAs, Total Rewards Appendix copy, ID copies, certifications, performance reviews); encrypted at rest; access logged.
- `FR-HR-010` [T] **MUST** support performance review cycles (annual; integrates with LEARN): self-assessment + manager assessment + peer feedback; outcome tags; "Meets Expectations" required for ESOP vesting.
- `FR-HR-011` [T] **MUST** support expense management:
  - Member submits expense (receipt photo, amount, currency, category, project ref optional)
  - Approval workflow: manager → Founder/CEO if >threshold
  - Reimbursement: included in next REW payslip P2 line
  - VAT/tax categorization for accounting
  - OCR receipt extraction via AI Gateway (P2 stretch)
- `FR-HR-012` [T] **MUST** support headcount planning: open roles, requisitions, candidate pipeline (lightweight; not a full ATS), forecast vs actual headcount.
- `FR-HR-013` [T] **MUST** log every PII access (compensation, government ID, address, bank account, leave reason) with actor + timestamp; HR/Ops Lead can review the access log.
- `FR-HR-014` [T] **MUST** expose `Member` as Federation entity with `@key(fields: "id")` for REW / LEARN / ESOP / RES / TIME / PROJ to reference.
- `FR-HR-015` [T] **MUST** support termination workflow: HR/Ops Lead initiates; classifies Good Leaver vs Bad Leaver; emits `member.terminated` event with full payload consumed by REW (BP settlement), ESOP (grant settlement), LEARN (close sabbatical), AUTH (revoke sessions, deactivate).
- `FR-HR-016` [T] **MUST** support DSAR: full Member data export + erasure cascading to all modules; PDPL Decree 356 + GDPR conformity.
- `FR-HR-017` [T] **MUST** support self-service: change own contact info, profile photo, emergency contact; submit expense; request leave; view payslip + BP balance + SP vesting (delegated read).

**Data model (Prisma sketch):**

```prisma
model Member {
  id                       String   @id @default(uuid())
  tenantId                 String
  authMemberId             String   @unique  // FK to AUTH.Member
  fullName                 String
  fullNameVi               String?
  email                    String
  phone                    String?
  govIdEncrypted           Bytes?
  govIdKeyVersion          Int?
  homeAddressEncrypted     Bytes?
  bankAccountEncrypted     Bytes?
  emergencyContact         Json?
  hireDate                 DateTime
  continuousServiceStart   DateTime
  employmentType           String
  role                     String
  department               String?
  managerId                String?
  location                 String?
  country                  String?
  status                   String   // active | on_leave | terminated
  baseSalaryVndEncrypted   Bytes?
  baseSalaryKeyVersion     Int?
  p2AllowancesEncrypted    Bytes?
  publicBio                String?
  profilePhotoUrl          String?
  @@index([tenantId, managerId])
}

model LeaveBalance {
  memberId      String
  leaveType     String
  fiscalYear    Int
  accruedDays   Decimal
  takenDays     Decimal
  remainingDays Decimal
  @@id([memberId, leaveType, fiscalYear])
}

model LeaveRequest {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  leaveType   String
  startDate   DateTime
  endDate     DateTime
  reasonText  String?  // ENCRYPTED if leave_type=SICK or COMPASSIONATE
  reasonKeyVersion Int?
  status      String   // pending | approved | rejected | cancelled
  approvedBy  String?
  approvedAt  DateTime?
  @@index([tenantId, memberId, startDate])
}

model OnboardingChecklist {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  templateId  String
  items       Json     // [{ task, dueDate, status, completedAt }]
  startedAt   DateTime
  completedAt DateTime?
}

model MemberDocument {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  kind        String   // contract | nda | id_copy | certification | review
  fileUrlEncrypted String
  uploadedBy  String
  uploadedAt  DateTime
}

model Expense {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  amount      Decimal
  currency    String
  categoryId  String
  description String
  receiptUrl  String?
  projectId   String?
  status      String   // draft | submitted | approved | rejected | paid
  approvedBy  String?
  approvedAt  DateTime?
  paidInPayslipId String?
  submittedAt DateTime
}

model PerformanceReview {
  id            String   @id @default(uuid())
  tenantId      String
  memberId      String
  fiscalYear    Int
  selfAssessment Json?
  managerAssessment Json?
  peerFeedback  Json?
  outcomeTag    String?  // exceeds | meets | below
  finalizedAt   DateTime?
  finalizedBy   String?
}

model PiiAccessLog {
  id        String   @id @default(uuid())
  tenantId  String
  actorId   String
  subjectMemberId String
  fieldKind String   // compensation | gov_id | address | bank | leave_reason
  reason    String?
  occurredAt DateTime @default(now())
}
```

**Federation:**
```graphql
type Member @key(fields: "id") {
  id: ID!
  email: String!
  fullName: String!
  hireDate: Date!
  continuousServiceStart: Date!
  employmentType: EmploymentType!
  role: String!
  managerId: ID
  status: MemberStatus!
  baseSalaryVnd: BigInt @requiresScopes(scopes: [["compensation:read"]])
  publicBio: String
}
```

**MCP tools:** `hr.find_member`, `hr.create_member`, `hr.terminate_member`, `hr.update_profile`, `hr.request_leave`, `hr.approve_leave`, `hr.get_team_org_chart`, `hr.get_sabbatical_eligibility`, `hr.submit_expense`, `hr.approve_expense`, `hr.create_onboarding_checklist`, `hr.list_documents`, `hr.upload_document`, `hr.start_performance_review`, `hr.submit_performance_self_assessment`, `hr.submit_manager_assessment`, `hr.export_member_data` (DSAR), `hr.list_pii_access_log`

### 4.13 Module: EMAIL — Email (full IMAP/SMTP) [P1] [FIXED]

**Purpose:** Per-Member personal mailbox + team shared inboxes. Full IMAP/SMTP scope (DEC-027).

**Functional requirements:**

- `FR-EMAIL-001` [T] **MUST** support per-Member personal mailbox connection via Google Workspace OAuth 2.1, Microsoft 365 OAuth 2.1, generic IMAP/SMTP with app passwords.
- `FR-EMAIL-002` [T] **MUST** store mailbox credentials (OAuth refresh tokens, app passwords) using envelope encryption (DEC-023 pattern).
- `FR-EMAIL-003` [T] **MUST** maintain IMAP IDLE connection per active mailbox; supervised via BullMQ; restart on disconnect within 30s.
- `FR-EMAIL-004` [T] **MUST** sync the latest 90 days of message bodies into encrypted Postgres + S3 cache (per-tenant residency); older messages fetched on-demand.
- `FR-EMAIL-005` [T] **MUST** present threaded conversation UI with Gmail-style grouping by Message-ID / References / In-Reply-To.
- `FR-EMAIL-006` [T] **MUST** support labels/folders mapped to provider semantics (Gmail labels → Postgres labels; M365 folders → Postgres folders).
- `FR-EMAIL-007` [T] **MUST** support compose: rich text, attachments, signatures (per-mailbox), schedule send, reply-all, forward, draft autosave.
- `FR-EMAIL-008` [T] **MUST** support search: server-side IMAP SEARCH where supported, fallback to local index.
- `FR-EMAIL-009` [T] **MUST** parse calendar invites (RFC 5545 ICS) and present preview + RSVP UI; RSVP sends iMIP REPLY back via SMTP.
- `FR-EMAIL-010` [T] **MUST** support shared inbox: team mailboxes (`hr@`, `info@`, `support@`) with assignment, internal notes, snooze, status `OPEN/PENDING/CLOSED`.
- `FR-EMAIL-011` [T] **MUST** integrate with CRM: thread participants suggested as Contacts; one-click "Log thread to Deal" creates `crm.activity`.
- `FR-EMAIL-012` [T] **MUST** support AI features (via AI Gateway): draft reply, summarize thread, extract action items into PROJ tasks.
- `FR-EMAIL-013` [T] **MUST** apply tenant residency to credential storage + body cache.
- `FR-EMAIL-014` [T] **MUST** support DSAR erasure: delete cached bodies, clear search index, revoke OAuth refresh tokens, document IMAP-side limitation.
- `FR-EMAIL-015` [T] **MUST** route outbound transactional email (notifications, magic links, MFA codes, invoice send) through Postmark or SES with full SPF/DKIM/DMARC on `cyberskill.world`.
- `FR-EMAIL-016` [T] **MUST** rate-limit outbound to comply with provider quotas; backoff on 4xx; quarantine on hard bounce.
- `FR-EMAIL-017` [T] **MUST** virus-scan incoming attachments; safe-preview for PDF/Office; quarantine suspicious.
- `FR-EMAIL-018` [T] **MUST** support email-to-task: forward to `task+{ProjCode}@{tenant}.cyberos.vn` to create a task in PROJ.
- `FR-EMAIL-019` [T] **MUST** emit `email.thread.received`, `email.thread.sent` (summary only) consumed by BRAIN.

**Data model (Prisma sketch):**

```prisma
model Mailbox {
  id              String   @id @default(uuid())
  tenantId        String
  ownerMemberId   String?  // null for shared
  kind            String   // PERSONAL | SHARED
  emailAddress    String
  provider        String   // 'google' | 'microsoft' | 'generic_imap'
  credentialEncrypted Bytes
  credentialKeyVersion Int
  lastSyncedAt    DateTime?
  syncStatus      String   // 'idle' | 'syncing' | 'error'
  ingestBodyToBrain Boolean @default(false)  // per-Member opt-in
  @@unique([tenantId, emailAddress])
}

model EmailThread {
  id            String   @id @default(uuid())
  tenantId      String
  mailboxId     String
  subject       String
  participants  Json
  lastMessageAt DateTime
  status        String?  // for SHARED: 'open' | 'pending' | 'closed'
  assigneeId    String?
  snoozedUntil  DateTime?
  @@index([tenantId, mailboxId, lastMessageAt(sort: Desc)])
}

model EmailMessage {
  id           String   @id @default(uuid())
  tenantId     String
  threadId     String
  messageId    String   @unique
  fromAddr     String
  toAddrs      Json
  ccAddrs      Json?
  bccAddrs     Json?
  subject      String
  body         String?  // null if older than 90 days
  bodyHtml     String?
  attachments  Json?
  bodySearchTsv Unsupported("tsvector")?
  receivedAt   DateTime
  @@index([tenantId, threadId, receivedAt(sort: Desc)])
}
```

**MCP tools:** `email.list_threads`, `email.read_thread`, `email.draft_reply`, `email.summarize_thread`, `email.extract_action_items`, `email.send`, `email.assign_shared_thread`, `email.set_shared_thread_status`, `email.snooze_thread`, `email.search`, `email.list_mailboxes`, `email.connect_mailbox`

### 4.14 Module: REW — Total Rewards [P1 core, P2 full pool] [FIXED]

**Purpose:** Encode Articles 1, 2, 3, 4 of the Total Rewards Appendix. Deterministic engine (DEC-030); parameter versioning (DEC-031); BP ledger immutable + Merkle-chained.

**Functional requirements:**

- `FR-REW-001` [T] **MUST** maintain `rew_parameter_version` table (immutable; UPDATE/DELETE blocked at DB-policy level).
- `FR-REW-002` [T] **MUST** compute monthly payslip via deterministic engine (signature in §4.14.2).
- `FR-REW-003` [T] **MUST** enforce P1 protection invariant: `gross_p1` always equals HR base × (working_days_actual / working_days_standard); never reduced as penalty (Article 2a). CI test: simulate worst-possible VP score → P1 unchanged.
- `FR-REW-004` [T] **MUST** enforce 300% P3 cap: if `proposed_p3_cash > 3.0 × gross_p1`, excess credits to BP ledger as `kind=p3_overflow_in` (Article 2b).
- `FR-REW-005` [T] **MUST** maintain BP ledger as append-only Merkle-chained event log.
- `FR-REW-006` [T] **MUST** credit BP balance with monthly interest: `interest = balance × (acb_12m_term_rate + board_margin_pct) / 12` as `kind=interest_credit` event.
- `FR-REW-007` [T] **MUST** support BP withdrawal request (`kind=withdrawal_out`); cap 100% of P1/month per Member; company-wide cap 20% CFO/month with prorated allocation if over-demand.
- `FR-REW-008` [T] **MUST** maintain Deferred Bonus Fund (`deferred_in` when client late-pays; `deferred_out` auto-released on `invoice.collected` event from INV).
- `FR-REW-009` [P2; T] **MUST** compute Project Bonus Pool: `pool = 0.05 × (project_revenue − direct_engineering_salary − cloud_cost)`. Allocation real-time by VP. Disbursed 70% on `proj.uat_signed_off` + 30% Holdback at end of `warranty_period`.
- `FR-REW-010` [P2; T] **MUST** implement Management Risk Shield: on `proj.client_bad_debt` event, pay 50% of Holdback from internal Risk Reserve Fund; emit `rew.risk_shield_paid` audit event.
- `FR-REW-011` [P2; T] **MUST** support MVP Award: at year-end with Founder approval, allocate 3–5% of net profit excess (no cap); emit `mvp_award_granted` event consumed by ESOP for next-year 1.5x SP grant multiplier.
- `FR-REW-012` [T] **MUST** apply PIT progressive Vietnamese rates per parameter version; payslip displays both gross and net VND.
- `FR-REW-013` [T] **MUST** generate bilingual VN/EN payslip PDF; Vietnamese display first/toggleable; Vietnamese is legal-prevailing (Article 7c).
- `FR-REW-014` [T] **MUST** enforce termination settlement (consumes `member.terminated` event from HR):
  - Good Leaver → BP balance fully paid in final payslip (`kind=good_leaver_payout`)
  - Bad Leaver → BP balance forfeited (`kind=bad_leaver_forfeit`, balance set to 0 in event chain; never deletes prior events)
- `FR-REW-015` [T] **MUST** support `rew.compute_payslip_preview` (idempotent; no side effects) and `rew.issue_payslip` (state-changing; emits `payslip.issued` event).
- `FR-REW-016` [T] **MUST** support `rew.recompute_payslip(payslip_id)` returning identical output as the original (anti-retroactive recompute test); CI verifies on stored payslips.
- `FR-REW-017` [T] **MUST** restrict `rew.publish_parameter_version` to roles `{owner, hr_lead}` with founder + HR/Ops Lead dual-sign workflow (or founder + engineering lead for VP-related Quality Multiplier rules).
- `FR-REW-018` [T] **MUST** redact P3 / P1 / BP / payslip values from AI Gateway prompts unless on the explicit `rew.payslip_explain` route which is PII-safe.
- `FR-REW-019` [T] [DEC-036] **MUST NOT** be ingested to BRAIN.

**Engine signature (TypeScript sketch):**

```typescript
function computePayslip(inputs: {
  baseSalary: bigint,             // HR
  allowances: bigint,             // HR
  workloadMinutes: number,        // TIME (week_approved aggregated)
  teamQualityMultiplier: number,  // LEARN
  individualQualityMultiplier: number,  // LEARN
  projectCashCollected: bigint,   // INV (P2+; 0 in P1)
  deferredReleases: bigint[],     // REW DBF
  bpLedgerStateAtMonthStart: { balance: bigint },
}, parameterVersion: RewParameterVersion): Payslip {
  const grossP1 = baseSalary * workingDaysFactor;
  const grossP2 = allowances * workingDaysFactor;
  const vp = (workloadMinutes / 60) * individualQualityMultiplier * teamQualityMultiplier;
  const p3PoolShare = computeP3PoolShare(vp, projectCashCollected, parameterVersion);
  const p3Cap = grossP1 * BigInt(parameterVersion.p3_cap_percent_of_p1);
  const p3Cash = min(p3PoolShare, p3Cap);
  const p3BpOverflow = max(p3PoolShare - p3Cap, 0n);
  const grossTotal = grossP1 + grossP2 + p3Cash + sum(deferredReleases);
  const pitWithheld = computePit(grossTotal, parameterVersion.pit_brackets);
  const netPayable = grossTotal - pitWithheld;
  return { grossP1, grossP2, grossP3Cash: p3Cash, grossP3BpOverflow: p3BpOverflow,
           pitWithheld, netPayable, ... };
}
```

**Data model (Prisma sketch):**

```prisma
model RewParameterVersion {
  id            String   @id @default(uuid())
  tenantId      String
  fiscalYear    Int
  versionLabel  String
  payload       Json     // p3_cap_percent_of_p1, bp_disbursement_cap_pct_cfo, bp_interest_benchmark, bp_interest_margin_pct, pit_brackets, mvp_award_pool_pct_min/max, industry_multiplier_for_year
  effectiveDate DateTime
  signedBy      Json
  signedAt      DateTime
  auditHash     String
  @@unique([tenantId, fiscalYear, versionLabel])
}

model Payslip {
  id                  String   @id @default(uuid())
  tenantId            String
  memberId            String
  periodMonth         String   // 'YYYY-MM'
  parameterVersionId  String
  grossP1Vnd          BigInt
  grossP2Vnd          BigInt
  grossP3CashVnd      BigInt
  grossP3BpOverflowVnd BigInt
  pitWithheldVnd      BigInt
  netPayableVnd       BigInt
  deferredInVnd       BigInt
  deferredOutVnd      BigInt
  bpInVnd             BigInt
  bpOutVnd            BigInt
  status              String   // DRAFT | APPROVED | PAID
  computedAt          DateTime
  computedBy          String
  auditHash           String
  @@unique([tenantId, memberId, periodMonth])
}

model BpLedger {
  id                 String   @id @default(uuid())
  tenantId           String
  memberId           String
  kind               String
  amountVnd          BigInt
  balanceAfterVnd    BigInt
  parameterVersionId String
  occurredAt         DateTime
  prevAuditHash      String?
  auditHash          String
  @@index([tenantId, memberId, occurredAt])
}

model ProjectBonusPool {
  id              String   @id @default(uuid())
  tenantId        String
  projectId       String
  poolVnd         BigInt
  poolFormulaJson Json
  allocations     Json
  uatSignedOffAt  DateTime?
  warrantyEndsAt  DateTime?
  status          String
}

model DeferredBonusFund {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  projectId   String
  amountVnd   BigInt
  reason      String
  status      String   // 'held' | 'released' | 'cancelled'
  releasedAt  DateTime?
  invoiceId   String?
}

model MvpAward {
  id            String   @id @default(uuid())
  tenantId      String
  fiscalYear    Int
  recipientKind String
  recipientIds  Json
  poolVnd       BigInt
  spMultiplier  Float    @default(1.5)
  approvedAt    DateTime
  approvedBy    String
}

model RiskReserveFund {
  id        String   @id @default(uuid())
  tenantId  String
  fiscalYear Int
  balanceVnd BigInt
  movements Json
}
```

**MCP tools:** `rew.compute_payslip_preview`, `rew.issue_payslip`, `rew.list_payslips`, `rew.get_bp_balance`, `rew.list_bp_ledger`, `rew.request_bp_withdrawal`, `rew.approve_bp_withdrawal`, `rew.publish_parameter_version`, `rew.list_deferred_bonuses`, `rew.payslip_explain`, `rew.recompute_payslip`, `rew.calculate_project_bonus_pool` (P2), `rew.disburse_holdback` (P2), `rew.trigger_management_risk_shield` (P2), `rew.grant_mvp_award` (P2)

### 4.15 Module: LEARN — Career Path & Learning [P1] [FIXED]

**Purpose:** Encode Article 6 of the Appendix.

**Functional requirements:**

- `FR-LEARN-001` [T] **MUST** maintain `learn_parameter_version` table (immutable per DEC-031) with VP Quality Multiplier rules per fiscal year.
- `FR-LEARN-002` [T] **MUST** compute VP entries: `vp = workload × individual_quality_multiplier × team_quality_multiplier`. Workload from TIME `time.week_approved`; multipliers from active parameter version.
- `FR-LEARN-003` [T] **MUST** maintain `career_level` per Member with immutable history of level changes.
- `FR-LEARN-004` [T] **MUST** support promotion nomination workflow: nominate → assemble defense pack (VP history + project lead notes + peer feedback) → Hội đồng Chuyên môn review → Council recommends → Founder/CEO approves → emits `learn.promotion_approved`.
- `FR-LEARN-005` [T] **MUST** be **seniority-independent** (Article 6a) — no time-in-level threshold; VP-and-defense gates only.
- `FR-LEARN-006` [T] **MUST NOT** stack-rank (Article 5b) — UI surfaces individual VP trends but never normalized rank against peers; stack-rank UI explicitly disallowed in code.
- `FR-LEARN-007` [T] **MUST** compute sabbatical eligibility: `eligible = floor((today − member.continuousServiceStart) / 5_years) − sabbaticalsTaken`.
- `FR-LEARN-008` [T] **MUST** support sabbatical grant: must be booked ≥3 months in advance; non-encashable; emits `learn.sabbatical_granted` consumed by HR (leave entry SABBATICAL kind) and REW (P3 = 0 during sabbatical month, P1 paid in full).
- `FR-LEARN-009` [T] **MUST** record Member training entries: courseName, provider, completionDate, evidenceUrl/file, internal certification status, cost.
- `FR-LEARN-010` [T] **MUST** maintain training catalog: curated trainings per career level / role; budget tracking.
- `FR-LEARN-011` [T] **MUST** maintain training budget per Member per year; over-budget requires Founder/CEO approval.
- `FR-LEARN-012` [T] **MUST** restrict `learn.publish_parameter_version` to `{owner, hr_lead, engineering_lead}` with dual-sign workflow.
- `FR-LEARN-013` [T] [Annex III §4 high-risk] **MUST** log every promotion-decision input + Founder approval/override; AI-assist for promotion-readiness assessment carries visible "AI-assisted; final decision by qualified human reviewer" UX label.
- `FR-LEARN-014` [T] [P2 high-risk conformity] **MUST** run quarterly bias testing on VP scoring + peer-review (demographic parity, equalized odds, 4/5ths rule) per Fairlearn / Aequitas / AIF360.

**Data model (Prisma sketch):**

```prisma
model LearnParameterVersion {
  id            String   @id @default(uuid())
  tenantId      String
  fiscalYear    Int
  versionLabel  String
  payload       Json     // VP Quality Multiplier rules
  effectiveDate DateTime
  signedBy      Json
  signedAt      DateTime
  auditHash     String
  @@unique([tenantId, fiscalYear, versionLabel])
}

model CareerLevel {
  id            String   @id @default(uuid())
  tenantId      String
  name          String
  ladder        String   // engineering | design | account | ...
  rank          Int
  competencyDescription String
  requiredVpMin Int?
  @@unique([tenantId, ladder, rank])
}

model MemberCareerLevel {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  careerLevelId String
  effectiveFrom DateTime
  effectiveTo   DateTime?
  reason      String  // initial | promotion | demotion (rare)
  @@index([tenantId, memberId, effectiveFrom])
}

model VpEntry {
  id                              String   @id @default(uuid())
  tenantId                        String
  memberId                        String
  weekStart                       DateTime
  workloadMinutes                 Int
  individualQualityMultiplier     Float
  teamQualityMultiplier           Float
  vp                              Float
  parameterVersionId              String
  computedAt                      DateTime
  @@unique([tenantId, memberId, weekStart])
}

model PromotionNomination {
  id          String   @id @default(uuid())
  tenantId    String
  memberId    String
  fromLevelId String
  toLevelId   String
  status      String   // nominated | reviewing | recommended | approved | rejected
  defensePackUrl String  // KB doc with VP history + peer feedback summary
  recommendedBy Json    // [{member_id, recommendation, comment}]
  approvedBy  String?
  approvedAt  DateTime?
}

model TrainingRecord {
  id            String   @id @default(uuid())
  tenantId      String
  memberId      String
  courseName    String
  provider      String?
  completionDate DateTime
  evidenceUrl   String?
  costVnd       BigInt?
  certificationStatus String?
}

model SabbaticalGrant {
  id              String   @id @default(uuid())
  tenantId        String
  memberId        String
  startDate       DateTime
  endDate         DateTime
  bookingRequestedAt DateTime
  approvedBy      String
  approvedAt      DateTime
}

model BiasTestRun {
  id           String   @id @default(uuid())
  tenantId     String
  fiscalQuarter String
  scope        String   // 'vp' | 'peer_review' | 'genie_persona'
  metricResults Json    // {demographic_parity: ..., equalized_odds: ..., four_fifths_ratio: ...}
  pass         Boolean
  notesUrl     String
  runAt        DateTime
}
```

**MCP tools:** `learn.get_career_status`, `learn.list_vp_entries`, `learn.nominate_promotion`, `learn.list_council_reviews`, `learn.submit_council_feedback`, `learn.approve_promotion`, `learn.publish_parameter_version`, `learn.get_sabbatical_eligibility`, `learn.book_sabbatical`, `learn.record_training`, `learn.list_training_catalog`, `learn.get_career_ladder`, `learn.run_bias_test`

### 4.16 Module: INV — Invoicing [P2] [FIXED]

**Purpose:** Generate invoices from approved time entries; collect via Stripe; reconcile.

**Functional requirements:**

- `FR-INV-001` [T] **MUST** generate invoice number per tenant per year (`{tenant_prefix}-{yyyy}-{nnnn}`).
- `FR-INV-002` [T] **MUST** aggregate line items from `time.list_entries({status: APPROVED})`; manual line items supported.
- `FR-INV-003` [T] **MUST** support currencies VND (default), USD, EUR, JPY, SGD.
- `FR-INV-004` [T] **MUST** track status `DRAFT/SENT/VIEWED/PARTIALLY_PAID/PAID/VOID/OVERDUE`.
- `FR-INV-005` [T] **MUST** generate PDF server-side (React PDF) with bilingual VN/EN; tenant-branded.
- `FR-INV-006` [T] **MUST** support Stripe payment links + webhook reconciliation; idempotent on duplicate webhook.
- `FR-INV-007` [T] **MUST** flag Vietnamese e-invoice format requirement when tenant locale is `vi-VN` (legal advisory pending — RSK-007); integrate with VN GDT e-invoice provider (OQ-016).
- `FR-INV-008` [T] **MUST** be append-only post-issue; corrections via credit notes.
- `FR-INV-009` [T] **MUST** emit `invoice.issued`, `invoice.viewed`, `invoice.collected`, `invoice.overdue` events.
- `FR-INV-010` [T] **MUST** payload `invoice.collected`: `{tenant_id, invoice_id, project_id, cash_collected_vnd, currency, exchange_rate_used}` consumed by REW + ESOP.
- `FR-INV-011` [T] **MUST** support recurring invoices (monthly retainers); auto-generated and emailed.
- `FR-INV-012` [T] **MUST** support proforma invoices.
- `FR-INV-013` [T] **MUST** support dunning (automated reminder cadence at 30/60/90 day overdue; CHAT alert to Account Manager).
- `FR-INV-014` [T] **MUST** support multi-tax (VN VAT 8%/10%; EU VAT; US sales tax via TaxJar P3 stretch).
- `FR-INV-015` [T] **MUST** export accountant-friendly monthly invoice register CSV/XLSX.

**MCP tools:** `invoicing.create_invoice`, `invoicing.send_invoice`, `invoicing.record_payment`, `invoicing.list_overdue`, `invoicing.create_credit_note`, `invoicing.create_recurring_schedule`, `invoicing.generate_pdf`, `invoicing.export_register`

### 4.17 Module: ESOP — Phantom Stock [P2] [FIXED]

**Purpose:** Encode Article 5 of the Appendix. Immutable append-only ledger (DEC-029).

**Functional requirements:**

- `FR-ESOP-001` [T] **MUST** maintain `sp_grant`, `sp_event`, `sp_valuation`, `cfo_input`, `put_option_request` as append-only tables; UPDATE/DELETE blocked at DB-policy level.
- `FR-ESOP-002` [T] **MUST** issue SP grant by `esop.issue_grant`; restricted to `{owner, board}`.
- `FR-ESOP-003` [T] **MUST** vest 4-year, 25% per year, even, no cliff. Vesting computed nightly from grant_date; conditioned on Member's annual review = "Meets Expectations" (≥100% individual KPI from LEARN). If condition not met, emit `vesting_paused`.
- `FR-ESOP-004` [T] **MUST NOT** stack-rank vesting (Article 5b).
- `FR-ESOP-005` [T] **MUST** publish annual valuation via `esop.publish_valuation` (Founder/CEO + Board dual-sign): `value_per_sp_vnd = (cfo × industry_multiplier) / outstanding_sp`. If `cfo ≤ 0`, emit valuation with `applied_floor=true` and copy prior year's value (Article 5a).
- `FR-ESOP-006` [T] **MUST** enforce pool size cap: refresh grants pre-validated `total_outstanding_sp + new_qty ≤ 0.15 × total_actual_shares`; over-cap rejected.
- `FR-ESOP-007` [T] **MUST** support put option request from Year 3 of grant: up to 25% of vested SP/year; company budget cap 15% CFO/year; over-cap structures into installment 6–12 months at ACB savings rate, valued at prior fiscal year's SP value.
- `FR-ESOP-008` [T] **MUST** support M&A acceleration: on `esop.simulate_acceleration` or actual `liquidity_event_executed` event, all SP for all Members vests immediately to 100%. Member elects cash at deal valuation OR conversion to actual ESOP shares.
- `FR-ESOP-009` [T] **MUST** enforce termination settlement (consumes `member.terminated` from HR):
  - Good Leaver → unvested cancelled; vested subject to Company right-of-first-refusal repurchase within 6–12 months at most-recent valuation; emits `good_leaver_repurchase_window_opened` with TTL
  - Bad Leaver → entire SP balance (vested AND unvested) forfeited to 0 VND; emits `bad_leaver_forfeit_all`
- `FR-ESOP-010` [T] **MUST** apply MVP Award SP multiplier: on `mvp_award_granted` event from REW, next-year SP grant for recipient gets ×1.5; recorded as event metadata.
- `FR-ESOP-011` [T] **MUST** maintain Merkle chain on `sp_event`: `audit_hash = sha256(prev_event_audit_hash || canonical_json(payload))` per grant.
- `FR-ESOP-012` [T] **MUST** materialize current vesting position + balance via materialized view refreshed hourly + on event; nightly reconciliation compares MV to event-log reconstruction; alert on drift.
- `FR-ESOP-013` [T] **MUST** redact SP balances + valuations from AI Gateway prompts unless on the explicit `esop.simulate_explain` PII-safe route.
- `FR-ESOP-014` [T] **MUST** support `esop.recompute_member_state(member_id, as_of_date)` returning vesting + balance + valuation reproducibly.
- `FR-ESOP-015` [T] [DEC-036] **MUST NOT** be ingested to BRAIN.

**Data model (Prisma sketch):**

```prisma
model SpGrant {
  id                       String   @id @default(uuid())
  tenantId                 String
  memberId                 String
  qty                      BigInt
  grantDate                DateTime
  vestingScheduleId        String   // 4y/25%/even template
  frozenIndustryMultiplier Float
  parameterVersionId       String
  status                   String   // 'active' | 'fully_vested' | 'cancelled' | 'forfeited'
  @@index([tenantId, memberId])
}

model SpEvent {
  id                String   @id @default(uuid())
  tenantId          String
  grantId           String?
  memberId          String?
  kind              String   // grant_issued | vesting_unlocked | vesting_paused | refresh_granted | valuation_published | put_option_requested | put_option_settled | m_and_a_accelerated | good_leaver_settlement | bad_leaver_forfeit
  payload           Json
  occurredAt        DateTime
  recordedBy        String
  prevAuditHash     String?
  auditHash         String
  @@index([tenantId, grantId, occurredAt])
}

model SpValuation {
  id                  String   @id @default(uuid())
  tenantId            String
  fiscalYear          Int
  cfoInputVnd         BigInt
  industryMultiplier  Float
  totalOutstandingSp  BigInt
  valuePerSpVnd       BigInt
  appliedFloor        Boolean
  boardResolutionId   String
  publishedAt         DateTime
  @@unique([tenantId, fiscalYear])
}

model CfoInput {
  id          String   @id @default(uuid())
  tenantId    String
  fiscalYear  Int
  totalCfoVnd BigInt
  source      String   // 'finance_module' | 'manual_board_entry' | 'aggregated_from_inv'
  signedBy    String
  capturedAt  DateTime
  @@unique([tenantId, fiscalYear])
}

model PutOptionRequest {
  id                  String   @id @default(uuid())
  tenantId            String
  memberId            String
  grantId             String
  qtyRequested        BigInt
  requestedAt         DateTime
  status              String   // 'pending' | 'partial_settled' | 'settled' | 'installment_active'
  installmentSchedule Json?
}

model VestingScheduleTemplate {
  id        String   @id @default(uuid())
  name      String   @unique  // '4y_25_no_cliff'
  durationMonths Int
  unlockPattern Json  // [{after_months, unlock_pct}]
}
```

**MCP tools:** `esop.issue_grant`, `esop.publish_valuation`, `esop.list_grants`, `esop.list_valuations`, `esop.request_put_option`, `esop.settle_put_option`, `esop.process_termination`, `esop.simulate_acceleration`, `esop.simulate_explain`, `esop.recompute_member_state`, `esop.execute_liquidity_event` (Founder + Board only)

### 4.18 Module: RES — Resource Allocation [P3] [FIXED]

**Purpose:** Capacity planning, scenario planning, skill matching.

**Functional requirements:**

- `FR-RES-001` [T] **MUST** support allocation per Member per week per project (hours, rate amount, rate currency).
- `FR-RES-002` [T] **MUST** provide capacity dashboard: utilization % per Member, over-allocation flags, idle capacity.
- `FR-RES-003` [T] **MUST** support drag-drop reassignment in MFE remote.
- `FR-RES-004` [T] **MUST** support skill matching from LEARN career level + training records + certifications.
- `FR-RES-005` [T] **MUST** support scenario planning ("what if we add this project?"); compare scenarios.
- `FR-RES-006` [T] **MUST** provide Gantt-style timeline view (per project, per Member).
- `FR-RES-007` [T] **MUST** detect conflicts (vacation / sabbatical / other allocation overlap).
- `FR-RES-008` [T] **MUST** forecast 4 / 8 / 12 weeks ahead utilization.
- `FR-RES-009` [T] **MUST** suggest best Member match (AI-assisted) based on skills + availability + past performance; defers to human assignment.
- `FR-RES-010` [T] **MUST** track planned vs actual hours per Member per project per week (variance).
- `FR-RES-011` [T] **MUST** support multi-allocation per week (e.g. 50% Project A + 50% Project B).
- `FR-RES-012` [T] **MUST** provide reports: utilization by department, billable %, overall capacity vs demand.

**MCP tools:** `res.allocate`, `res.update_allocation`, `res.get_capacity`, `res.suggest_assignment`, `res.create_scenario`, `res.compare_scenarios`, `res.get_skill_match`, `res.get_forecast`, `res.list_conflicts`, `res.report_utilization`

### 4.19 Module: OKR [P3] [FIXED]

**Purpose:** Quarterly OKRs with alignment, AI-generated check-ins.

**Functional requirements:**

- `FR-OKR-001` [T] **MUST** support cycles (typically quarterly; configurable to half-year or annual).
- `FR-OKR-002` [T] **MUST** support Objectives scoped to Tenant / Department / Team / Member.
- `FR-OKR-003` [T] **MUST** support Key Results with target value, unit, measure type (`SUM/LAST/AVG/PERCENTAGE`), source (`MANUAL/PROJ/CRM/REW/...`); KR can auto-update from source modules.
- `FR-OKR-004` [T] **MUST** support alignment: parent-child relationship between Objectives.
- `FR-OKR-005` [T] **MUST** support check-ins: weekly; Genie generates draft from PROJ + CHAT + KR auto-source.
- `FR-OKR-006` [T] **MUST** support confidence rating per KR (1-10 or red/yellow/green).
- `FR-OKR-007` [T] **MUST** roll up to team and tenant level.
- `FR-OKR-008` [T] **MUST** support quarterly retrospective workflow.
- `FR-OKR-009` [T] **MUST** support public visibility per Tenant policy (transparent OKRs by default within tenant).

**MCP tools:** `okrs.create_objective`, `okrs.add_key_result`, `okrs.update_kr_progress`, `okrs.create_check_in`, `okrs.summarize_team`, `okrs.list_objectives`, `okrs.align_objective`, `okrs.create_cycle`, `okrs.close_cycle`, `okrs.run_retrospective`

### 4.20 Module: DOC — Document Signing [P4] [FIXED]

**Purpose:** Wrap eIDAS QTSP (DEC-016).

**Functional requirements:**

- `FR-DOC-001` [T] **MUST** wrap eIDAS QTSP (DocuSign EU / Adobe Sign EU / Yousign / SES eIDAS) per residency.
- `FR-DOC-002` [T] **MUST** support contract templates with merge fields populated from CRM Deal + PROJ Project.
- `FR-DOC-003` [T] **MUST** support multi-signer workflows: sequential or parallel.
- `FR-DOC-004` [T] **MUST** store signed PDF + signing certificate + IP/timestamp per signer + signing intent declaration.
- `FR-DOC-005` [T] **MUST** support QES (Qualified Electronic Signatures) via QTSP and SES (Standard Electronic Signatures).
- `FR-DOC-006` [T] **MUST** be tamper-evident: signed PDF stored with hash; verification re-validates against QTSP authority.
- `FR-DOC-007` [T] **MUST** integrate with CRM (Deal moves to "Closed-Won" on signing) and HR (employment contracts archived).
- `FR-DOC-008` [T] **MUST** support reminder cadence: signer hasn't signed in 3/7/14 days.
- `FR-DOC-009` [T] **MUST** support bulk-send for templated contracts.
- `FR-DOC-010` [T] **MUST** append signing event to a Merkle audit chain.

**MCP tools:** `doc.send_for_signature`, `doc.get_status`, `doc.list_pending`, `doc.create_template`, `doc.cancel_request`, `doc.bulk_send`

### 4.21 Module: CP — Client Portal [P4] [FIXED]

**Purpose:** External Client view of projects + invoices + signing.

**Functional requirements:**

- `FR-CP-001` [T] **MUST** expose reduced-scope graph contract via Federation `@inaccessible` / `@tag` directives.
- `FR-CP-002` [T] **MUST** support auth via magic-link (default) or external IdP (OQ-009 final).
- `FR-CP-003` [T] **MUST** support tenant branded portal (logo, color, custom domain via DNS CNAME; per-Tenant CSS override).
- `FR-CP-004` [T] **MUST** provide client dashboard: their projects + status, recent task updates (read-only or comment-only), upcoming deliverables, recent invoices, pending signatures.
- `FR-CP-005` [T] **MUST** support project visibility: tasks shared via "Share with client" flag; client sees status + comments + selected attachments; cannot see internal tasks.
- `FR-CP-006` [T] **MUST** support comments routed to PROJ as Comment with `actor_kind=external_client`.
- `FR-CP-007` [T] **MUST** support approvals (UAT sign-off) → triggers `proj.uat_signed_off` event.
- `FR-CP-008` [T] **MUST** support invoice view + Stripe payment.
- `FR-CP-009` [T] **MUST** support documents archive (signed contracts).
- `FR-CP-010` [T] **MUST** support client-facing chat (optional per-Tenant; uses CHAT module with reduced scope).
- `FR-CP-011` [T] **SHOULD** support optional MCP for client-run agents (consent-gated; per-client OAuth scope).
- `FR-CP-012` [T] **MUST** support client onboarding via email invite + password + (optional) MFA.

**MCP tools:** `cp.invite_client`, `cp.share_project`, `cp.list_shared_projects`, `cp.list_invoices`, `cp.list_pending_documents`

---

## 5. Cross-Module Federation & Data Boundaries [FIXED]

### 5.1 Tenant Residency Engineering [FIXED]

(DEC-011) Every tenant has `residency` ∈ {`VN`, `EU`, `US`, `APAC`, `OTHER`}. Routing layer (Apollo Router + custom plugin) reads `tenant.residency` from JWT and dispatches read/write to the regional cluster.

| Region | Postgres host | Redis host | Object storage | LLM egress |
|---|---|---|---|---|
| **VN** | Viettel IDC / FPT Smart Cloud / VNG Cloud / AWS Hanoi LZ | VN-region Upstash | VN-region S3-compatible | Bedrock via SG/JP egress |
| EU | Neon EU | Upstash EU | Cloudflare R2 EU | Bedrock EU |
| US | Neon US | Upstash US | Cloudflare R2 US | Bedrock US |
| APAC | Neon APAC (Singapore) | Upstash APAC | R2 APAC | Bedrock SG/JP |

**CyberSkill is residency=VN** through P3 (DEC-026). External tenants in P4 select residency at signup.

A05 DPIA + CBTIA filings active before processing begins.

### 5.2 Federation Composition Rules

- Subgraphs declare entities with `@key(fields: ...)`; cross-module references are entity refs only — no cross-module DB reads.
- Schema PRs run `rover subgraph check` against the published supergraph.
- Breaking changes use progressive `@override` (Federation 2.7+); changes published only after 30-day zero-usage in GraphOS Insights.
- Field-level `@authenticated`, `@requiresScopes` directives at the Router enforce auth before reaching the subgraph.

### 5.3 Inter-Module Communication Contracts

- **Reads across modules:** GraphQL Federation entity reference (`@key`).
- **Writes across modules:** NATS event consumer pattern (publish, then react). Never call another module's REST endpoint or read its DB.
- **Long-running cross-module workflows** (P2+): Saga / Outbox pattern (§5.7).

### 5.4 Event Contract [FIXED]

NATS subject: `cyberos.{module}.{entity}.{verb}` — e.g., `cyberos.proj.task.created`.

Standard envelope:
```json
{
  "event_id": "uuid",
  "tenant_id": "string",
  "actor_id": "string",
  "actor_kind": "user | agent | system | genie",
  "occurred_at": "ISO-8601",
  "schema_version": "1.0",
  "residency": "VN | EU | US | APAC | OTHER",
  "payload": { ... }
}
```

**Critical events:**

| Event | Producer | Consumers |
|---|---|---|
| `auth.member_created` | AUTH | HR (auto-create Member profile) |
| `member.hired` | HR | LEARN (create career_level entry), REW (initial parameter binding), CHAT (welcome workflow) |
| `member.role_changed` | HR | LEARN (update career_level history), AUTH (update roles claim), REW (base salary adjustment) |
| `member.terminated` | HR | REW (BP settlement), ESOP (grant settlement), LEARN (close sabbatical), AUTH (revoke sessions), BRAIN (DSAR scheduling) |
| `chat.message_sent` | CHAT | OBS (audit ingest), BRAIN (RAG indexing) |
| `chat.channel_created` | CHAT | BRAIN (registration) |
| `proj.task.created/updated/completed` | PROJ | TIME, RES, REW (VP update on completion via LEARN), CHAT (notification), BRAIN |
| `proj.task_comment.created` | PROJ | CHAT (cross-link), BRAIN |
| `proj.uat_signed_off` | PROJ | REW (Project Bonus Pool 70% disbursement) |
| `proj.warranty_period_ended` | PROJ | REW (Project Bonus Pool 30% Holdback release) |
| `proj.client_bad_debt` | PROJ | REW (Management Risk Shield trigger) |
| `time.week_approved` | TIME | REW (workload input), LEARN (VP accumulation) |
| `crm.activity.created` | CRM | BRAIN, OBS |
| `crm.deal.stage_changed` | CRM | BRAIN, OBS, OKR (P3) |
| `kb.document.published/updated` | KB | BRAIN |
| `email.thread.received` | EMAIL | BRAIN (summary only by default), CRM (suggested log) |
| `invoice.collected` | INV | REW (P3 pool, deferred bonus release), ESOP (CFO input), OKR (revenue KR) |
| `invoice.overdue` | INV | CHAT (alert Account Manager) |
| `payslip.issued` | REW | OBS (audit), HR (paystub doc storage), Members (CHAT DM notification) |
| `learn.promotion_approved` | LEARN | HR (role + base salary update), ESOP (refresh grant eligibility), CHAT (announcement) |
| `learn.sabbatical_granted` | LEARN | HR (leave entry), REW (P3 = 0 during month), RES (capacity) |
| `mvp_award_granted` | REW | ESOP (next-year 1.5× multiplier metadata), HR (announcement), CHAT |
| `esop.valuation_published` | ESOP | OBS (audit), Members (CHAT DM optional) |
| `liquidity_event_executed` | ESOP | All ESOP grants → accelerated vesting events |
| `obs.alert_anonymized` | OBS | BRAIN (operational knowledge) |

### 5.5 Data Boundaries (PDPL + GDPR + Decree 356 Art. 30 Alignment) [FIXED]

- All `tenant_id` columns; RLS per §5.1
- AI-derived data (embeddings, LLM logs, agent traces, BRAIN chunks, summaries) classified as personal data per DEC-012
- DSAR endpoints in each module erase: rows, embeddings, AI cache references, audit log entries beyond legally required retention, BRAIN chunks
- Compensation, ESOP balances, BP balances: encrypted at rest (envelope, per-tenant KMS, BYOK at T3) per DEC-023
- BRAIN denylist enforced at ingestion (DEC-036)

### 5.6 Idempotency & Nonce Management [FIXED]

- All MCP write tools accept optional `idempotency_key`
- Server records `(tenant_id, tool, idempotency_key)` for 24 hours; duplicate within window returns prior result
- NATS event consumers idempotent on `event_id`
- BRAIN ingestion idempotent on `(tenant_id, source_module, source_entity_id, source_entity_version)`

### 5.7 Saga / Outbox Pattern (P2+ for cross-module workflows)

- Outbox table per module (`{module}_outbox`) within same tx as state change
- Worker drains outbox to NATS
- Compensating actions documented in module runbook for Saga rollback

---

## 6. AI Integration Architecture [FIXED]

### 6.1 Model Inventory (Initial)

| Use case | Model | Provider | Notes |
|---|---|---|---|
| Chat/text gen (default) | claude-3-5-sonnet | Bedrock primary; Anthropic direct fallback | ZDR + BAA available |
| Chat/text gen (fallback) | gpt-4o-mini | OpenAI ZDR | Cost-tier fallback |
| Embedding | text-embedding-3-small (1536 dim) | OpenAI ZDR | Cost-effective; cached |
| Vision (P3+) | claude-3-5-sonnet vision | Bedrock | Receipt OCR (HR expense), screenshot Q&A |
| Code (internal-only) | claude-3-5-sonnet | Bedrock | KB code-snippet help |
| Voice (P1 stretch) | Whisper API (provider TBD; OQ-020) | OpenAI / on-device | Genie voice input |

### 6.2 Pipeline Architecture

```
Caller (PROJ/CRM/KB/CHAT/EMAIL/REW-narrate-only/LEARN/ESOP-narrate-only/RES/OKR/Genie)
   │
   ▼
AI Gateway (Node) -- redaction -- budget check -- per-tenant model preference
   │
   ├── Embedding cache (Redis, 30-day TTL keyed on sha256(text))
   │
   ├── LLM provider (Bedrock primary, OpenAI/Anthropic fallback)
   │
   ├── Telemetry: New Relic AIM (input/output tokens, latency, cost)
   │
   ├── C2PA manifest signer (for AI outputs surfaced to users)
   │
   ▼
Caller receives response + provenance metadata
```

### 6.3 Inference Latency Budgets

| Use case | p50 | p95 | Hard cap |
|---|---|---|---|
| Chat reply (CHAT smart reply) | 800 ms | 1.5 s | 3 s |
| Email draft reply | 2 s | 4 s | 8 s |
| KB ask (RAG via BRAIN) | 2.5 s | 5 s | 10 s |
| BRAIN.ask | 2 s | 4 s | 8 s |
| Genie text response | 1 s | 2 s | 5 s |
| Genie voice transcription | 1.5 s | 3 s | 6 s |
| Payslip narrative explain | 2 s | 5 s | 10 s |
| Channel summary (CHAT) | 3 s | 8 s | 15 s |

### 6.4 API Contract — Inference Gateway

```typescript
type GatewayRequest = {
  route: 'chat.completion' | 'embed' | 'tool_call' | 'vision'
  tenantId: string
  actorId: string
  actorKind: 'user' | 'agent' | 'genie' | 'system'
  modelHint?: string
  prompt?: ChatMessage[]
  text?: string            // for embed
  image?: Base64String     // for vision
  redactPII?: boolean      // default true
  attachProvenance?: boolean // default true for end-user-facing
  idempotencyKey?: string
  piiSafeRoute?: boolean   // for rew.payslip_explain, esop.simulate_explain
}
type GatewayResponse = {
  output: string | Vector
  modelUsed: string
  tokensIn: number
  tokensOut: number
  costUsd: number
  c2paManifestUrl?: string
  redactionApplied?: string[]
}
```

### 6.5 Data Governance for AI [FIXED]

- All LLM logs treated as PD (DEC-012)
- ZDR / BAA verified per provider (DEC-018)
- DSAR purges: provider-side via API where supported; document gaps (provider stand-down letter)
- BRAIN ingestion respects denylist (DEC-036)

### 6.6 Model Governance & Change Control [FIXED]

- Model registry table tracks: provider, version, modality, training-data summary, eval results, deprecation date
- Annual model card review per AIMS (ISO 42001)
- Provider deprecation triggers fallback test before user-visible cutover

### 6.7 AI Compliance Primitives [FIXED]

The 7 primitives that satisfy CN/CA/VN/EU/CO/TX simultaneously:

1. **AI inventory + model registry** (provider, version, modality, training-data summary, eval results)
2. **Model + system + dataset cards** (Mitchell et al. format) — Annex IV equivalents in `kb://compliance/ai-transparency/`
3. **C2PA-signed manifests on all AI outputs** + visible "Generated by AI" label
4. **Human oversight UX** — Genie + REW + LEARN + ESOP outputs carry "AI-assisted; final decision by qualified human reviewer" label; audit log of overrides
5. **Bias testing pipeline** (Fairlearn, Aequitas, AIF360) — applied quarterly to VP scoring + peer-review + Genie persona behavior
6. **Prompt injection + RAG safety** (Lakera / Protect AI), source-citation forcing (RAG returns "I don't know" when no relevant chunks), MCP capability scoping per agent
7. **FRIA toolkit** for EU deployer customers

### 6.8 EU AI Act Annex III §4 High-Risk Conformity Pack — REW + LEARN [FIXED]

REW and LEARN make decisions affecting employment / variable compensation / promotion → Annex III §4 high-risk. Conformity pack required by P2 exit.

| Article | Requirement | Implementation |
|---|---|---|
| Art. 9 | Risk management system | `kb://compliance/ai-transparency/risk-mgmt-rew-learn` |
| Art. 10 | Data governance + bias detection | Bias testing pipeline applied to VP entries + peer-review (demographic parity, equalized odds, 4/5ths rule) — quarterly |
| Art. 12 | Logging | Every AI-assisted output to REW/LEARN logged with prompt + model + temperature + parameter_version_id |
| Art. 13 | Transparency | Visible "AI-assisted; final decision by qualified human reviewer" label; member opt-out |
| Art. 14 | Human oversight | Founder/CEO + HR/Ops Lead approval required on every promotion / parameter publish; override logged |
| Art. 15 | Robustness/cybersecurity | Adversarial prompt-injection tests in CI; encryption + audit hash chain on parameter versions |
| Art. 17 | QMS | `kb://compliance/qms` |
| Annex IV | Technical documentation | Model card + system card + dataset card per AI-Gateway use case |
| Art. 49 | EU database registration | Submitted before EU deployment goes live |
| Art. 22 | Authorised Representative | Appointed at P0 exit |

ESOP and GENIE are **not** classified as Annex III §4. ESOP: SP grant decision is a single Founder + Board-approved discretionary act, not algorithmic. GENIE: surfaces context and suggests with human confirm-step; never autonomously decides. AI Act labels still apply on Genie outputs touching high-risk module data.

### 6.9 Provider-Specific Compliance Posture [FIXED]

| Provider | Posture |
|---|---|
| AWS Bedrock | Primary; ZDR via opt-in; HIPAA BAA; supports VN-egress via SG/JP regions |
| OpenAI | Direct API + ZDR; not used for HIPAA paths |
| Anthropic | Direct API + ZDR + BAA; backup |
| Azure OpenAI | T2+ EU; not P0 |
| DeepSeek / CN-hosted | Geofenced out (DEC-018) |

---

## 7. Security & Compliance NFRs [FIXED]

### 7.1 Cryptographic Standards

- TLS 1.3 mandatory for all external endpoints (CHAT WebSocket, MCP, GraphQL, REST webhooks)
- AES-256-GCM for envelope-encrypted fields (compensation, mailbox creds, ESOP grant details with PII, government IDs, bank accounts, home address, SICK/COMPASSIONATE leave reasons)
- Argon2id for passwords (m=64MB, t=3, p=4)
- RSA-4096 or Ed25519 for signing JWTs and audit-hash chains
- HKDF-SHA-256 for key derivation
- PQC migration plan documented for 2030 TLS mandate (NFR-SEC-CRYPTO-PQC)

### 7.2 Identity, Authentication, Authorization

- OAuth 2.1 + PKCE for all programmatic access (MCP, third-party agents)
- TOTP MFA mandatory for all roles; WebAuthn passkeys at P2
- JWT exp ≤24h; refresh-token rotation with replay detection
- Per-Member API tokens for non-interactive use (limited scope, audited)
- Session revocation list propagated within 60s

### 7.3 API Security & Rate Limiting

- WAF + bot detection at edge (Cloudflare)
- Per-IP rate limit baseline; per-Member burst limits
- MCP rate limits per DEC-024
- GraphQL query depth limit (default 8); cost analysis (apollo-server cost plugin)
- Input validation via zod + GraphQL SDL types
- Output filtering: never leak ID-only foreign keys to wrong tenants

### 7.4 Secrets Management

- Doppler / AWS Secrets Manager for runtime secrets
- Per-tenant KMS data keys for envelope encryption
- BYOK supported at T3 (DEC-023)
- No secrets in env vars in source control; .env gitignored; pre-commit hook scanning (gitleaks)

### 7.5 Vulnerability Management

- Annual external pen test (P1 onward)
- Quarterly internal red-team exercises (P2 onward)
- AI-specific red-team: prompt injection, RAG poisoning, persona-jailbreak (annual P2+)
- SBOM (CycloneDX 1.6) generated per build; supply-chain attestation via SLSA
- Sigstore/Cosign signing of container images
- Dependency scanning via Renovate + Snyk + Grype; CVE SLAs:
  - Critical: 24h
  - High: 7d
  - Medium: 30d
- Pin-by-hash for non-major npm packages

### 7.6 Availability, Performance, DR

- Internal-only target: 99.5%; commercialized 99.9%
- p95 GraphQL ≤400ms; p95 CHAT message-deliver ≤200ms; p95 EMAIL inbox-list ≤1s; p95 Genie text response ≤2s; p95 BRAIN.search ≤250ms
- RPO ≤15min; RTO ≤4h
- Daily Postgres logical backup + point-in-time recovery; cross-region snapshot weekly (residency-respecting)
- BCDR tabletop annually

### 7.7 Pentest & Red-Team Cadence

| Cadence | Scope | Phase |
|---|---|---|
| Quarterly internal | New module + new DEC-touched surface | P1+ |
| Annual external | Full platform | P1+ |
| AI-specific red-team (prompt injection, RAG poisoning, Genie persona jailbreak) | Annual | P2+ |

### 7.8 PDPL Compliance (Vietnam 91/2025/QH15; 356/2025/ND-CP) [FIXED]

- Per-tenant residency enforced (DEC-011)
- A05 DPIA + CBTIA filings active before processing begins
- Internal DPO + DPD designated and registered
- 72-hour breach notification SLA tested quarterly
- Granular voluntary consent UX (no pre-tick, no bundled)
- AI-derived data treated as PD (DEC-012)
- DSAR endpoints in every module; pgvector + AI cache + BRAIN erasure included

### 7.8.1 Multi-Jurisdiction Privacy Mapping [FIXED]

| Regime | Trigger | Implementation |
|---|---|---|
| GDPR (EU) | Any EU data subject | Art. 27 Rep + DPA + EU SCCs Module 2 + DSAR endpoints |
| UK GDPR | UK data subject | UK GDPR Rep + UK Addendum |
| LGPD (Brazil) | BR data subject | Brazil-EU adequacy + DPO designation |
| PIPEDA (Canada) | CA data subject | Privacy notice + breach notification |
| PIPL (China) | CN data subject | **Geofenced out** (DEC-018) |
| Australia Privacy Act | AU data subject | DPO + breach notification + consent record |
| Singapore PDPA | SG data subject | Mandatory consent + access requests + breach 72h |
| HIPAA (US healthcare; T3+) | US covered entity | BAA + Bedrock+Anthropic only + audit log retention |

### 7.8.2 Breach Notification SLA (Multi-Clock) [FIXED]

| Regime | SLA | Trigger from |
|---|---|---|
| India CERT-In | 6 hours | Detection |
| NIS2 (EU early warning) | 24 hours | Detection |
| GDPR | 72 hours | Awareness |
| Vietnam PDPL | 72 hours | Detection |
| HIPAA | 60 days | Discovery |

Runbook tested quarterly via simulated breach exercise.

### 7.9 Logging, Monitoring, Audit [FIXED]

- All write operations emit audit events to NATS → ingested into immutable audit store (Postgres `audit_log` per module + central archival)
- Every audit event signed with HMAC; daily Merkle root published to Trust Center for tamper detection
- Logs retained ≥7 years for PDPL + financial records; legal hold capability

### 7.9.1 Supply Chain & SBOM [FIXED]

- CycloneDX 1.6 SBOM per build
- Sigstore/Cosign signing of container images
- Renovate + Snyk + Grype scanning in CI
- Pin-by-hash for non-major npm packages

### 7.9.2 Cryptographic Roadmap & PQC [FIXED]

- 2027: pilot hybrid TLS 1.3 with X25519 + Kyber768
- 2028: hybrid default for new deployments
- 2030-01-02: TLS 1.3 hard-mandated; PQC hybrid expected per CISA guidance
- BYOK keys supported in PQC variants by Q4 2029

### 7.9.3 EU Data Act Compliance [FIXED]

- Switching APIs: machine-readable export per module
- 2-month notice period configured in Tenant Admin (P4)
- 30-day transition window
- **Zero switching fees** from 2027-01-12

### 7.10 Scalability (Aqueduct)

- Postgres: monthly partitioning on high-volume tables (`chat_message`, `audit_log`, `email_message`, `knowledge_chunk`) at 100M-row threshold
- Redis: cluster mode for >10k concurrent Socket.IO connections
- NATS: clustering at >5k msgs/sec
- LLM: per-tenant request queue with backpressure to AI Gateway
- BRAIN: chunk count per tenant monitored; partition by month at 10M+ chunks

---

## 8. System-Wide Non-Functional Requirements [FIXED]

| ID | Category | Requirement | Verification |
|---|---|---|---|
| NFR-PERF-001 | Performance | p95 GraphQL ≤400ms at 100 RPS/tenant | T |
| NFR-PERF-002 | Performance | p95 CHAT message-deliver ≤200ms (server-side) | T |
| NFR-PERF-003 | Performance | p95 EMAIL thread list ≤1s | T |
| NFR-PERF-004 | Performance | p95 KB hybrid search ≤800ms | T |
| NFR-PERF-005 | Performance | p95 REW payslip compute ≤3s for 10 Members | T |
| NFR-PERF-006 | Performance | p95 BRAIN.search ≤250ms | T |
| NFR-PERF-007 | Performance | p95 BRAIN.ask ≤4s | T |
| NFR-PERF-008 | Performance | p95 Genie text response ≤2s | T |
| NFR-PERF-009 | Performance | BRAIN ingest p95 ≤5s from source event | T |
| NFR-AVAIL-001 | Availability | ≥99.5% (internal); ≥99.9% (commercialized) | A |
| NFR-REL-001 | Reliability | RPO ≤15min, RTO ≤4h | T |
| NFR-SEC-AUTH-001 | Security | TOTP MFA mandatory for all roles | T |
| NFR-SEC-CRYPTO-001 | Security | TLS 1.3 mandatory external | I |
| NFR-SEC-CRYPTO-002 | Security | AES-256-GCM envelope encryption for compensation, ESOP balances, mailbox creds, IDs, addresses | T |
| NFR-SEC-CRYPTO-003 | Security | BYOK at T3 | T |
| NFR-SEC-RLS-001 | Security | Cross-tenant negative test in CI per module | T |
| NFR-SEC-AUDIT-001 | Security | All write operations emit audit event with actor + before + after | T |
| NFR-SEC-AUDIT-002 | Security | BP ledger + SP event log Merkle chain verifiable | T |
| NFR-SEC-AUDIT-003 | Security | CHAT message Merkle chain verifiable | T |
| NFR-COMP-001 | Compliance | DSAR endpoints operational at P1 exit | D |
| NFR-COMP-002 | Compliance | A05 DPIA + CBTIA filed at P0 exit | D |
| NFR-COMP-003 | Compliance | C2PA manifest on all AI outputs | T |
| NFR-COMP-004 | Compliance | EU AI Act Annex III §4 conformity pack for REW + LEARN at P2 exit | D |
| NFR-COMP-005 | Compliance | Anti-retroactive recompute test passes on stored payslips | T |
| NFR-COMP-006 | Compliance | Parameter version table immutable at DB-policy level | T |
| NFR-COMP-007 | Compliance | BRAIN denylist enforced (zero REW/ESOP chunks) | T |
| NFR-INT-001 | Internationalization | vi-VN default + en-US parity; Vietnamese prevails on legal-text conflict | I |
| NFR-USE-001 | Usability | WCAG 2.2 AA compliance | I |
| NFR-COST-001 | Cost | Internal infra ≤$380/month at P2 | A |
| NFR-COST-002 | Cost | LLM spend ≤$150/month at internal scale | A |
| NFR-MAINT-001 | Maintainability | Module CI duration ≤10min | T |
| NFR-MAINT-002 | Maintainability | Statement coverage ≥70% per module | T |
| NFR-DEPL-001 | Deployment | Per-module independent deploy ≤10min | T |
| NFR-OBS-001 | Observability | Every external call traced + logged with correlation ID | I |
| NFR-CHAT-001 | Functional | CHAT supports ≥1000 concurrent WebSocket connections per node | T |
| NFR-EMAIL-001 | Functional | EMAIL supports ≥30 concurrent IMAP IDLE connections per worker | T |
| NFR-REW-001 | Functional | REW recompute_payslip produces byte-identical output for stored payslips | T |
| NFR-ESOP-001 | Functional | ESOP recompute_member_state reproduces vesting + valuation + balance | T |
| NFR-BRAIN-001 | Functional | BRAIN ANN recall ≥90% top-10 vs exhaustive | T |
| NFR-BRAIN-002 | Functional | DSAR cascade (`brain.forget`) p95 completes ≤10s for ≤100k chunks | T |
| NFR-GENIE-001 | Functional | Genie persona test suite passes 100% per release | T |
| NFR-GENIE-002 | Functional | Genie source-citation rate ≥98% | A |

---

## 9. Verification & Acceptance

### 9.1 Phase-Gate Criteria — Three-Lane Table (Bound with PRD §8 + §10)

| Phase | Technical evidence | Compliance evidence | Operational evidence |
|---|---|---|---|
| P0 exit | All P0 module FRs green; cross-tenant negative test passing; CHAT realtime load test ≥100 concurrent; BRAIN denylist test passing; Genie persona test suite passing | A05 DPIA + CBTIA filed; DPO/DPD designated; Authorised Reps appointed; Trust Center live | CHAT migration: Slack + Zalo decommissioned for CyberSkill; ≥9/10 Members daily on Genie; BRAIN ≥10k chunks |
| P1 exit | All P1 module FRs green; DSAR APIs working with pgvector + AI cache + BRAIN erasure | SOC 2 Type I report; CSA STAR L1; AI-CAIQ Valid-AI-ted | First payroll run via REW; ≥1 promotion review via LEARN; ≥1 shared inbox via EMAIL; HR full-feature operational |
| P2 exit | INV cash-collected → REW + ESOP integration tested end-to-end; AI Act conformity tests pass | SOC 2 Type II; ISO 27001:2022; CSA STAR L2; REW+LEARN AI Act Annex III §4 conformity pack | First Project Bonus Pool with 70/30 disbursement; first SP grant + valuation cycle; sabbatical accrual computed |
| P3 exit | RES + OKR FRs green; ISO 42001 controls operational | ISO 42001 cert; ISO 27701 if applicable | First quarterly OKR cycle closed; Singapore HoldCo flip executed if ARR threshold met |
| P4 exit | DOC + CP FRs green; switching API operational | EU Data Act compliance; per-tenant export + erasure verified within 30 days | First external paying tenant onboarded |

### 9.2 Module Ready (Per-Module Exit)

Module is "Ready" within its phase when:
- [ ] All FRs implemented with passing automated tests (statement coverage ≥70%)
- [ ] All Gherkin scenarios in `modules/{mod}/test/scenarios/` green
- [ ] MCP tools registered, with at least one read tool and one write tool
- [ ] MFE remote deployed; routes wired into shell
- [ ] Module-specific NFRs verified (perf, reliability)
- [ ] Audit log entries verified for every write tool
- [ ] RLS policy exists and is enforced (verified by negative cross-tenant test)
- [ ] Module README + runbook written
- [ ] Tenant data export job includes module's tables
- [ ] **For REW/LEARN/ESOP:** parameter version table populated with v0 ("genesis"); recompute tests passing; AI Gateway lint rule confirmed
- [ ] **For CHAT:** Socket.IO Redis adapter verified across multiple nodes; Merkle audit chain verified
- [ ] **For EMAIL:** IMAP IDLE supervised restart verified; OAuth refresh tested; per-tenant residency tagged credential storage verified
- [ ] **For BRAIN:** denylist test passing; DSAR cascade test passing; ingestion p95 ≤5s verified
- [ ] **For GENIE:** persona test suite passing; persona version v0 published; source-citation rate ≥98% on test queries

### 9.3 Traceability (SRS → PRD)

Each FR-{MOD}-{NNN} maps to one or more PRD §5.2 product behaviors. Each NFR maps to a PRD §4 KPI or §10 compliance regime. Mapping maintained in `kb://traceability` (auto-generated from FR/NFR frontmatter when added).

### 9.4 Testing Strategy

- **Unit:** Vitest per module
- **Integration:** Testcontainers Postgres + Redis + NATS per module
- **Contract:** `rover subgraph check` against published supergraph
- **E2E:** Playwright on shell + remote pages
- **Load:** k6 per module; CHAT scenario uses Socket.IO client driver
- **Security:** SAST (CodeQL), DAST (ZAP) in CI; quarterly internal red-team
- **AI:** prompt injection corpus (Lakera); RAG faithfulness eval; bias eval (Fairlearn) on REW + LEARN outputs; persona test suite for Genie
- **Compliance:** anti-retroactive recompute test as a CI guardrail; parameter version DB-policy test; cross-tenant negative test mandatory per module; BRAIN denylist test

---

## 10. Engineering Delivery Organization [FIXED]

### 10.1 Module Independence Contract

What a module team owns end-to-end:
1. **Subgraph** — `modules/{module}/api/` — Apollo Server 5 + Express + `@apollo/subgraph`
2. **Database** — `modules/{module}/db/` — Prisma schema + migrations (own tables only; cross-module FKs forbidden)
3. **Frontend remote** — `modules/{module}/web/` — Vite + React, exposed via Module Federation
4. **MCP tools** — `modules/{module}/mcp/tools.ts` registered at central MCP server startup
5. **Tests** — unit (Vitest), integration (Testcontainers), contract (`rover subgraph check`), e2e
6. **Deployment** — own Dockerfile, GitHub Actions matrix entry, Railway/Fly app
7. **Docs** — `modules/{module}/README.md`, runbook (folded into KB Space `compliance/runbooks` or per-module space)

What a module team does NOT own (shared platform):
- Authentication / JWT issuance (consumes from AUTH)
- AI Gateway abstraction (consumes from `@cyberos/ai-gateway`)
- Design system primitives (consumes from `@cyberos/ui`)
- GraphOS Router config (platform-owned)
- Tenancy middleware / RLS policy template (platform-owned, applied per module)
- Genie portable component (consumes from `@cyberos/genie-ui`)
- BRAIN client SDK (consumes from `@cyberos/brain-client` for opt-in module RAG)

To start a module — Kickoff Checklist:
- [ ] Module code added to module catalog
- [ ] FR-{MOD}-001..NNN drafted in SRS
- [ ] Owner role assigned (RACI)
- [ ] Dependencies on other modules listed; required entity `@key` fields documented
- [ ] BRAIN ingestion policy entry added (if applicable; allowlist vs denylist)
- [ ] Subgraph + MFE template scaffolded via `pnpm gen module {MOD}`
- [ ] First migration created (with `tenant_id` + RLS policy)
- [ ] Pipeline runs green (`turbo run lint test build`, `rover subgraph check`)

### 10.2 Squad Composition (P0–P1)

Through P1, the team is small. Effective composition:
- **Founder/CEO** — Approver on every architectural decision; daily Cowork driver
- **Engineering Lead** — Implements P0 + P1 modules; runs CI + deployment
- **HR/Ops Lead** — Owns REW, LEARN, HR module workflows; executes monthly payroll cycle (joins by P1)
- **Account Manager** — Owns CRM workflows; runs client engagements (joins by P1)
- **AI Agent (Claude)** — Continuous co-developer + co-operator; daily

Module owners hand-off to Members as the team grows past P1.

### 10.3 RACI — Technical Decision Rights

| Activity | Founder/CEO | Eng Lead | HR/Ops Lead | Module Owner | CWG |
|---|---|---|---|---|---|
| Locked Decision (cross-cutting) | A,R | C | C | I | C |
| Locked Decision (module-local) | A | R | C (if HR/REW/LEARN/ESOP) | R | I |
| Parameter version publish (REW/LEARN) | A,R | C | R | — | I |
| Parameter version publish (ESOP) + valuation | A,R + Board | C | I | — | I |
| Genie persona version publish | A,R | R | C | — | I |
| Production deploy | A | R | C | R | I |
| Incident response (sev-0) | A,R | R | C | C | C |

### 10.4 Delivery Cadence & Evidence Ledger

- Weekly CWG: risk register, audit prep, parameter publish queue, persona version queue, DSAR queue, breach SLA test
- Bi-weekly module owner sync via CHAT `#cyberos-build`
- Monthly REW payroll close: HR/Ops Lead drives; Founder/CEO approves
- Quarterly Phase-Gate review (when applicable)
- Quarterly bias testing on AI outputs
- Annual: SP valuation cycle, parameter version refresh, external pen test, model card review

Evidence ledger: every signed off deliverable referenced in CHANGELOG with link to artifact; SOC 2 / ISO auditors fed from this ledger.

### 10.5 Tooling

- **Source:** GitHub `cyberskill-official/cyberos`
- **Mono-repo:** Turborepo + pnpm + Changesets
- **CI:** GitHub Actions (matrix per module)
- **Registry:** Apollo GraphOS Free (DEC-021)
- **Observability:** New Relic Free + AIM
- **APM:** New Relic Node.js agent on every node process
- **Secrets:** Doppler / AWS Secrets Manager
- **Issue tracking:** PROJ (dogfood); CHAT for discussion
- **Compliance evidence:** Sprinto / Drata / Vanta (one of three by P1 exit)

---

## 11. Compliance Strategy (full) [FIXED]

This section is the single source for CyberOS compliance posture. (PRD §10 covers the strategic framing; this section covers operational implementation. There is no separate Compliance-Strategy document at v1.0.)

### 11.1 Tier Model Recap

See PRD §10.2 and SRS §3.1.1 for the full T1–T4 model.

### 11.2 Vietnam Home Regime Operational Detail

**A05 DPIA filing process:**
1. New processing identified (new module, new data class, new third-party)
2. DPIA template populated (KB doc `compliance/templates/a05-dpia`)
3. Internal DPO reviews
4. Filed with A05 within 60 days

**CBTIA filing process:**
1. New cross-border transfer identified
2. CBTIA template populated
3. DPO + counsel review
4. Filed with A05 within 60 days

**DSAR processing:**
1. Member submits via Trust Center or self-serve portal
2. Identity verified (MFA + email confirmation)
3. DPO reviews; routes to relevant module owners
4. Each module exports / erases per its DSAR API
5. BRAIN cascade via `brain.forget(member_id)`
6. Receipt issued within statutory period (30 days GDPR; per VN PDPL)

**Breach response (multi-clock):**
1. Detection: OBS sev-0 alert; Engineering Lead triage within 15 min
2. Founder/CEO + DPO convened
3. India CERT-In notification within 6h if applicable
4. NIS2 EU early warning within 24h if applicable
5. GDPR/PDPL notification within 72h if applicable
6. Affected Members + Tenants notified
7. Post-mortem within 7 days

### 11.3 Cert Sequence Operational Plan

| Cert | Phase | Cost (Y1) | Auditor candidates |
|---|---|---|---|
| Stripe SAQ-A AOC | P0 | $5k | self-attest |
| WCAG 2.2 AA (VPAT 2.5 INT) | P0 | $3k | self-attest with axe + manual review |
| CSA STAR L1 (CAIQ v4) | P0 | Free | self-attest |
| AI-CAIQ (Valid-AI-ted) | P0 | $595 | self-attest |
| SOC 2 Type I | P1 | $10–15k | one of: A-LIGN, Insight Assurance, Dansa D'Arata Soucia |
| SOC 2 Type II | P2 | $20–30k | same auditor as Type I |
| ISO 27001:2022 + 27017 + 27018 | P2 | $15–25k | one of: BSI, Schellman, Coalfire |
| CSA STAR L2 | P2 | bundled | with ISO 27001 |
| Cyber Essentials Plus (UK) | P2 if applicable | £3–5k | IASME-certified body |
| ISO 42001 (AIMS) | P3 | $20–30k | BSI / similar |
| ISO 27701 (PIMS) | P3 if applicable | $10–15k | bundled with 27001 renewal |
| HIPAA-eligible | P3 if healthcare prospects | $15k | self-assessment + BAA stack |

### 11.4 Authorised Representatives

| Role | Provider | Cost (Y1) |
|---|---|---|
| EU GDPR Art. 27 | Prighter / EDPO / DataRep | €500–2k |
| UK GDPR Rep | same provider | £500–2k |
| EU AI Act Art. 22 | same provider (post-REW/LEARN AI Act high-risk) | €1–3k |
| NIS2 Art. 26(3) | same provider (post-EU enterprise scope) | €500–2k |

### 11.5 Trust Center Operational Detail

`trust.cyberskill.world` — KB Space marked "public-readable." Contains:
- SOC 2 / ISO certs (when issued)
- CAIQ v4 + AI-CAIQ
- GDPR DPA + EU SCCs templates
- BAA template
- Sub-processor list with location + certifications
- Breach SLA documentation
- AI transparency pack:
  - Model cards per LLM in registry
  - System card for Genie + REW + LEARN
  - Dataset card
  - Training data summary
  - C2PA implementation note
  - FRIA template
- Security whitepaper
- BCDR summary
- SBOM (per release)
- CISA Secure by Design pledge listing
- Daily Merkle root of audit_log

### 11.6 Decline List

See PRD §10.12. Engineering rule: any feature request implying T4 government / classified scope is rejected at backlog triage with link to PRD §10.12.

### 11.7 Singapore HoldCo Flip Plan

Triggered at ARR $1.5–2M:
1. Singapore Pte Ltd incorporated; CyberSkill VN OpCo unchanged
2. IP transferred to SG HoldCo via arm's-length licensing agreement
3. Customer contracts shift to SG HoldCo on renewal
4. VN OpCo provides software-development services under arm's-length services agreement
5. Preserves 0% VAT export-of-services treatment for VN→SG intercompany flows
6. Singapore corporate counsel engaged
7. Avoid Delaware C-corp until US Tier-1 VC (GILTI/Subpart F treatment of VN OpCo as a CFC is punitive)

---

## 12. Appendices

### Appendix A — Module Dependency Graph

```
P0:
  AUTH ──┐
         ├── AI ──┐
         ├── MCP ─┤
         ├── OBS ─┤
         ├── CHAT ──> BRAIN ──> GENIE
         └────────────────────────┘
                 │ (BRAIN consumes CHAT events; GENIE depends on AI + MCP + BRAIN)

P1:
  AUTH ──> PROJ ──> BRAIN
  AUTH ──> TIME (depends on PROJ)
  AUTH ──> CRM ──> BRAIN
  AUTH ──> KB (depends on AI) ──> BRAIN
  AUTH ──> HR ──> BRAIN (non-comp only)
  AUTH ──> EMAIL ──> BRAIN (summaries only)
  AUTH + HR + TIME + LEARN ──> REW (compensation core; full pool waits on INV)
  AUTH + HR + TIME ──> LEARN ──> BRAIN (training + outcomes only)

P2:
  AUTH + TIME + CRM ──> INV
  AUTH + HR + INV + LEARN ──> ESOP
  INV ──> REW (full pool calc activates)

P3:
  AUTH + PROJ + HR + LEARN ──> RES
  AUTH + PROJ + CRM + REW + INV ──> OKR

P4:
  AUTH + CRM + PROJ + HR ──> DOC
  AUTH + PROJ + INV + DOC ──> CP
```

### Appendix B — Standard Subgraph Skeleton (Template)

Located in `modules/_template/api/`. Includes:
- Apollo Server 5 boot
- Federation v2 SDL with `@key` declarations
- JWT verification middleware
- Tenant middleware (sets `app.tenant_id` Postgres session var)
- RLS policy template
- Audit emit helper (NATS publish)
- BRAIN ingestion event emit helper
- MCP tool registry hook
- Prisma client per module
- Vitest + Testcontainers integration test setup

### Appendix C — Standard MFE Remote Skeleton (Template)

Located in `modules/_template/web/`. Includes:
- Vite + React 19+
- Module Federation expose config
- `@cyberos/ui` design system import
- `@cyberos/genie-ui` Genie portable component
- i18n setup (vi-VN + en-US)
- JWT-based fetch helper
- Error boundary
- axe accessibility test in CI
- Storybook setup for design review

### Appendix D — RLS Policy Template

```sql
ALTER TABLE {table_name} ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON {table_name}
  USING (tenant_id = current_setting('app.tenant_id')::text)
  WITH CHECK (tenant_id = current_setting('app.tenant_id')::text);

-- Optional: parameter version / immutable table
CREATE OR REPLACE FUNCTION raise_immutable() RETURNS trigger AS $$
BEGIN
  RAISE EXCEPTION 'Table is append-only; UPDATE/DELETE not permitted';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_update_delete BEFORE UPDATE OR DELETE ON {immutable_table}
  FOR EACH ROW EXECUTE FUNCTION raise_immutable();
```

### Appendix E — Glossary (technical terms only; PRD Appendix A has the full glossary)

| Term | Definition |
|---|---|
| **MV** | Materialized View — used in ESOP for current vesting + balance state derived from event log |
| **RRF** | Reciprocal Rank Fusion — k=60 used by KB, CRM, BRAIN hybrid search |
| **APQ** | Automatic Persisted Queries — Apollo client/server feature to reduce GraphQL payload |
| **ZDR** | Zero Data Retention — provider commitment not to log/retain prompts |
| **BAA** | Business Associate Agreement — HIPAA contract |
| **DSAR** | Data Subject Access Request |
| **ROPA** | Record of Processing Activities |
| **HNSW** | Hierarchical Navigable Small World — pgvector index type |
| **PGroonga** | PostgreSQL extension for full-text search; supports Vietnamese |
| **NATS JetStream** | Persistent streaming variant of NATS |
| **Federation v2.7+** | Apollo Federation with progressive `@override` |

### Appendix F — Compliance Cross-Reference

| Compliance regime | SRS sections | PRD sections |
|---|---|---|
| Vietnam PDPL Law 91/2025 + Decree 356 | §3.1.1, §5.1, §5.5, §6.5, §7.8, §11.2 | §10.3 |
| GDPR | §3.1.1, §5.5, §7.8.1 | §10 |
| EU AI Act Annex III §4 (REW + LEARN) | §3.1.1, §6.7, §6.8 | §10.6 |
| EU Data Act | §3.1.1, §7.9.3 | §10 |
| NIS2 | §7.8.2 | §10 |
| SOC 2 | §7.5–§7.10, §11.3 | §10.4 |
| ISO 27001:2022 + 27017 + 27018 | §7.5–§7.10, §11.3 | §10.4 |
| ISO 42001 | §6.6, §6.7, §11.3 | §10.4 |
| C2PA / SB 942 / VN AI Law / EU AI Act Art. 50 | §6.7 (primitive 3) | §10.5 |

### Appendix G — Reference Standards

- Apollo Federation v2.7+ specification
- Apollo Server 5 documentation
- Prisma 5.x ORM documentation
- MCP TypeScript SDK v2 + Spec 2025-11-25
- Module Federation Runtime
- Socket.IO 4.x
- IMAP4 (RFC 3501), SMTP (RFC 5321), iCalendar (RFC 5545), WebSocket (RFC 6455)
- Vietnamese PDPL Law 91/2025/QH15 + Decree 356/2025/ND-CP
- Vietnam AI Law 134/2025
- EU AI Act (Regulation 2024/1689)
- ISO/IEC 25010:2023 NFR taxonomy
- IEEE 830-1998 SRS structure
- ISO/IEC/IEEE 29148:2018 requirements engineering
- C2PA Content Credentials v2.0
- WCAG 2.2 AA
- NIST CSF 2.0
- OWASP Top 10:2021 + ASVS 4.0

---

*End of SRS v1.0 — official, single source of truth alongside PRD.md.*
