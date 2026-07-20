# Provision a CyberOS P0 server from scratch

This reproduces the exact path used to stand up https://os.cyberskill.world on a fresh Vultr VPS, backed by Supabase, with auto-deploy on push to main. Follow it top to bottom on a new server and you get the same result. The gotchas we actually hit are called out inline so a redo is painless.

Architecture in one line: a small Vultr VPS in Singapore runs auth, chat, and Caddy as containers; the database is Supabase (managed Postgres); images are built in GitHub Actions and pulled by the VPS; the console is served by Caddy on one origin, so there is no CORS.

Reference docs this one points at instead of duplicating: deploy/vps/auto-deploy.md (the CI/CD setup) and docs/deploy/auth-google-sso-runbook.md (the tenant bootstrap and Google IdP).

## A. Provision the VPS (Vultr)

- Deploy a Server. Region Southeast Asia (Singapore) - closest to the team, about 30-50 ms; do not pick a US region, and do not use Hetzner (no Asia datacenter).
- Plan: voc-c-2c-4gb-50s (2 vCPU / 4 GB / 50 GB NVMe). 4 GB is enough because the database is on Supabase and the box never compiles Rust (CI does that).
- Image: Ubuntu LTS.
- Turn on Limited User Login (gives a non-root sudo user `linuxuser`), attach your SSH key, set a firewall group that allows only ports 22, 80, and 443, and enable VPC Network in Singapore (so a future AI box can reach it privately).
- Record the public IP (this server: 149.28.158.169).

## B. Base setup on the VPS (as linuxuser)

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker "$USER"
newgrp docker        # or log out and back in
docker ps            # should run with no sudo
```

## C. Give the VPS read access to the repo, then clone

```bash
ssh-keygen -t ed25519 -C "cyberos-vps-deploy" -f ~/.ssh/id_ed25519 -N ""
cat ~/.ssh/id_ed25519.pub
# Add that public key on GitHub: repo Settings -> Deploy keys -> Add (read-only is enough).
git clone git@github.com:cyberskill-official/cyberos.git ~/cyberos
chmod +x ~/cyberos/deploy/vps/deploy.sh
```

This deploy key is read-only and is separate from your personal key.

## D. Log Docker into GHCR (to pull the prebuilt images)

Create a GitHub personal access token (classic) with only the read:packages scope, then:

```bash
echo "<READ_PACKAGES_PAT>" | docker login ghcr.io -u <your-github-username> --password-stdin
```

Gotcha: do not paste the token anywhere it gets shown or logged. If it is ever exposed, delete it on GitHub and create a new one.

## E. Let GitHub Actions SSH in (for auto-deploy)

```bash
ssh-keygen -t ed25519 -C "cyberos-ci" -f ~/ci_key -N ""
cat ~/ci_key.pub >> ~/.ssh/authorized_keys
cat ~/ci_key           # copy this PRIVATE key into the GitHub secret VPS_SSH_KEY, then:
rm ~/ci_key*
```

Then in GitHub, repo Settings -> Secrets and variables -> Actions, add: VPS_HOST (the IP or domain), VPS_USER (`linuxuser`), VPS_SSH_KEY (the private key just printed). This CI key is separate from the read-only deploy key in step C.

## F. Create the Supabase database

- New project, region Southeast Asia (Singapore). Unlink the GitHub integration, uncheck "Enable Data API" (CyberOS connects straight to Postgres and does not use Supabase's REST API), keep Postgres (default, not OrioleDB).
- Set and save the database password. Tip: make it letters and numbers only so you never have to percent-encode it.
- Get the connection string from the Connect button, the Session pooler tab.

Two gotchas that cost us time:
- Use the Session pooler string (port 5432), not the Direct connection. The direct `db.<ref>.supabase.co` host is IPv6-only and will not connect from an IPv4 VPS. The pooler host looks like `aws-1-ap-southeast-1.pooler.supabase.com` and the username becomes `postgres.<project-ref>`.
- Append `?sslmode=require`, and percent-encode any special characters in the password (`@` -> %40, `'` -> %27, and so on). An un-encoded `@` or apostrophe breaks the URL.

## G. Create .env.p0 on the VPS

The .env.p0.example template is gitignored (it matches the `.env.*` secret rule), so create the file directly on the box:

```bash
cd ~/cyberos/deploy/vps
cat > .env.p0 <<'EOF'
CYBEROS_APP_DOMAIN=os.cyberskill.world
CYBEROS_APP_URL=https://os.cyberskill.world
COMPOSE_PROJECT_NAME=cyberos-p0
CYBEROS_IMAGE_TAG=latest

AUTH_DATABASE_URL=PASTE_SESSION_POOLER_URL
CHAT_DATABASE_URL=PASTE_SESSION_POOLER_URL
CHAT_AUDIT_DATABASE_URL=
EOF
chmod 600 .env.p0
nano .env.p0   # paste the session-pooler URL (same value) into both lines, with ?sslmode=require
```

## H. Apply the migrations to Supabase (BEFORE the first deploy)

```bash
sudo apt-get update && sudo apt-get install -y postgresql-client
cd ~/cyberos
 export DB_URL='<the same session-pooler URL>'   # leading space keeps it out of shell history
for f in services/auth/migrations/*.sql services/chat/migrations/*.sql; do
  echo "applying $(basename "$f")"
  psql "$DB_URL" -v ON_ERROR_STOP=1 -f "$f"
done
unset DB_URL
```

Gotcha (the one that bit us): apply migrations before the first deploy. auth loads its signing keys from a table at boot, so with an empty database the auth container stays unhealthy and the rollout stops. If a `CREATE EXTENSION` (pg_trgm / unaccent) line errors, enable it from the Supabase dashboard (Database, Extensions) and re-run - the migrations are idempotent.

