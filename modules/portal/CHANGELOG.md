# Changelog — PORTAL

## 2026-05-15 — PORTAL module page rewritten to Gold (client-facing surface + scoped read-only views + external IdP)

Rewrote `website/docs/modules/portal.html` to Gold. Three strategic roles: (1) scoped read-only client surface (PROJ/INV/DOC/CHAT views filtered by Engagement membership + sync_class=client-visible), (2) per-tenant brand pack (white-label theme + custom CNAME), (3) external IdP integration (client logs in via own SAML/OIDC; JIT provisioning; never stores password).

Key changes:
- NEW §0 — 3-card layout + multi-tenant-within-multi-tenant Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-PORTAL-011..015): sync_class misconfig leak (Critical) · JIT role-mapping wrong · SVG XSS · Client AI cross-Engagement cite (Critical) · SCIM deprovision delay
- KPIs +6: sync_class filter pass (= 1.0) · JIT role accuracy (≥ 0.99) · SVG XSS blocks · cross-Engagement rejection rate · SCIM session-invalidation p95
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + feature-request-audit skill

