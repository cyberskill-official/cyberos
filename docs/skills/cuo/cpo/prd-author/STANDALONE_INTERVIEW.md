# Standalone-mode follow-up interview (3-5 questions)

> The brief covered 20 questions; these 3-5 cover PRD-specific decisions the brief deliberately didn't ask.

## Q1 — Feature-flag and rollout strategy

> "How should this ship? Options: (a) full-on at launch, (b) staged rollout (off-by-default → 1% → 10% → 50% → 100%), (c) internal-dogfood only first, then external in a follow-up release, (d) gated behind a per-account flag indefinitely (e.g., enterprise-tier only)."

Validate: one of {a, b, c, d}. Capture the user's reasoning verbatim — it lands in the PRD's quality bars + rollout plan.

## Q2 — Telemetry plan

> "What events MUST this feature emit on launch so we can know it's working? Name 3-5 specific events. Examples for software: `feature.foo.opened`, `feature.foo.action_taken`, `feature.foo.error`. For non-software: 'campaign emails sent', 'leads converted', 'hires made by month'."

Validate: ≥1 event named. The PRD's `## Quality Bars` will include "every event lands in `genie.action_log` of kind `ui_action` / `business_event`".

## Q3 — Approval workflow

> "Before this PRD flips from `draft` to `approved`, who needs to sign off? Options: (a) you (founder/owner) alone, (b) you + one engineering reviewer, (c) you + sales/CS (if client_visible), (d) you + CLO (if EU AI Act limited/high), (e) you + CLO + CSecO (if regulated confidentiality)."

Validate: list of personas. The PRD's frontmatter will encode this in `cl_sign_off` / `cseco_sign_off` fields.

## Q4 — Rollback triggers (skip if Q1 == 'a' full-on)

> "Under what observable signals would you rollback? Be concrete: 'p95 latency > 1.5x baseline for >10 min', 'support ticket volume >2x baseline', 'NPS drop > 5 pts in 7 days', 'p1 incident'."

Validate: ≥1 trigger named with a measurable threshold. Lands in the PRD's `## Quality Bars` and `## Compliance and Privacy` (rollback obligations) sections.

## Q5 — Authority-elevation pass (always run; not user-facing)

After Q1-Q4 + brief synthesis, the skill scans every Goal candidate. For any Goal still at `llm-implicit` authority:

> "I drafted this goal as: '<text>'. I couldn't cite a specific brief section, chat answer, or BRAIN memory for it. Choose:
> (a) elevate to `human-edited` if you confirm verbatim,
> (b) elevate to `llm-explicit` if I should cite <suggested-source>,
> (c) reword the goal,
> (d) drop the goal."

INV-002 forbids `llm-implicit` on Goals; this pass is the mechanism that ensures it.

## When the interview is skipped

- **Chained mode from `requirements-discovery`** with `triage_verdict: proceed`: the supervisor passes the brief; this skill validates + auto-runs Q1-Q4 if they're already answered in the brief's body (rare but possible if the user front-loaded them).
- **`proceed_despite_revise: true`** in input envelope: same as proceed, plus an additional question Q-Override: "Confirm you want to proceed despite triage flagging: <flags>. (yes/no)" — captured to the PRD's `## Reservations Recorded From Discovery` section.

## Citations

- Pattern source — sibling `cuo/cpo/requirements-discovery/STANDALONE_INTERVIEW.md` (longer, intake-side).
- Authority elevation — INV-002 + AGENTS.md §5.3.
