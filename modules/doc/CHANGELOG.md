# Changelog — DOC

## 2026-05-15 — DOC module page rewritten to Gold (document repository + e-sign workflow + contract lifecycle)

Rewrote `website/docs/modules/doc.html` to Gold. Three strategic roles: (1) document repository (versioned + ACL'd + 10-year retention), (2) e-sign workflow (partner-routed cryptography to eIDAS QTSP / AATL CA / VN CA; CyberOS-owned workflow + identity verification), (3) contract lifecycle (HR/CRM/ESOP integration + expiry alerts + renewal automation).

Key changes:
- NEW §0 — 3-card layout + partner-routed signing Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-DOC-011..015): cross-module trigger source mismatch · CUO renewal stale terms · expiry cascade miss · multi-jurisdiction cert chain · migrated DocuSign LTV failure
- KPIs +5: cross-module trigger validation (= 1.0) · renewal terms-stamp coverage (= 1.0) · expiry cascade completeness (= 1.0) · multi-jurisdiction cert-chain declaration (= 1.0) · LTV re-validation (≥ 0.95)
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + task-audit skill
