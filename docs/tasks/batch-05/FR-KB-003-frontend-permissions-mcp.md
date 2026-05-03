---
title: "KB — frontend remote at /kb, permissions UX, full mutation MCP surface, content migration"
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

Ship the KB Module-Federation remote at `/kb`: **left sidebar** with Spaces tree + favourited pages + recently viewed; **main editor surface** rendering the FR-KB-001 block editor; **right rail** with "ask this page" + "related pages" + "page metadata" + comments thread; **search bar** with hybrid retrieval (PGroonga + vector); **per-Space + per-Page permission UI**; the full **mutation MCP surface** (page CRUD, publish, archive, merge, restore) with destructive-confirmation gates; **import paths** from Notion + Confluence + Google Docs + plain Markdown for the team's existing institutional memory; and the **page lifecycle UX** — draft → published → republished → archived — with diff viewer for republish notes. The frontend ties together FR-KB-001 (schema + editor) and FR-KB-002 (AI Q&A) into the daily-driver surface.

## Problem

A schema + editor + AI without a frontend is unusable. Three failure modes the team will hit:

- **Discovery cliff.** Without a sidebar that reveals what exists, the KB is a black hole; Members cannot find pages they did not author.
- **Permission opacity.** Without a clear permission UI, a Member who created a "private to me" page accidentally publishes it tenant-wide; reputational + compliance risk.
- **Migration debt.** The team's existing Notion + Confluence + Google Docs content does not migrate itself; without an import path, KB starts empty and dies as the team falls back to the old tools.

## Proposed Solution

The shape of the answer is a Vite + React 19 Module-Federation remote consuming `@cyberskill/components` (FR-DESIGN-001) and the FR-KB-001 + FR-KB-002 GraphQL surfaces.

**Layout.**

```
┌──────────────┬───────────────────────────────────┬──────────────────────┐
│ Sidebar      │ Main editor                       │ Right rail           │
│              │                                   │                      │
│ ▾ Spaces     │ [Page title]                      │ Ask this page        │
│   Engineering│ [icon] [breadcrumbs]              │ ━━━━━━━━━━           │
│   Client     │                                   │                      │
│   HR Internal│ [Block 1]                         │ Related pages        │
│              │ [Block 2]                         │ • Acme deploy guide  │
│ ▾ Favourites │ [Block 3]                         │ • Edge config docs   │
│   Onboarding │ ...                               │                      │
│              │                                   │ Page metadata        │
│ ▾ Recent     │ [Slash menu when /]               │ Authors: [stack]     │
│   ALPHA spec │                                   │ Published: 2026-04-22│
│   Q3 review  │ [Floating publish bar]            │ Version: 4           │
│              │                                   │                      │
│ + New page   │                                   │ Comments (3)         │
└──────────────┴───────────────────────────────────┴──────────────────────┘
```

**Sidebar.**

- **Spaces tree.** Top-level Spaces; each expands to show a tree of Pages (parent_page_id-derived). Drag-and-drop to reorder or re-parent; respects per-Space write permissions.
- **Favourites.** A Member can favourite any Page they can read; favourites surface here per Member.
- **Recent.** Last 10 visited pages per Member; client-side; survives reload.
- **Search.** A search input at the top with `Cmd-K`-style fuzzy results across spaces / pages / blocks (PGroonga full-text + vector retrieval); selecting an item navigates.
- **Plus button.** `+ New page` creates a draft page in the current space.

**Main editor.**

- The FR-KB-001 block editor renders here.
- Floating top bar with: status chip (draft / published), version chip, Publish button (or "Republish" with diff viewer), share button, more menu (archive, duplicate, export as Markdown, version history).
- The publish flow:
  - "Publish" opens a modal with a Markdown diff vs. the previous version.
  - The author writes a one-line `publish_note_md` (required when the diff is non-trivial; suggested by CUO/CTO if the author leaves it empty: a one-sentence summary of the change).
  - "Confirm publish" triggers the mutation; the chip updates to "Published v5"; BRAIN ingestion fires (FR-KB-001).

**Right rail.**

- **Ask this page.** Input + last few Q&A history items (per Member, per page).
- **Related pages.** Up to 5 cards; click to navigate.
- **Page metadata.** Authors with avatar stack; primary author; published date; version; ACL chip ("Tenant-internal" / "Private to space" / "Client-visible summary").
- **Comments.** Threaded; the FR-KB-001 comment thread surface; resolved threads collapse.

**Permissions UX.**

A "Share" button in the top bar opens a modal:

- **ACL inheritance chain** at the top: shows where the current page's permission comes from (space default / page override).
- **Member list.** Add Members with role: Viewer / Commenter / Editor.
- **Visibility toggle.** Set per-Page visibility; defaults to inherit-from-Space; explicit override creates a `kb.page.visibility` non-null value.
- **Audit-ready note.** When changing from `tenant_internal` to `client_visible_full`, a confirmation step explicitly warns and requires step-up auth (FR-AUTH-003) — moves from internal to client-visible are sensitive operations.

**Frontend shows the permission state always.** The page header shows a chip ("Internal-only" / "Client-visible summary" / "Private to space members"); ambiguity is structurally avoided.

**Mutation MCP surface (extends FR-KB-001 read tools).**

