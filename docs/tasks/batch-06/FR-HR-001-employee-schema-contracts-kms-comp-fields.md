---
title: "HR — employee schema + contracts + role/title history; comp fields encrypted with separate KMS key; non-comp ingested into BRAIN"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q1"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the HR module's foundational schema and Apollo Federation subgraph: **Employee** (the canonical record per Member), **Contract** (the legally-binding employment agreement), **RoleHistory** (titles + reporting line + start/end-date events), **ProfileFields** (non-compensation personal data — phone, address, emergency contact), and the **comp-field encrypted store** (a separate Postgres schema `hr_secure` with column-level encryption keyed by a dedicated per-tenant KMS key — distinct from the per-tenant KMS key used for everything else, enforcing PRD §9.13's "comp fields encrypted with separate KMS key"). Non-compensation profile data is BRAIN-ingestible (FR-BRAIN-002 ingestion-side denylist excludes the encrypted compensation fields by structural design); the encrypted store is read only by REW (FR-REW-001..007) and HR/Ops Lead UI surfaces. This FR ships the data substrate; subsequent batch-06 FRs ship onboarding (FR-HR-002), 1:1 + directory UX (FR-HR-003), and the entire REW stack (FR-REW-001..007).

## Problem

The team's HR data today lives in: signed paper contracts in a folder; Excel for compensation; Notion for soft profiles; the founder's memory for promotions. Three failure modes the platform must structurally prevent:

- **Comp data leakage into BRAIN.** PRD §4.3 anti-metric: "Compensation / equity values appearing in BRAIN vector or filesystem layer = 0". Without architectural separation (separate schema + separate KMS key + ingestion-side denylist + denylist regex sweep), a single careless write leaks salary to the vector index.
- **Lost role-history.** "When was Khoa promoted to senior?" is answerable today only by asking the founder. Structured role history makes this queryable + audit-grade.
- **Per-Vietnamese-labour-law contract metadata missing.** Probation period, contract type (định-hạn / không-định-hạn), termination notice period, statutory leave entitlements — the law expects structured records; without them, severance disputes default to the worst-case interpretation for the company.

## Proposed Solution

The shape of the answer is two Postgres schemas (`hr` for normal-confidentiality data; `hr_secure` for compensation) + an Apollo subgraph + per-tenant KMS-key separation + RLS + audit.

**Public-confidentiality schema (`hr`).**

