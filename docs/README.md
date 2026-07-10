# `docs/` - global documentation sources

The home of every artifact that is genuinely global (FR-DOCS-002): specs, architecture, deploy runbooks, strategy, and reference sources. Module-owned documentation does NOT live here - each module keeps its own pages at `modules/<m>/docs/` (or `services/<s>/docs/` for service-implemented modules).

All user-facing documentation is served by the generated [docs site](https://cyberos-wiki.cyberskill.world/); everything under this folder is a markdown source for that build, never hand-authored HTML.

| Folder | Purpose |
|---|---|
| `feature-requests/` | The FR corpus: 489 FR specs across 29 domains (spec + `_audits/` companions). `BACKLOG.md` is the index of remaining active work; FR frontmatter `status` is the record of truth. Improvement-class FRs live in the same tree (`improvement/` + `(improvement)` tags in the backlog). |
| `non-functional-requirements/` | NFR specs with audit companions. |
| `architecture/` | Current-state architecture pages (tech stack, infrastructure, compliance, milestones, strategy, verification gate). |
| `deploy/` | Runbooks: `RELEASE.md` (the release process), go-live, VPS topology, CI/local checks, auth SSO. |
| `reference/` | Getting started, glossary and risk-register sources. |
| `strategy/` | Long-form strategy and audit reports (dated; historical by nature). |
| `reviews/` | Findings files awaiting operator rulings. |
| `assets/` | Images/files referenced by global pages. |

## How the site is built

`tools/docs-site/build.sh` renders every markdown source (this folder + each module/service `docs/`) plus the generated reference pages (FR catalog, NFR catalog, changelogs) into gitignored `dist/website/`. Nothing generated is committed.

    bash tools/docs-site/build.sh          # full site
    bash tools/docs-site/build.sh --docs   # doctrine pages only
    bash tools/docs-site/build.sh --fr     # FR catalog only

Freshness is enforced twice: the `docs-site-build` pre-commit hook verifies the build is green whenever a doc source is staged, and the `docs-prerender-gate` CI workflow rebuilds the site on every docs-touching PR.

## Why FR/NFR specs stay as raw markdown

They are workflow deliverables, not prose documentation: the `feature-request-author` skill generates them, `feature-request-audit` scores them, the ship-feature-requests workflow drives their `status`, and the site build reads their frontmatter to render the catalog pages.

## How the layers relate

```
SDP ── normative ──▶ FRs ── authority ──▶ Skill catalog ── compose ──▶ CUO workflows
                                                                            │
                                                                            ▼
                                                                      memory (memory module)
                                                                            │
                                                                            ▼
                                                                   Implementation modules
                                                                   (ai-gateway, auth, mcp, …)
```

- SDP - defines the stages every deliverable flows through (see [CUO appendices](https://cyberos-wiki.cyberskill.world/modules/cuo/appendices.html)).
- C-Suite Reference - defines the personas + the schema each persona spec must render (same appendices).
- FRs (`feature-requests/`) capture every concrete change request, tagged by phase + module.
- Skill catalog (`modules/skill/`) ships the author+audit pairs that materialise SDP stages into agentic Skills.
- CUO workflows (`modules/cuo/`) chain Skills into persona-owned deliverables.
- memory (`modules/memory/`) records every chain decision in an append-only audit chain.
- Implementation modules are the runtime services (`services/auth`, `services/ai-gateway`, etc.) that satisfy the FRs.

## Authoring rules (the short version)

- Markdown + per-scope `assets/` only; HTML is build output.
- A page describes the current state; history goes to the changelog sections.
- Module content belongs to the module's own `docs/` folder, not here.
- Adding a page = adding a file (the nav derives from the filesystem).
