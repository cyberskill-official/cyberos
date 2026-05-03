---
title: "ESOP — Year-3+ put-option mechanics, Good/Bad Leaver branches integration, read-only AI valuation simulator"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: high
target_release: "P2 / 2027-Q2"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Implement the **put-option lifecycle**: from Year 3+ a Member with vested phantom shares can **exercise a put option** to redeem up to 33% per year of their vested holdings for cash at the most-recent-blessed valuation (FR-ESOP-001's `esop_valuation_event`). The put flow is **request → review → settlement** with founder + DPO + legal-counsel-ref signatures. **Good Leaver / Bad Leaver branches** integrate with FR-REW-007 — Good Leavers retain vested phantom shares + can exercise puts on a one-year transition window; Bad Leavers forfeit all (vested + unvested) per the plan. **Liquidity event mechanics** (P3 stub; the architecture is forward-compatible) for M&A or fundraising-secondary triggers convert vested phantom shares to actual cash payout. The **read-only AI simulator** is a CUO/CHRO surface that produces "what-if" narrative — "if you put 33% of your vested 1,000 shares today at the current 12,500 VND/share valuation, your gross before tax would be 4,125,000 VND; after Vietnamese personal-income-tax-on-equity-payout (2.5% per current rules), your net would be 4,021,875 VND" — but never *recommends* action. The simulator runs through the AI Gateway with persona-stamping; the persona-scope contract excludes any tool that could mutate the put-option lifecycle.

## Problem

PRD §9.17 names "put options, Good Leaver / Bad Leaver branches" + "valuation, put options" as P2 scope plus the "SP put-option simulator (read-only AI)" in PRD §14.3.1 P2 scope. Three failure modes the platform must structurally prevent:

- **Premature put-option exercise.** A Member exercising before Year 3 violates the plan; the eligibility gate must be encoded.
- **Bad Leaver retains vested phantom shares.** PRD §2.3 Bet 5 + the Total Rewards Appendix's Bad Leaver clause specify forfeiture; the FR-REW-007 termination flow must trigger forfeiture on Bad Leaver classification.
- **AI recommends "you should put now / wait."** Compensation-decision recommendation is forbidden; the simulator can describe what would happen, never recommend.

## Proposed Solution

The shape of the answer is `hr_secure.esop_put_*` schema + the put-option lifecycle + the FR-REW-007 termination integration + the read-only simulator.

**Schema (extending FR-ESOP-001).**

```sql
-- Put-option request.
CREATE TABLE hr_secure.esop_put_request (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  grant_id UUID NOT NULL REFERENCES hr_secure.esop_grant(id) ON DELETE RESTRICT,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  request_kind TEXT NOT NULL,                                          -- "annual_window_put" | "good_leaver_transition_put"
                                                                       -- | "liquidity_event_distribution"
  shares_to_redeem BIGINT NOT NULL,
  valuation_event_id UUID NOT NULL REFERENCES hr_secure.esop_valuation_event(id),
                                                                       -- the valuation used for pricing
  per_share_value_minor_encrypted BYTEA NOT NULL,                      -- snapshotted at request
  gross_payout_minor_encrypted BYTEA NOT NULL,
  tax_withholding_minor_encrypted BYTEA NOT NULL,                       -- per Vietnamese Decree on equity payout taxation
  net_payout_minor_encrypted BYTEA NOT NULL,
  status TEXT NOT NULL DEFAULT 'submitted',                              -- "submitted" | "review" | "approved"
                                                                       -- | "settled" | "denied"
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  reviewed_by UUID,
  reviewed_at TIMESTAMPTZ,
  signed_by_founder_at TIMESTAMPTZ,
  signed_by_dpo_at TIMESTAMPTZ,                                         -- because put-option settlement is a payment movement
  signed_by_legal_counsel_ref TEXT,
  settled_at TIMESTAMPTZ,
  payment_method TEXT,                                                  -- "next_payroll" | "direct_transfer" | "company_card"
  payment_reference TEXT,
  denial_reason_md_encrypted BYTEA,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX esop_put_request_member_idx ON hr_secure.esop_put_request (tenant_id, employee_id, status);

-- Annual put-window per Member (eligibility tracker).
CREATE TABLE hr_secure.esop_annual_put_window (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  year INT NOT NULL,
  total_eligible_shares BIGINT NOT NULL,                               -- max(33%) of vested as of year-start
  shares_already_put BIGINT NOT NULL DEFAULT 0,
  shares_remaining_in_window BIGINT NOT NULL,
  window_opens_at DATE NOT NULL,
  window_closes_at DATE NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, employee_id, year)
);
```

**Annual put-window mechanics.**

A scheduled job at year-start (1 January or per-tenant configurable) opens the put-window per Member:

1. Compute total vested shares as of year-start (sum across active grants).
2. Determine eligibility:
   - If the Member's `vesting_start_date + 3 years <= year_start` → eligible; the Year-3 floor met.
   - Otherwise: window not opened for this Member this year.
3. Compute `total_eligible_shares = floor(total_vested * 0.33)`.
4. Insert `esop_annual_put_window` row.
5. Notify the Member via Genie panel (FR-GENIE-001) with explanatory text + link to `/esop/my` (FR-ESOP-003).
6. The Member can submit one or more `esop_put_request` rows during the year; the cumulative `shares_already_put` must not exceed `total_eligible_shares`.

**Put request lifecycle.**

1. **Submission.** Member opens `/esop/my/put-request`; selects the grant + the number of shares to redeem; the simulator (next section) shows the projected payout. The Member submits.
2. **Eligibility check.** Server-side validates: window is open; cumulative does not exceed the cap; the grant is `active`; the Member is `current_status: active` (not `notice_period` for involuntary; covered by termination integration below).
3. **Pricing.** The most-recent-blessed `esop_valuation_event` is snapshotted. `per_share_value` + `gross_payout` computed. Vietnamese tax withholding (currently 2.5% on equity disposal under Article 13 Income Tax Law as of 2026) computed deterministically. Net payout computed.
4. **Review.** HR/Ops Lead reviews; founder + DPO + legal-counsel-ref signatures gather (legal-counsel-ref required because equity disposal is high-stakes).
5. **Settlement.** On all signatures, the put is `approved`. The next payroll cycle (FR-REW-003) includes the gross + the tax withholding + the net (paid to the Member's bank account). Status `→ settled`.
6. **Vested-share decrement.** The grant's cumulative-vested counter is reduced by the redeemed shares; corresponding `esop_vesting_event` rows are marked with metadata `redeemed_via_put: <put_request_id>` (the events themselves are immutable; metadata adds the trace).
7. **Annual window updated.** `shares_already_put` increased.

**Good Leaver / Bad Leaver integration with FR-REW-007.**

When FR-REW-007 records a termination:

- **Good Leaver.** A "good_leaver_transition" put-window opens for 12 months post-termination; the Member can exercise puts on their vested shares during this window (subject to the same 33%/year cap, prorated for partial years). After the window closes, vested-but-unputted shares forfeit (the company "buys back" at zero — the standard phantom-stock pattern; documented in the plan).
- **Bad Leaver.** All shares (vested + unvested) forfeit immediately upon Bad Leaver classification. A `forfeiture_event` is recorded; the founder + DPO + legal-counsel-ref signatures captured. The Member receives a Notify + a copy of the forfeiture documentation.
- **Probation Failed.** Same as Bad Leaver — vested + unvested forfeit (typical at sub-1-year tenure when no cliff has been crossed; the impact is usually nil).
- **Liquidity event** (P3 stub; M&A or secondary). Vested shares are converted to cash at the event's per-share value; settlement is the responsibility of the corporate event's flow, not the per-Member put.

**Read-only valuation simulator.**

A `cyberos.esop.simulate_put` MCP tool (read-only):

Input: `grant_id`, `shares_to_redeem`, `valuation_basis: "current"|"specific_event_id"`.

Output: a structured + narrative simulation:

```json
{
  "computation": {
    "shares_to_redeem": 1000,
    "per_share_value_minor": 12500,
    "gross_payout_minor": 12500000,
    "tax_withholding_pct": 0.025,
    "tax_withholding_minor": 312500,
    "net_payout_minor": 12187500,
    "currency": "VND"
  },
  "narrative_md": "If you put 1,000 vested phantom shares at the current blessed valuation (12,500 VND per share, dated 2026-Q2), your gross would be 12,500,000 VND. Vietnamese personal-income-tax on equity disposal (2.5% per current rules) would withhold 312,500 VND, leaving a net of 12,187,500 VND. This would be paid via the next payroll cycle. Your annual put window has 4,000 shares remaining after this exercise.",
  "eligibility": {
    "eligible": true,
    "remaining_in_window": 5000,
    "after_this_request_remaining": 4000
  },
  "persona_version": "cuo-chro-v0.X",
  "ai_disclosure_id": "..."
}
```

The simulator **never recommends** ("you should put now"; "wait until next year"; "this is a good time"). The persona's prompt forbids prescriptive language; adversarial regression suite gates this. The disclosure chip is on every render.

**Persona scope contract.**

CUO/CHRO declares for the put-simulator path:
- `tools_allowed`: `cyberos.esop.simulate_put` (read), `cyberos.esop.my_grants` (read), `cyberos.esop.my_vesting_schedule` (read), `cyberos.esop.list_blessed_valuations` (read).
- `tools_forbidden_explicit`: any mutation tool; any cross-Member equity read (a Member cannot ask "how much equity does Khoa have?" — even via the simulator).

**Vietnamese tax withholding integration.**

The put-option payout's tax withholding is computed via FR-REW-004's statutory engine extended with the equity-disposal rate (Article 13 Income Tax Law, Vietnamese Decree 126/2020/NĐ-CP). The rate is parameter-version-locked + reviewed annually by the company's external accountant.

**Audit integration.**

`esop.put.{tenant}` audit scope. Every put-request lifecycle event audit-logged. The Compliance Cockpit panel surfaces:
- Active annual put-windows.
- Open put-requests by status.
- Year-to-date settled put-requests (aggregate).
- Forfeiture events (should be exceedingly rare, except probation-failed cases).

**MCP tool surface.**

- `cyberos.esop.list_my_put_requests(status?)` — read; calling Member's own; step-up.
- `cyberos.esop.get_my_annual_put_window(year?)` — read; step-up.
- `cyberos.esop.simulate_put(grant_id, shares, valuation_basis)` — read; the simulator; step-up.
- `cyberos.esop.list_open_put_requests` — read; HR/Ops + Founder + DPO; aggregate-only at this surface.

There are **no mutation MCP tools** for put requests or forfeitures. UI + step-up + multi-party sign chain only.

## Alternatives Considered

- **Allow puts before Year 3.** Rejected: plan term; eligibility gate is the floor.
- **Higher than 33%/year cap.** Rejected: plan term; the cap protects company cash.
- **AI-recommended exercise timing.** Rejected: explicit prohibition; the simulator is descriptive only.
- **Skip Vietnamese tax withholding.** Rejected: legal compliance.
- **Skip Good Leaver transition window; vested shares forfeit on departure.** Rejected: the contract specifies a transition window; skipping breaks the social contract.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) annual put-window opens for synthetic Member with 5+ years tenure; (2) Member submits a put-request via `/esop/my`; eligibility validated; simulation matches the deterministic compute; (3) founder + DPO + legal-counsel-ref signatures gathered; settlement feeds the next payroll; (4) FR-REW-007 Good-Leaver termination opens transition put-window for 12 months; (5) FR-REW-007 Bad-Leaver termination records forfeiture event; (6) the simulator regression suite catches an adversarial prompt asking for a recommendation.
- **Compliance metric.** Zero premature put-exercises; zero Bad-Leaver-retained vested shares; zero AI recommendations from the simulator; zero equity values in BRAIN.
- **Latency NFR.** Simulation runs ≤ 4 s p95.

## Scope

**In-scope.**
- The 2 schema additions (`esop_put_request`, `esop_annual_put_window`).
- Put-window opening scheduled job.
- Put-request lifecycle with multi-party sign chain.
- FR-REW-007 termination integration (Good Leaver transition window; Bad Leaver forfeiture).
- Vietnamese equity-disposal tax withholding (parameter-version-locked rate).
- Read-only valuation simulator with persona-scope contract + adversarial regression suite.
- The 4 read-only MCP tools.
- Audit + Compliance Cockpit integration.

**Out-of-scope (deferred to FR-ESOP-003).**
- Frontend remote at /esop (FR-ESOP-003).
- Liquidity-event distribution flow (P3 — when M&A / secondary happens).
- Multi-currency put-option payouts (P3+ — international Members).
- ESOP migration from external systems (P3+ — Carta integration).

## Dependencies

- FR-HR-001 / FR-REW-001 / FR-REW-003 / FR-REW-004 / FR-REW-007 (substrate + payroll feed + statutory engine + termination integration).
- FR-ESOP-001 (schema + plan + grants + vesting + valuations).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-CP-001 (Compliance Cockpit).
- FR-GENIE-001 / FR-GENIE-002 (Notify cards; CUO/CHRO simulator; persona-scope).
- External legal counsel for sign-and-publish path on each put.
- The Vietnamese accountant's review of the equity-disposal tax rate.
- Compliance: Vietnamese Income Tax Law Article 13 + Decree 126/2020/NĐ-CP (equity disposal tax); PDPL Decree 13; EU AI Act Articles 5-7 high-risk classification (compensation domain — no AI in compute or recommendation); GDPR Article 22; SOC 2 CC6.
- Locked decisions referenced: DEC-211 (annual put-window with 33%/year cap), DEC-212 (Good-Leaver 12-month transition put-window; Bad-Leaver immediate forfeit), DEC-213 (read-only simulator; no recommendation), DEC-214 (per-Member step-up on every reveal).

## AI Risk Assessment

The valuation simulator is the AI surface; the put lifecycle is fully deterministic. EU AI Act risk class: `high` (compensation + equity domain).

### Data Sources

The simulator reads the Member's grants + vesting schedules + the most-recent-blessed valuations. No third-party data; per-tenant residency. No cross-Member data.

### Human Oversight

- Put-requests require multi-party sign chain (Member submit; HR/Ops review; founder + DPO + legal-counsel-ref).
- The simulator describes; the human decides.
- Forfeitures trigger only via FR-REW-007 termination flow with its own multi-party sign.
- The kill-switch from FR-GENIE-002 silences the simulator (the deterministic compute remains for the human-only manual path).

### Failure Modes

- **Simulator recommends action.** Caught by adversarial regression suite; persona-version blocked.
- **Put-window cap exceeded.** Caught at server-side eligibility check; UI shows the remaining cap.
- **Good Leaver transition window expires unnoticed.** Mitigation: the window opening triggers a Notify + 30-day-before-close reminder + 7-day-before-close reminder.
- **Vietnamese tax rate change mid-window.** New parameter version takes effect at next year's window; in-flight requests use the rate at submission time.
- **Bad Leaver retains shares due to bug.** Mitigation: FR-REW-007's signed forfeiture event triggers the deterministic forfeiture write; trigger-protected.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted put lifecycle, Good/Bad Leaver integration, simulator architecture, persona-scope contract, failure modes.
- **Human review:** `@stephen-cheng` reviewed; legal counsel + Vietnamese accountant will review the put-option flow + equity-tax encoding before P2 production.
