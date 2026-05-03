---
title: "PROJ — Module-Federation frontend (board, list, timeline, search, keyboard shortcuts, command palette)"
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
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the PROJ Module-Federation remote at `/projects` consuming the GraphQL subgraph (FR-PROJ-001), the sync engine (FR-PROJ-002), the lifecycle rules (FR-PROJ-003), and the cycle dashboard payload (FR-PROJ-004). Three primary views: **board** (kanban; default for cycle work); **list** (dense issue table; for filtering, bulk operations, search); **timeline** (Gantt-like for project planning across cycles). Plus a **command palette** (`Cmd-K`) for any action; **keyboard shortcuts** matching Linear/GitHub conventions (`c` create, `j/k` navigate, `e` assign-to-me, `s` set state, `m` move-to-cycle, `/` search); **deep-linking** to any view + filter set; **per-Member view persistence** in IndexedDB; **collapsible Genie panel** along the right edge so the user keeps CUO context visible without leaving PROJ. Initial JS bundle ≤ 50 KB gzipped per the Module Ready criterion (PRD §7.2).

## Problem

A schema + sync engine + lifecycle is invisible without a tractable UI. Without this FR the team continues using the prior tracker because logging into a less-good interface to do the same work is irrational.

Three UX failures the prior tracker has and this FR must avoid:

- **Slow board drag.** Dragging an issue across columns takes ≥ 2 s round-trip. Optimistic UI from FR-PROJ-002 fixes the back-end side; the frontend must apply the visual update on the same event-loop tick.
- **No keyboard navigation.** Members who triage 30 issues a day cannot afford to mouse to every action; Linear's keyboard shortcut set is the floor.
- **No deep-linking.** Sharing "the at-risk issues for Acme cycle 14" via Slack today is a 5-minute screenshot exercise; deep links solve this.

The PRD §9.5.3 names the daily-driver behaviours ("CUO/COO-skill task triage", "auto-blocker detection") that depend on the views being usable.

## Proposed Solution

The shape of the answer is a thin Vite + React 19 Module-Federation remote authored in TypeScript, consuming `@cyberskill/components` (FR-DESIGN-001) and `@cyberskill/proj-sync` (FR-PROJ-002), rendered inside the host shell.

**Three views.**

1. **Board view** (`/projects/<key>/board?cycle=<id>`).
   - Columns are workflow states (FR-PROJ-003); ordered by `proj.workflow.position`; user-visible state labels.
   - Cards show: title; assignee avatar; estimate chip; priority chip; due-date chip if soon; labels; small icon stack for blocking-by / blocked-by / has-PR.
   - Drag-and-drop between columns; the optimistic mutation applies immediately; if the transition is disallowed by FR-PROJ-003's rules, the card snaps back with a toast.
   - Empty columns show a faint "drop here" indicator; full columns (over WIP cap) show a red ribbon.
   - Filters: assignee, label, priority, has-PR, search query. Filter state in the URL.
   - Group-by toggle: by state (default), by assignee, by label, by priority.

2. **List view** (`/projects/<key>/list?cycle=<id>&...`).
   - Dense table; 40-50 rows visible at standard zoom.
   - Sortable columns: number, title, state, assignee, priority, estimate, due, updated.
   - Multi-select with shift+click; bulk actions (move to cycle / assign / transition / cancel) via the command palette or top-bar.
   - Inline edit: click title to edit; click state chip to change (respecting workflow rules); click assignee chip to reassign.
   - Saved filters per Member (`proj.member_saved_filter`).

3. **Timeline view** (`/projects/<key>/timeline?range=<...>`).
   - Horizontal Gantt-like; one row per Member or per issue (toggle).
   - Cycles render as vertical bands; cycle goal in the band header.
   - Issues with `due_date` render as bars; without a due date show as anchor-points.
   - Click-and-drag to reschedule an issue's due date or move between cycles.
   - Zoom: day / week / month.

**Command palette (`Cmd-K` / `Ctrl-K`).**

