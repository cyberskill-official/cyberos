---
title: "Compliance Plane — Decree 13 full-regime graduation: formal DPO, full DPIA library, DSAR portal, regulator-ready artefacts"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Graduate the Compliance Plane from FR-CP-001's P0 skeleton to **Decree 13 full-regime status** as required by PRD §14.3.1 P2 scope ("Decree 13 full-regime graduation: formal DPO appointment, formal DPIA template populated for HR/REW/CRM/CHAT/EMAIL"). Ship the **formal DPO appointment** with documented role + tenant-recorded responsibility; the **full DPIA library** — DPIAs for every P2 module that processes personal data (HR-001, HR-002, HR-003, REW-001 through REW-007, LEARN-001 through LEARN-004, ESOP-001 through ESOP-003, OKR-001 through OKR-003, INV-001 through INV-004, RES-001 through RES-003) extending the 6 P0 DPIAs from FR-CP-001; the **DSAR portal** (data-subject-access-request) for Members + (P3-stub) external-subject access; **right-to-erasure (RTBE) workflow extensions** beyond FR-CP-002's synthetic-tenant drill — production-ready Member-self-service erasure-with-statutory-floors; **regulator-ready artefact bundle** generator; and **A05 filing automation** completion (the FR-CP-001 stub becomes a fully-filled-and-filed real submission with the Vietnamese Ministry of Public Security).

## Problem

P0 shipped the CP skeleton + 6 P0 DPIAs + an A05 filing draft (FR-CP-001) + a synthetic RTBE drill (FR-CP-002). P2's added scope (HR + REW + LEARN + ESOP + OKR + INV + RES) processes substantively more personal data — including high-sensitivity comp + identity data through `hr_secure`. PRD §14.3.1 makes Decree 13 full-regime graduation the precondition for the P2 → P3 exit gate. Three failure modes the platform must avoid:

- **Missing DPIA on a high-risk activity.** Each new P2 module's processing activity needs its own signed DPIA filed with the DPO; without the DPIA, Decree 13 compliance is incomplete.
- **DSAR opacity.** A Member or external subject requesting access to their data needs a clear self-service portal; without it, requests come via email + take days.
- **A05 filing not actually filed.** The P0 draft sat as a placeholder; before P2 can process production tenant data at scale, the filing must be real + acknowledged.

## Proposed Solution

The shape of the answer is `cp.*` schema extensions + the DPO-appointment workflow + the DPIA library expansion + the DSAR portal + the regulator-artefact bundler + the A05 filing finalisation.

**Schema extensions (`cp` module).**

