---
title: "CP — ISO/IEC 27001 Stage 1 + Stage 2 audit completion, SOC 2 Type I report publication, audit-firm engagement workflow"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: true
template: feature_request@1
---

# Summary

Complete the **ISO/IEC 27001 Stage 1 + Stage 2 audits** + publish the **SOC 2 Type I report** that PRD §14.4.1 schedules for P3. Stage 1 (documentation review) confirms the platform's ISMS documentation matches the standard; Stage 2 (operational audit) confirms the controls are actually operating effectively. SOC 2 Type I is a point-in-time audit (vs. Type II's 6-month operational period; Type II ships in P4). Deliverables: **audit-firm engagement workflow** (RFP, statement of work, scope sign-off, audit-period definition); **evidence-package generator** producing the auditor's deliverable from FR-OBS-003 + FR-CP-003 + the per-module FR's compliance evidence; **per-control mapping** (114 ISO 27001 Annex A controls + the 5 SOC 2 Trust Service Categories); **audit-finding remediation tracker**; **certificate publication** (ISO 27001 certificate becomes a public artefact at `https://trust.cyberos.world/certificates`); **SOC 2 Type I report** controlled-distribution under NDA. The two audits together unlock the enterprise-customer market (T3 plan tier).

## Customer Quotes

<untrusted_content source="founder_anticipation">
"Enterprise procurement at any of the customers we want to win starts with: send us your SOC 2 + ISO 27001 + DPA. Without those three artefacts, we don't get past the first call. P3 is when these become real, not aspirational." — anticipated by Stephen
</untrusted_content>

# Problem

PRD §14.4.1 P3 scope: "ISO/IEC 27001 Stage 1 audit completed; gaps closed; Stage 2 audit scheduled. SOC 2 Type I report published." PRD §14.4.2 P3 → P4 exit gate: "ISO/IEC 27001 Stage 2 audit completed and certificate issued." Three failure modes the platform must structurally avoid:

- **Audit-evidence chaos.** Without a structured evidence-package generator, the audit firm's request list (typically 200+ documents) takes 2-3 weeks to assemble; audit ROI suffers.
- **Finding-remediation drift.** Audit findings without structured tracking get forgotten; subsequent audits surface the same findings.
- **Certificate distribution opacity.** A signed ISO 27001 certificate buried in an email folder isn't useful to procurement; structured publication is the floor.

# Proposed Solution

**Audit-firm engagement workflow.**

A `cp.audit_engagement` table tracking per-audit lifecycle:

