---
title: "CRM — AI features (CUO/CRO next-action drafter, deal-aware reply suggestions, account-health, pipeline forecast confidence-bands, \"typical hold-up\" insight)"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Layer the AI-native CRM features on top of FR-CRM-001/002. **CUO/CRO next-action drafter** suggests the highest-value next step per active deal ("Acme — proposal sent 12 days ago; suggest a check-in email referencing the Q3 milestone"); **deal-aware reply suggestions** consumed by EMAIL FR-006 (the third suggestion when a recipient is in an active deal); **account-health derivation** (green/yellow/red derived from signal sentiment + activity recency + deal pipeline state); **pipeline forecast confidence-bands** (bands not single-value scores; PRD §14.2.1 "no scoring of people" preserves the constraint); the **"what's the typical hold-up at proposal stage?" insight** running graph queries over BRAIN's AGE layer (PRD §10.1 11:00 ICT example); and the **stale-deal Notify** ("Acme deal stuck at proposal 21 days; want me to draft a check-in?"). All AI surfaces operate through the CUO/CRO skill, render the EU AI Act Article 50 disclosure, respect persona-scope contracts, and never auto-act on deal stages.

## Problem

The PRD's CRM bet is that AI-driven next-action surfacing replaces the Account Manager's mental model of "what should I do next" with a structured, cited recommendation. PRD §9.5.3 + §9.4.3 + §10.1 + §14.2.3 commit to this — the P1 → P2 exit gate explicitly measures "Genie-drafted next-actions accepted by sales rep at ≥40%".

Three failure modes a small team must avoid:

- **Forgotten deals.** A deal at proposal stage with no follow-up for 21 days quietly dies; without surfacing, the Account Manager learns at month-close.
- **Generic reply suggestions.** EMAIL's three suggestions (FR-EMAIL-004) treat all replies the same; deal-aware context (engagement stage, prior commitments, win-criteria) is the differentiator.
- **Pattern blindness.** "What's the typical hold-up at proposal stage across Acme + Beta + Gamma deals over the last 6 months?" is a question only a graph + LLM combination can answer; without it, the Account Manager works case-by-case.

## Proposed Solution

Five integrated surfaces, all via the CUO/CRO skill (the Chief Revenue Officer specialist), through the AI Gateway with persona-scope-contract-enforced retrieval against CRM + EMAIL + PROJ + BRAIN.

**1. Next-action drafter.**

- **Trigger.** Per-Account-Manager 07:30 ICT pre-compute job (matches FR-GENIE-003 daily-flow pattern). The Genie panel "Today" tab and the `/crm/my` view show ranked next-actions per deal.
- **Inputs.** Each open deal's stage history, time-in-stage, last activity, recent signals, contact roles, BRAIN-derived account context (decision-maker map; prior similar deals).
- **Output.** Per deal: a one-sentence rationale + a one-click action chip ("Draft check-in email", "Schedule discovery call", "Move to negotiation", "Mark stale → discuss with founder", "Mark won → create Engagement").
- **Latency.** Pre-compute → instant render; on-demand recompute ≤ 4 s p95.

**2. Deal-aware reply suggestions.**

- **Surface.** EMAIL composer (FR-EMAIL-004 §"Suggested replies") — when the recipient is in an active deal, CUO/CRO produces a third deal-aware suggestion alongside the two generic ones from CUO/COO.
- **Inputs.** Active deal context + win/loss notes from prior similar deals + BRAIN community summary about the account.
- **Output.** A suggestion calibrated to the deal stage:
  - `discovery`: a question-rich reply that surfaces buying criteria.
  - `proposal`: a follow-up that references specific proposal items + timing.
  - `negotiation`: a defer-to-human suggestion ("This stage typically benefits from a synchronous call; do you want me to suggest available times?").
  - `closed_won`: a relationship-maintenance reply.
- **Latency.** Streamed alongside the other two suggestions; ≤ 1.4 s p95 for the first card.

**3. Account-health derivation.**

- **Trigger.** Daily 06:00 ICT recompute per active account.
- **Algorithm.** A small explainable scoring function (not a black-box model):
  - Recent signal sentiment (rolling 30 days, weighted by recency).
  - Activity recency vs. expected cadence (per account based on prior history).
  - Open-deal stage progression (deals advancing vs. stuck).
  - Open-blocker presence (linked PROJ blockers from FR-PROJ-007).
