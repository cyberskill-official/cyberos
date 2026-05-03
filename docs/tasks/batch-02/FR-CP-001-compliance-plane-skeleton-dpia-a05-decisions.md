---
title: "Compliance Plane (CP) skeleton — DPIA template, PDPL A05 filing automation, decisions-ledger query, regime status feed"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: not_ai
target_release: "P0 / 2026-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the **Compliance Plane (CP)** module's P0 skeleton: a single-tenant compliance evidence store + workflow engine that powers the OBS Compliance Cockpit and prepares the platform for the audit cycles ahead. P0 deliverables: the DPIA (Data Protection Impact Assessment) template + a populated DPIA per high-risk processing activity (BRAIN, GENIE/CUO, CHAT, AUTH); the **PDPL A05 filing automation** that drafts the Decree 13/2023 Article 5 personal-data-processing-impact filing for Vietnam's Ministry of Public Security; the **decisions-ledger query interface** that exposes the DEC-XXX log via natural language ("what locked decisions reference the EU AI Act?") through the CUO surface; and the **regime status feed** that publishes red/yellow/green states for every regulatory regime to the Compliance Cockpit (FR-OBS-001 / FR-OBS-002). CP also owns the cross-cutting compliance workflows that get heavier in P2 (full Decree 13 graduation), P3 (GDPR + ISO/IEC 27001 + SOC 2), and P4 (multi-tenant compliance plane); P0 is the substrate.

## Problem

The PRD's compliance posture (Part 12) is the cornerstone of its commercial viability — Vietnamese SMB consultancies cannot procure a SaaS that mishandles PDPL; EU buyers cannot procure one without an auditable EU AI Act + GDPR posture; US enterprise buyers will not start procurement without a SOC 2 path. PRD §12.7 lists the compliance backlog with explicit ownership; the P0 → P1 exit gate (PRD §14.1.3) requires "Compliance Cockpit shows green on Decree 20 SME regime and Compliance Backlog has zero P0-Sev0 items open."

Three failures the platform must avoid:

- **Drafting the A05 filing under deadline pressure.** PDPL Decree 13/2023 requires personal-data-processing-impact filings before processing begins for certain risk classes; doing this manually under deadline is the predictable cause of mistakes. The template + automation must exist before the platform processes a single Member's data in production.
- **Compliance Cockpit fed by hand-curated YAML forever.** The cockpit (FR-OBS-001) starts with a hand-curated YAML status file, but that path is unsustainable; it must be replaced by a CP-driven feed before P0 exit.
- **Decisions ledger as a hand-edited file no one reads.** The DEC-001..DEC-066 log is the architectural memory; without a query interface (CUO question, MCP tool, GraphQL field), it gets stale.

## Proposed Solution

The shape of the answer is a CP module — its own subgraph + Postgres schema + MCP server + a small set of UIs (DPIA editor, A05 filing reviewer, regime status board) — and a DPO-facing surface in the host shell.

**Postgres schema.**

```sql
CREATE SCHEMA cp;

-- Regulatory regimes the platform tracks
CREATE TABLE cp.regime (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  code TEXT NOT NULL UNIQUE,            -- "PDPL-D13", "PDPL-D53", "PDPL-D20", "GDPR", "EU-AIA",
                                        -- "SOC2-T1", "SOC2-T2", "ISO27001", "ISO42001", "PDPA-SG"
  display_name TEXT NOT NULL,
  jurisdiction TEXT NOT NULL,
  current_status TEXT NOT NULL,         -- "green" | "yellow" | "red" | "n/a"
  last_evaluated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  evidence_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
  applicable_from_phase TEXT NOT NULL    -- "P0" | "P3" | etc.
);

-- DPIA entries per high-risk processing activity
CREATE TABLE cp.dpia (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  processing_activity TEXT NOT NULL,    -- "BRAIN: per-tenant memory ingestion"
  module_code TEXT,
  data_classes TEXT[] NOT NULL,         -- ["name", "email", "phone", "chat_content", "voice_audio"]
  legal_basis TEXT NOT NULL,            -- "necessary for performance" | "consent" | "legal obligation" | "legitimate interest"
  risk_assessment_doc UUID NOT NULL,    -- references the canonical DPIA Markdown doc id
  controls JSONB NOT NULL,              -- structured controls list (technical + organisational)
  residual_risk TEXT NOT NULL,          -- "low" | "medium" | "high"
  signed_by_dpo_at TIMESTAMPTZ,
  signed_by_founder_at TIMESTAMPTZ,
  next_review_due TIMESTAMPTZ NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft', -- 'draft' | 'reviewed' | 'signed' | 'superseded'
  superseded_by UUID
);

-- A05 filings (PDPL Decree 13 Article 5 personal-data-processing-impact filings)
CREATE TABLE cp.a05_filing (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  filing_kind TEXT NOT NULL,            -- "initial" | "annual" | "material-change"
  draft_doc UUID NOT NULL,              -- the canonical filing Markdown doc id (reuses brain.layer1_file)
  evidence_dpia_ids UUID[] NOT NULL,
  signed_by_dpo_at TIMESTAMPTZ,
  filed_at TIMESTAMPTZ,
  filing_reference TEXT,                -- the receipt from MPS once filed
  status TEXT NOT NULL DEFAULT 'draft'  -- 'draft' | 'review' | 'filed' | 'acknowledged'
);

-- Decisions ledger view (mirrors docs/decisions.yaml; edits go through PR review)
CREATE TABLE cp.decision (
  id TEXT PRIMARY KEY,                   -- "DEC-001"
  title TEXT NOT NULL,
  rationale TEXT NOT NULL,
  alternatives TEXT NOT NULL,
  trade_offs TEXT NOT NULL,
  locked_at TIMESTAMPTZ NOT NULL,
  superseded_by TEXT,
  related_regimes TEXT[],                -- ["PDPL-D13", "GDPR"]
  related_modules TEXT[],
  references_url TEXT[]                  -- external citations
);

-- Compliance backlog (unresolved items from PRD §12.7 etc.)
CREATE TABLE cp.backlog_item (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  severity TEXT NOT NULL,                -- "p0-sev0" | "p1-sev0" | "p1-sev1" | etc.
  owner_role TEXT NOT NULL,
  due_at TIMESTAMPTZ,
  resolved_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'open',   -- 'open' | 'in_progress' | 'resolved' | 'wont_fix'
  related_regime TEXT,
  related_module TEXT
);
```

