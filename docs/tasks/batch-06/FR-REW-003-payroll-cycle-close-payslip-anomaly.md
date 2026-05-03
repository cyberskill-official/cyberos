---
title: "REW — payroll cycle close, payslip generation, anomaly detection during close, signed PDF storage"
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
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the **monthly payroll cycle close** that compiles each Member's compensation for the month: P1 Base (FR-REW-001) + P2 Allowance (FR-REW-001) + P3 Performance (the most-recent quarterly BP payout from FR-REW-002, prorated by month if the quarter spans the payroll month) + reimbursable expense flow-through (FR-TIME-003) − Vietnamese SI/PIT (FR-REW-004; consumed by this cycle). Generates **signed payslip PDFs** stored in the content-addressed blob store under the `hr_secure` KMS key. Runs **deterministic anomaly detection** over the cycle's outputs (e.g. > 20% month-over-month delta on any P1; an unsigned salary record falling into the cycle; a parameter version not yet published; a Member transitioning to/from `notice_period` mid-cycle) and **surfaces anomalies before commit** — never auto-resolves. The cycle's commit is the **two-step sign-and-execute**: HR/Ops Lead reviews + signs; Founder counter-signs; only then does the cycle become `paid_out` and feed downstream (bank-disbursement file generation; INV billing for billable client expense passthroughs). PRD §14.3.2 P2 → P3 exit gate: "Payroll cycle close has been completed entirely inside REW module for at least 2 consecutive cycles, with zero anomalies escaped to post-close discovery."

## Problem

The team's current payroll is a monthly Excel that the founder reconciles; mistakes are caught only when a Member raises a discrepancy. Three failure modes the platform must structurally prevent:

- **Anomaly escapes to discovery.** A doubled P2 allowance in a single Member's row has historically been caught only when the Member opens their bank app and notices. The PRD §14.3.2 explicit gate metric: "zero anomalies escaped to post-close discovery."
- **AI in the compute path.** PRD §6.4 + §2.5 + FR-REW-001/002 invariants: AI never computes compensation. This FR's compute pipeline is fully deterministic + auditable.
- **Payslip dispute opacity.** A Member disputes their P3 number; without a signed PDF artefact + the precise computation trace, the dispute resolution is "Stephen's memory vs. the Member's screenshot."

## Proposed Solution

The shape of the answer is `hr_secure.payroll_*` schema + the deterministic compute pipeline + the anomaly detector + the payslip-PDF generator + the two-step sign-and-execute commit.

**Schema.**

