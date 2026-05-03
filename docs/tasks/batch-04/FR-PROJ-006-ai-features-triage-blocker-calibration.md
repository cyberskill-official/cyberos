---
title: "PROJ — AI features: daily task triage, auto-blocker detection, status-update generation, estimate calibration, cross-project insight"
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
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Layer the AI-native PROJ features on top of FR-PROJ-001..005: **daily task triage** ("what should I work on today?" answered per Member from cycle plan + blockers + dependencies); **auto-blocker detection** that scans CHAT, EMAIL, comments, and inactivity for blocker patterns and surfaces Notify cards to the assigner; **status-update generation** (the cycle-review draft pipeline called by FR-PROJ-004); **estimate calibration** that compares estimated vs. actual hours per Member per task class and surfaces drift; **cross-project insight** that runs graph queries over the AGE memory layer to answer "what's the pattern in late tasks?". All AI-derived surfaces operate through CUO/COO + CUO/CTO + CUO/CPO skills, render the EU AI Act Article 50 disclosure chip, and honour the persona scope contract (FR-MCP-001).

## Problem

The PRD §9.5.3 names the AI-native features explicitly:

- "CUO/COO-skill task triage. Daily 'what should I work on' answer for each Member based on cycle plan, blockers, and dependencies."
- "Auto-blocker detection. Comments with 'blocked by', 'waiting on', or non-trivial inactivity are surfaced as a Notify nudge to the assigner."
- "Status-update generation. End of cycle: CUO drafts the cycle review for the Account Manager to edit and send."
- "Estimate calibration. CUO/CPO-skill compares estimated vs actual hours per Member per task class and surfaces calibration drift."
- "Cross-project insight. 'What's the pattern in late tasks?' runs a graph query across the AGE memory layer."

Without these features, PROJ is parity-with-Linear; with them, PROJ is the moat. The founder-cognitive-load goal (PRD §4.1 G8) depends on the AI doing the triage + drafting work the founder otherwise does himself.

## Proposed Solution

The shape of the answer is five integrated surfaces, each authored as a CUO skill or sub-skill, calling into the AI Gateway with persona-scope-contract-enforced retrieval against PROJ data + BRAIN context.

**Daily task triage.**

Trigger: a per-Member 07:00 ICT job (or per-Member configured start-time) computes the triage; the result lands in the Genie panel "Today" tab and the Founder Daily Flow page (FR-GENIE-003).

The triage prompt (CUO/COO):
- Inputs: the Member's open + assigned issues across all projects; their `due_date`s; their `blocked_by_issue_ids` graph; the current cycle's progress; the Member's recent activity (last 5 issue updates); BRAIN-derived priorities (the Member's stated focus, the engagement's commitments).
- Output: a ranked list of 3–7 issues with a one-sentence rationale per issue ("Pick this first because it unblocks ALPHA-1267 which Khoa starts tomorrow"); citations to the relevant data.

The Member can: accept (the issues open as a focused list); ask for a different ranking; override per-issue.

**Auto-blocker detection.**

Three sources scanned continuously:

1. **Comment patterns.** A `proj.issue_comment.body_md` containing `blocked by`, `waiting on`, `chờ`, `vướng` (Vietnamese), or a `@member` mention with a question and no follow-up reply within 24 hours.
2. **CHAT mentions.** A CHAT message containing the issue's key (`ALPHA-1234`) plus a blocker pattern, reaching CUO via the CHAT NATS consumer (FR-CHAT-001).
3. **Inactivity heuristic.** An `in_progress` issue with no `proj.issue_state_transition`, no comment, no assignee activity for ≥ 3 working days.

When triggered, CUO/COO emits a Notify card to the *assigner* (the Member who set the assignee or the project lead) — not the assignee directly, because the assignee may be the one stuck. The card explains the trigger, links the issue, and offers actions: "ask the assignee in CHAT", "reassign", "split into smaller issues", "transition to blocked with reason".

**Status-update generation (cycle review draft).**

The implementation of `cyberos.proj.draft_cycle_review(cycle_id)` called by FR-PROJ-004's close UI. Returns:

