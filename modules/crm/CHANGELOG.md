# Changelog — CRM

## 2026-05-15 — CRM module page rewritten to Gold (sales-pipeline spine + Deal-to-Engagement bridge + next-action engine)

Rewrote `website/docs/modules/crm.html` to Gold by encoding three strategic roles: (1) sales pipeline VN-first (Account · Contact · Deal with VN integrations: MST validation, VietQR, hóa đơn, salutation logic), (2) Deal-to-Engagement bridge to PROJ §2.5 join contract (deal.won → engagement.create with rate card pre-wired), (3) next-action engine (CUO ranks moves on every open deal; AI lead scoring; win/loss memories citable by future deals).

Key changes:
- Title/meta + hero reframed to 3 strategic roles
- Fact-grid extended (8→11 cards: + Strategic role, Deal → Engagement bridge One-click, Vertical-pack ready)
- NEW §0 "The bigger picture" — 3-card layout + CRM-in-orchestration-spine Mermaid + 9-row auto-vs-human matrix
- Risks +5 (R-CRM-011..015): bridge fails partially · wrong billing mode · CUO next-action inappropriate · vertical-pack drift · merge data loss
- KPIs +6: deal-to-Engagement conversion rate · conversion bridge p95 · win/loss memory citation rate · next-action acceptance · stage-stuck deal alert · forecast accuracy
- References expanded: §0 + 7 cross-module links + PROJ §2.5 join contract link + SKILL §3.6 vertical-pack pattern + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + task-audit skill + expanded PDPL articles

