---
title: "Append-only audit log with per-scope Merkle hash chain and 7y/10y retention"
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

Stand up the canonical audit log that every CyberOS module writes to and that the OBS, CP (Compliance Plane), and DPO surfaces read from. Every state-changing action across the platform — login, role change, MCP tool invocation, BRAIN write, CHAT message persisted, GraphQL mutation accepted, payslip published in P2+, equity grant in P2+ — emits a NATS event on the canonical subject and writes one row to the `audit.entry` table. The table is append-only at the database level (UPDATE and DELETE blocked by trigger and revoked at the role grant). Each row chains via a SHA-256 Merkle hash to the previous row in the same scope so any tampering — including a privileged DBA replacing rows — is detectable by re-walking the chain. Retention is 7 years for general audit and 10 years for compensation and equity audits (Vietnamese SI/PIT statutory floor). This FR is the contract that the rest of the platform is built on for compliance evidence; SOC 2 Type I (P3) and ISO/IEC 27001 (P3) cannot be passed without it.

## Problem

Three properties make audit logs especially fragile in a small team:

- A privileged operator can edit rows post-hoc. Standard append-only flags in Postgres are advisory; without a tamper-evident chain, an adversary with `superuser` rights can rewrite history undetectably. The PRD's compliance posture — Vietnamese PDPL, EU AI Act high-risk modules in P2+ (REW), SOC 2 Type II at P4 — fails an external audit if the chain is breakable.
- Every module writes to the same log, but the chain breaks if writes are out of order, dropped, or interleaved across tenants. The chain must be *per scope* (per tenant + per stream) so that a parallel write in tenant A never affects tenant B's chain integrity.
- The application that writes audit rows is the same application whose actions the audit log records. We need a write path that the application cannot bypass and a verification path that an external auditor can run without trusting the application code.

S0-2 sprint risk gate (PRD §17.2): "If a single row breaks the chain in synthetic load testing, sprint blocks." This FR is the single most important deliverable for compliance evidence in P0.

## Proposed Solution

The shape of the answer is a small, single-purpose audit subgraph and Postgres schema, owned by the Engineering Lead, with no CRUD UI of its own — the OBS module reads from it for dashboards and the CP module reads from it for regulator-ready reports.

**Schema.** A single table `audit.entry` lives in the canonical `audit` schema (separate from per-tenant schemas) and is partitioned by tenant + month for query performance:

```sql
CREATE TABLE audit.entry (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL,
  scope           TEXT NOT NULL,        -- e.g. "auth", "rew.cycle.2026-Q2", "esop.grant.2026"
  actor_kind      TEXT NOT NULL CHECK (actor_kind IN ('human','agent','system')),
  actor_subject   UUID,                 -- the human Member id
  actor_agent_id  UUID,                 -- the MCP client id, when actor_kind='agent'
  action          TEXT NOT NULL,        -- "login", "tool.invoke", "task.create", etc.
  resource_kind   TEXT,
  resource_id     TEXT,
  payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
  prev_hash       BYTEA NOT NULL,       -- SHA-256 of previous row in this (tenant, scope)
  this_hash       BYTEA NOT NULL,       -- SHA-256(prev_hash || canonical_payload)
  occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  region          TEXT NOT NULL,        -- residency region; same as tenant's
  request_id      UUID,                 -- correlates with traces in OBS
  source_ip       INET,
  user_agent      TEXT,
  -- denormalised metadata for fast dashboards:
  module          TEXT NOT NULL         -- "auth", "brain", "rew", ...
) PARTITION BY RANGE (occurred_at);

CREATE INDEX audit_entry_scope_idx     ON audit.entry (tenant_id, scope, occurred_at DESC);
CREATE INDEX audit_entry_actor_idx     ON audit.entry (tenant_id, actor_subject, occurred_at DESC);
CREATE INDEX audit_entry_resource_idx  ON audit.entry (tenant_id, resource_kind, resource_id, occurred_at DESC);
```

Monthly partitions are created automatically by a `pg_partman` job; partitions older than the retention floor (7 or 10 years per scope) are exported to S3-Glacier and detached, never dropped (the Glacier copy is the cold archive; deletion of the cold archive is a sev-0 incident).

**Append-only enforcement.** Three layers:

1. The `cyberos_app` Postgres role has `INSERT` only on `audit.entry`; `UPDATE`, `DELETE`, `TRUNCATE` are not granted.
2. A `BEFORE UPDATE OR DELETE` trigger on `audit.entry` raises `EXCEPTION 'audit log is append-only'` regardless of role; this is belt-and-braces against a misconfigured grant.
3. The `audit` schema's `OWNER` is a separate Postgres role `cyberos_audit_owner` whose login is disabled; the application cannot DROP the schema or recreate the trigger.

