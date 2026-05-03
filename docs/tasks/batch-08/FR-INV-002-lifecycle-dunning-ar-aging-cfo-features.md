---
title: "INV — invoice lifecycle, AR aging buckets, dunning email drafts (CUO/CFO), late-payer flags, multi-currency"
author: "@stephen-cheng"
department: finance
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Wire the **invoice lifecycle** (draft → sent → viewed → paid / overdue / void) with state-transition rules + audit + Notify cadence; **AR aging buckets** (current / 1-30 / 31-60 / 61-90 / 90+ days overdue) feeding the founder's weekly cash-flow dashboard; **dunning email drafts** authored by **CUO/CFO** (the Chief Financial Officer skill — new in this FR, alongside CSO from FR-OKR-002) when an invoice crosses 30 / 45 / 60 days overdue, with vi-VN / en-US locale per the client's CRM-recorded language preference; **late-payer flags** on CRM accounts (account-health degrades; CUO/CRO surfaces the signal in CRM); **multi-currency** with daily SBV-rate snapshots feeding tenant-currency reporting; **payment-reminder cadences** (gentle 5-day-before-due, firm 5-day-after-due, escalation 30-day-after-due). Read-only AI features only — CUO/CFO drafts dunning, suggests payment-plan options, narrates AR aging — but never compute invoice amounts, never auto-send, never auto-void. Architecturally consistent with REW + ESOP: AI describes, humans decide on financial state changes.

## Problem

Without lifecycle automation + AR aging visibility, the team's current pattern (manual tracking + ad-hoc dunning emails written by the founder) leaks revenue + slows collections. Three failure modes:

- **Slow dunning.** A 60-day-overdue invoice that nobody chased loses 1-2% of the team's monthly cash flow per occurrence.
- **No AR visibility.** "How much did clients owe us at the start of last month?" is unanswerable today; cash flow forecasting is gut-feel.
- **Tone-deaf dunning.** A formal English collection letter sent to a long-term Vietnamese client damages relationship; the dunning needs to honour the client's relationship + locale.

PRD §9.16 names "AR aging; ... tax compliance" + "dunning email drafts" in PRD §14.3.1. This FR wires them.

## Proposed Solution

The shape of the answer is the lifecycle state machine + the AR aging compute + the dunning draft pipeline + the multi-currency snapshot.

**Invoice lifecycle.**

States + allowed transitions:

```
draft → sent          (HR/Ops + Founder send action; step-up)
draft → void          (HR/Ops + Founder void; reason required)
sent → viewed_by_client (auto on client-portal open or open-tracking pixel; not authoritative)
sent → partially_paid (payment entry < total)
sent → paid           (payment entry >= total)
sent → overdue        (auto when due_date < today AND status = sent; daily job)
viewed_by_client → partially_paid / paid / overdue (same)
partially_paid → paid (subsequent payment closes the balance)
partially_paid → overdue (when due_date < today)
overdue → paid        (late payment received)
overdue → void        (very rare; reason required + Founder + DPO sign)
overdue → disputed    (when client formally disputes)
disputed → paid       (dispute resolved + payment received)
disputed → void       (dispute resolved in client's favour; Founder + DPO + legal-counsel-ref)
any → void            (with reason + Founder sign; cannot be undone except by issuing new invoice)
```

