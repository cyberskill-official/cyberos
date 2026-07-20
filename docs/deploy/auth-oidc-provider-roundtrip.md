# AUTH OIDC provider - local round-trip proof (TASK-AUTH-110)

Prove the unified-path provider end to end on your Mac: register a relying party (Mattermost stands in as `cyberos-chat`), seed a sign-in session, then watch `/authorize` hand back a code and `/token` turn it into a real id_token plus access_token, and `/userinfo` read the identity back. This is the owner-run live test, the same way the Google sign-in in `auth-google-sso-runbook.md` is owner-run.

The provider is the inverse of TASK-AUTH-104: there AUTH is Google's client; here AUTH is the provider that first-party apps federate to. Endpoints: `/.well-known/openid-configuration`, `/v1/auth/op/authorize`, `/v1/auth/op/token`, `/v1/auth/op/userinfo`, and the admin registry `/v1/admin/op/rp-clients`.

## Prerequisites

1. Postgres up and migrations applied, including the new `0027`-`0030`, via your usual `scripts/local_verify.sh`.
2. A signing key bootstrapped (so the id_token can be signed) - the same `cyberos-auth bootstrap` you run before any minting flow. This also creates the root tenant and a root-admin subject we reuse as the test user.
3. AUTH running on `:7700` exactly as in the Google runbook (Step 2 there):

   ```bash
   cd services
   export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
   AUTH_LISTEN_ADDR=0.0.0.0:7700 \
   AUTH_JWT_ISSUER=http://localhost:7700 \
   AUTH_CURSOR_SIGNING_SECRET="$(openssl rand -base64 48)" \
     cargo run -p cyberos-auth --bin cyberos-auth
   ```

Confirm discovery serves the pinned profile:

```bash
curl -s http://localhost:7700/.well-known/openid-configuration | python3 -m json.tool
# issuer, authorization_endpoint, token_endpoint, userinfo_endpoint, jwks_uri,
# response_types_supported ["code"], code_challenge_methods_supported ["S256"], RS256.
```

## Step 1 - register the relying party + seed a sign-in session (SQL)

The broker that mints an SSO session from a Google login is the slice-1b follow-up, so for this test we seed the session row directly (the honest stand-in). We store only the SHA-256 hash of the client secret, so compute it and insert it. Run from `services/`:

```bash
ROOT='00000000-0000-0000-0000-000000000000'
SECRET='test-secret-123'
SECRET_HASH=$(printf '%s' "$SECRET" | openssl dgst -sha256 -r | cut -d' ' -f1)
SSO_ID=$(uuidgen | tr 'A-Z' 'a-z')
echo "secret_hash=$SECRET_HASH  sso_session=$SSO_ID"

docker compose -f dev/docker-compose.yml exec -T postgres \
  psql -U cyberos -d cyberos -v ON_ERROR_STOP=1 <<SQL
SELECT set_config('app.current_tenant_id', '$ROOT', false);

-- the test user = the bootstrap root-admin subject in the root tenant
\gset
SELECT id AS subject_id FROM subjects
 WHERE tenant_id = '$ROOT' AND status = 'active'
 ORDER BY created_at LIMIT 1 \gset

-- register the RP (idempotent on client_id)
INSERT INTO auth_oidc_rp_clients
  (id, tenant_id, name, client_id, client_secret_hash, redirect_uris, created_by_subject_id)
VALUES
  (gen_random_uuid(), '$ROOT', 'CyberOS Chat (test)', 'cyberos-chat',
   '$SECRET_HASH', ARRAY['http://localhost:9999/callback'], :'subject_id')
ON CONFLICT (client_id) DO UPDATE SET
  client_secret_hash = EXCLUDED.client_secret_hash,
  redirect_uris      = EXCLUDED.redirect_uris,
  is_active          = true;

-- seed a silent-SSO session for that subject
INSERT INTO auth_sso_sessions (id, tenant_id, subject_id, absolute_expiry)
VALUES ('$SSO_ID', '$ROOT', :'subject_id', NOW() + INTERVAL '24 hours');

SELECT :'subject_id' AS seeded_subject;
SQL
```

Keep the printed `sso_session` value - it is the cookie for the next step.

## Step 2 - authorize, and watch a code come back

PKCE values use the RFC 7636 test vector (verifier and its S256 challenge):

