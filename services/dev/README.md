# CyberOS Local Dev Infra

This compose stack starts the shared local Postgres and Redis dependencies for
AUTH, MEMORY, and PROJ live tests. It builds the same Postgres 16 image used by
the VPS profile, with both Apache AGE and pgvector enabled.

## Start

```bash
docker compose -f services/dev/docker-compose.yml up -d
```

Fresh volumes create four databases:

| Database | Use |
|---|---|
| `cyberos` | Legacy scratch DB |
| `cyberos_auth` | `services/auth/migrations` |
| `cyberos_memory` | `services/memory/migrations` |
| `cyberos_proj` | `services/proj/migrations` |

The module databases are separate because each service has its own sqlx
migration sequence starting at `0001`; putting them in one DB would collide in
`_sqlx_migrations`.

## Migrate

```bash
scripts/local-live-test.sh migrate
```

It uses a Dockerized Postgres client and does not require local `sqlx` or
`psql`.

## Stop

```bash
docker compose -f services/dev/docker-compose.yml down
```

Use `down -v` only when you want to delete local databases and re-run
`postgres-init.sql` from scratch.