The state transitions are validated at mutation level (FR-INV-001's `invSendInvoice`, `invMarkInvoicePaid`, `invVoidInvoice`).

**AR aging buckets.**

A scheduled daily job at 06:00 ICT recomputes per-Engagement + per-Account + tenant-aggregate AR aging:

| Bucket | Definition |
|---|---|
| current | due_date >= today |
| 1-30 days overdue | due_date 1-30 days ago |
| 31-60 days overdue | due_date 31-60 days ago |
| 61-90 days overdue | due_date 61-90 days ago |
| 90+ days overdue | due_date > 90 days ago |

Per bucket: count of invoices + total amount (in tenant currency, normalized via daily SBV rate). Stored in `inv.ar_aging_snapshot` for time-series tracking.

**Dunning email drafts (CUO/CFO).**

The new **CUO/CFO** skill — Chief Financial Officer — joins CSO (FR-OKR-002), CRO (FR-CRM-003), CHRO (FR-HR/REW), CTO/CEO/COO from FR-GENIE-001. Persona authored at `~/.cyberos/skills/cuo/cfo/SKILL.md`; dual-signed.

When an invoice crosses 30 / 45 / 60 / 90 days overdue:
1. CUO/CFO reads: invoice + client account + recent CRM activity (FR-CRM-001 activities; was there a recent positive signal?) + Engagement-history (long-term retainer? new client?) + the client contact's preferred language (`crm.contact.language_default`).
2. Drafts a dunning email tailored to:
   - **30-day overdue** — gentle, relationship-first: "Just a friendly reminder, hope all is well; invoice INV-2026-001 was due 30 days ago. Our records show $X — would you like to confirm receipt or share when payment is expected?"
   - **45-day overdue** — firmer but warm: "Following up on INV-2026-001. We understand things come up; if there's an issue we should know about, please let us know. Otherwise, payment by [date 7 days out] would help us close out our books."
   - **60-day overdue** — formal escalation: "INV-2026-001 is now 60 days overdue. We need to discuss before this affects your account standing. Could we set up a 15-minute call this week?"
   - **90-day overdue** — formal-with-options: payment-plan suggestion + relationship recap + escalation warning.
3. The draft renders in `/inv/admin/dunning` (FR-INV-004) for HR/Ops Lead or Founder review.
4. **Never auto-sends.** The human reviews + edits + sends via FR-EMAIL-001's compose path (re-uses FR-EMAIL-005 vi-VN composition for Vietnamese clients).
5. The send action is a destructive-confirmation MCP call + an entry in `crm.activity` linking back to the invoice + audit-row in `inv.dunning.{tenant}`.

The dunning persona-scope contract:
- `tools_allowed`: `cyberos.inv.list_invoices` (read), `cyberos.inv.get_invoice` (read), `cyberos.crm.get_account` (read), `cyberos.crm.get_contact` (read), `cyberos.crm.list_activities` (read), `cyberos.email.compose_in_locale` (compose draft only), `cyberos.genie.draft_review` (Review-mode card).
- `tools_forbidden_explicit`: `cyberos.inv.void_invoice`, `cyberos.inv.mark_invoice_paid`, `cyberos.inv.send_invoice`, `cyberos.email.send_message` — every state-changing action goes through the human.

**Late-payer flags + CRM-side signal.**

When an invoice crosses 60 days overdue:
1. The client's `crm.account` gets a `metadata.late_payer_flag: { since: <date>, reason: "60_day_overdue" }`.
2. CRM account-health (FR-CRM-003) surfaces this in the health-score breakdown.
3. CUO/CRO + CUO/CFO collaboratively surface: a strategic decision card to the founder ("Acme's account standing is yellow; consider pause on upselling new work until INV-2026-001 resolves").
4. Repeat-offender pattern-detection: if the same account was 60+ days overdue on multiple invoices in 12 months, the flag elevates to `repeat_late_payer` + a stronger Notify.

**Multi-currency.**

Daily 06:00 ICT job:
1. Polls SBV (State Bank of Vietnam) published rates + ECB rates as fallback for non-VND.
2. Stores snapshots in `inv.fx_rate_snapshot{tenant_id, date, base_currency, target_currency, rate}`.
3. AR aging + cash-flow reports normalise to tenant currency using each invoice's `invoice_date` rate (historical rate, not current — preserves accounting integrity).
4. Multi-currency invoices retain their original currency on the PDF; reporting layers convert.

**Payment-reminder cadences (proactive, not dunning).**

For invoices in `sent` status:
- **5 days before due** — gentle reminder; CUO/CFO drafts; HR/Ops Lead reviews + sends.
- **On due date** — neutral notification; auto-sent (HR/Ops Lead pre-authorises via per-Engagement preference; otherwise Notify card to send).
- **5 days after due** — firm reminder (transitions to "1-30 day overdue" bucket; first dunning).

These are separate from the 30/45/60/90-day dunning escalations.

**Cross-module integrations.**

- **CRM activities.** Every dunning send + every reminder logs as `crm.activity{kind: "email_out", subject: "...", external_refs: [{ kind: "invoice", id: ... }]}` so the account 360 view (FR-CRM-002) shows the full collection history.
- **PROJ Engagement.** A repeated late-payer flag on the Account triggers a Notify to the Engagement's primary owner: "Consider scope/budget conversation with the client."
- **OBS.** AR aging trends published as Prometheus metrics for the founder's weekly dashboard.

**MCP tool surface.**

- `cyberos.inv.aging_report(as_of?, direction: "outbound"|"inbound")` — read; HR/Ops + Founder + Auditor.
- `cyberos.inv.list_overdue_invoices(bucket?, account_id?)` — read.
- `cyberos.inv.draft_dunning(invoice_id, escalation_kind?)` — read; calls CUO/CFO drafter; returns Review card.
- `cyberos.inv.draft_payment_reminder(invoice_id, kind: "before_due"|"on_due")` — read.
- `cyberos.inv.list_late_payer_flags(since?)` — read.
- `cyberos.inv.cash_flow_forecast(horizon_days)` — read; aggregated payment-due projections (deterministic; not AI).

**Audit + observability.**

- `inv.dunning.{tenant}` audit scope.
- Prometheus metrics: AR aging by bucket, total outstanding by direction, dunning-send count by escalation level, payment-reminder acceptance rate.
- OBS dashboard "Cash flow & AR" panel for Founder + HR/Ops Lead.

## Alternatives Considered

- **Auto-send dunning emails on schedule.** Rejected: relationship damage risk; human review is the floor.
- **Skip multi-currency snapshot; use current rate at report time.** Rejected: accounting integrity requires historical rates.
- **Single dunning template regardless of escalation level.** Rejected: tone matters; the four-stage ladder respects the relationship arc.
- **Bypass CRM linkage; treat invoices as standalone.** Rejected: the late-payer flag → CRM signal → account health is the architectural feedback loop.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) AR aging report renders for the team's 2 long-term clients with correct buckets; (2) a synthetic 30-day-overdue invoice triggers a CUO/CFO dunning draft in vi-VN; (3) the human reviews + edits + sends; the action logs as a CRM activity + audit row; (4) the SBV rate snapshot job runs successfully.
- **Adoption metric.** ≥ 90% of dunning sends route through CUO/CFO drafts (vs. founder writing from scratch); founder time on AR work reduced by ≥ 50% vs. pre-INV baseline.
- **Quality metric.** Average days-sales-outstanding (DSO) reduced by ≥ 7 days within 6 months of P2 launch.
- **Latency NFR.** AR aging report p95 ≤ 800 ms; CUO/CFO dunning draft p95 ≤ 6 s.