- **Output.** `crm.account.health_score` set to `green` / `yellow` / `red` with a JSON `health_breakdown` showing each input's contribution.
- **No people-scoring.** The function operates on signals + activities + stages — never on individual contacts' behaviour. PRD §14.2.1 invariant preserved.

**4. Pipeline forecast confidence-bands.**

- **Surface.** A `crmPipelineForecast` GraphQL field returning per-stage + per-Member + per-region totals with confidence bands (low / medium / high) rather than single probability values.
- **Algorithm.** Bands derived from rolling 12-month historical conversion rate per stage per owner per account-region; explicit caveats around small-sample-size bands.
- **No automated weighting on individual deal probabilities.** A single deal's `probability_band` is set by the human (Account Manager); the forecast aggregates these without overriding.

**5. "Typical hold-up" insight (PRD §10.1 11:00 ICT example).**

- **Surface.** Genie panel natural-language query input.
- **Example queries.**
  - "What's the typical hold-up at proposal stage?"
  - "Which competitors won most often in the last 6 months?"
  - "Which accounts have signals trending negative?"
  - "What's the longest-stuck deal?"
- **Path.**
  1. CUO/CRO extracts intent → query specification.
  2. Query runs over CRM data + AGE-graph patterns (e.g. for "typical hold-up at proposal stage", walk all closed_lost deals with prior `proposal_sent`; aggregate `loss_reason` by frequency; return top patterns).
  3. Sonnet narrates the result with citations to specific deals + activities.
- **Latency.** ≤ 6 s p95.

**6. Stale-deal Notify.**

