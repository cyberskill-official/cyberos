---
title: "REW — Good Leaver / Bad Leaver branches; termination flow with final payout, BP clawback rules, sabbatical-accrual handling"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: high
target_release: "P2 / 2027-Q1"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Implement the **Good Leaver / Bad Leaver** branches specified in CyberSkill's Total Rewards Appendix and named in PRD §2.3 Bet 5. Termination is the highest-stakes lifecycle event in compensation; this FR ships: **termination flow** (notice → last-day → final-payout-cycle → handover); **classification** (Good Leaver: voluntary resignation with proper notice, retirement, redundancy, illness; Bad Leaver: termination for cause, breach of contract, gross misconduct, post-resignation NDA breach); **final payout calculation** (prorated salary to last day + earned-but-unpaid P3 BP from the open quarter [Good Leaver only; Bad Leaver forfeits unredeemed BP per Appendix; the rule is parameter-version-versioned] + accrued unpaid sabbatical days [Good Leaver: paid out; Bad Leaver: forfeit] + statutory severance per Vietnamese Labour Code Article 46-47); **clawback rules** (rare; specific Bad Leaver cases where prior unauthorised payments are recovered; documented, signed, audited); **handover scaffolding** integration with PROJ + KB + EMAIL (re-assignment of issues, knowledge transfer pages, mailbox forwarding); the **legal-document trail** (termination letter, severance receipt, NDA reminder); and the **DSAR-on-termination** option (per FR-CP-002, a former employee can request data export + erasure within statutory limits). All compute deterministic; no AI; founder + HR/Ops Lead + DPO all sign every termination.

## Problem

The PRD §9.17 names "Good Leaver / Bad Leaver branches" as part of ESOP (P3) but the principle applies to REW today. Three failure modes the platform must structurally prevent:

- **Inconsistent classification.** The same circumstance treated differently for two different Members would be a discrimination liability. Structured classification rules + parameter-version anti-retroactive contract is the floor.
- **Lost work.** A terminated Member's open issues + KB authorship + email threads get stranded; the team scrambles to reconstruct ownership weeks later.
- **Statutory severance miscalculation.** Vietnamese Labour Code Article 46-47 specifies severance for involuntary terminations; getting the math wrong is a labour-disputes-tribunal liability.

## Proposed Solution

The shape of the answer is `hr_secure.termination_*` schema + the deterministic final-payout pipeline + the handover-scaffolding integrations + the legal-document trail + the DSAR-on-termination flow.

**Schema.**

