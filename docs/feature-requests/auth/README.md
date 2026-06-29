# AUTH module — feature request index

_Generated 2026-05-17 - 16 FRs, 139 engineering-hours total (FR-AUTH-110 OIDC provider added 2026-06-29)._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-AUTH-001](FR-AUTH-001-tenant-create.md) | MUST | 1 | 8 | Tenant create — root-admin in tenant 0 calls POST /v1/admin/tenants with idempotency + RLS provision |
| [FR-AUTH-002](FR-AUTH-002-subject-create.md) | MUST | 1 | 6 | Subject create — POST /v1/admin/subjects with bcrypt + role allow-list + idempotency + RLS-enforced  |
| [FR-AUTH-003](FR-AUTH-003-rls-enforcement.md) | MUST | 1 | 12 | RLS enforcement at every tenant-scoped table — USING + WITH CHECK + per-connection app.tenant_id + p |
| [FR-AUTH-004](FR-AUTH-004-jwt-jwks.md) | MUST | 1 | 12 | JWT issuance + JWKS endpoint (RS256) with tenant_id + agent_persona + scope_grants + dual-rate-limit |
| [FR-AUTH-005](FR-AUTH-005-admin-rest.md) | MUST | 1 | 8 | Admin REST: list tenants + list subjects + revoke subject + unrevoke + cursor pagination + jti deny- |
| [FR-AUTH-006](FR-AUTH-006-bootstrap-cli.md) | MUST | 1 | 6 | cyberos-auth bootstrap CLI: tenant 0 + root-admin + initial signing key + sweepers + idempotency-tab |
| [FR-AUTH-101](FR-AUTH-101-rbac-catalogue.md) | MUST | 1 | 12 | AUTH 22-role RBAC catalogue — closed enum + permission matrix + role-assignment REST + JWT claims +  |
| [FR-AUTH-102](FR-AUTH-102-totp-webauthn-mfa.md) | MUST | 1 | 10 | AUTH TOTP (RFC 6238) + WebAuthn Level 3 MFA — closed factor enum + enrolment FSM + challenge/respons |
| [FR-AUTH-103](FR-AUTH-103-saml-sso.md) | MUST | 1 | 12 | AUTH SAML 2.0 SSO — SP-initiated flow + per-tenant IdP config + XML signature verification + asserti |
| [FR-AUTH-104](FR-AUTH-104-oidc-sso.md) | MUST | 1 | 10 | AUTH OIDC SSO — RFC 8414 discovery + RFC 7517 JWKS rotation + per-tenant IdP config + PKCE + JIT sub |
| [FR-AUTH-105](FR-AUTH-105-passkey-enrolment-login.md) | MUST | 1 | 8 | AUTH Passkey enrolment + login — discoverable credentials (resident keys) + autofill UI + cross-plat |
| [FR-AUTH-106](FR-AUTH-106-impossible-travel.md) | SHOULD | 1 | 8 | Impossible-travel detection + adaptive MFA challenge |
| [FR-AUTH-107](FR-AUTH-107-hibp-breach-check.md) | SHOULD | 1 | 4 | HIBP password breach check (k-anonymity) on signup + rotation |
| [FR-AUTH-108](FR-AUTH-108-lumi-tenant-identity-jwt.md) | MUST | 1 | 6 | AUTH Lumi tenant-identity JWT shape — agent_persona + tenant_residency + lumi_org_tenant claims + pe |
| [FR-AUTH-109](FR-AUTH-109-stub-to-full-migration.md) | MUST | 1 | 5 | AUTH stub → full migration enforcer — 30-day grace window + cutover timestamp + rejection metric + p |
| [FR-AUTH-110](FR-AUTH-110-oidc-provider.md) | MUST | 1 | 12 | AUTH OIDC Provider - first-party authorization server: CHAT/PORTAL federate to one CyberOS identity; authorize brokers via SSO cookie / Google + revoke-gated; token + id_token + userinfo; PKCE S256; JWKS reuse |

## Cross-module dependencies

**This module is depended on by:**

- **AI**: FR-AI-006→FR-AUTH-004
- **memory**: FR-MEMORY-101→FR-AUTH-003
- **CHAT**: FR-CHAT-002→FR-AUTH-004, FR-CHAT-002→FR-AUTH-110 (unified-path SSO via OIDC; supersedes the AuthBridge-plugin approach)
- **CRM**: FR-CRM-001→FR-AUTH-003, FR-CRM-001→FR-AUTH-101
- **DOC**: FR-DOC-001→FR-AUTH-101, FR-DOC-006→FR-AUTH-105
- **EMAIL**: FR-EMAIL-002→FR-AUTH-004
- **HR**: FR-HR-001→FR-AUTH-003, FR-HR-001→FR-AUTH-101
- **INV**: FR-INV-003→FR-AUTH-101, FR-INV-004→FR-AUTH-101, FR-INV-005→FR-AUTH-101
- **KB**: FR-KB-001→FR-AUTH-003, FR-KB-001→FR-AUTH-101
- **MCP**: FR-MCP-001→FR-AUTH-004, FR-MCP-004→FR-AUTH-004
- **OBS**: FR-OBS-002→FR-AUTH-004
- **OKR**: FR-OKR-001→FR-AUTH-003, FR-OKR-001→FR-AUTH-101
- **PORTAL**: FR-PORTAL-003→FR-AUTH-103, FR-PORTAL-003→FR-AUTH-104, FR-PORTAL-003→FR-AUTH-110
- **PROJ**: FR-PROJ-001→FR-AUTH-001, FR-PROJ-001→FR-AUTH-003
- **REW**: FR-REW-001→FR-AUTH-101
- **TEN**: FR-TEN-001→FR-AUTH-001, FR-TEN-004→FR-AUTH-003, FR-TEN-101→FR-AUTH-104
- **TIME**: FR-TIME-001→FR-AUTH-003, FR-TIME-001→FR-AUTH-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._