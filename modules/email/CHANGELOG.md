# Changelog — EMAIL

## 2026-05-15 — EMAIL module page rewritten to Gold (capture surface + Genie draft + outbound defence)

Rewrote `website/docs/modules/email.html` to Gold. Three strategic roles: (1) capture surface (tracked-domain auto-log to CRM activity + PROJ thread-to-issue), (2) Genie draft (Ask Genie composes outbound replies grounded in sanitised thread + CRM + memory + KB), (3) outbound send + defence (DKIM/ARC/BIMI; CaMeL quarantine defeats EchoLeak class).

Key changes:
- Title/meta + hero reframed
- NEW §0 "The bigger picture" — 3-card layout + EMAIL-in-orchestration-spine Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-EMAIL-011..015): thread-to-issue wrong Engagement · Genie draft confidential leak (High) · bulk-send approval bypass · tracked-domain misconfig (auto-log personal) · CaMeL cost spike
- KPIs +5: thread-to-issue conversion accuracy · Genie draft confidential-leak rate (= 0) · bulk-send token compliance (= 1.0) · tracked-domain audit pass · CaMeL cost per inbound
- References expanded: §0 + 7 cross-module links + CaMeL paper + EchoLeak CVE + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + feature-request-audit skill

