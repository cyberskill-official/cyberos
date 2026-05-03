---
title: "OKR — frontend remote at /okr (cascade tree, alignment heatmap, check-in surface, founder review pages)"
author: "@stephen-cheng"
department: design
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q2"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the OKR Module-Federation remote at `/okr` consuming FR-OKR-001 + FR-OKR-002. Four primary surfaces: **cascade tree** (`/okr/tree`) showing the company → team → individual hierarchy with status chips per Objective + KR; **alignment heatmap** (`/okr/heatmap`) the founder + team leads' main view of cross-team alignment with company OKRs; **per-Member check-in surface** (`/okr/my`) for weekly individual updates; **founder review pages** (`/okr/review/<cycle-id>`) showing kickoff draft, mid-cycle review, and cycle-close draft with founder-edit + publish actions. Plus admin views for cycle creation + closing. The frontend is the daily-driver surface that ties together OKR alignment + cross-module signals (PROJ, CRM, OBS, KB) into one strategic view.

## Problem

OKRs without a usable UI fail adoption — the cycle becomes a checkbox exercise instead of a strategic instrument. Three failure modes:

- **Cascade invisibility.** Without a tree visualisation, "what does my team's objective trace to?" is unanswerable.
- **Heatmap blindness.** Without the heatmap, "which company OKRs are under-resourced?" is invisible until cycle-close retrospection.
- **Check-in friction.** A check-in form with 12 fields per KR drives Members to skip; the UI must be 60 seconds per check-in to drive weekly compliance.

## Proposed Solution

The shape of the answer is a Vite + React 19 Module-Federation remote at `/okr` consuming the GraphQL surfaces from FR-OKR-001/002.

**Cascade tree (`/okr/tree`).**

Default home for any Member.

- **Layout.** A vertical tree:
  - Top: Company objectives (3-5).
  - Middle: Team objectives (per team, indented under their company parent).
  - Bottom: Individual objectives (indented under their team parent).
- **Per node.**
  - Title + status chip (on_track / at_risk / off_track / achieved / missed).
  - Owner avatar + role.
  - KR summary chip (e.g. "3 of 4 KRs on track").
  - Click → opens the Objective detail in a side-drawer.
- **Filters.**
  - Cycle (default: current).
  - Scope (company / team / individual).
  - Owner.
  - Team.
  - Status.
- **Search.** Fuzzy search across objective + KR titles.
- **My OKRs anchor.** A persistent "My OKRs" filter chip; the tree highlights the calling Member's path through the cascade.
- **Empty / pre-cycle states.** When no cycle is active or before cascade is complete, the tree shows a "cycle not yet open" banner + the founder's kickoff draft (when ready).

**Alignment heatmap (`/okr/heatmap`).**

For founder + team leads.

- **Layout.** Matrix:
  - Rows: company objectives.
  - Columns: teams.
  - Cell: a colour-coded chip (green / yellow / red) showing coverage status.
  - Hover cell → tooltip with: count of team objectives + average KR confidence + at-risk count.
  - Click cell → opens a list of cascading objectives in a side-drawer.
- **Second matrix (toggle).** Per-Member coverage of their team's objectives.
- **Coverage gaps.** A "Gaps" section highlights company OKRs with red cells (no cascade) — actionable items for the founder + team lead.
- **Trend toggle.** When multiple cycles are present, a small inline trend per cell shows green-yellow-red stability over time.

**Per-Member check-in surface (`/okr/my`).**

The default Member view (alongside `/okr/tree`).

- **My objectives** (current cycle).
  - Per-objective card with KRs.
  - Per-KR: current value + target + progress bar + status chip.
  - "Quick check-in" inline action: numeric input + confidence picker + commentary textarea + submit. Target: ≤ 60 seconds per KR.
- **Pending check-ins.** A red-dot indicator per KR when last check-in was > 7 days ago.
- **My objectives' parent context.** Shows the team objective + company objective each individual KR cascades to (helps the Member see strategic context).
- **Linked artefacts.** Per KR: linked PROJ issues + CRM deals + KB pages. "+ Link artefact" CTA.

**Founder review pages (`/okr/review/<cycle-id>`).**

For founder + team leads.

- **Kickoff page.**
  - The CUO/CEO-drafted kickoff narrative (FR-OKR-002).
  - Edit-and-publish workflow (Markdown editor; the founder commits the canonical text).
  - Preview of company objectives in a tree.
- **Mid-cycle review.**
  - Highlights / At-risk / Misalignments / Learnings / Recommended adjustments — structured sections.
  - Each section's claims have citation chips (clickable to the underlying check-in / signal).
  - Founder edits + publishes; the published review goes to BRAIN as a strategic-context fact.
- **Cycle-close draft.**
  - Same shape as mid-cycle but more comprehensive.
  - Edit + sign + publish triggers `okrCloseCycle` mutation (status transition).
  - Post-publish: the review is visible to all; tagged with `persona_version` + `ai_disclosure_id`.

