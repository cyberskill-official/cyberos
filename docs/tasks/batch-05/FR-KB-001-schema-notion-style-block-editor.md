---
title: "KB — schema + Notion-style block editor with mentions, embeds, version history, BRAIN ingestion"
author: "@stephen-cheng"
department: engineering
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

Stand up the Knowledge Base (KB) module's foundation: a Postgres schema for Pages + Blocks + Spaces + Versions + Comments, an Apollo Federation v2 subgraph, and a **Notion-style block editor** with the canonical block kinds (paragraph, heading 1/2/3, bullet/numbered/todo list, quote, code, callout, divider, image, embed, table, file, mention, math, toggle); **`@mention`** semantics for Members and for cross-Page links; **version history** with point-in-time restore; **per-Block CRDT collaborative editing** (Yjs reused from FR-BRAIN-001); **draft → published lifecycle** with the publish action ingesting the page into BRAIN Layer 2 + Layer 3; **per-Space permissions** (private to Member, private to Project, tenant-internal, client-visible). The schema and editor ship in this FR; AI Q&A + GraphRAG cross-page navigation lands in FR-KB-002; the frontend + permissions UX lands in FR-KB-003.

## Problem

The team's living knowledge today is split across Notion (engineering runbooks), Google Docs (client deliverables), Markdown READMEs (per-repo documentation), and tribal memory (everything else). The PRD §9.12 commits to KB as the canonical living-document substrate — Notion-style blocks, AI Q&A grounded in cited pages, BRAIN-fed retrieval — and notes that "knowledge base ingested into BRAIN; AI Q&A grounded in cited KB pages; Notion-style block editor" is the surface that closes the loop.

Three failure modes a small team must avoid:

- **Knowledge fragmentation.** A new hire onboarding cannot find "the Acme architectural decision from Q1 2025" because it lives in three places, none authoritative.
- **Stale-by-default.** A document with no clear owner + no review cadence drifts; six months later, half its content is wrong but no one notices.
- **AI grounding gap.** CUO answers "what's our policy on weekend on-call?" with hallucination if the KB is not BRAIN-ingested. The PRD's "no answer without a citation" property collapses without KB as a primary source.

## Proposed Solution

The shape of the answer is a `kb` schema, an Apollo subgraph, the block editor (a TipTap-based React component), the version + draft-published lifecycle, and the BRAIN ingestion pipeline.

**Schema.**