```sql
-- The termination event record.
CREATE TABLE hr_secure.termination (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL UNIQUE REFERENCES hr.employee(id) ON DELETE RESTRICT,
  classification TEXT NOT NULL,                              -- "good_leaver_resignation" | "good_leaver_retirement"
                                                             -- | "good_leaver_redundancy" | "good_leaver_illness"
                                                             -- | "bad_leaver_for_cause" | "bad_leaver_breach"
                                                             -- | "bad_leaver_gross_misconduct" | "bad_leaver_nda_breach"
                                                             -- | "probation_failed"
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  notice_given_at DATE,                                       -- when the Member or company gave notice
  last_working_day DATE NOT NULL,
  effective_termination_date DATE NOT NULL,                   -- typically last_working_day; statutory rules sometimes differ
  reason_md_encrypted BYTEA,                                  -- full rationale; encrypted; senior-counsel-reviewed for Bad Leaver
  exit_interview_summary_md_encrypted BYTEA,                  -- captured by HR/Ops; visible only to HR/Ops + Founder
  -- Final payout components (encrypted under hr_secure KMS):
  prorated_p1_minor_encrypted BYTEA,
  prorated_p2_minor_encrypted BYTEA,
  unredeemed_bp_payout_minor_encrypted BYTEA,                  -- 0 for Bad Leaver per Appendix
  unpaid_sabbatical_payout_minor_encrypted BYTEA,              -- 0 for Bad Leaver per Appendix
  statutory_severance_minor_encrypted BYTEA,                   -- per Article 46-47 if involuntary
  reimbursable_expense_final_minor_encrypted BYTEA,
  clawback_minor_encrypted BYTEA,                               -- positive number reduces the payout (rare)
  total_final_payout_minor_encrypted BYTEA NOT NULL,
  -- Sign-off:
  signed_by_employee_at TIMESTAMPTZ,                            -- the former Member countersigns the calculation
  signed_by_founder_at TIMESTAMPTZ,
  signed_by_hr_ops_at TIMESTAMPTZ,
  signed_by_dpo_at TIMESTAMPTZ,                                  -- for Bad Leaver classification + clawback
  legal_review_ref TEXT,                                          -- external counsel reference for Bad Leaver
  status TEXT NOT NULL DEFAULT 'draft',                           -- "draft" | "notice_period" | "computed"
                                                                  -- | "signed" | "paid_out" | "completed" | "disputed"
  paid_out_at TIMESTAMPTZ,
  -- Final-payout cycle integration:
  final_payroll_cycle_month TEXT REFERENCES hr_secure.payroll_cycle(cycle_month),
  -- Handover:
  handover_completed BOOLEAN NOT NULL DEFAULT false,
  handover_completion_evidence JSONB,                              -- counts: issues reassigned, KB pages re-owned, mailbox forwarded
  -- DSAR on termination:
  dsar_requested BOOLEAN NOT NULL DEFAULT false,
  dsar_request_id UUID,                                            -- references cp.dsar_request when invoked
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE hr_secure.termination_clawback_event (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  termination_id UUID NOT NULL REFERENCES hr_secure.termination(id) ON DELETE RESTRICT,
  source TEXT NOT NULL,                                            -- "unauthorised_advance" | "loan_outstanding" | "company_property"
                                                                   -- | "training_clawback" | "nda_breach_penalty"
  amount_minor_encrypted BYTEA NOT NULL,
  reason_md_encrypted BYTEA NOT NULL,
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_dpo_at TIMESTAMPTZ NOT NULL,
  legal_review_ref TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Classification rules (encoded in parameter version).**

The active parameter version (FR-REW-001) carries `parameters.termination_classification_rules`:

```yaml
good_leaver_resignation:
  conditions:
    - voluntary
    - notice_period_met (>= notice_period_days from contract)
  benefits:
    - prorated_p1_p2: yes
    - unredeemed_bp_payout: yes (next-quarter close prorated)
    - unpaid_sabbatical_payout: yes
    - statutory_severance: no (voluntary)
    - retain_phantom_stock_vested: yes (P3 ESOP)

good_leaver_retirement:
  conditions:
    - reached_retirement_age (per Vietnamese Labour Code: 62 for males, 60 for females, phased to 62/60 by 2035)
  benefits: same as resignation + retirement-specific provisions

good_leaver_redundancy:
  conditions:
    - involuntary
    - role_eliminated (signed-by-founder reason)
  benefits:
    - prorated_p1_p2: yes
    - unredeemed_bp_payout: yes
    - unpaid_sabbatical_payout: yes
    - statutory_severance: yes (Article 46: 1 month per year of service after 12 months)
    - retain_phantom_stock_vested: yes

good_leaver_illness:
  conditions:
    - signed_medical_certification
    - founder + DPO sign
  benefits: same as redundancy

bad_leaver_for_cause:
  conditions:
    - documented gross misconduct or breach
    - founder + DPO + legal_review_ref signs
  benefits:
    - prorated_p1_p2: yes (statutory floor; cannot withhold)
    - unredeemed_bp_payout: NO (forfeit per Appendix)
    - unpaid_sabbatical_payout: NO (forfeit per Appendix)
    - statutory_severance: per legal review (typically not for cause)
    - retain_phantom_stock_vested: per ESOP P3 rules (Bad Leaver branch)

bad_leaver_breach:
  conditions:
    - post-resignation NDA / non-compete violation
    - legal_review_ref signs
  benefits: same as for_cause + clawback applicable