A pop-over with fuzzy search across:
- Actions ("Create issue", "Move issue to cycle", "Close cycle", "Assign all to me").
- Issues by code or title (`ALPHA-1234`).
- Cycles (`Sprint 14`).
- Members (`@stephen`).
- Saved filters.

Selecting an action opens its modal; selecting an entity navigates. The palette is the primary input surface for users who prefer keyboard.

**Keyboard shortcuts.**

Global (work in any view):
- `c` — create issue (modal).
- `g + p` — go to projects list.
- `g + i` — go to my issues.
- `g + b` — go to current project board.
- `Cmd-K` — command palette.
- `/` — focus search.
- `?` — show all shortcuts.

Within a view (with an issue focused via `j`/`k`):
- `j` / `k` — next / prev issue.
- `e` — assign to me.
- `Shift-e` — open assignee picker.
- `s` — change state (popover).
- `m` — move to cycle (popover).
- `p` — change priority.
- `l` — add label.
- `Shift-l` — manage labels.
- `Cmd-Enter` — save changes.
- `Esc` — close panel / cancel.

A Vietnamese-keyboard-aware variant rebinds shortcuts that conflict with `Telex`-input-method commitments (e.g. `s` for sắc tone) — the per-Member preference defaults to Telex-aware on a Vietnamese-detected keyboard.

**Deep-linking.**

Every URL is restorable without mutation: `/projects/ALPHA/board?cycle=42&assignee=@stephen&label=bug`. Deep links share via the platform's existing one-click share button; permissions are validated server-side (the recipient sees the same RLS-filtered view).

**Per-Member view persistence.**

`proj.member_view_state{member_id, project_id, view_kind, last_filters, last_sort, last_group_by, last_seen_at}` — the user's last view for a project is restored on next visit. Stored client-side in IndexedDB; mirrored server-side for cross-device continuity.

**Collapsible Genie panel.**

The right edge has a 380-px Genie panel that collapses to 64 px (FR-DESIGN-001 §"Genie tokens"). Collapsed, the panel shows mode-icon stack (Notify/Question/Review counts). Expanded, the panel shows the user's relevant cards. The panel state persists per Member.

**Issue detail panel.**

Clicking an issue opens a side-drawer (overlays board / list; replaces timeline-row). The drawer renders:
- Title + key + state chip + priority chip.
- Description (Markdown; rich-text editor with the Design System's `<DiffView>` style for the AI-assisted-fields chip when applicable).
- Assignee + reporter + cycle + project + labels.
- Activity feed: state transitions, comments, internal comments, system events (PR linked, blocker resolved).
- Comment composer (Markdown + `@member` mention + slash commands).
- Linked entities: source email thread (FR-EMAIL-007), GitHub PRs, blocking / blocked-by issues, parent / sub-issues, CRM contact (FR-PROJ-007 + batch-05).
- A `Cmd-Click` on any linked entity opens it in a side-side-drawer (chained drawer for context-keeping).

**Mobile responsive (P1 web; native P3).**

The remote degrades gracefully from 1280px down to 360px:
- Board: columns become a horizontal carousel.
- List: virtualised rows; columns collapse with priority-ordered visibility.
- Timeline: not supported below 768px (shown as "use a wider screen" placeholder).
- Drawer: full-screen below 640px.

The mobile experience is competent for triage on the go; the deep workflow (cycle planning, multi-issue bulk ops) targets desktop.

**Accessibility.**

- WCAG 2.1 AAA contrast on every default token pairing (FR-DESIGN-001).
- Full keyboard navigation; focus-visible everywhere.
- Screen-reader labels on every interactive element; live-region announcements for sync events ("issue ALPHA-1234 moved to In Progress by Stephen").
- Reduced-motion respect: drag animations and transitions tone down when `prefers-reduced-motion: reduce`.

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped (FR-PROJ-001 / PRD §7.2 "Module Ready").
- Time-to-interactive on a cold load ≤ 1.5 s p95 over a 4G connection.
- Board view with 200 issues renders in ≤ 600 ms p95.
- IndexedDB read-through cache for repeat visits; first render ≤ 200 ms p95 from cache.

**Audit integration.** No new audit rows from this FR — the views are read surfaces that ride on top of FR-PROJ-001..004 mutation paths.

## Alternatives Considered

- **Server-rendered HTML (no Module Federation remote).** Rejected: the host shell + federation pattern is the architectural floor (PRD §8.3); a server-rendered exception would invalidate the consistency story.
- **Native mobile app in P1.** Rejected: P3 deliverable per the PRD; the responsive web is the floor in P1 with P3-mobile-native for production.
- **Skip the timeline view in P1.** Rejected: cycle planning across multiple cycles is hard without it; the founder uses it weekly.
- **No command palette; rely on the menu bar.** Rejected: command palette is the modern PM-tool floor.
- **shadcn-ui primitives directly without `@cyberskill/components`.** Rejected: the components library wraps Radix UI primitives + adds the Vietnamese-locale + Genie-token discipline; bypassing it leaks design-system drift.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) all three views render; (2) drag a card on the board, optimistic mutation visible immediately, server confirm in ≤ 250 ms; (3) command palette finds and executes "create issue" + "move to cycle"; (4) every Vietnamese-keyboard shortcut works without conflicting with Telex input; (5) deep-link survives a copy-paste-into-CHAT.
- **Adoption metric.** P1 → P2 exit criterion: ≥ 100 issues created via the new UX in the 14-day pre-exit window; ≥ 80% of issues touched per week have at least one mutation that round-trips through the optimistic + server-confirm path.
- **Bundle metric.** Initial JS ≤ 50 KB gzipped (PRD §7.2 enforced via CI bundle-size check).
- **Latency NFR.** TTI ≤ 1.5 s p95 over 4G; board render ≤ 600 ms p95 over 200 issues.

