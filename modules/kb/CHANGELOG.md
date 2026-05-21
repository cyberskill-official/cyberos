# Changelog — KB

## 2026-05-15 — KB module page rewritten to Gold (RAG corpus + memory companion + auto-runbook catalogue source)

Rewrote `website/docs/modules/kb.html` to Gold. Three strategic roles: (1) RAG corpus with three-layer retrieval (FTS5/PGroonga + BGE-M3 + cross-encoder) + span-level citations, (2) memory companion (long-form versioned counterpart to chain-anchored memories; "promote to canonical" elevates to high-authority source consumable by Lumi cross-tenant synthesis), (3) runbook catalogue source for OBS auto-runbook router (KB outage breaks OBS triage = critical coupling).

Key changes:
- Title/meta + hero reframed
- NEW §0 "The bigger picture" — 3-card layout + KB-in-platform Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-KB-011..015): runbook catalogue drift · OBS-KB tight coupling (KB outage breaks triage, High impact) · span-citation drift · vendor-pack malicious markdown · doc-gap-detector underperforms
- KPIs +5: runbook applicability accuracy · span-citation integrity (= 1.0) · doc-gap-detector signal rate · cross-tenant retrieval reject rate · vendor-pack CSO-review rate (= 1.0)
- References expanded: §0 + 6 cross-module links + OBS §2.6 auto-runbook contract link + MEMORY_AUTOSYNC_DESIGN.md §6 + AUDIT_AND_PLAN + feature-request-audit skill

