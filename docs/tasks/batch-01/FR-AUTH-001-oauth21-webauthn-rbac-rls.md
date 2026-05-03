---
title: "AUTH module — OAuth 2.1 with passkey enrolment, RBAC, and Postgres RLS binding"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P0 / 2026-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the AUTH module that every other CyberOS module trusts for identity. AUTH owns user sign-in via OAuth 2.1 with WebAuthn (passkey) as the primary factor and TOTP as backup, MFA enforcement on roles that can sign or transfer, JWT (RS256) session tokens with rotating signing keys, opaque refresh tokens stored in HttpOnly cookies, the canonical role catalogue (Founder, Engineering Lead, HR/Ops Lead, Account Manager, Member, Auditor, DPO), the RBAC predicate engine, and the contract that binds the application's `app.tenant_id` setting to Postgres RLS on every transaction. Agents authenticate as Members through the same OAuth flow with no service-account back-door (PRD §8.6 last bullet — agent parity invariant).

## Problem

Every module's data integrity depends on AUTH being right on day one. Today CyberSkill has no internal SSO; identity is fragmented across Google Workspace, Slack, Asana, and HubSpot, with no way to prove that an agent operating "as" the founder actually was the founder. The PRD locks three properties that AUTH must satisfy or the platform's compliance posture collapses:

- **Tenant isolation enforced at the database, not the application.** A bug in any module cannot leak across tenants because RLS is the floor, not application code (PRD §8.6, §8.8). AUTH is the only component that can authoritatively set `app.tenant_id` for a request.
- **Agent parity.** AI agents see exactly what their human Member sees and operate under the same RBAC predicates (PRD §8.6 last bullet). There is no "service account with broad permissions" pattern; there is no impersonation path; the audit log distinguishes agent-initiated from human-initiated calls but the *authorisation* is identical.
- **MFA on irreversible operations.** Roles that can sign, transfer, or move money (Founder, Account Manager, HR/Ops Lead) require a second factor on every login (PRD §8.6 paragraph 3). The CUO never auto-acts on irreversible operations regardless of the human's MFA state (PRD §6.4 defer-to-human triggers); MFA is the precondition for the *human* to act, not for the agent.

S0-2 is the sprint that ships AUTH (PRD §17.2). The risk gate is explicit: "Audit chain integrity is non-negotiable." AUTH writes the first audit-log rows on which every later module's audit chain depends.

## Proposed Solution

The shape of the solution is a single AUTH subgraph + AUTH MCP server + frontend remote, owned by the Engineering Lead, deployed alongside the platform infrastructure delivered by FR-INFRA-001.

**Identity provider stack.** AUTH is its own identity provider. We do not delegate to Google Workspace, Auth0, Okta, or Clerk in P0 — the founder requires the identity primary to live inside the platform so that the same identity flow works for human Members today and external tenants in P3+. Optional federated sign-in (Google Workspace, GitHub, Microsoft 365) is added in P1 as a *bridge* — the federated identity is mapped to a CyberOS Member at first sign-in but the Member record is the authority going forward.

**OAuth 2.1 with PKCE.** All client flows (host shell, MCP gateway, native MCP clients like Claude.ai or Cursor) use OAuth 2.1 Authorization Code with PKCE. No implicit flow, no resource-owner password grant. Authorisation server URL: `https://auth.cyberos.world/oauth2/v1/authorize` (canonical tenant) or `https://{tenant-slug}.cyberos.world/oauth2/v1/authorize` (per-tenant tenants from P3). PKCE code challenge S256 is mandatory; `plain` is rejected. Redirect URIs are pre-registered per client; arbitrary redirect URIs are rejected at registration time and at runtime.

**Audience-bound tokens.** Every issued access token carries `aud` set to the resource server URI (e.g. `https://api.cyberos.world/graphql` for the supergraph or `https://mcp.cyberos.world/v1` for the MCP gateway). The Apollo Router and the MCP gateway reject tokens whose `aud` does not match — this defeats the X-Forwarded-Authorization "shadow handoff" flagged by the security community in 2025 (PRD §8.4.1) and prevents an Acme tenant's token from being replayed at the Beta tenant's gateway.

