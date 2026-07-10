---
title: AUTH — P0 · slice 2 stub → P3 full · Lumi tenant identity · Agent-equal · CyberOS
source: website/docs/modules/auth/index.html
migrated: FR-DOCS-002
---

AUTH is the **identity and authorisation service** for every actor inside CyberOS — humans (CEO, CFO, members, clients) and agents (CUO persona-versioned tokens, Skill bot accounts, scheduled tasks). It speaks OAuth 2.1 + PKCE to the world, OIDC for SSO federation, WebAuthn L3 for passkey enrolment, and TOTP / RFC 6238 for the soft-MFA fallback. Inside, it sits on PostgreSQL with row-level security keyed by `tenant_id`, a Redis hot-path session cache, and an HSM-resident RS256 signing key. Every authentication and authorisation decision lands as a chained audit row in memory — so "did this agent really have permission to delete that record?" is never an interpretation, it is a replay. 

P0 · slice 2 stub status

Designed · RFC drafted

5 slices · 5 open Qs in RFC §6

P3 full status

Designed · P3 · exit target

full WebAuthn L3 + per-tenant authz

P0 · slice 2 stub LoC

~1,500

slice 1: tenant + subject CRUD + RLS

P3 full LoC

~7,000

Rust (axum) + sqlx · 5 slices

Planned tests

120+

incl. WebAuthn / OAuth flow suite

RBAC roles

5 (stub) → 22 (full)

closed catalogue · see §RBAC

JWT

RS256 · 15 min

refresh: 30 d rotating · KMS at slice 5

MFA

TOTP (P0 · slice 2) → WebAuthn (P3)

passkey mandatory at T2+ once full

Depends on

memory · OBS · AI Gateway

audit + telemetry sink + cost-of-everything

Enables

Lumi's memory tenant isolation

Stage 3 of universal protocol

0

## The bigger picture — three strategic moves

AUTH on this page is presented at _three time horizons simultaneously_ : the P0 · slice 2 stub that unblocks the rest of P0, the P3 full identity platform, and the Lumi's memory tenant identity that turns memory's universal protocol into a multi-tenant cloud product. Holding all three lenses lets engineers reading the page cold understand both what ships first and what the end-state looks like — without conflating them. 

Move 1 · P0 · slice 2 stub

🚀

Unblock the rest of P0

Magic-link login + TOTP MFA + 5 hard-coded RBAC roles + JWT RS256 with dev keys. ~1,500 LoC. Lands at P0 · slice 2 so CHAT/CUO Phase 1/OBS instrumentation can identify the actor on every cross-module call. **Per research review §2.4 — AI Gateway is P0 #1, not AUTH.**

Scope: tenant + subject CRUD with RLS · admin REST · audit-chain bridge · 15 tests

Move 2 · P3 full

🔐

Production-grade identity

WebAuthn L3 passkeys · per-tenant OAuth 2.1 authz server with PKCE · OIDC SSO · KMS-wrapped signing keys with JWKS rotation · all 22 RBAC roles · Scope Contract Grants · impossible-travel detection · HIBP password-breach check · audit-chain emission on every decision. ~7,000 LoC across slices 2–5. P3 (P3 · exit) target.

Slices 2–5 in RFC §3 · 120+ tests · OAuth 2.1 conformance suite

Move 3 · Lumi tenant identity

☁️

Multi-tenant Lumi's memory

Issues tenant-scoped JWTs that Lumi's memory (cloud-hosted org tenant) consumes for row-level isolation. Stage 3 of the universal memory protocol depends on this. Per-tenant residency pinning (vn-hanoi-1 / sg-1 / eu-fra-1) flows through tenant config to AUTH session config to memory read scoping.

Per MEMORY_AUTOSYNC_DESIGN.md §6 · enables Lumi's memory deployment

### The reordered P0 build sequence (per research review §2.4)

gantt title P0 module build sequence — reordered per research review dateFormat YYYY-MM-DD axisFormat M+%w section Existing memory (shipped) :done, memory, 2026-05-01, 1d SKILL (shipped) :done, skill, 2026-05-01, 1d OBS instrumentation :active, obs, 2026-05-01, 90d section P0 · slice 1 AI Gateway slice 1 :crit, ai, 2026-06-01, 30d section P0 · slice 2 AUTH slice 1 (stub) :crit, auth1, after ai, 30d MCP Gateway slice 1 :mcp, after auth1, 15d section P0 · slice 3 → P0 · exit CHAT slice 1 :chat, after mcp, 30d CUO Phase 1 (shipped) :done, cuo, 2026-05-01, 1d section P0 · exit → P2 · exit AUTH slices 2-4 (full) :auth_full, after chat, 180d section P3 phase AUTH slice 5 (KMS+OIDC) :auth5, after auth_full, 60d Lumi's memory Stage 3 :crit, lumi, after auth5, 30d 

**Why this order:** AI Gateway is the cost-of-everything-else gate (every CHAT @genie, every CUO Phase 2, every memory semantic search depends on it). AUTH at P0 · slice 2 is a stub because for the first 6 months CyberSkill has 10 Members and one tenant — full WebAuthn + 22-role RBAC at P0 · slice 1 is over-engineering. Per reviewer: "AUTH at P0 · slice 2 can be magic-link + TOTP with a stub RBAC. WebAuthn passkeys for the Founder and a per-tenant authz server can land at P0 · exit without blocking any other module." 

1

## Why AUTH exists

Per-module auth is one of the easiest mistakes to make and one of the hardest to unmake. The first version of any platform ships with one module that has "owns its own login", the second module copies the pattern, the third inherits a subtle bug, and within a year there are four MFA implementations, three session models, and zero coherent answer to "who can do what". CyberOS treats this as a P0 design constraint: a single authentication service, a single authorisation predicate, a single audit trail. Every module that wants to know "can this actor do X to that resource?" asks AUTH the same way. 

🆔

One identity, many shapes

Humans, agents, service accounts, and API keys all materialise as `Subject` rows. Same predicate; different scopes.

⚖️

Agent-equal evaluation

An agent JWT is not a magic backdoor. The RBAC engine evaluates it with exactly the same code path as a human JWT — just with a smaller, persona-bound scope.

📜

Every decision auditable

Login, logout, MFA challenge, role grant, scope decision — each one writes a chained row to memory. "Did Compliance permit this?" → grep `auth.decision`.

The bet is the same bet memory makes about audit: pay the cost once at the infrastructure layer, and every module inherits the property for free. Without AUTH as a shared plane, every module re-implements login, every module ships its own MFA bug, and "who has access to what" is a vibes question. With AUTH as a shared plane, the answer is a row in `auth.decision` and a JWT claim — the regulator can read it, the auditor can verify it, the engineer can replay it. 

2

## What it does — 5W1H2C5M

A structured decomposition of AUTH's scope. Every cell traces back to + §9.6 + §11.2.3.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is AUTH?| A Postgres-backed OAuth 2.1 + OIDC server with WebAuthn / TOTP MFA, an RBAC engine, a Scope Contract Grant evaluator, and an HSM-resident JWT signing key. Single Rust binary, exposed over gRPC internally and HTTPS externally.  
**5W · Who**|  Who authenticates?| **Humans:** founders, members, clients, employees, contractors. **Agents:** CUO persona-stamped tokens, Skill bot accounts, scheduled tasks. **Services:** module-to-module mTLS clients (separate cert root). **Owner:** CSO seat (interim CEO until filled).  
**5W · When**|  When does auth happen?| (a) at session start (login + MFA); (b) at every cross-module API call (RBAC predicate); (c) at every MCP tool invocation (Scope Contract Grant); (d) on token refresh (rotation + reuse detection). Hot-path RBAC is sub-millisecond against Redis cache.  
**5W · Where**|  Where does it run?| P0: single region (Singapore SG-1) backed by AWS RDS Postgres + ElastiCache Redis. P3: multi-region active-active with eventual JWT consistency. WebAuthn / TOTP secrets at rest in KMS-wrapped column-level encryption.  
**5W · Why**|  Why a separate layer?| Because per-module auth is the single biggest source of platform-wide CVE class regression. Centralise the primitive, never the policy.  
**1H · How**|  How does it work?| OAuth 2.1 with PKCE for browser flows; `client_credentials` for service-to-service; `refresh_token` with rotation + reuse detection for long sessions; WebAuthn discoverable credentials for passkey login; TOTP RFC 6238 for fallback; OIDC for SSO federation with Google / Microsoft 365 / Okta. RBAC is a single Rego-like predicate evaluated at every cross-module call. Each decision is a chained audit row.  
**2C · Cost**|  Cost budget?| P0: ~$60/month (RDS t4g.small + ElastiCache cache.t4g.micro + one Fargate task). 50-tenant: ~$220/month. Per-login cost ~$0.000002 amortised; RBAC predicate ~$0.00000005.  
**2C · Constraints**|  Constraints?| (a) FIDO Alliance L3 WebAuthn for elevated roles. (b) Vietnamese Decree 53: data-residency proof for VN tenants. (c) Vietnam Decree 53 Art. 26 — incident notification ≤ 24h to MoPS. (d) PDPL Art. 14 — DSAR data export. (e) Per-module auth is strictly forbidden.  
**5M · Materials**|  Stack?| Rust 1.81 · axum 0.7 · sqlx · PostgreSQL 16 · Redis 7 · WebAuthn-rs · totp-rs · rsa for JWT signing · AWS KMS for signing-key wrap · OpenTelemetry SDK · LiteFS for read replicas at P3+.  
**5M · Methods**|  Method choices?| OAuth 2.1 (not 2.0 — PKCE mandatory). RBAC + Scope Contract Grants (no ABAC complexity at P0). JWT RS256 (not HS256 — separates signer from verifier). Argon2id for password hashing. Refresh-token rotation with reuse detection (RFC 6749 §10.4). Row-level security on every identity table keyed by tenant_id.  
**5M · Machines**|  Deployment?| Fargate task in SG-1 (P0). Multi-AZ Postgres RDS. Redis cluster mode at P3+. WebAuthn relying-party ID = the company root domain (P0: `cyberos.com`).  
**5M · Manpower**|  Who maintains?| 0.5 FTE today (covered by CEO). By P1: CSO seat owns 100% capacity + 24/7 on-call rotation with CTO.  
**5M · Measurement**|  How measured?| N(FR pending)..012 — zero tenant-data leakage, zero compensation in memory, mandatory mTLS, OWASP Gen AI Top-10 mitigations all green. Per-tenant auth-decision dashboard. Annual penetration test (P2+).  
  
