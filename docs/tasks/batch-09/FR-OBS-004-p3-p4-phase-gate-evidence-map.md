---
title: "OBS — P3 → P4 phase-gate evidence map; first external client run + multi-tenant load test + ISO 27001 Stage 2 + SaaS-readiness sign-off"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2028-Q1"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the **P3 → P4 phase-gate evidence map** that tracks every PRD §14.4.2 P3 → P4 exit-gate criterion + accumulates the audit-ready evidence required to flip CyberOS from "internal platform that survives external scrutiny" to "publicly-orderable SaaS product". Eight surfaces: **gate-readiness dashboard** showing each criterion's status (green/yellow/red); **first-external-client tenant-health panel** (30-day rolling pilot with NPS ≥ 8); **multi-tenant load-test evidence** (100 tenants × 10 users × 1,000 BRAIN ops/day with all PRD §11.2 SLOs green); **ISO/IEC 27001 Stage 2 certificate** + auditor-issued report (consumed from FR-CP-005); **SOC 2 Type II readiness panel** (12-month evidence-collection scaffolding for the Type II audit that will close in P4); **Singapore HoldCo flip closure evidence** (consumed from FR-CORP-001); **CUO unaided state-of-business deliverable** (one quarterly board-grade report drafted by the CUO with founder edits ≤ 10% of word count, attested); **founder + engineering lead + DPO + external auditor sign-chain** producing the Phase-Exit RFC. Extends FR-OBS-001 + FR-OBS-002 + FR-OBS-003 with the P3 → P4 specifics. The dashboard is the founder's primary surface during the SaaS-launch window; the evidence bundle is what the first three external prospects + their security-review teams consume during procurement.

## Problem

PRD §14.4.2 P3 → P4 exit gate is the most consequential gate in the 24-month plan: it converts CyberOS from an internal product into a publicly-orderable SaaS. The PRD specifies seven concrete criteria:

1. **First external client run.** "First external client has provisioned a tenant via the FR-TEN-002 self-service flow, used CyberOS for at least 30 consecutive calendar days, and reported NPS ≥ 8 in the in-product survey or in a written attestation to the Founder."
2. **Multi-tenant load-test green.** "Synthetic load test passes: 100 tenants × 10 users × 1,000 BRAIN ops/day sustained for 7 consecutive days with all PRD §11.2 NFRs green and zero cross-tenant data leaks (proven by FR-TEN-001 invariant tests + FR-AUTH-002 audit-chain analysis)."
3. **ISO/IEC 27001 Stage 2 closed.** "ISO/IEC 27001 Stage 2 audit completed by the chosen Conformity Assessment Body; certificate issued; non-conformities are limited to minor with documented remediation plans."
4. **SOC 2 Type II evidence-collection scaffolding ready.** "SOC 2 Type II auditor pre-engaged; evidence-collection automation produces 12-month auditable trail starting at P3 entry; first 3 months of evidence collected and reviewed."
5. **Singapore HoldCo flip closed.** "Singapore PTE HoldCo is fully operational as the top-of-stack legal entity; JSC OpCo subsidiary structure is in place; first international tenant has been billed by PTE HoldCo against its USD invoice with successful payment; transfer-pricing documentation is filed with both VN tax authority + IRAS."
6. **CUO state-of-business report.** "CUO/CEO produces ≥ 1 quarterly state-of-business report substantially unaided (Founder edits ≤ 10% of word count by character count); the report is read-only AI output (decisions remain human); the Founder attests under FR-AUTH-003 step-up that the report represents the company's state accurately."
7. **Founder + Engineering Lead + DPO + first-external-client signing.** "All four sign the Phase-Exit RFC; the audit-log entry includes the 22-month milestone-arc check (Part 1.4 of PRD)."

Three failure modes if this is not encoded as structured evidence:

- **First-external-client risk.** Without a 30-day rolling tenant-health panel that fuses NPS, error budget burn, support-ticket volume, and CUO/Genie acceptance rate, the founder discovers the pilot has failed only after the customer churns. By then the gate has slipped 2-3 months.
- **Load-test inconclusive results.** Without a structured load-test rig that runs against a non-production shard with synthetic tenants seeded from a deterministic faker, "did the multi-tenant test pass?" depends on an engineer's recollection. The PRD §11.2 NFRs become unverifiable.
- **CUO unaided-report failure mode.** Without a structured authorship-attribution captured at draft + edit + sign time, "did the founder edit ≤ 10% of word count?" is unmeasurable. The criterion silently degrades into "founder wrote it themselves and the CUO assisted" — which is the inverse of the PRD intent ("the CUO can describe the company's state by Month 22").

## Proposed Solution