**DPIA template + first six DPIAs.** The DPIA template is a Markdown file in `templates/dpia/DPIA.md` covering: processing activity description, lawful basis, data classes, data subjects, recipients, retention, technical controls, organisational controls, transfer assessment (if cross-border), risk assessment, residual risk, mitigation actions, sign-offs.

P0 ships **six populated DPIAs** for the activities live in P0:

1. BRAIN — per-tenant memory ingestion across Layers 1 / 2 / 3.
2. GENIE / CUO — persona-driven AI generation grounded in tenant data.
3. CHAT — message persistence, voice transcription, smart replies.
4. AUTH — identity, session, audit log.
5. AI Gateway — outbound LLM calls to Bedrock + ZDR fallbacks.
6. MCP — agent operability surface.

Each DPIA cites the relevant FR (e.g. `FR-BRAIN-002 §"Denylist filter"`), the controls (technical + organisational), and the residual-risk assessment. The DPO + founder dual-sign each DPIA before the activity goes live in production.

**A05 filing automation.** The A05 filing is a multi-section Vietnamese-language document submitted to Vietnam's Ministry of Public Security (MPS). The automation:

1. Reads the active DPIAs.
2. Fills the A05 template fields from the DPIA structured fields plus tenant metadata (legal entity name, address, DUNS, registered DPO, lawful-basis catalogue).
3. Produces a Vietnamese-language draft (the template is bilingual VN-primary, EN-secondary; only the VN copy is filed).
4. Produces a one-page summary in English for the founder + Engineering Lead review.
5. Stores the draft in `cp.a05_filing` with status `draft`.
6. Routes to the DPO for review; the DPO signs; the filing is exported as a PDF for the human to submit through MPS's filing portal (the portal does not have a public API as of 2026-05).
7. On acknowledgement, the DPO records the filing reference number; status becomes `acknowledged`.

**Decisions-ledger query interface.** The `cp.decision` table mirrors `docs/decisions.yaml`. A small loader runs in CI on every PR to `decisions.yaml` and updates the table. Queries:

- GraphQL: `cpDecisions(query: "what touches the AI Gateway?", regime: "EU-AIA"): [Decision!]!` runs hybrid retrieval against the table's text columns.
- MCP: `cyberos.cp.search_decisions(query)` exposes the same to agents.
- CUO surface: "Genie, what locked decisions reference EU AI Act?" → CUO/CTO skill answers with citations to the DEC IDs.

The query returns each decision's full body plus related modules + regimes plus the `superseded_by` chain.

**Regime status feed.** The `cp.regime` table is updated by:

- A nightly job that re-evaluates each regime against its evidence rules (e.g. "PDPL-D13 = green if every active DPIA is signed AND the A05 filing is acknowledged AND every audit-row scope retains for the regulatory floor").
- Manual adjustments by the DPO (with audit-row entry).
- Module-emitted events (`cyberos.{tenant}.cp.evidence.{kind}.{updated}`) when a control's status changes.

The cockpit reads from this table; FR-OBS-001's hand-curated YAML is retired by P0 exit.

**DPO surface.** A `/compliance` route in the host shell with three tabs:

- **DPIA register.** List + edit + sign + view diff per DPIA. The DPO is the primary author; the founder is the second signer.
- **A05 filings.** Draft → review → file workflow.
- **Compliance backlog.** The `cp.backlog_item` triage view; the DPO can promote items to the OBS alerting routes.

**Cross-cutting workflows the CP module owns.**

