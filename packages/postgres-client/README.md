# packages/postgres-client

> **Scope:** Tenant-aware Postgres client (schema-per-tenant + RLS, DEC-013)

This is a stub. Replace with real code as the FR cluster ships.

## Feature Requests

See `docs/tasks/` for full specs. The FR cluster covered here:

```
Tenant-aware Postgres client (schema-per-tenant + RLS, DEC-013)
```

## Status

`stub` — directory exists, no implementation yet. Created 2026-05-03.

## Wiring

- TypeScript + pnpm workspace package
- Build orchestrated by Turborepo (root `turbo.json`)
- Apollo Federation v2 subgraph (services only) / Module Federation remote (frontends only) / shared library (packages only)