## I. Point DNS at the box

Add a DNS A record: host `os`, value the VPS IP (149.28.158.169), DNS-only (grey cloud if on Cloudflare, so Caddy can complete the TLS challenge). Verify:

```bash
dig +short os.cyberskill.world      # must print the VPS IP
```

## J. First deploy

```bash
bash ~/cyberos/deploy/vps/deploy.sh
```

It pulls the auth, chat, and caddy images and starts the stack. Verify with `docker ps` (auth shows healthy) and open https://os.cyberskill.world/ - Caddy fetches the TLS certificate on the first request, then the console login page loads. From here on, a push to main builds and deploys on its own (step E set that up; details in deploy/vps/auto-deploy.md).

## K. Seed the team tenant and a first admin

The schema exists but has no users yet, so a sign-in would fail. Seed the team tenant plus your admin in one transaction. The tenants and subjects tables FORCE row-level security, and the policies only let the root context create a tenant or cross tenant boundaries, so the seed sets `app.current_tenant_id` to the root tenant (`00000000-0000-0000-0000-000000000000`). The running auth already created its signing key on boot, so no key seed is needed. Run on the VPS (it reads the database URL from .env.p0 and prompts for the admin password):

```bash
sudo apt-get install -y apache2-utils                      # provides htpasswd (bcrypt)
DB_URL="$(grep -m1 '^AUTH_DATABASE_URL=' ~/cyberos/deploy/vps/.env.p0 | cut -d= -f2-)"
printf 'Admin password (min 12 chars): '; read -rs PW; echo
HASH="$(htpasswd -nbBC 12 x "$PW" | cut -d: -f2)"

psql "$DB_URL" -v ON_ERROR_STOP=1 -v hash="$HASH" <<'SQL'
BEGIN;
SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000';
INSERT INTO tenants (id, slug, display_name)
VALUES (gen_random_uuid(), 'cyberskill', 'CyberSkill')
ON CONFLICT (slug) DO UPDATE SET display_name = EXCLUDED.display_name;
INSERT INTO subjects (tenant_id, handle, display_name, email, kind, password_hash, status, roles)
SELECT t.id, '@stephen', 'Stephen Cheng', 'stephen@cyberskill.world', 'human', :'hash', 'active', ARRAY['admin']::text[]
FROM tenants t WHERE t.slug = 'cyberskill'
ON CONFLICT (tenant_id, handle) DO UPDATE SET password_hash = EXCLUDED.password_hash, status = 'active', roles = EXCLUDED.roles;
INSERT INTO subject_roles (tenant_id, subject_id, role, granted_by)
SELECT s.tenant_id, s.id, 'tenant-admin', s.id
FROM subjects s JOIN tenants t ON t.id = s.tenant_id
WHERE t.slug = 'cyberskill' AND s.handle = '@stephen'
ON CONFLICT (subject_id, role) DO NOTHING;
COMMIT;
SQL
unset PW HASH DB_URL
```

Test it: sign in at https://os.cyberskill.world/ with workspace `cyberskill`, handle `@stephen`, and that password. You should land on the dashboard. Use the email at your Workspace domain so a later Google sign-in links to this same account.

## L. Register Google sign-in

Create a Google OAuth client (Web application) with the redirect URI `https://os.cyberskill.world/v1/auth/oidc/callback`, then register it. Get a token from your admin login and POST the IdP config with allowed_domains set to your Workspace domain (create_idp_config needs only a valid token, not admin):

```bash
printf 'Your admin password: '; read -rs PW; echo; export PW
TOKEN="$(python3 -c 'import json,os,urllib.request as u;d=json.dumps({"grant_type":"password","tenant_slug":"cyberskill","handle":"@stephen","password":os.environ["PW"]}).encode();print(json.load(u.urlopen(u.Request("https://os.cyberskill.world/v1/auth/token",d,{"content-type":"application/json"})))["access_token"])')"
unset PW
read -rs -p 'Google client secret: ' GOOGLE_SECRET; echo; export GOOGLE_SECRET
curl -s -X POST https://os.cyberskill.world/v1/admin/oidc/idp-configs \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d "$(GOOGLE_CLIENT_ID='YOUR_CLIENT_ID' python3 -c 'import json,os;print(json.dumps({"name":"google","discovery_url":"https://accounts.google.com/.well-known/openid-configuration","client_id":os.environ["GOOGLE_CLIENT_ID"],"client_secret":os.environ["GOOGLE_SECRET"],"redirect_uri":"https://os.cyberskill.world/v1/auth/oidc/callback","allowed_domains":["cyberskill.world"],"auto_provision":True,"default_roles":["tenant-member"]}))')"
echo; unset GOOGLE_SECRET
```

A success returns a small JSON with an id and allowed_domains. Then the Sign in with Google button on the console works for any @cyberskill.world account; a personal Gmail is rejected. Offboarding and the deeper Google notes are in docs/deploy/auth-google-sso-runbook.md and docs/deploy/p0-google-chat-runbook.md.

## This server, for reference

- Vultr "CyberOS-Core", Singapore, 149.28.158.169, Ubuntu 26.04 LTS, 2 vCPU / 4 GB / 50 GB NVMe, user linuxuser.
- Domain os.cyberskill.world (A record to the IP), TLS via Caddy.
- Database: Supabase project (Singapore), session pooler, one project holds both auth and chat schemas.
- Images: ghcr.io/cyberskill-official/cyberos-{auth,chat}, built by .github/workflows/deploy.yml.
