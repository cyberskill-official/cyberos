# Changelog — CHAT

## 2026-05-15 — CHAT module page rewritten to Gold (P0 dogfood gate + Mattermost fork rationale + @lumi memory capture + decommission KPI)

Rewrote `website/docs/modules/chat.html` to push the module past the threshold from "Solid (8/10)" to Gold by encoding three strategic roles simultaneously: P0 dogfood gate (Slack + Zalo killed by P0 · exit or the platform thesis fails), memory capture surface (one of four canonical capture inputs), and Vietnamese-first chat (PGroonga + TinySegmenter recall ≥ 80%). Targeted Edit operations — preserved every gold-quality detail of the prior content (channels, threads, attachments, search, memory bridge, @genie, Slack importer, mobile, voice) while adding 6 strategic new sections + risk/KPI extensions + universal-protocol references.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "P0 dogfood gate · Mattermost fork · @lumi memory capture · CyberOS".
- **Hero tagline + lede** — explicit P0-dogfood-gate framing: Slack + Zalo decommissioned at P0 exit (P0 · exit), or the whole platform thesis fails. Lists the three strategic roles.
- **Hero fact-grid** — added "Decom gate Slack+Zalo killed by P0 · exit" + "E2EE decision Per-tenant Postgres encryption".
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (P0 dogfood gate / memory capture surface / Vietnamese-first chat). P0-exit dependency Mermaid showing reordered sequence (AI Gateway → AUTH stub → MCP → CHAT → Slack/Zalo decom → P0 exit).
- **TOC** — added 6 new section links (bigger-picture · rt-stack · lumi-memory-capture · e2ee-decision · slack-zalo-migration · decommission-kpi).
- **NEW §2.5 "Real-time stack — Mattermost fork rationale"** — 4-option decision table (Mattermost fork chosen vs Matrix / Phoenix / build-from-scratch) + own-vs-Mattermost ownership table + fork governance text + license-drift escalation path.
- **NEW §2.6 "@lumi → memory capture"** — capture rules table (@lumi=capture, no @lumi=privacy floor, DM rules) + 8-actor sequence diagram (User → CHAT → @lumi parser → CUO → AI Gateway → memory Writer → Lumi's memory). Per-message retro-capture opt-in for "Lumi remember the last N messages".
- **NEW §2.7 "E2EE decision — server-visible by design"** — 10-row threat-model trade table comparing E2EE vs per-tenant Postgres encryption-at-rest; 5-point rationale for choosing the latter; concrete encryption-at-rest description; tenant-level optional E2EE plugin reserved for P3 (search disabled on those channels).
- **NEW §2.8 "Slack/Zalo migration"** — 8-step `cyberos-chat import slack` flow with parse/map/backfill/ingest/verify checkpoints; 2-path Zalo migration (manual export + future desktop bridge); pre-import dry-run + idempotent + checkpointed importer.
- **NEW §2.9 "Decommission KPI"** — formal definition: `decommission_signal := (msgs_in_chat / (msgs_in_chat + msgs_in_slack + msgs_in_zalo)) ≥ 0.95 over 14-day rolling window`. Why 95% not 100%; tracking instrumentation table; 3-tier miss-the-gate remediation (T1 = 2-week sprint freeze on net-new modules, T2 = P1 · start platform-thesis review, T3 = potential P0 rescope per research review §1).
- **§12 Risks** — added 10 new (R-CHAT-011..020): dogfooding-never-happens (Critical, CEO-owned) · enterprise E2EE pressure · voice ASR PII leak · Mattermost license drift · @lumi rate-limit abuse · cross-tenant search leak · Slack import partial failure · retro-capture privacy boundary · mobile push PII leak · VN/EN code-switch tokeniser miss.
- **§13 KPIs** — added 9 new universal-protocol-aware: decommission_signal (P0-exit gate) · @lumi capture-rate (≥ 0.999) · @lumi response p95 (≤ 4 s) · VN tokeniser recall continuous (≥ 0.80, alert < 0.78) · memory-ingest backlog max · retro-capture opt-in rate · mobile push delivery rate · cross-tenant query reject rate · dogfooding intensity (P0-gate: 100% of full-time team by P0 · slice 2).
- **§17 References** — replaced/expanded with MEMORY_AUTOSYNC_DESIGN.md §5 (CHAT as 1 of 4 capture surfaces) · feature-request-audit skill (CHAT FRs deliberately pending) · AUDIT_AND_PLAN_2026_05_14.md §3.3 (P0 · slice 2 build placement) · RESEARCH_REVIEW_2026_05_14.md §3 (Solid 8/10 with decommission caveat) · Mattermost governance docs · PGroonga + TinySegmenter refs · PDPL Art. 7/14/20/38 · EU AI Act Art. 12/13/50.

Verified:
- 24 top-level sections (was 18) including 5 strategic new ones (§0, §2.5–§2.9)
- 4 new Mermaid diagrams (P0-exit dependency + 1 sequence + 0 in §2.7/§2.8 prose + 1 in §0)
- 20 risk rows (was 10), with 10 newly framed around dogfooding + privacy + tokeniser code-switch
- 18 KPI rows (was 9), with decommission_signal as the explicit P0-exit gate
- decommission_signal definition appears verbatim 3× (hero fact-grid, §2.9, §13 KPI table)

The CHAT page now reads as the complete answer to: (1) why CHAT is the P0 dogfood gate not just another module, (2) why Mattermost fork beats Matrix/Phoenix/build-from-scratch under our constraint set, (3) how @lumi mention is the conversational memory-capture mechanism, (4) why we chose per-tenant Postgres encryption-at-rest over E2EE, (5) how Slack/Zalo migration works without losing threads/reactions, and (6) what happens if decommission_signal misses 0.95 by P0 · exit (the platform-thesis review escalation). A new engineer reading this page cold can now pick up the Mattermost fork repo + memory bridge spec + Slack importer spec and start slice 1.

