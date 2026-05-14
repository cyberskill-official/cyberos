# Standalone-mode discovery interview (20 questions)

> 5 triage gating questions + 15 discovery questions. Project-kind-agnostic (works for software, marketing, hiring, partnerships, research). Skill folds answers into a `project_brief@1` markdown.

## Phase 1 — Initial pitch (Q0)

### Q0 — Tell me about the project

> "What's the project? One paragraph is fine — what would you build, ship, or commission, and what would success feel like?"

If `initial_prompt` was supplied in the input envelope, skip Q0.

Validate: free-text response, ≥30 chars. If shorter, prompt for more.

After Q0, classify into `project_kind`:
- `software_product` / `software_consulting_engagement` / `internal_tooling` / `marketing_campaign` / `hiring_plan` / `partnership` / `research_spike` / `other`.

If ambiguous between two kinds, ask: "This sounds like both X and Y. Which is the dominant frame?"

## Phase 2 — Triage (Q1-Q5; gating)

### Q1 — Strategic fit

> "I've read your `company/locked-decisions.md` and `company/values.md`. The most relevant locked decisions are: [...list 3 most relevant...]. Does this project align with them, or does it require revisiting any?"

Reads BRAIN scope: `company:locked-decisions` + `company:values`.

If user says "revisit a locked decision" → set triage_verdict `revise`, route to `cuo-clo` (locked decisions are write-locked per AGENTS.md §9.6).

### Q2 — Capacity

> "I see in `member/` you have <N> people right now: [...names + roles...]. Total available headcount over the next [<target_release>] window is roughly <X> engineer-weeks. Is the project realistic for that capacity, or does it require hiring / contracting / reprioritisation?"

Reads BRAIN scope: `member:*` (excluding `member:*/private/`).

If user says capacity is insufficient → ask: "Should we proceed and trigger a hiring-plan project, scope-down to fit current capacity, or reject?"

### Q3 — Runway

> "What's the budget envelope? (`under_5k` / `5k_to_25k` / `25k_to_100k` / `over_100k` / `none` / `undisclosed`.) And what's the latest acceptable ship-by date?"

Validate against `target_release`. If date precedes feasible scope, flag.

### Q4 — Customer signal strength

> For `client_visible: true`: "How strong is the request from <client>? Is it (a) a pilot CSM mentioning it once, (b) a written request from their decider, (c) blocking renewal, or (d) part of a signed SOW?"
>
> For `client_visible: false`: "How many independent signals point at this? I'm looking at `memories/projects/` and `memories/decisions/` for prior signals. The most recent ones I see are: [...]. Are you observing additional signals not yet in BRAIN?"

Below "≥3 independent signals" or "≥(b) for client_visible" → triage flag.

### Q5 — Reversibility

> "If we start this and decide to stop in <2 weeks / 2 months>, what's the cost? (a) trivial, throw away docs, (b) modest, return some hours, (c) meaningful, refunds + reputation, (d) severe, contractual penalties or platform lock-in.)"

(d) requires CLO sign-off before triage_verdict `proceed`.

### Triage verdict computation

After Q1-Q5:
- All 5 pass at green → `proceed`.
- 1-2 amber (capacity, runway, signal strength) → `revise` + surface to user with explicit reasoning.
- 3+ amber OR 1 red (locked-decision conflict, severe irreversibility, hard capacity ceiling) → `reject`.

If `revise`, ask user: "I'd suggest revising [...]. Want to (a) amend the proposal now, (b) proceed anyway with my reservations recorded in the brief, or (c) stop?"

## Phase 3 — Discovery (Q6-Q20; only if triage_verdict ∈ {proceed, revise+proceed-anyway})

### Q6 — Primary outcome

> "If we ship this and it works, what's the SINGLE most-important user-visible behaviour change?"

### Q7 — Secondary outcomes

> "Any other outcomes you'd consider success indicators?" (Optional; 0-3 entries.)

### Q8 — Primary success metric

> "Which metric would you track to know we hit Q6's outcome? Answer in the form: <metric name> from <baseline> to <target> by <date>. If the baseline isn't known, say 'baseline TBD'."

Vanity metrics (signups, views, followers without engagement context) flagged.

### Q9 — Guardrail metric

> "Is there a metric that MUST NOT degrade as we ship this? (e.g., page load time, support volume, churn.)" (Optional.)

### Q10 — Time-to-value

> "When does the user feel the value? Day 1, week 1, month 1, longer?"

### Q11 — Audience specificity

> "WHO benefits? Be specific — name a persona, segment, or named client. 'Users' is too vague."

### Q12 — Demand evidence

> "What's the strongest piece of evidence that this audience wants this?"

### Q13 — Prior art

> "Any solutions out there (ours or competitors') that do something similar? What do you like / dislike about them?"

### Q14 — Hard timeline constraint

> "Is there a date this MUST ship by, or a window we MUST hit? (e.g., contract milestone, regulatory deadline.)"

### Q15 — Regulatory / compliance touchpoints

> "Does this touch any of: PII, payments, healthcare data, biometric data, AI-driven decisions about people (hiring, credit, education, insurance), real-time biometric ID, social scoring? If yes, which?"

If yes, set `eu_ai_act_risk_class` preliminarily and route to `cuo-clo` for confirmation.

### Q16 — Threat-model triggers

> "Does this involve: new auth surfaces, new data flows out of our infrastructure, secret stores, encryption-at-rest decisions, or partner-data handling? If yes, which?"

If yes, route to `cuo-cseco` for sign-off before phase 5.

### Q17 — Confidentiality

> "Public / internal / client-confidential / regulated?"

### Q18 — Stakeholder map

> "Who's the decider? Who reviews? Who needs to be informed but doesn't gate?"

### Q19 — Kill criteria

> "Under what observable conditions would you STOP this project? Be specific."

### Q20 — BRAIN-cross-reference closing question

> "I've found these in BRAIN that look relevant: [...list 3-5 most-relevant memories from `memories/projects/` + `memories/decisions/`...]. Anything I should weight differently or contradict?"

## Phase 4 — Targeted BRAIN reads (post-interview, no user-facing questions)

After Q20, the skill issues up to 10 BRAIN queries based on named entities + project_kind. Hits inform the `## Prior Art (BRAIN)` section. Budget enforced by INV-006.

## Phase 5 — Synthesise + amendment-batch

Skill writes v1 of the brief. User reviews + batches amendments. Skill applies, increments `discovery_iteration`, repeats until user approves (or quits, in which case the brief stays at its current iteration with `prd_status: draft`).

## Phase 6 — Write + emit

Final brief written. NATS subject `cuo.requirements_discovery.brief_written` published. `next_skill_recommendation: cuo/cpo/prd-author` set IFF triage_verdict == proceed.

## When the interview is skipped

Only one case: user provides a fully-formed `initial_prompt` covering all 20 answers (rare). Skill validates the prompt's coverage; if <80% of answers present, skill prompts for the missing fields rather than proceeding with a thin brief.

## Citations

- Pattern source — `cuo/cpo/fr-author/STANDALONE_INTERVIEW.md` and `cuo/cpo/fr-audit/STANDALONE_INTERVIEW.md`.
- Triage rationale — Q3 of registry v0.2.4 design conversation: "fold in" project-triage, no separate skill.
- Project-kind taxonomy — Q2 of the same conversation: fr-author handles software + non-software.
- BRAIN read budget — INV-006 in this skill's `INVARIANTS.md`.
