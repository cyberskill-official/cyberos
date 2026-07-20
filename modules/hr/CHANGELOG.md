# Changelog — HR

## 2026-05-15 — HR module page rewritten to Gold (member lifecycle + onboarding orchestrator + performance signal aggregator)

Rewrote `website/docs/modules/hr.html` to Gold. Three strategic roles: (1) member lifecycle owner with AUTH-provisioned subject + multi-module event fan-out, (2) onboarding orchestrator (LEARN + KB + PROJ ramp plans saga-fired automatically), (3) performance signal aggregator (read-only consumer of PROJ + TIME + LEARN signals; comp number lives in REW, never HR).

Key changes:
- Title/meta + hero reframed
- NEW §0 — 3-card layout + Member-id spine Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-HR-011..015): HR signals used as sole comp basis · cross-tenant Member-id collision (Critical) · onboarding fires before AUTH ready · VN labour-law mid-year amendment · sabbatical tick misclassification
- KPIs +5: signal-only comp decision rate (= 1.0) · onboarding playbook saga p95 · labour-law version stamp coverage (= 1.0) · HR-to-REW handoff p95 · statutory-leave classification accuracy
- References expanded: §0 + 7 cross-module links + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + task-audit skill