## Scope

**In-scope.**
- Invoice lifecycle state machine + transition validation.
- AR aging buckets + daily snapshot job.
- New CUO/CFO persona authoring + dual-sign.
- Dunning email draft pipeline (4 escalation levels).
- Payment-reminder pre-due + on-due cadence.
- Late-payer flag + CRM signal integration.
- Multi-currency daily rate snapshot.
- The 6 MCP tools.
- OBS dashboard panel.
- Audit integration in scope `inv.dunning.{tenant}`.

**Out-of-scope (deferred to FR-INV-003 / FR-INV-004).**
- Stripe + Wise + VNPay payment integrations + reconciliation (FR-INV-003).
- Frontend remote at /inv (FR-INV-004).
- Auto-send of pre-due reminders (P3 if acceptance metrics support it).
- AI-suggested payment plans beyond template offering (P3).
- Cross-tenant late-payer reputation pool (forbidden by design).

## Dependencies

- FR-INV-001 (schema).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-EMAIL-001..010 (composer + send path).
- FR-CRM-001 / FR-CRM-002 / FR-CRM-003 (account + activities + late-payer signal).
- FR-PROJ-007 (Engagement primary owner notification).
- FR-OBS-001 / FR-OBS-002.
- FR-GENIE-001 / FR-GENIE-002 (CUO/CFO persona).
- SBV published-rates feed.
- Compliance: Vietnamese accounting standards on AR + tax-period reporting; PDPL Decree 13; EU AI Act Article 50 (CUO/CFO drafts render disclosure chip); GDPR Article 22 (no automated collection decisions).
- Locked decisions referenced: DEC-230 (CUO/CFO emergent C-skill), DEC-231 (4-stage dunning ladder), DEC-232 (no auto-send of dunning).

## AI Risk Assessment

CUO/CFO dunning drafts + payment-plan suggestions are AI-derived content visible to clients (after human send). EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: invoice + CRM + Engagement context. CUO/CFO runs through the AI Gateway with persona-stamping. No third-party data; no compensation values in scope.

### Human Oversight

- All dunning emails are human-reviewed + sent.
- Lifecycle transitions (send / void / mark-paid) are human-only; AI cannot mutate.
- Late-payer flag is informational; the founder + Account Manager decide what action to take.

### Failure Modes

- **Tone mismatch in dunning draft.** Mitigation: the CRM contact's `language_default` + the engagement's relationship history feed the persona; sampled review by Account Manager flags drift.
- **Draft references wrong invoice.** Mitigation: citation-correctness regression suite gates persona-version PR.
- **SBV rate poll failure.** Mitigation: prior-day rate as fallback + sev-1 alert.
- **Late-payer flag false-positive** (client paid; flag stale). Mitigation: payment-entry creation auto-clears the flag.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted lifecycle state machine, AR aging compute, CUO/CFO dunning ladder, multi-currency handling, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the CUO/CFO persona's first SKILL.md + the dunning templates will be reviewed with the company's external accountant before P2 production.
