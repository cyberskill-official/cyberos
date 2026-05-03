---
title: "REW — frontend remote at /rew (Member view + HR/Ops admin) + read-only AI payslip narrator (CUO/CHRO; never compute)"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q1"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the REW frontend remote at `/rew`: **Member view** with current-month payslip, payslip history, BP-balance + projected next P3 payout, sabbatical accrual (read from FR-TIME-002), pending-action items (countersign new salary records, dispute resolutions); **HR/Ops Lead admin views** for cycle management (open / compute / review / sign / paid_out), parameter version drafting + publishing, salary-record drafting + publishing, BP earning-event creation, statutory rate-table management; **Founder views** for counter-signing + the Compliance Cockpit deep-dive; **read-only payslip narrator** — a CUO/CHRO surface that reads a Member's payslip and produces a plain-language explanation ("Your gross was 50M VND; we deducted 4M for SI/HI/UI per the statutory cap; PIT was 3.2M after your personal + 1-dependent deductions; net is 42.8M into your Vietcombank account ending 1234"). The narrator **never computes** compensation — it only describes the deterministic engine's outputs (FR-REW-003 + FR-REW-004); the boundary is enforced by the persona-scope contract (FR-MCP-001). All amount-bearing surfaces require **step-up auth** (FR-AUTH-003). `/rew` is the highest-confidentiality user surface in the platform.

## Problem

The data + compute pipeline from FR-REW-001..004 is invisible without a UI. Three failure modes the platform must avoid:

- **Member opacity.** A Member who can't see their own salary structure / BP balance / payslip history loses trust in the platform. The Total Rewards Appendix is the contract; without transparency, the contract is a black box.
- **HR/Ops friction.** The HR/Ops Lead runs the monthly close + the quarterly BP close + the year-end reconciliation. Without a clean UI, every cycle is a 4-hour Excel reconstruction.
- **Payslip illiteracy.** Vietnamese payslips encode SI/HI/UI/PIT computation — the typical Member doesn't know what each line means. The PRD §9.14 explicitly names "payslip narrative explainer (read-only AI)" as a P2 feature.

## Proposed Solution