```sql
-- A monthly payroll cycle.
CREATE TABLE hr_secure.payroll_cycle (
  tenant_id UUID NOT NULL,
  cycle_month TEXT NOT NULL,                                -- "2026-09"
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  status TEXT NOT NULL DEFAULT 'open',                       -- "open" | "computing" | "review" | "signed" | "paid_out" | "rolled_back"
  open_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  computed_at TIMESTAMPTZ,
  reviewed_at TIMESTAMPTZ,
  reviewed_by_hr_ops UUID,
  signed_by_founder_at TIMESTAMPTZ,
  paid_out_at TIMESTAMPTZ,
  total_gross_minor_encrypted BYTEA,                         -- aggregate; encrypted
  total_net_minor_encrypted BYTEA,
  total_employer_si_minor_encrypted BYTEA,
  total_employer_health_minor_encrypted BYTEA,
  total_employer_unemployment_minor_encrypted BYTEA,
  anomalies_count INT NOT NULL DEFAULT 0,
  bank_disbursement_file_id UUID,                             -- generated post-sign
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, cycle_month)
);

-- Per-Member payroll record for the cycle.
CREATE TABLE hr_secure.payroll_record (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  cycle_month TEXT NOT NULL REFERENCES hr_secure.payroll_cycle(cycle_month),
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  -- All amounts encrypted under hr_secure KMS key:
  p1_base_minor_encrypted BYTEA NOT NULL,                    -- from hr_secure.salary
  p2_allowance_minor_encrypted BYTEA NOT NULL,
  p3_performance_minor_encrypted BYTEA,                       -- prorated from BP fund quarterly close
  p3_quarter_source TEXT,                                     -- "2026-Q3" — for traceability
  reimbursable_expense_minor_encrypted BYTEA,                 -- from FR-TIME-003 approved expenses
  reimbursable_expense_breakdown_encrypted BYTEA,             -- JSONB encrypted: per-expense detail
  gross_compensation_minor_encrypted BYTEA NOT NULL,
  -- Vietnamese SI/PIT components from FR-REW-004:
  social_insurance_employee_minor_encrypted BYTEA NOT NULL,
  health_insurance_employee_minor_encrypted BYTEA NOT NULL,
  unemployment_insurance_employee_minor_encrypted BYTEA NOT NULL,
  personal_income_tax_minor_encrypted BYTEA NOT NULL,
  pre_tax_deductions_minor_encrypted BYTEA NOT NULL DEFAULT '\x'::bytea,
  net_compensation_minor_encrypted BYTEA NOT NULL,
  -- The deterministic compute trace; encrypted; audit-grade:
  computation_trace_encrypted BYTEA NOT NULL,                 -- step-by-step; what value came from where
  -- The signed PDF:
  payslip_pdf_blob_id UUID,                                   -- references the content-addressed blob store
  payslip_pdf_signed_at TIMESTAMPTZ,
  payslip_pdf_signature_sha256 TEXT,                          -- SHA-256 of the signed PDF
  status TEXT NOT NULL DEFAULT 'computed',                    -- "computed" | "anomaly_flagged" | "approved" | "paid"
  anomalies JSONB,                                            -- array of detected anomalies for this row
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, cycle_month, employee_id)
);

CREATE INDEX payroll_record_cycle_idx ON hr_secure.payroll_record (tenant_id, cycle_month, status);

-- Bank disbursement file (generated post-sign).
CREATE TABLE hr_secure.payroll_disbursement_file (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  cycle_month TEXT NOT NULL REFERENCES hr_secure.payroll_cycle(cycle_month),
  bank_format TEXT NOT NULL,                                   -- "vietcombank_ibank" | "techcombank_corp" | "manual_csv"
  file_blob_id UUID NOT NULL,                                  -- the actual file in encrypted blob store
  generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  generated_by UUID NOT NULL,
  status TEXT NOT NULL DEFAULT 'generated',                    -- "generated" | "uploaded_to_bank" | "confirmed_paid"
  uploaded_at TIMESTAMPTZ,
  bank_confirmation_ref TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

**Compute pipeline (fully deterministic; no AI).**

A scheduled job kicks off on the 28th of each month at 18:00 ICT (Vietnamese pay-cycle convention is end-of-month for the same month):

1. **Open the cycle.** Create `payroll_cycle` row with `status: open`. Lock the active `parameter_version` reference at this moment (the cycle uses the version published before this date).
2. **Enumerate employees.** All `hr.employee` with `current_status NOT IN ('terminated', 'candidate')` and a `hire_date <= cycle_end`.
3. **Fetch per-Member salary.** Read `hr_secure.salary` for each employee with `effective_from <= cycle_end AND (effective_to IS NULL OR effective_to >= cycle_start)`.
4. **Compute P1 + P2 prorated.** If the Member joined or transitioned mid-cycle, prorate by working days. The proration formula is deterministic + parameter-version-aware.
5. **Compute P3 prorated from BP.** If a quarterly BP close happened in the cycle's month, that quarter's per-Member `cash_payout` is the P3 source. Otherwise P3 is 0 (pays out only in BP-close months — typically Jan/Apr/Jul/Oct).
6. **Add reimbursable expenses.** From `time.expense` with `status: approved` + `is_reimbursable: true` + `reimbursement_method: payroll`.
7. **Subtract pre-tax deductions** (rare; e.g. employee-elected savings; from `hr_secure.payroll_pretax_deduction` if any).
8. **Call FR-REW-004 SI/PIT engine.** Pass each Member's gross + their statutory profile (dependents, regional minimum wage zone); receive employee + employer SI/health/unemployment + PIT.
9. **Compute net.**
10. **Build computation trace.** A structured JSON: each step's source, inputs, outputs, parameter version reference. Persisted encrypted.
11. **Run anomaly detector** (next section).
12. **Status → review.** Cycle is awaiting HR/Ops Lead review + sign.

**Anomaly detector.**

Runs deterministic rules — *not* statistical / AI. Flags any `payroll_record` matching:

- **Month-over-month P1 delta > 20%.** Caught: a doubled P1 from a typo.
- **Month-over-month total-gross delta > 25%** (excluding planned promotions or P3-bonus-month spikes).
- **P1 < tenant-minimum-wage parameter.** Caught: a P1 erroneously at zero.
- **No active salary record covering the cycle.** Caught: a Member missing a salary parameter.
- **A salary record's signed_by_employee_at IS NULL** (unsigned by Member).
- **A parameter version unsigned at cycle open.**
- **A Member in `notice_period` with no expected last-day match.**
- **A force_reduction_with_legal_review_id set on the Member's salary** (always flagged; review every cycle until resolved).
- **Reimbursable-expense aggregate > 50% of P1** (suspicious; usually correct but always reviewed).
- **A computation step where the input parameter version differs from the cycle's locked version.**

Each anomaly: `severity: warn|block`, `rule_id: <code>`, `description_md`, `suggested_action_md`. Block-severity anomalies must be resolved (parameter fixed; signature obtained; reason documented) before the cycle can advance from `review` → `signed`.

**Two-step sign-and-execute commit.**

1. **HR/Ops Lead review.** Opens `/rew/payroll/<cycle_month>` (FR-REW-005). Sees per-Member rows + anomalies. Resolves blocking anomalies (or skips warn-level with documented justification). Signs.
2. **Founder counter-sign.** Reviews the HR/Ops-signed cycle; counter-signs. Step-up auth required. The cycle status transitions `review → signed`.
3. **Payslip-PDF generation.** Each `payroll_record` produces a signed PDF (Vietnamese-language; using the docx skill / a templated layout):
   - Header: company info + cycle month + Member info.
   - Earnings: P1 + P2 + P3 + reimbursable detail.
   - Deductions: SI/health/unemployment/PIT with statutory rate references.
   - Net.
   - Computation trace summary (one-line per step).
   - QR code linking to `/rew/payslip/<id>` for the Member's verification (auth-gated).
   - The PDF is hash-signed (SHA-256 + Ed25519 with the tenant's payslip-signing key); `payslip_pdf_signature_sha256` recorded.
   - Stored under the `hr_secure` KMS key.
4. **Bank disbursement file generation.** Produces a CSV / IBank format file per the configured bank (Vietcombank IBank / Techcombank Corp Pay / fallback CSV). Each line: Member's bank account + amount + memo. The file itself is encrypted; the HR/Ops Lead downloads + uploads to the bank portal manually (Vietnamese banks lack public payroll APIs as of 2026).
5. **Status → paid_out.** When the HR/Ops Lead confirms the bank upload + receives confirmation refs, the cycle transitions `signed → paid_out`.

**Rollback.**

Within 7 days of `paid_out` (and only if no actual bank disbursement has been processed), the cycle can be rolled back: status `paid_out → rolled_back`; payslip PDFs marked `rolled_back: true` (preserved for audit; not deleted); a corrective cycle is run. After 7 days, corrections are per-Member adjustments via FR-TIME-001's adjustment pattern + a one-off corrective payroll record in the next cycle.

**Member-side payslip view.**

`/rew/my/payslips` (FR-REW-005) lists the calling Member's signed payslips:
- PDF download (auth-gated; step-up).
- Inline rendering (with the EU AI Act Article 50 disclosure on the AI-narrator section, which is FR-REW-005's read-only narrative).
- Verification badge (the SHA-256 + Ed25519 signature is verifiable by an external party with the public key).

**Compliance Cockpit panel.**

- Cycle status + anomaly counts per cycle.
- Time-to-sign metrics (HR/Ops sign latency; Founder sign latency).
- Anomaly-rule-firing distribution (which rules fire most often).
- Force-reduction events.
- Bank-confirmation latency.

**MCP tool surface (read-only).**

- `cyberos.rew.list_cycles(status?)` — read; aggregate visibility (no per-Member amounts).
- `cyberos.rew.get_cycle(cycle_month)` — read; aggregate.
- `cyberos.rew.list_anomalies(cycle_month)` — read; HR/Ops + Founder + DPO.
- `cyberos.rew.my_payslip(cycle_month)` — read; calling Member's own; step-up at gateway.

There are **no mutation MCP tools**. Cycle commands (open, compute, sign, paid_out) are HR/Ops + Founder UI surfaces only.

## Alternatives Considered

- **Use a hosted payroll provider (MISA, KiotViet, Tanca).** Considered. Rejected: residency + the Total Rewards Appendix's specific structure (3P + BP + roll-forward + P1-protection) cannot be encoded; the tenant data leaks; lock-in. The platform may eventually integrate downstream with a hosted bank-disbursement service at P3 if a Vietnamese bank exposes a public API.
- **Skip anomaly detection; rely on HR/Ops eyeballing.** Rejected: P2 → P3 exit gate explicitly requires zero anomalies escaped.
- **AI-driven anomaly detection.** Rejected: deterministic rules are auditable + reproducible; AI in the compensation compute path is forbidden (PRD §2.5).
- **Single-step founder-only sign.** Rejected: HR/Ops + Founder dual-sign is the floor (separation of duties).
- **Auto-upload to bank API.** Rejected for P2: Vietnamese banks lack reliable APIs; the human-upload step is the floor with a P3 reconsideration once banking APIs mature.

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate: 2 consecutive monthly cycles complete entirely inside REW with zero anomalies escaped to post-close discovery.
- **Anomaly catch rate.** Synthetic test cycle (intentionally injected anomalies) catches all 100% of injected anomalies.
- **Latency NFR.** Compute pipeline ≤ 4 minutes for the 10-employee cycle.
- **Signature compliance.** 100% of `paid_out` cycles signed by HR/Ops + Founder with step-up evidence.

## Scope

**In-scope.**
- The 3 schema additions (`payroll_cycle`, `payroll_record`, `payroll_disbursement_file`).
- Compute pipeline (deterministic; no AI).
- Anomaly detector with 10+ rules.
- Two-step sign-and-execute commit flow.
- Payslip PDF generator with SHA-256 + Ed25519 signature.
- Bank disbursement file generator (Vietcombank + Techcombank + CSV fallback).
- 7-day rollback window.
- Member's payslip view (read-only; step-up).
- Compliance Cockpit panel.
- The 4 read-only MCP tools.
- Audit integration in scope `rew.payroll.{tenant}`.

**Out-of-scope (deferred).**
- Direct bank-API auto-upload (P3 if Vietnamese banks publish reliable APIs).
- Multi-currency payroll (P3 — international hires).
- Tax-filing automation with the Vietnamese tax authority (P3).
- Year-end personal-income-tax reconciliation report (P2; in FR-REW-004's scope addendum).

## Dependencies

- FR-HR-001 / FR-REW-001 / FR-REW-002.
- FR-REW-004 (SI/PIT engine — co-shipped).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-TIME-003 (reimbursable expenses feed).
- FR-CP-001 (Compliance Cockpit panel).
- FR-OBS-001 / FR-OBS-002.
- The `docx` / `pdf` skill for payslip PDF generation.
- A Vietnamese-language payslip template (legal counsel-approved).
- Compliance: Vietnamese Labour Code on wage protection (Article 95+); Vietnamese Tax Law on PIT calculation; PDPL Decree 13; EU AI Act high-risk classification (no AI in compute path); GDPR Article 22.
- Locked decisions referenced: DEC-174 (deterministic compute, no AI), DEC-175 (HR/Ops + Founder dual-sign required), DEC-176 (10-rule anomaly catalogue), DEC-177 (7-day rollback window).

## AI Risk Assessment

This FR explicitly forbids AI in the compute path. EU AI Act risk class: `high` (compensation domain).

### Data Sources

The compute is deterministic. No AI surface in this FR; the narrator (FR-REW-005) consumes the data downstream as `read_only` content with the Article 50 transparency chip.

### Human Oversight

- HR/Ops Lead reviews every cycle and signs.
- Founder counter-signs after HR/Ops.
- Block-severity anomalies cannot be silenced without resolution.
- Member receives a signed PDF + a step-up-authenticated dispute path.

### Failure Modes

- **Compute pipeline error mid-run.** Status remains `computing`; HR/Ops Lead can resume; partial results are not committed.
- **Anomaly detector false-positive.** HR/Ops can mark a warn-anomaly as resolved with a documented reason; block anomalies require parameter / data fix before commit.
- **Bank file format change** (the bank updates its IBank format). Mitigation: per-bank format specs are versioned + tested with a synthetic file on each parameter-version publish.
- **Payslip PDF signature key compromise.** Mitigation: the signing key is in HashiCorp Vault under the `hr_secure` policy; rotation procedure documented; key-access audit-logged.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, compute pipeline, anomaly detector, sign-and-execute flow, failure modes.
- **Human review:** `@stephen-cheng` reviewed; legal counsel + Vietnamese accountant will review the payslip template + SI/PIT formulas (FR-REW-004) before P2 production.