```sql
-- Formal DPO appointment record.
CREATE TABLE cp.dpo_appointment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  appointed_member_id UUID NOT NULL REFERENCES auth.member(id),
  appointment_date DATE NOT NULL,
  appointment_basis_md TEXT NOT NULL,                                 -- the legal-document trail
  contact_email TEXT NOT NULL,                                         -- public-facing DPO contact
  contact_phone TEXT,
  responsibilities_md TEXT NOT NULL,                                    -- explicit scope of DPO role
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_appointee_at TIMESTAMPTZ NOT NULL,
  filed_with_authority_ref TEXT,                                        -- MPS reference once filed
  filed_at DATE,
  active BOOLEAN NOT NULL DEFAULT true,
  superseded_by UUID REFERENCES cp.dpo_appointment(id),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- DSAR request log (extending CP-002's synthetic-tenant pattern to production).
CREATE TABLE cp.dsar_request (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  request_kind TEXT NOT NULL,                                          -- "member_self" | "external_subject" | "regulator_request"
  subject_member_id UUID,                                               -- when request_kind = "member_self"
  subject_external_email TEXT,                                          -- when request_kind = "external_subject"
  subject_external_proof_blob_id UUID,                                  -- identity-verification document
  request_scope TEXT NOT NULL,                                          -- "access_only" | "erasure" | "rectification" | "objection"
  request_md TEXT NOT NULL,                                             -- the subject's free-text request
  legal_basis_assessment_md TEXT,                                       -- DPO's assessment of grounds
  status TEXT NOT NULL DEFAULT 'received',                              -- "received" | "verified" | "in_progress"
                                                                       -- | "fulfilled" | "denied" | "escalated_to_authority"
  due_at TIMESTAMPTZ NOT NULL,                                           -- 30-day SLA per Decree 13 + GDPR
  fulfilled_at TIMESTAMPTZ,
  fulfillment_artefact_blob_id UUID,                                    -- the export bundle
  denial_reason_md TEXT,
  signed_off_by_dpo_at TIMESTAMPTZ,
  audit_trail JSONB NOT NULL DEFAULT '[]'::jsonb,                       -- every action on this request
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX dsar_status_due_idx ON cp.dsar_request (tenant_id, status, due_at);

-- Regulator-ready artefact bundle (a snapshot for handing to an auditor or regulator).
CREATE TABLE cp.regulator_artefact_bundle (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  audience TEXT NOT NULL,                                              -- "vn_mps" | "external_audit_iso27001"
                                                                       -- | "external_audit_soc2" | "regulator_inquiry"
  generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  generated_by UUID NOT NULL,
  bundle_blob_id UUID NOT NULL,                                         -- the signed zip in encrypted blob store
  contains JSONB NOT NULL,                                              -- structured manifest of what's in the bundle:
                                                                       -- { dpo_appointment: ..., dpias: [...], dsar_log: [...],
                                                                       --   audit_chain_summary: ..., parameter_versions: [...],
                                                                       --   incident_log: [...], training_records: [...] }
  expires_at TIMESTAMPTZ,                                               -- bundles older than 90 days are auto-revoked
  delivered_to TEXT,                                                    -- the receiving party reference
  delivered_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

**Formal DPO appointment.**

The DPO role transitions from "founder also fills" (P0 conflated state per FR-CP-001) to a formally-appointed Member:

1. The Founder appoints a Member as DPO via `/compliance/admin/dpo-appoint` (HR/Ops Lead + Founder + the appointee co-sign).
2. The appointee's role responsibilities are documented (per Decree 13's required DPO duties: oversee compliance, advise on processing, liaise with regulator, monitor DPIAs).
3. The appointment is filed with MPS as part of the A05 filing finalisation.
4. The appointee gets the `auth.role: 'DPO'` predicate (FR-AUTH-001's role catalogue).
5. The previous "founder-fills-DPO" record is archived with `superseded_by: <new-appointment-id>`.

When the DPO leaves the role (resignation, termination), a transition flow ensures continuity:
- A new DPO must be appointed before the prior DPO's `appointment_active_until` date.
- The transition is documented + filed.
- DSAR + RTBE + DPIA work-in-flight transfers to the new DPO.

**DPIA library expansion.**

Adding to the 6 P0 DPIAs (BRAIN, GENIE/CUO, CHAT, AUTH, AI Gateway, MCP from FR-CP-001), new DPIAs for every P1 + P2 module:

| Module | DPIA scope |
|---|---|
| EMAIL | mail content + headers + attachments + S/MIME + CaMeL processing |
| PROJ | issue + comment + activity log content |
| KB | block-level personal-data scanning + Yjs collaborative-editing |
| TIME | time entries + leave + expense + receipt OCR |
| CRM | account + contact + activity + signal data |
| HR | employee + contract + role-history + statutory profile |
| HR_SECURE | identity + national-ID + bank-account |
| REW + REW_SECURE | salary + BP fund + payslips + PIT |
| LEARN | VP scores + 360 syntheses + Council deliberations |
| ESOP | grants + vesting + valuations + put requests |
| OKR | objectives + check-ins (mostly low-personal-data; included for completeness) |
| INV + INV_SECURE | invoice + payment + vendor banking |
| RES | allocations + capacity + skill profiles |

Each DPIA follows the FR-CP-001 template structure: processing activity description, lawful basis, data classes, data subjects, recipients, retention, technical controls, organisational controls, transfer assessment, risk assessment, residual risk, mitigation actions, sign-offs (DPO + Founder).

The full DPIA library (P0 6 + P1/P2 13 = ~19 DPIAs total) is reviewed by the company's external legal counsel before P2 production rollout.

**DSAR portal.**

Two surfaces:

1. **Member-self DSAR** at `/auth/account/dsar`.
   - Member submits: scope (access / erasure / rectification / objection) + free-text request.
   - Identity verification: passkey + step-up — the calling Member is the verified subject.
   - 30-day SLA; status-tracking inline; the DPO's progress visible.
   - On fulfilment: a structured export bundle (signed zip; same pattern as FR-BRAIN-001's `.zip` export) with all per-Member data across modules.
2. **External-subject DSAR** at `https://privacy.cyberos.world/dsar/{tenant-slug}` (P3 readiness; P2 ships the Member-self surface; external surface is stub-only).

