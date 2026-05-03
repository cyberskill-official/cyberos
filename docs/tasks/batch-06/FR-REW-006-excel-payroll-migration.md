---
title: "REW — migration from existing Excel payroll: historical reconstruction, salary history backfill, BP balance seeding, validation drill"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: high
target_release: "P2 / 2027-Q1"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Migrate the team's existing Excel-based payroll history into REW so the platform takes over as the system-of-record at P2 → P3 cutover. Migration covers: **salary history backfill** (every Member's P1 + P2 trajectory back to hire date, with `signed_by_founder_at` reconstructed from the founder's records); **historical payslips** (PDF or scanned-paper records imported into the content-addressed blob store); **BP balance seeding** (the founder's records of Bonus Points earned + remaining balance per Member, signed by the founder + engineering lead); **statutory profile capture** (zone + dependents + SI start dates per Member); **closing the prior-Excel month** (the last Excel-paid cycle is recorded as `migrated` status; the next cycle runs in CyberOS); **validation drill** comparing the next CyberOS-run cycle's outputs against the prior-Excel cycle's outputs (any deviation > 1% requires investigation); and **rollback safety** — within 7 days of cutover, the migration can be unwound; after 7 days, the prior Excel becomes archive-only. The entire migration runs under step-up auth; every reconstructed historical record is signed by the founder confirming "this matches my records." This is the highest-stakes migration in the entire P2 program.

## Problem

The team's payroll history sits in a founder-maintained Excel workbook. Three failure modes the platform must avoid:

- **Lost historical context.** A 5-year record of Member salary changes in Excel cannot transparently feed REW's anti-retroactive parameter-version contract; a reconstruction is the floor.
- **First-CyberOS-cycle anomaly drift.** Without a validation drill comparing the first CyberOS cycle to the last Excel cycle, an engine bug could produce systematically wrong amounts that no one notices for months.
- **BP balance opacity.** The founder maintains a mental + Excel model of who's earned how many Bonus Points; without explicit reconstruction, Members at cutover would lose their accumulated standing.

PRD §14.3.2 P2 → P3 exit gate: "Payroll cycle close has been completed entirely inside REW module for at least 2 consecutive cycles, with zero anomalies escaped." Migration is a precondition.

## Proposed Solution

The shape of the answer is `cyberos-rew-migrate` CLI + service implementing a multi-phase migration with validation drill + reversible cutover.

**Phase 1 — Pre-migration data collection.**

The HR/Ops Lead + Founder collaborate via a structured Excel template the platform provides (`templates/rew-migration/REW_MIGRATION_v1.xlsx`):

- **Sheet 1: Employees.** One row per Member: legal name, hire date, current zone, dependents count, current P1, current P2, current contract kind.
- **Sheet 2: Salary history.** Per Member, a chronological list of P1/P2 changes with dates + reasons.
- **Sheet 3: BP earnings.** Per Member, per quarter, points earned + source.
- **Sheet 4: BP redemptions.** Per Member, per quarter, points redeemed + cash payout.
- **Sheet 5: Statutory deductions historical.** Per Member, per cycle, gross + SI/HI/UI/PIT actuals. (Used for the validation drill.)
- **Sheet 6: Reimbursable expenses paid via payroll.** Aggregated history.
- **Sheet 7: Notes.** Per Member, any one-off corrections or adjustments not captured above.

The founder fills the template against the existing Excel (a 2-3 hour exercise for the 10-employee team); the HR/Ops Lead reviews + flags inconsistencies for founder resolution.

**Phase 2 — Import + reconstruction.**

The `cyberos-rew-migrate import` command:

