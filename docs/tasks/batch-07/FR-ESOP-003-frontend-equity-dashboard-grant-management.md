---
title: "ESOP — frontend remote at /esop (Member equity dashboard, founder grant management, valuation lifecycle)"
author: "@stephen-cheng"
department: design
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q2"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the ESOP Module-Federation remote at `/esop` consuming FR-ESOP-001 + FR-ESOP-002. Three primary surfaces: **Member equity dashboard** (`/esop/my`) showing grants, vesting schedule with timeline visualisation, current vested vs. unvested counts, most-recent-blessed valuation + per-share value, simulated current value of vested holdings, annual put-window status + put-request history; **HR/Ops + Founder grant management** (`/esop/admin/grants`) for creating + signing new grants; **Founder + Engineering Lead valuation lifecycle** (`/esop/admin/valuations`) for drafting + blessing valuation events with structured evidence references. All amount-bearing surfaces require step-up auth (FR-AUTH-003); the read-only simulator from FR-ESOP-002 surfaces inline. The frontend ties together the ESOP cluster + integrates with FR-REW for the put-payout-via-payroll path + FR-HR for the Member context.

## Problem

The ESOP schema + put lifecycle without a frontend produce no Member-side transparency — and equity transparency is the core property of the Bet 5 moat. Three failure modes:

- **Equity opacity.** A Member who can't see their own grant + vesting + current value loses trust in the equity component of compensation.
- **Founder grant friction.** Without a structured surface, every new grant is a manual workflow; the legal-document trail breaks.
- **Valuation drift.** Without a clear "draft → blessed" UI, valuation events get skipped or rushed.

## Proposed Solution

Vite + React 19 Module-Federation remote at `/esop` consuming the GraphQL surfaces from FR-ESOP-001/002.

**Member equity dashboard (`/esop/my`).**

Default for any Member with an active grant.

- **Header.** "Hi <preferred_name>. Your ESOP holdings."
- **Top-level cards.**
  - Total phantom shares granted (encrypted; revealed on step-up).
  - Vested as of today (count + percentage of total grant).
  - Unvested remaining.
  - Most-recent blessed valuation: `12,500 VND/share (as of 2026-Q2)`.
  - Simulated current value of vested holdings (gross; via FR-ESOP-002 simulator).
- **Vesting timeline visualisation.**
  - Horizontal timeline: x-axis = grant_date through grant_date + 48 months.
  - Markers at cliff (Year 1, 25%) + each monthly tranche thereafter.
  - Today's date marker.
  - Hover any marker → "On 2026-09-15, 41 shares vested; cumulative 458 shares."
  - Future markers shown faded.
- **Grant detail card.**
  - Grant date + vesting start.
  - Plan reference (clickable; opens read-only plan terms).
  - Signed-by chain.
  - Reason / context (the Member-visible portion of the grant rationale).
- **Annual put-window card** (shown when eligible, i.e. Year 3+).
  - Window opens / closes dates.
  - Eligible shares this window.
  - Already put this window.
  - Remaining.
  - "Submit put request" CTA (opens the put-request form with simulator inline).
- **Past put requests.**
  - Status timeline per request (submitted → review → approved → settled).
  - Per-request: shares, gross, tax, net, settlement date.
- **Read-only simulator panel.**
  - "What if I put X shares today?"
  - Slider or numeric input; live updates.
  - The narrative + the structured payout breakdown.
  - The "AI-generated · CUO/CHRO v0.X · descriptive only" disclosure chip.
- **Forfeiture history** (when applicable).
  - Any forfeiture events tied to a transition (probation failed, Bad Leaver) — with the documented basis.

Every reveal of an encrypted amount requires step-up auth.

**HR/Ops + Founder grant management (`/esop/admin/grants`).**

For HR/Ops Lead + Founder + DPO.

- **Grants table.** All active grants; per-row: Member, total shares, grant date, vesting status (cliff / partial / fully vested), most-recent activity.
- **"Issue new grant" form.**
  - Select Member (auto-populated with active employees not yet granted, or grant-extension flag).
  - Select plan (the active version is default).
  - Total phantom shares (with the cap-remaining indicator).
  - Vesting start date (defaults to grant date or hire date).
  - Vesting schedule (defaults to plan default; override option for special cases).
  - Reason / context (Markdown).
  - Founder sign action; opens step-up.
  - On founder sign: status `signed_by_founder_at`; the grant goes pending Member countersign.
- **Pending Member countersigns.** Grants where the founder signed but the Member has not yet countersigned; reminder Notifies surfaced to the Member.
- **Forfeiture management.** When FR-REW-007 records a Bad Leaver, the corresponding forfeiture is auto-staged here; HR/Ops + Founder + DPO sign + legal-counsel-ref.

**Founder + Engineering Lead valuation lifecycle (`/esop/admin/valuations`).**

For HR/Ops + Founder + Engineering Lead + DPO + Auditor.

- **Valuation timeline.** Chronological list of `esop_valuation_event` rows with status chips.
- **"Draft new valuation" form.**
  - Valuation date.
  - Total company valuation (encrypted; founder + Engineering Lead enter via step-up).
  - Per-phantom-share value computed automatically from `total_authorised_shares`.
  - Currency.
  - Basis selector (internal_board_review / external_409a / fundraising_round / secondary_transaction).
  - Evidence references (board-review minutes link; 409A report blob ref; term sheet ref).
  - Save draft action.