**Sessions.** Access tokens are JWT (RS256) with a 60-minute lifetime, signed by a key in the rotating-key pool (90-day rotation, three-key sliding window: prior, current, next). The signing-key rotation is automated; the JWKS endpoint at `https://auth.cyberos.world/.well-known/jwks.json` lists all three keys with the `kid` header. Refresh tokens are opaque (server-side state, 30-day max lifetime, sliding-window rotation, single-use revoke-on-refresh). Refresh tokens live exclusively in HttpOnly cookies with `Secure; SameSite=Lax; Path=/oauth2/v1/refresh`. The frontend never sees the refresh token; access tokens are held in memory only and re-acquired via the refresh endpoint on tab focus.

**WebAuthn passkey enrolment.** The first-factor login is passkey: Apple Passkeys, Windows Hello, Yubikey, Android passkey. The enrolment flow uses the WebAuthn `create()` ceremony with `attestation: "direct"`, `userVerification: "required"`, and the relying party ID set to the parent domain `cyberos.world` so passkeys roam across `app.cyberos.world` and `auth.cyberos.world`. The credential is stored server-side as a row in `cyberos_meta.webauthn_credentials` with the credential ID, public key, sign counter, transports, and attestation statement. A Member can register multiple credentials; the Founder/CEO and any role with `mfa_required: true` must register at least two before completing onboarding.

**TOTP backup.** TOTP is the backup factor (RFC 6238, SHA-1, 6 digits, 30-second step). Enrolment writes the secret, encrypted at rest with the AUTH module's column key (KMS-wrapped), and emits a QR code for the authenticator app. Backup codes (10 codes, single-use, hashed at rest with `argon2id`) are issued at enrolment and shown once. A Member who loses both factors recovers via a founder-initiated reset flow that triggers an in-person verification step recorded in the audit log.

**Role catalogue.** PRD §8.6.1 names the role catalogue; AUTH seeds the canonical roles at first run:

| Role | `mfa_required` | Default RBAC predicates (predicate language defined below) |
|---|---|---|
| Founder/CEO | true | `*` (all predicates; explicit allowlist not deny-list) |
| Engineering Lead | true | `*.read`, `infra.*`, `obs.*`, `auth.read`, `brain.*`, `genie.persona.read`, `chat.*`, `proj.*`, `mcp.tool.register` |
| HR/Ops Lead | true | `hr.*`, `rew.*`, `learn.*`, `time.read`, `kb.write`, `auth.invite_member`, `inv.read` |
| Account Manager | true | `crm.*`, `proj.read`, `time.read`, `inv.write`, `chat.*`, `email.*` |
| Member | false | `proj.{own_or_assigned}.*`, `time.{self}.*`, `chat.*`, `kb.read`, `kb.write_draft`, `crm.read_assigned`, `genie.notify.read`, `mcp.tool.invoke` |
| Auditor | false | `audit.*.read`, `obs.*.read`, `cp.*.read`, `brain.*.read` (read-only across the platform; cannot write) |
| DPO | true | `cp.*`, `audit.*.read`, `dsar.*`, `rtbe.*`, `dpia.*`; can initiate tenant-deletion |

The role catalogue is *not* hardcoded; it is a seed migration that initialises a `cyberos_meta.role` table. New roles can be added at P3+ for tenant-customisation through a parameter-version migration; the audit log records every role change.

**RBAC predicate language.** Predicates are dotted strings: `{module}.{resource}.{action}` with `*` wildcards. The Apollo Router invokes a small Rego policy bundle on every request; the bundle's input is `{ subject_id, role, tenant_id, predicate, resource_id }` and the output is `allow: bool`. Per-record ACLs override role defaults — for example, a private DM whitelists exactly the two parties even if a role would otherwise grant `chat.read`. Per-record ACLs are stored in the resource module's tables; the Rego bundle queries them via a small "ACL fetcher" subgraph.

