# PORTAL module — feature request index

_Generated 2026-05-17 — 8 FRs, 61 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-PORTAL-001](FR-PORTAL-001-scoped-views/spec.md) | MUST | 1 | 12 | PORTAL scoped read-only views — PROJ/INV/DOC/CHAT filtered by Engagement membership + sync_class=cli |
| [FR-PORTAL-002](FR-PORTAL-002-per-tenant-brand-pack/spec.md) | MUST | 1 | 8 | PORTAL per-tenant brand pack — logo + colour palette + custom CNAME + email template overrides + ACM |
| [FR-PORTAL-003](FR-PORTAL-003-external-idp-scim-jit/spec.md) | MUST | 1 | 10 | PORTAL external IdP — SAML 2.0 + OIDC sign-in for client-tenant users + SCIM 2.0 JIT provisioning +  |
| [FR-PORTAL-004](FR-PORTAL-004-scim-deprovision/spec.md) | MUST | 2 | 8 | PORTAL SCIM deprovision — session invalidation ≤ 30 s on IdP user removal + grace period + cascade r |
| [FR-PORTAL-005](FR-PORTAL-005-branded-genie-chat/spec.md) | SHOULD | 2 | 6 | PORTAL branded Genie chat — CUO scope-narrowed by JWT scope_grants + per-Engagement brand pack + IdP |
| [FR-PORTAL-006](FR-PORTAL-006-client-initiated-workflows/spec.md) | MUST | 2 | 6 | PORTAL client-initiated workflows — new project request / billing inquiry / support ticket → CHAT th |
| [FR-PORTAL-007](FR-PORTAL-007-pwa-installable/spec.md) | SHOULD | 2 | 6 | PORTAL PWA installable — mobile-first Progressive Web App with offline-capable view cache + push not |
| [FR-PORTAL-008](FR-PORTAL-008-dsar-self-service/spec.md) | MUST | 2 | 5 | PORTAL DSAR self-service — GDPR Art. 15 + PDPL Art. 17 client-initiated data subject access request  |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-PORTAL-003→FR-AUTH-103, FR-PORTAL-003→FR-AUTH-104
- **CHAT**: FR-PORTAL-006→FR-CHAT-005
- **CUO**: FR-PORTAL-005→FR-CUO-101
- **TEN**: FR-PORTAL-001→FR-TEN-101, FR-PORTAL-002→FR-TEN-101

**This module is depended on by:**

- **EMAIL**: FR-EMAIL-008→FR-PORTAL-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._