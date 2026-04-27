# CyberOS

**AI-native internal operations platform by CyberSkill.**  
Slogan: *Turn Your Will Into Real*

CyberOS is a multi-tenant, modular platform that runs CyberSkill's entire business — projects, time, CRM, HR, payroll, communications, knowledge, and AI tooling — on a single federated stack. It is built internal-first (CyberSkill is the only tenant through P3) and designed for global commercialization from P4.

---

## Documentation

| Document | Description |
|---|---|
| [PRD.md](./PRD.md) | Product Requirements Document — vision, user stories, module catalog, success metrics |
| [SRS.md](./SRS.md) | Software Requirements Specification — architecture, data models, GraphQL SDL, NFRs, DECs |
| [TASKS.md](./TASKS.md) | Full P0–P4 build checklist — every task across all phases and compliance gates |
| [CHANGELOG.md](./CHANGELOG.md) | Release history and notable changes per module |

---

## Architecture in one paragraph

Each of the 22 modules is an independent **Apollo Federation v2 subgraph** (Express + TypeScript) with its own Prisma schema, PostgreSQL RLS policies, and Module Federation MFE remote. Modules communicate via GraphQL entity refs for reads and NATS JetStream for domain events — no cross-module DB reads. The **Apollo GraphOS Router** composes all subgraphs into a single supergraph. The **MCP server** wraps every subgraph tool under `https://mcp.cyberos.vn/mcp` for AI-agent access. **BRAIN** (pgvector + tsvector hybrid index) ingests events from all modules and powers semantic search. **GENIE** is the company AI assistant, omnipresent across every screen via a floating button and `⌘+G`.

---

## Module roadmap

| Phase | Modules |
|---|---|
| **P0** Core | AUTH · AI · MCP · OBS · CHAT · BRAIN · GENIE |
| **P1** MVP | PROJ · TIME · CRM · KB · HR · EMAIL · REW · LEARN |
| **P2** Ops | INV · ESOP · REW (full pool) |
| **P3** Strategy | RES · OKR |
| **P4** Commercial | DOC · CP |

---

## Quick start (local dev)

```bash
# Prerequisites: Node 22, pnpm 9, Docker

git clone git@github.com:cyberskill-official/cyberos.git
cd cyberos
pnpm install

# Start infra (Postgres 17+pgvector, Redis, NATS)
docker compose -f infra/docker/docker-compose.yml up -d

# Copy env and fill secrets (or use Doppler)
cp .env.example .env
# doppler setup && doppler run -- pnpm dev

# Run all DB migrations
pnpm db:migrate

# Start all modules
pnpm dev

# Or start a single module
pnpm --filter @cyberos/auth dev
```

---

## Repo layout

```
cyberos/
├── docs/           ← all documentation (PRD, SRS, TASKS, CHANGELOG, README)
├── apps/
│   └── shell/      ← Module Federation host app
├── modules/        ← 22 modules; each is a Federation subgraph + MFE remote
├── packages/       ← shared configs, GraphQL scalars, DB helpers
├── infra/
│   └── docker/     ← docker-compose for local dev infra
├── .env.example
├── package.json    ← pnpm workspace root
├── pnpm-workspace.yaml
└── turbo.json
```

Each module follows the pattern in `modules/auth/` — the canonical reference implementation.

---

## Tech stack

| Layer | Choice |
|---|---|
| Runtime | Node.js 22 + TypeScript 5.7 |
| Backend | Express 4 + Apollo Server 5 (no NestJS) |
| API | Apollo Federation v2 + GraphOS Router |
| Database | PostgreSQL 17 + pgvector + Neon + Prisma |
| Frontend | React + Vite · Module Federation (Rspack) |
| Design system | `@cyberos/ui` from `../Design System/` |
| Monorepo | pnpm 9 + Turborepo 2 + Changesets |
| Events | NATS JetStream |
| Queue | BullMQ on Redis |
| MCP | TypeScript SDK v2 · Streamable HTTP |
| Observability | New Relic (Node agent + Apollo plugin) |
| Secrets | Doppler |
| Hosting | Railway / Fly.io · Neon · Upstash · Cloudflare R2 |

Full locked decisions: [SRS.md §3.3](./SRS.md).

---

## Contributing

See `CONTRIBUTING.md` at the repo root. Every new module must pass the **module start checklist** before its first PR merges.

## License

Proprietary — © CyberSkill (Software Solutions Consultancy And Development Joint Stock Company). All rights reserved.