```sql
CREATE SCHEMA kb;

-- Spaces: top-level containers (e.g. "Engineering", "Client Engagements", "HR Internal").
CREATE TABLE kb.space (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  slug TEXT NOT NULL,                          -- "engineering", "client-engagements"
  name TEXT NOT NULL,
  description_md TEXT,
  visibility_default TEXT NOT NULL,            -- "private_member" | "private_project" | "tenant_internal"
                                               -- | "client_visible_summary" | "client_visible_full"
  primary_owner_member_id UUID NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  UNIQUE (tenant_id, slug)
);

-- Pages: documents inside a space; can be nested via parent_page_id.
CREATE TABLE kb.page (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  space_id UUID NOT NULL REFERENCES kb.space(id) ON DELETE CASCADE,
  parent_page_id UUID REFERENCES kb.page(id),
  slug TEXT NOT NULL,
  title TEXT NOT NULL,
  icon TEXT,                                   -- emoji or icon-name
  cover_image_url TEXT,
  status TEXT NOT NULL DEFAULT 'draft',         -- "draft" | "published" | "archived"
  visibility TEXT,                              -- inherits from space if null
  authors UUID[] NOT NULL,
  primary_author_member_id UUID NOT NULL,
  last_published_version INT,
  current_version INT NOT NULL DEFAULT 1,
  body_pgrn TSVECTOR_TYPE NOT NULL,             -- aggregated full-text from all blocks
  preview_text TEXT,                            -- first ~256 chars of plain content
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  ingested_to_brain_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  UNIQUE (tenant_id, space_id, slug)
);

CREATE INDEX page_space_idx ON kb.page (tenant_id, space_id, status);
CREATE INDEX page_pgrn_idx  ON kb.page USING pgroonga (body_pgrn);

-- Block: the atomic content unit; Notion-style polymorphic shape.
CREATE TABLE kb.block (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  page_id UUID NOT NULL REFERENCES kb.page(id) ON DELETE CASCADE,
  parent_block_id UUID REFERENCES kb.block(id),
  position TEXT NOT NULL,                       -- fractional indexing: "a0", "a0V", "a1" — collision-safe inserts
  block_kind TEXT NOT NULL,                     -- "paragraph" | "heading1" | "heading2" | "heading3"
                                                -- | "bullet_list_item" | "numbered_list_item" | "todo"
                                                -- | "quote" | "code" | "callout" | "divider"
                                                -- | "image" | "embed" | "table" | "file" | "math"
                                                -- | "toggle" | "mention_member" | "mention_page"
                                                -- | "issue_embed" | "decision_embed"
  content JSONB NOT NULL,                       -- shape varies by kind
  text_content TEXT,                            -- denormalised for FTS aggregation up to page.body_pgrn
  yjs_doc BYTEA,                                -- Yjs CRDT state vector for collaborative editing
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ
);

CREATE INDEX block_page_position_idx ON kb.block (tenant_id, page_id, position) WHERE deleted_at IS NULL;
CREATE INDEX block_kind_idx          ON kb.block (tenant_id, block_kind);

-- Version: a published snapshot of a page at a point in time.
CREATE TABLE kb.page_version (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  page_id UUID NOT NULL REFERENCES kb.page(id) ON DELETE CASCADE,
  version INT NOT NULL,
  title TEXT NOT NULL,
  body_md TEXT NOT NULL,                        -- canonicalised Markdown serialisation of the page
  blocks_snapshot JSONB NOT NULL,               -- complete blocks tree at this version
  published_by_member_id UUID NOT NULL,
  published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  publish_note_md TEXT,                         -- changelog entry; required when materially changed
  brain_l3_doc_id UUID,                         -- the BRAIN Layer 3 ingestion ID
  UNIQUE (tenant_id, page_id, version)
);

-- Comment + suggestion thread on a page or block.
CREATE TABLE kb.thread (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  page_id UUID NOT NULL REFERENCES kb.page(id) ON DELETE CASCADE,
  block_id UUID REFERENCES kb.block(id) ON DELETE CASCADE,
  thread_kind TEXT NOT NULL,                    -- "comment" | "suggestion"
  status TEXT NOT NULL DEFAULT 'open',          -- "open" | "resolved"
  created_by UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  resolved_by UUID,
  resolved_at TIMESTAMPTZ
);

CREATE TABLE kb.thread_message (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  thread_id UUID NOT NULL REFERENCES kb.thread(id) ON DELETE CASCADE,
  author_member_id UUID NOT NULL,
  body_md TEXT NOT NULL,
  mentions UUID[],
  edited_at TIMESTAMPTZ,
  deleted_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Per-Page ACL override (default permissions come from space.visibility_default).
CREATE TABLE kb.page_acl (
  tenant_id UUID NOT NULL,
  page_id UUID NOT NULL REFERENCES kb.page(id) ON DELETE CASCADE,
  member_id UUID NOT NULL,
  role TEXT NOT NULL,                           -- "viewer" | "commenter" | "editor"
  added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, page_id, member_id)
);
```

**Block editor.**

The editor is a TipTap-based React component (`@cyberskill/components/<KbBlockEditor>`). TipTap on top of ProseMirror is the canonical choice (Notion uses a similar architecture; the open-source ecosystem is mature). Each block kind is a TipTap node; the canonical 17 kinds in P1:

