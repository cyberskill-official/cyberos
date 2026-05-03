---
title: "OBS — P2 → P3 phase-gate evidence map; ISO/IEC 27001 gap-list ≥85%; SOC 2 Type I scope; gate-readiness dashboard"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the **P2 → P3 phase-gate evidence map** that tracks every PRD §14.3.2 P2 → P3 exit-gate criterion + accumulates the audit-ready evidence to pass it. Six surfaces: **gate-readiness dashboard** showing each criterion's status (green/yellow/red); **ISO/IEC 27001 gap-list** at ≥85% completion (PRD §14.3.2 explicit threshold); **SOC 2 Type I evidence map** populated for the Common Criteria + Trust Service Criteria; **PDPL Decree 13 full-regime evidence bundle** (FR-CP-003 outputs surfaced); **NFR coverage report** showing every PRD §11.2 NFR against measured values; **founder gate-readiness signing flow** that consumes the dashboard's green status to produce a phase-exit RFC. Extends FR-OBS-001 + FR-OBS-002 with the P2 → P3 specifics. The dashboard is the founder's primary surface during the gate window; the evidence bundle is what the external auditor consumes (SOC 2 Type I auditor pre-engagement + ISO/IEC 27001 Stage 1 auditor at P3 entry).

## Problem

PRD §14.3.2 P2 → P3 exit gate is precise: "Payroll cycle close has been completed entirely inside REW module for at least 2 consecutive cycles, with zero anomalies escaped to post-close discovery. OKR cycle close has been completed entirely inside OKR module for at least 1 quarter. All 10 (or more, if hired) employees have a populated HR record, signed contract in DOC (P3 module shimmed in P2), and an active LEARN career-path entry. Decree 13 full-regime graduation is signed off by Founder/DPO; ISO/IEC 27001 gap-list is ≥85% complete. Compliance Cockpit shows green; CUO acceptance rate ≥40% across 7-day rolling window; auto-pause behaviour has been triggered and recovered at least once. Founder signs gate-readiness; the audit-log entry includes the 14-month milestone-arc check (Part 1.4)."

Three failure modes:

- **Gate-criteria opacity.** Without a structured dashboard, "are we gate-ready?" is answerable only by manual checklist walk; the founder cannot focus efforts.
- **ISO 27001 + SOC 2 evidence accumulation lag.** Audits at P3 entry need evidence that started accumulating in P0; without an evidence map, the founder discovers gaps days before the audit.
- **NFR drift unnoticed.** PRD §11.2 NFRs that fell amber/red weeks before gate are unrecoverable in days; structured tracking is the floor.

## Proposed Solution

The shape of the answer is a `obs.gate_*` schema + the gate-readiness dashboard + the evidence-map auto-collectors + the founder sign-and-publish flow.

**Schema extensions.**

