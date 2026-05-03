---
title: "Federation gateway, host shell, and multi-tenant Postgres scaffold (P0 foundation)"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P0 / 2026-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the foundational platform layer for CyberOS in the first two-week sprint (S0-1) so every subsequent module has a predictable place to land. This feature delivers (a) an Apollo Federation v2 supergraph composed by an Apollo Router, (b) a Vite + React 19 host shell that lazy-loads Module-Federation remotes, (c) a PostgreSQL 17 cluster with schema-per-tenant scaffolding and Row-Level Security (RLS) enabled by default on every tenant-scoped table, (d) NATS JetStream for the event substrate, and (e) the CI/CD pipeline (GitHub Actions → ArgoCD → Hetzner-hosted Kubernetes) that every module will use to ship. After this feature ships, a trivial `HEALTH` subgraph plus a `HEALTH` Module-Federation remote run end-to-end, two synthetic tenants exist with verifiably non-overlapping data, and the CI pipeline completes in under ten minutes.

## Problem

The PRD is a 22-module commitment. Without a deterministic, opinionated foundation in week one, every subsequent module owner re-invents auth-plumbing, schema isolation, deployment, and event-publishing — and the tenant-isolation guarantee that the entire platform's compliance story rests on becomes a per-module liability rather than a system invariant. Today CyberSkill's two long-term engagements run on a fragmented stack (Notion, Slack/Zalo, Asana, HubSpot, Gmail, Excel-payroll, paper contracts) with no cross-tool tenant boundary and no event spine. The PRD's strategic bets — agent parity (MCP), CUO as brand, BRAIN as substrate — all assume a federation-shaped platform exists. The cost of postponing this work to "after the first feature ships" is exactly the cost we paid five years ago: every later decision rebuilds the foundation under duress.

The S0-1 sprint exit criterion is therefore a non-negotiable architectural prerequisite, not a "nice to have". The PRD §17.1 lists the exact demo deliverables — host shell loads with tenant context, two synthetic tenants exist, cross-tenant invisibility is verifiable, CI ≤ 10 minutes, ArgoCD deploys to a prod-shaped staging — and explicitly marks any cross-tenant leakage observed during the synthetic test as **sprint-blocking**.

## Proposed Solution

The shape of the answer is a five-part platform layer, each part owning one concern.

**1. Apollo Federation v2 supergraph.** A single Apollo Router process (running in the production cluster as a Deployment with three replicas behind a Cloudflare-fronted Service) composes subgraphs published by individual modules. The router enforces persisted queries — every production query hash is pre-registered at deploy time; any unregistered query returns HTTP 400 with `code: PERSISTED_QUERY_NOT_REGISTERED`. The router rejects introspection in production. Subgraphs publish via the Apollo Studio schema registry (or a self-hosted schema-registry service when Studio is dropped at SRS DEC-053+); composition checks run in CI on every subgraph PR and a failed composition blocks merge. Federation directives in scope for v2.5+: `@key`, `@external`, `@requires`, `@provides`, `@shareable`, `@inaccessible`, `@override`. Schema changes follow a deprecation-first protocol: new fields must be additive; deprecated fields are tagged `@deprecated(reason: "...")` and live for at least one phase before removal. The supergraph URL is `https://api.cyberos.world/graphql` for the canonical CyberSkill tenant and `https://{tenant-slug}.cyberos.world/graphql` for every additional tenant created in P3+.

