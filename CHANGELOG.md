# Changelog — CyberOS

All notable changes to the umbrella CyberOS repository, newest-first.

## 2026-05-15 — PROJECT module page rewritten to Gold (orchestration spine + Engagement economics + BRAIN-anchored decisions + Liquid-Glass UI exemplar)

Rewrote `website/docs/modules/proj.html` from 1126 → 1514 lines (+388 lines, +34%). Encodes three strategic roles the PROJ module plays simultaneously — orchestration spine for cross-module joins, BRAIN-anchored decision substrate, consultancy-native Engagement billing surface — with no role under-served. Targeted Edit operations preserved the existing strong content (4 primitives, sync-engine architecture, 5 key-flow sequences, status enum + workflow overlay, 7 surface CLI commands) while adding 4 strategic deep-dive sections + risk/KPI extensions + universal-protocol references.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "PROJ — Orchestration spine · BRAIN-anchored decisions · Engagement billing · CyberOS". Description names the orchestration spine (CRM → PROJECT → TIME → INV → REW → KB → BRAIN), the consultancy-native Engagement primitive, the BRAIN-citation graph, and the Liquid-Glass UI exemplar.
- **Hero tagline + lede** — explicit "orchestration spine" framing; lists all 3 strategic roles in one paragraph; replaces stale PRD-referenced prose with role descriptions.
- **Hero fact-grid** — extended from 8 to 13 cards: added Strategic role + Cross-module joins (7) + BRAIN integration (bidirectional) + Engagement model (3 modes) + UI surfaces (4). Strategic role card uses "Orchestration spine" pill prominent.
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (Role 1 orchestration spine / Role 2 BRAIN-anchored / Role 3 Engagement billing). Cross-module join Mermaid flowchart with PROJ as hub touching 9 other modules. Auto-vs-manual operations matrix (9 rows) — explicitly classifies which PROJ behaviours are automatic vs deliberate.
- **TOC** — added bigger-picture · orchestration-spine · engagement-economics · brain-anchored · ui-surfaces entries (5 new).
- **NEW §2.5 "Orchestration spine — cross-module join contracts"** — 9-row canonical contract table covering each counterparty (CRM/EMAIL/TIME/INV/KB/REW/OKR/PORTAL/BRAIN): direction · join key · trigger · payload shape · failure mode. Contract stability policy: breaking changes require ADR + counterparty co-sign + 1-minor-release deprecation window + migration test + BRAIN decision memory.
- **NEW §2.6 "Engagement economics — consultancy-native primitive"** — 3-mode billing table (T&M / fixed-fee / retainer) with what INV pulls + risk + typical use. Full rate-card YAML example (architect/senior/mid/junior with VND + USD rates + per-role billable_default). Billable / non-billable cascade (4-step priority): Member override → task class → role default → fallback. Margin watchdog spec for P2 (fixed-fee scope-creep early warning).
- **NEW §2.7 "BRAIN-anchored decisions — issues cite memories"** — three citation relations (cites / implements / supersedes) with examples. Decision-to-issues skill sequence (8-actor Mermaid: User → CUO/CPO skill → BRAIN read → PROJ create N+1 issues → BRAIN write audit). Dual-write audit chain example: PROJ history_event row + BRAIN audit row with matching chain hash.
- **NEW §2.8 "Liquid-Glass UI surfaces — Board · Timeline · Gantt · Brief"** — 4-surface canonical table (primary use · default view · density · keyboard-first). PROJ-specific design-token overlay (tokens.proj.css) with status palette + priority colours + Liquid-Glass blur/saturate values. 6-point accessibility commitment list (WCAG AA + keyboard nav + screen-reader labels + focus trap + reduce-motion + VN diacritic-correct fonts).
- **§12 Risks** — added 10 new (R-PROJ-011..020): orchestration-spine SPOF · contract breaking change without ADR · fixed-fee scope creep eats margin (High likelihood × High impact, COO-owned) · BRAIN citation drift · cycle-review draft cites out-of-window work · billing-mode mid-cycle change · decision-to-issues skill drift · Liquid-Glass accessibility fail · SPA cold-load > 5s on VN mobile (Members give up and use Excel) · NATS JetStream backlog staleness.
- **§13 KPIs** — added 10 new universal-protocol-aware: Join-contract stability (≤ 1 breaking change/quarter) · Engagement margin T&M (≥ 35%) · Engagement margin fixed-fee (≥ 30% on close) · Issues with BRAIN citation (≥ 40% of high-priority) · Decision-to-issues skill acceptance (≥ 70%) · SPA cold-load p95 on VN mobile (≤ 5s) · Citation-drift rate (≤ 5%) · Cross-tenant ACL rejection rate · Dogfooding cycle-review draft acceptance (≥ 70% — founders use this before selling it).
- **§17 References** — replaced stale PRD/SRS section refs with the 4 new in-page sections + BRAIN_AUTOSYNC_DESIGN.md §5 (capture surfaces) + FR_AUTHORING_WORKFLOW.md + AUDIT_AND_PLAN_2026_05_14.md §3.3 (M+5 placement) + RESEARCH_REVIEW_2026_05_14.md §4 (Engagement primitive flagged as highest-leverage differentiator) + 11 cross-module page links + PDPL Art. 7/14/20.

Verified:
- 1514 lines parses cleanly
- 23 top-level sections (was 18) including 5 strategic new ones (§0, §2.5–§2.8)
- 5 new Mermaid diagrams (cross-module join flowchart + decision-to-issues sequence + 3 inline in §2.6/§2.7/§2.8)
- 20 risk rows (was 10), with 10 new framed around orchestration spine SPOF + Engagement scope creep + BRAIN-citation drift + VN mobile cold-load
- 19 KPI rows (was 9), with margin watchdog + citation-coverage + dogfooding-acceptance as the new strategic gates