- **Right-to-explanation requests.** Members or external data-subjects can ask "explain why the platform's decision affected me"; the CP module routes this to the relevant module owner with a 30-day SLA. P0 stub; P3 full surface.
- **Privacy-by-design checklist** for every new FR (this PR-review template is part of the FR template's `Dependencies` and `AI Risk Assessment` sections; CP enforces the validator).
- **Data Processing Agreement (DPA) registry** — third-party providers (AWS, Anthropic, OpenAI, Hetzner, Cloudflare, Mattermost) and their signed DPAs.

**MCP tool surface.**

- `cyberos.cp.list_regimes` — read.
- `cyberos.cp.get_regime_status(code)` — read.
- `cyberos.cp.list_dpias(module?, status?)` — read.
- `cyberos.cp.get_dpia(id)` — read.
- `cyberos.cp.search_decisions(query, regime?)` — read.
- `cyberos.cp.list_backlog(severity?, status?)` — read.

There are intentionally no write MCP tools — sign, file, approve are human surfaces only.

## Alternatives Considered

- **Hosted compliance platform (Vanta / Drata / Secureframe).** Considered and rejected for P0: Vietnamese-residency is unverifiable; PDPL-specific workflows (A05 filing) are not in their catalogue. We will *also* run a hosted platform from P3 for SOC 2 evidence collection, but the canonical compliance store remains in CyberOS for residency reasons.
- **Hand-maintain DPIAs as Markdown files only (no Postgres mirror).** Rejected: the cockpit feed and the cross-cutting checks need queryable structured data.
- **Skip CP in P0; hand-author DPIAs and an A05 filing manually.** Rejected: the manual path is exactly what fails when the platform processes its first real data; the automation is the structural mitigation.
- **Combine CP with OBS into one module.** Rejected: ownership is different (DPO + Founder for CP; Engineering Lead for OBS); the cleavage matters for accountability.

## Success Metrics

- **Primary metric.** P0 → P1 exit-gate criterion (PRD §14.1.3): "Compliance Cockpit shows green on Decree 20 SME regime and Compliance Backlog has zero P0-Sev0 items open." The cockpit's data feed is real (CP-driven, not hand-curated YAML) by S0-6.
- **DPIA coverage.** Six P0 DPIAs are populated, dual-signed, and linked into the A05 filing draft.
- **Decisions-ledger query.** "Genie, what locked decisions reference EU AI Act?" returns the matching DEC IDs with their bodies in ≤ 2 s p95.
- **A05 filing.** A draft is produced + DPO-reviewed by P0 exit; the actual MPS filing happens before any external tenant onboarding (P3).

## Scope

**In-scope (S0-5 + S0-6).**
- The `cp` schema with the five tables.
- The DPIA template + the six populated P0 DPIAs.
- The A05 filing automation (draft generator + review workflow + status tracking).
- The decisions-ledger loader from `docs/decisions.yaml` to `cp.decision`.
- The decisions search GraphQL field + MCP tool.
- The regime status nightly job feeding the Compliance Cockpit.
- The DPO surface at `/compliance`.
- The DPA registry for third-party providers.
- The MCP read tools.
- Audit integration in scope `cp.{tenant}`.

**Out-of-scope (deferred).**
- GDPR + DSAR full workflow (P3).
- ISO/IEC 27001 + SOC 2 evidence collection (P2 onwards).
- Multi-tenant compliance plane (P3 — per-tenant DPIA + per-tenant A05 filing).
- Right-to-explanation full surface (P3).
- Automated MPS portal submission (no public API as of 2026-05; manual upload remains).

## Dependencies

- FR-INFRA-001 (Postgres + NATS).
- FR-AUTH-001 / FR-AUTH-002 (DPO role + audit log).
- FR-AI-001 (DPIA AI Gateway DPIA cross-references).
- FR-MCP-001 (CP read MCP tools).
- FR-BRAIN-001 (DPIA + A05 docs persisted as `brain.layer1_file` rows in `decisions/` + `compliance/`).
- FR-OBS-001 / FR-OBS-002 (Compliance Cockpit data feed).
- The DPO must be appointed by P0 exit (PRD §14.1.1 lists DPO seeding as part of the P0 scope; in P0 the founder fills the role with an audit-row note that the conflation must be resolved by P2 entry per Decree 13 full-regime graduation, PRD §14.3.1).
- Compliance: PDPL Decree 13/2023 (Article 5 A05 filing requirement; the automation is the control); PDPL Decree 53/2022 (cybersecurity audit trail; OBS audit log feeds CP); EU AI Act Articles 5–7 (risk classification per FR; this FR's tooling enforces the classification at PR-review time).
- Locked decisions referenced: DEC-067 (CP module owns the A05 filing automation), DEC-068 (DPO + Founder dual-sign on every DPIA), DEC-069 (decisions-ledger queryable through CUO).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The CP module is workflow + storage; the AI-driven decisions search uses the existing AI Gateway and inherits its persona-stamping + transparency posture but does not introduce a new AI surface.
