# PRD — Saved Searches & Saved Filters

> Sample PRD authored 2026-05-06 as a worked example for the `cuo/cpo/fr-author` skill. This is not a real product requirement; it's a realistically-shaped input that demonstrates what fr-author consumes and how its PLAN phase decomposes a single document into multiple Feature Requests.

## Background

CyberOS's customer dashboard currently has a powerful filter panel — users can filter their workspace by date range, tag, owner, status, and several derived attributes. The filter is forgotten on every page load. Power users (CSMs at our larger clients, our internal QA team, two pilot customers we've interviewed) repeatedly construct the same 3-5 filter sets every morning to triage their work.

We've heard this pain three times in the last six weeks: once from a CSM at one of our pilot accounts, once in a Common Room signal cluster, and once via the in-product feedback channel. Common Room and the pilot CSM both used the phrase "save my filter" verbatim.

## Goals

1. **Eliminate the daily filter-rebuild ritual** for power users. Target: a power user (defined as ≥10 unique filter applications per week) reaches their first triage view in ≤5 seconds.
2. **Make filter sets shareable.** Saved filters can be sent to a teammate (within the same workspace) so a CSM can hand off a triage view to a colleague.
3. **Stay calm under load.** Saved filters MUST not degrade dashboard load latency p95 above 800ms (current baseline 620ms).

## Non-goals

- Saved searches across workspaces. Out of scope for v1; saved filters are workspace-scoped.
- Filter-set scheduling (e.g., "email me this filter every Monday"). Plausible follow-up but not v1.
- Cross-account sharing (i.e., sharing a filter with someone outside your workspace). Triggers GDPR / multi-tenancy concerns we haven't worked through yet.

## User stories

### Story 1 — As a CSM, I want to save my morning triage filter

> "Every morning I filter the dashboard to: status=open, priority∈{p0,p1}, owner=me, last-update<7d. Then I work my way down. I want to save that exact filter set as 'Morning triage' and apply it with one click each day."

Acceptance:
- A "Save filter" affordance is visible when ≥1 filter is active.
- The save dialog asks for a name (max 60 chars) and saves the current filter snapshot.
- Saved filters appear in a panel/menu accessible from the dashboard header.
- Clicking a saved filter applies it instantly (no page reload).

### Story 2 — As a CSM, I want to share a filter with my teammate

> "When I go on PTO I want to send Jane my morning triage filter so she can take over my queue without rebuilding it from scratch."

Acceptance:
- Each saved filter has a "Share" affordance.
- Sharing produces a URL (workspace-scoped) that any workspace member can open.
- Opening the URL applies the filter and (optionally) lets the recipient save it under their own name.
- The original owner can revoke share access at any time.

### Story 3 — As a workspace admin, I want to remove someone's saved filters when they leave

> "When someone offboards we need to clean up their saved filters so we don't have stale shares pointing to filters owned by deleted accounts."

Acceptance:
- Workspace admin sees a list of all saved filters across the workspace.
- Admin can transfer ownership of a filter to another workspace member, or delete it.
- Deletion cascades to any active shares of that filter.

## Quality bars

- **Performance.** Dashboard p95 load with saved-filters-feature enabled MUST stay ≤ 800ms. Measure across 7 days of production traffic; pre-existing baseline 620ms; budget 180ms additional.
- **Availability.** Saved-filter operations (save, apply, share, revoke) MUST be available whenever the dashboard is available. 99.9% target.
- **Privacy.** Saved filters MUST NOT contain PII other than user IDs. The filter snapshot is structured data (column names + values), not free-text user notes.
- **Auditability.** Every save / share / revoke action emits an `genie.action_log` row with `row_kind: ui_action`.

## Open questions

1. Should the "Share" URL be an opaque token or a structured URL with the filter encoded? Opaque token has better privacy + revocation; structured URL is more debuggable. **Open until tech-spec phase.**
2. Cap on saved filters per user? Suggest 50; storage isn't the constraint, UX clutter is. **Open until UX review.**
3. Should sharing default to "view-only" (recipient applies but can't modify) or "fork-on-apply" (recipient gets a private copy)? **Strong opinion needed; defer to PM lead.**

## EU AI Act considerations

This feature involves no AI/ML inference. The filter-snapshot is a structured query against existing data. Not in scope of EU AI Act.

## Compliance + privacy

- Workspace-scoped, no cross-tenant data flow.
- No new PII collected. Filter values that contain user IDs ride existing data-residency policy.
- Sharing audit trail enables right-to-erasure (admin can revoke + delete; cascades to shares).

## Rough sizing

This document feels like 2-3 distinct Feature Requests at the FR-create granularity:

1. Core saved filters (Story 1) — M.
2. Sharing (Story 2) — M.
3. Admin lifecycle management (Story 3) — S.

Total: M + M + S ≈ 1.5 engineer-months. Rough; tech spec will refine.

## What success looks like (12 weeks post-launch)

- ≥30% of power users (≥10 filter-applications/week pre-launch) have created at least one saved filter.
- Median time to "first triage view" for power users drops from ~22 seconds (current) to ≤6 seconds.
- Zero p1 incidents attributable to the feature in the first 30 days.
- Dashboard p95 load latency stays ≤ 800ms (the budget).

## Appendix — research signals that triggered this PRD

- **Common Room signal cluster** (2026-04-12 to 2026-05-02): 8 distinct posts mentioning "save my filter" or "filter presets" in the customer-dashboard channel.
- **Pilot CSM interview** (2026-04-19): 25-minute call with [pilot account]'s lead CSM. Verbatim quote: "the filter rebuild every morning is dumb, just let me save it." Recorded in `memories/projects/2026-04-19-pilot-csm-feedback.md`.
- **In-product feedback** (2026-04-28): one user submitted a structured feedback form requesting "saved filters." Logged in PROJ ticket `CYB-1247`.
