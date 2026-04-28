# CyberOS

> AI-native modular internal operations platform built by CyberSkill, for CyberSkill — and, eventually, for the world.
>
> *Turn Your Will Into Real.*

---

## What this repository is

CyberOS is a multi-tenant operational platform composed of **22 independently deployable modules**, each a federated GraphQL subgraph + Module Federation frontend remote + MCP toolset + Postgres schema slice + NATS event producer. The platform replaces CyberSkill's fragmented stack (Notion, Slack/Zalo, Asana, HubSpot, Gmail, Excel-payroll, paper contracts) with one cohesive AI-rich system, internal-first, with external commercialization gated to Phase 4.

This repository holds **everything that governs CyberOS**: the spec, the roadmap, the templates, and the tooling that turns the spec into work tickets.

There are no separate ADR files, no separate compliance documents, no separate runbooks at v1.0. The PRD and the SRS together are the only governing documents.

---

## Where to start

| If you are… | Read first |
|---|---|
| New to the project | [PRD.md §1 Executive Summary](./PRD.md), then [PRD.md §5 Module Catalog](./PRD.md), then [PRD.md §8 Phase Plan](./PRD.md) |
| An engineer about to implement | [SRS.md §0.5 ID Conventions](./SRS.md), then [SRS.md §4 Per-Module Specs](./SRS.md), then the FR files under [feature-requests/](./feature-requests/) |
| A PM filing a new feature | [CONTRIBUTING.md](./CONTRIBUTING.md), then [templates/feature_request.md](./templates/feature_request.md) |
| Building a roadmap view | [ROADMAP.md](./ROADMAP.md) and [roadmap/tasks.yaml](./roadmap/tasks.yaml) |
| On the compliance working group | [PRD.md §10](./PRD.md) + [SRS.md §7](./SRS.md) + [compliance/eu-ai-act-risk-classes.md](./compliance/eu-ai-act-risk-classes.md) |

---

## Repository layout

```
cyberos/                                  # pnpm workspace + Turborepo monorepo
├── docs/                                 # all documentation lives here
│   ├── README.md                         # this file — the hub
│   ├── PRD.md                            # Product Requirements Document v1.0
│   ├── SRS.md                            # Software Requirements Specification v1.0
│   ├── ROADMAP.md                        # phased FR tree (generated)
│   ├── CONTRIBUTING.md                   # filing rules, generator workflows, conventions
│   ├── compliance/eu-ai-act-risk-classes.md
│   ├── templates/                        # vendored from @cyberskill/templates v1.0.0
│   │   ├── feature-request/{FEATURE_REQUEST.md, README.md, README_VI.md}
│   │   ├── bug-report/{BUG_REPORT.md, README.md, README_VI.md}
│   │   └── pull-request/{PULL_REQUEST_TEMPLATE.md, README.md, README_VI.md}
│   ├── roadmap/tasks.yaml                # canonical FR inventory — feeds gen-features
│   └── feature-requests/{P0..P4}/{MOD}/FR-{MOD}-{NNN}.md   # 318 generated tickets
├── apps/                                 # 21 module subgraphs + shell + router
│   ├── _template/                        # canonical module shape (gen-module source)
│   ├── auth/  ai/  mcp/  obs/  chat/  brain/  genie/    (P0)
│   ├── proj/  time/  crm/  kb/  hr/  email/  rew/  learn/   (P1)
│   ├── inv/  esop/                                          (P2)
│   ├── res/  okr/                                           (P3)
│   └── doc/  cp/                                            (P4)
├── packages/                             # cross-cutting libraries — no module logic
│   ├── shared/                           # types, errors, tenancy primitives
│   ├── observability/                    # pino + traceparent + New Relic boot
│   ├── events/                           # NATS JetStream wrapper (§5.4 contract)
│   ├── db/                               # Prisma multi-schema + RLS helpers
│   ├── mcp-server/                       # MCP TS SDK wrapper for tool registration
│   └── subgraph-kit/                     # Apollo Server 5 + Express bootstrap
├── schemas/                              # vendored JSON Schemas (draft 2020-12)
├── scripts/                              # generator + validator CLIs
│   ├── gen-features.ts                   # tasks.yaml → 318 FR markdown files
│   ├── gen-roadmap.ts                    # tasks.yaml → ROADMAP.md
│   ├── gen-module.ts                     # modules.yaml + apps/_template → apps/{module}/
│   ├── validate-fr.ts                    # FR markdown validator
│   ├── validate-modules.ts               # modules.yaml + apps/ tree integrity check
│   └── lib/                              # shared parsing + schema helpers
├── modules.yaml                          # canonical 21-module manifest (port, schema, deps)
├── cyberskill.config.json                # pins @cyberskill/templates v1.0.0
├── pnpm-workspace.yaml  turbo.json  tsconfig.base.json  lefthook.yml
├── .github/workflows/ci.yml              # validate + typecheck + lint + test on every PR
├── package.json  tsconfig.json  .nvmrc  .editorconfig  .env.example  .gitignore
```

Three things make the structure rigid:

