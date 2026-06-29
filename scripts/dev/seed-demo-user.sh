#!/usr/bin/env bash
# Seed a demo workspace + user into the auth DB so you can sign in locally.
# Dev only. Override with DEMO_TENANT / DEMO_HANDLE / DEMO_EMAIL / DEMO_PASSWORD.
set -euo pipefail

PGC="${PGCONTAINER:-cyberos-postgres}"
TENANT_SLUG="${DEMO_TENANT:-cyberskill}"
DISPLAY="${DEMO_DISPLAY:-CyberSkill}"
HANDLE="${DEMO_HANDLE:-@stephen}"
EMAIL="${DEMO_EMAIL:-stephen@cyberskill.world}"
PASSWORD="${DEMO_PASSWORD:-CyberOS-Demo-2026!}"

# bcrypt hash: prefer htpasswd, fall back to python bcrypt.
HASH=""
if command -v htpasswd >/dev/null 2>&1; then
  HASH="$(htpasswd -nbBC 12 x "$PASSWORD" | cut -d: -f2)"
else
  HASH="$(P="$PASSWORD" python3 -c "import bcrypt,os;print(bcrypt.hashpw(os.environ['P'].encode(),bcrypt.gensalt(12)).decode())" 2>/dev/null || true)"
fi
[ -n "${HASH}" ] || { echo "could not produce a bcrypt hash (need 'htpasswd' or python 'bcrypt')" >&2; exit 1; }

docker exec -i "$PGC" psql -U cyberos -d cyberos -v ON_ERROR_STOP=1 \
  -v hash="$HASH" -v slug="$TENANT_SLUG" -v disp="$DISPLAY" -v handle="$HANDLE" -v email="$EMAIL" >/dev/null <<'SQL'
WITH t AS (
  INSERT INTO tenants (id, slug, display_name) VALUES (gen_random_uuid(), :'slug', :'disp')
  ON CONFLICT (slug) DO UPDATE SET display_name = EXCLUDED.display_name
  RETURNING id
)
INSERT INTO subjects (tenant_id, handle, display_name, email, kind, password_hash, status, roles)
SELECT id, :'handle', :'disp', :'email', 'human', :'hash', 'active', ARRAY['admin']::text[] FROM t
ON CONFLICT (tenant_id, handle) DO UPDATE SET password_hash = EXCLUDED.password_hash, status = 'active', roles = EXCLUDED.roles;
SQL

echo "seeded: workspace=${TENANT_SLUG}  handle=${HANDLE}  password=${PASSWORD}"