```bash
SSO_ID=<paste the sso_session printed above>
VERIFIER='dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk'
CHALLENGE='E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM'

curl -s -o /dev/null -D - \
  "http://localhost:7700/v1/auth/op/authorize?client_id=cyberos-chat&redirect_uri=http%3A%2F%2Flocalhost%3A9999%2Fcallback&response_type=code&scope=openid&state=xyz&code_challenge=$CHALLENGE&code_challenge_method=S256" \
  -H "Cookie: __Host-cyberos_sso=$SSO_ID" | tr -d '\r'
# Expect: HTTP/1.1 303 See Other
#         location: http://localhost:9999/callback?code=<CODE>&state=xyz
```

If you see `error=login_required` the session id is wrong or expired; `error=access_denied` means the subject is revoked; an error page (not a redirect) means an unknown client or a redirect_uri mismatch - all the gates working. Copy the `code` value out of the location.

## Step 3 - exchange the code for tokens

```bash
CODE=<paste the code>
curl -s -X POST http://localhost:7700/v1/auth/op/token \
  -H 'content-type: application/x-www-form-urlencoded' \
  --data-urlencode 'grant_type=authorization_code' \
  --data-urlencode "code=$CODE" \
  --data-urlencode 'redirect_uri=http://localhost:9999/callback' \
  --data-urlencode 'code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk' \
  --data-urlencode 'client_id=cyberos-chat' \
  --data-urlencode 'client_secret=test-secret-123' | python3 -m json.tool
# Expect: { "access_token": "...", "id_token": "...", "token_type": "Bearer",
#           "expires_in": 3600, "scope": "openid" }
```

Replay the exact same curl: the second time returns `{"error":"invalid_grant"}` - single-use codes working. A wrong `code_verifier` or `client_secret` returns `invalid_grant` / `invalid_client`.

Decode the id_token payload to see the identity it carries:

```bash
ID_TOKEN=<paste the id_token>
echo "$ID_TOKEN" | cut -d. -f2 | tr '_-' '/+' | \
  awk '{ l=length($0)%4; if(l>0) for(i=0;i<4-l;i++) $0=$0"="; print }' | base64 -d | python3 -m json.tool
# iss = http://localhost:7700, sub = <subject>, aud = cyberos-chat,
# email, roles, tenant_id, exp = iat + 3600.
```

## Step 4 - read it back through userinfo

```bash
ACCESS=<paste the access_token>
curl -s http://localhost:7700/v1/auth/op/userinfo \
  -H "authorization: Bearer $ACCESS" | python3 -m json.tool
# { "sub": ..., "email": ..., "email_verified": ..., "tenant_id": ..., "roles": [...] }
```

## Step 5 - prove the kick

Revoke the subject, then re-run Step 2. The authorize redirect now carries `error=access_denied`, and userinfo with the old access token returns `401 revoked`. That is kick-by-revoke into the provider: a revoked person cannot get a new session in any first-party app, and the existing token is refused at userinfo. (Killing an already-open downstream app session in real time is the back-channel-logout follow-up.)

```bash
SUBJECT_ID=<the seeded_subject from Step 1>
curl -s -X POST "http://localhost:7700/v1/admin/subjects/$SUBJECT_ID/revoke" \
  -H "authorization: Bearer <ADMIN_TOKEN>" -H "content-type: application/json" \
  -H "idempotency-key: $(uuidgen)" -d '{}'
```

## What this proves, and what is still open

Green here means the unified-path provider works end to end against a real database: an RP federates to one CyberOS identity, PKCE and the single-use code and the revoke gate all hold, and the id_token is signed by the same TASK-AUTH-004 key the JWKS publishes.

Deferred, to close after this live run:

- Duplicate `client_id` on register currently returns 500 rather than 409 (a one-line error mapping).
- The seven `op_*` audit rows are built but not yet anchored into the l1 chain (the payload builders are done and tested; wiring them mirrors `memory_bridge::emit_subject_revoked`).
- The upstream-Google broker-and-resume that creates the SSO session from a real Google login, replacing the seeded session in Step 1.

## Wiring the real Mattermost

Once the curl round-trip is green, point Mattermost's native OIDC connector (the open-source build's GitLab/OpenID connector, configured with explicit endpoint URLs) at:

- discovery / issuer: `http://localhost:7700` (`/.well-known/openid-configuration`)
- authorize: `/v1/auth/op/authorize`, token: `/v1/auth/op/token`, userinfo: `/v1/auth/op/userinfo`
- client_id `cyberos-chat`, the client_secret from registration, redirect_uri the Mattermost `signin/oidc/complete` URL (registered exactly, byte for byte).

That is the moment chat starts trusting the one CyberOS identity.
