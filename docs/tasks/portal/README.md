# PORTAL module — task index

_Generated 2026-05-17 — 8 FRs, 61 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-PORTAL-001](TASK-PORTAL-001-scoped-views/spec.md) | MUST | 1 | 12 | PORTAL scoped read-only views — PROJ/INV/DOC/CHAT filtered by Engagement membership + sync_class=cli |
| [TASK-PORTAL-002](TASK-PORTAL-002-per-tenant-brand-pack/spec.md) | MUST | 1 | 8 | PORTAL per-tenant brand pack — logo + colour palette + custom CNAME + email template overrides + ACM |
| [TASK-PORTAL-003](TASK-PORTAL-003-external-idp-scim-jit/spec.md) | MUST | 1 | 10 | PORTAL external IdP — SAML 2.0 + OIDC sign-in for client-tenant users + SCIM 2.0 JIT provisioning +  |
| [TASK-PORTAL-004](TASK-PORTAL-004-scim-deprovision/spec.md) | MUST | 2 | 8 | PORTAL SCIM deprovision — session invalidation ≤ 30 s on IdP user removal + grace period + cascade r |
| [TASK-PORTAL-005](TASK-PORTAL-005-branded-genie-chat/spec.md) | SHOULD | 2 | 6 | PORTAL branded Genie chat — CUO scope-narrowed by JWT scope_grants + per-Engagement brand pack + IdP |
| [TASK-PORTAL-006](TASK-PORTAL-006-client-initiated-workflows/spec.md) | MUST | 2 | 6 | PORTAL client-initiated workflows — new project request / billing inquiry / support ticket → CHAT th |
| [TASK-PORTAL-007](TASK-PORTAL-007-pwa-installable/spec.md) | SHOULD | 2 | 6 | PORTAL PWA installable — mobile-first Progressive Web App with offline-capable view cache + push not |
| [TASK-PORTAL-008](TASK-PORTAL-008-dsar-self-service/spec.md) | MUST | 2 | 5 | PORTAL DSAR self-service — GDPR Art. 15 + PDPL Art. 17 client-initiated data subject access request  |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-PORTAL-003→TASK-AUTH-103, TASK-PORTAL-003→TASK-AUTH-104
- **CHAT**: TASK-PORTAL-006→TASK-CHAT-005
- **CUO**: TASK-PORTAL-005→TASK-CUO-101
- **TEN**: TASK-PORTAL-001→TASK-TEN-101, TASK-PORTAL-002→TASK-TEN-101

**This module is depended on by:**

- **EMAIL**: TASK-EMAIL-008→TASK-PORTAL-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._