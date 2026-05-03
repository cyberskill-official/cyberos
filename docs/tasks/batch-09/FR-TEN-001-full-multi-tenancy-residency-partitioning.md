---
title: "TEN — full multi-tenancy with residency partitioning (vn/sg/eu/us shards) + per-tenant CUO persona isolation"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Promote CyberOS from single-tenant (CyberSkill alone, P0–P2) to **full multi-tenant with residency partitioning** per PRD §8.8 and §14.4.1. Four residency shards: **vn-shard** (Vietnam-resident tenants on AWS Singapore + Vietnam-edge CDN; PDPL Decree 13 home regime), **sg-shard** (Singapore-resident tenants on AWS Singapore), **eu-shard** (EU-resident tenants on AWS Frankfurt; GDPR posture from FR-CP-004), **us-shard** (US-resident tenants on AWS Ohio; SOC 2 + future state-privacy laws). Each shard is a **separate Apollo Federation supergraph + separate Postgres cluster + separate NATS JetStream + separate KMS key root**; cross-shard writes are **forbidden by architecture** (the federation gateway rejects cross-shard cross-references); cross-shard reads are forbidden except for the founder-tier shared services (the platform's own marketing site, the Trust Center, the Public APIs in P4). **Per-tenant CUO persona isolation** — each tenant has its own `cyberos_meta.persona_version_active` and its own copy of the Skills directory; one tenant's persona-pause does not affect others. Tenant residency is **immutable at provisioning** (changing residency requires a paid migration). The schema-per-tenant + RLS-as-floor pattern from FR-INFRA-001 carries forward; this FR adds the cluster-level partitioning + the cross-shard-rejection enforcement.

## Problem

Through P0–P2 the platform runs as single-tenant for CyberSkill alone (`tenant_id` is plumbed but only one tenant exists). PRD §14.1.2 explicitly notes: "Multi-tenancy — single-tenant only; tenant_id is plumbed but only one tenant exists." Going from this to full multi-tenancy at P3 → P4 (where the first external paying client is the gate criterion) involves three structural changes the platform must make architecturally, not patches:

- **Residency hard partitioning.** A Vietnamese tenant's data must never traverse an AWS US region. PRD §8.8: "Cross-region writes are forbidden." Cluster-level separation is the floor; in-cluster RLS is necessary but not sufficient because a single Postgres cluster cannot satisfy "data never leaves region X" if the cluster is in region Y.
- **Per-tenant CUO persona isolation.** Tenant A's founder publishes a v0.5 persona; Tenant B's founder publishes v0.6 with different scope contracts. The two tenants must run independently — one tenant's auto-pause cannot affect the other; one tenant's prompt-injection cannot escape into another's data.
- **Cross-shard reference forbidden.** Every cross-module reference inside CyberOS today (`crm.account_id` → `proj.engagement.client_account_id` → `okr.linked_artefact.artefact_id`) must be tenant-scoped; the federation gateway must reject any query that would resolve a foreign-tenant entity.

## Proposed Solution

The shape of the answer is a per-shard cluster topology + the cross-shard rejection at federation gateway + the per-tenant persona isolation primitive + the tenant-residency-locked-at-provisioning rule + a shard-aware operations + alerting set.

**Shard topology.**

Each shard is a fully independent stack:

| Shard | AWS Region | CDN | Data Sovereignty Profile |
|---|---|---|---|
| **vn-shard** | ap-southeast-1 (Singapore) | Cloudflare Vietnam-edge | PDPL Decree 13 + Decree 53 + Decree 20 |
| **sg-shard** | ap-southeast-1 (Singapore) | Cloudflare global | Singapore PDPA |
| **eu-shard** | eu-central-1 (Frankfurt) | Cloudflare EU-edge | GDPR + EU AI Act |
| **us-shard** | us-east-2 (Ohio) | Cloudflare global | SOC 2 + state-privacy (CCPA/CPRA/etc.) |

Each shard has:
- Its own Apollo Router supergraph at `https://api-{shard}.cyberos.world/graphql` (e.g. `api-vn.cyberos.world`).
- Its own per-shard Postgres cluster.
- Its own per-shard NATS JetStream cluster.
- Its own per-shard `cyberos-mcp-gateway` deployment.
- Its own per-shard `cyberos-ai-gateway` routing to the region-local Bedrock endpoint.
- Its own per-shard HashiCorp Vault root for KMS keys.
- Its own per-shard observability stack (Loki + Prometheus + Tempo).
- Its own per-shard Trust Center status page (`status-{shard}.cyberos.world`).

The vn-shard + sg-shard share the Singapore region but are *separate clusters* — the data sovereignty profiles differ (PDPL home regime vs. Singapore PDPA) and tenant-billing ownership differs (Vietnamese-tenant invoices issued by `cyberskill_jsc`; international-tenant invoices issued by the Singapore HoldCo from FR-CORP-001).

**Tenant residency provisioning.**

When a tenant signs up:

1. The tenant administrator (the founder of an external customer) selects residency at provision time.
2. Residency is a **first-class field** in `cyberos_meta.tenant`:
   ```sql
   CREATE TABLE cyberos_meta.tenant (
     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
     slug TEXT NOT NULL UNIQUE,
     legal_entity_name TEXT NOT NULL,
     residency_shard TEXT NOT NULL,                       -- "vn" | "sg" | "eu" | "us"
                                                         -- IMMUTABLE post-provisioning except via paid migration
     residency_locked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     legal_jurisdiction_country TEXT NOT NULL,
     status TEXT NOT NULL DEFAULT 'provisioning',          -- "provisioning" | "active" | "suspended" | "archive_pending"
                                                         -- | "deletion_pending" | "deleted"
     plan_tier TEXT NOT NULL,                              -- "T1_starter" | "T2_growth" | "T3_enterprise"
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     metadata JSONB NOT NULL DEFAULT '{}'::jsonb
   );
   ```
3. Provisioning routes to the correct shard's cluster — the tenant's row is created in `{shard}.cyberos_meta.tenant` (each shard's Postgres has its own `cyberos_meta` schema + tenant table).
4. The tenant's user-facing URL is `https://{slug}.cyberos.world` which DNS-routes to the correct shard via Cloudflare Workers (`pick_shard(slug) → cluster IP`).
5. Changing residency requires a paid migration: full export from source shard + import to destination shard + re-issue of all signed certificates + update of all federation references. Tracked separately as a P4+ FR.

**Cross-shard reference forbidden.**

Three architectural enforcements:

1. **Federation gateway-level.** Apollo Router on shard X cannot resolve `@external` entities pointing to shard Y; the federation registry per shard contains only entities for that shard's tenants.
2. **Database-level.** The `tenant_id` UUID in any per-tenant table on shard X cannot reference a tenant living on shard Y; the validation runs at provisioning + at every cross-tenant-attempt.
3. **NATS subject-level.** A shard's NATS cluster's JetStream subjects do not include other shards' subject patterns; durable consumers cannot subscribe across shards.

The Apollo Router + the AI Gateway + the MCP Gateway all check the tenant's shard at request entry and reject if the request's resolved tenant lives on a different shard than the one serving.

**Per-tenant CUO persona isolation.**

Each tenant has its own copy of the persona Skills directory + its own `cyberos_meta.persona_version_active`:

```sql
-- Per-tenant persona version active (extending FR-GENIE-002).
CREATE TABLE cyberos_meta.tenant_persona_version (
  tenant_id UUID NOT NULL REFERENCES cyberos_meta.tenant(id),
  skill_id TEXT NOT NULL,                                  -- "cuo-ceo" | "cuo-coo" | etc.
  active_version TEXT NOT NULL,                             -- "cuo-ceo-v0.4.2"
  paused_at TIMESTAMPTZ,
  paused_reason TEXT,
  signed_by_tenant_admin_at TIMESTAMPTZ NOT NULL,           -- the tenant's founder signs (not the CyberOS founder)
  signed_by_tenant_engineering_at TIMESTAMPTZ,
  signed_by_cyberos_caio_review_at TIMESTAMPTZ,             -- platform-level review for high-risk persona changes
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, skill_id)
);
```

When Tenant A pauses their CUO/COO due to a regression, Tenant B's CUO/COO continues running. The pause flows through the AI Gateway's per-tenant routing — the gateway reads the calling tenant's `tenant_persona_version` and returns the appropriate persona-system-prompt or rejects with `PERSONA_PAUSED` per tenant.

**Tenant defaults vs. tenant overrides.**

The platform maintains a "platform-default" persona-version per skill (the version CyberSkill itself uses, plus the version recommended for new external tenants). A tenant can:
- Use the platform-default (default behaviour; opt-out only).
- Pin a specific version (with their own dual-sign + the CyberOS CAIO review for compliance-relevant skills like CFO/CHRO/CRO).
- Pause an entire skill (with their own founder sign).

Cross-tenant migration of a persona version (the platform releases v0.5; tenants on v0.4 receive a Notify "v0.5 available; review changelog") is a coordinated rollout pattern.

**Schema-per-tenant + RLS unchanged.**

The FR-INFRA-001 schema-per-tenant + RLS pattern carries forward inside each shard's Postgres cluster. The shard adds another layer above: the cluster itself contains only tenants of one residency. So the layered defence is:

1. Cluster-level: only tenants of this residency live here.
2. Schema-per-tenant: each tenant has its own `tenant_{slug}` schema.
3. RLS-as-floor: every tenant-scoped table has `FORCE ROW LEVEL SECURITY` with the `app.tenant_id` check.
4. PgBouncer transaction-mode: cross-tenant connection-leak impossible.

A breach of any single layer is mitigated by the others.

**Shared services (per-shard).**

Some surfaces are tenant-shared but shard-isolated:
- The auth + magic-link surfaces (`auth-{shard}.cyberos.world`).
- The MCP gateway (per shard).
- The AI Gateway (per shard).
- The Trust Center (per shard).
- The compliance-cockpit aggregate dashboards (per shard).

The platform's marketing site + the Public APIs (P4) are *cross-shard* — they live on a separate "founder-tier" deployment that does NOT have access to any tenant data.

**Operations: shard provisioning + DR.**

A new shard takes ~6 weeks to provision:
- Provision AWS region + Bedrock access.
- Provision Hetzner equivalent if Hetzner is the cluster (Hetzner has EU + US presence; vn-shard uses AWS Singapore from FR-INFRA-001's Singapore decision).
- Provision per-shard Postgres + NATS + Vault.
- Provision per-shard observability stack.
- Provision per-shard Cloudflare DNS + WAF.
- Provision per-shard Trust Center.

DR: cross-region replication within the same shard (e.g. vn-shard's Postgres replicated from ap-southeast-1 to ap-southeast-2 as DR; primary stays ap-southeast-1).

**MCP tool surface.**

Per-shard read-only:

- `cyberos.tenant.my_tenant_info` — read; the calling tenant's metadata.
- `cyberos.tenant.list_my_tenant_users` — read; tenant administrators see their own tenant's users.
- `cyberos.tenant.list_my_persona_versions` — read.
- `cyberos.tenant.platform_recommended_persona_versions` — read; informational.

Mutation tools (tenant-administrator-only):

- `cyberos.tenant.set_persona_version(skill_id, version)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`. Tenant admin's dual-sign required + CAIO review for high-risk skills.
- `cyberos.tenant.pause_persona(skill_id, reason)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.tenant.resume_persona(skill_id)` — `destructive: true; requires_confirmation: true`.

There are no MCP tools to mutate residency, plan tier, or status — those are platform-level ops surfaces for the CyberSkill founder + CyberSkill engineering team only.

**Audit integration.** `tenant.{tenant}` audit scope at the per-tenant level + `platform.shard.{shard}` at the platform level for shard-level operations.

## Alternatives Considered

- **Single global cluster with strong RLS.** Rejected: cluster-level partitioning is the structural answer to "data never leaves region X"; RLS alone cannot satisfy data-sovereignty laws.
- **Database-per-tenant inside a single cluster.** Rejected for the standard tier; this remains the upgrade for T3 enterprise tenants per PRD §8.8 (a per-cluster row-isolation + a per-T3 separate database hybrid).
- **Cross-shard federation for shared accounts.** Rejected: the architectural rule is per-shard isolation; a CyberOS user with two tenants on different shards has two separate identities.
- **Skip multi-region in P3; defer to P4.** Rejected: PRD §14.4.1 explicitly scopes "full residency partitioning (vn-shard, sg-shard, eu-shard)" for P3; the first external client at the P3 → P4 gate may be Vietnamese (most likely) or international (possible).

## Success Metrics

- **Primary metric.** P3 sprint demo passes: (1) the 4 shards (vn/sg/eu/us) are deployed with the per-shard topology; (2) a synthetic Vietnamese tenant + a synthetic EU tenant are provisioned to vn-shard + eu-shard respectively; (3) cross-shard query attempts are rejected at the federation gateway with `code: CROSS_SHARD_REFERENCE_FORBIDDEN`; (4) per-tenant persona pause on the synthetic Vietnamese tenant does not affect the synthetic EU tenant.
- **Compliance metric.** Zero cross-shard data leakage in the synthetic test suite; zero in production.
- **Latency NFR.** Per-shard latency budgets per FR-INFRA-001 maintained; cross-shard query rejection ≤ 50 ms p95 (just identifies + rejects).

## Scope

**In-scope.**
- 4-shard topology deployed.
- Per-shard Apollo + Postgres + NATS + Vault + AI Gateway + MCP Gateway + observability + Trust Center.
- `cyberos_meta.tenant` schema with residency + lifecycle.
- Cross-shard reference rejection at gateway + DB + NATS.
- `cyberos_meta.tenant_persona_version` with per-tenant pause + version pinning.
- Cloudflare Workers DNS routing per shard.
- DR replication per shard (cross-region within same shard).
- The 4 read MCP tools + 3 mutation MCP tools.
- Audit integration in `tenant.{tenant}` + `platform.shard.{shard}` scopes.

**Out-of-scope (deferred).**
- Tenant lifecycle UI (FR-TEN-002).
- Per-tenant theme overrides (FR-TEN-003).
- Cross-shard tenant migration (P4+).
- T3 enterprise per-database isolation upgrade (P4+).
- Cross-shard CyberSkill-internal aggregate analytics (P4 — the founder's view of all CyberOS tenants is a P4 product surface, not a P3 ops surface).

## Dependencies

- All P0-P2 FRs (the underlying platform replicated per-shard).
- AWS account in 4 regions + per-region Bedrock access.
- Hetzner accounts where applicable.
- Cloudflare zone for `cyberos.world` with per-shard subdomain routing + WAF rules.
- HashiCorp Vault per-shard cluster.
- DNS + DNSSEC for per-shard subdomains.
- Compliance: PDPL Decree 13 + 53 + 356/2025 (vn-shard); Singapore PDPA (sg-shard); GDPR + EU AI Act (eu-shard); SOC 2 + state-privacy (us-shard); ISO 27001 cluster-level controls.
- Locked decisions referenced: DEC-257 (4-shard topology), DEC-258 (residency immutable post-provisioning), DEC-259 (cross-shard reference forbidden at all 3 layers), DEC-260 (per-tenant persona-version isolation).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Multi-tenancy + residency partitioning is deterministic infrastructure; AI surfaces continue to operate per-tenant per-shard with the existing classifications.