**Admin views (`/okr/admin`).**

For founder + Engineering Lead.

- **Cycle creation.**
  - "Create new cycle" form with quarter + dates + kickoff narrative seed.
- **Active cycle management.**
  - Open / closing windows per phase.
  - Manual override: extend cascade phase / extend cycle / close prematurely.
- **Cycle archive.**
  - Past cycles + their published reviews + their close summaries.

**Cross-module integration.**

- Linked PROJ issues open in the PROJ side-drawer (FR-PROJ-005's pattern).
- Linked CRM deals open in the CRM side-drawer (FR-CRM-002's pattern).
- Linked KB pages open in the KB side-drawer (FR-KB-003's pattern).
- The "Engagement dashboard" (FR-PROJ-007) embeds OKR signals when an Engagement is linked to a KR.

**Vietnamese-locale rendering.**

- vi-VN default for the canonical CyberSkill tenant.
- Status chips + dates per Vietnamese convention.
- Markdown rendering in vi-VN typography (Be Vietnam Pro per FR-DESIGN-001).

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped.
- Cascade tree p95 ≤ 600 ms over 100 objectives.
- Heatmap p95 ≤ 400 ms.
- Check-in submit ≤ 800 ms p95.
- Mobile-responsive: tree becomes a vertical scroll; heatmap becomes per-row card stack on small screens.

**Empty + pre-cycle states.**

- New tenant with no cycle: prompts founder to create the first cycle.
- Cycle in pre-cycle phase: shows kickoff draft + cascade-pending banner.
- Cycle in cascade phase: shows partial tree + cascade-due reminders.
- Cycle in close phase: shows the close draft + sign action.

**MCP tool surface (read-only).**

- `cyberos.okr.my_dashboard_payload` — read; calling Member's check-in surface data.
- `cyberos.okr.tree_payload(cycle_id?)` — read; the cascade-tree data.
- `cyberos.okr.heatmap_payload(cycle_id?)` — read; the heatmap matrix data.
- `cyberos.okr.review_payload(cycle_id, kind: "kickoff"|"mid_cycle"|"close")` — read.

## Alternatives Considered

- **Skip the heatmap; rely on a flat list.** Rejected: the heatmap is the founder's most-used artefact for cross-team alignment.
- **Use Notion / Coda for OKR rendering.** Rejected: residency + cross-module linkage + CUO persona-stamped reviews.
- **Single mega-page combining tree + heatmap + check-ins.** Rejected: separation of concerns; each view has different access patterns.
- **Mobile-first.** Rejected: OKR cascade visualisation works better on desktop; mobile-responsive is the floor.

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate progress: 100% of Members complete weekly check-ins on ≥ 75% of weeks in the cycle; founder publishes mid-cycle + cycle-close reviews; alignment heatmap rendered + reviewed at the founder's weekly cadence.
- **Adoption metric.** ≥ 80% of Members open `/okr/my` weekly; ≥ 90% of team leads use `/okr/heatmap` monthly.
- **Latency NFR.** Per the budgets above; bundle ≤ 50 KB.

## Scope

**In-scope.**
- The Module-Federation remote at `/okr`.
- Cascade tree view.
- Alignment heatmap view.
- Per-Member check-in surface.
- Founder review pages (kickoff / mid-cycle / cycle-close).
- Admin views (cycle creation + management).
- Cross-module side-drawer integration.
- Vietnamese-locale rendering.
- Mobile-responsive layouts.
- The 4 read-only MCP tools.
- Audit integration in scope `okr.ui.{tenant}`.

**Out-of-scope (deferred).**
- Multi-cycle trend visualisation in heatmap (P3).
- Public OKR surface for Client Portal (P4).
- Mobile native (P3).
- Per-team customisation of cycle cadence (P3).
- Voice-input check-ins (P3 mobile).

## Dependencies

- FR-OKR-001 / FR-OKR-002.
- FR-PROJ-001 / FR-PROJ-005 / FR-PROJ-007 (linked artefacts + side-drawer).
- FR-CRM-001 / FR-CRM-002 (linked artefacts).
- FR-KB-001 / FR-KB-003 (linked KB pages).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002.
- FR-OBS-001 / FR-OBS-002.
- Compliance: PDPL Decree 13 (OKR data may include personal targets); EU AI Act Article 50 (review surfaces inherit FR-OKR-002's disclosure).
- Locked decisions referenced: DEC-224 (4-surface frontend layout: tree + heatmap + check-in + review), DEC-225 (≤ 60 seconds per check-in target), DEC-226 (cross-module side-drawer pattern reused).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The frontend itself is deterministic UI; AI surfaces (kickoff/mid-cycle/cycle-close drafts) inherit FR-OKR-002's classification.
