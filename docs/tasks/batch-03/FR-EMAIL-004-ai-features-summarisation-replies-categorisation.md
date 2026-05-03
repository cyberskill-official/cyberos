---
title: "EMAIL — AI features: thread summarisation, suggested replies, auto-categorisation, snooze-reminder draft"
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

Layer the AI-native EMAIL features on top of FR-EMAIL-001 + FR-EMAIL-002 + FR-EMAIL-003: **thread summarisation** (a CUO-authored summary of a long thread, BRAIN-friendly so the summary itself becomes a Layer 2 fact); **suggested replies** in the CyberSkill voice with the Design System §3.13 honesty rule applied; **auto-categorisation** (sales / support / personal / spam / newsletter / transactional) per Member with personalisation; **smart snooze suggestions** ("snooze until Monday 09:00 ICT — they typically reply Mondays"); **on-time CUO reminders** for snoozed threads ("Snoozed thread is back: re: Acme proposal — last action: you replied Friday with pricing"); **per-Member preferences** for AI density (off / minimal / default / aggressive). All AI-derived surfaces operate on CaMeL-sanitised outputs only and render the EU AI Act Article 50 transparency disclosure chip with the active persona-version.

## Problem

The Stalwart core + Missive UX put the team on equal footing with Gmail-as-shared-inbox; that is the floor, not the moat. The PRD's bet that CUO is the brand (Bet 2) requires that EMAIL feel materially better than Gmail at the work the team actually does — answering customers in Vietnamese with the right register, triaging ten threads in two minutes, never forgetting a snooze. Without these AI features, EMAIL is parity at best; with them, it is the first surface that earns the founder's trust in CUO at high volume.

PRD §9.4.3 lists the AI-native features explicitly: thread summarisation in BRAIN-friendly format (Notify mode for Member, Review mode for shared inboxes); suggested replies in CyberSkill voice; auto-categorisation; "snooze until Monday" with on-time CUO reminder.

## Proposed Solution

Five integrated surfaces, one Notify/Question/Review model, one persona scope contract.

**Thread summarisation.**

- **Trigger.** A thread crossing 8 messages or 8 KB of accumulated body text auto-generates a summary (Notify mode for personal inbox; Review mode for shared inbox). The Member can also click "Summarise" on any thread regardless of length.
- **Path.** The thread's CaMeL-sanitised facts (FR-EMAIL-003) plus retrieval against BRAIN Layer 2 for context (the customer's prior threads, the active engagement, prior decisions) feed the CUO/COO skill (default for sales / support); CUO/CTO for technical threads; CUO/CEO for strategic correspondence.
- **Output.** A 4–8 sentence summary in the Member's preferred locale (vi-VN by default for VN-team threads; en-US for international) with citations to specific messages in the thread plus relevant BRAIN facts.
- **BRAIN-friendly.** The summary itself is ingested into BRAIN Layer 2 as a `fact` with `subject_uri: thread:<thread-id>` and `predicate: has_summary` so future retrieval can cite the summary rather than re-summarising. The summary is updated when the thread receives a new message that materially changes it (heuristic: ≥ 200-byte addition with semantic-similarity drift > 0.3).
- **Surface.** A collapsible summary card pinned to the top of the thread detail pane, with a "view full thread" toggle to scroll past it. Citation chips in the summary are clickable and scroll the thread to the cited message.

**Suggested replies.**

- **Trigger.** When the Member opens the reply composer, three suggestions are generated in parallel; the first appears within the latency budget (Haiku-class p95 ≤ 1.4 s).
- **Path.** Inputs: the thread's CaMeL-sanitised facts; the Member's recent prior replies to similar threads (style learning, scoped per Member); the recipient's prior emails (tone calibration); BRAIN context about the recipient's role / engagement.
- **Voice.** The CyberSkill voice is encoded in the persona's `SKILL.md` voice block (FR-GENIE-001) — peer-to-peer, direct, no AI-filler ("As an AI", "I hope this finds you well"). The Design System §3.13 voice-honesty rule applies: the suggestion never claims certainty the platform does not have.
- **Vietnamese-aware.** When the recipient is Vietnamese, the suggestion uses appropriate honorifics (`Anh/Chị/Bạn`) per the recipient's prior threads (cached per-recipient via BRAIN). The Vietnamese-style detail is in FR-EMAIL-005.
- **UX.** Three suggestion cards above the composer; clicking inserts the suggestion into the composer (the Member can edit, append, or replace). Each card carries the persona-version chip + a "regenerate" button. Persona acceptance counted in `genie.acceptance_rate{mode: notify, persona_version, intent: email-suggested-reply}`.

**Auto-categorisation.**