**Merkle hash chain.** On insert, a `BEFORE INSERT` trigger computes the hash:

```
canonical_payload = sha256(
  to_jsonb(NEW.id) || to_jsonb(NEW.tenant_id) || to_jsonb(NEW.scope) ||
  to_jsonb(NEW.actor_kind) || to_jsonb(NEW.actor_subject) || to_jsonb(NEW.actor_agent_id) ||
  to_jsonb(NEW.action) || to_jsonb(NEW.resource_kind) || to_jsonb(NEW.resource_id) ||
  NEW.payload || to_jsonb(NEW.occurred_at) || to_jsonb(NEW.region) ||
  to_jsonb(NEW.request_id) || to_jsonb(NEW.source_ip) || to_jsonb(NEW.user_agent) ||
  to_jsonb(NEW.module)
)
NEW.prev_hash = (SELECT this_hash FROM audit.entry
                 WHERE tenant_id = NEW.tenant_id AND scope = NEW.scope
                 ORDER BY occurred_at DESC, id DESC
                 LIMIT 1)  -- defaults to a 32-byte zero seed for the first row in a scope
NEW.this_hash = sha256(NEW.prev_hash || canonical_payload)
```

The canonical payload is computed by a deterministic JSON canonicaliser (RFC 8785 / JCS) so that re-serialisation does not change the hash. The trigger acquires a transaction-level advisory lock keyed on `(tenant_id, scope)` so concurrent inserts into the same scope are serialised; cross-scope and cross-tenant inserts proceed in parallel. The lock is held for the duration of the transaction, not the connection.