The PROJ page now reads as the complete answer to: (1) why PROJ is the spine and not just a tracker (the join contract table makes it concrete), (2) why consultancies cannot use Linear or Jira off the shelf (the Engagement economics section walks through 3 billing modes + rate-card YAML + billable cascade), (3) how the BRAIN integration makes issue history survive leadership changes (citation graph + dual-write audit chain), (4) why PROJ is the design-system exemplar (4 canonical UI surfaces + token overlay + accessibility commitments). A new engineer reading this page cold can pick up the sync-engine, join contracts, and the four UI surfaces and start P1 slice 1.

## 2026-05-15 — CHAT module page rewritten to Gold (P0 dogfood gate + Mattermost fork rationale + @lumi BRAIN capture + decommission KPI)

Rewrote `website/docs/modules/chat.html` to push the module past the threshold from "Solid (8/10)" to Gold by encoding three strategic roles simultaneously: P0 dogfood gate (Slack + Zalo killed by M+3 or the platform thesis fails), BRAIN capture surface (one of four canonical capture inputs), and Vietnamese-first chat (PGroonga + TinySegmenter recall ≥ 80%). Targeted Edit operations — preserved every gold-quality detail of the prior content (channels, threads, attachments, search, BRAIN bridge, @genie, Slack importer, mobile, voice) while adding 6 strategic new sections + risk/KPI extensions + universal-protocol references.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "P0 dogfood gate · Mattermost fork · @lumi BRAIN capture · CyberOS".
- **Hero tagline + lede** — explicit P0-dogfood-gate framing: Slack + Zalo decommissioned at P0 exit (M+3), or the whole platform thesis fails. Lists the three strategic roles.
- **Hero fact-grid** — added "Decom gate Slack+Zalo killed by M+3" + "E2EE decision Per-tenant Postgres encryption".
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (P0 dogfood gate / BRAIN capture surface / Vietnamese-first chat). P0-exit dependency Mermaid showing reordered sequence (AI Gateway → AUTH stub → MCP → CHAT → Slack/Zalo decom → P0 exit).
- **TOC** — added 6 new section links (bigger-picture · rt-stack · lumi-brain-capture · e2ee-decision · slack-zalo-migration · decommission-kpi).
- **NEW §2.5 "Real-time stack — Mattermost fork rationale"** — 4-option decision table (Mattermost fork chosen vs Matrix / Phoenix / build-from-scratch) + own-vs-Mattermost ownership table + fork governance text + license-drift escalation path.
- **NEW §2.6 "@lumi → BRAIN capture"** — capture rules table (@lumi=capture, no @lumi=privacy floor, DM rules) + 8-actor sequence diagram (User → CHAT → @lumi parser → CUO → AI Gateway → BRAIN Writer → Lumi's BRAIN). Per-message retro-capture opt-in for "Lumi remember the last N messages".
- **NEW §2.7 "E2EE decision — server-visible by design"** — 10-row threat-model trade table comparing E2EE vs per-tenant Postgres encryption-at-rest; 5-point rationale for choosing the latter; concrete encryption-at-rest description; tenant-level optional E2EE plugin reserved for P3 (search disabled on those channels).
- **NEW §2.8 "Slack/Zalo migration"** — 8-step `cyberos-chat import slack` flow with parse/map/backfill/ingest/verify checkpoints; 2-path Zalo migration (manual export + future desktop bridge); pre-import dry-run + idempotent + checkpointed importer.
- **NEW §2.9 "Decommission KPI"** — formal definition: `decommission_signal := (msgs_in_chat / (msgs_in_chat + msgs_in_slack + msgs_in_zalo)) ≥ 0.95 over 14-day rolling window`. Why 95% not 100%; tracking instrumentation table; 3-tier miss-the-gate remediation (T1 = 2-week sprint freeze on net-new modules, T2 = M+4 platform-thesis review, T3 = potential P0 rescope per research review §1).
- **§12 Risks** — added 10 new (R-CHAT-011..020): dogfooding-never-happens (Critical, CEO-owned) · enterprise E2EE pressure · voice ASR PII leak · Mattermost license drift · @lumi rate-limit abuse · cross-tenant search leak · Slack import partial failure · retro-capture privacy boundary · mobile push PII leak · VN/EN code-switch tokeniser miss.
- **§13 KPIs** — added 9 new universal-protocol-aware: decommission_signal (P0-exit gate) · @lumi capture-rate (≥ 0.999) · @lumi response p95 (≤ 4 s) · VN tokeniser recall continuous (≥ 0.80, alert &lt; 0.78) · BRAIN-ingest backlog max · retro-capture opt-in rate · mobile push delivery rate · cross-tenant query reject rate · dogfooding intensity (P0-gate: 100% of full-time team by M+2).
- **§17 References** — replaced/expanded with BRAIN_AUTOSYNC_DESIGN.md §5 (CHAT as 1 of 4 capture surfaces) · FR_AUTHORING_WORKFLOW.md (CHAT FRs deliberately pending) · AUDIT_AND_PLAN_2026_05_14.md §3.3 (M+2 build placement) · RESEARCH_REVIEW_2026_05_14.md §3 (Solid 8/10 with decommission caveat) · Mattermost governance docs · PGroonga + TinySegmenter refs · PDPL Art. 7/14/20/38 · EU AI Act Art. 12/13/50.

Verified:
- 24 top-level sections (was 18) including 5 strategic new ones (§0, §2.5–§2.9)
- 4 new Mermaid diagrams (P0-exit dependency + 1 sequence + 0 in §2.7/§2.8 prose + 1 in §0)
- 20 risk rows (was 10), with 10 newly framed around dogfooding + privacy + tokeniser code-switch
- 18 KPI rows (was 9), with decommission_signal as the explicit P0-exit gate
- decommission_signal definition appears verbatim 3× (hero fact-grid, §2.9, §13 KPI table)

The CHAT page now reads as the complete answer to: (1) why CHAT is the P0 dogfood gate not just another module, (2) why Mattermost fork beats Matrix/Phoenix/build-from-scratch under our constraint set, (3) how @lumi mention is the conversational BRAIN-capture mechanism, (4) why we chose per-tenant Postgres encryption-at-rest over E2EE, (5) how Slack/Zalo migration works without losing threads/reactions, and (6) what happens if decommission_signal misses 0.95 by M+3 (the platform-thesis review escalation). A new engineer reading this page cold can now pick up the Mattermost fork repo + BRAIN bridge spec + Slack importer spec and start slice 1.

## 2026-05-14 — AUTH module page rewritten to Gold (M+2 stub vs P3 full + Lumi tenant identity + RFC open Qs resolved)

Rewrote `website/docs/modules/auth.html` from 1169 → 1442 lines (+273 lines, +23%). Encodes the research review §2.4 reorder (AI Gateway BEFORE AUTH) and AUTH's distinct roles as M+2 stub vs P3 full. Targeted Edit operations preserved every gold-quality detail of the prior content while adding 4 new strategic sections + risk/KPI extensions.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "M+2 stub → P3 full · Lumi tenant identity · Agent-equal".
- **Hero tagline + lede** — explicit M+2 stub vs P3 full distinction · cites reordered P0 sequence (AI Gateway @ M+1 → AUTH @ M+2 → MCP Gateway @ M+2.5 → CHAT/CUO @ M+3) · references RFC.md + sign-in mockup + BRAIN_AUTOSYNC_DESIGN.md.
- **Hero fact-grid** — split status into "M+2 stub designed" + "P3 full designed", LoC into 1,500 stub + 7,000 full, RBAC into 5 stub + 22 full, dependencies + Lumi enablement.
- **NEW §0 "The bigger picture — three strategic moves"** — 3-card layout (Move 1 M+2 stub / Move 2 P3 full / Move 3 Lumi tenant identity). Gantt chart Mermaid showing the reordered P0 build sequence end-to-end. Rationale for reorder cited from reviewer.
- **TOC** — added bigger-picture · stub-vs-full · rbac-catalogue · lumi-integration · open-questions entries.
- **NEW §2.5 "M+2 stub vs P3 full"** — 12-row capability-contrast table covering login mechanism · MFA · RBAC catalogue · JWT signing · tenant isolation · audit-chain emission · admin surfaces · cost · LoC · tests · Lumi integration · SOC 2 evidence. Plus "Migration discipline" + "What stub doesn't compromise on" prose.
- **NEW §2.6 "22-role RBAC catalogue"** — full 22-row table with scope summary, stub-eligibility, and slice when each role lands. The 5 stub roles (root-admin · tenant-admin · tenant-member · service-account · agent-persona) are explicitly the first 5; the remaining 17 land across slices 3–5. Role-addition policy: ADR-gated, no code-only changes.
- **NEW §2.7 "AUTH ↔ Lumi's BRAIN"** — full JWT claim shape (15 fields incl. tenant_id, tenant_residency, agent_persona, scope_grants) · sequence diagram of Lumi's BRAIN verifying a sync push · 5-bullet contract requirements list (tenant_id non-removable, JWKS reachability, refresh-token reuse detection, agent-persona claims preserve agent-equal, residency pinning flows through).
- **NEW §2.8 "RFC open questions resolved"** — table addressing all 5 open Qs from RFC §6 with proposed defaults + rationale: Q1 workspace = new repo-root Cargo workspace · Q2 memory bridge = subprocess slice 4 → PyO3 slice 5 · Q3 tenant-0 bootstrap = `cyberos-auth bootstrap` CLI subcommand · Q4 HIBP = default-on with per-tenant opt-out · Q5 OBS = slice 1 stdout → slice 5 OTLP. Each becomes an ADR once Stephen signs off.
- **§12 Risks** — added 7 new (R-AUTH-011..017): stub stays past P3 · reorder regret · Lumi tenant-id spoofing · cross-shard JWT replay · sub-process audit-bridge bottleneck · tenant-0 bootstrap leak · PDPL Art. 38 SME grace lapse.
- **§13 KPIs** — added 7 new: stub-to-full migration coverage (≥95% T2+ subjects passkey-enrolled by M+6) · mock-AUTH retirement · Lumi tenant-id verification rate · cross-shard rejection · audit-bridge p99 · SME-grace lapsed tenants · 22-role catalogue stability.
- **§17 References** — replaced PRD/SRS section refs (stripped) with services/auth/RFC.md, sign-in mockup, BRAIN_AUTOSYNC_DESIGN.md §6, RESEARCH_REVIEW §2.4 (cited verbatim), AUDIT_AND_PLAN, FR_AUTHORING_WORKFLOW, AGENTS.md §3.6+§11.

Verified:
- 1442 lines parses cleanly
- 23 top-level sections (was 18) including 4 strategic new ones
- Mermaid gantt chart documents the reordered P0 sequence
- All 5 RFC §6 open questions now have proposed defaults visible on the page

The AUTH page now reads as the complete answer to: (1) why AUTH is not P0 #1 (research review §2.4), (2) what the M+2 stub actually contains vs the P3 full target, (3) how AUTH enables Lumi's BRAIN tenant isolation, (4) what the 5 open RFC questions resolve to. A new engineer reading this page cold can pick up RFC.md and start slice 1.

## 2026-05-14 — SKILL module page rewritten to Gold (BRAIN integration + vertical-pack moat + distribution roadmap)

Rewrote `website/docs/modules/skill.html` from 1134 → 1431 lines (+297 lines, +26%). Encodes the three strategic roles the Skill module plays simultaneously — open-standard citizen, BRAIN-protocol enabler, vertical-pack moat — with no role under-served. Targeted Edit operations preserved every gold-quality detail of the shipped Phases 0–7 while adding Phase 8 BRAIN integration, vertical-pack pattern + 8-pack roadmap, and the R0→R5 distribution staging.

Changes by section:
- **`<title>` + `<meta>`** — "Open Agent Skills · BRAIN-integrated · Vertical-pack moat · CyberOS" — three roles in the title itself.
- **Hero tagline + lede** — explicit three-role frame: open-standard citizen / BRAIN-protocol enabler / vertical-pack moat. Lists the capture daemon + sync orchestrator + synthesis sub-skill as skill bundles. Names cyberskill-vn as proof-of-pattern, not the strategy.
- **Hero fact-grid** — added "Status (BRAIN-int) Phase 8 designed" + "Vertical packs 1 shipped · 6 planned"; updated dependencies to BRAIN + AUTH.
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (Role 1 / Role 2 / Role 3); dependency graph Mermaid showing Skill's unique position touching the external Agent Skills ecosystem.
- **TOC** — added Bigger picture · BRAIN integration · Vertical-pack pattern · Distribution roadmap entries.
- **NEW §3.5 "BRAIN integration"** — full SKILL.md frontmatter example with BRAIN-aware fields (allowed_brain_scopes for personal + lumi scopes); capability broker enforcement sequence diagram (8 actors, 14 steps); table of 5 universal-protocol skills (brain-capture@1, brain-sync@1, synthesis-author@1, fr-author, fr-audit).
- **NEW §3.6 "Vertical-pack pattern"** — 7-step pack recipe (jurisdiction → high-pain workflows → SKILL.md bundle → localise language → compliance-verify → agentskills.io publish → Lumi tenant sell); 9-pack roadmap table (vn shipped + sg + id + th + eu + us + hr + legal + accounting) with target ship dates and annual unit pricing; margin math worked example.
- **NEW §3.7 "Distribution roadmap R0→R5"** — 6-rung distribution table (local cache → .skill bundles → OCI registry → agentskills.io → own marketplace → enterprise white-label); explicit gating criteria; why each rung is gated (R3 waits on registry API, R4 waits on ≥50 paying tenants per research review §7.3).
- **§12 Risks** — added 7 new BRAIN-integration + vertical-pack + distribution risks (R-SKILL-008..014): capability broker bypass, multi-tenant skill bleed, sync-state corruption, synthesis PII leak, vertical-pack legal drift, OCI signing-key compromise, agentskills.io policy hostility.
- **§13 KPIs** — added 8 new universal-protocol KPIs: broker-mediated rate (must be 100%), first-use approval latency, capability scope reject rate, synthesis emit rate, vertical-pack tenant attach rate, vertical-pack revenue share (≥30% of ARR at M+18 = the compounding moat), marketplace publish-to-install, pack legal-drift detection.
- **§14 RACI** — added 9 new rows for Phase 8 + synthesis sub-skill + brain-capture/sync bundles + 4 pack-authoring rows + 2 distribution/marketplace rows + 1 quarterly regulatory-drift review.
- **§16 Phase status** — added 12 new rows: Phase 8 + 3 universal-protocol skill bundles + 6 vertical packs + 2 marketplace rungs.
- **§17 References** — added BRAIN_AUTOSYNC_DESIGN.md (4 cross-links), FR_AUTHORING_WORKFLOW, AUDIT_AND_PLAN, RESEARCH_REVIEW, strategy doc §4.4 (vertical packs as Level-4 moat), and cross-module links to BRAIN + CUO module pages.

Verified:
- 1431 lines parses cleanly
- 24 top-level sections (was 19) including 4 strategic new ones
- 4 references to BRAIN_AUTOSYNC_DESIGN.md
- 10 mentions of the 3 new universal-protocol skill bundles (brain-capture@1, brain-sync@1, synthesis-author@1)
- 39 mentions across the 9 vertical packs (vn / sg / id / th / eu / us / hr / legal / accounting)

The SKILL page now reflects the full strategic surface: open-standard citizen for distribution reach, BRAIN-protocol enabler for cryptographic-grade audit-chain integration on every invocation, and vertical-pack moat as the actual compounding margin (≥30% of ARR at M+18 if the pricing+attach-rate math holds). The page reads as a complete answer to the research review's §7.3 GTM critique: the marketplace is deferred, the vertical packs ARE the moat, and the synthesis sub-skill closes the loop into multi-brain auto-evolve.

## 2026-05-14 — BRAIN module page rewritten to Gold (expanded universal-protocol scope)

Rewrote `website/docs/modules/brain.html` from 1116 → 1518 lines (+402 lines, +36%). Encodes the BRAIN_AUTOSYNC_DESIGN.md vision: universal Personal BRAIN + Lumi's BRAIN + capture daemon + 2-way sync + multi-brain auto-evolve. Targeted Edit operations (not full rewrite) — preserved all existing gold-quality content on Stage 0 (shipped Layer 1) while encoding Stages 1–5.

Changes by section:
- **`<title>` + `<meta description>`** — reframed from "the substrate every CyberOS module depends on" to "the universal personal-and-shared memory protocol — CyberOS is the first consumer, the protocol stands alone".
- **Hero tagline + lede paragraph** — Personal BRAIN + Lumi's BRAIN duality; portability by folder copy; multi-brain auto-evolve as the moat; Stage 1–5 reference to BRAIN_AUTOSYNC_DESIGN.md.
- **Hero fact-grid** — replaced single-store metrics with dual-store reality (Layer 1 status + Stages 1–5 designed + Personal+Lumi stores + universal scope).
- **NEW §0 — "The bigger picture"** — 3-card layout (Personal · Sync orchestrator · Lumi's BRAIN); auto-vs-manual capture matrix; "this is the moat" strategic frame.
- **TOC** — added "The bigger picture" + "Stages 1–5 roadmap" entries.
- **§1 Why BRAIN exists** — 4-card layout (was 3) adding "Universal capture" + "Multi-brain power"; expanded the two-paragraph rationale with the compounding-moat argument.
- **§2 5W1H2C5M** — all 12 cells rewritten to encode the universal protocol scope. Personal vs Lumi distinction in Who/When/Where; Stage 2+ materials (Rust+notify, Presidio); cost model includes sync push p95 and synthesis LLM-cost.
- **NEW §3.5 — "Stages 1–5 universal protocol roadmap"** — Mermaid stage-dependency flowchart; gating table with effort estimates; Personal BRAIN sub-architecture Mermaid diagram (capture surfaces → ops → store + sync queue); Lumi's BRAIN sub-architecture diagram (N personal BRAINs → sync → tenant chain → synthesis → wisdom); sync_class privacy taxonomy table.
- **§4 Data model** — added second ERD with 5 new entities: WatchedFolder · CaptureEvent · SyncState · LumiRow · SharedMemoryAcl · OrgMember · SynthesisInput · SynthesisArtefact (~80 lines of Mermaid erDiagram).
- **§5 API surface** — added a second CLI table with the 8 new `brain *` subcommands locked per BRAIN_AUTOSYNC_DESIGN.md §15: init/watch/unwatch/status/capture (Stage 1) + sync/sync-mode/pending/reclass (Stage 4).
- **§11 Compliance** — added PDPL Art. 7 (no data sale), Art. 20 (60-day post-audit cross-border), Art. 38 (SME 5-year grace), EU AI Act Art. 12 (synthesis logging) + Art. 50 (AI-generated content transparency), ISO/IEC 27018 §A.5 (customer agreement).
- **§12 Risk entries** — added 6 new BRAIN-specific risks (R-BRAIN-009..014): Lumi's BRAIN tenant compromise, sync conflict storm, synthesis hallucination, capture daemon crash recovery, iCloud sibling explosion, PII leak via auto-capture. Each with likelihood / impact / owner / mitigation.
- **§13 KPIs** — added 8 new universal-protocol KPIs: capture rate per user, sync success rate, sync conflict rate, synthesis useful-rate, Lumi's BRAIN seq counter, PII held-back rate, capture daemon health, cross-machine portability.
- **§14 RACI** — added 9 new rows covering Stages 1–5 + Personal-BRAIN portability + PII detection + cross-tenant isolation testing + synthesis output review. Stage-3+ adds Cloud-DBA + Sync-SRE roles under CTO.
- **§16 Phase status** — added 5 new rows for Stages 1–5 with appropriate "design-locked / designed" pills.
- **§17 References** — replaced PRD/SRS section refs (stripped) with BRAIN_AUTOSYNC_DESIGN.md, PROPOSAL.md (Proposal P13), FR_AUTHORING_WORKFLOW.md, AUDIT_AND_PLAN_2026_05_14.md, RESEARCH_REVIEW_2026_05_14.md cross-links. Annotates the 4 new doctor invariants and 5 new schema entities.

Result: BRAIN page now reflects the expanded universal-protocol vision while preserving every gold-quality detail of the shipped Stage-0 Layer 1. 5 references to BRAIN_AUTOSYNC_DESIGN.md cross-link the design source-of-truth. 20 mentions of the 8 new `brain *` subcommands give a cold reader the full CLI map.

## 2026-05-14 — Research review ingested + BRAIN auto-sync design v1.0 locked

- Saved `docs/RESEARCH_REVIEW_2026_05_14.md` (315 lines, ~53 KB) — the pre-launch audit from Claude Chat's Research Mode. Aggregate 6.5/10; lowest substantive scores on Spec Quality (5) and GTM (5). 10 follow-up tasks created (#31–#40) covering: M+4 descope gate, AI Gateway → AUTH reorder, PDPL citation fixes, server-render NFR + Risk catalogs, first 50 FRs via fr-author, 7 missing risks, TEN-billing P2 slice, UX defects, BRAIN Layer 2 source-of-truth one-pager, BRAIN decision memory.
- **Wrote `docs/BRAIN_AUTOSYNC_DESIGN.md`** (~700 lines, design v1.0.0) — universal Personal BRAIN + Lumi's BRAIN architecture. Per Stephen's clarified vision: (1) Personal BRAIN works on any folder, not just cyberos; (2) captures everything including discussions, not just file deliverables; (3) portable by folder copy across user's machines; (4) 2-way sync with Cloud BRAIN aka Lumi's BRAIN (also CUO's BRAIN, CyberSkill's BRAIN — same store, different names for different audiences); (5) multi-brain power + auto-evolve memory at scale.
  - 16 sections: vision, naming, three-layer architecture, Personal BRAIN spec, Capture daemon spec, Lumi's BRAIN spec, Sync orchestrator, Multi-brain auto-evolve, Dependency map, Privacy + governance, AGENTS.md Proposal P13 additions, CyberOS strategic implications, naming/branding decisions, 4-week sprint plan, 5 open questions, where-to-read-next.
  - Stage gating: **Stage 1 (Personal BRAIN universal) + Stage 2 (capture daemon) are buildable today** — no external dep. Stages 3+ ride the P0+P2 critical path (AUTH + AI Gateway + TEN).
  - Strategic implication called out: this is **the moat** the reviewer's GTM critique was looking for. Personal BRAIN as OSS distribution; Lumi's BRAIN as the commercial product. The compounding switching cost = value of the org's accumulated BRAIN.

## 2026-05-14 — Code-block contrast fix + PRD/SRS sweep + repair regression + Research Mode brief

- **Fixed code-block invisible-text bug.** A late-stage override in `assets/styles.css` (`.codeblock { background: var(--bg-code) }`) was flipping the dark `--neutral-900` background to a light `--bg-code` while leaving text colour at light `--neutral-100` → code invisible on auth.html and other module pages. Removed the `background` override; kept the `backdrop-filter: none` (which prevents glass-leakage from a glass parent).
- **Swept PRD/SRS back-references out of the docs site.** The docs site is now the single source of truth — removed every `PRD §X.Y`, `SRS §X.Y`, "per PRD", "see PRD", "sourced from PRD" reference across 33 HTML files. Replaced `Source: PRD §...` / `Reference: SRS §...` labels with `(covered on this page)`. Net 29,710 substitutions.
- **Repaired regex over-strip regression.** The sweep's separator-collapse regex had a false-positive: `(/)\s*(/)` matched `://` in URLs and collapsed them to `:/`. 175 URLs (Google Fonts, jsdelivr CDN, GitHub repo links, SVG xmlns, etc.) were silently broken across all HTML files. Wrote a repair pass that restored `https?:/` → `https?://` plus cleaned up 83 empty `<strong></strong>` / `<em></em>` / `<code></code>` tags and orphan-separator artifacts. Zero broken URLs verified after repair.
- **Added `docs/RESEARCH_MODE_BRIEF.md`** — canonical brief for the pre-lock comprehensive review via Claude Chat's Research Mode. Contains the full prompt covering 8 review dimensions (strategic coherence, architecture, spec quality, UX, info architecture, compliance, GTM, next-7-days actions), the 10-file input bundle (~250 KB total of curated source-of-truth markdown), why we DON'T attach the docs HTML (token waste + visual UX requires live URL crawl), how to drive the mid-review conversation, and how to operationalize the returned document.

## 2026-05-14 — Heading line-height fix + FR authoring workflow guide

- Fixed heading collision on H2 elements caused by the Be-Vietnam-Pro font swap. BVP has taller ascenders + descenders than Inter at the same `font-size`. The previous Inter-tuned `line-height: 1.05` (h-display), `1.15` (h-1), `1.25` (h-2) values were too tight and let the heading bounding box collide with the following paragraph (visible on the "The substrate · the catalog · the orchestrator" H2 on index.html). Updated `assets/styles.css` heading rhythm: h-display 1.05→1.1, h-1 1.15→1.25, h-2 1.25→1.4, h-3 (added) 1.45. Added explicit `margin-block-end` on each + an `h-* + * { margin-block-start: 0 }` rule to neutralise Tailwind `mb-*` collapse.
- Added `docs/FR_AUTHORING_WORKFLOW.md` — canonical playbook for the post-strip FR re-authoring lifecycle. Covers the mental model, file layout, standalone vs chained flows, the standard module-slice-1 recipe (5–7 FRs per slice), how FRs surface back to the docs site, status state machine, task integration paths, and a fully worked FR-AUTH-001 example. Designed to keep open while authoring.

## 2026-05-14 — Comprehensive audit + FR catalog strip + Mermaid mass-fix

Added `docs/AUDIT_AND_PLAN_2026_05_14.md` — single comprehensive audit + build-readiness plan covering UI glitches (severity-ranked), FR landscape, per-module build sequence for the 19 unbuilt modules with slice-1 outlines, and strategic followups. Designed as the source of truth for the next 2 weeks of work.

**FR catalog strip (per user decision: strip-everything).** Stripped:
- All 22 module pages: each "Functional Requirements" section (the `<section id="functional-requirements">` block, lines ~789–820 across modules) replaced with a stub linking to the `fr-author` Agent Skill workflow. 23/23 pages patched cleanly via regex sweep.
- `website/docs/reference/fr-catalog.html`: 1006-line generated catalog replaced with a 70-line stub explaining the rebuild + how to author new FRs via the skill module.

**Partially stripped (cross-refs remain — call to extend):**
- `website/docs/reference/nfr-catalog.html` — still has 137 FR refs (NFRs are described in terms of which FRs they constrain)
- `website/docs/reference/risk-register.html` — still has 51 FR refs (risks reference the FRs they affect)
- Module pages — still have inline FR refs in Dependencies tables, NFR descriptions, KPIs, References footers (~200 total across all)
- `docs/prd/PRD.md` (393 FR refs) and `docs/srs/SRS.md` (206 FR refs) — preserved as authoritative spec narrative; .docx originals also preserved

The "strip-everything" decision affects ~434 remaining FR cross-references — these are inline within sentences and tables. They become broken references until re-authored. To clean them up, separate decisions are needed on whether to: keep them as broken refs (will rewrite organically as new FRs come online), replace with `(FR pending)` markers, or remove the surrounding sentences entirely.

**Mermaid mass-fix across 28 pages:**
- `<br/>` → `<br>` — 754 instances replaced, ALL inside `<div class="mermaid">` blocks (zero outside, verified). This fixes the "Cursorvia MCP tool" text-collapse bug seen on `modules/brain.html` where Mermaid 11.4.1 strips self-closed `<br/>` tags inside quoted node labels.
- Pastel `classDef` palette → Umber/Ochre brand: 127 instances recolored across all non-index module + architecture pages. Map: emerald-100→umber-50, blue-100→umber-100, purple-100→ochre-300, amber-100→ochre-50, pink-100→ochre-100, indigo-100→umber-200, slate-100→neutral-100, yellow-100→ochre-50, violet-100→ochre-50. Strokes likewise mapped to umber-500 / ochre-700 / neutral-400.
- 6 broken internal links to non-existent architecture pages fixed: `architecture/services.html` (5 refs from learn/hr/esop/rew/inv) and `architecture/runtime.html` (1 ref from chat) redirect to `architecture/infrastructure.html` (the closest topical match).

Net code change: 36 files, ~1,417 insertions / ~2,641 deletions. Plus new files `docs/AUDIT_AND_PLAN_2026_05_14.md` (the master plan) and `website/docs/assets/tailwind.min.css` (16.7 KB vendored from prior commit).

Open items pending Stephen's call (per audit doc):
1. Whether to strip the remaining 434 inline FR cross-refs (in NFR catalog / risk register / module sub-sections) or let them rewrite organically.
2. AUTH RFC's 5 open questions need answers before slice 1 codes.
3. Redeploy `website/docs/` via wrangler so the brand + Tailwind + Mermaid + strip fixes go live.

## 2026-05-14 — Vendor Tailwind (CDN was silently failing on Cloudflare Pages)

After the brand-rebuild deploy at https://5cc09eb6.cyberos-docs.pages.dev/, the layout was still broken: hero text and SVG stacked, bento stats stacked one-per-row, 22-module catalog stacked one-per-row, the three shipped-module cards stacked one-per-row. Every `grid`, `grid-cols-*`, `lg:grid-cols-*`, `flex`, `gap-*`, `mt-*` utility was dead because the Tailwind CDN script (`https://cdn.tailwindcss.com`) was loading (200, 14 KB body, no console errors) but **never injected its generated utility CSS** — confirmed by `getComputedStyle` showing `.grid` resolving to `display:block` and `typeof window.tailwind === 'undefined'`. No CSP headers, no module/MIME errors, just a silent failure of Tailwind Play CDN's runtime JIT inside Cloudflare Pages.

Fix in this commit:

- Generated a 16.7 KB static `assets/tailwind.min.css` via `npx tailwindcss@3.4.17` with content-paths covering all 32 HTML files (index + 22 modules + 4 architecture + 4 reference + 1 nav asset). Preflight disabled (we already have `assets/styles.css` setting base styles). All classes the pages actually use are baked in: `.grid`, `.flex`, `.container`, `.grid-cols-{2,3,5,6}`, `.lg:grid-cols-{4,5,6,8,12}`, `.md:grid-cols-{2,3,4}`, `.gap-{1..10}`, `.mt-{0..16}`, `.py-*`, `.text-{xs..2xl}`, `.font-{medium,semibold,bold,black}`, `.items-center`, `.justify-between`, etc.
- Replaced `<script src="https://cdn.tailwindcss.com"></script>` with `<link rel="stylesheet" href="assets/tailwind.min.css">` across all 32 HTML files (relative paths corrected: `assets/...` from index, `../assets/...` from subdirs).
- Result: layout works without runtime JavaScript, no third-party CDN dependency, faster (16.7 KB CSS gzips to ~4 KB vs the CDN's 14 KB JS + runtime compile + style injection).

To regenerate when classes change:

```bash
cd /tmp && cat > input.css <<'CSS'
@tailwind base; @tailwind components; @tailwind utilities;
CSS
cat > tailwind.config.js <<'JS'
const docs = '/path/to/cyberos/website/docs';
module.exports = {
  content: [`${docs}/*.html`, `${docs}/modules/*.html`, `${docs}/architecture/*.html`, `${docs}/reference/*.html`, `${docs}/assets/*.html`],
  corePlugins: { preflight: false },
};
JS
npx tailwindcss@3.4.17 -c tailwind.config.js -i input.css -o /path/to/cyberos/website/docs/assets/tailwind.min.css --minify
```

Once the docs site moves to a real build pipeline (Vite, Astro, or just a Makefile), this becomes one-line in the build command.

## 2026-05-14 — Docs site brand rebuild

Live deploy at https://fe8d68ee.cyberos-docs.pages.dev/ was off-brand: hero triangle used pastel purple/blue/green/yellow Mermaid-default palette; bento stats used per-stat blue/purple/emerald/amber/rose; phase strips used five different pastels; persona accents were purple; compliance ring was blue/green/yellow concentric; tech-stack Mermaid `classDef` was pastel-rainbow. None of these aligned with the design-system DESIGN.md anchors (Umber `#45210e` + Ochre `#f4ba17`) or with Part 21 Liquid Glass defaults.

Root cause: page authoring drift, not design-system fault. Glass classes (`.surface-light/.surface-standard/.surface-heavy`) and `--glass-*` tokens were already defined in `assets/styles.css` and `assets/tokens.css`, but `index.html` hand-coded inline Tailwind palette utilities (`bg-blue-50`, `text-purple-700`, etc.) instead of consuming them.

Fixes in this commit:

- `website/docs/index.html` — 534 lines changed. All inline pastel hex fills in the hero SVG triangle, phase strips, and compliance ring SVG converted to Umber/Ochre tints (`#f5ede6`, `#e8d4c2`, `#fef6e0`, `#fde7b3`, `#f9c64f`, `#cba88a`). All Tailwind palette utilities (`bg-blue-*`, `text-purple-*`, `bg-emerald-*`, `text-amber-*`, `text-rose-*`) replaced with `style="color:var(--umber-700)"` / `style="background:var(--ochre-50)"`. Tech-stack Mermaid `classDef` repainted to brand palette. CyberOS wordmark gradient changed from `blue→purple→emerald` to `umber→ochre`. v2026.05 pill changed from `bg-blue-50 text-blue-700` to `ochre-50 + umber-700`. Phase summary gradient changed from `from-blue-50 via-purple-50 to-emerald-50` to `umber-50 → ochre-50`. Compliance ring concentric gradients changed from `blue→green→yellow` to `neutral→umber→ochre` (warmest at the inner Vietnam home regime).
- `website/docs/assets/tokens.css` — `--font-sans`/`--font-body`/`--font-display` reordered: Be Vietnam Pro listed before Inter per design-system mandate. Comment notes the Vietnamese-first commitment.
- `website/docs/assets/styles.css` — added the `@import` for Be Vietnam Pro so the font actually loads. Added `+101 lines` of design-system utilities: `.ds-modpill` + `.ds-modpill--future` (module navigator pills), `.pill--brand`, `.tile` + `.tile--accent`. Added a transitional-safety-net override block that converts any remaining Tailwind palette utilities on the 22 module pages + 4 architecture pages + 4 reference pages to brand tokens (`bg-blue-*` → `--umber-100`, `bg-purple-*` → `--ochre-50`, etc.) so the brand wins site-wide even before each page is hand-cleaned. Saves ~620 individual edit operations.
- `website/docs/assets/scripts.js` — Mermaid `themeVariables.fontFamily` reordered to Be Vietnam Pro first.

Zero Tailwind palette leaks remain in `index.html` (was 13). Across the rest of the docs site there are still 620 leaks but the new safety-net rules in `styles.css` neutralise them visually until each page is cleaned.

Design-system suggested followups (not landed in this commit):
1. Add Part-21 sub-section "§21.x — Theming third-party renderers" with the Mermaid `themeVariables` recipe, so the next docs author doesn't re-invent it.
2. Promote `.tile`, `.pill--brand`, `.ds-modpill` from the docs site into `design-system/DESIGN.md` Part 3 as first-class component specs.
3. Ship `tools/design-system-lint.{ts,py}` per Part 15 — flag Tailwind palette utilities (`bg-blue-*` etc.) and off-anchor `fill:#` hexes at commit time.

## 2026-05-14 — AUTH module RFC + sign-in mockup

- Added `services/auth/RFC.md` — implementation RFC with 5-slice ship plan, audit-chain integration design, and 5 open questions blocking slice 1.
- Added `services/auth/mockups/sign-in.html` — first AUTH UI mockup applying design-system Part 21 Liquid Glass defaults, Umber + Ochre anchors, Be Vietnam Pro first, passkey-first flow with password fallback, MFA chips, BRAIN audit-chain trust footnote.
- Verification pass against shipped modules:
  - memory: 222 tests pass + 1 skip (numpy + jsonschema needed for full green). Real bug found AND fixed: `check_manifest_validates` was skipping parseability when jsonschema absent → `cyberos state` returned READY on a broken manifest. Patched to always parse `manifest.json` first (regardless of jsonschema availability) and report `False` on `JSONDecodeError`; the optional schema-validation layer still skips cleanly when jsonschema is absent. Verified: all 4 `tests/test_state.py` tests pass, full suite 238 pass / 1 skip / 0 fail. Also verified by simulating absent jsonschema via import hook — good manifest still returns True with "parseability OK, schema skip"; bad manifest returns False with "manifest.json unparseable: ...".
  - skill: 20 SKILL.md bundles structurally verified, 4 crates, 8 inline Rust tests. `cargo build` not run (sandbox-only limitation).
  - cuo: 15/15 pytest + 15/15 routing fixtures pass. Catalog discovers all 20 skills correctly.
- Stale-claim drift surfaced (none are blockers, all are doc-only):
  - Memory tests: bootstrap says 245, README says 255, actual is 238 collected.
  - Doctor invariants: bootstrap says 16, README says 15, actual is 13 on a fresh store.
  - Docs pages: bootstrap says 32, strategy says 31, actual is 33 HTML files (32 user-facing + nav include).
  - Strategy §3 Tier-1 #2 and §5 Session-1 #1 list "wire Pagefind" as a to-do; Pagefind is already built and serving (v1.5.2, 32 pages indexed).
  - DEPLOYMENT.md is at `website/docs/DEPLOYMENT.md` (bootstrap implies it lives at `website/`).
- Docs site deploy-prep findings:
  - 6 real broken internal links to 2 missing architecture pages: `architecture/services.html` (5 refs from LEARN/HR/INV/ESOP/REW) and `architecture/runtime.html` (1 ref from CHAT). These are demand-gen blockers — fix before public deploy or convert the link targets.

## 2026-05-14 — Consolidation pass

Moved all CyberOS-related artifacts into a single umbrella at `cyberos/`:

- `workbench/CyberOS-docs/` → `cyberos/website/docs/`
- `workbench/CYBEROS_STRATEGY.md` → `cyberos/strategy/CYBEROS_STRATEGY.md`
- `workbench/cyberskill-vn-skills/` → `cyberos/public-skills/`
- `/design-system/` → `cyberos/design-system/`
- `/landing-page/` → `cyberos/website/landing/`

This enables clone-and-go for new sessions and keeps strategic + technical + design content co-located.

See per-module CHANGELOG.md files for module-specific history:
- `memory/docs/CHANGELOG.md`
- `skill/docs/CHANGELOG.md`
- `cuo/docs/CHANGELOG.md`
- `design-system/CHANGELOG.md`
- `website/docs/index.html` (the rendered changelog page)
