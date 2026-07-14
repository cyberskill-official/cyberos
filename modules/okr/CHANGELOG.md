# Changelog — OKR

## 2026-05-15 — OKR module page rewritten to Gold (cascade orchestrator + KR auto-progress + face-saving retros)

Rewrote `website/docs/modules/okr.html` to Gold. Three strategic roles: (1) cascade orchestrator (Company → Team → Member quarterly), (2) KR auto-progress engine (each KR's progress_source query reads PROJ/INV/HR/LEARN; nightly batch), (3) face-saving retro engine (Vietnamese cultural framing: "what did we learn?").

Key changes:
- NEW §0 — 3-card layout + auto-progress data-flow Mermaid + 8-row auto-vs-human matrix
- Risks +5 (R-OKR-010..014): progress source schema drift · face-saving framing weaponised · CUO digest hallucination · OKR-weight skews REW · retro cross-tenant leak
- KPIs +5: progress source schema drift · face-saving pattern detection · digest hallucination rate (≤ 0.01) · OKR-share-of-VP correctness (= 1.0) · retro sync_class default compliance (= 1.0)
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + task-audit skill

