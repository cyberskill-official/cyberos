# Changelog — REW

## 2026-05-15 — REW module page rewritten to Gold (compensation engine + payroll bridge + bonus orchestrator)

Rewrote `website/docs/modules/rew.html` to Gold. Three strategic roles: (1) compensation record owner (encrypted, HR-isolated, structurally excluded from memory per DEC-036), (2) payroll bridge (monthly VND cycle with BHXH/BHYT/BHTN, immutable parameter versioning, byte-identical PDF replay), (3) bonus orchestrator (BP fund + calibration → P3 distribution + CEO/CFO sign-off; P1-protection invariant DB-CHECK enforced).

Key changes:
- Title/meta + hero reframed; "Bet 5 moat" + EU AI Act Annex III §4 high-risk framing preserved
- NEW §0 — 3-card layout + REW-isolated-by-design Mermaid (HR/TIME/PROJ → REW → CFO+CHRO co-sign → payslips → banks/BHXH; memory explicitly disconnected with structural-exclusion line) + 10-row auto-vs-human matrix
- Risks +5 (R-REW-011..015): HR signals weaponised for P3 cut · BHXH mid-month rate change · Lumi attempts read REW (Catastrophic) · cross-Member cache leak · CFO+CHRO collusion (P1 protection at DB CHECK, not app layer alone)
- KPIs +5: P3 distribution sign-off completeness (= 1.0) · parameter mid-month transition correctness · Lumi-attempted reads (= 0) · cross-Member cache leak attempts (= 0) · P1 DB-CHECK constraint violations (any > 0 = sev-0)
- References expanded: §0 + 6 cross-module links + MEMORY_AUTOSYNC_DESIGN.md §5 + DEC-036 + AUDIT_AND_PLAN + task-audit skill