```sql
CREATE SCHEMA hr;

-- Employee: 1:1 with auth.member; the HR canonical record.
CREATE TABLE hr.employee (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL UNIQUE REFERENCES auth.member(id),  -- federation
  employee_code TEXT NOT NULL,                                -- "EMP-0001" (sequential, never reused)
  legal_full_name TEXT NOT NULL,                              -- official Vietnamese-name with diacritics
  preferred_name TEXT NOT NULL,
  date_of_birth DATE,
  gender TEXT,                                                 -- self-declared; optional
  national_id_country TEXT,                                    -- "VN" | "US" | etc.
  national_id_redacted_last4 TEXT,                             -- only the last 4; full ID lives in hr_secure
  primary_email TEXT NOT NULL,
  personal_email TEXT,
  primary_phone TEXT,
  hire_date DATE NOT NULL,
  termination_date DATE,
  termination_reason TEXT,                                     -- "voluntary" | "involuntary" | "redundancy" | "good_leaver" | "bad_leaver"
                                                               -- (full Good/Bad Leaver classification in FR-REW-007)
  current_status TEXT NOT NULL DEFAULT 'active',               -- "candidate" | "probation" | "active" | "leave"
                                                               -- | "notice_period" | "terminated"
  primary_owner_member_id UUID NOT NULL,                       -- the employee's manager (HR/Ops Lead seeds)
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX employee_status_idx ON hr.employee (tenant_id, current_status);
CREATE INDEX employee_member_idx ON hr.employee (tenant_id, member_id);

-- Profile fields: address, emergency contact, banking metadata (the bank account number lives in hr_secure).
CREATE TABLE hr.profile (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL UNIQUE REFERENCES hr.employee(id) ON DELETE CASCADE,
  permanent_address_redacted TEXT,                              -- "Tan Dinh Ward, HCMC, VN"
  current_address_redacted TEXT,
  emergency_contact_name TEXT,
  emergency_contact_relationship TEXT,
  emergency_contact_phone_last4 TEXT,                            -- last 4 only; full lives in hr_secure
  bank_name TEXT,                                                -- "Vietcombank" — provider name only
  -- bank_account_number lives in hr_secure
  social_insurance_id_last4 TEXT,                                -- last 4 only; full in hr_secure
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, employee_id)
);

-- Contract: the legally-binding employment agreement; one active at a time.
CREATE TABLE hr.contract (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  contract_kind TEXT NOT NULL,                                  -- "definite_term" | "indefinite_term" | "internship"
                                                                -- (per Vietnamese Labour Code Articles 20-21)
  start_date DATE NOT NULL,
  end_date DATE,                                                 -- null for indefinite_term
  probation_period_months INT,                                   -- 0..6 per Vietnamese Labour Code Article 25
  notice_period_days INT NOT NULL,                               -- statutory floor: 30 for indefinite, 7-30 for definite
  signed_document_id UUID,                                       -- references the DOC module's signed-PDF when DOC ships P3
  signed_at DATE,
  primary_workplace TEXT,                                        -- "remote" | "Ho Chi Minh City office" | etc.
  weekly_working_days REAL NOT NULL,                             -- typically 5.0 or 5.5
  daily_working_hours REAL NOT NULL,                             -- typically 8.0
  superseded_by UUID REFERENCES hr.contract(id),                 -- a renewal supersedes the prior contract
  status TEXT NOT NULL DEFAULT 'active',                          -- "draft" | "active" | "ended" | "superseded" | "terminated"
  termination_date DATE,
  termination_reason TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX contract_employee_idx ON hr.contract (tenant_id, employee_id, status);

-- Role history: every title/reporting-line/team change captured.
CREATE TABLE hr.role_history (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  effective_from DATE NOT NULL,
  effective_to DATE,                                              -- null for current
  title TEXT NOT NULL,                                            -- "Senior Software Engineer"
  level TEXT,                                                     -- "L3", "Senior", "Staff", etc. (per LEARN P2 career path)
  team TEXT,                                                       -- "Engineering — Backend"
  reports_to_employee_id UUID REFERENCES hr.employee(id),
  is_promotion BOOLEAN NOT NULL DEFAULT false,
  promotion_decision_id UUID,                                      -- references LEARN module's Hội đồng Chuyên môn decision
  reason_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX role_history_employee_idx ON hr.role_history (tenant_id, employee_id, effective_from DESC);
```

**Comp-secure schema (`hr_secure`).**

```sql
CREATE SCHEMA hr_secure;

-- The KMS key for hr_secure is provisioned distinctly from the tenant's main KMS key (FR-CP-002).
-- The application stores key-ID + envelope-encrypts column values via pgcrypto + the tenant's hr_secure key.

-- Sensitive identity fields.
CREATE TABLE hr_secure.identity (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL UNIQUE REFERENCES hr.employee(id) ON DELETE CASCADE,
  national_id_full_encrypted BYTEA,                              -- VN CCCD 12-digit; encrypted
  passport_number_encrypted BYTEA,
  date_of_birth_full DATE,                                        -- duplicated here under encryption for completeness
  social_insurance_id_full_encrypted BYTEA,                      -- VN BHXH ID
  personal_income_tax_id_encrypted BYTEA,                         -- VN MST cá nhân
  bank_account_number_encrypted BYTEA,                            -- bank routing number is in plain hr.profile
  emergency_contact_phone_full_encrypted BYTEA,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, employee_id)
);

-- Salary history is in REW (FR-REW-001) — that schema is also under the hr_secure KMS-key envelope.
```

