---
title: "Founder Daily Flow (UC-FLOW-001) — morning login → CUO digest → triage → sprint review with auto-prepared agenda"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P0 / 2026-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Wire the **Founder Daily Flow** (UC-FLOW-001 from PRD §10.1 and §17.5) end-to-end as the canonical S0-5 demo and as the daily-driver path the founder uses from P0 onward. Morning login → CUO daily digest from BRAIN → review of pending OBS alerts → CHAT triage → kick off PROJ sprint review with an auto-prepared agenda. The flow stitches together AUTH (sign-in), GENIE (digest + Notify cards from CUO/CEO + CUO/COO + CUO/CTO skills), BRAIN (citations), CHAT (recent unread + thread summaries), PROJ (open issues + cycle status from the freshly-shipped FR-PROJ-001 in batch-04 — for P0 the PROJ portion is **stubbed with synthetic data** because PROJ ships in P1; the integration seam is exercised so P1 makes a clean cutover), OBS (pending alerts), and TIME (calendar conflicts). The S0-5 risk-gate (PRD §17.5): **total time from login to first action ≤ 90 seconds** and **founder daily review ≤ 10 minutes** (NFR-USAB-004); two consecutive demos exceeding the budget is sprint-blocking.

## Problem

The PRD's strategic goal G8 (PRD §4.1) is the founder-cognitive-load reduction: "CUO reduces founder hours-per-week spent on ops/admin/exec-thinking by ≥ 40% by P1 exit, measured by self-reported time logs validated against TIME entries." That goal is concrete and load-bearing — the platform either reduces the founder's daily admin time or the dogfooding bet collapses.

The Daily Flow is the surface that produces or fails to produce that reduction. A flow whose login takes 30 seconds and whose digest renders four irrelevant cards is *worse* than not having the flow — the founder will route around it. A flow whose login is 5 seconds, whose digest cites four high-confidence facts the founder did not yet know, and whose first card is the one most likely to need action — that flow earns the founder's trust and gets used daily.

The PRD §10.1 narrative is specific:

- 07:30 ICT — open browser; Genie panel shows weekly digest from CEO-skill + CFO-skill cashflow snapshot + a Question card from CHRO-skill ("Member X has not logged time in 3 days; want me to ask?"). [P0 ships CEO + COO + CTO; CFO + CHRO are P2; their cards are stubbed in P0 with placeholder text indicating "skill not yet shipped".]
- 09:00 ICT — standup channel: founder types `@genie summarise yesterday for the team`. CUO produces a CHAT-grounded paragraph with citations, posted as a thread.
- 11:00 ICT — Account Manager ping; founder asks Genie "what's the typical hold-up at proposal stage?"; CRO-skill aggregates BRAIN signals (CRO is P2; in P0 the CEO skill answers with a caveat).

S0-5 sprint exit (PRD §17.5) demands the demo passes with the time-on-task NFR.

## Proposed Solution

The shape of the answer is a coordinated layout in the host shell + a `cyberos-daily-flow` orchestrator that pre-computes the morning bundle + an authored "Founder Daily" route at `/today` + integration with the PROJ stub.

**The pre-computed morning bundle.** A small service `cyberos-daily-bundler` runs at 07:00 ICT every weekday for every Member (configurable per-Member start time and timezone). It pre-computes the morning bundle so login does not block on synchronous LLM calls:

1. CUO/CEO weekly + daily digest: a 5–7 sentence summary citing 4–6 BRAIN facts. Includes "yesterday's key events", "today's commitments", "watch items".
2. OBS alert summary: count + severity by class for the last 24 hours.
3. CHAT unread: count by channel with the top 3 most-likely-relevant threads ranked by CUO/COO Notify-relevance score.
4. PROJ status (stub in P0; real in P1): cycle-progress, blockers, your-assigned-tasks count.
5. TIME calendar conflicts: any double-bookings or back-to-back-meetings flagged.
6. Pending Questions and Reviews from CUO (FR-GENIE-002).
7. Memory pending: count of unresolved BRAIN conflicts (FR-BRAIN-CONFLICT-001).

The bundle is cached in `genie.daily_bundle{tenant_id, member_id, computed_for_date}`; expiry 24 hours; recomputed on Member-initiated refresh button or on significant signal change (a sev-0 alert during the night re-runs the bundle).

**The `/today` route.** A Module-Federation remote authored as part of GENIE shipping at `/today`. Layout:

