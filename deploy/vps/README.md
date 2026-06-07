# CyberOS VPS Deployment

This is the first production-shaped deployment path for CyberOS. It runs the
real AUTH and MEMORY Rust services, applies AUTH/MEMORY/PROJ SQL migrations,
starts Postgres with Apache AGE + pgvector, and puts Caddy in front of the
internal services.

CHAT is included as an optional Compose profile because the Mattermost fork
build is network-heavy and slower than the core AUTH/MEMORY stack.

## Local Smoke

Start Docker Desktop, then run:

```bash
deploy/vps/scripts/local-smoke.sh
```

The script creates `deploy/vps/.env.local` if it is missing, builds the core
images, starts Postgres/Redis/migrator/AUTH/MEMORY/Caddy, and verifies:

```text
http://127.0.0.1:8080/healthz
http://127.0.0.1:8080/auth/healthz
http://127.0.0.1:8080/memory/healthz
```

## VPS Bootstrap

On a fresh Ubuntu VPS:

```bash
sudo REPO_DIR=/opt/cyberos/repo deploy/vps/scripts/bootstrap.sh
sudo nano /opt/cyberos/repo/deploy/vps/.env
sudo systemctl start cyberos
```

For a public VPS, update `.env`:

```env
CYBEROS_HTTP_BIND=0.0.0.0:80
CYBEROS_HTTPS_BIND=0.0.0.0:443
CYBEROS_CADDY_SITE=cyberos.example.com
AUTH_JWT_ISSUER=https://cyberos.example.com/auth
```

Then point DNS at the VPS and restart:

```bash
sudo systemctl restart cyberos
```

## Manual Commands

```bash
docker compose --env-file deploy/vps/.env -f deploy/vps/compose.prod.yml up -d --build
docker compose --env-file deploy/vps/.env -f deploy/vps/compose.prod.yml ps
docker compose --env-file deploy/vps/.env -f deploy/vps/compose.prod.yml logs -f cyberos-auth cyberos-memory
docker compose --env-file deploy/vps/.env -f deploy/vps/compose.prod.yml down
```

Run the optional CHAT profile:

```bash
docker compose --env-file deploy/vps/.env -f deploy/vps/compose.prod.yml --profile chat up -d --build
```

## Backups

```bash
deploy/vps/scripts/backup.sh deploy/vps/.env
```

This writes timestamped `pg_dump -Fc` files for the CyberOS databases and a
tarball of `CYBEROS_MEMORY_ROOT` when that path exists.

Restore into an already-running stack:

```bash
deploy/vps/scripts/restore.sh deploy/vps/.env backups/20260526T120000Z
```

## Notes

- Do not reuse `env.production.example` secrets on a VPS.
- Keep `POSTGRES_PASSWORD` URL-safe or URL-encode it in the `*_DATABASE_URL`
  values.
- PROJ currently has migrations and Rust library coverage but no HTTP daemon.
  The deployment applies PROJ schema so the future daemon can be added without
  changing the database layout.