- **Sign + bless flow.**
  - Founder reviews + signs.
  - Engineering Lead reviews + signs.
  - Legal counsel ref captured (mandatory for external_409a / fundraising_round / secondary_transaction).
  - "Bless" action transitions status `draft → blessed`; immutability trigger thereafter rejects modifications.
- **Cadence reminder.** If `now() - last_blessed_valuation > 18 months`, surface a sev-2 alert + Notify card to founder + Engineering Lead suggesting a fresh valuation.

**Founder views.**

A subset focused on:
- Awaiting-counter-sign queue (grants pending Member countersign).
- Valuation publish queue.
- Put-request approval queue (founder + DPO + legal-counsel-ref).
- Forfeiture sign queue.
- Compliance Cockpit deep-link for ESOP-related metrics.

**Vietnamese-locale rendering.**

- vi-VN default for the canonical CyberSkill tenant.
- Currency formatting per Vietnamese convention.
- Plan terms + Member explanations rendered in vi-VN by default.

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped.
- `/esop/my` first-paint ≤ 1.5 s on 4G.
- Step-up reveal of encrypted amounts ≤ 2 s p95.
- Simulator response ≤ 4 s p95.

**MCP tool surface.**

(All read-only; no mutation MCP for ESOP — UI + step-up + multi-party sign only.)

- `cyberos.esop.my_dashboard_payload` — read; calling Member's own; step-up.
- `cyberos.esop.admin_grants_summary` — read; HR/Ops + Founder + DPO; aggregate view.
- `cyberos.esop.admin_valuations_summary` — read; HR/Ops + Founder + Engineering Lead + DPO + Auditor.

## Alternatives Considered

- **Skip the Member dashboard; deliver per-quarter PDF statements only.** Rejected: continuous transparency is a core property of the equity component; PDFs are too coarse.
- **Allow per-grant custom vesting schedules in the standard form.** Considered. P2 ships standard plan-default-only; custom schedules require explicit founder-driven override (rare).
- **AI-suggested put timing in the simulator.** Rejected: explicit prohibition.
- **Real-time valuation updates.** Rejected: valuations are point-in-time blessed events; daily fluctuation isn't meaningful for phantom stock.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder issues grants for the 10 employees via the admin UI; (2) every Member opens `/esop/my` + completes step-up + sees their grant + simulator; (3) the founder + Engineering Lead bless the first valuation event; (4) a synthetic Year-3+ Member submits a put request through the form; the multi-party sign chain executes; settlement feeds the next payroll.
- **Adoption metric.** Every active Member opens `/esop/my` at least quarterly; ≥ 80% understand their grant after the first walkthrough (sampled survey).
- **Latency NFR.** Per the budgets above; bundle ≤ 50 KB.

## Scope

**In-scope.**
- The Module-Federation remote at `/esop`.
- Member equity dashboard with vesting timeline + simulator.
- HR/Ops + Founder grant management UI.
- Founder + Engineering Lead valuation lifecycle UI.
- Forfeiture management UI integrated with FR-REW-007.
- Founder counter-sign queue.
- Vietnamese-locale rendering.
- Step-up enforcement on every encrypted reveal.
- The 3 read-only MCP tools.
- Audit integration in scope `esop.ui.{tenant}`.

**Out-of-scope (deferred).**
- Liquidity-event distribution UI (P3).
- Mobile native (P3).
- Custom vesting schedules in standard form (P3).
- Cap-table visualisation showing the founder's holding + collective Member holdings (P3 — useful for fundraising prep).
- Public ESOP plan terms surface for prospective hires (P4 — recruiting integration).

## Dependencies

- FR-ESOP-001 / FR-ESOP-002.
- FR-HR-001 / FR-REW-007 (Member context + termination integration).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002 (Notify cards; CUO/CHRO simulator).
- FR-OBS-001 / FR-OBS-002.
- FR-CP-001 (Compliance Cockpit deep-link).
- Compliance: PDPL Decree 13 (equity data is highest-sensitivity personal data); EU AI Act Article 50 (simulator surface renders disclosure chip); GDPR Article 22.
- Locked decisions referenced: DEC-215 (Member dashboard with vesting timeline), DEC-216 (step-up on every encrypted reveal), DEC-217 (no real-time valuation).

## AI Risk Assessment

The simulator is the AI surface (inherits FR-ESOP-002's classification). The frontend itself is deterministic UI. EU AI Act risk class: `limited` for the consumer surface (high for the underlying compensation domain).

### Data Sources

UI consumes ACL-scoped GraphQL data. Simulator runs through the AI Gateway with persona-stamping. Per-tenant residency.

### Human Oversight

- All mutations go through the parent FRs' multi-party sign chains.
- AI-derived elements (simulator output) carry the disclosure chip + the explicit "descriptive only, not a recommendation" framing.
- The kill-switch from FR-GENIE-002 silences the simulator while preserving the deterministic compute path.

### Failure Modes

- **Simulator UI shows a recommendation by mistake.** Mitigated by the persona regression suite + the UI's framing chip; if the chip is missing on a render, the page fails the Storybook a11y + content-required checks.
- **Step-up bypass.** Mitigated by FR-AUTH-003 server-side enforcement.
- **Valuation timeline visualisation drift.** Mitigated by deterministic compute from `esop_vesting_event` rows; UI reads what the engine writes.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted dashboard layouts, grant + valuation lifecycle UIs, mobile + a11y considerations, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the founder + a vi-native Member will validate `/esop/my` in a usability walkthrough before P2 production.