```sql
-- Per-gate criterion (P0 → P1, P1 → P2, P2 → P3, P3 → P4).
CREATE TABLE obs.gate_criterion (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  gate_kind TEXT NOT NULL,                                              -- "p0_to_p1" | "p1_to_p2" | "p2_to_p3" | "p3_to_p4"
  criterion_code TEXT NOT NULL,                                         -- e.g. "rew_2_consecutive_cycles_clean"
  display_name TEXT NOT NULL,
  description_md TEXT NOT NULL,
  measurement_query JSONB NOT NULL,                                      -- structured: how the criterion is measured
                                                                       -- e.g. { kind: "prometheus", expr: "..." }
                                                                       -- or { kind: "sql", query: "SELECT count(*) ..." }
                                                                       -- or { kind: "manual_attestation", evidence_blob_id_field: "..." }
  threshold JSONB NOT NULL,                                              -- e.g. { gte: 2 } or { eq: 0 } or { pct_gte: 85 }
  current_value JSONB,                                                   -- updated by daily evaluator job
  status TEXT NOT NULL DEFAULT 'pending',                                 -- "pending" | "green" | "yellow" | "red" | "n_a"
  evidence_blob_ids UUID[],                                              -- supporting documents (e.g. payroll-cycle audit reports)
  attestation_md TEXT,                                                   -- founder/DPO/etc. attestation when manual
  signed_off_at TIMESTAMPTZ,
  signed_off_by UUID,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, gate_kind, criterion_code)
);

-- Per-NFR measurement.
CREATE TABLE obs.nfr_measurement (
  tenant_id UUID NOT NULL,
  nfr_id TEXT NOT NULL,                                                  -- "NFR-PERF-AUTH-001" etc. from PRD §11.2
  evaluation_date DATE NOT NULL,
  measured_value NUMERIC,
  threshold_value NUMERIC,
  status TEXT NOT NULL,                                                  -- "green" | "yellow" | "red"
  measurement_evidence_blob_id UUID,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, nfr_id, evaluation_date)
);

-- Phase-exit RFC (the founder's published signed gate-readiness statement).
CREATE TABLE obs.phase_exit_rfc (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  gate_kind TEXT NOT NULL,
  rfc_md TEXT NOT NULL,                                                  -- the founder-authored RFC
  criteria_snapshot_at TIMESTAMPTZ NOT NULL,                             -- when the criteria were evaluated for this RFC
  criteria_snapshot JSONB NOT NULL,                                      -- the criteria status at sign time
  nfr_snapshot JSONB NOT NULL,                                            -- NFR coverage at sign time
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ,
  signed_by_dpo_at TIMESTAMPTZ,                                            -- for compliance-relevant gates (P2 → P3 + P3 → P4)
  audit_log_entry_id UUID,                                                 -- the canonical audit row's ID
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**P2 → P3 criterion catalogue (seed).**

| Code | Description | Threshold |
|---|---|---|
| rew_2_consecutive_cycles_clean | 2 consecutive payroll cycles closed entirely in REW with 0 anomalies escaped | count >= 2 |
| okr_1_quarter_in_module | 1 OKR quarter closed entirely inside OKR module | count >= 1 |
| hr_records_complete | All employees with populated HR record + active contract + active LEARN entry | pct = 100 |
| decree_13_full_regime_signed | DPO + Founder signed FR-CP-003 graduation | bool = true |
| iso_27001_gap_list_85 | ISO/IEC 27001 gap-list completion ≥ 85% | pct_gte = 85 |
| compliance_cockpit_green | Compliance Cockpit shows green on all P2 regimes | all_green = true |
| cuo_acceptance_40_rolling | CUO acceptance rate ≥ 40% rolling-7-days for ≥ 14 days | days_at_threshold >= 14 |
| cuo_auto_pause_observed | Auto-pause behaviour has triggered + recovered ≥ once | count >= 1 |
| founder_gate_signing | Founder gate-readiness signed | bool = true |
| 14_month_milestone_arc_check | The PRD §1.4 milestone-arc audit-log entry | exists = true |

These are seeded in `obs.gate_criterion` for `gate_kind: "p2_to_p3"` at P2 mid-cycle (typically Month 8 of P2). Each criterion has a `measurement_query` defining how it's evaluated.

**Daily evaluator job.**

Runs at 06:00 ICT:
1. Per criterion: execute `measurement_query` (Prometheus expr / SQL / manual attestation check); update `current_value` + `status`.
2. Per PRD §11.2 NFR: execute the NFR's measurement query; update `obs.nfr_measurement`.
3. Per regulatory regime (PDPL-D13, GDPR (P3-readiness), EU AI Act, SOC 2 Type I scope, ISO 27001): aggregate panel-status from underlying signals.
4. Push aggregate status to OBS dashboards; trigger founder Notify if any criterion regresses from green → yellow.

**Gate-readiness dashboard (`/obs/gate-readiness/p2-to-p3`).**

For Founder + Engineering Lead + DPO + Auditor.

- **Header.** Gate name + target date + days-to-target.
- **Criteria grid.** Each criterion as a card: status chip (green/yellow/red), current_value vs. threshold, evidence-blobs link, "view trend" mini-chart.
- **NFR coverage panel.** Per-NFR roll-up table: green count, yellow count, red count; click to drill into a specific NFR's history.
- **Compliance Cockpit deep-link.** Embedded panel with the 6 regime statuses (PDPL-D13, PDPL-D53, PDPL-D20, GDPR-readiness, EU-AIA, SOC 2 + ISO 27001).
- **External auditor evidence panel.** Generated FR-CP-003 regulator-artefact bundles + their delivery status.
- **Founder sign action.** Disabled until all criteria are green (with explicit yellow allowed when documented mitigation plan exists); enables the "Author + Sign Phase-Exit RFC" CTA.

**Phase-exit RFC flow.**

When all criteria are green (or yellow with documented mitigation):

1. Founder navigates to `/obs/gate-readiness/p2-to-p3/sign`.
2. Step-up auth.
3. CUO/CAIO drafts a Phase-Exit RFC narrative (read-only AI; founder edits + finalises):
   - The 14-month milestone-arc check from PRD §1.4.
   - Each criterion's status with cited evidence.
   - Open risks + mitigation plans.
   - Confirmation of Founder + Engineering Lead + DPO sign chain.
4. Founder signs (`signed_by_founder_at`); Engineering Lead signs; DPO signs (mandatory for P2 → P3 due to compliance graduation).
5. The RFC is written as `obs.phase_exit_rfc` row; the audit-log entry's ID is captured.
6. The Compliance Cockpit's "P2 → P3 status" flips green; downstream P3 modules can begin (multi-tenancy + DSAR external + DOC P3 + GDPR + ISO/SOC audits).

**SOC 2 Type I evidence map.**

A subset of `obs.gate_criterion` rows tagged with the SOC 2 Common Criteria they map to:
- CC1: Control Environment.
- CC2: Communication & Information.
- CC3: Risk Assessment.
- CC4: Monitoring Activities.
- CC5: Control Activities.
- CC6: Logical Access (mapped to FR-AUTH + FR-MCP + FR-HR_SECURE controls).
- CC7: System Operations (mapped to FR-OBS + FR-CP).
- CC8: Change Management (mapped to FR-INFRA-001's CI/CD + FR-REW-001's parameter-version flow).
- CC9: Risk Mitigation.

Each Common Criterion has 5-10 underlying criteria across the platform; the dashboard rolls up.

**ISO/IEC 27001 gap-list.**

A separate `obs.iso27001_gap_list` table mapping each Annex A control (114 controls) to the platform's implementation status:

```sql
CREATE TABLE obs.iso27001_gap (
  tenant_id UUID NOT NULL,
  control_id TEXT NOT NULL,                                              -- "A.5.1.1", "A.6.2.1", etc.
  control_title TEXT NOT NULL,
  status TEXT NOT NULL,                                                  -- "implemented" | "in_progress" | "not_implemented" | "not_applicable"
  implementation_evidence_blob_ids UUID[],
  notes_md TEXT,
  reviewed_by UUID,
  reviewed_at TIMESTAMPTZ,
  PRIMARY KEY (tenant_id, control_id)
);
```

The 85% threshold = 97 of 114 controls in `implemented` or `not_applicable` status.

**MCP tool surface (read-only).**

- `cyberos.obs.gate_status(gate_kind)` — read; HR/Ops + Founder + DPO + Auditor.
- `cyberos.obs.list_nfr_measurements(since?)` — read.
- `cyberos.obs.iso27001_gap_summary` — read.
- `cyberos.obs.draft_phase_exit_rfc(gate_kind)` — read; CUO/CAIO drafts.

There are no mutation MCP tools for gate criteria — sign + publish goes through Founder UI + step-up + multi-party-sign.

## Alternatives Considered

- **Manual checklist tracking in a Notion page.** Rejected: status drifts; no audit trail.
- **Skip the phase-exit RFC; treat the Compliance Cockpit's green as sufficient.** Rejected: the RFC is the founder's structural attestation; auditors expect it.
- **Auto-flip P2 → P3 status when all criteria green.** Rejected: the founder's sign is the floor.
- **Skip ISO 27001 gap-list at P2; defer to P3.** Rejected: PRD §14.3.2 explicitly requires ≥85% gap-list at P2 → P3.

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate is *passed*: every criterion green or yellow-with-mitigation; founder + Engineering Lead + DPO sign the Phase-Exit RFC; the audit-log entry is recorded.
- **Evidence completeness.** ISO/IEC 27001 gap-list ≥ 85%; SOC 2 Type I evidence map populated to the auditor's pre-engagement level; PDPL Decree 13 full-regime certificate filed.
- **Latency NFR.** Daily evaluator job p95 ≤ 10 minutes for the full criterion + NFR set.

## Scope

**In-scope.**
- The 3 schema additions (`gate_criterion`, `nfr_measurement`, `phase_exit_rfc`, `iso27001_gap`).
- P2 → P3 criterion catalogue seed (10 criteria).
- ISO/IEC 27001 gap-list seed (all 114 Annex A controls).
- SOC 2 Type I criterion-mapping seed.
- Daily evaluator job.
- Gate-readiness dashboard.
- Phase-exit RFC sign + publish flow.
- The 4 read-only MCP tools.
- Audit integration in scope `obs.gate.{tenant}`.

**Out-of-scope (deferred).**
- P3 → P4 gate criteria (P3 sets them up before its end).
- Automated SOC 2 Type II evidence pipeline (P3 — Type II requires 6+ months observation).
- Multi-tenant gate dashboards (P3+ — when external tenants).
- Auto-mitigation suggestion (P3 — CAIO surfaces actionable items but never auto-resolves).

## Dependencies

- FR-OBS-001 / FR-OBS-002 (skeleton + per-module dashboards).
- FR-CP-001 / FR-CP-002 / FR-CP-003 (Compliance Cockpit + DPIA + DSAR + Decree 13 graduation).
- All P2 module FRs (each criterion's evidence sources).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-GENIE-004 (CAIO drafts the Phase-Exit RFC).
- External auditor selection for SOC 2 Type I + ISO 27001 Stage 1 (typically engaged 6 months before the planned P3 → P4 audit; for P2 → P3 the gate criterion is "evidence map populated" + "gap-list ≥ 85%", not "audit completed").
- Compliance: PDPL Decree 13 + ISO/IEC 27001 + SOC 2 Trust Service Criteria; EU AI Act Articles 5-7 (high-risk evidence map for REW + ESOP + LEARN drives the evidence accumulation).
- Locked decisions referenced: DEC-253 (per-gate criterion catalogue + measurement-query pattern), DEC-254 (founder + Engineering Lead + DPO three-party sign on P2 → P3), DEC-255 (ISO 27001 ≥ 85% as gate floor), DEC-256 (SOC 2 Type I evidence map populated; Type II audit at P3 → P4).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The gate-readiness dashboard + evidence map are deterministic; the Phase-Exit RFC draft uses CAIO (FR-GENIE-004) which inherits its `limited` risk classification, but the RFC's final canonical text is founder-authored.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
