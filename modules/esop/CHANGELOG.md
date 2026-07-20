# Changelog — ESOP

## 2026-05-15 — ESOP module page rewritten to Gold (Phantom Stock vesting + Good/Bad Leaver branch + HoldCo flip)

Rewrote `website/docs/modules/esop.html` to Gold. Three strategic roles: (1) grant lifecycle (issue/vest/cliff/cancel/put), (2) Good Leaver vs Bad Leaver branch on HR offboarding (CFO+CEO co-sign required), (3) liquidity-event simulator (annual valuation + put option exec + Singapore HoldCo flip trigger at ARR ≥ $1.5M).

Key changes:
- NEW §0 — 3-card layout + cap-table spine Mermaid showing memory exclusion + 10-row auto-vs-human matrix
- Risks +5 (R-ESOP-011..015): Leaver branch AI auto-route (Critical) · put-option ARR-trigger drift · vesting accrual on statutory leave · M&A acceleration without Member notice · HoldCo partial-flip rollback
- KPIs +5: Good/Bad Leaver co-sign integrity (= 1.0) · vesting accrual statutory-leave correctness · M&A notification SLA (≤ 5 days) · HoldCo flip cohort success (= 1.0 rollback on partial) · put-option exec query latency
- References expanded: §0 + 5 cross-module links + MEMORY_AUTOSYNC_DESIGN.md + DEC-036 + AUDIT_AND_PLAN + task-audit skill