bad_leaver_gross_misconduct:
  conditions:
    - documented gross misconduct
    - founder + DPO + legal_review_ref + (optionally) labour-tribunal-pending flag
  benefits: same as for_cause

probation_failed:
  conditions:
    - within probation period
    - manager + HR/Ops review documented
  benefits:
    - prorated_p1_p2: yes
    - unredeemed_bp_payout: prorated (typically minimal at probation period)
    - unpaid_sabbatical_payout: 0 (no accrual yet)
    - statutory_severance: no (probation)
```

**Termination flow.**

1. **Notice or initiation.** Either the Member submits resignation (with `last_working_day`) OR HR/Ops Lead + Founder initiate involuntary termination. A `termination` row is created with `status: draft`.
2. **Classification.** HR/Ops Lead proposes the classification; founder + (for Bad Leaver) DPO + legal counsel review. Classification is captured with rationale + signatures. `status: notice_period`.
3. **Notice period.** From the day of notice through `last_working_day`. During this window: handover scaffolding kicks off; Member's `current_status` transitions to `notice_period`; a Notify card surfaces to the Member's manager + Project Lead + Account Manager (Engagement linkage from FR-PROJ-007); KB-page primary-author reassignment is suggested; PROJ-issue assignment is suggested for redistribution.
4. **Final-payout compute.** A scheduled job at `last_working_day - 7d` produces a draft final payout in the next month's `payroll_cycle` (FR-REW-003 consumes; the cycle gets a special flag `includes_termination: true`).
5. **Sign-off.** Member countersigns the final-payout calculation. Founder signs. HR/Ops signs. DPO signs (Bad Leaver only). `status: signed`.
6. **Final-payroll cycle.** The next monthly cycle includes the terminated Member's final payout; the cycle close + sign + paid_out flow runs as normal (FR-REW-003).
7. **`status: paid_out`.** The final payout is disbursed; `effective_termination_date` is observed.
8. **Handover completion.** The handover scaffolding (next section) is verified; `handover_completed: true`; `status: completed`.
9. **Post-termination.**
   - `hr.employee.current_status` → `terminated`.
   - All sessions invalidated (FR-AUTH-001).
   - All agent OAuth clients revoked (FR-AUTH-003).
   - The Member's BRAIN data is *retained* under existing retention rules (no auto-erasure; see DSAR-on-termination below).
   - The Member retains read-only access to their own historical payslips + the official termination letter + final payout receipt for 90 days post-termination (via a special `terminated_employee_self_service` token).

**Handover scaffolding.**

- **PROJ.** A Notify card to the Member's PROJ project leads: "Khoa is leaving on 2026-09-30; reassign their open issues." A bulk-reassign tool surfaces the issues; the Member can suggest reassignees via the propose-then-commit pattern (FR-PROJ-008).
- **KB.** A Notify card: "Khoa authored 12 KB pages. Re-owner candidates suggested by CUO/CHRO." HR/Ops Lead + the Member confirm the re-owner.
- **EMAIL.** The Member's `cyberskill.world` mailbox is set to forward to a designated colleague (typically their manager) for 30 days post-termination; auto-reply during the forward window.
- **CRM.** All deals + accounts the Member primary-owned are reassigned via the propose-then-commit pattern (FR-CRM-002 mutations).
- **Equipment.** Out-of-scope here (P3 IT module if it ever exists); HR/Ops Lead manually tracks for now.

The `handover_completion_evidence` JSONB captures counts + reassigned-to references + dates.

**Statutory severance per Vietnamese Labour Code.**

- **Article 46** governs severance for indefinite-term contracts: **1/2 month base salary per year of service** (rounded up; first 12 months excluded).
- **Article 47** governs job-loss support (similar formula in different circumstances).
- The deterministic engine (similar to FR-REW-004's PIT engine) computes the statutory severance based on years-of-service + average-monthly-base-of-last-6-months + classification.

The engine is parameter-version-versioned; rate changes (e.g. Vietnamese law amendments) flow through the same publish-and-sign pipeline as everything else.

**Bad Leaver clawback.**

For specific Bad Leaver cases (NDA breach with documented penalty; unauthorised advance not yet repaid; training clawback per signed agreement), a `termination_clawback_event` row is created. Each event:

- Has a documented `source` + `reason_md_encrypted` + `legal_review_ref`.
- Requires founder + DPO sign.
- Reduces the `total_final_payout` by the clawback amount.
- Cannot reduce the prorated P1 below the Vietnamese statutory minimum-wage floor for the proration period (Article 95 wage protection invariant).

Clawback is rare (expected ≤ 1 per several years); audit-heavy; surfaced in the Compliance Cockpit.

**Legal-document trail.**

For each termination, the platform generates / supports:

1. **Termination letter** — Vietnamese-language; stored as a signed PDF in the content-addressed blob store under the `hr_secure` KMS key. Templates exist for each classification.
2. **Final payout receipt** — generated automatically post-paid_out; signed by founder + Member.
3. **NDA reminder** — for all leavers; reminder of post-termination obligations from the original contract.
4. **Statutory filings** — the platform produces the data the Vietnamese-tax-authority filing requires (T0 form + final personal income tax statement); the company's external accountant files manually.

**DSAR-on-termination.**

A terminated Member can invoke a DSAR (FR-CP-002) within 90 days post-termination:

- Receives a structured export of all their personal data the platform holds.
- Can subsequently invoke the right-to-erasure (RTBE) flow per FR-CP-002, with the caveat that statutory-retention items (audit log, payroll records — PDPL Decree 13 + Vietnamese-tax-law specify retention floors) are pseudonymised but not deleted.

**Frontend surfaces.**

- **HR/Ops Lead view** at `/rew/admin/terminations`: in-flight terminations + draft + signed + paid-out + completed; per-termination workflow.
- **Founder view**: counter-sign queue.
- **Member view (during notice period)** at `/rew/my/termination`: read-only view of their proposed classification + final-payout calculation; countersign action.
- **Terminated-employee read-only view** for 90 days: payslip history + termination letter + final-payout receipt.

**MCP tool surface (read-only; very narrow).**

- `cyberos.rew.list_terminations(status?)` — read; HR/Ops + Founder + DPO + Auditor; aggregate.
- `cyberos.rew.get_termination(id)` — read; same audience.
- `cyberos.rew.my_termination_calculation` — read; if the calling Member is in `notice_period`; their own; step-up.

There are **no mutation MCP tools**. Termination is the highest-stakes path; UI + step-up only.

**Compliance Cockpit panel.**

- Active terminations + classification distribution.
- Sign-off latency per stage.
- Clawback events (should be exceedingly rare).
- DSAR-on-termination requests.
- Statutory severance compliance audit.

## Alternatives Considered

- **Treat all leavers identically.** Rejected: the contract distinguishes; treating differently is the legal commitment.
- **Auto-reassign all PROJ issues / KB pages without confirmation.** Rejected: human-in-the-loop floor; the Member's last contributions are too important to auto-redistribute.
- **Skip the DSAR-on-termination option.** Rejected: PDPL + GDPR (P3+) require it; offering it explicitly is the floor.
- **Allow AI to suggest classification.** Rejected: classification is a legal call; AI suggestion would create liability + bias risk.
- **Skip clawback path entirely.** Rejected: rare-but-real cases (NDA penalty, unauthorised advance) exist; the path with extreme audit + sign requirements is the floor.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) a synthetic resignation triggers the full Good-Leaver flow end-to-end with proper compute + sign + paid_out + handover; (2) a synthetic Bad-Leaver-for-cause case requires the founder + DPO + legal-counsel-ref signatures; (3) statutory severance for a synthetic redundancy case matches the hand-computed Article 46 amount; (4) the handover scaffolding reassigns 100% of the synthetic Member's PROJ + KB + CRM artefacts.
- **Compliance metric.** Zero Article-95-wage-protection violations from a clawback (the floor is enforced).
- **Latency / cycle.** End-to-end termination cycle (notice → paid_out) ≤ contract notice-period + 30 days.

## Scope

**In-scope.**
- The 2 schema additions (`termination`, `termination_clawback_event`).
- 9 classification rules encoded in the parameter version.
- Termination flow with sign-off chain.
- Statutory severance compute (Article 46-47).
- Handover scaffolding (PROJ, KB, EMAIL, CRM integration).
- Bad-Leaver clawback path with founder + DPO + legal-counsel-ref signatures.
- Legal-document trail (4 document types).
- DSAR-on-termination invocation.
- 90-day terminated-employee read-only access.
- The 3 read-only MCP tools.
- Audit integration in scope `rew.termination.{tenant}`.
- Compliance Cockpit panel.

**Out-of-scope (deferred).**
- ESOP / phantom-stock Good-Leaver / Bad-Leaver branches (P3 — separate FR cluster).
- Multi-jurisdictional severance computation (P3+ international hires).
- Automated NDA-breach detection (out of scope; legal counsel + founder review is the floor).
- Wrongful-termination dispute workflow (P3+ if needed; for now, the audit log + the legal-counsel-ref are the trail).

## Dependencies

- FR-HR-001 / FR-HR-002 / FR-HR-003 / FR-REW-001..006.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-CP-001 / FR-CP-002 (Compliance Cockpit + DSAR-on-termination).
- FR-PROJ-001..008 / FR-KB-001..003 / FR-EMAIL-001..010 / FR-CRM-001..004 (handover scaffolding consumers).
- FR-OBS-001 / FR-OBS-002.
- The signed Total Rewards Appendix.
- External legal counsel for Bad-Leaver classifications + clawback authorisation.
- Compliance: Vietnamese Labour Code Articles 35-49 (termination), Article 95 (wage protection); PDPL Decree 13 (DSAR-on-termination); EU AI Act Articles 5-7 high-risk classification (compensation domain — no AI in classification or compute path); GDPR Article 17 (right to erasure for the EU-residency case in P3+); SOC 2 CC8.
- Locked decisions referenced: DEC-188 (9 classifications), DEC-189 (Bad Leaver requires founder + DPO + legal_review_ref), DEC-190 (clawback never breaches Article 95 floor), DEC-191 (90-day post-termination read-only access), DEC-192 (DSAR-on-termination available; statutory retention floors honoured).

## AI Risk Assessment

This FR explicitly forbids AI in the classification or compute path. EU AI Act risk class: `high` (compensation + employment-decision domain).

### Data Sources

The data is the terminating Member's compensation history + role history + any documented circumstances. No AI inputs in this FR; CUO/CHRO has zero scope on termination.

### Human Oversight

- Classification is human (HR/Ops + Founder; DPO for Bad Leaver; legal counsel for Bad Leaver).
- Compute is deterministic.
- Sign-off is multi-party.
- The terminating Member can countersign or dispute.
- Clawback events are exceptionally audit-heavy.
- The Compliance Cockpit surfaces everything.

### Failure Modes

- **Wrong classification.** Mitigation: dual + (for Bad Leaver) triple sign-off; legal review for Bad Leaver; the Member can dispute via labour tribunal (and the audit log is the company's defence).
- **Statutory-severance miscompute.** Mitigation: the engine is regression-tested by the accountant; per-year-of-service + average-base mechanics versioned.
- **Clawback breaches wage protection.** Mitigation: the floor is structurally enforced (the engine checks against the regional minimum-wage proration before reducing).
- **Handover incomplete at termination date.** Mitigation: `handover_completed: false` blocks status `→ completed`; the team has time pressure to finish; if blocked, the next-month cycle pulls the issue forward + escalates to founder.
- **DSAR-on-termination racing with statutory retention.** Mitigation: the export contains everything; the erasure honours statutory floors with pseudonymisation (FR-CP-002 pattern).

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted classification matrix, termination flow, statutory severance computation, handover scaffolding, clawback rules, failure modes.
- **Human review:** `@stephen-cheng` reviewed; legal counsel + Vietnamese-labour-law specialist will review the classification rules + Article 46-47 encoding before P2 production.