**Postgres RLS binding.** On every GraphQL request, the Apollo Router calls a tiny `auth.bind` Postgres function as the first statement of the transaction, passing the resolved `subject_id` and `tenant_id`. The function executes:

```sql
CREATE OR REPLACE FUNCTION auth.bind(p_subject UUID, p_tenant UUID)
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
  PERFORM set_config('app.subject_id', p_subject::text, true);
  PERFORM set_config('app.tenant_id', p_tenant::text, true);
  PERFORM set_config('app.role',     (SELECT role FROM cyberos_meta.member WHERE id = p_subject)::text, true);
END;
$$;
```

The `true` third argument scopes the setting to the transaction (`SET LOCAL`). PgBouncer runs in `transaction` mode (delivered by FR-INFRA-001), so a connection cannot leak settings across tenants. RLS policies on tenant-scoped tables read `current_setting('app.tenant_id', true)::uuid` and reject any row where `tenant_id` does not match. The "true" second argument to `current_setting` returns NULL rather than erroring on missing setting — RLS treats NULL as a deny, closing the fail-open vector.

**Agent authentication.** Agents receive their own OAuth 2.1 client credentials per Member they act on behalf of. The flow is "Bring Your Own Key" — a Member launches an MCP client (Claude.ai, Cursor, Claude Desktop), authenticates via the same passkey ceremony, and the client receives an access token bound to the Member's identity. The token's `aud` is the MCP gateway URI; the `sub` claim matches the Member; the token includes a `mcp_client` claim naming the client. There is no shared service-account credential. Tool calls invoked by the agent show up in the audit log with `actor_kind: "agent"`, `actor_human_subject: <member-id>`, `actor_agent_client: <client-id>` — auditors can answer "what did the founder do today?" as the union of human and agent actions.

**Magic-link onboarding.** New Members enrol via a magic-link email sent from the AUTH module to the address pre-registered by HR/Ops Lead. The link expires in 30 minutes and is single-use. The first action after click is the WebAuthn enrolment ceremony plus TOTP backup; only after both succeed is the Member marked `active` and can sign in. The magic-link channel is exclusively for first-time enrolment and password-less recovery; it is not a sign-in factor.

