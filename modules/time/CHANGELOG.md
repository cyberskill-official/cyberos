# Changelog — TIME

## 2026-05-15 — TIME module page rewritten to Gold (billable-hours engine + PROJ-INV bridge + Labour-law guardrails)

Rewrote `website/docs/modules/time.html` to Gold by encoding three strategic roles: (1) hours entry (timer + manual + auto-detect from PROJ activity), (2) billable rules engine (4-step cascade per PROJ §2.6: Member override → task class → role default → fallback; decision snapshotted on row), (3) PROJ-INV bridge (per-cycle billable rollup feeds INV).

Key changes:
- Title/meta + hero reframed; fact-grid extended (8→11 cards: + Strategic role, Billable cascade, Labour caps VN Code Art. 107)
- NEW §0 "The bigger picture" — 3-card layout + spine Mermaid (PROJ → Member → TIME → Billable cascade → AM → CFO + INV/REW/memory) + 9-row auto-vs-human matrix
- Risks +5 (R-TIME-011..015): billable cascade snapshot divergence (High) · auto-detect wrong Issue · VN Labour Code 2026 amendment · cycle-rollup runs before all submissions · multi-currency drift
- KPIs +6: cascade snapshot integrity (= 1.0 hard floor) · auto-detect acceptance · PROJ-TIME issue match rate · cycle-rollup completeness · VN Labour Code version coverage (= 1.0)
- References expanded: §0 + 6 cross-module links + PROJ §2.6 billable cascade link + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + feature-request-audit skill