- **Top hero.** Single-paragraph CUO/CEO digest with inline citation chips (each chip is clickable → opens Layer 3 source paragraph in a side-panel).
- **Action lane.** Three-column grid: "Now" (≤ 3 highest-priority Notify or Question cards from any skill), "Today" (≤ 6 medium-priority), "Watch" (≤ 6 low-priority informational).
- **Adjacent panes.** OBS-alert mini-dashboard (sev-0 / sev-1 / sev-2 counts with click-through). CHAT recent-unread mini-list. PROJ cycle-progress bar. TIME conflicts banner.
- **Sprint-review kicker.** A button "Open sprint review" that creates a pre-prepared agenda from PROJ + BRAIN — for the founder's weekly cadence. The agenda is itself a CUO Review (FR-GENIE-002).

**The login → first-action path.**

1. Founder navigates to `app.cyberos.world` and authenticates via passkey (FR-AUTH-001). Target: ≤ 5 seconds for the auth ceremony.
2. Host shell mounts the GENIE remote at `/today` (Module-Federation lazy-load; ≤ 1.5 s).
3. The pre-computed morning bundle hydrates the page in ≤ 500 ms (the bundle is already in `genie.daily_bundle`; rendering is a single GraphQL query).
4. The first interactive moment (the "Now" lane is fully rendered with at least one actionable card) is ≤ 3 seconds after login.
5. The founder's first action (click an action chip, accept a Notify, open a Question) is ≤ 90 seconds after login. The risk-gate measurement is a trace span tagged `cyberos.flow.daily.first_action`.

**The 09:00 standup `@genie summarise yesterday for the team`.** Wire the CHAT `@genie` mention from FR-CHAT-001 to the CUO/COO skill's daily-summary tool. The summary draws from `cyberos.{tenant}.chat.message.posted` events from the prior day plus the prior day's PROJ updates (stubbed in P0). Output is a short paragraph cited with deep-links back to specific CHAT messages. Posted as a thread reply in the channel where invoked.

**The 11:00 "what's the typical hold-up at proposal stage?".** This is a graph query over BRAIN's AGE layer (FR-BRAIN-002) plus the CUO/CEO skill's narrative response. The query: walk from `Engagement` nodes with `status: "stuck-at-proposal"` to their related `Communication` and `Decision` nodes; aggregate by predicate; surface the three most common patterns. The persona narrates with citations. In P0, with no live PROJ data, this query relies on the synthetic seed corpus the team writes during S0-5; in P1 the corpus grows organically.

**Founder kill-switch shortcut.** The `/today` page has a small "Pause Genie" button in the top-right (FR-GENIE-002 §"Founder kill-switch") so the founder does not need to memorise a CLI command for the rare case Genie is misbehaving on a critical day.

**Per-Member adaptation.** The Daily Flow page is rendered for *every* Member, not just the founder, but the bundle is per-role. An Engineering Lead's bundle prioritises OBS alerts + tech-debt PRs; a HR/Ops Lead's bundle prioritises pending HR tickets + onboarding. The skill-tag of each Notify card determines which Members see it. The founder's view in P0 sets the design baseline; the role-tuned bundles ship across P1–P2 as those roles' skills land.

**Voice-input fast path.** Voice-to-text on the `@genie` field uses the same Whisper-large-v3 (FR-CHAT-001) so the founder can dictate the 09:00 summarise command rather than type it. Latency budget for voice → action: p95 ≤ 4 s for a 10-second utterance.

**Acceptance-rate exposure.** The `/today` page footer shows "Genie acceptance: 47% (7-day rolling)" — a small number that calibrates the founder's trust real-time. Below 40%, the number turns yellow and links to the persona-quality dashboard. This is the calibration loop closing visibly.

## Alternatives Considered

- **Synchronous LLM calls on page load.** Rejected: 6+ seconds of latency on login fails NFR-USAB-004 (≤ 10 minutes daily review starts with ≤ 90 s to first action).
- **A fully customisable widget grid (Notion-style).** Rejected for P0: too much UI surface to author and tune in S0-5; the fixed three-column layout is the floor; customisation is a P2 deliverable.
- **A CHAT-bot-only flow (no separate page).** Rejected: the founder navigates between several signals (CHAT, OBS, PROJ, BRAIN) and a single panel cannot show all of them well; the dedicated page is the right canvas.
- **Pre-compute the bundle at midnight regardless of timezone.** Rejected: per-Member start-time + timezone means the bundle is freshest when the user actually opens it; midnight bundles would be stale by the founder's 07:30 ICT login.
- **Email digest instead of in-app page.** Rejected: the platform's own panel must be the home for the daily flow; an email digest may follow as an alternative surface (P1 EMAIL FR cluster).

## Success Metrics