2.5

## P0 · slice 2 stub vs P3 full — what ships when

Per the reviewer's reorder, AUTH ships in two distinct shapes separated by 10 months. The **P0 · slice 2 stub** is mergeable in one week; it unblocks every other P0 module by giving them an actor identity. The **P3 full** AUTH ships over slices 2–5 across P0 · exit through P3 · exit and is what the SOC 2 Type II auditor evaluates. Confusing the two is the most common bug in module-page reading; this table is the disambiguator. 

Capability| P0 · slice 2 stub (slice 1)| P3 full (slices 2–5)  
---|---|---  
**Login mechanism**|  Magic-link via email · valid 15 min · single-use| WebAuthn L3 passkeys (mandatory at T2+) · OAuth 2.1 + PKCE · OIDC SSO (Google / Microsoft 365 / Okta)  
**MFA**|  TOTP (RFC 6238) optional · enforced for tenant admins| TOTP + WebAuthn · passkey mandatory for T2+ subjects · impossible-travel challenge · HIBP password-breach check  
**RBAC catalogue**|  5 hard-coded roles: `root-admin`, `tenant-admin`, `tenant-member`, `service-account`, `agent-persona`| 22-role closed catalogue (see §RBAC) · Scope Contract Grants · agent-equal predicate evaluation  
**JWT signing**|  RS256 with dev keys on disk · 15 min access · 30 d refresh · rotating| RS256 via AWS KMS (key never touches disk) · JWKS rotation endpoint · per-tenant signing-key scope at slice 5  
**Tenant isolation**|  Postgres RLS on every identity table keyed by `tenant_id` · admin REST scoped to caller's tenant| Same RLS · plus per-tenant residency pinning (vn-hanoi-1 / sg-1 / eu-fra-1) · per-tenant authz server endpoint (slice 4)  
**Audit-chain emission**|  Every login, every RBAC decision, every session revocation appends a row to memory via the canonical Writer (subprocess shim acceptable)| Same emission · PyO3 binding for < 1ms p99 audit-bridge latency · streaming append at high decision rate  
**Admin surfaces**|  REST: tenant CRUD · subject CRUD · role grant · session revoke| Same REST · plus gRPC RBAC.Check internal API · MCP tool catalogue (token-issue, RBAC-check) · CLI `cyberos-auth` 25 subcommands  
**Cost**|  ~$60/month (RDS t4g.small + ElastiCache cache.t4g.micro + 1 Fargate task)| ~$220/month at 50 tenants (P3) · ~$1.2k/month at 500 tenants · KMS adds ~$10/month per active signing-key version  
**LoC**|  ~1,500 Rust| ~7,000 Rust (cumulative over slices 1–5)  
**Tests**|  15 (tenant + subject CRUD + RLS isolation)| 120+ (OAuth 2.1 conformance + WebAuthn flow + RBAC predicate matrix + audit-chain integration)  
**Lumi's memory integration**|  JWT carries `tenant_id` claim; Lumi's memory deployment (Stage 3) comes online in parallel| Tenant residency claim · agent-persona claim · Scope Contract Grant claim · all consumed by Lumi's memory read scoping  
**SOC 2 / ISO 27001 evidence**|  Not auditor-ready · stub mode is internal-only| P1 SOC 2 Type I evidence window opens at P0 · exit · P2 ISO 27001:2022 + Type II at P2 · exit · ISO 42001 AIMS at P3 · exit  
  
### Migration discipline

The transition from P0 · slice 2 stub to P3 full is not a rewrite — it is a layered addition. Each slice adds capability without changing the P0 · slice 2 contract. Pre-existing tenants/subjects/JWT claims continue to validate at every slice boundary. The conformance test suite runs both old and new code paths through P3 · exit; once Type I evidence is collected, the stub-only paths are retired. 

