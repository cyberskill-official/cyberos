# `docs/` - global documentation sources

The home of every artifact that is genuinely global (TASK-DOCS-002): specs, architecture, deploy runbooks, strategy, and reference sources. Module-owned documentation does NOT live here - each module keeps its own pages at `modules/<m>/docs/` (or `services/<s>/docs/` for service-implemented modules).

All user-facing documentation is served by the generated [docs site](https://cyberos-wiki.cyberskill.world/); everything under this folder is a markdown source for that build, never hand-authored HTML.

| Folder | Purpose |
|---|---|
| `tasks/` | The task corpus: 489 task specs across 29 domains (spec + `_audits/` companions). `BACKLOG.md` is the index of remaining active work; task frontmatter `status` is the record of truth. Improvement-class tasks live in the same tree (`improvement/` + `(improvement)` tags in the backlog). |
| `non-functional-requirements/` | NFR specs with audit companions. |
| `architecture/` | Current-state architecture pages (tech stack, infrastructure, compliance, milestones, strategy, verification gate). |
| `adrs/` | Architecture decision records. |
| `deploy/` | Runbooks: `RELEASE.md` (the release process), go-live, VPS topology, CI/local checks, auth SSO. |
| `reference/` | Getting started, glossary and risk-register sources. |
| `absorptions/` | The gate-aligned intake path for absorbing external projects (`INTAKE.md` + `incoming/`). |
| `auto-work/` | Ledgers of unattended agent sessions (dated evidence records). |
| `ci/` | CI hardening notes and follow-ups. |
| `knowledge/` | Knowledge sources (e.g. `VN_GLOSSARY.md`, the glossary's source data). |
| `legal/` | Employment document templates (VN labor contract, NDNCA and IP, total rewards appendix). |
| `verification/` | Verification ledgers and gate design notes. |
| `strategy/` | Long-form strategy and audit reports (dated; historical by nature). |
| `reviews/` | Findings files awaiting operator rulings, and dated triage records (e.g. the pre-go-live known-issues log). |

Per-scope images/files live in an `assets/` folder next to the pages that use them (created on first need).

## How the site is built

`tools/docs-site/build.sh` renders every markdown source (this folder + each module/service `docs/`) plus the generated reference pages (task catalog, NFR catalog, changelogs) into gitignored `dist/website/`. Nothing generated is committed.

    bash tools/docs-site/build.sh          # full site
    bash tools/docs-site/build.sh --docs   # doctrine pages only
    bash tools/docs-site/build.sh --fr     # task catalog only

Freshness is enforced twice: the `docs-site-build` pre-commit hook verifies the build is green whenever a doc source is staged, and the `docs-prerender-gate` CI workflow rebuilds the site on every docs-touching PR.

## Why task/NFR specs stay as raw markdown

They are workflow deliverables, not prose documentation: the `task-author` skill generates them, `task-audit` scores them, the ship-tasks workflow drives their `status`, and the site build reads their frontmatter to render the catalog pages.

## How the layers relate

```
SDP ── normative ──▶ tasks ── authority ──▶ Skill catalog ── compose ──▶ CUO workflows
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
- tasks (`tasks/`) capture every concrete change request, tagged by phase + module.
- Skill catalog (`modules/skill/`) ships the author+audit pairs that materialise SDP stages into agentic Skills.
- CUO workflows (`modules/cuo/`) chain Skills into persona-owned deliverables.
- memory (`modules/memory/`) records every chain decision in an append-only audit chain.
- Implementation modules are the runtime services (`services/auth`, `services/ai-gateway`, etc.) that satisfy the tasks.

## Authoring rules (the short version)

- Markdown + per-scope `assets/` only; HTML is build output.
- A page describes the current state; history goes to the changelog sections.
- Module content belongs to the module's own `docs/` folder, not here.
- Adding a page = adding a file (the nav derives from the filesystem).