**2. Module-Federation host shell.** The host shell is a thin Vite + React 19 app served from a Cloudflare-cached static origin. The shell knows three things and nothing more: (i) the current authenticated identity (resolved from the AUTH module's session cookie), (ii) the current tenant context (derived from subdomain), and (iii) a typed registry of Module-Federation remote URLs. On route entry the shell lazy-loads the relevant remote via Webpack 5 Module-Federation runtime, mounts the remote's exported React component, and provides three host-shell-only services to remotes: a `<DesignTokens>` provider sourced from `@cyberskill/tokens`, a `<GeniePanel>` registry slot that any remote can render cards into, and a `<Tenant>` context. CSS is scoped exclusively through CSS Modules; no global selectors except design-token CSS variables. Initial JS bundle for the shell ≤ 80 KB gzipped; each remote ≤ 50 KB initial JS bundle (PRD §7.2 "Module Ready" requirement). The shell registers persisted queries automatically during the `pnpm build` step and emits the registration manifest to the Apollo Router CI pipeline.

**3. PostgreSQL 17 cluster with schema-per-tenant + RLS.** The cluster runs Postgres 17 with the following extensions enabled at provisioning: `pgvector` (BRAIN Layer 2 vector index), `apache-age` (BRAIN Layer 2 graph), `pgroonga` (Vietnamese full-text search), `pg_jsonschema` (column-level JSON validation), `pgcrypto` (KMS-wrapped encryption helpers). Schema isolation: each tenant gets a dedicated Postgres schema named `tenant_{slug}` (slug is `^[a-z][a-z0-9_-]{1,30}$`); tables are created in that schema. Tenant-shared metadata (the `cyberos_meta` schema) holds the tenant registry, the parameter-version log, the audit log root index, and the federation schema registry's snapshot rows. Row-Level Security: **every table** in a `tenant_*` schema has `ROW LEVEL SECURITY ENABLED` plus a `FORCE ROW LEVEL SECURITY` clause; the canonical RLS policy is `USING (tenant_id = current_setting('app.tenant_id', true)::uuid)` and the application sets `app.tenant_id` via `SET LOCAL` on every transaction's first statement. Connection pooling uses PgBouncer in `transaction` mode in front of the cluster; the application enforces `SET LOCAL` per transaction, never per session, so a borrowed-from-pool connection cannot leak the previous tenant's setting. Storage: separate volume per tenant for the largest tenants (T3 enterprise plan from P3); shared volume for T1/T2. Backups: 15-minute WAL archiving to S3-compatible object storage (Cloudflare R2 or AWS S3 in the tenant's residency region), 35-day point-in-time recovery, weekly logical-dump verification.

**4. NATS JetStream event substrate.** A three-node JetStream cluster runs in the production cluster. Subject naming follows PRD §8.10: `cyberos.{tenant}.{module}.{entity}.{verb}` — for example, `cyberos.cyberskill.proj.task.created`. Subjects are tenant-scoped at the subject level so a subscriber that mis-uses a wildcard cannot accidentally cross tenant boundaries (the gateway-level enforcement is belt-and-braces). Streams retain 30 days by default, 90 days for `*.rew.*` and `*.esop.*`. Durable consumers are the only consumer kind allowed in production code; ephemeral consumers are restricted to debugging. Per-tenant credentials are NATS NKey-derived and rotated quarterly through a sealed-secrets workflow.

**5. CI/CD pipeline.** GitHub Actions runs the canonical pipeline: lint → typecheck → unit tests → contract tests (Pact-style for subgraph consumers) → integration tests (against an ephemeral Postgres + NATS pair via `testcontainers`) → schema-registry composition check → bundle-size check → security scan (Trivy on container images, `pnpm audit` for npm) → image build → image push to GitHub Container Registry. ArgoCD watches the `infra/` directory in the platform monorepo and rolls out changes to the Hetzner-hosted Kubernetes cluster (HCloud k3s + Cilium CNI; SRS DEC-058+ for Hetzner choice). Total CI duration ≤ 10 minutes for a single-module PR (PRD §7.2). Cluster: three control-plane nodes, six worker nodes initially, autoscaler enabled but capped at 12 worker nodes during P0 to keep the monthly bill under $380 (PRD §4.1 G7 cost target).

The first end-to-end exercise is the trivial `HEALTH` subgraph — a single GraphQL field `health: HealthStatus!` that returns `{ status: OK, tenantId, gitSha, schemaVersion }`. The `HEALTH` Module-Federation remote renders a one-card status panel in the host shell showing the same payload plus the resolved auth identity. The two-tenant test creates `tenant_cyberskill` (the canonical tenant) and `tenant_acmecorp` (a synthetic future-customer tenant), seeds three rows into a `health_log` table in each tenant's schema, and asserts via a synthetic cross-tenant request (a query for `tenant_acmecorp.health_log` while the request is authenticated for `tenant_cyberskill`) that exactly zero rows are returned and an audit-log entry is written marking the attempted access.

## Alternatives Considered

- **Single GraphQL service (no federation).** Rejected: the PRD commits to module-by-module ownership (§7.2) and to module-federation frontend (§8.3); a single service forces a coordinated release cadence we do not have the headcount to manage. The transition cost from a single service to federation later in P1 would consume an entire phase.
- **REST + OpenAPI per module.** Rejected: the agent-operability moat (Bet 1, §2.3) requires a typed schema that an MCP tool can introspect; OpenAPI plus a hand-rolled type generator was the prior plan but pushes complexity onto each module owner. GraphQL Federation gives us strong typing, a composition checker, and persisted queries in one stack.
- **Database-per-tenant (no schema-per-tenant).** Rejected for P0: per-database isolation is the upgrade for T3 enterprise tenants in P3 (PRD §8.8) and forces an N-times-larger Postgres footprint for the internal-only P0 use case. Schema-per-tenant with `FORCE ROW LEVEL SECURITY` and per-transaction `SET LOCAL` gives the same isolation guarantee at one-tenant-cluster cost. The provisioning code emits per-tenant boundary metadata so a P3 migration to per-database tenants is a parameter flip, not a refactor.
- **Kafka instead of NATS.** Rejected: Kafka's operational footprint (ZooKeeper or KRaft, JVM heap tuning) is the wrong shape for a 10-engineer team; NATS JetStream's single-binary footprint, built-in subject hierarchy, and 1-RTT pub/sub fit the cadence we need. Kafka stays on the radar for P3+ if external-tenant scale demands it.
- **Vercel / Render / Railway managed hosting.** Rejected: all three impose region constraints that cannot satisfy the Vietnamese-residency requirement (PRD §8.8). Hetzner gives us EU and US footprints today and a partner Vietnamese DC for the vn-shard at P3.