1. **Validate.** Schema check on the input workbook; reject on missing fields or inconsistent dates.
2. **Create parameter version v0.** A historical placeholder for everything before CyberOS — labelled `migrated_history_v0`; published with `signed_by_founder_at` + `signed_by_engineering_lead_at` + a special `legal_counsel_ref: "internal-migration-2026-Q1"` flag indicating it's a reconstructed record. Future versions will reference this as their base predecessor.
3. **Backfill `hr_secure.salary` rows.** For each Member, walk Sheet 2 chronologically; create one `hr_secure.salary` row per change with the historical amounts (envelope-encrypted under the `hr_secure` KMS key); set `effective_from` to the change date; link `superseded_by` chain. Each row's `signed_by_founder_at` is set to the migration timestamp (rather than the original change date — the row is *historical reconstruction*, not a backdated sign).
4. **Backfill `hr_secure.bp_fund_quarter` and `hr_secure.bp_balance` rows.** From Sheet 3 + Sheet 4. The fund cash for each historical quarter is the founder's recorded "what we put into the BP pool"; the per-Member earned points come from VP evaluations + ad-hoc grants.
5. **Backfill `hr_secure.bp_earning_event` rows.** From Sheet 3, with `source: 'migrated_historical'`; `signed_by_founder_at` = migration timestamp.
6. **Backfill `hr_secure.statutory_profile` rows.** From Sheet 1. Every Member must have one before any cycle can run.
7. **Backfill historical `hr_secure.payroll_record` rows** (optional). From Sheet 5. These are *informational* — they don't drive any new computation but populate the Member's `/rew/my` history view back to hire date. If the founder's Excel doesn't have per-month detail, backfill at quarter-aggregate granularity.
8. **Mark prior cycle.** The last Excel-paid month is recorded as `hr_secure.payroll_cycle{cycle_month: <last>, status: 'migrated', metadata: { source: 'excel-2024-2026' }}`.
9. **Audit row trail.** Each backfilled row writes an audit row in scope `rew.migration.{tenant}` with `source_kind: 'historical_reconstruction'` so the Compliance Cockpit clearly distinguishes reconstructed-historical vs. CyberOS-native records.

The full import for the 10-employee team is expected to complete in ≤ 30 minutes.

**Phase 3 — Sign-off.**

After import, the founder reviews the imported salary history + BP balances per Member via `/rew/admin/migration` and individually signs off:

- For each Member: the imported history rendered in chronological narrative form; the founder confirms "this matches my records."
- For each historical BP balance: the founder confirms.
- For the parameter version v0: the founder + engineering lead countersign.

Sign-off is required before the next CyberOS cycle can run.

**Phase 4 — Validation drill (the most-load-bearing step).**

Before the first real CyberOS-driven cycle:

1. **Re-run the prior month's Excel cycle in CyberOS using the imported parameters.** The compute pipeline (FR-REW-003) runs against the historical data; outputs are produced but NOT committed (mode: `dry_run`).
2. **Compare row-by-row** to the actual Excel-paid amounts (Sheet 5).
3. **Per-Member deviation tolerance.** Any deviation > 1,000 VND OR > 0.1% of gross (whichever larger) is flagged.
4. **Investigate every flag.** Possible causes: wrong dependent count; wrong zone; rounding-strategy mismatch; an ad-hoc adjustment in Excel that wasn't captured in Sheet 7; a statutory-rate-table delta. Each flag is resolved with HR/Ops Lead + founder + accountant; any remaining deviation > 0.1% blocks cutover.
5. **Sign the drill report.** Founder + engineering lead sign the drill outcome; the report is filed in CP-001's compliance archive.

**Phase 5 — Cutover.**

After Phase 4 sign:

1. **Open the next cycle in CyberOS** (the first real CyberOS cycle).
2. **Run the cycle** through FR-REW-003's full flow (compute → review → sign → paid_out).
3. **Continue for 2 consecutive cycles.** After 2 successful cycles, declare the P2 → P3 exit gate met.
4. **Excel becomes archive-only.** The founder retains the Excel as historical reference; no new entries; the team's payroll system-of-record is CyberOS.

**Phase 6 — Rollback safety.**

Within 7 days of cutover: the migration can be unwound — the imported records are flagged `rolled_back: true`; CyberOS cycles are cancelled; the team reverts to Excel for the next month; the migration is rescheduled. After 7 days, rollback requires a full architectural re-migration.

**Migration UI (`/rew/admin/migration`).**

Restricted to HR/Ops Lead + Founder + DPO.

- **Import status.** Per-sheet validation results; per-Member backfill progress.
- **Sign-off panel.** Per-Member historical review + founder sign action.
- **Drill report.** Per-Member dry-run vs. Excel comparison.
- **Cutover button.** Disabled until all sign-offs + drill pass; step-up auth.
- **Rollback button.** Active for 7 days post-cutover; step-up + DPO sign.

**MCP tool surface (read-only; very narrow).**

- `cyberos.rew.migration_status` — read; HR/Ops + Founder + DPO; aggregate.
- `cyberos.rew.migration_drill_report` — read; aggregate.

There are **no mutation MCP tools**. The migration runs via the CLI + the admin UI only; agent-driven migration of compensation data is forbidden.

**Compliance Cockpit.**

