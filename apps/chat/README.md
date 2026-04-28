<!-- module: CHAT -->

# @cyberos/chat — Internal Chat

> Phase **P0** · Port **4005** · GraphQL namespace `chat` · Prisma schema `chat` · MCP namespace `chat.*`

## What this module owns

Single-responsibility: Internal Chat. See [SRS §4 — Module CHAT](../../docs/SRS.md) for the authoritative spec.

## Functional requirements

This module implements the FRs under [`docs/feature-requests/P0/CHAT/`](../../docs/feature-requests/P0/CHAT). Each FR is a separate `feature_request@1` markdown file that links back to the SRS line; pick one up, mark it `in_progress` in the frontmatter, ship a PR.

## Run locally

```bash
# from the repo root
pnpm install
pnpm dev --filter @cyberos/chat      # subgraph on http://localhost:4005/graphql
curl http://localhost:4005/health
```

The subgraph picks up `PORT_CHAT` from `.env` (default `4005`).

## File shape (all 21 modules look identical)

```
apps/chat/
├── package.json
├── tsconfig.json
├── vitest.config.ts
├── Dockerfile
├── README.md                   <- you are here
└── src/
    ├── index.ts                <- subgraph entry — boots through @cyberos/subgraph-kit
    ├── graphql/
    │   ├── schema.ts           <- federated SDL
    │   └── resolvers/          <- one file per top-level type
    ├── db/
    │   └── schema.prisma       <- module-scoped Prisma schema
    ├── mcp/
    │   └── tools.ts            <- MCP tools (`chat.*`)
    ├── events/
    │   ├── publishers.ts       <- events this module emits
    │   └── subscribers.ts      <- events this module consumes
    └── services/               <- business logic (thin resolvers, fat services)
```

> **Do not invent a different shape.** If you need a deviation, raise it in `#cyberos-eng` — the structure is enforced by `pnpm gen:module` and validated in CI.

## Dependencies

- `AUTH` — see [docs/feature-requests/](../../docs/feature-requests/) for the dependency contract
- `BRAIN` — see [docs/feature-requests/](../../docs/feature-requests/) for the dependency contract
- `GENIE` — see [docs/feature-requests/](../../docs/feature-requests/) for the dependency contract

## Conventions

- **GraphQL:** every type prefixed with `Chat`; query/mutation field names start with `chat`. Prevents cross-module name collisions when subgraphs compose.
- **MCP tools:** named `chat.{verb}` (snake_case). Required scopes declared in the tool definition; AUTH gates them at the gateway.
- **Events:** emitted as `chat.{verb}.{noun}` per SRS §5.4. At-least-once; consumers MUST dedupe via `event.idempotency_key`.
- **DB:** all models live in Postgres schema `chat`. Cross-module reads go through GraphQL federation, never raw Prisma.
- **Errors:** throw `CyberOSError` (or one of `Errors.*`) from `@cyberos/shared`; the formatter turns them into uniform GraphQL/MCP error codes.
