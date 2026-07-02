---
id: FR-CHAT-013
title: "CHAT native OIDC SSO - Mattermost federates to the FR-AUTH-110 CyberOS OIDC provider via its built-in OIDC/OAuth connector (replaces the closed FR-CHAT-002 AuthBridge plugin)"
module: CHAT
priority: MUST
status: superseded
superseded_by: FR-CHAT-101 (first-party native chat replaced the Mattermost fork wholesale; still-wanted intents re-homed as FR-CHAT-102..106)
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-06-29
memory_chain_hash: null
supersedes: [FR-CHAT-002]
related_frs: [FR-CHAT-001, FR-CHAT-002, FR-CHAT-003, FR-AUTH-110, FR-AUTH-005, FR-AUTH-101, FR-PORTAL-004]
depends_on: [FR-CHAT-001, FR-AUTH-110, FR-CHAT-003]
blocks: []

source_pages:
  - website/docs/modules/chat.html#auth
source_decisions:
  - DEC-2500 (Mattermost federates to the FR-AUTH-110 CyberOS OIDC provider through Mattermost's OWN native connector - not a plugin. FR-CHAT-002's AuthBridge approach is closed: a Mattermost plugin cannot replace the core /api/v4/users/login route, and the shipped plugin is a non-working simulation)
  - DEC-2501 (open-source build: use Mattermost's GitLab-OAuth connector repurposed by overriding its endpoint URLs to the provider's authorize/token/userinfo - the standard free-edition method for an arbitrary OIDC provider; the Enterprise OpenID connector is the alternative on E20+)
  - DEC-2502 (Mattermost JIT-provisions a REAL Mattermost user from the OIDC claims - email + preferred_username from the id_token / userinfo - via the native sign-in flow, producing a real Mattermost user row + a real Mattermost session, unlike the simulation)
  - DEC-2503 (the kick: a CyberOS revoke (FR-AUTH-005) makes the provider's authorize refuse - re-login is blocked into chat and every first-party app; an already-open Mattermost session dies via FR-AUTH-110 back-channel logout (its slice 2) or the Mattermost session TTL; instant cross-app deprovision is SCIM = FR-PORTAL-004)
  - DEC-2504 (HTTPS required end to end - OIDC redirects + the provider's __Host- SSO cookie need TLS; local dev uses a TLS tunnel or a one-line local drop of the cookie Secure flag)
  - DEC-2505 (Mattermost team <-> CyberOS tenant: the id_token tenant_id claim drives team membership; a single-tenant deploy maps to one team; multi-tenant team/group sync is slice 2)
  - DEC-2506 (drop the FR-CHAT-002 build patches 010-disable-builtin-auth + 011-load-authbridge-plugin; builtin password sign-up is turned off via Mattermost's own config (EnableSignUpWithEmail=false, the SSO-only settings), not a patch or plugin)
  - DEC-2507 (roles: the id_token roles claim (FR-AUTH-101) is available to map CyberOS roles to Mattermost system/team roles where the connector supports it; the mapping is operator-configured, not hard-coded)
  - DEC-2508 (the cyberos-chat RP client_secret is the one-time reveal from the FR-AUTH-110 registry, stored only in Mattermost config/env, never committed; the redirect_uri is registered byte-exact)
  - DEC-2509 (verification is an owner-run live browser sign-in, the same way the Google flow is owner-verified; the connector configuration is checked into deploy config, not a unit test)
  - OIDC Core 1.0; OIDC Discovery 1.0; RFC 6749 (OAuth 2.0); Mattermost OAuth 2.0 / OpenID SSO configuration
  - PDPL Art. 13 (data minimisation - chat receives only the id_token claims it needs: sub, email, name)

language: config + deploy (no new Go in the fork)
service: cyberos/services/chat/
new_files:
  - services/chat/deploy/oidc-sso-config.md                 # the Mattermost connector configuration + runbook
  - services/chat/deploy/mattermost-oidc.config.json        # the SSO-relevant config fragment (no secret committed)
modified_files:
  - services/chat/patches/010-disable-builtin-auth.patch    # REMOVE (builtin auth disabled via Mattermost config, not a patch)
  - services/chat/patches/011-load-authbridge-plugin.patch  # REMOVE (no plugin)

allowed_tools:
  - file_read: services/chat/**
  - file_read: services/auth/src/op/**
  - file_write: services/chat/deploy/**

disallowed_tools:
  - reintroduce the AuthBridge plugin or any plugin-based login interception (DEC-2500)
  - commit the client_secret to git (DEC-2508)
  - run OIDC over plaintext HTTP in production (DEC-2504)

effort_hours: 6
sub_tasks:
  - "1.0h: register the cyberos-chat RP in the FR-AUTH-110 provider; capture the one-time secret"
  - "1.5h: configure Mattermost's OAuth/OpenID connector with the provider endpoints + client + redirect_uri"
  - "0.8h: turn off builtin password sign-up via config; remove patches 010/011"
  - "0.7h: team/tenant + role mapping for the single-tenant case"
  - "1.0h: owner-run live browser sign-in proof (login + JIT user + logout)"
  - "1.0h: the deploy config doc + runbook"

risk_if_skipped: "Without this, chat cannot federate to the one CyberOS identity FR-AUTH-110 provides - it falls back to a separate Mattermost password store (two sources of truth, no kick-by-revoke) or stays on the closed FR-CHAT-002 AuthBridge plugin, which does not work (a Mattermost plugin cannot replace the login route; the shipped one is a simulation). The whole unified-identity story for the team's chat depends on this wiring. The effort is small because the heavy lifting is the provider (FR-AUTH-110, built + green); this FR is configuration plus an owner-run live sign-in."

---

## §1 - Description (BCP-14 normative)

CHAT (the Mattermost fork, FR-CHAT-001) **MUST** authenticate users by federating to the FR-AUTH-110 CyberOS OIDC provider through Mattermost's own native OIDC/OAuth connector, replacing the closed FR-CHAT-002 AuthBridge plugin. Each requirement:

1. **MUST** use Mattermost's native connector, not a plugin (DEC-2500). FR-CHAT-002 is closed because a Mattermost plugin cannot intercept the core `/api/v4/users/login` route (plugins serve only under `/plugins/<id>/`) and the shipped plugin is a non-working simulation. SSO in Mattermost is configured in the server (GitLab/OpenID/SAML connectors), never a plugin.

2. **MUST**, on the open-source build, repurpose the GitLab-OAuth connector by overriding its endpoint URLs to the provider's (DEC-2501): authorize → `<issuer>/v1/auth/op/authorize`, token → `<issuer>/v1/auth/op/token`, userinfo → `<issuer>/v1/auth/op/userinfo`. On Enterprise (E20+), the native OpenID Connect connector with the provider's discovery URL `<issuer>/.well-known/openid-configuration` is the alternative.

3. **MUST** let Mattermost JIT-provision a real Mattermost user from the OIDC claims on first sign-in (DEC-2502): `email` and `preferred_username` from the id_token / userinfo map to the Mattermost user's email + username. This is the native flow - it creates a real Mattermost user row and a real Mattermost session, which the AuthBridge simulation never did.

4. **MUST** register `cyberos-chat` as a relying party in the FR-AUTH-110 provider (`POST /v1/admin/op/rp-clients`), with the Mattermost sign-in-complete URL as the single registered redirect_uri, byte-exact (DEC-2508). The one-time client_secret is stored only in Mattermost config/env, never committed.

5. **MUST** disable builtin password sign-up via Mattermost's own configuration, not a patch (DEC-2506): set `EmailSettings.EnableSignUpWithEmail=false` (and the related password-login settings) so the only way in is the CyberOS connector. The FR-CHAT-002 patches `010-disable-builtin-auth` and `011-load-authbridge-plugin` are removed.

6. **MUST** support the kick (DEC-2503): a CyberOS revoke (FR-AUTH-005) makes the provider's `/authorize` refuse, so the person cannot sign into chat again. An already-open Mattermost session is killed by the FR-AUTH-110 back-channel-logout follow-up (its slice 2) or expires by the Mattermost session TTL; instant cross-app deprovision is SCIM (FR-PORTAL-004). The FR states this boundary plainly rather than claiming instant logout.

7. **MUST** run over HTTPS end to end in production (DEC-2504): OIDC redirects and the provider's `__Host-cyberos_sso` cookie require TLS. Local development uses a TLS tunnel, or a one-line local-only drop of the cookie `Secure` flag in the provider.

8. **MUST** map Mattermost team membership to the CyberOS tenant for the single-tenant case (DEC-2505): the id_token `tenant_id` claim identifies the tenant; a single-tenant deploy maps every user to one team. Multi-tenant team/group sync is slice 2.

9. **MUST** make the `roles` claim (FR-AUTH-101) available for operator-configured mapping to Mattermost system/team roles where the connector supports it (DEC-2507). No hard-coded role mapping.

10. **MUST** verify by an owner-run live browser sign-in (DEC-2509): a user with no session opens chat, is sent to the provider, brokered through Google (the FR-AUTH-110 broker), returned with a session, lands in chat as a real provisioned user; then a revoke blocks the next sign-in. The connector configuration is checked into deploy config; there is no unit test for a browser SSO handshake.

11. **MUST** carry only the needed claims into chat (PDPL Art. 13): `sub`, `email`, `name` / `preferred_username`, `tenant_id`, `roles`. No additional PII is requested.

---

## §2 - Why this design (rationale for humans)

Why the native connector, not the plugin. FR-CHAT-002 assumed a Mattermost plugin could replace login. It cannot: a plugin only receives requests under `/plugins/<id>/`, so the AuthBridge's attempt to intercept `/api/v4/users/login` never fires, and the shipped code is a simulation - no Mattermost SDK, in-memory fake users, a fabricated session string. Mattermost has a real, supported way to do SSO: its server-side OAuth/OpenID connectors. We use that.

Why the GitLab connector on the free build. The open-source Mattermost edition gates the Office365/Google/generic-OpenID connectors to Enterprise, but the GitLab connector is in the free edition and lets you override its three endpoint URLs. Pointing those at our provider is the standard, well-trodden way to do free-edition SSO with an arbitrary OIDC provider. On Enterprise the native OpenID connector is cleaner, so we support both.

Why JIT through the native flow. Mattermost's own SSO sign-in creates a genuine user and session and applies its own security (rate limits, session management). The simulation produced none of that. Letting Mattermost own user creation from the OIDC claims is both less code and correct.

Why the kick is honest. Revoking a CyberOS subject blocks the next sign-in immediately, because the provider's authorize refuses. It does not, by itself, kill a Mattermost session already open - Mattermost holds its own session after login. We are explicit: instant logout needs back-channel logout (the provider's slice 2) or SCIM (FR-PORTAL-004); the session TTL bounds the gap meanwhile.

Why HTTPS is non-negotiable. The provider's SSO cookie is the strict `__Host-` form, which a browser only stores over TLS, and OIDC itself should never run over plaintext. Production is HTTPS; local testing uses a tunnel or the curl path (which sidesteps the browser cookie).

Why drop the patches. With the native connector doing auth, the two FR-CHAT-002 patches that disabled builtin auth and loaded the dead plugin are unnecessary; builtin sign-up is turned off through Mattermost's own settings, which is the supported lever.

---

## §3 - Configuration contract

### 3.1 - Register the RP in the provider

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

(The redirect path is Mattermost's GitLab-complete URL on the open-source build; on Enterprise OpenID it is the OpenID-complete URL. Register exactly the one Mattermost uses.)

### 3.2 - Mattermost connector (open-source, GitLab connector repurposed)

`config.json` fragment (the secret comes from env, never committed):

```json
{
  "GitLabSettings": {
    "Enable": true,
    "Id": "cyberos-chat",
    "Secret": "${CYBEROS_CHAT_OIDC_SECRET}",
    "Scope": "openid email profile",
    "AuthEndpoint":   "https://auth.cyberos.world/v1/auth/op/authorize",
    "TokenEndpoint":  "https://auth.cyberos.world/v1/auth/op/token",
    "UserAPIEndpoint":"https://auth.cyberos.world/v1/auth/op/userinfo"
  },
  "EmailSettings": {
    "EnableSignUpWithEmail": false,
    "EnableSignInWithEmail": false,
    "EnableSignInWithUsername": false
  }
}
```

The provider's userinfo returns `sub`, `email`, `email_verified`, `name`, `preferred_username`, `tenant_id`, `roles`; Mattermost maps `id`←`sub`, `email`←`email`, `username`←`preferred_username`.

### 3.3 - Enterprise (OpenID connector) alternative

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

---

## §4 - Acceptance criteria

1. **Sign-in works end to end** - a fresh user opens chat, is sent to the provider, brokered through Google, returns, and lands in chat as a real provisioned Mattermost user.
2. **No builtin password login** - the email/password form is gone; the only path is the CyberOS connector.
3. **JIT user is real** - the Mattermost users table has a real row with the email + username from the claims (not an in-memory fake).
4. **redirect_uri exact** - a mismatched redirect is rejected by the provider (FR-AUTH-110), never redirected.
5. **Kick blocks re-login** - after a CyberOS revoke, the next chat sign-in fails at the provider's authorize (access_denied).
6. **Secret not in git** - the client_secret is only in env/config, never committed; grep of the repo finds no secret.
7. **HTTPS enforced** - production refuses the flow over plaintext; the cookie is set only over TLS.
8. **Patches removed** - `010-disable-builtin-auth` and `011-load-authbridge-plugin` are gone; the fork builds without them.

---

## §5 - Verification

Owner-run live sign-in (DEC-2509), mirroring the Google runbook:

1. Register the RP (§3.1), put the secret in the Mattermost env.
2. Apply the connector config (§3.2 or §3.3), restart Mattermost.
3. Open chat in a browser - confirm only the CyberOS sign-in button shows, click it, complete Google, land in chat.
4. Check the Mattermost System Console - the user exists with the right email/username.
5. Revoke the subject in AUTH, sign out, try to sign in again - it fails at the provider.

The provider side (authorize/token/userinfo) is already proven by `docs/deploy/auth-oidc-provider-roundtrip.md`. This FR's proof is that Mattermost's native connector consumes it.

---

## §6 - Implementation skeleton

The work is configuration (§3) plus removing two patches; there is no new Go in the fork. The provider (FR-AUTH-110) is the implementation; this FR wires Mattermost to it.

---

## §7 - Dependencies

Upstream:
- FR-AUTH-110 - the CyberOS OIDC provider Mattermost federates to (built + green).
- FR-CHAT-001 - the Mattermost fork being configured.
- FR-CHAT-003 - per-tenant deployment (provides the HTTPS endpoint OIDC needs).
- FR-AUTH-005 - revoke (the kick the provider enforces).
- FR-AUTH-101 - roles claim for optional role mapping.

Supersedes:
- FR-CHAT-002 - the AuthBridge plugin (closed; non-working simulation).

Downstream / related:
- FR-PORTAL-004 - SCIM auto-deprovision for instant cross-app removal.
- FR-AUTH-110 slice 2 - back-channel logout, for instant session kill in chat.

---

## §8 - Example payloads

### 8.1 - the id_token Mattermost reads (from the provider)

```json
{
  "iss": "https://auth.cyberos.world",
  "sub": "cf0f35f7-7770-4598-a656-50493e635351",
  "aud": "cyberos-chat",
  "email": "[email protected]",
  "email_verified": true,
  "name": "thai-anh.trinh",
  "preferred_username": "thai-anh.trinh",
  "tenant_id": "00000000-0000-0000-0000-000000000000",
  "roles": ["tenant-admin"]
}
```

---

## §9 - Open questions

Deferred (named slices, not gaps):
- **Instant session kill in chat** - FR-AUTH-110 slice 2 (back-channel logout) + FR-PORTAL-004 (SCIM). Until then, revoke blocks re-login and the session TTL bounds the window.
- **Multi-tenant team/group sync** - slice 2; slice 1 is single-tenant one-team.
- **Role-to-Mattermost-role mapping** - operator-configured where the connector supports it; a richer sync is a follow-up.
- **Local browser testing over HTTP** - needs a TLS tunnel or a local-only Secure-flag drop; the curl round-trip is the HTTP-friendly proof.

---

## §10 - Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Plugin reintroduced | review / disallowed_tools | rejected | Use the native connector |
| redirect_uri mismatch | provider exact-match | provider error, no redirect | Register the exact Mattermost URL |
| Secret committed | repo grep / CI | blocked | Rotate + move to env |
| OIDC over HTTP (prod) | TLS check | refused | Terminate TLS at the proxy |
| Revoked user re-login | provider authorize gate | access_denied | Designed (the kick) |
| Open session after revoke | (slice 2) | not killed instantly | Back-channel logout / TTL |
| Builtin auth still on | config check | password form shows | Set EnableSignUpWithEmail=false |
| userinfo claim missing | Mattermost JIT | sign-in fails | Provider returns sub+email+username |
| Free build lacks OpenID connector | edition check | use GitLab connector | DEC-2501 |

---

## §11 - Implementation notes

- Native Mattermost connector, never a plugin (FR-CHAT-002 closed).
- Open-source: GitLab connector with overridden endpoint URLs; Enterprise: OpenID connector with the discovery URL.
- JIT provisions a real Mattermost user from sub/email/preferred_username.
- Kick = revoke blocks re-login now; instant session kill is back-channel logout (slice 2) / SCIM (FR-PORTAL-004).
- HTTPS end to end; the __Host- cookie needs TLS.
- Single-tenant one-team for slice 1; multi-tenant sync later.
- Secret one-time from the FR-AUTH-110 registry, env-only, never committed.
- Remove patches 010/011; disable builtin sign-up via Mattermost config.
- Proof is an owner-run live browser sign-in, the same as the Google flow.

---

*End of FR-CHAT-013.*