A dedicated panel during migration:
- Status: not_started / importing / awaiting_signoff / drill_in_progress / drill_passed / cut_over / archived.
- Per-Member sign-off progress.
- Drill report deviation distribution.
- 7-day rollback window countdown.

## Alternatives Considered

- **Skip migration; start CyberOS at zero history.** Rejected: Members lose their accumulated BP standing; year-over-year salary trajectory invisible; the platform doesn't earn trust as system-of-record.
- **Live two-way sync between Excel and CyberOS.** Rejected: spreadsheets are not a stable source-of-truth; the cutover is the floor.
- **Skip the validation drill; start fresh and reconcile if discrepancies emerge.** Rejected: P2 → P3 exit gate explicitly requires zero anomalies; the drill is the structural protection.
- **Allow AI to assist with reconstruction.** Rejected: same architectural prohibition as FR-REW-001..005 — compensation reconstruction is human-only.
- **Cutover without sign-off.** Rejected: founder sign-off is the legal commitment that the reconstruction matches the contract.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the 10-employee team's history fully reconstructed in CyberOS; (2) founder signs off every Member's history; (3) drill compares first-CyberOS-cycle dry-run to last-Excel-cycle within 0.1% deviation per Member; (4) cutover succeeds; (5) two consecutive CyberOS cycles run without anomaly escape.
- **Compliance metric.** Drill deviation > 0.1% on any single Member blocks cutover; resolved cause documented; re-drill until clean.
- **Latency NFR.** End-to-end migration (import + sign-off + drill + cutover) ≤ 5 business days for the 10-employee team.

## Scope

**In-scope.**
- Excel migration template at `templates/rew-migration/REW_MIGRATION_v1.xlsx`.
- `cyberos-rew-migrate` CLI + service.
- Multi-phase migration: import → reconstruct → sign-off → drill → cutover → rollback-window.
- Migration parameter version v0 with `legal_counsel_ref: "internal-migration-2026-Q1"`.
- Per-Member historical sign-off UI.
- Validation drill (dry-run + row-by-row compare with 0.1% tolerance).
- 7-day rollback window.
- `/rew/admin/migration` UI.
- Compliance Cockpit migration panel.
- The 2 read-only MCP tools.
- Audit integration in scope `rew.migration.{tenant}`.

**Out-of-scope (deferred).**
- Migration from non-Excel payroll systems (P3 if a customer engagement requires).
- Multi-currency historical reconstruction (P3 — international historical hires).
- Equity grant reconstruction (P3 — ESOP module).
- Year-end PIT reconciliation reconstruction beyond the current year (P3 — accountant-aided one-off if disputed).

## Dependencies

- FR-HR-001 / FR-REW-001 / FR-REW-002 / FR-REW-003 / FR-REW-004.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-CP-001 (Compliance Cockpit panel).
- The xlsx skill (template authoring + validation).
- The signed Total Rewards Appendix.
- The team's existing Excel payroll workbook (input).
- The Vietnamese accountant's review of the migration drill outputs.
- Compliance: PDPL Decree 13 (the migration is a personal-data-processing event); SOC 2 CC8 (change management); audit-grade preservation per CP-001.
- Locked decisions referenced: DEC-184 (migration template + drill), DEC-185 (0.1% deviation tolerance for cutover), DEC-186 (7-day rollback window), DEC-187 (no AI assistance in migration).

## AI Risk Assessment

The migration explicitly forbids AI in the path. EU AI Act risk class: `high` (compensation-domain migration; data is reconstructed historical compensation).

### Data Sources

The migration consumes the team's Excel + the founder's records. No third-party data; no AI inputs. Per-tenant residency.

### Human Oversight

- Every reconstructed record is founder-signed.
- The validation drill is the structural integrity check.
- Cutover requires explicit step-up.
- Rollback window is 7 days.
- DPO + Founder + HR/Ops Lead all part of the sign-off chain.

### Failure Modes

- **Excel data error.** Caught by validation step at import; the founder corrects + reimports.
- **Drill deviation > 0.1%.** Investigation finds the cause; cutover blocked until resolved.
- **Sign-off race condition.** UI surfaces partial sign-off state; a re-import overrides.
- **Cutover bug discovered post-cutover within 7 days.** Rollback path used.
- **Cutover bug discovered after 7 days.** Re-migration with new template version + re-cutover; the prior records are flagged `superseded_by_migration: <new-id>` and preserved for audit.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted multi-phase migration design, drill mechanics, rollback safety, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the actual migration template + the legal-counsel review of the reconstruction approach will happen before P2 deployment.