## Scope

**In-scope.**
- The three views with the layouts described.
- Drag-and-drop for the board view.
- Inline edit for the list view.
- Reschedule-by-drag for the timeline.
- Command palette + global + per-view keyboard shortcuts.
- Vietnamese-keyboard-aware shortcut variant.
- Deep-linking with restored filter state.
- Per-Member view persistence (client + server mirrored).
- Collapsible Genie panel.
- Issue detail side-drawer with chained drawer for linked entities.
- Mobile-responsive layouts.
- A11y AAA conformance on default tokens.
- Bundle-size CI check.

**Out-of-scope (deferred).**
- Native mobile app (P3).
- Customisable column widths beyond standard responsive (P2).
- Drag-and-drop on mobile (the touch-drag UX is fragile; mobile uses tap-menus instead).
- Bulk operations on the timeline view (P2; bulk lives in the list view).
- Markdown WYSIWYG editor (the comment + description editor is plain Markdown with preview toggle; rich-text WYSIWYG is P2).

## Dependencies

- FR-PROJ-001 / FR-PROJ-002 / FR-PROJ-003 / FR-PROJ-004.
- FR-INFRA-001 (host shell + Module-Federation runtime).
- FR-AUTH-001 (identity + RBAC).
- FR-DESIGN-001 (`@cyberskill/components`, design tokens, genie-tokens, lint plugin).
- FR-GENIE-001 / FR-GENIE-002 (Genie panel substrate).
- FR-EMAIL-007 (linked-email-thread surface).
- FR-CRM-002 (linked-CRM-contact surface; stubbed today, real in batch-05).
- Compliance: WCAG 2.1 AAA enforced via CI a11y tests; PDPL Decree 13 (the views surface personal data of Members + counterparties; the RLS predicates from FR-PROJ-001 are the floor).
- Locked decisions referenced: DEC-111 (three views — board / list / timeline), DEC-112 (command palette is the universal action surface), DEC-113 (Vietnamese-keyboard shortcuts default-aware).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The frontend is deterministic UI; AI-derived surfaces inside (Notify cards, Review drafts, suggested replies) inherit FR-GENIE-001 / FR-GENIE-002 risk classification.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
