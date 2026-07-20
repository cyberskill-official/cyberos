# Changelog — RES

## 2026-05-15 — RES module page rewritten to Gold (capacity-vs-forecast integrator + hiring forecast + allocation engine)

Rewrote `website/docs/modules/res.html` to Gold. Three strategic roles: (1) capacity-vs-forecast integrator (joins HR + PROJ + TIME + LEARN on Member-id × week; integrator not source-of-truth), (2) hiring forecast (skill-gap × CRM pipeline × LEARN mastery → hire trigger before deliverables drop), (3) allocation engine (CUO/COO drafts rebalance recommendations; VN Labour Code Art. 107 OT caps hard-floor).

Key changes:
- NEW §0 — 3-card layout + integration-model Mermaid (HR/PROJ/TIME/LEARN/CRM → RES → CUO → hiring memo/rebalance proposal) + 10-row auto-vs-human matrix
- Risks +5 (R-RES-010..014): RES forecast becomes CEO-decision dependency · Member-preference flags ignored under high-priority · VN OT-cap version drift · cross-Engagement reallocation rate-card mismatch · Lumi RES synthesis leaks Engagement intel
- KPIs +6: hiring memo CEO acceptance rate · Member-preference override rate (= 1.0) · cross-Engagement rate-card alignment · cap version stamp coverage (= 1.0) · Lumi cross-tenant sign-off (= 1.0)
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + task-audit skill
