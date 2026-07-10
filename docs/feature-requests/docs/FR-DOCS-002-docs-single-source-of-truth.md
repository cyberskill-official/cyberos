---
id: FR-DOCS-002
title: Documentation single source of truth - module-owned markdown, generated website
module: docs
class: product
status: implementing
priority: MUST
depends_on: []
routed_back_count: 0
awh: N/A
---

# FR-DOCS-002 - Documentation single source of truth

## Context (the architecture decision)

The public wiki lives in `website/` as hand-authored HTML that duplicates and drifts from the real sources. Decision (operator 2026-07-10, format delegated): documentation moves to ONE source of truth, authored in Markdown with a separated `assets/` folder per scope, and every display surface (public website, web console, desktop app) renders from that source. HTML is an OUTPUT of the site build, never an authoring format. Ownership follows the code: each module owns its own documentation and step-by-step guides next to its code; only genuinely global artifacts stay in `docs/`.

Why markdown over authored HTML: agents and humans author it natively, it diffs cleanly in review, one source renders to all three platforms, and the site build already generates its reference pages (FR/NFR catalogs, changelogs) from repo data - this extends that proven pattern to the doctrine pages instead of maintaining a parallel hand-written HTML tree.

## 1. Normative clauses

1. Module-owned docs MUST live at `modules/<m>/docs/` and `services/<s>/docs/`, each containing `index.md` (the module page), optional `guides/*.md` (step-by-step guidelines), optional `appendices.md`, and an `assets/` folder for that module's images/files. Global artifacts (architecture, ADRs, deploy, strategy, feature-requests, glossary-level reference) MUST stay under `docs/` with `docs/assets/`.
2. The website MUST be generated: a deterministic builder (`website/build/render-docs.mjs`, wired into `website/build/build.sh`) walks the global docs set and every module/service `docs/` folder, renders markdown into the existing site chrome (nav/styles/tokens), copies each scope's `assets/`, and writes under `website/docs/`. Same input MUST produce byte-identical output (matches FR-DOCS-001 §1 #3).
3. The markdown renderer MUST be dependency-free (no node_modules), supporting the documented subset: headings, paragraphs, bold/italic/inline code, links, images, fenced code blocks, ordered/unordered lists, tables, blockquotes, and horizontal rules. Unsupported constructs MUST pass through as escaped text, never break the build.
4. Every hand-authored HTML doctrine page currently in `website/docs/` (architecture pages, module index/appendices pages, getting-started, glossary, risk-register) MUST be migrated to a markdown source at its owning scope, and the generated page replaces the hand-authored file. Already-generated pages (fr-catalog, nfr-catalog, changelogs) keep their existing builders.
5. A `docs-manifest` per scope is NOT required: the builder MUST derive the nav from the filesystem (module list + each scope's files), so adding a doc is adding a file.
6. The migration MUST NOT lose content: each migrated page's headings and body text are preserved (markdown conversion), with assets relocated to the owning scope's `assets/`.
7. `cyberos doctor`-style honesty: the builder MUST fail non-zero on a missing referenced asset or unreadable source file.

## 2. Acceptance criteria

- [ ] `website/build/build.sh` full build regenerates the whole docs site from md sources; running twice yields no diff.
- [ ] All previously hand-authored doctrine pages exist as md under their owning scope; the old hand-authored HTML files are gone from version control.
- [ ] Site renders with the existing chrome (nav, styles) and working asset links.
- [ ] Web console and desktop app can consume the same md sources (path contract documented in this FR; their viewers are follow-up FRs).

## 3. Gate

Machine: build twice + diff (determinism), link/asset check pass, `bash website/build/build.sh` exit 0. Review + final acceptance: HITL per STATUS-REFERENCE §1.4.