- **Categories.** `sales` (new prospect, deal in flight); `support` (existing customer issue); `personal` (non-business); `spam` (already filtered by Stalwart's RBL + ARC; this is the second-pass classifier); `newsletter`; `transactional` (invoices, receipts, calendar invites); `internal` (cyberskill.world to cyberskill.world).
- **Path.** The CaMeL output's `classification` field is the primary signal; a per-Member personalisation overlay learns from the Member's manual recategorisations over time (stored in `email.member_personalisation` with simple counts; no fine-tuning).
- **Effect.** Each Member's personal inbox is auto-foldered by category (a thread with `classification: newsletter` lands in a `Newsletters` virtual mailbox). Shared inboxes are not auto-foldered (their categorisation is for surfacing in dashboards, not routing).
- **Reversibility.** Manual recategorisation is one click; the override is logged and feeds the personalisation overlay; auto-classification can be turned off per Member.

**Smart snooze suggestions.**

- **Trigger.** When a Member opens the snooze dropdown, the default options are "tomorrow 09:00 ICT", "next Monday 09:00 ICT", and three contextual suggestions derived from the recipient's prior reply patterns: "they typically reply on Mondays — snooze until Monday 09:00", "they replied within 4 hours last 5 times — try waiting 1 day", "this thread has been quiet for 7 days — snooze until customer replies".
- **Path.** A small per-recipient model: rolling histogram of `(reply_received_at - sent_at)` from prior threads; confidence threshold ≥ 6 prior replies before showing the suggestion.
- **Trust calibration.** Suggestions carry a confidence chip; below medium confidence the suggestion text reads "I'm not sure of their pattern" and is informational only.

**On-time CUO snooze reminders.**

- **Trigger.** The snooze-until time arrives. CUO/COO emits a Notify-mode card to the snoozer's panel.
- **Card content.** Subject + recipient + last action ("you replied Friday with pricing") + CUO-suggested next step ("draft a follow-up?", "extend snooze 3 days?", "mark resolved?"). The card has three accept-buttons that all route through the destructive-confirmation gate.
- **Reminder grace.** If the Member is offline, the reminder waits up to 4 hours then surfaces on next login; if the Member is in a meeting (TIME module's calendar status, P1+), the reminder waits until the meeting ends.

**Per-Member AI density preference.**

- **Levels.** `off` (no AI surfaces in EMAIL); `minimal` (auto-categorisation only; no suggested replies, no summaries); `default` (everything above); `aggressive` (auto-summary on every thread > 4 messages; pre-drafted reply staged in composer on thread open). The default per Member is `default` for the founder + Account Managers and `minimal` for Members in roles that don't usually do customer email.
- **Storage.** `email.member_preference{member_id, ai_density, locale_default, suggested_reply_count}` table.
- **Effect.** Surfaces respect the level globally; switching level takes effect on next page render.

**Persona scope contract.** The CUO/COO skill (default for EMAIL surfaces) declares `tools_allowed: ["cyberos.email.compose_draft", "cyberos.email.summarise_thread", "cyberos.brain.search", "cyberos.crm.search_*", "cyberos.proj.search_*"]` and `tools_forbidden_explicit: ["cyberos.email.send_message"]`. Sending requires the human's destructive confirmation + step-up auth (FR-EMAIL-001 + FR-AUTH-003); CUO drafts, the human sends.

**MCP tool surface (extends FR-EMAIL-001 + FR-EMAIL-002).**

- `cyberos.email.summarise_thread(thread_id, persona_skill?)` — `destructive: false`; returns the summary + citations; idempotent for ≤ 1h.
- `cyberos.email.suggest_replies(thread_id, count: 3)` — `destructive: false`; returns 3 suggestions.
- `cyberos.email.classify_thread(thread_id)` — `destructive: false`; returns the category.
- `cyberos.email.suggest_snooze(thread_id, recipient?)` — `destructive: false`; returns the contextual suggestions.

CUO uses these tools internally; they are also exposed for any agent to compose richer email-handling workflows.

**Audit integration.** Every AI surface call writes a row in `email.ai.{tenant}` with `surface_kind`, `thread_id`, `persona_version`, `latency_ms`, `tokens`, `cached`. Acceptance / dismissal counted toward `genie.acceptance_rate`.

**Latency budgets (per FR-AI-001 §"Latency budgets").**

- Suggested-reply first card: p95 ≤ 1.4 s (Haiku; cached prefix prompts).
- Thread summarisation: p95 ≤ 6 s (Sonnet for threads > 8 messages; Haiku for ≤ 8).
- Auto-categorisation: p95 ≤ 800 ms (Haiku; the CaMeL classification is the primary input).

## Alternatives Considered

- **Use a generic email-AI library (Superhuman-style) without CUO.** Rejected: CUO is the brand; a separate AI surface in EMAIL fragments the persona-versioning + acceptance-rate calibration.
- **Auto-send suggested replies on Member confirmation only.** This *is* the chosen path (the suggestion is inserted into the composer; the Member edits and clicks send; send is destructive + step-up). The alternative considered was auto-send if the Member didn't edit — rejected because even a one-click send without inspection is a customer-experience risk.
- **Skip per-Member personalisation; one-size-fits-all categorisation.** Rejected: a customer-facing role's "support" mailbox and an engineer's "internal alerts" mailbox have different category boundaries.
- **Train a fine-tuned model on the team's prior emails.** Rejected for P1: the personalisation overlay (counts + heuristics) is sufficient; fine-tuning is a P3 initiative if quality plateaus.

## Success Metrics

- **Primary metric.** P1 → P2 gate progress: suggested-reply acceptance rate ≥ 30% across the team on a 14-day rolling window; thread-summary acceptance rate ≥ 40%; auto-categorisation manual-override rate ≤ 15%.
- **Founder-cognitive-load metric.** Founder time per email day-over-day decreases by ≥ 25% on the 30-day window vs. the pre-EMAIL baseline (rough proxy for PRD §4.1 G8).
- **Latency.** Per the budgets above; breaches reported in OBS dashboards.
- **Vietnamese quality.** Sampled review (founder + a Vietnamese-native Member) of 30 vi-VN suggested replies rates ≥ 4/5 average on register-correctness.

## Scope

**In-scope.**
- Thread summarisation surface + BRAIN-fact ingestion of summaries.
- Suggested replies (3 per composer open) with voice rules.
- Auto-categorisation with per-Member personalisation overlay.
- Smart snooze suggestions per recipient + on-time CUO reminders.
- Per-Member AI-density preference UI.
- The four MCP tools.
- Persona scope contract enforcement.
- Audit + acceptance metrics + OBS dashboard panels.

**Out-of-scope (deferred).**
- Vietnamese-aware composition (FR-EMAIL-005; co-ships).
- CRM integration for "next-action drafted" replies (FR-EMAIL-006 + batch-05).
- PROJ promote-to-task suggestions (FR-EMAIL-007).
- Voice-input for composing replies (P3 mobile; the Whisper integration in CHAT carries over conceptually).
- Fine-tuned model on team's prior emails (P3).

## Dependencies

- FR-EMAIL-001 / FR-EMAIL-002 / FR-EMAIL-003.
- FR-AI-001 (model routing + cost + latency budgets).
- FR-MCP-001 (tool registration + persona-scope).
- FR-BRAIN-001 / FR-BRAIN-002 (context retrieval; summary persistence as facts).
- FR-GENIE-001 / FR-GENIE-002 (Notify/Question/Review modes; acceptance metrics; persona-version stamping).
- FR-OBS-001 / FR-OBS-002 (acceptance dashboards).
- Compliance: EU AI Act Article 50 (transparency disclosure on every AI surface); EU AI Act Article 14 (human oversight on every send-equivalent action).
- Locked decisions referenced: DEC-084 (CUO-only; no separate AI assistant in EMAIL), DEC-085 (suggested-reply count = 3 by default).

## AI Risk Assessment

EMAIL AI surfaces are the highest-volume AI-derived content emitted toward natural persons in P1. EU AI Act risk class: `limited`.

### Data Sources

All AI surfaces consume CaMeL-sanitised outputs (FR-EMAIL-003) plus per-tenant BRAIN content. No third-party training data; per-tenant residency. The personalisation overlay is per-Member counts only — not a fine-tuned model.

### Human Oversight

- Suggested replies are drafts; the human edits and sends; send is destructive + step-up.
- Summaries are surfaced; the Member reads or skips; no side-effect.
- Auto-categorisation is reversible in one click; manual overrides feed the personalisation.
- Snooze suggestions are informational; the Member picks the time.
- Snooze reminders are Notify-mode; no auto-action.

### Failure Modes

- **Summary hallucinates a fact.** Caught by citation-correctness regression; the cited message must support the summary statement.
- **Suggested reply leaks out-of-scope information** (the Member is in a sales thread; the suggestion mentions internal HR detail). Mitigation: persona-scope contract limits CUO/COO's BRAIN access to the relevant subject_uri scope; the regression suite includes cross-scope-leakage cases.
- **Auto-categorisation drifts.** Per-Member personalisation overlay corrects over time; quarterly review of override patterns.
- **Snooze reminder never fires.** OBS alert if `snooze_until` in past + reminder not delivered within 5 minutes.
- **AI density "aggressive" floods the Genie panel with summaries.** Per-Member kill switch + the founder kill-switch from FR-GENIE-002.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted summarisation + suggested-replies surfaces, density-preference design, snooze-suggestion logic, failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; the Vietnamese-voice rules will be co-authored with a vi-native Member at PR-review.