The shape of the answer is a `obs.gate_*` extension (re-using FR-OBS-003's schema) + a four-surface dashboard + new evidence-map auto-collectors specific to P3 → P4 + the founder + DPO + external-auditor + first-external-client sign-and-publish flow.

**Schema extensions (additive on top of FR-OBS-003).**

```sql
-- First external client tenant-health rollup, evaluated daily.
CREATE TABLE obs.external_pilot_health (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,                                                  -- the external pilot tenant (e.g. "acme-co")
  evaluation_date DATE NOT NULL,
  pilot_day INT NOT NULL,                                                   -- 1..30+ (day of pilot)
  nps_score NUMERIC,                                                        -- in-product survey
  nps_response_count INT,
  cuo_acceptance_rate NUMERIC,                                              -- rolling-7-day
  error_budget_burn NUMERIC,                                                -- % of monthly error budget consumed
  support_tickets_opened INT,
  support_tickets_resolved INT,
  brain_ops_count INT,
  active_users_count INT,
  modules_active TEXT[],                                                     -- which modules the pilot has actually used
  written_attestation_blob_id UUID,                                          -- if NPS was a written attestation
  status TEXT NOT NULL,                                                      -- "healthy" | "watch" | "at_risk" | "failed"
  notes_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, evaluation_date)
);

-- Multi-tenant load-test runs (each row = one full 7-day soak).
CREATE TABLE obs.multi_tenant_load_test (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  test_run_label TEXT NOT NULL UNIQUE,                                       -- e.g. "p3-gate-soak-2027-12-15"
  started_at TIMESTAMPTZ NOT NULL,
  ended_at TIMESTAMPTZ,
  shard TEXT NOT NULL,                                                        -- which shard the test ran on (typically a dedicated load-test shard)
  config_blob_id UUID NOT NULL,                                               -- the parameter file: 100 tenants × 10 users × 1,000 BRAIN ops/day
  config_hash TEXT NOT NULL,                                                  -- SHA-256 of the config blob (immutable evidence)
  k6_script_blob_id UUID NOT NULL,                                            -- the k6/Locust/custom script
  result_blob_id UUID,                                                        -- the full Prometheus snapshot at end-of-test
  nfr_results JSONB,                                                          -- per-NFR pass/fail
  cross_tenant_leakage_detected BOOLEAN,                                      -- always must be false to pass
  audit_chain_anomalies INT,                                                  -- count of anomalies detected by FR-AUTH-002 audit-chain analysis
  status TEXT NOT NULL,                                                       -- "running" | "passed" | "failed" | "inconclusive"
  signed_off_by_engineering_lead_at TIMESTAMPTZ,
  signed_off_by_dpo_at TIMESTAMPTZ,                                           -- DPO signs off on cross-tenant leakage attestation
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- CUO unaided-report authorship attribution.
CREATE TABLE obs.cuo_unaided_report (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  report_quarter TEXT NOT NULL,                                              -- "2027-Q4"
  cuo_draft_blob_id UUID NOT NULL,                                           -- the original CUO output, immutable
  cuo_draft_word_count INT NOT NULL,
  cuo_draft_char_count INT NOT NULL,
  cuo_persona_version TEXT NOT NULL,                                         -- which persona signed the draft (FR-GENIE-001)
  cuo_skill_version_chain TEXT[] NOT NULL,                                    -- e.g. ["ceo@2.3.1", "cfo@1.4.2"]
  cuo_genie_session_id UUID,                                                  -- the LangSmith trace for transparency
  founder_final_blob_id UUID NOT NULL,                                        -- the final published report
  founder_final_word_count INT NOT NULL,
  founder_final_char_count INT NOT NULL,
  founder_edit_pct_chars NUMERIC NOT NULL,                                    -- (final - draft) / draft × 100, by chars
  founder_edit_pct_words NUMERIC NOT NULL,                                    -- by words
  founder_attestation_md TEXT NOT NULL,                                       -- "I attest this report represents the company's state accurately"
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  step_up_auth_method TEXT NOT NULL,                                          -- "passkey" | "totp"
  audit_log_entry_id UUID NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  CHECK (founder_edit_pct_chars <= 10),                                       -- the criterion threshold, enforced at insert
  CHECK (founder_edit_pct_words <= 10)
);
```

**P3 → P4 criterion catalogue (seed).**

These are inserted into `obs.gate_criterion` (from FR-OBS-003) for `gate_kind: "p3_to_p4"` at P3 mid-cycle (typically Month 19 of the 24-month plan, i.e. Month 7 of P3).

| Code | Description | Threshold | Measurement |
|---|---|---|---|
| `external_pilot_30day_complete` | First external client tenant has been live ≥ 30 calendar days | `count_days >= 30` | SQL: `SELECT MAX(pilot_day) FROM obs.external_pilot_health WHERE tenant_id = $1` |
| `external_pilot_nps_8` | First external client NPS ≥ 8 by in-product survey or written attestation | `nps >= 8` | SQL: latest `nps_score` for the pilot tenant |
| `multi_tenant_load_test_passed` | 100 × 10 × 1,000 sustained 7 days, all NFRs green, zero leakage | `status = 'passed' AND cross_tenant_leakage_detected = false AND audit_chain_anomalies = 0` | SQL on `obs.multi_tenant_load_test` |
| `iso_27001_stage_2_certificate_issued` | Certificate issued by Conformity Assessment Body (FR-CP-005 output) | `bool = true` | manual_attestation, evidence_blob_id = the certificate PDF |
| `soc2_type2_evidence_3_months` | SOC 2 Type II auditor pre-engaged, ≥ 3 months of evidence collected + reviewed | `months >= 3 AND auditor_review_completed = true` | manual_attestation referencing `obs.soc2_evidence_log` |
| `singapore_holdco_flip_closed` | PTE HoldCo + JSC OpCo + first international tenant billed in USD by PTE | `bool = true AND first_intl_tenant_billed_at IS NOT NULL` | SQL on `corp.legal_entity` + `corp.tenant_billing_entity` + `bill.invoice` (FR-CORP-001 + FR-BILL-001 outputs) |
| `cuo_unaided_quarterly_report` | CUO produced ≥ 1 quarterly state-of-business report with founder edit ≤ 10% chars | `count >= 1 AND founder_edit_pct_chars <= 10` | SQL on `obs.cuo_unaided_report` |
| `compliance_cockpit_green_p3` | Compliance Cockpit shows green on all P3 regimes (PDPL-D13, PDPL-D53, PDPL-D20, GDPR, EU AIA, SOC 2 Type I + Type II in-progress, ISO 27001) | `all_green = true` | aggregate from FR-CP-001/002/003/004/005 panels |
| `cuo_acceptance_50_rolling` | CUO acceptance rate ≥ 50% rolling-7-days for ≥ 30 days (gate raised from 40% in P2 → P3) | `days_at_threshold >= 30` | Prometheus expression on FR-GENIE-001 metrics |
| `nfr_full_coverage_green` | Every PRD §11.2 NFR is green for ≥ 14 consecutive days at gate-window | `green_streak_days >= 14 AND red_count = 0 AND yellow_count = 0` | aggregate on `obs.nfr_measurement` |
| `founder_gate_signing` | Founder gate-readiness signed | `bool = true` | manual_attestation |
| `engineering_lead_gate_signing` | Engineering Lead gate-readiness signed | `bool = true` | manual_attestation |
| `dpo_gate_signing` | DPO gate-readiness signed | `bool = true` | manual_attestation |
| `external_auditor_gate_signing` | External auditor (ISO 27001 CAB) gate-readiness statement received | `bool = true` | manual_attestation, evidence_blob = the CAB letter |
| `external_client_gate_signing` | First external client written attestation that pilot met expectations | `bool = true` | manual_attestation, evidence_blob = the customer letter |
| `22_month_milestone_arc_check` | The PRD §1.4 milestone-arc audit-log entry at Month 22 | `exists = true` | SQL on `audit.events` (FR-AUTH-002) |

These are seeded at P3 mid-cycle. Each criterion's `measurement_query` is filled per the table above.

**Daily evaluator job (extends FR-OBS-003's job).**

Runs at 06:00 ICT on a dedicated shard-N evaluator (the load-test shard does not back its own evaluator):

1. Per criterion: execute `measurement_query`; update `current_value` + `status`.
2. Per NFR (FR-OBS-003 logic): update `obs.nfr_measurement`.
3. **First external pilot: evaluate `obs.external_pilot_health`** — pull NPS + acceptance + error-budget + ticket counts from FR-PORTAL-001 (P4 emergent shim landed in P3 to support pilot) + FR-GENIE-001 + FR-OBS-002 + FR-CRM-001 ticket queue; compute pilot_day; classify status (healthy / watch / at_risk / failed) per gating thresholds.
4. **Multi-tenant load test:** if a load-test run is in progress, ingest its k6 + Prometheus metrics live; compute cross-tenant-leakage detection by running FR-TEN-001 invariant tests against the load-test tenants every 6 hours; compute audit-chain anomalies by running FR-AUTH-002's external CLI verifier against each tenant's audit chain.
5. Push aggregate status to OBS dashboards; trigger Notify on the Founder + Engineering Lead + DPO if any criterion regresses from green → yellow.

**Multi-tenant load-test rig.**

A new test artefact lives at `infra/load-tests/p3-gate-soak/`:

- `config.yaml` — declarative: 100 tenants, 10 users per tenant, 1,000 BRAIN ops per user per day, distribution of op kinds (PUT 30%, QUERY 50%, UPDATE 15%, DELETE 5%), modules exercised (BRAIN, GENIE, CHAT, EMAIL, PROJ, CRM, KB), test duration (7 days).
- `seed.sh` — deterministic Faker-based tenant + user seeding (each tenant gets a Vietnamese-locale dataset + an English-locale dataset so PGroonga + bge-m3 are exercised).
- `scenarios/` — one k6/Locust scenario per module (`brain-mixed-ops.js`, `genie-skill-invocation.js`, `chat-message-roundtrip.js`, etc.).
- `cross-tenant-invariants/` — pytest harness that runs every 6 hours during the test:
  - For every (tenantA, tenantB) pair, attempt to read tenantB's data via tenantA's credentials. Must always 403/404.
  - For every (tenantA, tenantB) pair, run a malicious BRAIN query crafted to bleed across tenant boundaries. Must always return 0 records.
  - For every persona-scope contract, attempt to invoke a tool-call with the wrong persona-scope. Must always be rejected at AI Gateway, MCP Gateway, AND module server (defence in depth).
  - Detected leakage is a **hard fail** of the load test; the soak terminates.
- `reports/` — at end of soak: full Prometheus snapshot (per-tenant per-module per-NFR), audit-chain verifier output, k6 summary, executive summary.
- The shard used is a dedicated `load-test` shard provisioned for the duration of the test (so production tenants are unaffected). After the soak, the shard is decommissioned.

Test runs are recorded as `obs.multi_tenant_load_test` rows. Engineering Lead + DPO sign off each run.

**CUO unaided-report flow.**

This is the most subtle criterion in the gate. The intent is: "by Month 22, the CUO can describe the company's state as a peer co-worker would, and the founder edits cosmetically not substantively." The flow:

1. **Schedule.** `obs.cuo_unaided_report` is initiated quarterly; a CUO/Genie session is launched at the first business day after quarter close (~Q+10 calendar days).
2. **CUO drafting.** The CUO is invoked in a "state-of-business" mode (a new GENIE skill flag — read-only, all data sources allowed: BRAIN, OKR, INV, REW aggregates, RES allocation, CRM pipeline, OBS metrics). The CUO drafts a multi-section report (financials at a glance, team headcount + turnover, OKR progress, top wins / top risks, market environment, ask-for-the-board). The draft is captured immutably as `cuo_draft_blob_id`; word + char count recorded; persona-version + skill-version chain stamped.
3. **Founder edit.** Founder opens the draft in the standard CyberOS editor (FR-KB-001's block editor). Edits are tracked diff against the immutable original.
4. **Founder finalisation.** Founder presses "Sign and publish":
   - Step-up auth.
   - System computes `founder_edit_pct_chars` + `founder_edit_pct_words` against the immutable original.
   - **CHECK constraint enforced at insert:** if either > 10, the system refuses to record this as an unaided report — the row is rejected, and the report is recorded as `obs.cuo_assisted_report` (a separate, lower-bar table) instead.
   - If ≤ 10%, the row is inserted with the founder attestation md.
   - Audit-log entry written with the diff + persona-version + skill-version + final-text hash.

The criterion `cuo_unaided_quarterly_report` requires `count >= 1` rows in `obs.cuo_unaided_report` (i.e. at least one quarter where the founder's edit was substantive but ≤ 10%).

**Architectural note.** This is the only place in the platform where "the CUO did this substantively without me" is a measurable contract. The PRD §1.4 milestone arc treats this as the qualitative test for whether the CUO is genuinely operating at C-skill level. The data model encodes the test explicitly so it cannot be silently relaxed.

**Gate-readiness dashboard (`/obs/gate-readiness/p3-to-p4`).**

For Founder + Engineering Lead + DPO + External Auditor.

- **Header.** Gate name, target date, days-to-target, current state ("X of 16 criteria green").
- **Criteria grid.** Each criterion as a card; status chip (green/yellow/red); current_value vs. threshold; evidence-blobs link; "view trend" mini-chart; "view raw query" deep-link for transparency.
- **First external pilot panel.** 30-day strip showing pilot_day, NPS trajectory, error-budget burn, support tickets, modules-actively-used; in-product NPS survey response trail; written attestation blob if any; NPS-8 threshold marker.
- **Multi-tenant load-test panel.** Latest run: started_at, current pilot_day of soak, current cross-tenant leakage status (must be ZERO across the 7 days), per-NFR pass/fail grid; "view full Prometheus snapshot" deep-link.
- **NFR coverage panel.** (extends FR-OBS-003): per-NFR roll-up table; green count, yellow count, red count; click to drill into a specific NFR's history; required: zero red, zero yellow at gate-window.
- **Compliance Cockpit deep-link.** Embedded panel with the 8 regime statuses (PDPL-D13, PDPL-D53, PDPL-D20, GDPR Art. 35 + DSAR portal, EU-AIA, SOC 2 Type I + Type II evidence collection in-progress, ISO 27001 Stage 2 certificate issued).
- **Singapore HoldCo flip evidence panel.** From FR-CORP-001 + FR-BILL-001: PTE entity status, JSC subsidiary linked, first-international-tenant invoice ID + USD amount + payment status, transfer-pricing filing status (VN tax authority + IRAS).
- **CUO unaided-report panel.** Latest `obs.cuo_unaided_report` row: quarter, draft word count, founder edit pct chars, founder edit pct words, link to draft + final, founder attestation, persona-version + skill-version chain.
- **External auditor evidence panel.** ISO 27001 Stage 2 certificate PDF; SOC 2 Type II auditor letter of engagement; first 3 months of evidence-collection log.
- **Founder + Engineering Lead + DPO + External Auditor + First External Client sign action.** Disabled until all criteria are green; enables the "Author + Sign Phase-Exit RFC" CTA when ready.

**Phase-exit RFC flow.**

When all 16 criteria are green:

1. Founder navigates to `/obs/gate-readiness/p3-to-p4/sign`.
2. Step-up auth.
3. CUO/CAIO drafts a Phase-Exit RFC narrative (read-only AI; founder edits + finalises):
   - The 22-month milestone-arc check from PRD §1.4 ("the CUO co-runs CyberSkill with the Founder and at least one persona has reached genuine C-skill autonomy in its domain").
   - Each criterion's status with cited evidence-blob IDs.
   - First external client letter quote (NDA-respecting redacted version).
   - Open risks + mitigation plans for P4.
   - Confirmation of all 5 signatures (Founder + Engineering Lead + DPO + External Auditor + First External Client).
   - The 12-week P4 plan kick-off (PORTAL + TEN P4 features + Public APIs + go-to-market).
4. Founder signs (`signed_by_founder_at`); Engineering Lead signs; DPO signs; External Auditor signs (the CAB letter is captured as evidence — the auditor does not directly sign in the platform); First External Client signs (their attestation letter is captured as evidence).
5. The RFC is written as `obs.phase_exit_rfc` row; the audit-log entry's ID is captured.
6. The Compliance Cockpit's "P3 → P4 status" flips green; downstream P4 modules can begin (PORTAL + TEN P4 features + Public APIs + first external sales motion).

**SOC 2 Type II evidence-collection scaffolding.**

This is forward-looking: SOC 2 Type II requires 12 months of operational evidence, which means the collection has to start at P3 entry (Month 18) to have anything ready by P4 close (Month 30+). The criterion only asks for ≥ 3 months at gate window (Month 22), so this scaffold proves the discipline is in place.

A new `audit.soc2_evidence_log` table accumulates daily:
- Access reviews (FR-AUTH-001 user lifecycle).
- Change management (FR-INFRA-001 deployment events; FR-REW-001 parameter version events; FR-GENIE-001 persona-version events).
- Incident response (FR-OBS-002 alerts + post-mortem links).
- Vulnerability management (CVE patch cadence).
- Vendor management (sub-processor change log from FR-CP-004).
- Backup + DR (per-tenant + per-shard).
- Logical access (per-persona + per-tool MCP grants from FR-MCP-001).

The scaffolding does not produce reports — those come at SOC 2 Type II audit close in P4. But the data is collected from Day 1 of P3.

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- The actual SOC 2 Type II audit close (lands in P4; depends on the 12-month evidence trail completing).
- Any further phase gates (this is the last gate before SaaS GA; subsequent gates are minor releases inside P4).
- The first external client's commercial contract negotiation (handled outside the platform; this FR captures the platform-side evidence of the pilot's success).
- The Singapore HoldCo flip itself (FR-CORP-001 owns the flip; this FR consumes its closure evidence).
- The external auditor selection (FR-CP-005 owns the CAB engagement; this FR consumes its certificate).
- Multi-region failover (deferred to post-P4; P3 → P4 gate evaluates within-region SLOs only).

## Dependencies

**Direct (this FR consumes outputs of):**
- FR-OBS-001 — observability skeleton.
- FR-OBS-002 — full SLOs/alerting/per-module dashboards.
- FR-OBS-003 — P2 → P3 phase-gate evidence map (provides the `obs.gate_*` schema reused here).
- FR-AUTH-001 — RBAC + RLS (for cross-tenant invariant testing).
- FR-AUTH-002 — append-only Merkle-chained audit log (for the audit-chain verifier in load-test).
- FR-AUTH-003 — step-up auth (for the founder sign flow).
- FR-MCP-001 — MCP Gateway (for persona-scope contract testing in load-test).
- FR-AI-001 — AI Gateway (for persona-scope contract testing in load-test).
- FR-BRAIN-001/002/003 — three-layer memory (load-test op surface).
- FR-GENIE-001 — CUO base persona (the unaided-report draft surface).
- FR-CP-001/002/003/004/005 — full compliance plane (regime statuses surfaced).
- FR-TEN-001 — full multi-tenancy partitioning (provides the invariant tests).
- FR-TEN-002 — tenant lifecycle (provides the self-service provisioning the external pilot used).
- FR-TEN-003 — per-tenant theming + custom domains (the pilot uses a vanity domain).
- FR-DOC-001 — document signing (the external client's pilot agreement is signed here).
- FR-BILL-001 — subscription billing (proves the pilot is being billed and the first international tenant is billed by PTE in USD).
- FR-CORP-001 — Singapore HoldCo flip (provides the closure evidence).
- FR-PORTAL-001 (P4 shim) — basic external-tenant onboarding surface used by the pilot.

**Locked decisions (PRD §11.1 + SRS Decisions Log):**
- DEC-013 Postgres schema-per-tenant + RLS as floor.
- DEC-014 RBAC + RLS as floor.
- DEC-019..DEC-023 audit log Postgres-native append-only + Merkle hash chain.
- DEC-050..DEC-052 OBS single-pane Loki + Prometheus + Tempo + Trust Center public.
- DEC-XXX (TBD) — load-test shard topology (defer to gap-filling decision in P3).
- DEC-XXX (TBD) — SOC 2 Type II auditor selection.

**External:**
- ISO/IEC 27001 Conformity Assessment Body (CAB) selected at P3 entry per FR-CP-005.
- SOC 2 Type II auditor pre-engaged at P3 entry per FR-CP-005.
- First external client onboarded at P3 mid-cycle (M19-M21 of the 24-month plan).

## Constraints

- **AI never decides whether the gate is passed.** The CUO drafts the Phase-Exit RFC narrative (read-only AI) but the four humans sign. The criteria-evaluation job is deterministic (Prometheus + SQL + manual attestation). No AI judgement enters the gate decision.
- **Load-test shard isolation.** The 7-day soak runs on a dedicated `load-test` shard; production data is never touched. The shard is decommissioned after each soak.
- **Cross-tenant leakage is a hard fail.** If any cross-tenant invariant test fails during the soak, the load-test status is `failed`, and the gate criterion is red until a re-run passes. There is no override.
- **CUO unaided-report threshold is enforced at INSERT, not at UI.** The 10% threshold is a CHECK constraint on `obs.cuo_unaided_report` so it cannot be silently relaxed. Reports above 10% are recorded in `obs.cuo_assisted_report` instead.
- **First external client NPS source is auditable.** Either an in-product survey response (with response_count ≥ 3 to avoid n=1 bias) or a written attestation blob (countersigned by the customer's stakeholder). No verbal attestations.
- **Anti-retroactive on criteria-thresholds.** Once `obs.gate_criterion` rows for `gate_kind = "p3_to_p4"` are inserted at P3 mid-cycle, their thresholds become immutable. Adjustments require a new gate version with audit reason and DPO + Founder co-signing.
- **No private-data exfil into external auditor evidence packages.** FR-CP-005's evidence packager is the only path; all evidence must be sanitised through that pipeline.

## Compliance / Privacy

- **PDPL Decree 13/2023 + 53/2022:** the gate is the moment the platform graduates from "internal product" to "regulated SaaS data processor"; PDPL DPIA must be refreshed (FR-CP-001) and re-signed by Founder + DPO at gate.
- **PDPL Decree 20/2026:** cross-border-transfer exposure increases at P3 → P4 (international tenants on EU/US shards + DPO must sign). This FR surfaces the cross-border posture in the dashboard.
- **GDPR Article 35 DPIA:** the PRD §14.4 P3 GDPR work (FR-CP-004) must be substantially complete; this FR surfaces the DPIA + audit-log of DSAR drills as evidence.
- **EU AI Act Articles 5–7 + 14 + 50:** the CUO's quarterly state-of-business report is a high-risk Article 14 oversight surface (the CUO is a peer co-worker drafting board-grade content); the founder attestation + persona-version + skill-version chain are the audit trail.
- **ISO/IEC 27001 Stage 2:** the certificate is the criterion. FR-CP-005 owns the audit; this FR consumes the certificate.
- **SOC 2 Type II:** scaffolding only; the audit closes in P4.
- **`obs.external_pilot_health` data:** pseudonymised pilot-tenant identifiers in the dashboard for non-Founder/non-DPO viewers; full identifiers in the underlying row for Founder + DPO + Engineering Lead.
- **`obs.cuo_unaided_report`:** the founder attestation md may contain PII or sensitive business strategy; access is restricted to Founder + DPO; audit-log entries are scope-tagged so an external auditor reviewing the SOC 2 audit-chain sees the anonymised verifier output, not the report content.

## Risk Assessment (AI-emitting features)

Three AI surfaces are visible in this FR:

**(a) CUO unaided-report draft (read-only output).**
- **EU AI Act risk class:** limited (not high — read-only, content goes to Founder for substantive edit + sign before any external surface; not used for compensation/equity/hiring/legal-decision-support).
- **Article 50 transparency:** every CUO draft is stamped with persona-version + skill-version chain + LangSmith trace ID; the founder's edit + sign is the human-in-the-loop control.
- **Failure mode:** CUO hallucinates a financial number, founder doesn't catch it, report ships as quarterly board reading. Mitigation: CUO state-of-business skill is constrained to consume *only* aggregate values from REW/INV/OKR (not raw transactions); every numeric assertion in the draft must include the source-table + query-id; founder attestation explicitly references "I attest the report represents the company's state accurately" — i.e. the founder is the accountable party.

**(b) Phase-Exit RFC narrative (read-only AI draft, founder + ELead + DPO sign).**
- **EU AI Act risk class:** limited (read-only).
- **Article 50 transparency:** the RFC explicitly attributes the AI-drafted sections; founder + ELead + DPO co-sign the final.
- **Failure mode:** AI overstates readiness; humans sign without verification. Mitigation: each criterion's evidence-blob ID is auto-inserted from `obs.gate_criterion`; the AI cannot fabricate criterion status — it can only narrate around the deterministic numbers.

**(c) Daily evaluator job's status classification ("healthy" / "watch" / "at_risk" / "failed" for the pilot).**
- **EU AI Act risk class:** not_ai (deterministic threshold rule, no model).
- N/A.

The dashboard itself is `eu_ai_act_risk_class: not_ai` — the gate decision is human + deterministic measurement.

## Vietnamese-locale considerations

- **Founder attestation** for `obs.cuo_unaided_report` may be written in Vietnamese. The audit-log entry stores the original-language text with a locale tag.
- **External auditor** (ISO 27001 CAB + SOC 2 Type II auditor) communicates in English; the auditor evidence-blob may be English; the dashboard presents auditor letters in their original language (no auto-translation, since legal weight depends on the original).
- **First external client letter** may be in Vietnamese (if VN-shard client) or English; preserved verbatim.
- **Phase-Exit RFC narrative** is drafted in English by default (because the audit chain is English-anchored), with an optional Vietnamese summary for the Founder's internal audience.
- **Dashboard typography**: Be Vietnam Pro typography per FR-DESIGN-001; date format is ISO 8601 with vi-VN month names available in tooltip.

## Scope (acceptance criteria — auditable)

**Schema + evaluator.**
- [ ] `obs.external_pilot_health` table exists with the schema above; populated daily by the evaluator job.
- [ ] `obs.multi_tenant_load_test` table exists; populated by load-test runs.
- [ ] `obs.cuo_unaided_report` table exists with the CHECK constraints; insert above 10% threshold is rejected by the database.
- [ ] `obs.cuo_assisted_report` table exists for above-threshold drafts.
- [ ] `obs.gate_criterion` rows for `gate_kind = "p3_to_p4"` are seeded at P3 mid-cycle (M19) with all 16 criteria from the catalogue.
- [ ] Daily evaluator job runs at 06:00 ICT, evaluates all 16 criteria, updates `current_value` + `status`, push to OBS dashboards.

**First external pilot panel.**
- [ ] First external pilot is provisioned via FR-TEN-002's self-service flow, signed via FR-DOC-001, billed via FR-BILL-001, themed via FR-TEN-003.
- [ ] In-product NPS survey is delivered to the pilot's primary contact at days 14, 21, 30.
- [ ] Pilot health row is computed daily for 30+ days.
- [ ] Pilot's NPS reaches ≥ 8 with response_count ≥ 3 OR a written attestation blob is signed.
- [ ] Pilot's CUO acceptance rate ≥ 50% rolling-7-days for ≥ 14 days during the pilot.
- [ ] Pilot's error-budget burn ≤ 100% of monthly budget (i.e. SLOs not breached).

**Multi-tenant load-test rig.**
- [ ] Load-test artefacts at `infra/load-tests/p3-gate-soak/` exist + are CI-tested in dry-run mode.
- [ ] At least 1 successful 7-day soak completes on a dedicated load-test shard.
- [ ] Soak shows zero cross-tenant leakage across all 6-hour invariant test runs.
- [ ] Soak shows zero audit-chain anomalies via FR-AUTH-002 verifier.
- [ ] All PRD §11.2 NFRs are green for the full 7 days at the 100×10×1,000 scale.
- [ ] Engineering Lead + DPO sign off the run.

**ISO 27001 + SOC 2.**
- [ ] FR-CP-005 ISO/IEC 27001 Stage 2 certificate is issued + uploaded as evidence-blob.
- [ ] SOC 2 Type II auditor letter of engagement uploaded.
- [ ] First 3 months of `audit.soc2_evidence_log` collected + auditor-reviewed; review summary uploaded.

**Singapore HoldCo flip.**
- [ ] FR-CORP-001 evidence consumed: PTE + JSC entities active in `corp.legal_entity`; `corp.tenant_billing_entity` populated for ≥ 1 international tenant routed to PTE; ≥ 1 USD invoice issued from PTE + paid via FR-BILL-001 + FR-INV-003.

**CUO unaided-report.**
- [ ] At least 1 row in `obs.cuo_unaided_report` exists with `founder_edit_pct_chars <= 10` AND `founder_edit_pct_words <= 10` AND a non-empty founder attestation md.
- [ ] The persona-version + skill-version chain is recorded.
- [ ] The LangSmith trace is referenced.
- [ ] The audit-log entry exists in `audit.events` for the founder's sign action.

**Compliance Cockpit + NFR coverage.**
- [ ] All 8 P3 regimes show green for ≥ 14 consecutive days at gate-window.
- [ ] All PRD §11.2 NFRs show green for ≥ 14 consecutive days at gate-window.
- [ ] Compliance Cockpit's "P3 → P4 readiness" panel shows green.

**Dashboard + sign flow.**
- [ ] `/obs/gate-readiness/p3-to-p4` route exists, accessible to Founder + Engineering Lead + DPO + External Auditor (read-only for the latter, scoped to their evidence).
- [ ] Sign action is disabled until all 16 criteria are green; UI shows the blocker(s) explicitly.
- [ ] Phase-Exit RFC sign flow: founder + Engineering Lead + DPO sign in-platform; External Auditor + First External Client letters captured as evidence-blobs.
- [ ] `obs.phase_exit_rfc` row is created with all 5 signatures' evidence captured.
- [ ] Audit-log entry's ID is captured in the row.
- [ ] After signing, Compliance Cockpit's "P3 → P4 status" flips green.

**Gherkin (PRD §19.18).**

```gherkin
Feature: Phase-Exit RFC for P3 → P4 cannot be signed if any criterion is not green

  Scenario: Founder attempts to sign with one yellow criterion
    Given gate "p3_to_p4" exists with 16 criteria seeded
    And 15 criteria have status "green"
    And 1 criterion "external_pilot_nps_8" has status "yellow" (NPS = 7)
    When Founder navigates to /obs/gate-readiness/p3-to-p4/sign
    Then the "Sign" CTA is disabled
    And the UI shows "1 criterion not yet green: external_pilot_nps_8"
    And clicking the criterion deep-links to the first external pilot panel

  Scenario: Founder attempts to publish a CUO unaided report with > 10% founder edit
    Given a CUO draft exists with char_count = 5,000
    And the Founder's final has char_count = 5,800 (16% increase)
    When the Founder presses "Sign and publish as unaided"
    Then the system computes founder_edit_pct_chars = 16
    And the database INSERT into obs.cuo_unaided_report fails with CHECK violation
    And the report is offered to be recorded as obs.cuo_assisted_report instead
    And no audit-log entry references this as an "unaided" report

  Scenario: Multi-tenant load-test soak detects cross-tenant leakage
    Given a load-test soak is in progress on the load-test shard
    And the cross-tenant invariant test runs every 6 hours
    When at hour 18 the test detects tenantA reading tenantB's data via a malicious BRAIN query
    Then obs.multi_tenant_load_test.cross_tenant_leakage_detected = true
    And the soak status flips to "failed"
    And the gate criterion "multi_tenant_load_test_passed" flips red
    And the Engineering Lead + DPO are notified within 5 minutes
    And the Phase-Exit RFC sign flow is blocked until the leak is investigated + a new soak passes

  Scenario: Founder signs Phase-Exit RFC after all 16 criteria green
    Given gate "p3_to_p4" has all 16 criteria green for ≥ 14 consecutive days
    And FR-CP-005 has issued the ISO 27001 Stage 2 certificate (evidence uploaded)
    And FR-CORP-001 has confirmed Singapore HoldCo flip closed
    And first external client has signed an attestation letter (NPS-equivalent ≥ 8)
    When Founder navigates to /obs/gate-readiness/p3-to-p4/sign and presses "Author + Sign Phase-Exit RFC"
    And step-up auth is performed via passkey
    And CUO/CAIO drafts the RFC narrative consuming all evidence-blobs
    And Founder edits + finalises
    And Engineering Lead signs
    And DPO signs
    And External Auditor letter is uploaded
    And First External Client letter is uploaded
    Then a row is inserted into obs.phase_exit_rfc with all 5 sign points
    And an audit-log entry is created with persona-version + skill-version chain + 22-month milestone-arc check
    And the Compliance Cockpit's "P3 → P4 status" flips green
    And the P4 phase is unlocked (PORTAL + TEN P4 features + Public APIs + go-to-market)
```

## Success Metrics

**Outcome metrics (measured at P3 → P4 gate-window).**
- Gate criteria green: 16 / 16.
- Days each criterion has been green at sign-time: ≥ 14 (zero criterion in green-streak < 14).
- Founder edit pct on CUO unaided report: ≤ 10% by chars AND words.
- Multi-tenant load-test soak: zero cross-tenant leakage events; zero audit-chain anomalies; all NFRs green; 7 consecutive days.
- First external client NPS: ≥ 8 with response_count ≥ 3 OR signed written attestation.
- Compliance Cockpit: green on all 8 P3 regimes for ≥ 14 consecutive days.
- Phase-Exit RFC: signed by 5 stakeholders (Founder + Engineering Lead + DPO + External Auditor + First External Client letters captured) within 7 calendar days of all-green status.

**Process metrics (measured continuously across P3).**
- Daily evaluator job: < 5 min runtime; < 0.1% failure rate; 100% audit-log coverage of criterion-status changes.
- `obs.cuo_unaided_report` accept rate: at least 1 row per quarter in P3 (Q1 + Q2 of P3); founder edit pct trends downward across the 4 quarters of P3 (M19 → M22) — operational evidence the CUO is becoming progressively more autonomous.
- Cross-tenant invariant test runs during load-test: 28 runs (every 6 hours × 7 days), zero failures.
- Phase-exit RFC review-cycle time: ≤ 14 calendar days from "all green" to all 5 signatures captured.

**Anti-metrics (must NOT happen).**
- Phase-exit RFC published with any criterion not green at sign-time.
- CUO report recorded as "unaided" when founder edit > 10% (CHECK constraint must enforce).
- Cross-tenant data leakage detected during load-test that does not trigger immediate red-flag + Notify.
- Founder signs Phase-Exit RFC without step-up auth.
- A P4 module ships before this gate is signed.

## Open Questions

- **OQ-OBS-004-01.** Should the first external client be a paying customer or a free-tier pilot? PRD §14.4 is silent; the BILL-001 flow can serve either. **Decision needed by Founder by P3 entry (M18).** Default proposal: paying customer at T1 plan tier with first 60 days credited as pilot rebate.
- **OQ-OBS-004-02.** Should we accept multiple external pilots in parallel (e.g. 2-3) so the gate-criterion is ≥ 1 of N rather than the single-pilot dependency? **Decision needed by Founder + Account Manager by P3 mid-cycle (M19).** Default proposal: 1 primary pilot for the gate; up to 2 additional shadow pilots that don't gate-block but inform the dashboard.
- **OQ-OBS-004-03.** Should the load-test config include a "regulated-data" tenant (one tenant with synthetic compensation + special-category data) to verify the BRAIN denylist (DEC-036) holds at scale? **Decision: yes, add a single regulated-data tenant to the 100; add invariant tests verifying the denylist suppresses ingestion.** Routed for confirmation in P3 entry planning.
- **OQ-OBS-004-04.** Should the CUO unaided-report skill have a special "no-PII guard" mode that suppresses any sentence mentioning a specific employee by name (since the report is read-only and intended for board consumption)? **Decision needed by Founder + DPO.** Default proposal: yes; the skill emits aggregate stats (headcount, turnover, total comp) but no individual employee names; founder can override during edit.
- **OQ-OBS-004-05.** Should the Phase-Exit RFC be made public (Trust Center) at sign-time? **Decision needed by Founder.** Default proposal: a redacted version (no NDA-covered customer names, no specific financial numbers below P&L line totals) is published to the Trust Center; the full version is internal + auditor-only.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §1.4 Milestone arc (22-month CUO C-skill autonomy test).
- PRD §14.4.2 P3 → P4 exit gate criteria (verbatim).
- PRD §11.2 Non-Functional Requirements (NFR catalogue evaluated by this FR).
- PRD §16 governance (status discipline + RFC sign chain).
- PRD §19.18 Gherkin acceptance-criteria style.
- PRD §20.2 Module ↔ Role ↔ Phase matrix.
- SRS Decisions Log: DEC-013, DEC-014, DEC-019..DEC-023, DEC-050..DEC-052; DEC-XXX (TBD) load-test shard topology + SOC 2 Type II auditor selection.
- FR-OBS-003 (provides reused `obs.gate_*` schema).
- FR-CP-005 (provides ISO 27001 Stage 2 + SOC 2 Type I; this FR consumes the certificate; SOC 2 Type II scaffolding starts here).
- FR-CORP-001 (provides Singapore HoldCo flip closure evidence).
- FR-TEN-001/002/003 + FR-DOC-001 + FR-BILL-001 (provide the first external pilot's tenant + signing + billing surfaces).
- FR-GENIE-001 (CUO base persona; the unaided-report skill mode is a new flag on this persona).
- FR-AUTH-001/002/003 (sign chain + audit chain + step-up).

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03 against PRD §14.4.2 + SRS Decisions Log. Final wording is the Founder's responsibility.*
