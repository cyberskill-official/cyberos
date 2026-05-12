# fr-with-tasks STANDALONE_INTERVIEW

> When invoked standalone (operator paste, no upstream chain), the skill asks 3 short questions before generating. In chained mode these come from the chain manifest.

## Q1 — Target sprint (optional)

> Which sprint should these FRs target? `current | next | unsequenced` (default: `unsequenced`)

Used to set the `sprint:` field on every emitted task. Skipping it leaves `sprint: null`.

## Q2 — AI-agent budget (per FR)

> Default AI-agent token budget per FR? Round number, e.g. 30000.

Used to bound `estimated_tokens` summed across tasks tagged `ai-agent`. If summed exceeds the budget, skill flags the FR for split.

## Q3 — Acceptable risk tier (EU AI Act)

> What's the highest risk tier you're willing to ship as solo-profile? `minimal | limited | high_risk` (default: `limited`)

Any FR auto-classified above this tier triggers HITL pause + escalates to `cuo-clo`.

## Optional Q4 — Reviewer cohort

> Who must approve before tasks transition out of `draft`? Comma-separated subject IDs (default: just `created_by`).

Populates `review_cohort` on every emitted task.

## Optional Q5 — Project tracker target

> Which proj backend will receive these tasks? `linear | jira | github | none` (default: `none`).

Stored in the manifest for downstream `cyberos proj sync`. Not used directly by this skill.
