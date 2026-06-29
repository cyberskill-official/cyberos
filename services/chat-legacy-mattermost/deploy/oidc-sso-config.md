# CyberOS CHAT - OIDC SSO via the FR-AUTH-110 provider

This wires the Mattermost fork to the CyberOS OIDC provider (FR-AUTH-110) using
Mattermost's own native connector, replacing the closed FR-CHAT-002 AuthBridge
plugin. There is no plugin and no new Go: SSO is server configuration plus a
relying-party registration in the provider. See `FR-CHAT-013`.

The provider side (authorize, token, userinfo, the revoke kick) is already
proven end to end - see `docs/deploy/auth-oidc-provider-roundtrip.md`. What
remains is pointing Mattermost at it and a live browser sign-in.

## Endpoints

| Item | Production | Local dev |
|---|---|---|
| Provider issuer | `https://auth.cyberos.world` | `http://localhost:7700` |
| authorize | `<issuer>/v1/auth/op/authorize` | same path |
| token | `<issuer>/v1/auth/op/token` | same path |
| userinfo | `<issuer>/v1/auth/op/userinfo` | same path |
| Mattermost base | `https://chat.cyberos.world` | `http://localhost:8065` |
| Redirect URI (GitLab connector) | `<mm-base>/signup/gitlab/complete` | same path |

## Step 1 - register the RP in the provider

Register `cyberos-chat` with Mattermost's GitLab-complete URL as the single,
byte-exact redirect_uri. Capture the one-time secret; it is shown once.

```bash
curl -s -X POST https://auth.cyberos.world/v1/admin/op/rp-clients \
  -H "authorization: Bearer <ADMIN_TOKEN>" -H "content-type: application/json" \
  -d '{
    "name": "CyberOS Chat",
    "client_id": "cyberos-chat",
    "redirect_uris": ["https://chat.cyberos.world/signup/gitlab/complete"],
    "post_logout_redirect_uris": ["https://chat.cyberos.world/login"]
  }'
# -> { "client_id": "cyberos-chat", "client_secret": "<ONE-TIME>", ... }
```

Local dev: the same call against `http://localhost:7700`, with redirect_uri
`http://localhost:8065/signup/gitlab/complete`.

## Step 2 - supply the secret as an env override (never commit it)

Mattermost overrides any config value from an env var named `MM_<SECTION>_<KEY>`.
Put the secret there, not in the file:

```bash
export MM_GITLABSETTINGS_SECRET='<the one-time secret from Step 1>'
```

The committed fragment (`mattermost-oidc.config.json`) carries a
`${CYBEROS_CHAT_OIDC_SECRET}` placeholder so no secret is ever in git; the env
override above is what Mattermost actually uses at runtime.

## Step 3 - apply the connector config

Merge `mattermost-oidc.config.json` into the Mattermost `config.json` (or set
the equivalent values in the System Console). It does two things:

- enables the GitLab connector pointed at the provider's three op endpoints
  (open-source build, per DEC-2501);
- turns off builtin email and username sign-in and sign-up, so the only way in
  is the CyberOS connector.

For local dev, override the three endpoint URLs to the `http://localhost:7700`
forms (env: `MM_GITLABSETTINGS_AUTHENDPOINT`, `MM_GITLABSETTINGS_TOKENENDPOINT`,
`MM_GITLABSETTINGS_USERAPIENDPOINT`).

Enterprise (E20+) alternative: use the native OpenID connector instead of the
GitLab one, with just the discovery URL:

```json
{
  "OpenIdSettings": {
    "Enable": true,
    "DiscoveryEndpoint": "https://auth.cyberos.world/.well-known/openid-configuration",
    "Id": "cyberos-chat",
    "Secret": "${CYBEROS_CHAT_OIDC_SECRET}",
    "Scope": "openid email profile"
  }
}
```

## Step 4 - HTTPS (the cookie needs TLS)

The provider's SSO cookie is the strict `__Host-cyberos_sso` form, which a
browser stores only over HTTPS. Production terminates TLS at Caddy (the deploy
front), so this is satisfied. For a local browser test you need one of:

- a TLS tunnel in front of both Mattermost and the provider (for example
  cloudflared or ngrok), so both are reached over `https`; or
- a local-only build of the provider with the cookie `Secure` flag dropped
  (development convenience only; never in production).

The already-proven curl round-trip sidesteps the browser cookie, so the
provider itself is verifiable without a tunnel - only the browser flow needs
one.

## Step 5 - restart and sign in (the live proof, DEC-2509)

1. Restart Mattermost with the env override and the merged config.
2. Open the Mattermost URL in a browser. Confirm the email/password form is
   gone and only the CyberOS sign-in button shows.
3. Click it. You are sent to the provider, brokered through Google, and
   returned to chat as a real, JIT-provisioned Mattermost user.
4. In the System Console, confirm the user exists with the email and username
   from the id_token claims (`id` from `sub`, `email` from `email`, `username`
   from `preferred_username`).
5. Revoke the subject in AUTH, sign out, and try again - the next sign-in fails
   at the provider's authorize (access_denied). That is the kick.

## Mapping and scope (slice 1)

- Claims consumed: `sub`, `email`, `email_verified`, `name`,
  `preferred_username`, `tenant_id`, `roles` (PDPL data minimisation - nothing
  more is requested).
- Tenant to team: a single-tenant deploy maps every user to one team; the
  `tenant_id` claim identifies the tenant. Multi-tenant team and group sync is
  slice 2.
- Roles: the `roles` claim is available for operator-configured mapping to
  Mattermost system and team roles where the connector supports it; nothing is
  hard-coded.

## The kick, honestly scoped

A CyberOS revoke (FR-AUTH-005) blocks the next sign-in immediately, because the
provider's authorize refuses. It does not by itself end an already-open
Mattermost session - Mattermost holds its own session after login. Instant
session kill is the FR-AUTH-110 back-channel-logout follow-up (its slice 2) or
SCIM (FR-PORTAL-004); the Mattermost session TTL bounds the window until then.

## Notes

- Native connector only, never a plugin: FR-CHAT-002 is closed because a
  Mattermost plugin cannot replace `/api/v4/users/login`.
- The FR-CHAT-002 build patches `010-disable-builtin-auth` and
  `011-load-authbridge-plugin` are out of the active series (archived under
  `../patches/superseded/`); builtin sign-up is turned off via config.
- The secret is one-time from the provider registry, env-only, never committed.