- **Trigger.** Every active deal with no activity for > 14 days (configurable per stage; default 14d at every stage except `negotiation`/`closed_*` where it's 7d).
- **Surface.** Notify card to the deal's primary owner: "Deal `Acme – Onboarding` stuck at `proposal` 21 days. Suggest: draft a check-in email."
- **Action chips.** "Draft check-in email" (one-click runs FR-EMAIL-006 deal-aware suggestion + opens composer), "Schedule a call", "Mark stale → discuss with founder", "Move to closed-lost" (opens close-out modal).
- **Frequency limit.** Max 1 stale-deal Notify per deal per 7 days to prevent fatigue.

**Persona scope contract.**

CUO/CRO declares:
```
tools_allowed:
  - cyberos.crm.list_*
  - cyberos.crm.get_*
  - cyberos.crm.search
  - cyberos.crm.pipeline_forecast
  - cyberos.crm.next_action_for_deal
  - cyberos.crm.deal_insight
  - cyberos.crm.account_health_explain
  - cyberos.crm.list_stale_deals
  - cyberos.email.suggest_replies
  - cyberos.email.compose_in_locale
  - cyberos.brain.search
  - cyberos.proj.engagement_dashboard
  - cyberos.genie.notify
  - cyberos.genie.draft_review

tools_forbidden_explicit:
  - cyberos.crm.create_*
  - cyberos.crm.update_*
  - cyberos.crm.transition_deal_stage
  - cyberos.crm.close_deal_*
  - cyberos.crm.delete_*
  - cyberos.crm.set_*
  - cyberos.email.send_message
```

**Account-health Notify suppressions.**

When `health_score` flips from green → yellow or yellow → red, a Notify card lands with the explanation breakdown ("Sentiment trending negative; 2 stale deals; 1 PROJ blocker"). The Account Manager + Founder are recipients. Flips back to green also Notify (positive signal).

**MCP tool surface.**

- `cyberos.crm.next_action_for_deal(deal_id)` — read.
- `cyberos.crm.next_action_for_owner(owner_id, top_n)` — read.
- `cyberos.crm.deal_insight(query)` — read.
- `cyberos.crm.account_health_explain(account_id)` — read.
- `cyberos.crm.list_stale_deals(owner_id?, stage?)` — read.
- `cyberos.crm.pipeline_forecast(stage?, owner_id?, region?)` — read.
- `cyberos.crm.suggest_close_outcome(deal_id)` — read; returns "ready to close-won?" / "trending toward close-lost?" assessment.

CUO/CRO uses these internally; the human commits any state change.

## Alternatives Considered

- **Black-box deal-probability model that scores each deal.** Rejected: PRD §14.2.1 explicitly excludes people-scoring; the explainable bands pattern is the floor. Black-box scoring also fails GDPR Article 22.
- **Auto-stage-transition on Notify acceptance.** Rejected: stage transitions are deal-defining; human-in-the-loop floor.
- **Skip account-health derivation; rely on Account Manager intuition.** Rejected: the health surface is the founder's surface for early-warning across the team's accounts.
- **Use a hosted CRM-AI service (Drift, People.ai, Gong).** Rejected: residency + Engagement-aware retrieval cannot be enforced.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate: Genie-drafted next-action acceptance rate ≥ 40% on a 7-day rolling window for at least 14 consecutive days; ≥ 5 active accounts; ≥ 10 deals.
- **Quality metric.** Citation correctness on insight answers ≥ 95% (sampled review by founder + Account Manager).
- **Health-score accuracy.** Quarterly review by founder; ≥ 80% of red flips correspond to a real account issue (true-positive rate).
- **Stale-deal Notify acceptance.** ≥ 50% of stale-deal Notifies result in a logged follow-up activity within 48 hours.
- **Latency NFR.** Per-deal next-action ≤ 4 s p95; insight query ≤ 6 s p95.

## Scope

**In-scope.**
- The five AI surfaces.
- Daily 06:00 ICT account-health recompute job.
- Daily 07:30 ICT next-action pre-compute job.
- Stale-deal Notify scanner.
- Pipeline-forecast confidence-band derivation.
- The seven MCP tools.
- Persona scope contract for CUO/CRO.
- Audit integration in scope `crm.ai.{tenant}`.
- OBS dashboard panels: next-action acceptance, health-score distribution, insight-query thumbs-up rate.

**Out-of-scope (deferred).**
- Auto-create Engagement on close-won (FR-PROJ-007 + FR-CRM-002 ship the human-in-the-loop suggestion).
- Per-deal probability model beyond bands (P3 if explainable + auditable).
- Web-tracking visitor signals (P3+).
- Voice-input "tell me my next action" (P3 mobile).
- Cross-tenant insight (forbidden by design).

## Dependencies

- FR-CRM-001 / FR-CRM-002.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-EMAIL-004 / FR-EMAIL-006 (deal-aware reply consumer).
- FR-PROJ-001 / FR-PROJ-007 (Engagement linkage).
- FR-BRAIN-001 / FR-BRAIN-002 / FR-BRAIN-003 (graph-query substrate; community summaries).
- FR-GENIE-001 / FR-GENIE-002 (Notify/Question/Review modes; persona-scope; acceptance metrics).
- FR-OBS-002.
- Compliance: EU AI Act Article 22 (no fully automated decisions on individuals — bands not scores; reversibility; human commit); Article 50 (transparency disclosure on every AI surface); PDPL Decree 13 (CRM data is personal data; per-tenant residency).
- Locked decisions referenced: DEC-151 (CUO/CRO is the CRM AI persona), DEC-152 (confidence bands not single-value probabilities), DEC-153 (no people-scoring).

## AI Risk Assessment

CRM AI surfaces materially shape sales decisions; in some markets, sales decisions intersect employment/discrimination rules. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: CRM + EMAIL + PROJ + BRAIN. No third-party. CUO/CRO runs through the AI Gateway with persona-stamping.

### Human Oversight

- Every state change (deal stage, account status, close outcome) is human-committed.
- Notify cards are dismissable.
- Account-health is informational; never blocking.
- Forecast confidence-bands are explanatory; never automated decision.
- The kill-switch (FR-GENIE-002) silences all CRM AI in 30 seconds.

### Failure Modes

- **Wrong next-action.** Acceptance rate is the calibration signal; below 40% over 7 days the persona auto-pauses (FR-GENIE-002).
- **Health-score false-positive (red on a healthy account).** Mitigation: the explanation breakdown shows what's contributing; the Account Manager can flag a false-positive; the heuristic adjusts.
- **Insight hallucinates a competitor.** Caught by citation-correctness regression; persona-version gating.
- **Stale-deal Notify on legitimately-paused-by-customer deal.** Mitigation: the per-deal `metadata.snooze_stale_until` field; the Account Manager can mute for N days.
- **Forecast bands too wide to be useful.** Mitigation: the explanation makes sample-size constraints visible; the user accepts the uncertainty.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted five-surface architecture, persona scope, MCP tool surface, failure modes.
- **Human review:** `@stephen-cheng` reviewed; CRM-specific eval cases authored at PR-review with the Account Manager.
