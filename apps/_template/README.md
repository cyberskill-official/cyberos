<!-- module: {{MODULE}} -->

# {{PACKAGE}} — {{NAME}}

> Phase **{{PHASE}}** · Port **{{PORT}}** · GraphQL namespace `{{NAMESPACE}}` · Prisma schema `{{SCHEMA}}` · MCP namespace `{{NAMESPACE}}.*`

## What this module owns

Single-responsibility: {{NAME}}. See [SRS §4 — Module {{MODULE}}](../../docs/SRS.md) for the authoritative spec.

## Functional requirements

This module implements the FRs under [`docs/feature-requests/{{PHASE}}/{{MODULE}}/`](../../docs/feature-requests/{{PHASE}}/{{MODULE}}). Each FR is a separate `feature_request@1` markdown file that links back to the SRS line; pick one up, mark it `in_progress` in the frontmatter, ship a PR.

## Run locally

```bash
# from the repo root
pnpm install
pnpm dev --filter {{PACKAGE}}      # subgraph on http://localhost:{{PORT}}/graphql
curl http://localhost:{{PORT}}/health
```

The subgraph picks up `PORT_{{MODULE}}` from `.env` (default `{{PORT}}`).

## File shape (all 21 modules look identical)

```
apps/{{module}}/
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
    │   └── tools.ts            <- MCP tools (`{{NAMESPACE}}.*`)
    ├── events/
    │   ├── publishers.ts       <- events this module emits
    │   └── subscribers.ts      <- events this module consumes
    └── services/               <- business logic (thin resolvers, fat services)
```

> **Do not invent a different shape.** If you need a deviation, raise it in `#cyberos-eng` — the structure is enforced by `pnpm gen:module` and validated in CI.

## Dependencies

{{DEPENDS_ON}}

## Conventions

- **GraphQL:** every type prefixed with `{{TYPE}}`; query/mutation field names start with `{{NAMESPACE}}`. Prevents cross-module name collisions when subgraphs compose.
- **MCP tools:** named `{{NAMESPACE}}.{verb}` (snake_case). Required scopes declared in the tool definition; AUTH gates them at the gateway.
- **Events:** emitted as `{{NAMESPACE}}.{verb}.{noun}` per SRS §5.4. At-least-once; consumers MUST dedupe via `event.idempotency_key`.
- **DB:** all models live in Postgres schema `{{SCHEMA}}`. Cross-module reads go through GraphQL federation, never raw Prisma.
- **Errors:** throw `CyberOSError` (or one of `Errors.*`) from `@cyberos/shared`; the formatter turns them into uniform GraphQL/MCP error codes.
