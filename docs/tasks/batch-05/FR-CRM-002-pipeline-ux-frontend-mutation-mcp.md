---
title: "CRM — pipeline kanban + account 360 UX, frontend remote at /crm, mutation MCP surface, per-Member sales views"
author: "@stephen-cheng"
department: design
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the CRM Module-Federation remote at `/crm`: **pipeline kanban** with deals as cards, stages as columns, drag-and-drop transitions; **account 360 view** showing the account + linked contacts + linked deals + activity timeline + signals + linked Engagement + linked PROJ projects; **contact card** with role / language / honorific / activity history; **deal detail drawer** with stage history + close-won/close-lost actions; **per-Member "my deals" view** for Account Managers; **per-account ACL UX**; **search + filters** with Vietnamese-aware tokenisation; the **full mutation MCP surface** (account/contact/deal/activity CRUD + stage transitions + close-won-creates-engagement) with destructive-confirmation gates; **proposal-tracking workflow** (proposal_sent activity → proposal_accepted-or-declined Notify after configurable delay); and the **bidirectional EMAIL ↔ CRM seam consumption** (FR-EMAIL-006's contact-resolver lands here as the rendered side).

## Problem

Without a pipeline UX, the schema in FR-CRM-001 is invisible. The team's Account Manager needs:

- A glance-able view of "where every deal stands" — kanban is the floor.
- A glance-able view of "everything we know about Acme" — the 360 surface.
- A one-click way to log a touchpoint (call notes, meeting summary).
- A clear bidirectional link between EMAIL threads and CRM activities.
- A way to navigate from a CRM contact to the active PROJ engagement.

PRD §14.2.3 P1 → P2 exit gate: "CRM has at least 5 active client records and 10 deal records, with Genie-drafted next-actions accepted by sales rep at ≥40%". This FR ships the surfaces that meet the criteria.

## Proposed Solution

The shape of the answer is the Module-Federation remote, the four primary views (pipeline / account 360 / contact / deal), the proposal-tracking workflow, and the mutation MCP catalogue.

**Layout.**

```
┌──────────────┬──────────────────────────────────────┬──────────────────────┐
│ Sidebar      │ Main view                            │ Right rail           │
│              │                                      │                      │
│ ▾ Views      │ Pipeline / Account 360 / Deal /      │ Genie panel          │
│   Pipeline   │ Contact / Search                     │                      │
│   My deals   │                                      │ For Account 360:     │
│   Accounts   │                                      │ Suggested next-      │
│   Contacts   │                                      │ action (FR-CRM-003)  │
│              │                                      │                      │
│ ▾ Saved      │                                      │ For Deal:            │
│   Hot deals  │                                      │ Stage history;       │
│   Stale 30d  │                                      │ time-in-stage chart  │
│              │                                      │                      │
│ + New deal   │                                      │ For Contact:         │
└──────────────┴──────────────────────────────────────┴──────────────────────┘
```

**Pipeline view (`/crm/pipeline`).**

- Columns: stages (Lead / Discovery / Proposal / Negotiation / Closed-Won / Closed-Lost; the last two collapsed into a "Closed" tab).
- Cards: Deal name + Account + amount + expected-close + probability-band chip + owner avatar + last-activity-age.
- Drag-and-drop between columns triggers `crmTransitionDealStage`; if the transition is `closed_won` or `closed_lost`, a modal opens for the close-won-Engagement-linkage or close-lost-reason capture before the transition commits.
- Filters: owner, account, region, amount-band, stage-age. Filter state in URL.
- Group-by toggle: by stage (default), by owner, by account.
- Aggregate metrics at column tops: count + total amount + median time-in-stage.

**Account 360 view (`/crm/accounts/<slug>`).**

- Header: account name + status chip + region + primary owner + health score (FR-CRM-003 derives; informational here in FR-CRM-002).
- Tabs: Overview / Contacts / Deals / Activities / Signals / Engagement / Documents (links to KB pages tagged for the account).
- **Overview** — primary contact card; latest activity; open deals snapshot; linked Engagement (FR-PROJ-007); top BRAIN facts about the account.
- **Contacts** — contact cards in a grid; primary contact pinned; click opens the contact card.
- **Deals** — deal table with stage / amount / expected-close; click opens the deal drawer.
- **Activities** — chronological timeline; filter by kind; quick-add "log call" / "log meeting" / "log internal note" floating action.
- **Signals** — same shape as activities; surfaced separately because they are async + lower-confidence.
- **Engagement** — when linked, embedded `engagementDashboard` (FR-PROJ-007) shows project + cycle progress.
- **Documents** — KB pages tagged with this account; quick-add "new doc for this account".

**Deal drawer.**

Side-drawer overlay (chained-drawer-friendly with FR-PROJ-005's pattern):
- Header: deal name + stage chip + amount + close date + owner.
- Tabs: Activities / Stage history / Close-out.
- **Stage history** — chronological transitions with reasons; time-in-stage chart per stage.
- **Close-out** — actions: Move to Closed-Won (opens Engagement-creation modal) / Move to Closed-Lost (requires reason + competitor); both step-up-auth-gated.
- Linked email threads (from FR-EMAIL-006 activity logs); click opens EMAIL drawer.

**Contact card.**

- Avatar + name + preferred-name + honorific chip + role chip.
- Email + phone + LinkedIn (with safe-link warn for outbound; FR-EMAIL-010 reused).
- Language + timezone preferences.
- Compose-email-to-this-contact button (opens EMAIL composer with FR-EMAIL-005 honorific pre-applied).
- Activity history filtered to this contact.

**Search.**

PGroonga + vector hybrid query across accounts / contacts / deals / activities. Vietnamese-aware tokenisation (FR-EMAIL-005 + FR-BRAIN-002 reuses). Saved searches per Member. Quick filters: open deals, hot accounts (recent positive signals), stale deals (no activity 30d+).

**Per-account ACL UX.**

A "Share" button on the account 360 view opens the ACL modal:
- Member list with role: viewer / editor / owner.
- Step-up auth required when adding a Member to a previously-private account.
- Audit row written; the Member's daily flow surfaces "you've been added to Acme account" Notify.

**Proposal-tracking workflow.**

When an `activity.kind: 'proposal_sent'` is logged, a server-side timer starts: configurable per-account default (typically 14 days). On expiry without `proposal_accepted` / `proposal_declined` activity, CUO/CRO emits a Notify card to the Account Manager: "Acme proposal sent 14 days ago — chase?". The Notify is one-click "log a follow-up email draft" (using the EMAIL FR-004 + FR-006 deal-aware suggestion).

**Bidirectional EMAIL ↔ CRM seam consumption.**

- FR-EMAIL-006 emits `crm.create_activity(kind: "email_out"|"email_in", ...)` and `crm.create_signal(kind: "email_in", ...)`. The CRM frontend renders these in the timeline.
- A flip from CRM activity to the source email thread is one click (the activity card has a "View email thread" link).
- "Add new contact" in the EMAIL thread surfaces here when the resolver fails (FR-EMAIL-006's "should this be a CRM contact?" Notify lands as a Add-contact suggestion).

**Per-Member "my deals" view.**

`/crm/my` for Account Managers:
- All deals where the Member is `primary_owner`.
- Sorted by next-action-required (CUO/CRO infers; informational here in FR-CRM-002).
- Quick metrics: total open pipeline value, expected-close-this-month, deals stuck > 14 days.

**Mutation MCP surface.**

The full mutation tool catalogue:

- `cyberos.crm.create_account`, `update_account`, `archive_account` — `destructive: true; requires_confirmation: true`. archive_account is `sensitivity: high; step_up_required: true`.
- `cyberos.crm.create_contact`, `update_contact`, `merge_contacts` — `destructive: true; requires_confirmation: true`. merge_contacts is `sensitivity: high; step_up_required: true`.
- `cyberos.crm.create_deal`, `update_deal`, `transition_deal_stage` — `destructive: true; requires_confirmation: true`.
- `cyberos.crm.close_deal_won(id, actual_close_date, engagement_input_for_creation?, existing_engagement_id?)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.crm.close_deal_lost(id, actual_close_date, loss_reason, competitor?)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.crm.create_activity`, `update_activity`, `delete_activity` — first two `destructive: false; idempotent: true; sensitivity: low` (activities are non-destructive logs); delete is `destructive: true; requires_confirmation: true`.
- `cyberos.crm.add_account_member`, `remove_account_member` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.crm.set_account_visibility(id, visibility)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.crm.nlcrud_propose_deal(utterance, account_id?)` / `nlcrud_commit_deal(token)` — propose-then-commit pair (CUO can DRAFT, human commits).
- `cyberos.crm.nlcrud_propose_contact(utterance, account_id?)` / `nlcrud_commit_contact(token)` — same.

CUO scope contract: read all + propose; commit forbidden. Consistent with PROJ FR-PROJ-008.

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped.
- Pipeline view p95 ≤ 800 ms over 200 deals.
- Account 360 view p95 ≤ 1.2 s over 500 activities.

**Mobile responsive.**

- Pipeline kanban becomes a horizontal carousel below 1024 px.
- Account 360 tabs become a swipe-able ribbon below 768 px.
- Deal drawer is full-screen below 640 px.

## Alternatives Considered

- **Use HubSpot's UI via embedded iframe.** Rejected: residency + integration + auth-cookie + design-token consistency.
- **Skip the kanban; only ship a list view.** Rejected: deal-stage visibility is the floor; kanban is the canonical sales tool.
- **Single mega-table for all CRM types.** Rejected: tab-by-tab navigation in account 360 keeps each surface focused.
- **Auto-archive deals stale > 90 days.** Rejected: human-in-the-loop floor; CUO/CRO surfaces a Notify suggesting archive; the Account Manager decides.
- **No proposal-tracking workflow.** Rejected: proposal-stage decay is the most-cited deal-stuck reason; tracking is the floor.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: 5+ accounts active; 10+ deals tracked; pipeline kanban used daily by Account Manager; proposal-tracking workflow fires on 100% of `proposal_sent` activities.
- **Activity logging metric.** ≥ 90% of email threads to CRM contacts auto-log as activities (the EMAIL-006 seam wired correctly here).
- **Latency NFR.** Per the budgets above; bundle ≤ 50 KB.

## Scope

**In-scope.**
- Module-Federation remote at `/crm` with the four views.
- Pipeline kanban with drag-and-drop stage transitions.
- Account 360 with all seven tabs.
- Deal drawer with close-won + close-lost flows.
- Contact card.
- Saved-searches per Member.
- Per-account ACL UX with step-up auth.
- Proposal-tracking workflow + Notify cards.
- The full mutation MCP surface.
- Bidirectional EMAIL ↔ CRM seam consumption.
- Mobile-responsive layouts.
- Audit integration.

**Out-of-scope (deferred to FR-CRM-003 / FR-CRM-004).**
- AI features: next-action drafter, suggested replies, "typical hold-up" insight (FR-CRM-003).
- HubSpot migration (FR-CRM-004).
- Web tracking + visitor signals (P3).
- Quote / proposal generation (P2; touches contracts).
- Multi-currency display preferences per Member (P2).

## Dependencies

- FR-CRM-001 (schema + Apollo subgraph).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-DESIGN-001.
- FR-EMAIL-006 (the seam this FR consumes).
- FR-PROJ-001 / FR-PROJ-007 (Engagement linkage; embedded engagementDashboard).
- FR-KB-003 (Documents tab embeds KB pages).
- FR-GENIE-001 / FR-GENIE-002 (proposal-tracking Notifies; persona scope).
- FR-OBS-001 / FR-OBS-002.
- Compliance: PDPL Decree 13 (CRM data is heavily personal-data-loaded; the audit + RLS + step-up controls apply); EU AI Act Article 14 (deal closing is human-in-the-loop).
- Locked decisions referenced: DEC-148 (kanban as the canonical pipeline surface), DEC-149 (close-won requires step-up + Engagement linkage), DEC-150 (proposal-tracking auto-Notify floor).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The frontend + mutation surface are deterministic; AI surfaces in FR-CRM-003 inherit GENIE risk classification.