The `hr_secure` schema is encrypted at the column level via `pgcrypto.pgp_sym_encrypt` with the per-tenant `hr_secure` KMS key (separate from the tenant's main KMS key — a leak of the main key does not decrypt comp/identity data; defence in depth). Application reads decrypt at retrieval time within the request-scoped transaction; decrypted values never enter the audit log payload (only "redacted: true" markers). The KMS key lives in HashiCorp Vault under a distinct `hr_secure/{tenant_id}` path; audit-trail every key access.

**RLS + ACL.**

- All `hr.*` tables: tenant-RLS + Member ACL — a Member sees their own employee record, their direct reports' records (if they manage), and full data if they have `hr.read.*` (HR/Ops Lead, Founder, Auditor, DPO).
- All `hr_secure.*` tables: tenant-RLS + a *more restrictive* ACL — only HR/Ops Lead, Founder, and DPO can SELECT; even Auditor cannot. Every read writes an audit row with `actor_subject` and the `field_kind` accessed (e.g. "national_id_full_decrypted").

**BRAIN ingestion contract.**

- `hr.employee.preferred_name`, `hr.employee.primary_email`, `hr.role_history.title`, `hr.role_history.team`, and `hr.contract.contract_kind` are BRAIN-ingestible (non-sensitive professional facts).
- Everything else in `hr.*` is *opt-out* of BRAIN ingestion (specifically `national_id_redacted_last4`, addresses, emergency contacts, bank-name).
- The entirety of `hr_secure.*` is **structurally excluded** from BRAIN by ingestion-side denylist (FR-BRAIN-002 §"Denylist filter") — the source-listener's allowlist on `hr.*` events does not include `hr_secure.*` events at all.
- A nightly denylist-sweep over `brain.fact.text` re-asserts the absence of any compensation/identity values (FR-BRAIN-002 guardrail metric).

**Federation directives.**

- `HrEmployee @key(fields: "id")` — exposes the public-confidentiality fields to other subgraphs.
- `Member @key(fields: "id") @external` — references AUTH.
- `HrEmployee @extends type ProjEngagement { primaryOwnerEmployee: HrEmployee @requires("primaryOwnerMemberId") }` etc. — cross-subgraph joins.

**GraphQL subgraph.**

```graphql
type Query {
  hrEmployees(status: [String!], teamId: ID, first: Int = 50): HrEmployeeConnection!
  hrEmployee(id: ID, memberId: ID, code: String): HrEmployee
  hrEmployeeContracts(employeeId: ID!): [HrContract!]!
  hrEmployeeRoleHistory(employeeId: ID!): [HrRoleHistoryEntry!]!
  hrEmployeeProfile(employeeId: ID!): HrProfile
  # secure surface — separate query path, more-restrictive ACL, audited:
  hrSecureIdentity(employeeId: ID!): HrSecureIdentity     # only HR/Ops + Founder + DPO; every read audited
}

type Mutation {
  hrCreateEmployee(input: HrEmployeeInput!): HrEmployee!
  hrUpdateEmployee(id: ID!, patch: HrEmployeePatch!): HrEmployee!
  hrUpdateProfile(employeeId: ID!, patch: HrProfilePatch!): HrProfile!
  hrUpsertSecureIdentity(employeeId: ID!, patch: HrSecureIdentityPatch!): HrSecureIdentity!  # writes to hr_secure under KMS key
  hrCreateContract(input: HrContractInput!): HrContract!
  hrSupersedeContract(prevContractId: ID!, newInput: HrContractInput!): HrContract!
  hrTerminateContract(id: ID!, terminationDate: Date!, reason: String!): HrContract!
  hrCreateRoleHistory(input: HrRoleHistoryInput!): HrRoleHistoryEntry!
  hrEndRoleHistory(id: ID!, effectiveTo: Date!): HrRoleHistoryEntry!
}
```

Persisted-queries discipline applies. The `hrSecureIdentity` queries and `hrUpsertSecureIdentity` mutations carry an `@stepUp` directive (FR-AUTH-003) — every secure-identity touch requires step-up auth.

**Audit integration.**

- `hr.{tenant}` audit scope for public-confidentiality writes.
- `hr_secure.{tenant}` audit scope for any read or write of encrypted fields. Every audit row carries the `field_kind` (e.g. "national_id_full") + the `purpose` (a free-text justification the requester provides; required for every hr_secure read).
- Audit-row payloads never include the actual decrypted value (the chain hash and forensic recoverability are sufficient).

**MCP tool surface (read-only, public-confidentiality only).**

- `cyberos.hr.list_employees(status?, team?)`
- `cyberos.hr.get_employee(id_or_member_id_or_code)`
- `cyberos.hr.list_contracts(employee_id)`
- `cyberos.hr.list_role_history(employee_id)`
- `cyberos.hr.get_profile(employee_id)` — returns the redacted-public fields; *not* the secure-identity fields.

`hr_secure.*` is **deliberately not exposed via MCP** — agents cannot read encrypted identity / banking data even with confirmation. A human visits the HR/Ops UI for those fields. This is the same architectural principle as PRD §6.4 (CUO defers compensation to humans).

## Alternatives Considered

- **Single schema for everything; rely on RLS alone.** Rejected: a single misconfigured RLS policy + a single careless column read leaks comp data. Two schemas + two KMS keys is defence in depth.
- **Separate database for hr_secure.** Considered. Rejected for P2: schema-with-distinct-KMS-key inside the same Postgres cluster meets the threat model; separate database adds operational cost. P3 may revisit for T3 enterprise tenants.
- **Plaintext storage with column-level encryption only at rest (Postgres TDE).** Rejected: TDE protects against disk-theft only; in-process attacks see plaintext. Application-level envelope encryption with a separate key prevents that.
- **Hosted HRIS (BambooHR, HiBob, Rippling).** Rejected: residency + the Total Rewards Appendix is not modelled by any HRIS we evaluated; lock-in.
- **No role-history table; query promotion log from elsewhere.** Rejected: a queryable role-history is a primary HR primitive; promotion data living in LEARN P2 is the *decision* artefact, not the *operative state*.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder creates the 10 existing employees + their contracts + their role histories; (2) RLS denies a non-manager Member's read of a peer's profile; (3) the secure-identity write requires step-up auth; (4) BRAIN ingestion contains zero values from `hr_secure.*` (verified by the nightly denylist sweep).
- **Compliance metric.** Zero comp/identity values in BRAIN over the lifetime of the platform (PRD §4.3 anti-metric, asserted nightly).
- **Audit completeness.** 100% of `hr_secure.*` reads audit-logged with `purpose` non-empty.
- **Latency NFR.** `hrEmployee` query p95 ≤ 100 ms; `hrSecureIdentity` p95 ≤ 200 ms (envelope-encryption overhead is acceptable on a per-record path).

## Scope

**In-scope.**
- The `hr` schema (5 tables) and `hr_secure` schema (1 table; REW adds more in FR-REW-001).
- Per-tenant separate KMS key for `hr_secure`.
- Application-level envelope encryption for `hr_secure.*` columns.
- RLS policies + role-based ACL for both schemas.
- `@stepUp` directive enforcement on `hrSecureIdentity` queries + writes.
- Apollo Federation v2 subgraph with the queries + mutations + federation directives.
- BRAIN ingestion contract (allowlist for `hr.*`; denylist `hr_secure.*` structural exclusion).
- Audit integration in scope `hr.{tenant}` + `hr_secure.{tenant}` with `field_kind` + `purpose` capture.
- The 5 read-only MCP tools (public-confidentiality only).
- Seed migration for the existing 10 employees (data sourced from the founder's records; the migration itself is FR-REW-006).

**Out-of-scope (deferred).**
- Onboarding workflow + Genie checklist (FR-HR-002).
- 1:1 templates + employee directory UX (FR-HR-003).
- Compensation schema + payroll (FR-REW-001..007).
- Performance reviews + Hội đồng Chuyên môn (LEARN P2 — separate FR cluster in batch-07).
- Org chart visualisation (FR-HR-003).
- DOC integration for signed contracts (P3 when DOC ships).

## Dependencies

- FR-INFRA-001 (Postgres + extensions including `pgcrypto`).
- FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 (identity, audit, step-up auth).
- FR-MCP-001 (read-only tool registration).
- FR-CP-002 (per-tenant KMS keys; this FR provisions a *second* per-tenant key for `hr_secure`).
- FR-BRAIN-002 (ingestion-side denylist; `hr_secure.*` structurally excluded).
- HashiCorp Vault for the per-tenant `hr_secure` KMS key.
- Compliance: PDPL Decree 13 (employment data + identity is personal data; the `hr_secure` separation is the structural control); Vietnamese Labour Code Articles 20-25 (contract types, probation, notice); GDPR Article 32 (security of processing); ISO 27001 Annex A.10 (cryptography); SOC 2 CC6 (logical access).
- Locked decisions referenced: DEC-156 (separate `hr_secure` schema with separate per-tenant KMS key), DEC-157 (audit `hr_secure` reads with `field_kind` + `purpose`), DEC-158 (`hr_secure` not exposed via MCP).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The HR data substrate is deterministic; AI surfaces (1:1 prep brief, onboarding checklist) ride on top in FR-HR-002 / FR-HR-003. REW AI surfaces (payslip narrator) live in FR-REW-005 with explicit `not_ai` for compute paths and `limited` for the read-only narrative.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
