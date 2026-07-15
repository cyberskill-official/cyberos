# AUTH module — task index

_Generated 2026-05-17 - 16 tasks, 139 engineering-hours total (TASK-AUTH-110 OIDC provider added 2026-06-29)._

## tasks

| Task | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-AUTH-001](TASK-AUTH-001-tenant-create/spec.md) | MUST | 1 | 8 | Tenant create — root-admin in tenant 0 calls POST /v1/admin/tenants with idempotency + RLS provision |
| [TASK-AUTH-002](TASK-AUTH-002-subject-create/spec.md) | MUST | 1 | 6 | Subject create — POST /v1/admin/subjects with bcrypt + role allow-list + idempotency + RLS-enforced  |
| [TASK-AUTH-003](TASK-AUTH-003-rls-enforcement/spec.md) | MUST | 1 | 12 | RLS enforcement at every tenant-scoped table — USING + WITH CHECK + per-connection app.tenant_id + p |
| [TASK-AUTH-004](TASK-AUTH-004-jwt-jwks/spec.md) | MUST | 1 | 12 | JWT issuance + JWKS endpoint (RS256) with tenant_id + agent_persona + scope_grants + dual-rate-limit |
| [TASK-AUTH-005](TASK-AUTH-005-admin-rest/spec.md) | MUST | 1 | 8 | Admin REST: list tenants + list subjects + revoke subject + unrevoke + cursor pagination + jti deny- |
| [TASK-AUTH-006](TASK-AUTH-006-bootstrap-cli/spec.md) | MUST | 1 | 6 | cyberos-auth bootstrap CLI: tenant 0 + root-admin + initial signing key + sweepers + idempotency-tab |
| [TASK-AUTH-101](TASK-AUTH-101-rbac-catalogue/spec.md) | MUST | 1 | 12 | AUTH 22-role RBAC catalogue — closed enum + permission matrix + role-assignment REST + JWT claims +  |
| [TASK-AUTH-102](TASK-AUTH-102-totp-webauthn-mfa/spec.md) | MUST | 1 | 10 | AUTH TOTP (RFC 6238) + WebAuthn Level 3 MFA — closed factor enum + enrolment FSM + challenge/respons |
| [TASK-AUTH-103](TASK-AUTH-103-saml-sso/spec.md) | MUST | 1 | 12 | AUTH SAML 2.0 SSO — SP-initiated flow + per-tenant IdP config + XML signature verification + asserti |
| [TASK-AUTH-104](TASK-AUTH-104-oidc-sso/spec.md) | MUST | 1 | 10 | AUTH OIDC SSO — RFC 8414 discovery + RFC 7517 JWKS rotation + per-tenant IdP config + PKCE + JIT sub |
| [TASK-AUTH-105](TASK-AUTH-105-passkey-enrolment-login/spec.md) | MUST | 1 | 8 | AUTH Passkey enrolment + login — discoverable credentials (resident keys) + autofill UI + cross-plat |
| [TASK-AUTH-106](TASK-AUTH-106-impossible-travel/spec.md) | SHOULD | 1 | 8 | Impossible-travel detection + adaptive MFA challenge |
| [TASK-AUTH-107](TASK-AUTH-107-hibp-breach-check/spec.md) | SHOULD | 1 | 4 | HIBP password breach check (k-anonymity) on signup + rotation |
| [TASK-AUTH-108](TASK-AUTH-108-lumi-tenant-identity-jwt/spec.md) | MUST | 1 | 6 | AUTH Lumi tenant-identity JWT shape — agent_persona + tenant_residency + lumi_org_tenant claims + pe |
| [TASK-AUTH-109](TASK-AUTH-109-stub-to-full-migration/spec.md) | MUST | 1 | 5 | AUTH stub → full migration enforcer — 30-day grace window + cutover timestamp + rejection metric + p |
| [TASK-AUTH-110](TASK-AUTH-110-oidc-provider/spec.md) | MUST | 1 | 12 | AUTH OIDC Provider - first-party authorization server: CHAT/PORTAL federate to one CyberOS identity; authorize brokers via SSO cookie / Google + revoke-gated; token + id_token + userinfo; PKCE S256; JWKS reuse |

## Cross-module dependencies

**This module is depended on by:**

- **AI**: TASK-AI-006→TASK-AUTH-004
- **memory**: TASK-MEMORY-101→TASK-AUTH-003
- **CHAT**: TASK-CHAT-002→TASK-AUTH-004, TASK-CHAT-002→TASK-AUTH-110 (unified-path SSO via OIDC; supersedes the AuthBridge-plugin approach)
- **CRM**: TASK-CRM-001→TASK-AUTH-003, TASK-CRM-001→TASK-AUTH-101
- **DOC**: TASK-DOC-001→TASK-AUTH-101, TASK-DOC-006→TASK-AUTH-105
- **EMAIL**: TASK-EMAIL-002→TASK-AUTH-004
- **HR**: TASK-HR-001→TASK-AUTH-003, TASK-HR-001→TASK-AUTH-101
- **INV**: TASK-INV-003→TASK-AUTH-101, TASK-INV-004→TASK-AUTH-101, TASK-INV-005→TASK-AUTH-101
- **KB**: TASK-KB-001→TASK-AUTH-003, TASK-KB-001→TASK-AUTH-101
- **MCP**: TASK-MCP-001→TASK-AUTH-004, TASK-MCP-004→TASK-AUTH-004
- **OBS**: TASK-OBS-002→TASK-AUTH-004
- **OKR**: TASK-OKR-001→TASK-AUTH-003, TASK-OKR-001→TASK-AUTH-101
- **PORTAL**: TASK-PORTAL-003→TASK-AUTH-103, TASK-PORTAL-003→TASK-AUTH-104, TASK-PORTAL-003→TASK-AUTH-110
- **PROJ**: TASK-PROJ-001→TASK-AUTH-001, TASK-PROJ-001→TASK-AUTH-003
- **REW**: TASK-REW-001→TASK-AUTH-101
- **TEN**: TASK-TEN-001→TASK-AUTH-001, TASK-TEN-004→TASK-AUTH-003, TASK-TEN-101→TASK-AUTH-104
- **TIME**: TASK-TIME-001→TASK-AUTH-003, TASK-TIME-001→TASK-AUTH-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._