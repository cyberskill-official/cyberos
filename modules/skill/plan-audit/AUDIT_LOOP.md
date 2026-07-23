# `plan-audit` — audit loop

This skill implements the **canonical 8-step audit-loop algorithm** documented at `cyberos/skill/docs/AUDIT_LOOP.md` (and summarised in SKILL.md §4). Do not duplicate the algorithm here. Customize only the artefact-specific aspects below.

## Artefact-specific bindings

| Field | Value |
|---|---|
| `artefact_extension` | `.md` (`docs/plans/PLAN-<slug>-<YYYYMMDD>/plan.md`) |
| `audit_extension` | `.audit.md` (sibling of the plan) |
| `rubric_file` | `../rubrics/plan_rubric.md` (`plan_rubric@1.0`) via this bundle's `RUBRIC.md` binding; vendored path `.cyberos/cuo/rubrics/plan_rubric.md` |
| `report_format_file` | `REPORT_FORMAT.md` (this bundle) |
| `max_iterations` | 10 |
| `hitl_categories` | `operator_verdict` (PLAN-GATE-001), `unknown_artefact_version` |

## Termination policy override

Default termination is defined in `cyberos/skill/docs/AUDIT_LOOP.md` §7. This skill overrides one rule's termination behaviour:

| rule_id | override | reason |
|---|---|---|
| `PLAN-GATE-001` | When the recorded operator verdict is absent, the loop terminates with `needs_human` immediately, even if other issues are still open. | The decision gate is a HITL halt the rubric cannot decide (SKILL.md blockers); auto-fix rounds against an unapproved plan would polish an artefact that must not exist yet. |

## Auto-fix discipline

Auto-fixable rules (`FM-002`, `FM-004`, `FM-105` — see the rubric's auto-fix catalogue) apply minimal textual changes. The three load-bearing completeness rules (`PLAN-OPT-001`, `PLAN-DEC-001`, `PLAN-OUT-001`) are NEVER auto-fixed: fabricating an option, a decision, or an out-list is authoring, which is `plan-author`'s job under its own gate.

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md` — the canonical algorithm.
- `RUBRIC.md` (bundle root) — the rubric binding this loop walks.
- `REPORT_FORMAT.md` (bundle root) — the report shape this loop writes.
- `../rubrics/plan_rubric.md` — the canonical rule tables.
