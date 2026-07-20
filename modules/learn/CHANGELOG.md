# Changelog — LEARN

## 2026-05-15 — LEARN module page rewritten to Gold (skills catalogue + VP roll-up + Hội đồng Chuyên môn workflow)

Rewrote `website/docs/modules/learn.html` to Gold. Three strategic roles: (1) skills catalogue (skill tree × 1-5 mastery × bằng cấp/chứng chỉ evidence), (2) VP (Voting Power) roll-up engine (PROJ + TIME + KB → VP score → REW BP distribution), (3) Hội đồng Chuyên môn (Specialist Council) promotion workflow (3-5 peer judges; per-judge scores never exit the LEARN boundary; aggregate-only to HR).

Key changes:
- NEW §0 — 3-card layout + signal-flow Mermaid showing per-judge boundary explicitly + 10-row auto-vs-human matrix
- Risks +5 (R-LEARN-011..015): per-judge score export misconfig (Critical) · VP signal skews toward PROJ-dominant Members · Lumi skill catalogue pushes conflict · Council deliberation memory ingestion (psychological safety) · skill self-claim spam
- KPIs +5: per-judge export attempts blocked · VP fairness variance (≤ 0.40) · skill claim evidence rate (≥ 0.95) · deliberation transcript purge (≤ 30 d) · HR-to-LEARN-to-REW signal latency
- References expanded: §0 + 6 cross-module links + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + task-audit skill