The shape of the answer is a Vite + React 19 Module-Federation remote at `/rew` consuming the GraphQL surfaces from FR-REW-001..004, the Yjs collaboration substrate (for the narrator's review-mode draft cards), and the AI Gateway via the CUO/CHRO read-only persona.

**Member view (`/rew/my`).**

Default home for any Member who isn't HR/Ops or Founder.

- **Header.** "Hi <preferred_name>. Here's your compensation overview."
- **This-month section** — once the cycle is `paid_out`:
  - Net pay (large, prominent).
  - Gross | total deductions | net (line summary).
  - Click "View payslip" → opens the signed PDF inline (with the AI narrator alongside).
  - Bank disbursement reference (last 4 digits + payment date).
- **Year-to-date section.**
  - Total gross YTD; total tax YTD; total SI/HI/UI YTD.
  - Visualisation: stacked bar by month.
- **Bonus Points section.**
  - Current quarter's earned points (with breakdown by source — VP evaluation, Hội đồng Chuyên môn, ad-hoc recognition).
  - Rolled-forward balance.
  - Projected next-quarter payout (best-case + worst-case bands, derived deterministically from the BP fund's published cash-pool + expected total points; not AI-generated).
  - Click "Why this many?" → opens the read-only narrator.
- **Salary section.**
  - Current P1 + P2 (encrypted; revealed on step-up).
  - History (last 5 changes; with effective dates + reasons).
  - Pending: any new salary record awaiting countersign — clear CTA.
- **Sabbatical accrual.** Read from `time.sabbatical_accrual` (FR-TIME-002).
  - "You've accrued X.Y years of continuous service. Your next sabbatical eligibility is on YYYY-MM-DD."
- **Documents.**
  - Past payslips (signed PDFs).
  - Current contract (links to DOC P3 when shipped; for P2, links to a placeholder).
  - Year-end PIT reconciliation reports.

Every amount-bearing line requires step-up auth on first reveal-per-session.

**HR/Ops admin view (`/rew/admin`).**

Restricted to HR/Ops Lead + Founder + DPO.

- **Cycle dashboard.** Open / current cycle's status; per-Member rows; anomaly counts; sign progress.
- **Cycle management.**
  - "Open new cycle" (system-driven on the 28th; manual override available).
  - "Run compute" (idempotent; reproduces the deterministic outputs).
  - "Review anomalies" (per-anomaly resolution UI; documented justification required for warn-level resolves; block-level requires data fix before continuing).
  - "Sign cycle" (HR/Ops sign step; step-up).
  - "Generate bank file" (post-sign).
  - "Mark paid_out" (after bank confirms).
- **Salary drafts.**
  - List of pending salary changes; per-change: Member, old vs. new amounts (encrypted; revealed on step-up), reason, founder-sign-status.
  - "Draft new salary" form.
- **Parameter versions.**
  - List of versions; current; pending drafts.
  - "Draft new version" with full parameter editing.
  - Publish flow (founder + engineering-lead sign).
- **BP fund management.**
  - Per-quarter view with fund cash + total points.
  - "Add cash to fund" (founder-only; signed).
  - "Run quarterly close" (system-driven on quarter+1; manual override available).
  - Earning-event creation form.
- **Statutory rate management.**
  - Current rate-table version + history.
  - "Draft new rate-table" form.
- **Year-end PIT reconciliation.**
  - Per-Member reconciliation reports.
  - File-with-tax-authority workflow.

**Founder views.**

A subset of HR/Ops admin focused on counter-sign + cockpit deep-dive:

- "Awaiting your sign" inbox: salary-record counter-signs, parameter-version publishes, BP earning-event signs, payroll-cycle counter-signs, year-end-rate-table publishes.
- Each item: full context, amounts (encrypted; revealed on step-up), audit trail, sign / deny CTA.
- "Compliance Cockpit deep-dive" — the founder-only operational view into anomalies, force-reductions, comp-leakage sweep results.

**Read-only payslip narrator.**

When a Member opens their payslip view, the page renders:

1. The signed PDF.
2. Alongside, a narrative panel: a CUO/CHRO-generated plain-language explanation.

The narrator's prompt:
- **Inputs.** The Member's payslip's `computation_trace` (decrypted under step-up); the parameter version + rate tables; their statutory profile (dependent count, zone); the prior month's payslip for delta.
- **Output.** A 4-8 sentence Vietnamese-or-English (per Member's `language_default`) explanation:
  > "Your gross compensation this month is 50,000,000 VND, the same as last month. We deducted 4,000,000 VND for social, health, and unemployment insurance (calculated on a 50,000,000 VND base, which is below the statutory cap of 99,200,000 VND for Zone I). Personal income tax is 3,200,000 VND, computed on a taxable base of 30,600,000 VND after your 11,000,000 VND personal deduction and one 4,400,000 VND dependent deduction; the tax bracket applied is 20%. Your net pay is 42,800,000 VND, transferred to your Vietcombank account ending 1234 on 28 September 2026. Your bonus points balance is 145; the next P3 payout is expected at quarter-end (October 2026)."
- **Persona-scope contract.** CUO/CHRO is allowed `read_only: true` access to the payslip's compute-trace via the `cyberos.rew.my_payslip` MCP tool; *zero* mutation tools are in scope. The persona is configured to `refuse: true` on any prompt asking it to compute, modify, or recommend changes to compensation; refusal-correctness is part of the regression eval suite.
- **EU AI Act Article 50 disclosure.** The narrative panel renders the chip "AI-generated explanation · CUO/CHRO v0.X · Verify against your signed payslip".
- **Disputed?** The Member can flag any sentence as "this seems wrong" — the flag opens a dispute thread with HR/Ops Lead; the narrator's output is preserved with the flag.

The narrator **never computes** the underlying numbers — it reads them from the deterministic engine's output. A regression-test corpus of 30 synthetic payslips with hand-written reference narrations runs in CI on every persona-version PR; deviation > 10% in semantic similarity blocks the PR.

**Payroll period reports (HR/Ops, Founder, DPO, Auditor).**

`/rew/admin/reports`:
- Aggregate by period (monthly, quarterly, annual).
- Total gross / net / SI / HI / UI / PIT — *no per-Member breakdown unless the requester has SELECT permissions on `hr_secure.payroll_record`*.
- Auditor sees only aggregate roll-ups.

**Vietnamese-locale rendering.**

- Default `vi-VN` for the canonical CyberSkill tenant.
- Monetary formatting: thousand-separator (typical Vietnamese: 50.000.000 VND or 50,000,000 VND configurable).
- Date formatting per Vietnamese convention (DD/MM/YYYY).
- The narrator's language follows the Member's `language_default` (FR-EMAIL-005 contact-profile pattern reused).

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped.
- `/rew/my` first-paint ≤ 1.5 s on 4G.
- Step-up reveal of an encrypted amount ≤ 2 s p95 (the WebAuthn ceremony + the decrypt round-trip).

**MCP tool surface.**

(All read-only; no mutation MCP for REW — see FR-REW-001 §"MCP tool surface".)

- `cyberos.rew.my_overview` — read; the `/rew/my` data payload; step-up.
- `cyberos.rew.my_payslip(cycle_month)` — read; step-up.
- `cyberos.rew.my_payslip_narrative(cycle_month)` — read; calls the narrator; step-up.
- `cyberos.rew.my_bp_balance(quarter?)` — read; step-up. (Already in FR-REW-002.)
- `cyberos.rew.list_admin_dashboard_summary` — read; HR/Ops + Founder; aggregate only.

**Compliance Cockpit deep-link.**

The Founder's view links into the Cockpit (FR-CP-001) to show, per regulatory regime, the REW-related evidence:
- Number of unsigned salary records (should be 0).
- Force-reductions in the last quarter (should be 0).
- Comp-leakage sweep results (should be 0).
- Statutory-rate parameter-version validity (should be `green`).
- Anomaly-resolution latency (should be median ≤ 1 day).

## Alternatives Considered

- **Hosted Vietnamese-payroll-portal-style UI.** Rejected: residency + the Total Rewards Appendix structure + the persona-narrator + the audit grade — not viable hosted.
- **Skip the narrator.** Rejected: PRD §9.14 explicitly names it. The transparency-via-narrative is part of how the platform earns trust.
- **Allow the narrator to suggest "your raise should be X" or "your next promotion target".** Rejected: explicit prohibition; the narrator is read-only on past-and-current data; future-modeling is forbidden.
- **Single-step auth for Member's own data.** Rejected: comp data is highest-sensitivity; step-up is the floor.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder + 1 Member view their `/rew/my` with all sections populated; (2) the narrator produces a plain-language explanation matching the regression-corpus reference within tolerance; (3) HR/Ops admin runs a full synthetic monthly cycle (open → compute → review → sign → paid_out → narrator); (4) Founder counter-sign requires step-up.
- **Adoption metric.** Every Member opens their `/rew/my` at least once per month after the cycle's `paid_out`; ≥ 80% open the narrator at least once for a payslip dispute / understanding session.
- **Quality metric.** Narrator regression suite passes ≥ 95% on every persona-version PR.
- **Latency NFR.** First-paint ≤ 1.5 s; step-up reveal ≤ 2 s; narrator generation ≤ 6 s p95.

## Scope

**In-scope.**
- Module-Federation remote at `/rew`.
- Member view (`/rew/my`) with all 7 sections.
- HR/Ops admin views.
- Founder views.
- Read-only payslip narrator + persona scope contract + EU AI Act disclosure.
- Vietnamese-locale rendering.
- Step-up enforcement on every amount-bearing reveal.
- Payroll period reports (aggregate).
- The 5 read-only MCP tools (compensation surfaces are read-only).
- Persona regression eval corpus (30 synthetic payslips with reference narrations).
- Audit integration in scope `rew.ui.{tenant}`.

**Out-of-scope (deferred).**
- Salary planning / forecast tools (not appropriate for REW; LEARN P2 may surface career paths).
- Equity-related views (P3 — ESOP).
- Multi-currency display (P3 — international hires).
- Mobile-native (P3).
- Tax-planning advice (out of scope forever).

## Dependencies

- FR-HR-001 / FR-REW-001 / FR-REW-002 / FR-REW-003 / FR-REW-004.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002 (CUO/CHRO persona; persona-scope; acceptance metrics).
- FR-OBS-001 / FR-OBS-002.
- FR-CP-001 (Compliance Cockpit deep-link).
- The signed Total Rewards Appendix.
- Compliance: PDPL Decree 13 (highest-sensitivity personal data); EU AI Act Article 50 (transparency disclosure on the narrator); Article 14 (human oversight via step-up + the Member's "this seems wrong" dispute path).
- Locked decisions referenced: DEC-181 (read-only narrator; persona-scope forbids compute), DEC-182 (step-up on every amount-bearing reveal), DEC-183 (regression eval corpus is the persona-version gate).

## AI Risk Assessment

The payslip narrator is the only AI surface in the REW stack. EU AI Act risk class: `limited` (AI-generated content visible to a natural person; no automated decision; no compute).

### Data Sources

Per-tenant only: payslip data + parameter versions + statutory profile + prior payslips for delta. CUO/CHRO runs through the AI Gateway with persona-stamping. Per-tenant residency.

### Human Oversight

- The narrator is read-only on past-and-current data; it cannot suggest changes.
- The Member sees the narrative + the signed PDF side-by-side; can verify line-by-line.
- The "this seems wrong" flag opens a human dispute path with HR/Ops.
- The persona regression eval corpus blocks bad versions.
- The kill-switch from FR-GENIE-002 silences the narrator; the signed PDF remains the canonical source.

### Failure Modes

- **Narrator hallucinates a number.** Mitigation: regression eval corpus; the cited values must match the payslip exactly; deviation blocks PR.
- **Narrator phrases the explanation in a confusing way.** Mitigation: human review of low-acceptance-rate cases; per-language calibration.
- **Narrator suggests changes** (out-of-scope). Mitigation: persona-scope refuse-on-prompt; regression cases include explicit "tell me why my salary is too low" prompts that the narrator must refuse with the right escalation message ("This is a question for your manager + HR/Ops Lead").
- **Step-up bypass.** Mitigated by the FR-AUTH-003 architecture; replay detection on the step-up token.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted frontend layout, narrator architecture, persona-scope contract, regression-corpus design, failure modes.
- **Human review:** `@stephen-cheng` reviewed; founder + Vietnamese-language reviewer + DPO will validate the narrator on real payslips before P2 production.