```json
{
  "narrative_md": "Cycle 14 closed with 87 of 100 committed points completed (87% completion). The cycle goal — 'Ship Acme onboarding flow' — was met for the primary milestone (ALPHA-1234, ALPHA-1245) but slipped on the secondary milestone (ALPHA-1252) due to a Stripe integration regression that surfaced on Wednesday. Khoa unblocked ALPHA-1245 by extending Acme's integration window by 3 days; this is captured as a learning for next cycle's estimation. 13 carryover points move to Cycle 15…",
  "citations": [
    { "kind": "issue", "id": "...", "title": "ALPHA-1234 Acme onboarding flow" },
    { "kind": "issue", "id": "...", "title": "ALPHA-1252 Stripe integration regression" },
    { "kind": "brain_fact", "id": "...", "subject": "engagement:acme", "predicate": "has_extension", "object": "+3d on integration window" },
    { "kind": "chat_message", "id": "...", "context": "Khoa's update on the regression" }
  ],
  "calibration_note": "committed: 100 / completed: 87 / carried: 13; velocity rolling-6: 92; this cycle ran 5% under rolling average",
  "next_cycle_recommendations_md": "Reduce committed points to ~92; reserve buffer for known regression risk on the Stripe path; assign ALPHA-1252 carryover to Khoa given his context.",
  "persona_version": "cuo-coo-v0.4.2",
  "ai_disclosure_id": "..."
}
```

The draft is offered for the user to edit in the cycle-close UI; persona-acceptance counted on save vs. discard.

**Estimate calibration.**

A weekly job at Friday 17:00 ICT computes per-Member-per-task-class estimate-vs-actual:

- "Task class" derived from issue labels + project + a clustering of titles (`cluster:auth-related`, `cluster:ui-polish`, `cluster:devops`).
- "Actual" derived from time entries (FR-TIME-001 in batch-05; in P1 PROJ alone we use first-state-transition-to-`in_progress` → `done` clock-time as a proxy).
- "Drift" = (actual / estimate × estimate's t-shirt scale).

The output is a per-Member chart in the Genie panel "Tech" tab + a one-sentence narrative ("You consistently underestimate auth-related tasks by ~1.5×; consider sizing UP next time"). Surfaced weekly as a Notify card; never blocking; informational.

**Cross-project insight.**

A natural-language query surface on the Genie panel: "what's the pattern in late tasks?". The query runs:

1. CUO/COO extracts intent → graph-query specification.
2. The query runs against the AGE graph layer in BRAIN (FR-BRAIN-002) over PROJ-derived facts (issue → assignee, issue → label, issue → cycle, issue → completion-vs-due).
3. Result: a small list of patterns with citations ("Issues labelled `auth` are 1.4× more likely to slip than the baseline; the most common cause is third-party integration delay (5 of 7 cases)").

Other example queries:
- "Which engagements have the most carryover?"
- "Who is the most-blocked Member this cycle?"
- "Show me issues that took 3× their estimate."
- "Pattern in cancelled issues this quarter."

**Persona scope contract.**

CUO/COO declares `tools_allowed`:
```
- cyberos.proj.list_issues
- cyberos.proj.get_issue
- cyberos.proj.list_cycles
- cyberos.proj.get_cycle_progress
- cyberos.proj.list_at_risk_issues
- cyberos.proj.search
- cyberos.brain.search
- cyberos.chat.search_messages       (read-only)
- cyberos.email.search                (read-only)
- cyberos.crm.search_*                (read-only; CRM in batch-05)
- cyberos.genie.notify
- cyberos.genie.ask_question
- cyberos.genie.draft_review
```

`tools_forbidden_explicit`:
```
- cyberos.proj.create_issue
- cyberos.proj.update_issue
- cyberos.proj.transition_issue
- cyberos.proj.assign_issue
- cyberos.proj.move_to_cycle
- cyberos.proj.close_cycle
```

CUO drafts; the human applies. The mutation tools land in FR-PROJ-008 with destructive-confirmation gates.

**Latency budgets.**

- Daily triage compute: pre-computed at 07:00 ICT (no user-facing latency).
- Auto-blocker detection: per-event ≤ 4 s p95 from event arrival to Notify card.
- Cycle-review draft: ≤ 8 s p95.
- Estimate calibration: pre-computed weekly (no user-facing latency).
- Cross-project insight: ≤ 6 s p95 for the answer.

**MCP tool surface.**

- `cyberos.proj.daily_triage(member_id?)` — read.
- `cyberos.proj.draft_cycle_review(cycle_id)` — read; the FR-PROJ-004 consumer.
- `cyberos.proj.draft_status_update(project_id, range)` — read; ad-hoc status updates between cycles.
- `cyberos.proj.list_calibration_drift(member_id?, since)` — read.
- `cyberos.proj.cross_project_insight(query)` — read.
- `cyberos.proj.list_blocker_signals(since, project_id?)` — read; for the founder-review of what's been auto-flagged.

## Alternatives Considered

- **Make CUO auto-create issues from CHAT mentions.** Rejected: human-in-the-loop floor; CUO suggests, human creates (the EMAIL→PROJ promote in FR-EMAIL-007 is the canonical pattern).
- **Train a fine-tuned model on the team's prior cycles.** Rejected for P1: the persona + retrieval pattern is sufficient; fine-tuning is a P3 initiative if quality plateaus.
- **Skip cross-project insight; offer a simple report instead.** Rejected: the natural-language graph query is a key differentiator and the founder uses it weekly.
- **Run estimate calibration daily.** Rejected: weekly cadence prevents over-fitting on a single noisy day; the user can click "recompute" if they want fresh.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: daily-triage acceptance rate ≥ 40%; cycle-review-draft acceptance-rate ≥ 50% (vs. discard); auto-blocker-Notify acceptance rate ≥ 35%; cross-project-insight thumbs-up rate ≥ 50% on a 30-day window.
- **Founder-cognitive-load metric.** Cycle-review-draft saves the founder ≥ 30 minutes per cycle vs. manual.
- **Citation correctness.** 100% of citation chips link to the actual source; regression suite blocks any persona-version PR that introduces drift.

## Scope

**In-scope.**
- The five surfaces (triage, blocker detection, status drafting, calibration, cross-project insight).
- The pre-compute jobs (07:00 ICT triage; weekly calibration).
- The continuous blocker-signal consumer.
- Persona scope contract enforcement.
- The six MCP tools.
- BRAIN-fact ingestion of cycle reviews + calibration narratives.
- OBS persona-quality dashboard panels for PROJ-AI surfaces.

**Out-of-scope (deferred).**
- Auto-action on irreversible operations (forbidden by architectural rule).
- Fine-tuned model on the team's prior cycles (P3).
- Cross-tenant insight (forbidden by design).
- Voice-input triage (P3 mobile).
- Per-Member triage personalisation beyond confidence thresholds (P2).

## Dependencies

- FR-PROJ-001 / FR-PROJ-002 / FR-PROJ-003 / FR-PROJ-004 / FR-PROJ-005.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-BRAIN-001 / FR-BRAIN-002 (context retrieval; cycle-review-fact ingestion).
- FR-GENIE-001 / FR-GENIE-002 (Notify/Question/Review modes; persona-scope contract).
- FR-CHAT-001 (CHAT signal source for blocker detection).
- FR-EMAIL-001..010 (EMAIL signal source for blocker detection).
- FR-CRM-001 + FR-CRM-002 (batch-05) (read-only access through CUO scope contract).
- FR-TIME-001 (batch-05) (real time-tracking data for calibration; P1 uses state-transition proxy).
- FR-OBS-002 (dashboards).
- Compliance: EU AI Act Article 50 (every AI-derived element renders the disclosure); Article 14 (human approves every action).
- Locked decisions referenced: DEC-114 (CUO drafts; human applies for every mutation), DEC-115 (estimate calibration is informational, never blocking).

## AI Risk Assessment

PROJ AI surfaces materially shape how work is prioritised and reviewed. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: PROJ data, CHAT messages, EMAIL messages, CRM data, BRAIN facts. No third-party training data; no cross-tenant.

### Human Oversight

- Every mutation that PROJ AI surfaces could imply (create issue, transition state, assign, close cycle) goes through the human, not the AI.
- Notify cards are dismissible.
- Review-mode cycle-review drafts are explicitly approved before commit.
- Insight queries are informational; the user decides what to do.
- Founder kill-switch (FR-GENIE-002) silences all PROJ AI surfaces in 30 seconds.

### Failure Modes

- **Wrong triage ranking.** Members override; the override doesn't change the next compute (the algorithm is calibrated quarterly, not per-call).
- **Spurious blocker Notify** ("waiting on" in a sentence about lunch). Mitigation: the regex + classifier pattern is calibrated against the regression suite; false-positive rate ≤ 10% target.
- **Cycle-review hallucinates a fact.** Mitigation: citation-correctness regression test; the reviewer (founder + Engineering Lead) edits before commit.
- **Calibration drift spurious.** Mitigation: requires ≥ 5 same-class issues to compute drift; below threshold the calibration is silent.
- **Cross-project query mis-extracts intent.** Mitigation: the answer surfaces the inferred query for the user's review; mis-extracts are rare given the structured nature of the data.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted five-surface architecture, persona scope contract, MCP tool surface, failure modes.
- **Human review:** `@stephen-cheng` reviewed; persona-version eval cases for each surface authored at PR-review.