**Per-scope chain.** "Scope" is a free-form string assigned by the writing module. Examples:
- `auth` — all AUTH events
- `auth.session.<member-id>` — per-session granular chain (a Member's auditor view)
- `rew.cycle.2026-Q2` — the compensation cycle's chain (Vietnamese SI/PIT auditor scope)
- `esop.grant.<grant-id>` — a single equity grant's lifecycle
- `brain.fact.<fact-id>` — a single BRAIN fact's edits
- `proj.task.<task-id>` — a project task's lifecycle
- `cuo.persona.<version>` — Genie/CUO persona-version sign-off chain

Per-scope chains let an auditor request "the chain for the 2026-Q2 payroll cycle" and verify that exact chain without re-walking the entire platform's history.

**Verification path.** A CLI tool `cyberos-audit-verify` (a small Rust binary published to `crates.io/cyberos-audit-verify` and installable via `cargo install`) takes a tenant ID, a scope, and an optional date range, reads rows in order, and re-walks the chain to verify every `this_hash` matches `sha256(prev_hash || canonical_payload)`. The tool reports the position of any mismatch. The tool runs as part of a daily CI job per tenant per major scope (`auth`, `rew.*`, `esop.*`); failures page the on-call. External auditors can run the same tool given a read-only credential and a Glacier-shipped chain dump.

**NATS publishing.** Every audit row also publishes to `cyberos.{tenant}.audit.entry.created` on NATS so OBS dashboards and the Compliance Cockpit react in real time without polling. The NATS publish is deliberately fire-and-forget *after* the database commit succeeds; if NATS is unhealthy, the audit row is still durable, and a small reconciler service re-publishes any rows missing from JetStream within 60 seconds.

**Retention.** Default 7 years; scopes prefixed `rew.`, `esop.`, `hr.contract.`, `inv.tax.`, and `cp.dsar.` retain 10 years (Vietnamese SI/PIT plus accounting law). A tenant administrator can extend retention but cannot reduce it below the regulatory floor. The retention enforcement runs in `pg_partman` as a partition-attach-to-archive job; the cold archive is in `s3://cyberos-audit-archive-{region}/` with Object Lock in `Compliance` mode (immutable, no override).

**API.** The `audit` GraphQL subgraph exposes:

```graphql
type Query {
  auditEntries(scope: String!, since: DateTime, until: DateTime, first: Int = 100, after: String): AuditEntryConnection!
  auditChainSummary(scope: String!): AuditChainSummary!  # head hash, count, last_verified_at
  auditChainVerify(scope: String!): AuditChainVerifyResult!  # runs the verifier
}
```

There are no mutations on the audit subgraph — writes happen via the platform-internal `audit.write` library called by every other module, never via GraphQL.

**MCP tool surface.**

- `cyberos.audit.search` (read-only; scoped by tenant + RBAC; takes `scope`, `since`, `until`, `actor_subject`, `action` filters)
- `cyberos.audit.verify_chain` (read-only; runs the verifier on a scope and returns the result)

`destructive: false` on both. RBAC: only `Auditor`, `DPO`, and `Founder` can invoke `cyberos.audit.search` across all scopes; Members can search their own scope (`auth.session.<self>`).

## Alternatives Considered

- **External SIEM (Splunk, Datadog Audit, AWS CloudTrail).** Rejected for the source of truth role: the audit log must be inside the same Postgres cluster as the data it audits so an outage in the SIEM does not lose audit events, and so cross-references between `resource_id` and the actual resource row work without a join across systems. We will *also* forward the chain to a SIEM at P3 for redundancy, but the database is the authority.
- **Append-only via WORM storage at S3 only.** Rejected: read latency is too high for the dashboards and the chain verification step; the database is the hot tier, S3 Glacier with Object Lock is the cold tier.
- **Hyperledger Fabric / blockchain.** Rejected: complexity-to-value ratio fails for our scale class. A SHA-256 Merkle chain in Postgres provides the same tamper-evidence at three orders of magnitude lower operational cost.
- **Hash chain over batches rather than per-row.** Rejected: per-row chains let us isolate a single tampered or corrupted row to the millisecond; batch-level chains lose this resolution and complicate the auditor's reconstruction.

## Success Metrics

- **Primary metric.** Daily `cyberos-audit-verify` runs against the CyberSkill tenant pass for 100% of scopes for 14 consecutive days at P0 exit.
- **Guardrail metric.** Zero successful UPDATE or DELETE against `audit.entry` for the lifetime of the platform. (A successful UPDATE would mean both the GRANT layer and the trigger layer were defeated; this is sev-0.)
- **Performance NFR.** Audit insert path adds ≤ 8 ms to the writing transaction at p99 under 1,000 inserts/sec synthetic load (NFR-PERF-AUDIT-001).

## Scope

**In-scope (S0-2).**
- `audit` schema, `audit.entry` table, partitioning, triggers, role grants, deny-list for UPDATE/DELETE.
- `audit.write(payload)` library function (Rust + Node + Python bindings) used by every module.
- Merkle-chain trigger with deterministic JSON canonicalisation.
- `cyberos-audit-verify` CLI published to `crates.io`.
- `audit` GraphQL subgraph + MCP server with read-only tools.
- NATS publish on `cyberos.{tenant}.audit.entry.created` and the reconciler that bridges DB→NATS gaps.
- `pg_partman` job for monthly partitioning + retention windowing.
- Daily CI verifier job paging on chain failure.
- AUTH (FR-AUTH-001) integration: every login, MFA enrolment, session revoke writes to the `auth` scope.
- Synthetic 1,000-insert/sec load test demonstrating chain integrity holds under concurrency.

**Out-of-scope (deferred).**
- S3 Glacier cold archive + retention export job (P0 ships with hot-tier retention only; cold archive lands in S0-6 by FR-OBS-001 or its successor).
- SIEM forwarding (P3).
- Per-Member auditor portal UI (P3 — for now, OBS dashboards plus the GraphQL API are the surfaces).
- Right-to-erasure handling: erasure of a Member affects audit rows, but PDPL and accounting law floor over GDPR — the audit row is preserved and the personal payload is pseudonymised. This is fully specified in a P3 FR; P0 records the policy as `payload.pii_pseudonymisation_pending: true` on the relevant rows so the deferred job has a worklist.

## Dependencies

- FR-INFRA-001 (Postgres + NATS scaffold) must be shipped.
- FR-AUTH-001 (the first writer of audit rows) co-ships in S0-2.
- HashiCorp Vault for the column-encryption keys used by some `payload.encrypted_*` fields in P2+.
- Compliance: PRD §12.1.1 — Vietnamese SI/PIT statute requires 10y retention on compensation; PRD §12.1.2 — Cybersecurity Law Decree 53 requires audit log preservation for security-incident investigation.
- Locked decisions referenced: DEC-019 (Postgres-native append-only audit log), DEC-020 (Merkle hash chain), DEC-021 (per-scope chain), DEC-022 (7y/10y retention), DEC-023 (chain verifier as external CLI).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The audit log is fully deterministic; no AI-derived behaviour is part of the write or verify path. AI-system audits ride on top of this infrastructure (the AI Gateway writes audit rows for every LLM call; FR-AI-001 specifies the payload shape) but the audit log itself does not "use AI".

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
