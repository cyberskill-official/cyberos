# P0 go-live: Google sign-in plus team chat on one origin

This is the shortest path to your team using CyberOS in production: each person signs in with their
Google account, lands on the dashboard, opens Chat, and does group channels, direct messages, and file
or image sharing. Everything runs behind one origin (https://os.cyberskill.world in the examples), so
there is no cross-origin request and the dev CORS flags stay off.

The pieces are already built and verified locally. What this runbook does not do for you is the three
things only you can provide: a Google Cloud OAuth client, a server, and a domain. Each is called out
below.

## What is in the box

The product surface is the static console at apps/console (app.html is the login and dashboard,
chat.html is the chat client). It talks to two Rust services: cyberos-auth (password sign-in, Google
OIDC, and a tenant people directory) and cyberos-chat (channels, direct messages, messages, and
attachments, with a live websocket). Caddy terminates TLS and routes one origin to all three:

- `/` serves the console.
- `/v1/auth/*`, `/v1/admin/*`, and `/.well-known/*` go to cyberos-auth.
- `/v1/chat/*` (including the `/v1/chat/ws` upgrade) goes to cyberos-chat.

Deploy files: deploy/vps/Caddyfile.p0, deploy/vps/docker-compose.p0.yml, deploy/vps/.env.p0.example.

Three behaviours were added for this path and are worth knowing:

- Google sign-in is restricted to your Google Workspace domain. The IdP config carries an
  allowed_domains list; the callback rejects any login whose verified email is outside it, so a personal
  Gmail cannot get in.
- Google and a password resolve to one CyberOS account. On a Google sign-in the callback first looks for
  an existing subject with that verified email and links Google to it; only if none exists does it create
  a new account. So a teammate who already has a password account simply gets Google attached, and either
  way in lands on the same account. CyberOS stays the single source of truth for identity.
- After Google, the auth callback returns the browser to the console with the token in the URL fragment.
  The return target is allow-listed by AUTH_OIDC_RETURN_ALLOW so the token cannot be redirected to another
  origin.

## Step 1 - DNS and the domain (your action)

Point a host record (for example os.cyberskill.world) at the server's public IP. Caddy provisions TLS
automatically once the name resolves and ports 80 and 443 are open.

## Step 2 - create the Google OAuth client (your action)

In the Google Cloud Console, under APIs and Services:

1. OAuth consent screen: set it to Internal so only your Workspace can use it.
2. Credentials -> Create credentials -> OAuth client ID -> Web application.
3. Authorized redirect URI, exactly: `https://os.cyberskill.world/v1/auth/oidc/callback`
4. Copy the Client ID and Client secret. The secret goes only into the IdP config row in Step 4, never
   into a committed file.

Scopes stay the default openid, email, profile.

## Step 3 - create the Supabase database and apply migrations

CyberOS uses Supabase for managed Postgres, so there is no database on the VPS - auth and chat connect to
Supabase. They share one Supabase project: their tables do not collide and neither service auto-migrates,
so a single database holds both. (Split into two projects later only if you want hard isolation.)

1. Create a Supabase project in the Singapore region (close to the VPS keeps query latency low) and set a
   strong database password.
2. In Project Settings -> Database -> Connection string, copy the "Session pooler" string (it is IPv4 and
   session-mode on port 5432). Put it in both AUTH_DATABASE_URL and CHAT_DATABASE_URL in .env.p0, with
   `?sslmode=require` on the end. Do not use the transaction pooler (port 6543) - it drops the
   session-scoped role GUC the auth middleware sets.
3. Apply the migrations to the Supabase database, in order, from a host that has psql and the repo:

```bash
DB_URL='postgresql://postgres.PROJECT-REF:PASSWORD@aws-0-REGION.pooler.supabase.com:5432/postgres?sslmode=require'
for f in services/auth/migrations/*.sql services/chat/migrations/*.sql; do
  echo "applying $(basename "$f")"
  psql "$DB_URL" -v ON_ERROR_STOP=1 -f "$f"
done
```

The migrations enable two extensions (pg_trgm and unaccent) and create a few NOLOGIN helper roles; both
are permitted on Supabase under the default `postgres` role. If a `CREATE EXTENSION` line errors, enable
that extension from the dashboard (Database -> Extensions) and re-run - the migrations are idempotent, so
re-running is safe.

## Step 3b - bring up the stack

On the VPS, with Docker installed and the repo checked out:

```bash
cd deploy/vps
cp .env.p0.example .env.p0
# edit .env.p0: set CYBEROS_APP_DOMAIN, CYBEROS_APP_URL, and the Supabase session-pooler string in both
# AUTH_DATABASE_URL and CHAT_DATABASE_URL (same value for both).
docker compose --env-file .env.p0 -f docker-compose.p0.yml up -d --build
```

Confirm health from the server: `curl -fsS https://os.cyberskill.world/.well-known/jwks.json` returns a
key set, and the console loads at `https://os.cyberskill.world/`.

## Step 4 - create the team tenant and register Google

Bootstrap tenant 0 and a root admin, then create your team tenant (slug cyberskill) and register the
Google IdP under it. The bootstrap and admin-token mechanics are in docs/deploy/auth-google-sso-runbook.md;
the one P0 difference is that the IdP config must carry allowed_domains so only your Workspace can sign
in. With an admin bearer token for the cyberskill tenant:

```bash
curl -fsS -X POST https://os.cyberskill.world/v1/admin/oidc/idp-configs \
  -H "authorization: Bearer <ADMIN_TOKEN>" \
  -H "content-type: application/json" \
  -d '{
        "name": "google",
        "discovery_url": "https://accounts.google.com/.well-known/openid-configuration",
        "client_id": "<GOOGLE_CLIENT_ID>",
        "client_secret": "<GOOGLE_CLIENT_SECRET>",
        "redirect_uri": "https://os.cyberskill.world/v1/auth/oidc/callback",
        "allowed_domains": ["cyberskill.world"],
        "auto_provision": true,
        "default_roles": ["tenant-member"]
      }'
```

The console Sign in with Google button calls
`/v1/auth/oidc/initiate?tenant_slug=cyberskill&idp=google`, so the tenant slug must be cyberskill and the
IdP name must be google. AUTH_OIDC_RETURN_ALLOW is already set to the app URL in the compose file, so the
post-login hand-back is permitted.

## Step 5 - sign in and use it (the test)

1. Open `https://os.cyberskill.world/` and click Sign in with Google. Sign in with a @cyberskill.world
   account. You land on the dashboard; the first sign-in provisions your CyberOS account.
2. Open the Chat tile. Click New group, name it, pick teammates, and send a message - that is group chat.
3. Click New DM, pick one teammate - that is a direct message; sending again later reuses the same thread.
4. Use the paperclip to attach an image or a file; it renders inline or as a download chip.

A teammate signing in with a personal Gmail is rejected at the callback (email_domain_not_allowed), which
is the Workspace restriction working.

## Step 6 - before you hand it to the team

- Remove the local demo account. The dev bring-up seeds @stephen / a dev password for local testing; do
  not carry it into production. Revoke or delete that subject (revoke endpoint in the Google runbook).
- Keep AUTH_DEV_CORS and CHAT_DEV_CORS unset - the single origin needs no CORS, and the compose file
  leaves them off.
- Offboarding: remove the person from Google Workspace to block new logins, and call the admin revoke
  endpoint to kill any live session now (both steps are in docs/deploy/auth-google-sso-runbook.md).

## Known follow-ups, not blockers for P0

- The OIDC pending state (state token to PKCE verifier) is an in-memory map, so this assumes a single auth
  instance. For more than one, move it to Redis or a short-TTL table; oidc_login_history already records
  the state token and verifier hash.
- The chat hash-chained audit log writes to the memory database via CHAT_AUDIT_DATABASE_URL (the
  chat->brain link, DEC-2713). See "Company-brain capture (TASK-MEMORY-122)" below for how to turn it on and
  verify it; it stays off by default and does not affect the P0 team test.
- Real-time voice or video (WebRTC media and TURN), mobile push (APNS or FCM), and the desktop app are
  later phases; the chat signalling relay and device registration that back them already exist server-side.

## Company-brain capture (TASK-MEMORY-122)

This is where employee platform work-interactions (sign-ins, presence, chat activity) start flowing into
the company brain. It is OFF by default and has two independent switches; BOTH must be on for any data to be
recorded, and even then only for a subject who has acknowledged the monitoring notice.

1. The master switch: `CAPTURE_ENABLED`. Default `false`. When false or unset, AUTH and CHAT emit no
   interaction-events at all and behave exactly as before. Deploying this build with `CAPTURE_ENABLED`
   unset changes nothing about sign-in or chat, so it is safe to ship during a live team test. Set it to
   `true` only when you intend to begin recording.
2. The chat->brain link: `CHAT_AUDIT_DATABASE_URL` (DEC-2713). Point it at the memory module's
   `l1_audit_log` database (the same Supabase database in P0). When it is set, chat's audit rows and (when
   capture is on) its interaction-events chain into MEMORY. When it is empty, chat logs events instead of
   chaining them. AUTH writes to its own database, which is the same brain database, so no separate variable
   is needed for AUTH.

The consent prerequisite. Capture routes every event through the TASK-MEMORY-121 emit path, which is gated on
the TASK-EVAL-001 acknowledgment ledger. A subject is recorded only after an acknowledgment of the tenant's
current published monitoring notice is on file (normally the signed employment-document clause, recorded by
HR). A subject with no acknowledgment produces zero rows, by design - that is why a given person may show no
capture even when capture is on.

Verify capture is live. With `CAPTURE_ENABLED=true` and the link set, run this against the brain database:

```sql
SELECT count(*) FROM l1_audit_log
 WHERE event_type = 'memory.interaction_event'
   AND body::jsonb -> 'payload' ->> 'module' = 'chat';
```

A non-zero count means chat interaction-events are chaining into the brain. (The row-level `event_type`
column carries the row kind `memory.interaction_event`; the interaction's own verb, like
`chat.message_created`, is in `body.payload.event_type`.) To see a specific person's capture, add
`AND subject_id = '<subject-uuid>'`; an empty result for someone who has not acknowledged the notice is
expected, not a bug.

To turn capture off again, set `CAPTURE_ENABLED=false` (or remove it) and redeploy; emitters immediately go
back to no-ops.