The DSAR enumerator service walks every per-Member data class:
- AUTH (sessions, audit rows touching this Member).
- BRAIN (Layer 1 + 2 + 3 referencing the Member).
- CHAT (messages authored).
- EMAIL (messages sent + received).
- PROJ (issues + comments authored / assigned).
- KB (pages authored + comments).
- HR (employee + contract + role-history + statutory profile + onboarding records).
- HR_SECURE (identity + bank).
- REW (salary + BP earnings + payslips).
- LEARN (VP outcomes + Council case as subject).
- ESOP (grants + vesting + put requests).
- OKR (objectives owned + check-ins).
- INV (invoices touching this Member; assets assigned).
- RES (allocations + skill assignments).
- TIME (entries + leave + expenses).
- CRM (when the Member is also a contact).

The export bundle is a zip per category with structured JSON + a Markdown summary.

**RTBE production-ready extension.**

The synthetic drill from FR-CP-002 is extended to support real Member-self erasure:
- The Member submits an erasure request via DSAR.
- Statutory-retention floors are honoured: audit-log rows preserved (PDPL Decree 13 + Vietnamese accounting law require 10 years for compensation-related; 7 years for general); per-row PII pseudonymised but the row preserved.
- Audit trail is comprehensive; the DPO + founder sign every erasure.
- A "certificate of erasure" PDF is produced for the Member's records.

**Regulator-artefact bundler.**