| Kind | Notes |
|---|---|
| `paragraph` | inline rich-text (bold, italic, link, code, strike, mention) |
| `heading1` `heading2` `heading3` | TOC-aware |
| `bullet_list_item` `numbered_list_item` | nested |
| `todo` | checkbox + Member assignment optional |
| `quote` | |
| `code` | language-tagged; syntax highlight via `shiki`; copy-button; line numbers toggle |
| `callout` | icon + body; emphasised colour (info/warn/success/danger) |
| `divider` | |
| `image` | inline / full-width; caption; per-tenant blob store via Stalwart's content-addressed store (FR-EMAIL-001 reused) |
| `embed` | YouTube / Loom / Figma / Vimeo via oEmbed; safe-link guard via the warn-page (FR-EMAIL-010) |
| `table` | rows + columns + header row; sortable in viewer; not for relational data (use a dedicated database block at P2 if needed) |
| `file` | attached file; clamav-scanned via the EMAIL chain; preview where supported |
| `math` | KaTeX rendering |
| `toggle` | collapsible content with a heading + body |
| `mention_member` | inline `@member` reference; resolves to Member federation |
| `mention_page` | inline `↗ Page` reference; bidirectional link recorded in `kb.page_link` |
| `issue_embed` | live-rendered PROJ issue card (FR-PROJ-005's drawer in miniature) |
| `decision_embed` | live-rendered DEC entry from the locked-decisions ledger (FR-CP-001) |

Editor toolbar exposes the kinds via slash command (`/`) — typing `/heading1` inserts a heading; typing `/code rust` inserts a Rust code block. Slash command UX is the floor (Notion convention); a separate fixed toolbar surfaces formatting and structure operations.

**Mentions.**

`@member` triggers the member picker (with avatar + role + recent activity hover card; same component as PROJ FR-PROJ-005). `@page` triggers a page picker scoped to current space then expanding to tenant (with previews). Mentions are reactive: a mention to a Member surfaces a Notify card on first save (`kb.page.mentioned`); a mention to a Page records a backlink in `kb.page_link{from_page_id, to_page_id}` so "what links here" answers correctly.

**Per-Block CRDT.**

Each block's `yjs_doc` holds a Yjs state vector (FR-BRAIN-001 reuses Yjs); concurrent edits to the same block (two Members on the same page) merge non-destructively at character level. Cross-block operations (insert a block, reorder via fractional indexing) are not CRDT — they go through the GraphQL mutation path with optimistic-update + server-confirm semantics (the same pattern as FR-PROJ-002). The fractional-indexing position string ("a0", "a0V", "a1") allows safe inserts between any two existing positions without renumbering.

**Version + draft-published lifecycle.**

A page is created as `status: 'draft'`. The author can edit freely; collaborators with `editor` role can edit too. Clicking **Publish**:

1. Computes a canonical Markdown serialisation of the blocks tree (deterministic: position-ordered traversal; block kinds map to Markdown extensions where possible, JSON-fenced where not).
2. Computes a diff vs. the prior published version; if the diff is non-trivial, prompts the author for a `publish_note_md` (changelog entry).
3. Increments `kb.page.current_version`; writes `kb.page_version` row.
4. Triggers BRAIN Layer 2 + Layer 3 ingestion: the canonical Markdown body becomes a Layer 3 doc with `source_kind: "kb.page"`; the page's facts are extracted by the BRAIN extractor (FR-BRAIN-002) into Layer 2 with provenance back to the page version.
5. Sets `kb.page.last_published_version` and `kb.page.ingested_to_brain_at`.
6. Audit row in scope `kb.{tenant}`.

A reader sees the latest published version unless they have edit permission and toggle "show draft". Draft edits do not touch BRAIN until republished. Unpublishing is reversible (it sets `last_published_version = NULL` but preserves history); BRAIN ingestion is paused and the page's facts are marked `status: 'archived'` (not deleted).

**Point-in-time restore.**

A page's history view shows every published version + the working draft. "Restore to v3" copies that version's blocks tree back into the live blocks; the next publish bumps to v(current+1) with `publish_note_md: "Restored from v3"`. Restore is a write — no historical mutation; the audit log captures the restore explicitly.

**GraphQL subgraph.**

```graphql
type Query {
  kbSpaces(scope: KbScope = MEMBER): [KbSpace!]!
  kbSpace(id: ID, slug: String): KbSpace
  kbPages(spaceId: ID, parentPageId: ID, status: String, first: Int = 50): KbPageConnection!
  kbPage(id: ID, slug: String): KbPage
  kbPageVersions(pageId: ID!): [KbPageVersion!]!
  kbPageBlocks(pageId: ID!): [KbBlock!]!
  kbSearch(query: String!, spaceIds: [ID!], statuses: [String!], first: Int = 50): [KbPage!]!
  kbBacklinks(pageId: ID!): [KbPage!]!
  kbThreads(pageId: ID!, blockId: ID): [KbThread!]!
}

type Mutation {
  kbCreateSpace(input: KbSpaceInput!): KbSpace!
  kbUpdateSpace(id: ID!, patch: KbSpacePatch!): KbSpace!
  kbCreatePage(input: KbPageInput!): KbPage!
  kbUpdatePage(id: ID!, patch: KbPagePatch!): KbPage!
  kbPublishPage(id: ID!, publishNoteMd: String): KbPageVersion!
  kbRestorePageVersion(pageId: ID!, version: Int!): KbPage!
  kbArchivePage(id: ID!, reason: String): KbPage!
  kbCreateBlock(input: KbBlockInput!): KbBlock!
  kbUpdateBlock(id: ID!, patch: KbBlockPatch!): KbBlock!
  kbDeleteBlock(id: ID!): Boolean!
  kbReorderBlocks(pageId: ID!, blockIds: [ID!]!): [KbBlock!]!
  kbAddPageMember(pageId: ID!, memberId: ID!, role: String!): Boolean!
  kbCreateThread(input: KbThreadInput!): KbThread!
  kbAddThreadMessage(threadId: ID!, body: String!): KbThreadMessage!
  kbResolveThread(threadId: ID!): KbThread!
}

type Subscription {
  kbPageStream(pageId: ID!): KbPageEvent!
  kbBlockStream(pageId: ID!): KbBlockEvent!
}
```

**MCP tool surface (read-only in this FR; AI tools in FR-KB-002).**

- `cyberos.kb.list_spaces`
- `cyberos.kb.get_space`
- `cyberos.kb.list_pages`
- `cyberos.kb.get_page`
- `cyberos.kb.search`
- `cyberos.kb.get_backlinks`
- `cyberos.kb.list_threads`

## Alternatives Considered

- **Use Notion via API** as the KB substrate. Rejected: residency + lock-in + latency; the platform's other modules (BRAIN, CUO, GENIE) need first-class access.
- **Markdown files in a git repo as the canonical store.** Rejected: per-block collaboration, mentions, embeds need typed structure; gitops as the editor floor is too engineering-heavy for everyone outside the engineering team.
- **AsciiDoc / restructuredText.** Rejected: TipTap + Markdown round-trip is the industry standard; the team's existing Markdown READMEs migrate cleanly.
- **Skip CRDT collaboration; let the last-saver win.** Rejected: the team co-authors KB pages routinely; CRDT is the floor.
- **Bring entire Notion-block-kind catalogue (50+).** Rejected: 17 kinds is the floor for P1; databases/synced-blocks/timelines come at P2 if demand justifies.

## Success Metrics

- **Primary metric.** P1 sprint demo passes: (1) the founder creates a Space + Page + 10 blocks of mixed kinds; (2) the page publishes; the version is recorded; the page is searchable via `kbSearch`; (3) a Member edits a block while another Member edits a different block on the same page concurrently — both edits land; (4) `@page` mention to a target page creates a backlink reachable via `kbBacklinks`.
- **Adoption metric.** Per Member ≥ 5 published pages by P1 → P2 exit (~50 across the team).
- **BRAIN ingestion.** 100% of published page versions ingested into BRAIN Layer 2 + Layer 3 within 60 s p95 of publish.

## Scope

**In-scope.**
- The `kb` schema with all six tables.
- The TipTap-based block editor with the 17 block kinds.
- Slash command + toolbar UX.
- Per-block CRDT for inline edits.
- Fractional-indexing positions for safe insertion.
- Draft → published lifecycle with deterministic Markdown serialisation.
- Point-in-time restore.
- BRAIN Layer 2 + Layer 3 ingestion pipeline.
- Mentions (Member + Page) with backlink recording.
- Issue + Decision embed blocks (live-render).
- Per-page ACL override.
- The seven read MCP tools.
- Audit integration in scope `kb.{tenant}`.

**Out-of-scope (deferred to FR-KB-002 / FR-KB-003).**
- AI Q&A "ask this page" + GraphRAG cross-page navigation (FR-KB-002).
- Frontend remote at `/kb` (FR-KB-003).
- Database blocks (P2 — relational tables inside KB).
- Synced blocks (P2 — block X mirrors block Y across pages).
- Public publishing (P4 — Client Portal exposes `client_visible` pages).
- Asynchronous translation between vi-VN and en-US (P3).

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001.
- FR-BRAIN-001 (Yjs library).
- FR-BRAIN-002 / FR-BRAIN-003 (BRAIN ingestion targets).
- FR-DESIGN-001 (`@cyberskill/components`).
- FR-EMAIL-001 (content-addressed blob store reused).
- FR-EMAIL-010 (warn-page reused for embed safe-link guard).
- FR-PROJ-001 (issue_embed live-render reads from PROJ).
- FR-CP-001 (decision_embed reads from `cp.decision`).
- TipTap + ProseMirror + Yjs vendored at known versions.
- `shiki` for code-block syntax highlight; `KaTeX` for math rendering; `oEmbed` for embed previews (proxy through our backend to avoid third-party CSP exposure).
- Compliance: PDPL Decree 13 (KB pages contain personal data; per-tenant residency + the BRAIN denylist on ingestion).
- Locked decisions referenced: DEC-127 (TipTap as block-editor floor), DEC-128 (17 block kinds in P1; databases + synced-blocks defer P2), DEC-129 (publish triggers BRAIN ingestion; draft never ingested), DEC-130 (fractional indexing for block positions).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The schema + editor are deterministic; AI surfaces (Q&A, GraphRAG) live in FR-KB-002.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