**Frontend remote.** The AUTH frontend remote renders three surfaces in the host shell: `/auth/sign-in` (passkey-first, TOTP fallback), `/auth/enroll` (the WebAuthn + TOTP enrolment ceremony), and `/auth/account` (Member's own credentials, devices, recent activity). The remote ships at ≤ 50 KB initial JS bundle; the WebAuthn ceremony's binary blobs are loaded lazily.

**MCP tool surface.** AUTH exposes MCP tools (PRD §8.4.2 naming convention `cyberos.{module}.{verb}_{noun}`):

- `cyberos.auth.whoami` (read-only, returns the resolved subject + role + tenant for the calling token)
- `cyberos.auth.list_members` (read-only, scoped to tenant)
- `cyberos.auth.invite_member` (HR/Ops or Founder only; emits a magic-link)
- `cyberos.auth.revoke_session` (Founder, Engineering Lead, or self)

`destructive: false` on the read tools; `destructive: true; requires_confirmation: true` on `invite_member` and `revoke_session` so the MCP gateway forces the human-in-the-loop confirmation.

## Alternatives Considered

- **Auth0 / Clerk / WorkOS.** Rejected: a hosted IdP makes per-tenant authorisation servers (PRD §8.4.1) and the Vietnamese-residency story (PRD §8.8) hard or impossible. Auth0's per-MAU pricing also grows past the $4/active user/month budget at 50-tenant scale.
- **Keycloak.** Rejected: Keycloak is a reasonable OSS choice but its plugin model is heavy for our 10-engineer footprint, and the resource-owner-password-grant default is exactly the surface we want gone. We may revisit at P3+ if external-tenant scale demands it.
- **Password-first with optional WebAuthn.** Rejected: passwords are the single largest vector for support-ticket load and credential phishing in our scale class. Passkey-first removes a 2026-relevant attack class entirely.
- **Service-account credentials for AI agents.** Rejected: this is the explicit anti-pattern called out by PRD §8.6. The agent-parity invariant collapses if an agent has more (or fewer) permissions than the human it acts for.
- **Magic-link as an ongoing sign-in factor.** Rejected: email is too compromised a channel; we restrict it to first-enrolment.

## Success Metrics

- **Primary metric.** S0-2 demo passes: (1) Founder logs in with a passkey from a fresh device in ≤ 30 seconds end-to-end, (2) the Founder enrols a TOTP backup factor in ≤ 60 seconds, (3) a synthetic cross-tenant request authenticated as Tenant A while requesting Tenant B's data is denied at RLS with audit-log entry recorded, (4) the agent flow: an MCP client receives a token, calls `cyberos.auth.whoami`, and sees its own subject identity, (5) signing-key rotation script runs end-to-end with no service interruption (zero 401s during rotation in synthetic load).
- **Guardrail metric.** Audit chain integrity = 100% over the lifetime of P0. A single broken row in the audit hash chain (FR-AUTH-002) or a single AUTH-issued token rejected by an RLS policy with the wrong `tenant_id` is sev-0.

## Scope

**In-scope (S0-2).**
- AUTH subgraph with the OAuth 2.1 flow, WebAuthn enrolment, TOTP backup, magic-link first-enrolment, sessions (JWT + opaque refresh), audience-bound tokens, JWKS endpoint, key rotation.
- Role catalogue seeded; RBAC predicate engine running as a Rego bundle invoked by the Apollo Router.
- `auth.bind` Postgres function and RLS binding contract; integration tests prove RLS enforces tenant isolation.
- AUTH frontend remote: `/auth/sign-in`, `/auth/enroll`, `/auth/account`.
- AUTH MCP server with `whoami`, `list_members`, `invite_member`, `revoke_session`.
- Per-Member credential lifecycle UI: list devices, rename, revoke; revoke triggers session invalidation cluster-wide.
- Federation directives (`@key(fields: "id")` on `Member`) so other subgraphs can `@external` reference Members.
- The 10 CyberSkill employees enrolled (the on-platform seed).

**Out-of-scope (deferred).**
- Federated sign-in via Google Workspace / GitHub / Microsoft 365 (P1; bridge-only).
- SCIM provisioning for external tenants (P3).
- Step-up auth for irreversible operations beyond MFA on login (P1; FR-AUTH-003 covers per-action confirmation).
- SSO across multiple tenants for the same human identity (P4; PRD §8.8 forbids cross-tenant writes by default and a Member's identity is per-tenant in P0).
- DPoP / mTLS access tokens (P3; OAuth 2.1 access tokens are bearer in P0).

## Dependencies

- FR-INFRA-001 (federation gateway, host shell, multi-tenant Postgres, NATS) must be `shipped` before this FR can begin S0-2 work.
- KMS provider selected per SRS Decisions Log (DEC-061+) — Hetzner has no native KMS, so we use HashiCorp Vault self-hosted on the cluster for AUTH's column-encryption keys.
- WebAuthn-capable devices for every CyberSkill employee (Apple, Windows, Android, or Yubikey 5).
- Compliance: PRD §12.1.1 PDPL Decree 13/2023 — biometric data (the WebAuthn signature is biometric-derived) does not leave the device; only the public key is stored. The DPIA template captures this for the A05 filing.
- Locked decisions referenced: DEC-009 (OAuth 2.1 + PKCE for MCP), DEC-010 (audience-bound tokens), DEC-011 (per-tenant authorisation servers), DEC-014 (RBAC + RLS as floor), DEC-015 (agent parity), DEC-016 (passkey-first first-factor).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. AUTH ships zero AI-derived behaviour. The passkey ceremony, RBAC engine, and RLS binding are deterministic.