`cp.regulator_artefact_bundle` is a snapshot generator: given an audience (MPS / ISO 27001 audit / SOC 2 audit / regulator inquiry), it bundles:
- DPO appointment record.
- Active DPIAs.
- DSAR log (aggregate counts; never per-subject content unless the regulator's request specifies).
- Audit chain summary (Merkle hash chain head + sample verification).
- Active parameter versions (for legal-counsel review of REW + ESOP + LEARN parameters).
- Incident log (sev-0 + sev-1 incidents).
- Training records (per-Member compliance-training completion from FR-LEARN-003).
- Cluster of attestations (SOC 2 evidence map progress; ISO 27001 control coverage).

The bundle is signed (Ed25519 with the tenant's compliance-signing key) + encrypted at rest in the content-addressed blob store. Delivery is via a one-time download link (auth-gated; expires in 7 days).

**A05 filing finalisation.**

The P0 draft becomes a real filing:
- The DPO + Founder + Engineering Lead complete the FR-CP-001 A05 template against the now-full DPIA library.
- Vietnamese legal counsel reviews + signs.
- The HR/Ops Lead (or DPO) submits via the MPS portal manually (no public API).
- The acknowledgement reference is recorded in `cp.a05_filing.filing_reference`.
- The Compliance Cockpit's "PDPL-D13-Full-Regime" status flips to green.

**Compliance Cockpit graduation.**

The Cockpit (FR-OBS-001 + FR-CP-001) gets a new "Decree 13 Full Regime" panel that surfaces:
- DPO appointment status.
- DPIA coverage (X of Y modules covered + signed).
- DSAR queue (open + overdue counts).
- A05 filing status.
- Regulator-artefact bundle history.

Status flips from yellow (P0 partial) to green when all the criteria are met.

**Audit integration.** `cp.full_regime.{tenant}` audit scope.

**MCP tool surface (read-only; very narrow).**

- `cyberos.cp.dpo_status` — read; everyone (the DPO is publicly-knowable role).
- `cyberos.cp.list_dpias` — read; HR/Ops + Founder + DPO + Auditor.
- `cyberos.cp.list_my_dsars` — read; calling Member.
- `cyberos.cp.full_regime_status` — read; the cockpit panel data.

There are no mutation MCP tools — DPO appointment + DPIA sign + DSAR fulfilment are UI + step-up + multi-party-sign only.

## Alternatives Considered

- **Skip DPIA expansion; reuse the P0 6.** Rejected: each new module's processing activity is structurally distinct; Decree 13 compliance is per-activity.
- **Skip the regulator-artefact bundler; assemble manually on request.** Rejected: an audit happens; the bundle should be one click.
- **AI-author DPIAs.** Considered. P2 ships human-authored DPIAs (the DPO writes; legal counsel reviews); a P3 FR may add a CUO-drafted-then-reviewed pattern.
- **External-subject DSAR portal in P2.** Deferred to P3; P2 ships Member-self only.

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate: PDPL Decree 13 full-regime graduation signed off by Founder + DPO; A05 filing acknowledged by MPS; ISO/IEC 27001 gap-list ≥ 85% complete (PRD §14.3.2).
- **DPIA coverage.** 100% of P1 + P2 modules processing personal data have a signed DPIA before P2 → P3 cutover.
- **DSAR latency.** Median fulfilment time ≤ 14 days (well under 30-day SLA).
- **Audit readiness.** A regulator-artefact bundle generation is < 5 minutes; the bundle passes a synthetic external-counsel review.

## Scope

**In-scope.**
- Formal DPO appointment record + transition flow.
- 13 new DPIAs (one per P1/P2 module-cluster) with full sign chain.
- Member-self DSAR portal at `/auth/account/dsar`.
- Production-ready RTBE flow with statutory retention floors.
- Regulator-artefact bundle generator.
- A05 filing finalisation + MPS submission.
- Compliance Cockpit "Decree 13 Full Regime" panel.
- The 4 read-only MCP tools.
- Audit integration in scope `cp.full_regime.{tenant}`.

**Out-of-scope (deferred).**
- External-subject DSAR portal (P3).
- AI-drafted DPIAs (P3).
- Automated MPS portal submission (P3 if Vietnamese authorities publish APIs).
- Cross-jurisdictional DPIA mapping (P3+ — when international expansion).

## Dependencies

- FR-CP-001 / FR-CP-002 (skeleton + synthetic drill).
- All P1 + P2 module FRs (DPIA targets).
- FR-AUTH-001 (DPO role + step-up).
- FR-INFRA-001 / FR-OBS-001 / FR-OBS-002.
- FR-EMAIL-001 (DSAR notification path).
- External Vietnamese legal counsel for DPIA + A05 review.
- The Vietnamese-licensed accountant for A05 + tax-compliance review.
- Compliance: PDPL Decree 13 (full regime); Decree 53/2022 + 356/2025 (Cybersecurity Law); EU AI Act Articles 5-7 (high-risk evidence map for REW + ESOP + LEARN); GDPR (P3 readiness); SOC 2 + ISO 27001 + ISO 42001 (P3-P4).
- Locked decisions referenced: DEC-247 (formal DPO required by P2 → P3 cutover), DEC-248 (per-module DPIA coverage 100% required), DEC-249 (Member-self DSAR portal in P2; external-subject in P3).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The compliance plane is workflow + evidence storage; no AI in the path.