- `cyberos.kb.create_space` — `destructive: true; requires_confirmation: true`.
- `cyberos.kb.update_space` — `destructive: true; requires_confirmation: true`.
- `cyberos.kb.create_page` — `destructive: true; requires_confirmation: true`.
- `cyberos.kb.update_page` — `destructive: true; requires_confirmation: true`.
- `cyberos.kb.publish_page` — `destructive: true; requires_confirmation: true; sensitivity: medium` (publishing changes BRAIN-derived facts visible to teammates).
- `cyberos.kb.republish_page` — `destructive: true; requires_confirmation: true`.
- `cyberos.kb.archive_page` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.kb.merge_pages(source_ids, target_id, redirect)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.kb.restore_page_version` — `destructive: true; requires_confirmation: true`.
- `cyberos.kb.set_page_visibility(page_id, visibility)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true` when crossing into `client_visible_*`.
- `cyberos.kb.add_page_member` — `destructive: true; requires_confirmation: true`.
- `cyberos.kb.create_block` / `update_block` / `delete_block` / `reorder_blocks` — `destructive: true; requires_confirmation: true`. The propose-then-commit pattern is provided via:
- `cyberos.kb.nlcrud_propose_page(utterance, space_id?)` — `destructive: false; idempotent: true`.
- `cyberos.kb.nlcrud_commit_page(confirmation_token)` — `destructive: true; requires_confirmation: true`.

CUO scope contract: drafts allowed; commits forbidden (consistent with PROJ FR-PROJ-008).

**Import paths.**

A `cyberos-kb-import` CLI + service supporting:

- **Notion.** Notion API export → block-by-block migration; preserves nested structure, mentions (Member matching by email), embeds, attachments. Cleanly handles Notion's quirky "synced blocks" by promoting them to pages.
- **Confluence.** Confluence Cloud API → block conversion; macros mapped to closest CyberOS block kind or fallback callout.
- **Google Docs.** OAuth-authorised pull; Docs are converted via the Docs API → Markdown → block tree. Comments are preserved as KB threads.
- **Markdown files.** A folder of `.md` files (e.g. an existing repo's `docs/`); titles from H1 + filename; folder structure → space + page tree.

The import UI shows progress per source per workspace; preserves source URLs as `metadata.imported_from.{source}.{external_url}` so legacy links redirect via `link.cyberos.world/legacy/...` (the same redirect surface as FR-PROJ-009 + FR-EMAIL-010).

A 7-day rollback window matches PROJ migration semantics.

**Markdown export.**

The "Export as Markdown" action serialises the page (and optionally subtree) to a downloadable `.md` file or a `.zip` of files preserving structure. Useful for the Trust Center "your data is portable" property and for the regulator-grade `.zip` export from FR-BRAIN-001.

**Mobile responsive.**

- Sidebar collapses to a drawer below 1024 px.
- Right rail collapses to a bottom-tab below 768 px.
- The block editor's floating slash menu adjusts for touch.
- Read-only on mobile is pleasant; editing is intentional but functional. Native mobile is P3.

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped (PRD §7.2 "Module Ready").
- Page load p95 ≤ 1.5 s on 4G.
- Editor first-keystroke-to-render ≤ 30 ms.

## Alternatives Considered

- **Skip the dedicated remote; embed KB inside the host shell's home page.** Rejected: KB is a daily-driver surface that warrants its own URL space + sidebar.
- **One mutation tool per intent (e.g. "edit_page") rather than per granular operation.** Rejected: tool-name precision is the floor; granular tools allow agents to perform partial edits with appropriate confirmation gates.
- **Skip Notion / Confluence / Google Docs imports; only Markdown.** Rejected: the team's existing institutional memory lives in Notion + Google Docs; without those imports, KB starts empty.
- **No step-up on the visibility-change to client-visible.** Rejected: client-visibility flip is the highest-leverage information-leak vector.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: each Member has at least 5 published KB pages; the team consults KB ≥ 50 times per week (Q&A + page reads); ≥ 30% of Notion / Confluence / Google Docs pre-existing content imported.
- **Latency NFR.** Page load p95 ≤ 1.5 s; editor first-keystroke ≤ 30 ms; bundle ≤ 50 KB.
- **Permission safety.** 0 unintended visibility flips during the 14-day pre-exit window (the step-up auth on cross-boundary changes is the structural floor).

## Scope

**In-scope.**
- The Module-Federation remote at `/kb` with sidebar + main + right rail.
- The publish + republish flow with diff viewer + publish-note suggestion.
- The Share / permissions modal with step-up on cross-boundary changes.
- The full mutation MCP surface.
- `cyberos-kb-import` for Notion, Confluence, Google Docs, Markdown.
- Markdown export + zip export.
- Mobile-responsive layouts.
- Bundle-size CI check.
- Audit integration.

**Out-of-scope (deferred).**
- Native mobile (P3).
- Real-time co-editing presence indicators beyond the existing CRDT (P2 — show who is currently viewing).
- Slack / Microsoft Teams integration for "share KB page to channel" (P3 — CHAT integration is the floor).
- Customisable Space themes (P2 — design tokens enforce uniformity in P1).

## Dependencies

- FR-KB-001 / FR-KB-002.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001 (`@cyberskill/components`).
- FR-GENIE-001 / FR-GENIE-002 (Notify cards for promote-to-canonical, stale, draft assistance).
- FR-OBS-001 / FR-OBS-002.
- Cloudflare Workers (legacy-link redirect reused).
- Notion API + Confluence Cloud API + Google Docs API credentials at import time.
- Compliance: PDPL Decree 13 + GDPR Article 17 (P3) — KB pages contain personal data; the export flow + per-tenant residency are the controls.
- Locked decisions referenced: DEC-134 (step-up on cross-boundary visibility flip), DEC-135 (P1 imports cover Notion + Confluence + Google Docs + Markdown).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The frontend itself is deterministic UI; AI features inherit FR-KB-002's classification.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