1. **`modules.yaml`** — the 21 modules and their fixed properties (port, package name, GraphQL/Prisma/MCP namespace, deps). Adding a 22nd module is one entry plus `pnpm gen:module`.
2. **`apps/_template/`** — the canonical 14-file shape every module conforms to. `pnpm gen:module` stamps it.
3. **`pnpm validate:modules`** — CI gate that catches duplicate ports, orphan folders, broken `depends_on`, and modules missing from the manifest.

> **Templates source-of-truth:** the `docs/templates/` and `schemas/` trees are vendored copies of [`@cyberskill/templates`](https://github.com/cyberskill-official/templates) v1.0.0 (pinned in `cyberskill.config.json`). Do not hand-edit them; bump the pin and re-vendor when the package releases.

> **Convention:** all documentation lives under `docs/`. There is no root-level README. Tooling (scripts, configs) lives at the repo root because it isn't documentation.

---

## Modules at a glance (22)

| Phase | Modules | Theme |
|---|---|---|
| **P0** (7) | AUTH · AI · MCP · OBS · CHAT · BRAIN · GENIE | Foundation: identity, multi-tenancy, AI gateway, MCP, observability, chat, knowledge layer, mascot |
| **P1** (8) | PROJ · TIME · CRM · KB · HR · EMAIL · REW (core) · LEARN | Run the company on CyberOS |
| **P2** (3) | INV · ESOP · REW (full pool) | Invoicing, phantom stock, project bonus pool |
| **P3** (2) | RES · OKR | Resource allocation, objectives & key results |
| **P4** (2) | DOC · CP | Document signing, client portal — first external tenant |

Full module catalog in [PRD §5](./PRD.md) and per-module specs in [SRS §4](./SRS.md).

---

## How tasks become tickets

1. The **SRS** is the source of truth. Every functional requirement has a stable `FR-{MOD}-{NNN}` ID.
2. [`docs/roadmap/tasks.yaml`](./roadmap/tasks.yaml) is the machine-readable inventory of every FR — module, phase, MoSCoW priority, EU AI Act risk class, dependencies. Bookkeeping fields live here, not in the artifact frontmatter.
3. [`scripts/gen-features.ts`](../scripts/gen-features.ts) reads `tasks.yaml` plus the canonical [`docs/templates/feature-request/FEATURE_REQUEST.md`](./templates/feature-request/FEATURE_REQUEST.md) and emits one `feature_request@1` markdown file per FR under `docs/feature-requests/{phase}/{module}/FR-{MOD}-{NNN}.md`. Frontmatter is canonical-only (no extra keys); body is English-only with a `<!-- source: SRS … -->` back-reference.
4. The validator (`scripts/validate-fr.ts`) enforces the canonical schema — the same checks that `npx cyberskill validate` will run, and that the future GitHub integration will run on every PR.
5. When CyberOS itself ships (P4) the same generator output will be POSTed to GitHub via `gh issue create` (or the GitHub MCP) so the PM workflow has zero migration cost.

Run the generators:

```bash
pnpm install

# FR tickets
pnpm gen:features              # 318 feature_request@1 files from tasks.yaml
pnpm gen:roadmap               # ROADMAP.md from tasks.yaml

# Module scaffolds
pnpm gen:module                # 21 module folders from modules.yaml + apps/_template
pnpm gen:module -- --module AUTH    # just one
pnpm gen:module -- --phase P0       # phase filter
pnpm gen:module -- --dry-run        # preview

# Validation
pnpm validate:fr               # FR markdown
pnpm validate:yaml             # tasks.yaml integrity
pnpm validate:modules          # modules.yaml + apps/ tree
pnpm validate:all              # all of the above
pnpm validate:templates        # delegate to the canonical cyberskill CLI
```

## How an engineer starts work

Step-by-step:

1. **Pick an FR.** Open [ROADMAP.md](./ROADMAP.md), find an unowned `[ ]` checkbox in the right phase/module, click through to the FR file, set `status: in_progress` in the frontmatter, set `author: @your-handle`, commit.
2. **Open the module.** `cd apps/{module}`, read the README, see what FRs are yours.
3. **Run the subgraph.** `pnpm dev --filter @cyberos/{module}` from repo root — boots Apollo Server 5 + Express on the assigned port; `curl http://localhost:{PORT}/health` should return `{ ok: true }`.
4. **Write the resolver / service / Prisma model.** Resolvers stay thin; logic in `src/services/`. Cross-module calls go through GraphQL federation, never raw Prisma.
5. **Add tests.** `vitest` is wired; add `*.test.ts` next to the file under test.
6. **Open a PR** using `templates/pull-request/PULL_REQUEST_TEMPLATE.md`. CI runs typecheck, lint, validate, build, test. Merge requires all green.

---

## Status

- **PRD:** v1.0, approved 2026-04-28 (CYBEROS-PRD-1.0)
- **SRS:** v1.0, approved 2026-04-28 (CYBEROS-SRS-1.0)
- **Phase:** P0 in flight — see [ROADMAP.md](./ROADMAP.md) and [PRD §8](./PRD.md)

---

## Governance

Changes to PRD/SRS go through the change-control process in [PRD §14](./PRD.md). Architectural decisions (DEC-001..DEC-037) are captured inline in [SRS §3.3](./SRS.md). New decisions follow the maintenance process in SRS §3.4 — pin them with a new `DEC-{NNN}` and reference from the affected sections.
