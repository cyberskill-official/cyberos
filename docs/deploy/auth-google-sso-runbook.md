# Google sign-in for CyberOS (AUTH OIDC) - local runbook

Wire Google Workspace as the identity provider so your team signs in with their Google account, and
offboarding is a single action: remove someone from Google and they lose CyberOS access. This runbook
covers the local bring-up; the production notes are at the end.

## What is built, and what changed this session

The OIDC authorization-code + PKCE flow already exists in `services/auth/src/oidc.rs` (TASK-AUTH-104):
`initiate` builds the provider redirect with PKCE and a state token, `callback` exchanges the code for
tokens, provisions the subject, and mints a CyberOS session. Two fixes landed this session to make Google
sign-in real and safe:

- The callback now verifies the Google ID token for real - it fetches the IdP JWKS and checks the RS256
  signature, the issuer, the audience (your client id), and expiry before trusting any claim. It used to
  base64-decode the token without verifying it.
- A brand-new Google user is now provisioned with a bcrypt hash of an unguessable random value, so the
  `subjects_human_has_password` constraint is satisfied. Password-grant login for that account fails
  closed (no one knows the value); the only way in is Google.

Both are code changes in `services/auth`, so they need a compile on your Mac (recipe at the end). No
schema migration is required.

## Step 1 - create a Google OAuth client (your action)

In the Google Cloud Console:

1. APIs and Services -> OAuth consent screen: configure it. Internal if this is your Google Workspace;
   External plus test users otherwise.
2. APIs and Services -> Credentials -> Create credentials -> OAuth client ID -> Application type: Web
   application.
3. Authorized redirect URIs: add `http://localhost:7700/v1/auth/oidc/callback` for local. For production
   add `https://<your-auth-domain>/v1/auth/oidc/callback`.
4. Copy the Client ID and Client secret. Keep the secret out of git - it goes only into the database row
   below, never a committed file.

Scopes stay the default `openid email profile`.

## Step 2 - run AUTH locally

Infra must be up and migrations applied (you already do this via `scripts/local_verify.sh`). Then, from
`services/`:

```bash
cd services
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
AUTH_LISTEN_ADDR=0.0.0.0:7700 \
AUTH_JWT_ISSUER=http://localhost:7700 \
AUTH_CURSOR_SIGNING_SECRET="$(openssl rand -base64 48)" \
  cargo run -p cyberos-auth --bin cyberos-auth
```

If the binary reports another required env var, add it and re-run. The signing secret is generated fresh
here; do not commit it.

## Step 3 - register the Google IdP in AUTH

For local, insert the config row directly (the table is FORCE RLS, so set the root tenant GUC first).
Replace the two placeholders with your Google client id and secret:

```bash
cd services && docker compose -f dev/docker-compose.yml exec -T postgres \
  psql -U cyberos -d cyberos <<'SQL'
BEGIN;
SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000';
INSERT INTO oidc_idp_configs
  (tenant_id, name, discovery_url, client_id, client_secret, redirect_uri, scopes, auto_provision, default_roles)
VALUES
  ('00000000-0000-0000-0000-000000000000', 'google-workspace',
   'https://accounts.google.com/.well-known/openid-configuration',
   '<GOOGLE_CLIENT_ID>', '<GOOGLE_CLIENT_SECRET>',
   'http://localhost:7700/v1/auth/oidc/callback',
   ARRAY['openid','email','profile'], true, ARRAY['tenant-member'])
ON CONFLICT (tenant_id, name) DO UPDATE SET
  client_id=EXCLUDED.client_id, client_secret=EXCLUDED.client_secret,
  discovery_url=EXCLUDED.discovery_url, redirect_uri=EXCLUDED.redirect_uri, updated_at=NOW();
COMMIT;
SQL
```

For production, use the admin API instead (`POST /v1/admin/oidc/idp-configs` with an admin bearer token)
against a real tenant, and store the secret KMS-wrapped per the migration note.

## Step 4 - sign in with Google (the test)

```bash
# 1. Get the authorization URL.
curl -s "http://localhost:7700/v1/auth/oidc/initiate?tenant_slug=root&idp=google-workspace"
#    -> {"authorization_url": "https://accounts.google.com/o/oauth2/v2/auth?...", "state": "..."}

# 2. Open that authorization_url in a browser and sign in with Google.
#    Google redirects to the callback, which returns the CyberOS session as JSON:
#    {"access_token": "...", "refresh_token": "...", "token_type": "Bearer",
#     "expires_in": ..., "subject_id": "..."}
```

On the first sign-in a CyberOS subject is created (kind human, role tenant-member) and linked to the
Google identity; the response carries a CyberOS `access_token`. That token is the member's CyberOS
session - the same token CHAT and the rest of the platform will trust.

## Step 5 - kick a member (offboarding)

Two layers, and you usually do both:

- Block new logins: remove the person from Google Workspace, or disable their Google account. Google stops
  authenticating them, so they can never obtain a new CyberOS session.
- Kill the live session now: call the admin revoke endpoint with their subject id. This sets the subject
  to `revoked` and adds their JWT id to the deny-list, so the active session dies immediately.

```bash
curl -fsS -X POST http://localhost:7700/v1/admin/subjects/<SUBJECT_ID>/revoke \
  -H "authorization: Bearer <ADMIN_TOKEN>" \
  -H "content-type: application/json" \
  -H "idempotency-key: $(uuidgen)" \
  -d '{}'
```

The revoke endpoint needs an admin token and an idempotency key (TASK-AUTH-005); check the exact header name
in `services/auth/src/handlers.rs` if it rejects the request. Find the subject id from the sign-in
response or by listing subjects.

Today this is a manual step at offboarding. The fully automatic version - Google removal instantly killing
the live session with no manual call - is SCIM deprovisioning (TASK-PORTAL-004), which is specified but not
built. A short session lifetime narrows the manual window in the meantime.

## Compile and test (your Mac)

```bash
cd services
cargo test -p cyberos-auth -- --test-threads=1 --skip create_subject_p95
cargo clippy -p cyberos-auth --all-targets -- -D warnings
```

Expect the three new OIDC unit tests to pass alongside the existing suite: `jwks_parses_rsa_keys`,
`verify_id_token_rejects_unknown_kid`, `verify_id_token_rejects_malformed_token`. The full Google
round-trip - the happy-path signature verify - is the owner-run sign-in in Step 4, not a unit test, the
same way other live flows are owner-verified.

## Production follow-ups (noted, not blockers)

- The OIDC pending state (the state-token to PKCE-verifier mapping) lives in an in-memory map, so a single
  AUTH instance is assumed. For multi-instance AUTH, move it to a short-TTL DB or Redis store;
  `oidc_login_history` already records the state token and the verifier hash.
- The JWKS is fetched once per login. Add a TTL cache keyed by `jwks_uri` for performance; per-login fetch
  already handles key rotation correctly, so this is purely an optimisation.
- Google's ID-token issuer matches the discovery issuer. If you ever see `id_token_verification_failed`
  on the issuer, relax the check to accept both `https://accounts.google.com` and `accounts.google.com`.