What the P0 · slice 2 stub does _not_ compromise on: (a) agent-equal evaluation (the agent-persona role is in the 5-role stub catalogue), (b) audit-chain emission (every decision is chain-linked from day one), (c) RLS isolation (the database scope is set correctly from slice 1, even if WebAuthn isn't there). What it _does_ defer: signing key in KMS, OIDC SSO, full 22-role catalogue, JWKS rotation, passkey enrolment, impossible-travel challenge, HIBP integration. 

2.6

## The 22-role RBAC catalogue (closed)

The closed RBAC catalogue is the same shape per the original AUTH spec, refactored slightly so the P0 · slice 2 stub catalogue is a strict subset. Adding a new role to the catalogue is an ADR — not a code change. The 5 stub roles are the first 5; the remaining 17 land across P0→P3. 

#| Role| Scope summary| Stub (P0 · slice 2)?| Lands at  
---|---|---|---|---  
1| `root-admin`| Cross-tenant superuser. Reserved for CyberSkill operators. Cannot be self-assigned.| ✅ yes| slice 1  
2| `tenant-admin`| Full admin within one tenant. Manage subjects, roles, billing, residency.| ✅ yes| slice 1  
3| `tenant-member`| Regular member. Read all shareable+, write to own personal scopes, MFA-required for sensitive ops.| ✅ yes| slice 1  
4| `service-account`| Non-human identity. Module-to-module mTLS clients. Token-exchange via slice-5 flow.| ✅ yes| slice 1  
5| `agent-persona`| Persona-versioned agent identity (CUO + 47 C-suite persona workflows). JWT stamped with persona-version.| ✅ yes| slice 1  
6| `founder`| Founder-CEO equivalent. WebAuthn passkey required. Cross-module privileged read.| —| slice 3 (WebAuthn)  
7| `cfo`| CFO seat. Read all financial · approve disbursements · ESOP grant signoff.| —| slice 4 (RBAC catalogue)  
8| `cto`| CTO seat. Tech-debt signoff · security advisory ack · OBS digest target.| —| slice 4  
9| `coo`| COO seat. Cross-module status digests · blocker triage · process changes.| —| slice 4  
10| `chro`| CHRO seat. HR records · onboarding · performance review · PII-read elevated.| —| slice 4  
11| `cmo`| CMO seat. Campaign briefs · content calendars · external comms approval.| —| slice 4  
12| `cpo`| CPO seat. Product-brief signoff · roadmap commits · feature-request-author canonical role.| —| slice 4  
13| `cso`| CSO (Strategy) seat. OKR cascade · scenario modelling · competitive intel read.| —| slice 4  
14| `cseco`| CSO (Security) seat. Security review · key rotation approval · vulnerability triage.| —| slice 4  
15| `clo`| CLO (Compliance/Legal) seat. Contract redline · DSAR triage · regulatory drift signoff.| —| slice 4  
16| `cdo`| CDO (Data) seat. Data quality · lineage · residency review · memory owner-of-record at P1+.| —| slice 4  
17| `dpo`| Data Protection Officer. DSAR fulfilment · breach-notification authority · purge approval.| —| slice 4  
18| `caio`| Chief AI Officer (emerging P2+). AI Gateway budget owner · synthesis sub-skill review.| —| slice 5  
19| `client-portal-user`| External-facing tenant user (PORTAL module · P4). Read shareable scopes via PORTAL filter only.| —| slice 5 + PORTAL  
20| `auditor`| External auditor (SOC 2 / ISO). Read-only · time-bounded · scope-pinned to evidence window.| —| slice 5  
21| `regulator`| External regulatory authority (MoPS, GDT, PDPC). Read-only · DSAR + breach-evidence scopes.| —| slice 5  
22| `billing-system`| Stripe / VietQR / Momo webhook identity. Write-restricted to billing event schema.| —| slice 5 + TEN  
  
**Adding a new role** requires an ADR (Architecture Decision Record) with: business rationale, scope-creep risk assessment, audit-trail implications, and explicit DPO + CSEC review. **Removing a role** requires a deprecation window of 90 days with shadow-monitoring of the role's actual usage. The 22-role boundary is a design assertion — ABAC complexity is deferred indefinitely. 

2.7

## AUTH ↔ Lumi's memory — tenant identity for the universal protocol

Per [MEMORY_AUTOSYNC_DESIGN.md §6](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>), Lumi's memory is the cloud-hosted org-tenant instance of the memory protocol. It depends on AUTH for two non-negotiable properties: **(1)** a tenant-scoped JWT issued by AUTH that identifies which tenant a sync request belongs to, **(2)** a subject-scoped JWT with the agent-persona claim that identifies which user within that tenant is reading or writing. Without both, multi-tenant isolation is impossible. 

### The JWT claim shape
    
    
    {
      "iss": "https://auth.cyberos.com",
      "sub": "user:stephen@cyberskill.world",
      "aud": ["memory.cyberos.com"],
      "iat": 1778765128,
      "exp": 1778766028,                          // 15 min
      "jti": "",
      // === Tenant identity ===
      "tenant_id": "org:cyberskill",
      "tenant_residency": "sg-1",                 // sg-1 | vn-hanoi-1 | eu-fra-1
      "tenant_plan": "pro",                       // free | pro | enterprise
      // === Subject scope ===
      "role": ["tenant-member", "cpo"],
      "rbac_grants": ["feature-request-author:execute", "kb:read", "memory:read:personal:*"],
      // === Persona stamp (when agent-issued) ===
      "agent_persona": null,                      // or "cuo-cpo@0.4.1" for agent JWTs
      "agent_persona_version": null,
      // === Scope Contract Grants (slice 4+) ===
      "scope_grants": [
        {"resource": "lumi:org:cyberskill:shareable", "rights": ["read"]},
        {"resource": "lumi:org:cyberskill:fr-decisions", "rights": ["read", "write"]}
      ]
    }

### How Lumi's memory enforces the JWT

sequenceDiagram autonumber participant U as User (sync client) participant A as AUTH (auth.cyberos.com) participant L as Lumi's memory (memory.cyberos.com) participant P as Postgres (RLS-scoped) participant W as Writer U->>A: POST /oauth/token (refresh) A->>A: verify refresh · rotate · sign new JWT A-->>U: {access_token, expires_in: 900} U->>L: POST /v1/sync/push with Bearer JWT L->>L: verify JWT signature against JWKS (cached) L->>L: extract tenant_id, role, residency alt residency mismatch L-->>U: 403 (wrong shard for this tenant) end L->>L: extract sync payload · validate sync_class L->>P: SET LOCAL app.current_tenant_id = '<tenant_id>' L->>W: write through canonical Writer (lumi-scoped) W->>P: INSERT with RLS · tenant_id auto-bound P-->>W: row inserted W-->>L: lumi_chain_hash L-->>U: 201 + lumi_chain_hash + sync_confirmation 

### What this contract requires from AUTH

  * **tenant_id is non-removable.** Every JWT — human, agent, service account — carries a tenant_id. Tenant-less JWTs are rejected at issuance.
  * **JWKS endpoint is reachable from Lumi's memory cloud region.** ~1 RTT cost on cold cache; cached for the JWT's lifetime.
  * **Refresh-token reuse detection is enforced at AUTH.** A reused refresh token revokes the whole session (per AUTH RFC §3 slice 2). Lumi's memory's pull queue must handle 401 by triggering re-auth, not by silently failing.
  * **Agent-persona claims survive the agent-equal property.** When the CUO router issues a JWT on a user's behalf, the `agent_persona` claim is set; Lumi's memory's read scope is then determined by the _intersection_ of user RBAC + persona Scope Contract Grant.
  * **Tenant residency pinning flows through.** A `vn-hanoi-1`-pinned tenant's JWT is rejected at the `sg-1` Lumi's memory shard. Cross-shard sync requires a special `cross-shard-migrate` claim (CDO + CLO approval).

2.8

## RFC open questions — proposed defaults

The [services/auth/RFC.md §6](<../../services/auth/RFC.md>) lists 5 open decisions that block slice 1. The proposed defaults below were drafted from the research review §2 + the MEMORY_AUTOSYNC_DESIGN.md Stage 3 dependencies. They land as ADRs once Stephen signs off. 

RFC §6 question| Proposed default| Rationale  
---|---|---  
**Q1 — Workspace membership**| **New repo-root Cargo workspace** with `skill/` and `services/auth/` as members. Lift the workspace root to `cyberos/Cargo.toml`.| Skill module already has its own workspace at `cyberos/modules/skill/Cargo.toml`. Lifting to repo-root lets `services/*` share dependency resolution + dev-dep with skill (e.g. `tokio`, `tracing`, `sqlx`). Common-dependency drift gets caught at build time.  
**Q2 — Memory bridge timing**| **Subprocess shim at slice 4 → PyO3 at slice 5.** Slice 4 (RBAC + audit-chain bridge) accepts `cyberos --store $memory put …` via stdin; slice 5 evaluates PyO3 binding for < 1ms p99.| Subprocess overhead acceptable at audit-chain write rate (≤ 100 writes/sec in P0). Mergeable in 1 week vs PyO3 binding's ~2 week effort. Defer the optimisation to when its absence is felt.  
**Q3 — Tenant-0 bootstrap**| **`cyberos-auth bootstrap` CLI subcommand** runs as root, seeds tenant 0 + the first `root-admin` subject with a one-time enrolment URL.| Avoids the chicken-and-egg "no admin to create the first admin" problem. Single-use URL with 1-hour expiry minimises bootstrap-credential risk window. CLI requires sudo + audit-emit.  
**Q4 — HIBP integration toggle**| **Enabled by default; per-tenant opt-out via`tenant.security_config.hibp_check = false`.**| Outbound HTTPS to `api.pwnedpasswords.com` is one extra dependency but k-anonymity-protected (only first 5 chars of password hash leave the host). For Vietnamese-residency-paranoid tenants (Decree 53), the opt-out exists. Default-on protects ≥ 95% of users at near-zero engineering cost.  
**Q5 — OBS deferral**| **Slice 1 emits structured tracing logs to stdout; slice 5 switches to OTLP once OBS lands.**|  OBS at P0 · start (parallel to memory) means OTLP is available by AUTH slice 5 at P3 · exit. Slice 1 stdout logs are sufficient for engineer-local development. Slice 5 single-config swap to OTLP doesn't change AUTH's design.  
  
**Once Stephen signs off,** each default becomes a numbered ADR (`services/auth/decisions/DEC-AUTH-001..005.md`) — same as the existing DEC-058 pattern. Slice 1 unblocks immediately after Q1 + Q3 are signed (workspace + bootstrap are mechanical); Q2 + Q4 + Q5 can land later without code rewrites. 

3

## Architecture

AUTH is one Rust service with five surfaces (OIDC issuer, OAuth 2.1 token endpoint, WebAuthn endpoints, RBAC gRPC API, admin REST), three stores (Postgres for identity / role / session, Redis for hot-path session and RBAC cache, KMS for signing-key wrap), and a single audit sink (memory). The diagram below shows the canonical request flow for a cross-module call. 

graph TB subgraph CLIENTS ["Clients"] BROWSER["Browser SPA  
(OIDC + PKCE)"] MOBILE["Mobile (P3)"] AGENT["CUO agent  
persona-stamped JWT"] SVC["Module service  
(mTLS · client_credentials)"] end subgraph EDGE ["Edge"] AR["Apollo Router  
verifies JWT @JwtAuth"] MCP["MCP Gateway  
verifies OAuth 2.1 PRM"] end subgraph AUTH ["AUTH service (Rust · axum)"] OIDC["OIDC issuer  
/.well-known/openid-configuration"] OAUTH["OAuth 2.1 token endpoint  
PKCE · audience-bound"] WA["WebAuthn / TOTP  
passkey + 2FA"] RBAC["RBAC engine  
predicate evaluator"] SCOPE["Scope Contract  
Grant resolver"] ADMIN["Admin REST  
role / tenant CRUD"] end subgraph STORES ["Stores"] PG[("PostgreSQL  
identity · roles · sessions  
RLS by tenant_id")] RED[("Redis 7  
session + RBAC cache  
TTL ≤ 15 min")] KMS[("AWS KMS  
RS256 signing key  
wrapped on disk")] end subgraph SINKS ["Audit & telemetry"] memory["🧠 memory  
auth.decision rows"] OBS["👁 OBS  
traces + metrics"] end BROWSER --> OIDC MOBILE --> OIDC OIDC --> OAUTH AGENT --> OAUTH SVC --> OAUTH OAUTH --> WA WA --> PG OAUTH --> PG OAUTH --> KMS AR --> RBAC MCP --> RBAC RBAC --> SCOPE SCOPE --> PG RBAC --> RED ADMIN --> PG OAUTH --> memory RBAC --> memory AUTH --> OBS classDef planned fill:#cba88a,stroke:#4338ca classDef store fill:#f5f3ff,stroke:#7c3aed classDef sink fill:#f5ede6,stroke:#45210e class OIDC,OAUTH,WA,RBAC,SCOPE,ADMIN,AR,MCP planned class PG,RED,KMS store class memory,OBS sink 

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`oidc.rs`| services/auth/src/oidc.rs| OIDC issuer — discovery doc, JWKS endpoint, ID-token issuance. Pure OpenID Connect Core 1.0 compliance.  
`oauth.rs`| services/auth/src/oauth.rs| OAuth 2.1 token endpoint — PKCE-required, audience-bound, refresh-token rotation, reuse detection.  
`webauthn.rs`| services/auth/src/webauthn.rs| WebAuthn L3 — credential creation, assertion verification, attestation parsing. Wraps webauthn-rs.  
`totp.rs`| services/auth/src/totp.rs| RFC 6238 TOTP — enrollment QR generation, code verification, replay window enforcement.  
`rbac.rs`| services/auth/src/rbac.rs| RBAC predicate evaluator. Input: (subject, action, resource). Output: allow / deny + audit record. Hot-path cache in Redis.  
`scope.rs`| services/auth/src/scope.rs| Scope Contract Grant resolver. Reads agent-persona scope sheets from memory; emits effective scope set per JWT.  
`session.rs`| services/auth/src/session.rs| Session manager — issue, validate, revoke. Device tracking + impossible-travel detection.  
`jwt.rs`| services/auth/src/jwt.rs| JWT issuance / verification. RS256 via KMS; key rotation; JWKS publication.  
`impossible_travel.rs`| services/auth/src/impossible_travel.rs| Geographic-velocity check on consecutive logins ((FR pending)); challenge if velocity > 1,000 km/h.  
`password.rs`| services/auth/src/password.rs| Argon2id hashing, password-policy enforcement, breach-list check via HIBP k-anonymity API.  
`device.rs`| services/auth/src/device.rs| Device fingerprinting ((FR pending)), new-device email, force-logout endpoint.  
`magic_link.rs`| services/auth/src/magic_link.rs| Magic-link onboarding flow ((FR pending)) — single-use, time-bound, onboarding-only.  
`audit_bridge.rs`| services/auth/src/audit_bridge.rs| Writes every decision to memory via the canonical writer. Includes `actor`, `action`, `resource`, `decision`, `reason`.  
`admin.rs`| services/auth/src/admin.rs| Admin REST — create tenant, assign role, revoke session, view audit. CSO + CEO scope only.  
`migrations/`| services/auth/migrations/| sqlx migrations. Every table has RLS by `tenant_id`. Indices for hot-path queries.  
  
4

## Data model

Identity, role, and session live in PostgreSQL with row-level security keyed by `tenant_id`. The schema is normalised but optimised for hot-path RBAC: the `subject_role` and `role_permission` tables are de-normalised into a materialised `subject_effective_permission` view that Redis caches with a 15-minute TTL. 

erDiagram TENANT ||--o{ SUBJECT: "owns" TENANT ||--o{ ROLE: "defines" SUBJECT ||--o{ SESSION: "creates" SUBJECT ||--o{ API_KEY: "owns" SUBJECT ||--o{ WEBAUTHN_CREDENTIAL: "registers" SUBJECT ||--o| TOTP_SECRET: "has" SUBJECT ||--o{ SUBJECT_ROLE: "holds" ROLE ||--o{ SUBJECT_ROLE: "granted" ROLE ||--o{ ROLE_PERMISSION: "carries" PERMISSION ||--o{ ROLE_PERMISSION: "grants" SUBJECT ||--o{ SCOPE_CONTRACT_GRANT: "has agent scope" SUBJECT ||--o{ AUTH_DECISION: "subject of" SUBJECT ||--o{ DEVICE: "trusts" SESSION }o--|| DEVICE: "issued from" SERVICE_ACCOUNT ||--|| SUBJECT: "is a" TENANT { uuid id PK string slug string display_name string country "VN or SG or other" string data_residency "vn-hanoi or sg-1" timestamp created_at } SUBJECT { uuid id PK uuid tenant_id FK string kind "human or agent or service or api_key" string email string display_name string status "active or locked or disabled" string password_hash "argon2id - null for agents" timestamp created_at timestamp last_login_at } ROLE { uuid id PK uuid tenant_id FK string code "CEO or CFO or CHRO or CTO or other" string display_name string scope_level "platform or tenant or project" } PERMISSION { string code PK "rew.write_run or memory.put or other" string action "read or write or delete or invoke" string resource_class } ROLE_PERMISSION { uuid role_id FK string permission_code FK } SUBJECT_ROLE { uuid subject_id FK uuid role_id FK timestamp granted_at timestamp expires_at uuid granted_by FK } SESSION { uuid id PK uuid subject_id FK string jti "JWT ID" string refresh_jti timestamp issued_at timestamp expires_at string ip_address string user_agent uuid device_id FK string status "active or revoked" } API_KEY { uuid id PK uuid subject_id FK string prefix "ck_live_…" string secret_hash "argon2id" string scopes "comma-separated" timestamp last_used_at timestamp expires_at } WEBAUTHN_CREDENTIAL { bytea credential_id PK uuid subject_id FK bytea public_key bigint sign_count string aaguid string transports timestamp created_at } TOTP_SECRET { uuid subject_id PK bytea secret_encrypted "KMS-wrapped" timestamp enrolled_at } DEVICE { uuid id PK uuid subject_id FK string fingerprint_sha256 string label timestamp first_seen_at timestamp last_seen_at bool trusted } SCOPE_CONTRACT_GRANT { uuid id PK uuid subject_id FK "agent persona" string persona_version "cuo-v2.3.1" string scope_set "comma-separated permission codes" timestamp valid_from timestamp valid_to } SERVICE_ACCOUNT { uuid subject_id PK string client_id bytea client_secret_hash string allowed_audiences } AUTH_DECISION { uuid id PK uuid tenant_id FK uuid subject_id FK string action string resource string decision "allow or deny or challenge" string reason_code timestamp ts string memory_chain "linked audit row chain hash" } 

### RBAC role catalogue ( — closed)

Role code| Display| Scope level| Examples of permissions  
---|---|---|---  
`founder`| Founder / CEO| platform| All; sign authority.  
`cfo`| Chief Financial Officer| tenant| `finance.*`, `rew.read_run` (not write).  
`chro`| Chief Human Resources Officer| tenant| `hr.*`, `rew.write_run` (with CFO co-sign).  
`cto`| Chief Technology Officer| tenant| `engineering.*`, `auth.admin`.  
`cso`| Chief Security Officer| tenant| `auth.*`, `obs.audit_read`, key-rotation.  
`clo`| Chief Legal Officer| tenant| `legal.*`, `memory.delete_purge_approve`.  
`dpo`| Data Protection Officer| tenant| `memory.dsar_export`, `auth.audit_read`.  
`cdo`| Chief Data Officer| tenant| `memory.admin`, `obs.read`.  
`cpo`| Chief Product Officer| tenant| `product.*`, `kb.write`.  
`cmo`| Chief Marketing Officer| tenant| `marketing.*`, `crm.read`.  
`cco`| Chief Customer Officer| tenant| `support.*`, `crm.write`.  
`member`| Operating Member| tenant| Scope-narrowed per module; default `chat.*` \+ `memory.read_own`.  
`contributor`| External Contributor| project| Project-scoped read + comment only.  
`client`| Client User| project| Per-project shared workspace; no cross-tenant access.  
`service`| Service Account| tenant| `client_credentials` only; explicit audience.  
`agent.cuo`| CUO Agent| tenant| Routing only; never destructive without human gate.  
`agent.skill`| Skill Agent| tenant| Per-skill scope; capability-broker gated.  
`agent.scheduled`| Scheduled Task| tenant| Single-purpose; cron-defined scope.  
\+ 4 more| (auditor, intern, vendor, partner)| tenant / project| See.  
  
5

## API surface

Four surfaces: an OIDC + OAuth 2.1 HTTPS edge for browsers and agents, a gRPC API for internal RBAC predicate evaluation, an MCP tool catalogue (the security-sensitive subset), and a small admin CLI surface. 

### GraphQL subgraph (federated)

AUTH publishes a thin federated subgraph for cross-module identity queries. Sensitive writes (role grant, session revoke) remain on the admin REST API.
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@external", "@shareable", "@requiresScopes"])
    
    type Subject @key(fields: "id") {
     id: ID!
     tenantId: ID!
     kind: SubjectKind!
     email: String
     displayName: String!
     status: SubjectStatus!
     roles: [Role!]! @requiresScopes(scopes: [["auth.read"]])
     effectivePermissions: [String!]! @requiresScopes(scopes: [["auth.read"]])
     lastLoginAt: DateTime
     createdAt: DateTime!
    }
    
    type Role @key(fields: "code tenantId") {
     code: String!
     tenantId: ID!
     displayName: String!
     scopeLevel: ScopeLevel!
    }
    
    type Session @key(fields: "id") @requiresScopes(scopes: [["auth.session_read"]]) {
     id: ID!
     subjectId: ID!
     issuedAt: DateTime!
     expiresAt: DateTime!
     status: SessionStatus!
     device: Device
    }
    
    type Device {
     id: ID!
     label: String!
     trusted: Boolean!
     lastSeenAt: DateTime!
    }
    
    type AuthDecision @key(fields: "id") @requiresScopes(scopes: [["auth.audit_read"]]) {
     id: ID!
     subjectId: ID!
     action: String!
     resource: String!
     decision: Decision!
     reasonCode: String!
     ts: DateTime!
     memoryChain: String!
    }
    
    enum SubjectKind { HUMAN AGENT SERVICE API_KEY }
    enum SubjectStatus { ACTIVE LOCKED DISABLED }
    enum ScopeLevel { PLATFORM TENANT PROJECT }
    enum SessionStatus { ACTIVE REVOKED EXPIRED }
    enum Decision { ALLOW DENY CHALLENGE }
    
    type Query {
     me: Subject!
     subject(id: ID!): Subject
     decisions(subjectId: ID, since: DateTime, limit: Int = 50): [AuthDecision!]!
     @requiresScopes(scopes: [["auth.audit_read"]])
    }
    
    type Mutation {
     revokeSession(id: ID!): Boolean!
     @requiresScopes(scopes: [["auth.session_revoke"]])
     grantRole(subjectId: ID!, roleCode: String!, expiresAt: DateTime): Boolean!
     @requiresScopes(scopes: [["auth.role_grant"]])
     rotateApiKey(id: ID!): ApiKeyRotation!
     @requiresScopes(scopes: [["auth.api_key_rotate"]])
    }

### REST + OAuth surface (planned)

Method| Path| Purpose  
---|---|---  
GET| `/.well-known/openid-configuration`| OIDC discovery document.  
GET| `/.well-known/oauth-authorization-server`| OAuth 2.1 metadata (RFC 8414).  
GET| `/.well-known/jwks.json`| JWKS for JWT verification.  
GET| `/oauth/authorize`| Authorisation endpoint (PKCE-required).  
POST| `/oauth/token`| Token endpoint — authorization_code · refresh_token · client_credentials.  
POST| `/oauth/revoke`| Token revocation (RFC 7009).  
POST| `/oauth/introspect`| Token introspection (RFC 7662) — internal mTLS only.  
POST| `/webauthn/register/begin`| Start passkey registration.  
POST| `/webauthn/register/finish`| Finish passkey registration.  
POST| `/webauthn/authenticate/begin`| Start passkey assertion.  
POST| `/webauthn/authenticate/finish`| Finish passkey assertion.  
POST| `/totp/enroll`| Begin TOTP enrolment (QR generation).  
POST| `/totp/verify`| Verify TOTP code.  
POST| `/magic-link/request`| Request onboarding magic link.  
GET| `/magic-link/consume?token=…`| Consume magic link.  
GET| `/admin/sessions`| List sessions (admin scope).  
POST| `/admin/sessions/{id}/revoke`| Revoke session.  
POST| `/admin/roles/grant`| Grant role.  
POST| `/admin/sso/saml/config`| Configure SAML IdP (P1).  
  
### gRPC RBAC API (internal)
    
    
    syntax = "proto3";
    package cyberos.auth.v1;
    
    service RBAC {
     / Hot-path predicate. Sub-millisecond Redis cache.
     rpc Check(CheckRequest) returns (CheckResponse);/ Batch variant for module gateways.
     rpc CheckBatch(CheckBatchRequest) returns (CheckBatchResponse);/ Resolve effective permissions for a subject.
     rpc EffectivePermissions(SubjectRef) returns (PermissionSet);
    }
    
    message CheckRequest {
     string subject_jwt = 1;/ verified upstream
     string action = 2;/ "rew.write_run"
     string resource = 3;/ "rew/run/2026-05"
     map<string, string> attrs = 4;/ optional ABAC hints (P3+)
    }
    
    message CheckResponse {
     bool allow = 1;
     string reason_code = 2;/ "policy.role.cfo_required" | "ok"
     string memory_chain = 3;/ audit row chain hash
    }

### MCP tool catalogue

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cyberos.auth.whoami`| —| Subject| readonly · scope=auth.read  
`cyberos.auth.check_permission`| action, resource| {allow, reason}| readonly  
`cyberos.auth.list_sessions`| subject_id| Session| readonly · scope=auth.session_read  
`cyberos.auth.revoke_session`| session_id| {ok}| destructive · scope=auth.session_revoke · human-confirm  
`cyberos.auth.grant_role`| subject_id, role_code, expires_at?| {ok, audit_row}| destructive · scope=auth.role_grant · human-confirm  
`cyberos.auth.list_decisions`| subject_id?, since?, limit| AuthDecision| readonly · scope=auth.audit_read  
`cyberos.auth.rotate_api_key`| api_key_id| {new_key, prefix}| destructive · scope=auth.api_key_rotate  
  
6

## Key flows

### Flow 1 — Human login with WebAuthn passkey

sequenceDiagram autonumber participant U as User (browser) participant SPA as CyberOS SPA participant A as AUTH /oauth/authorize participant W as AUTH /webauthn/* participant PG as PostgreSQL participant K as AWS KMS participant B as memory audit U->>SPA: open app SPA->>A: GET /oauth/authorize?client=cyberos-spa&PKCE;_S256=… A-->>SPA: redirect to login form SPA->>W: POST /webauthn/authenticate/begin {email} W->>PG: lookup credential by email PG-->>W: credential metadata W-->>SPA: assertion challenge SPA->>U: navigator.credentials.get(challenge) U->>SPA: signed assertion SPA->>W: POST /webauthn/authenticate/finish {assertion} W->>W: verify signature against stored public key W->>PG: update sign_count, last_seen_at W-->>A: subject_id authenticated A->>K: sign JWT (RS256, 15-min lifetime) K-->>A: signed access + refresh tokens A->>B: append auth.decision {subject, action:"login", decision:"allow"} A-->>SPA: 302 redirect with auth code SPA->>A: POST /oauth/token (code + PKCE verifier) A-->>SPA: {access_token, refresh_token} Note over SPA: subsequent calls carry Bearer JWT 

WebAuthn replaces the password leg entirely for elevated roles (CEO / CFO / CHRO / CSO / CLO). TOTP remains as fallback for member-tier accounts ((FR pending)).

### Flow 2 — TOTP MFA login (member tier)

sequenceDiagram autonumber participant U as User participant SPA as SPA participant A as AUTH /oauth/authorize participant P as AUTH /password participant T as AUTH /totp/verify participant K as KMS participant B as memory U->>SPA: open app SPA->>A: GET /oauth/authorize A-->>SPA: login form U->>SPA: email + password SPA->>P: verify password (argon2id) P-->>SPA: 200 — TOTP required SPA-->>U: show TOTP prompt U->>SPA: 6-digit code SPA->>T: POST /totp/verify {code} T->>T: window-check (±1 step, replay-list check) alt valid + not replayed T-->>A: subject authenticated A->>K: sign JWT K-->>A: tokens A->>B: auth.decision {decision:"allow"} A-->>SPA: tokens else replayed code T->>B: auth.decision {decision:"deny", reason:"totp.replay"} T-->>SPA: 401 end 

### Flow 3 — Agent authentication (CUO persona-stamped token)

sequenceDiagram autonumber participant CUO as CUO router participant A as AUTH /oauth/token participant SC as scope.rs (Scope Contract resolver) participant BR as 🧠 memory read participant K as KMS participant B as memory write Note over CUO: CUO needs to invoke a skill on behalf of stephen@… CUO->>A: POST /oauth/token  
grant_type=urn:cyberos:agent-impersonation  
actor_jwt=cuo-service-jwt  
on_behalf_of=stephen@…  
persona_version=cuo-v2.3.1 A->>SC: resolve scope for persona cuo-v2.3.1 SC->>BR: read meta/persona/cuo/v2.3.1/scope.md BR-->>SC: {scope_set, valid_to} SC-->>A: effective scope ⊂ user's scope ∩ persona scope A->>K: sign JWT with claims {sub:stephen, act:cuo-v2.3.1, scope:…} K-->>A: signed token A->>B: auth.decision {subject:cuo, action:"impersonate", on_behalf_of:stephen, allow} A-->>CUO: {access_token, ttl:15min} Note over CUO: every downstream call carries this persona-stamped JWT;  
memory audit rows capture both `actor` and `on_behalf_of`. 

The agent token's effective scope is the intersection of (a) what the user could do, (b) what the persona is granted, and (c) what AUTH's policy allows. Three-way narrowing means an agent can never exceed its user, nor its persona contract.

### Flow 4 — RBAC predicate on cross-module call

sequenceDiagram autonumber participant M as Module (e.g. REW) participant AR as Apollo Router participant R as AUTH RBAC.Check (gRPC) participant RC as Redis cache participant PG as PostgreSQL participant B as memory M->>AR: GraphQL mutation rewriteRun(…) AR->>AR: verify JWT signature + audience AR->>R: Check{subject_jwt, action:"rew.write_run", resource:"rew/run/2026-05"} R->>RC: GET subject_effective_permissions: alt cache hit RC-->>R: permission set else cache miss R->>PG: SELECT * FROM subject_effective_permission WHERE subject_id=… PG-->>R: rows R->>RC: SET subject_effective_permissions: TTL=15min end alt allowed R-->>AR: {allow:true, reason:"ok"} AR->>M: forward request R->>B: auth.decision (async) else denied R-->>AR: {allow:false, reason:"policy.role.cfo_required"} AR-->>M: 403 Forbidden R->>B: auth.decision {decision:"deny"} end 

Hot-path latency budget: ≤ 8 ms p95 with cache hit, ≤ 25 ms p95 with miss. Audit write is fire-and-forget over a bounded channel.

### Flow 5 — Service-account token-exchange (module-to-module)

sequenceDiagram autonumber participant SVC as CHAT service participant A as AUTH /oauth/token participant K as KMS participant B as memory Note over SVC: CHAT needs to call memory's put endpoint SVC->>A: POST /oauth/token  
grant_type=client_credentials  
client_id=chat-service  
client_secret=…  
scope=memory.put  
audience=memory.cyberos.internal A->>A: verify client secret (argon2id) A->>A: check audience ∈ allowed_audiences A->>K: sign JWT {sub:chat-service, aud:memory, scope:memory.put, exp:15m} K-->>A: token A->>B: auth.decision {subject:chat-service, action:"token_exchange"} A-->>SVC: {access_token, ttl:900} 

Service-to-service calls always use `client_credentials` with an _audience_ claim — receiving services reject tokens not aimed at them, eliminating confused-deputy attacks.

7

## Session lifecycle

A session traverses six states from issuance to expiry, with three terminal states (revoked, expired, replaced). Every transition writes an `auth.decision` row to memory. 

stateDiagram-v2 [*] --> Authenticating: user submits credentials Authenticating --> MFA_Required: password verified, 2FA outstanding Authenticating --> Locked: too many failures MFA_Required --> Active: TOTP / WebAuthn assertion verified MFA_Required --> Locked: too many 2FA failures Active --> Refreshing: access_token within 1 min of expiry Refreshing --> Active: rotated refresh + new access issued Refreshing --> Revoked: reuse detected (refresh replay) Active --> Revoked: admin revoke OR impossible-travel challenge Active --> Expired: refresh_token TTL reached (30 d) Active --> Replaced: user logs in from a new device, old session optionally evicted Revoked --> [*] Expired --> [*] Replaced --> [*] Locked --> Active: lockout cleared (15 min auto OR admin unlock) 

### Token lifetime budget

Token type| Lifetime| Rotation| Notes  
---|---|---|---  
Access token (JWT RS256)| 15 minutes| none (issue new)| (FR pending) — short-lived  
Refresh token| 30 days| every use| (FR pending) — replay invalidates session  
OAuth authorization code| 60 seconds| single-use| PKCE-bound  
Magic link| 15 minutes| single-use| (FR pending) — onboarding only  
Agent persona token| 15 minutes| each tool call may refresh| scope ⊂ user × persona × policy  
API key| configurable (default 90 d)| manual| argon2id-hashed; `ck_live_…` prefix  
WebAuthn credential| indefinite| user-initiated revoke| sign_count tracked to detect cloning  
  
8

## Functional Requirements

Wave 2 shipped five FRs (FR-AUTH-001 through FR-AUTH-005) on 2026-05-18. Eleven more land sequentially before P3 · exit. All AUTH FRs land via the [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) \+ `feature-request-audit` Agent Skill pair.

FR| Title| Status  
---|---|---  
`FR-AUTH-001`| Tenant + signing-key migrations (`0001_tenants.sql`, `0006_signing_keys.sql`)| shipped · 2026-05-18  
`FR-AUTH-002`| Subject + RLS migrations (`0003_subjects.sql`, `0004_rls_roles.sql`, `0005_rls_enable_on_tables.sql`) — USING + WITH CHECK on every tenant-scoped table| shipped · 2026-05-18  
`FR-AUTH-003`| Admin REST + idempotency layer (`POST /v1/admin/tenants`, `POST /v1/admin/subjects`, idempotency-key replay protection via `0002_admin_idempotency.sql`)| shipped · 2026-05-18  
`FR-AUTH-004`| JWT (RS256) + JWKS — `POST /v1/auth/token` accepts `grant_type=password` and `grant_type=refresh_token`; `GET /.well-known/jwks.json` publishes public key; auto-bootstrap RSA-2048 on AppState boot (`keygen.rs`) — and the JWT-verification middleware (`middleware.rs` · `verify_jwt` \+ `require_scope`) gating every admin endpoint| shipped · 2026-05-18  
`FR-AUTH-005`| Admin REST list + revoke + unrevoke — `GET /v1/admin/tenants`, `GET /v1/admin/subjects`, `POST /v1/admin/subjects/{id}/revoke`, `POST /v1/admin/subjects/{id}/unrevoke`| shipped · 2026-05-18  
`FR-AUTH-101`| 22-role RBAC catalogue (closed enum + permission matrix)| planned · next  
  
Additional FRs (WebAuthn enrolment, OIDC SSO, KMS rotation, impossible-travel, HIBP integration, Scope Contract Grants) land sequentially per `services/auth/RFC.md` slice plan.

9

## Non-Functional Requirements

security NFRs all flow through AUTH. Cross-referenced at [nfr-catalog.html#auth](<../../reference/nfr-catalog.html#auth>).

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Tenant data leakage incidents| = 0 (sev-0)| quarterly pen-test + automated cross-tenant test gate  
`N(FR pending)`| P1 base-salary system-reduction| = 0 — legal commitment| policy gate on REW + audit replay  
`N(FR pending)`| Agent auto-act on irreversible op without confirm| = 0 — runtime check| MCP gateway annotation check; CI test  
`N(FR pending)`| Prompt-injection exfiltration via email/document| = 0 — CaMeL enforced| CaMeL test suite + red-team  
`N(FR pending)`| TLS for all inter-service traffic| mTLS in cluster; HTTPS external| config inspection + Istio audit  
`N(FR pending)`| Penetration-test cadence| annual (P2+); after major releases| contract evidence  
`N(FR pending)`| Vulnerability remediation SLO| Critical ≤ 24 h; High ≤ 7 d| JIRA SLA tracking  
`N(FR pending)`| Sub-processor list public on Trust Center| always| web-page diff alert  
`N(FR pending)`| OWASP Gen AI Top-10 mitigations| all addressed| annual attestation  
`N(FR pending)`| RBAC predicate p95 (cache hit)| ≤ 8 ms| bench/rbac.rs · nightly  
`N(FR pending)`| RBAC predicate p95 (cache miss)| ≤ 25 ms| bench/rbac.rs · nightly  
`N(FR pending)`| OAuth token endpoint p95| ≤ 250 ms| k6 load test  
`N(FR pending)`| WebAuthn assertion verify p95| ≤ 50 ms| k6 load test  
`N(FR pending)`| AUTH availability (28-day)| ≥ 99.95%| SLO monitor (N(FR pending))  
`N(FR pending)`| Auth-decision durability| 0 dropped rows under crash| chaos test + memory ledger walk  
`N(FR pending)`| Refresh-token reuse detection rate| 100% (zero false negatives)| property-based test  
`N(FR pending)`| Session revoke propagation| ≤ 5 s end-to-end| (FR pending) test  
  
10

## Dependencies

AUTH depends on three primitives (memory for audit, OBS for telemetry, KMS for the signing key) and is depended on by every module that does anything cross-tenant or cross-module — which is to say, all of them. 

graph LR subgraph upstream ["AUTH depends on"] memory["🧠 memory  
auth.decision rows"] OBS["👁 OBS  
traces + alerts"] KMS["🔑 AWS KMS  
RS256 signing key"] PG["🗄 PostgreSQL  
identity store"] REDIS["⚡ Redis 7  
session + RBAC cache"] end AUTH["🔐 AUTH"] subgraph downstream ["Used by all 22 modules"] AR["Apollo Router  
JWT verify @ edge"] MCP["🔌 MCP Gateway  
tool-call gating"] AI["🧠 AI Gateway  
per-tenant routing"] CHAT["💬 CHAT"] REW["💎 REW"] HR["👥 HR"] CRM["🏢 CRM"] OTHERS["…16 more"] end memory --> AUTH OBS --> AUTH KMS --> AUTH PG --> AUTH REDIS --> AUTH AUTH --> AR AUTH --> MCP AUTH --> AI AUTH --> CHAT AUTH --> REW AUTH --> HR AUTH --> CRM AUTH --> OTHERS classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class memory shipped class AUTH,AR,MCP,AI,CHAT,REW,HR,CRM,OTHERS,OBS planned class KMS,PG,REDIS shipped 

11

## Compliance scope

AUTH is the regulator's first stop. The audit chain it produces — co-owned with memory — answers every identity, access, and breach-notification question across PDPL, GDPR, ISO 27001, and SOC 2.

Regulation / standard| Article / clause| AUTH feature that satisfies it  
---|---|---  
Vietnam PDPL (Law 91/2025)| Art. 4 — Lawful processing basis| Each subject row records consent basis; each auth.decision carries reason_code.  
Vietnam PDPL| Art. 14 — DSAR| `cyberos-auth dsar-export` bundles identity + decisions for a subject.  
Vietnam Decree 13/2023| Art. 17 — Personal data processing log| auth.decision rows are the processing log for identity events.  
Vietnam Decree 53/2022| Art. 26 — Data localisation; breach notification ≤ 24h| Per-tenant data-residency tag; auto-page MoPS template generator (planned P1).  
GDPR (EU 2016/679)| Art. 32 — Security of processing| WebAuthn L3 · TLS 1.3 · KMS-wrapped signing · row-level security · Argon2id.  
GDPR| Art. 33 — Breach notification| auth.decision chain provides forensic timeline; alert templates auto-fill.  
EU AI Act (Reg. 2024/1689)| Art. 14 — Human oversight| Destructive tool calls require human-confirm; agent JWTs cannot escalate.  
ISO/IEC 27001:2022| A.5.16 — Identity management| Closed role catalogue + scope contract grants.  
ISO/IEC 27001:2022| A.5.17 — Authentication information| Argon2id hashes; KMS-wrapped TOTP secrets.  
ISO/IEC 27001:2022| A.8.5 — Secure authentication| OAuth 2.1 + PKCE + MFA mandatory.  
SOC 2 Type II| CC6.1 — Logical access| RBAC predicate at every API boundary + audit chain.  
SOC 2 Type II| CC6.6 — Restricted access| RLS + scope contract grants narrow agents to a subset of users.  
OWASP Gen AI Top-10 (2025)| LLM02: Insecure output handling| MCP destructive-confirm gating routed through AUTH RBAC.  
OWASP Gen AI Top-10 (2025)| LLM08: Excessive agency| Three-way scope narrowing (user × persona × policy).  
NIST SP 800-63B| AAL2 / AAL3| TOTP for AAL2; WebAuthn (phishing-resistant) for AAL3.  
  
12

## Risk entries

AUTH-specific risks tracked in the [risk register](<../../reference/risk-register.html#auth>). AUTH carries the highest single-module risk weight; one bug here is platform-wide.

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-AUTH-001`| JWT signing-key compromise| Low| Catastrophic| CSO| KMS-wrapped at rest; rotation every 90 d; old JWKS retained 30 d for verification.  
`R-AUTH-002`| Refresh-token replay leading to permanent session hijack| Low| High| CSO| Rotation-on-use with reuse detection. Reuse → all sessions for that subject revoked.  
`R-AUTH-003`| Cross-tenant RLS bypass via subquery| Low| Catastrophic| CTO| RLS enabled on every table at migration time; CI test verifies cross-tenant read fails.  
`R-AUTH-004`| Agent JWT scope-escalation via prompt injection| Medium| High| CSO| Scope is policy-resolved at AUTH not prompt-derived; injection cannot widen.  
`R-AUTH-005`| WebAuthn relying-party-ID mismatch breaks logins after domain change| Low| Medium| CTO| RP-ID stored per credential; migration tooling re-enrols passkeys with new RP-ID.  
`R-AUTH-006`| HIBP outage during signup blocks new users| Medium| Low| CTO| HIBP check is best-effort; on timeout, log + allow with policy reviewer note.  
`R-AUTH-007`| TOTP secret leakage via DB backup| Low| High| CSO| Column-level KMS encryption; backups encrypted with separate keys.  
`R-AUTH-008`| OAuth implementation CVE (e.g., authorization-code injection)| Medium| High| CSO| OAuth 2.1-only (mandatory PKCE); annual pen-test; subscribe to RFC drafts watchlist.  
`R-AUTH-009`| Compensation co-sign bypass via direct DB write| Low| Catastrophic| CSO| (FR pending) enforced at REW boundary; direct DB write requires CSO+CFO co-sign; audit alarm.  
`R-AUTH-010`| Audit-chain write outage blinds the SOC| Low| High| CTO| Bounded local buffer + retry; alert on backlog > 60 s; AUTH refuses new logins after 5 min buffer.  
Reordering + stub + Lumi integration risks (per research review)  
`R-AUTH-011`| **P0 · slice 2 stub stays in production past P3** — the team relies on magic-link for too long, never adopts WebAuthn passkeys; SOC 2 evidence window opens with the stub still primary| High| Medium| CTO| SOC 2 readiness gate at P0 · exit explicitly requires WebAuthn enrolment for T2+ subjects. P1 exit blocked if > 50% of T2+ subjects haven't enrolled a passkey. Stub-only paths retire at P1 · exit per migration discipline.  
`R-AUTH-012`| **AI-Gateway-before-AUTH reorder regret** — early modules embed mock auth that becomes hard to replace at P0 · slice 2| Medium| Medium| CTO| Mock-auth at P0 · slice 1 (during AI Gateway slice 1) MUST use a single shared "mock-AUTH" library that lives in `services/auth/mock/` with the SAME API as the real AUTH. At P0 · slice 2 slice 1, the mock library imports the real AUTH via feature flag; mock-only paths retire at P0 · exit.  
`R-AUTH-013`| **Lumi's memory tenant-id claim spoofing** — attacker steals an AUTH-issued JWT, modifies tenant_id, presents to Lumi's memory| Low| Catastrophic| CSO| JWT is RS256-signed; modifying any claim invalidates signature. JWKS endpoint over HTTPS-only with certificate pinning. Lumi's memory re-verifies signature on every request (cached JWKS for JWT-lifetime ≤ 15 min). Audit-chain emission on every Lumi push includes the verifying key's `kid`.  
`R-AUTH-014`| **Cross-shard JWT replay** — a tenant pinned to `sg-1` has its JWT presented to the `eu-fra-1` Lumi's memory shard| Medium| High| CTO| JWT carries `tenant_residency` claim. Each Lumi's memory shard rejects JWTs whose `tenant_residency` doesn't match its own shard ID. Cross-shard migration requires explicit `cross-shard-migrate` claim (CDO + CLO co-signed, 1-hour TTL, one-time-use).  
`R-AUTH-015`| **Sub-process audit-bridge slow path** — slice-4 subprocess shim becomes a bottleneck at > 100 auth decisions/sec, audit-chain backlog grows| Medium| Medium| CTO| Slice 5 PyO3 binding target < 1ms p99. If slice 4 subprocess hits > 60 s backlog at sustained load, AUTH refuses new logins (per R-AUTH-010). Bench at every slice ship; alert if p95 of subprocess invocation > 50 ms.  
`R-AUTH-016`| **Tenant-0 bootstrap leak** — the one-time enrolment URL for tenant 0's first root-admin is shared, intercepted, or leaks via logs| Low| Catastrophic| CSO| One-time URL has 1-hour expiry and single-use semantics. Bootstrap CLI requires sudo + audit-emit. URL is printed once to stdout (never logged to file). All bootstrap actions audit-chained with the originating shell session ID.  
`R-AUTH-017`| **PDPL Art. 38 SME grace lapse** — tenant ages out of SME 5-year grace (post-2031-01-01) and AUTH hasn't gated the DPO-required workflows by then| High| Medium| CLO| AUTH tenant config has `pdpl_sme_grace_expires_at`. At 90 days before expiry, AUTH emits a "DPO required" advisory to the tenant-admin role. At expiry, AUTH refuses to issue tenant-admin login until a DPO subject is enrolled.  
  
13

## KPIs

AUTH health rolls up into 9 KPIs covering authentication success, authorisation correctness, performance, and compliance posture.

KPI| Formula| Source| Target  
---|---|---|---  
**Successful login rate**| `logins.success / logins.total`| auth.decision| ≥ 99.5% / day  
**MFA challenge success**| `mfa.success / mfa.total`| auth.decision| ≥ 98%  
**RBAC predicate p95 (cache hit)**|  Prometheus histogram| OBS| ≤ 8 ms  
**Token endpoint p95**|  histogram| OBS| ≤ 250 ms  
**Refresh-reuse detections**|  count / 28 d| auth.decision| tracked; expect < 5/month  
**Impossible-travel challenges**|  count / 28 d| auth.decision| tracked; alert on > 50/day  
**Auth-decision durability**| `rows_in_memory / rows_emitted`| chaos test| 100%  
**Session revoke p95 (end-to-end)**|  histogram| OBS| ≤ 5 s ((FR pending))  
**Penetration-test critical findings**|  count| annual report| = 0 critical · ≤ 2 high  
P0 · slice 2-stub-vs-P3-full + Lumi-integration KPIs  
**Stub-to-full migration coverage**| `subjects_with_webauthn / total_T2+_subjects`| auth subjects table| ≥ 50% by P1 · start · ≥ 95% by P1 · exit (P1 exit gate)  
**Mock-AUTH retirement**| `days_since_module_X_consumed_mock_auth`| module dependency graph + CI| 0 modules consuming mock-AUTH by P0 · exit (all migrated to real AUTH)  
**Lumi tenant-id verification success rate**| `verified_jwts / total_lumi_sync_requests`| Lumi's memory access logs| ≥ 99.99% — non-verification means JWKS outage  
**Cross-shard rejection rate**| `rejected_cross_shard / total`| Lumi's memory edge proxy| 0% under steady state; spike means tenant residency misconfigured  
**Audit-bridge p99 latency**|  subprocess shim or PyO3| OBS| slice 4: ≤ 50 ms · slice 5: ≤ 1 ms (PyO3 target)  
**SME-grace lapsed tenants**|  count| auth tenant config| 0 lapsed without DPO enrolled (R-AUTH-017 mitigation)  
**22-role catalogue stability**| `ADR-validated role changes / total schema changes`| git log + ADRs| 100% — every role addition is an ADR  
  
14

## RACI matrix

AUTH is owned by the CSO seat. Today (CSO vacant), the CEO is interim accountable with the CTO as engineering owner.

Activity| CEO| CTO| CSO| CDO| CLO| DPO  
---|---|---|---|---|---|---  
Service design + spec| A| C| R| I| C| I  
Implementation| I| A| R| I| I| I  
On-call rotation| I| R| A| I| I| I  
Penetration testing| C| C| A/R| I| I| I  
Role catalogue changes| A| I| R| I| C| I  
Key rotation (JWT signing)| I| C| A/R| I| I| I  
Incident response (auth breach)| A| R| R| C| C| R  
Decree 53 MoPS notification| C| I| R| I| A| R  
DSAR fulfilment (auth scope)| I| I| C| R| C| A  
  
**R** Responsible · **A** Accountable · **C** Consulted · **I** Informed.

15

## Planned CLI surface

A single admin CLI `cyberos-auth` for tenant operators. Every destructive command writes a chained audit row before exit.

### 1\. Create a tenant
    
    
    $ cyberos-auth tenant create \
     --slug stephen-personal \
     --display "Stephen Personal" \
     --country VN \
     --residency vn-hanoi-1
    
    [tenant created]
     id: 01HZJ8R4M2K7QXP3F9D8YN7B2T
     slug: stephen-personal
     residency: vn-hanoi-1
     audit: memory seq=14823 chain=9f3e2a1b…8d4c

### 2\. Enrol a passkey (interactive)
    
    
    $ cyberos-auth webauthn enrol --subject stephen@cyberskill.com
    
    → open https://auth.cyberos.com/webauthn/enrol?token=eyJhbGc… on the device
    ✓ credential registered
     credential_id: AaJ9K… (TouchID, MacBook Pro 16, 2023)
     aaguid: adce0002-35bc-c60a-648b-0b25f1f05503

### 3\. Grant a role
    
    
    $ cyberos-auth role grant \
     --subject acme-cfo@acme.com \
     --role cfo \
     --expires 2027-12-31T00:00:00Z
    
    [role granted] subject=acme-cfo@acme.com role=cfo expires=2027-12-31
    [audit] memory seq=14831 chain=b8d4…e3f7

### 4\. Revoke a session
    
    
    $ cyberos-auth session revoke --id 01HZJ8…JTC
    
    [revoke] session 01HZJ8…JTC → status=revoked
    [propagated] Redis cache invalidated; Apollo Router will reject within 5 s
    [audit] memory seq=14832 chain=c9e5…f4a8

### 5\. Read auth decisions for a subject (DSAR-style)
    
    
    $ cyberos-auth decisions list --subject stephen@cyberskill.com --since 7d --format jsonl | head -3
    {"ts":"2026-05-13T09:21:08Z","action":"login","decision":"allow","reason":"webauthn.aal3","device":"MacBook Pro 16"}
    {"ts":"2026-05-13T09:21:09Z","action":"memory.put","decision":"allow","reason":"role.founder","resource":"memories/decisions/holdco-flip.md"}
    {"ts":"2026-05-13T11:04:32Z","action":"rew.write_run","decision":"deny","reason":"policy.cfo_co_sign_required","resource":"rew/run/2026-05"}

### 6\. Rotate the JWT signing key
    
    
    $ cyberos-auth keys rotate --reason "quarterly-scheduled"
    
    [rotate] new key id: 2026-q2-sig
    [kms] old key id 2026-q1-sig retained for verification (30 d)
    [jwks] /.well-known/jwks.json updated; downstream propagation: ≤ 60 s
    [audit] memory seq=14841 chain=d1f3…a8c7

### 7\. Export DSAR bundle
    
    
    $ cyberos-auth dsar-export --subject acme-contact@acme.com --output dsar.zip
    
    [dsar] subject: acme-contact@acme.com
    [dsar] identity: 1 row
    [dsar] decisions: 4,217 rows (28 d)
    [dsar] sessions: 12 rows
    [dsar] devices: 3 rows
    [dsar] webauthn: 0 credentials (none registered)
    [dsar] written: dsar.zip (412 KB)

16

## Phase status & estimates

Status

Planned

P0 design phase · P0 · slice 1 start

Est. LoC (Rust)

~7,000

services/auth + sqlx migrations

Planned tests

120+

unit · integration · OAuth conformance

External libs

~12

axum · sqlx · webauthn-rs · totp-rs · rsa

CLI subcommands

~25 planned

`cyberos-auth` entrypoint

P0 budget

~$60/mo

RDS + Redis + Fargate

Capability| Status  
---|---  
OAuth 2.1 + PKCE token endpoint| planned · P0  
WebAuthn L3 (passkeys)| planned · P0  
TOTP MFA| planned · P0  
RBAC predicate gRPC API| planned · P0  
JWT RS256 with KMS signing| planned · P0  
Refresh-token rotation + reuse detection| planned · P0  
Service-account client_credentials| planned · P0  
Magic-link onboarding| planned · P0  
Audit-chain integration (auth.decision)| planned · P0  
Agent token-exchange (persona-stamped)| planned · P0  
OIDC SSO (Google · M365 · Okta)| planned · P1  
Impossible-travel detection| planned · P1  
Device tracking + new-device email| planned · P1  
Decree 53 MoPS-notification automation| planned · P1  
Multi-region active-active| planned · P3+  
SAML 2.0 enterprise SSO| planned · P2+  
FIDO2 hardware-key attestation| planned · P2+  
  
17

## References

  * **services/auth/RFC.md** (drafted 2026-05-14) — [implementation RFC](<../../services/auth/RFC.md>): 5-slice ship plan · 5 open questions · audit-chain bridge design · DoD. The source-of-truth for everything P0 · slice 2 → P3.
  * **services/auth/mockups/sign-in.html** — [first AUTH UI mockup](<../../services/auth/mockups/sign-in.html>) — Liquid Glass + Umber/Ochre · Be Vietnam Pro · passkey-first flow · MFA chips · memory audit-chain trust footnote. Implements design system Part 21.
  * **MEMORY_AUTOSYNC_DESIGN.md** §6 — [Lumi's memory spec](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>). AUTH issues the tenant-scoped JWT that Lumi consumes for row-level isolation. AUTH is gated by AI Gateway + TEN per the reordered P0 sequence.
  * **archive/2026-05-14/RESEARCH_REVIEW.md** §2.4 — `archive/2026-05-14/RESEARCH_REVIEW.md` (archived; see `cyberos/CHANGELOG.md`). The "AUTH not P0 #1" reorder rationale. Section 2.4 verbatim: "AUTH at P0 · slice 2 can be magic-link + TOTP with a stub RBAC. WebAuthn passkeys for the Founder and a per-tenant authz server can land at P0 · exit without blocking any other module."
  * **archive/2026-05-14/AUDIT_AND_PLAN.md** — `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`). AUTH's 5-slice progression and slice ordering relative to AI Gateway / MCP Gateway / OBS / CHAT.
  * **AUTHORING_DISCIPLINE.md** — [FR authoring playbook (feature-request-author + audit pair)](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>). AUTH FRs land here; FR-AUTH-001 through FR-AUTH-005 all ship via this discipline.
  * **AGENTS.md** §3.6 + §11 — [memory protocol](<../../memory/docs/AGENTS.md>). `allowed_memory_scopes` \+ trust model · the basis for AUTH's audit-chain bridge contract.
  * **Decree 53/2022/NĐ-CP (Vietnam)** — Cybersecurity Law implementing decree; data-residency + breach notification.
  * **Decree 13/2023/NĐ-CP (Vietnam)** — Personal data processing protection.
  * **Law 91/2025/QH15 (Vietnam PDPL)** — Personal Data Protection Law.
  * **RFC 6749 / draft-ietf-oauth-v2-1** — OAuth 2.1 specification.
  * **RFC 7636** — Proof Key for Code Exchange (PKCE).
  * **RFC 6238** — Time-Based One-Time Password.
  * **RFC 7519** — JSON Web Token (JWT).
  * **RFC 8414** — OAuth Authorization Server Metadata.
  * **WebAuthn Level 3** — W3C Recommendation 2023.
  * **NIST SP 800-63B** — Digital Identity Authenticator Assurance Levels.
  * **Architecture context:** [infrastructure.html#auth](<../../architecture/infrastructure.html#auth>).



★

## Personas & skill bundles that touch AUTH

AUTH issues 47 distinct `agent-persona` JWT shapes (one per active CUO persona, plus the human roles in the 22-role RBAC catalogue). The personas below govern AUTH's policy itself, while every other persona consumes the JWTs AUTH issues.

Persona governance (5 of 47)

  * [chief-security-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-security-officer/workflows>) · converged-security-strategy (AUTH policy owner)
  * [chief-information-security-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-information-security-officer/workflows>) · monthly-vuln-management + SOC2 evidence
  * [chief-privacy-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-privacy-officer/workflows>) · privacy-impact-assessment + data-subject-request
  * [chief-information-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-information-officer/workflows>) · IT-vendor-scorecard (SSO providers + KMS)
  * [chief-trust-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-trust-officer/workflows>) · trust-portal updates referencing AUTH's RBAC posture



Skill bundles AUTH gates

  * [soc2-evidence-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/soc2-evidence-author>) \+ audit · evidence harvest for AUTH controls (CC6, CC7)
  * [security-strategy-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/security-strategy-author>) \+ audit · annual posture + AUTH roadmap
  * [pen-test-report-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/pen-test-report-author>) \+ audit · per-engagement test of AUTH endpoints
  * [breach-notification-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/breach-notification-author>) \+ audit · per-credential-compromise 72h pack
  * [vietnam-vneid-integration](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-vneid-integration>) · VN high-assurance identity assertion



Every other CUO persona (42 of 47) consumes AUTH JWTs to read scope-limited memory data per the Scope Contract Grant mechanism. See § RBAC catalogue above.

[← All modules](<../index.html#catalog>) [Next module: AI Gateway →](<../ai/index.html>)