## Success Metrics

- **Primary metric.** End-to-end S0-1 demo passes: (1) host shell loads at `https://app.cyberos.world` and resolves the founder's identity in ≤ 1.5 s p95, (2) the `HEALTH` subgraph returns a 200 with `tenantId` matching the subdomain, (3) the cross-tenant synthetic test reports `leakedRows: 0`, (4) CI pipeline reports total duration ≤ 10 minutes on the median PR over a 14-day rolling window, (5) ArgoCD rollout from merge-to-`main` to `prod-staging` ≤ 3 minutes p95.
- **Guardrail metric.** Cross-tenant data leakage incidents = 0 for the lifetime of P0 (PRD §4.3 anti-metric). A single confirmed incident is sev-0, triggers phase rollback, and re-opens this FR.

## Scope

**In-scope (P0 / S0-1).**
- Apollo Router deployed with three replicas, persisted-queries enforced, introspection disabled in prod.
- One canonical subgraph (`HEALTH`) + the schema-registry composition pipeline.
- Vite + React 19 host shell with Module-Federation runtime, design-token provider, Genie-panel slot registry, and tenant context.
- One Module-Federation remote (`HEALTH`) consumed by the shell.
- Postgres 17 cluster with `pgvector`, `apache-age`, `pgroonga`, `pg_jsonschema`, `pgcrypto` extensions; schema-per-tenant scaffold; `FORCE ROW LEVEL SECURITY` policy on every tenant-scoped table; per-transaction `SET LOCAL app.tenant_id`; PgBouncer in `transaction` mode.
- NATS JetStream three-node cluster with subject naming convention enforced; one durable consumer wired to the audit log writer (which is otherwise stubbed in S0-1, completed in S0-2 by FR-AUTH-002).
- GitHub Actions CI with the seven canonical stages and the ≤ 10-minute duration budget.
- ArgoCD deployed and watching `infra/`; one staging rollout demonstrated.
- Two synthetic tenants seeded; cross-tenant invisibility verified via an automated test suite that runs in CI on every PR.
- Trust Center status page placeholder at `status.cyberos.world` (real values populated in S0-6 by FR-OBS-001).

**Out-of-scope (deferred).**
- Real auth (covered by FR-AUTH-001 in S0-2; S0-1 uses a hardcoded JWT signed by a dev key).
- AI Gateway (FR-AI-001, S0-2).
- BRAIN module (FR-BRAIN-001 + FR-BRAIN-002, S0-3).
- All other functional modules.
- Multi-region routing (single `eu-central` region in P0 for cost; tenant-residency routing is added in P3 by a later FR).
- Per-tenant DB isolation (deferred to P3, see Alternatives §3).
- Auto-scaling policies beyond a hard cap of 12 worker nodes.

## Dependencies

- Hetzner account provisioned with billing approved (cost ceiling $380/month at P0 internal scale; PRD §4.1 G7).
- Cloudflare zone for `cyberos.world` provisioned and DNSSEC enabled.
- GitHub organisation `cyberskill` provisioned, monorepo `cyberos-platform` created, GitHub Container Registry namespace registered.
- Apollo Studio account or self-hosted schema registry decided per SRS Decisions Log (DEC-053+ if Studio is dropped); a selection must be locked before this FR can ship.
- Postgres 17 image published with the five required extensions (we maintain a small Dockerfile in `infra/postgres/`).
- Compliance: the Hetzner DPA is signed; an A05 filing template is prepared by the DPO for the eu-central region (PRD §12.1.1).
- Locked decisions referenced: DEC-001 (multi-tenant from day one), DEC-005 (Apollo Federation v2), DEC-006 (Module Federation), DEC-013 (Postgres schema-per-tenant + RLS), DEC-018 (NATS JetStream), DEC-040 (Hetzner). FR-INFRA-001 inherits these without modification.

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. This feature ships zero AI-derived behaviour. The first AI surface is FR-AI-001 (S0-2) and is risk-classified there.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