```sql
CREATE TABLE cp.audit_engagement (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  audit_kind TEXT NOT NULL,                                            -- "iso_27001_stage_1" | "iso_27001_stage_2"
                                                                       -- | "iso_27001_surveillance" | "iso_27001_recertification"
                                                                       -- | "soc2_type_1" | "soc2_type_2"
                                                                       -- | "iso_42001"
  audit_firm TEXT NOT NULL,
  audit_firm_lead_auditor TEXT,
  scope_md TEXT NOT NULL,
  audit_period_start DATE NOT NULL,
  audit_period_end DATE NOT NULL,
  status TEXT NOT NULL DEFAULT 'planning',                              -- "planning" | "scoped" | "kick_off"
                                                                       -- | "evidence_collection" | "field_work"
                                                                       -- | "draft_report" | "finalised" | "remediation"
                                                                       -- | "certified" | "expired"
  signed_engagement_letter_doc_id UUID,                                 -- FR-DOC-001 envelope reference
  audit_report_blob_id UUID,
  certificate_blob_id UUID,                                             -- the issued certificate when applicable
  certificate_expires_at DATE,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Evidence-package generator.**

Extending FR-OBS-003's regulator-artefact bundler:

For ISO 27001 Stage 1 audit, the bundle includes:
- The Information Security Policy (FR-CP-001 + FR-CP-003 + this FR).
- The Statement of Applicability (per-control implementation status).
- The Risk Assessment + Risk Treatment Plan (FR-CP-001 + FR-OBS-003).
- Per-Annex-A-control evidence (the FR-OBS-003 ISO 27001 gap-list).
- Per-module DPIAs (FR-CP-003).
- Audit-log Merkle-chain verification report (FR-AUTH-002).
- Internal audit reports (the platform's own quarterly internal audits).

For ISO 27001 Stage 2 audit, additional:
- Operational evidence over the audit period (≥ 90 days of audit-log + observability + incident-management evidence).
- Internal audit + management-review reports for the audit period.
- Corrective-action tracking from any prior surveillance audits.

For SOC 2 Type I:
- The 5 Trust Service Categories (Security + Availability + Processing Integrity + Confidentiality + Privacy) per-criterion evidence.
- Same operational-evidence pattern but point-in-time (≤ 30 days of evidence sufficient).

The bundler is invoked: `cyberos-audit-bundler --audit-id <id> --output-dir <path>` produces a structured directory the audit firm consumes.

**Per-control mapping.**

Three mappings per audit:
- **ISO 27001 Annex A** — 114 controls; mapped to platform FRs that implement them (e.g. A.5.1.1 "Information security policy" → FR-CP-001 + FR-CP-003; A.9.4.1 "Information access restriction" → FR-AUTH-001 RBAC + RLS).
- **SOC 2 Trust Service Categories** — 5 categories × ~10 criteria each; mapped to platform controls.
- **Per-criterion to FR-OBS-003 gate-criterion** linkage so the gate-readiness dashboard flips green when all criteria for the audit are evidenced.

**Audit-finding remediation tracker.**

```sql
CREATE TABLE cp.audit_finding (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  audit_engagement_id UUID NOT NULL REFERENCES cp.audit_engagement(id),
  finding_kind TEXT NOT NULL,                                          -- "non_conformity_major" | "non_conformity_minor"
                                                                       -- | "observation" | "opportunity_for_improvement"
  related_control_id TEXT,                                              -- e.g. "A.9.4.1"
  description_md TEXT NOT NULL,
  severity TEXT NOT NULL,                                               -- "critical" | "major" | "minor" | "advisory"
  status TEXT NOT NULL DEFAULT 'open',                                  -- "open" | "in_remediation" | "verified" | "accepted_risk"
  remediation_plan_md TEXT,
  remediation_target_date DATE,
  remediation_completed_at DATE,
  remediation_evidence_blob_id UUID,
  signed_off_by_lead_auditor_at TIMESTAMPTZ,
  signed_off_by_dpo_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

Findings are surfaced in the Compliance Cockpit with severity-coloured chips; non-conformities (major or minor) block certificate issuance until remediated; observations + opportunities are advisory.

**Certificate publication.**

ISO 27001 certificate published at:
- `https://trust.cyberos.world/certificates` — public-facing list of active certificates per platform.
- Per-tenant Trust Center (FR-TEN-003) when applicable.
- API endpoint for prospective customers: `https://trust.cyberos.world/api/certificates` returns the active set.

The certificate blob is signed by the audit firm; the platform's signed-record proves authenticity. The certificate's expiry date triggers a 90-day-before-expiry Notify to schedule the surveillance/recertification audit.

SOC 2 Type I report is NOT publicly published — it's controlled-distribution under NDA. A `cp.soc2_report_distribution` log tracks who has been granted access. Distribution requires founder + DPO sign + the recipient's NDA acceptance.

**Frontend additions.**

`/compliance/audits` (HR/Ops + Founder + DPO + Auditor):
- Audit-engagement timeline (past + active + planned).
- Per-audit detail with status, evidence-package link, findings tracker.
- Certificate library.
- SOC 2 distribution log.
- ISO 27001 surveillance-audit cadence reminder.

**MCP tool surface (read-only).**

- `cyberos.cp.list_audit_engagements(audit_kind?, status?)` — read.
- `cyberos.cp.get_audit_engagement(id)` — read.
- `cyberos.cp.list_audit_findings(audit_id, status?, severity?)` — read.
- `cyberos.cp.list_certificates(active_only?)` — read; everyone (certificates are public).
- `cyberos.cp.generate_evidence_package(audit_id)` — read; HR/Ops + Founder + DPO; runs the bundler.

# Alternatives Considered

- **Use a hosted compliance-evidence platform (Vanta, Drata, Secureframe).** Considered + accepted as a complementary tool that reads the platform's audit + control data. The canonical evidence remains in CyberOS for residency.
- **Skip ISO 27001 in P3; only SOC 2 Type I.** Rejected: enterprise customers in EU + Asia + regulated US industries expect both.
- **Single audit firm for ISO + SOC 2.** Considered + accepted; firms like A-LIGN + Schellman + Bishop Fox can do both.
- **Skip surveillance audit cadence; treat each audit as one-off.** Rejected: ISO 27001 requires annual surveillance + 3-yearly recertification; tracking the cadence is the floor.

# Sales/CS Summary

CyberOS reaches enterprise audit posture in P3: ISO/IEC 27001 certification (the global information security standard) + SOC 2 Type I (point-in-time control attestation). Both are real audits by independent firms, not self-attestation. The certificates are publicly verifiable; the SOC 2 report is shareable under NDA. Combined with our ongoing GDPR + PDPL + EU AI Act evidence maps, the platform satisfies the standard procurement-due-diligence question set for any customer up to mid-market enterprise.

# Success Metrics

- **Primary metric.** P3 → P4 exit-gate (PRD §14.4.2): "ISO/IEC 27001 Stage 2 audit completed and certificate issued." SOC 2 Type I report published + distributed to the first 2-3 prospective enterprise customers under NDA.
- **Audit timeline.** Stage 1 + Stage 2 + remediation-period + certificate issuance ≤ 6 months from audit-firm RFP signing.
- **Finding rate.** Major non-conformities ≤ 0; minor ≤ 5 (industry typical for first-time audits).

# Scope

**In-scope.**
- `cp.audit_engagement` + `cp.audit_finding` schemas.
- Audit-firm RFP + scope-of-work template envelope (FR-DOC-001 reused).
- Evidence-package generator (extending FR-OBS-003 bundler).
- Per-control mapping for ISO 27001 Annex A + SOC 2 Trust Service Categories.
- Finding remediation tracker.
- Certificate publication at `trust.cyberos.world/certificates`.
- SOC 2 Type I distribution log.
- The 5 read-only MCP tools.
- `/compliance/audits` frontend surface.
- Audit integration in scope `cp.audits.{tenant}`.

**Out-of-scope (deferred).**
- SOC 2 Type II (P4 — requires 6-month operational period after Type I).
- ISO/IEC 42001 (AI Management System) (P4+ — voluntary; alongside or after ISO 27001).
- Industry-specific audits (HIPAA, PCI DSS Level 1) (P4+ — per customer demand).
- Auto-generated Statement of Applicability (P4 — currently DPO-authored).

# Dependencies

- FR-CP-001 / FR-CP-002 / FR-CP-003 / FR-CP-004.
- FR-OBS-001 / FR-OBS-002 / FR-OBS-003 (gap-list + evidence map).
- FR-AUTH-001 / FR-AUTH-002 (audit chain).
- FR-DOC-001 (audit-engagement-letter envelopes).
- FR-TEN-003 (per-tenant Trust Center for SOC 2 Type II in P4).
- Audit firm engaged (typically 6 months pre-Stage-1).
- External legal counsel for SOC 2 NDA + audit-firm engagement-letter review.
- Compliance: ISO/IEC 27001:2022 + ISO/IEC 27002:2022 (controls catalogue) + AICPA SOC 2 Trust Services Criteria (TSP 100); EU AI Act + GDPR alignment; per-customer-region procurement requirements.
- Locked decisions referenced: DEC-279 (ISO 27001 + SOC 2 Type I in P3; Type II in P4), DEC-280 (certificates publicly published; SOC 2 reports NDA-distributed), DEC-281 (audit-finding remediation tracked + audit-row preserved).

# AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The audit workflow is deterministic; the evidence pipeline reads from existing platform data. The audit firm's evaluation of AI-related controls (FR-CAIO + FR-CP-003 EU AI Act evidence) is part of the audit; the platform doesn't introduce new AI surfaces here.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