- **Primary metric.** S0-5 demo passes the four sub-criteria from PRD §17.5: (1) founder opens host shell at 09:00; (2) daily digest appears within 5 seconds, citing 4–6 BRAIN sources; (3) two pending Notify items are addressable in one click each; (4) sprint review opens with auto-prepared agenda from PROJ (stub) + BRAIN; (5) total time from login to first action ≤ 90 seconds. Two consecutive demos exceeding 90 s is sprint-blocking.
- **NFR-USAB-004.** Median founder-daily-review session ≤ 10 minutes for the 14-day P0-exit observation window.
- **G8 reduction.** Founder hours-per-week on ops/admin/exec-thinking trending toward the 40% reduction by P1 exit; P0 baselines and P1 measures.
- **Acceptance metric.** Daily Flow card-acceptance rate ≥ 40% combined across Notify / Question / Review (PRD §14.2.3 P1 gate; P0 establishes the baseline).

## Scope

**In-scope (S0-5).**
- `cyberos-daily-bundler` service running at the configured per-Member start time.
- `genie.daily_bundle` table + GraphQL query + cache invalidation on signal change.
- The `/today` route in the GENIE remote with hero, three-column action lane, adjacent panes, sprint-review kicker.
- CHAT `@genie summarise yesterday for the team` integration in CHAT (the CUO/COO daily-summary tool).
- AGE-graph query for the "typical hold-up at proposal stage" answer (P0 uses the synthetic seed corpus authored by the team).
- Voice-input fast path on the `@genie` field.
- Per-Member start-time + timezone preference in `/auth/account`.
- Founder kill-switch button in the page header.
- Acceptance-rate footer chip.
- The PROJ stub (synthetic data) wired through the same GraphQL surface PROJ will populate at P1.
- Trace-span tags `cyberos.flow.daily.{login_to_first_paint, first_paint_to_first_action, daily_review_total}`.
- Audit integration in scope `genie.daily_flow.{tenant}`.

**Out-of-scope (deferred).**
- Real PROJ data (P1 — FR-PROJ-001 ships the data feed).
- CFO + CHRO + CSO + CRO daily-bundle elements (P2; placeholder text in P0).
- Per-Member layout customisation (P2).
- Mobile clients (P3).
- Public Daily Digest URL (P3 PORTAL).

## Dependencies

- FR-INFRA-001 (host shell + Postgres + NATS).
- FR-AUTH-001 / FR-AUTH-002 (login + audit).
- FR-AI-001 (digest synthesis + voice transcription routing).
- FR-MCP-001 (the action chips invoke confirmable tool calls).
- FR-BRAIN-001 / FR-BRAIN-002 / FR-BRAIN-003 (citation provenance).
- FR-BRAIN-NLCRUD-001 (the action chips that involve memory writes route through NLCRUD).
- FR-CHAT-001 (`@genie` mention; voice transcription).
- FR-GENIE-001 + FR-GENIE-002 (Notify + Question + Review modes).
- FR-OBS-001 / FR-OBS-002 (alert mini-dashboard).
- Compliance: EU AI Act Article 50 (every CUO-derived element on the page renders the disclosure chip with persona-version).
- Locked decisions referenced: DEC-061 (pre-computed daily bundle is the latency mechanism; synchronous LLM on page load is forbidden).

## AI Risk Assessment

The Daily Flow is the founder's primary AI-mediated surface. EU AI Act risk class: `limited`.

### Data Sources

The bundle composes outputs from CUO skills (CEO/COO/CTO) running through the AI Gateway with per-tenant residency and persona-version stamping. Citations are anchored to per-tenant BRAIN content. No third-party training data, no cross-tenant leakage.

### Human Oversight

- Every action chip on the page invokes a confirmable destructive operation through MCP (no chip auto-acts).
- Notify cards retain the same dismiss / accept floor.
- Question and Review cards keep their respective lifecycles.
- The kill-switch is on the page header for the rare case the persona is misbehaving.
- The acceptance-rate footer is the visible calibration loop.

### Failure Modes

- **Bundle pre-compute fails or stale.** The page falls back to a "computing now" spinner that runs the bundle synchronously (with a < 6 s p95 latency target); if even that fails, the page renders the deterministic non-AI sections (CHAT unread count, OBS alert count) only, with a banner "Genie summary unavailable — please retry".
- **Citation drift.** Caught by the BRAIN regression tests; the daily-flow eval suite includes a citation-correctness case set.
- **AGE-graph query slow.** Falls back to the vector retrieval path; latency budget enforced; degraded answer is still cited.
- **Voice transcription error.** The voice-input field shows the transcription before submitting; the user can edit before pressing enter.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted page layout, the bundle pre-compute pipeline, the login-to-first-action latency budget, failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; the founder's daily routine details inform the per-role bundles in P1+.
