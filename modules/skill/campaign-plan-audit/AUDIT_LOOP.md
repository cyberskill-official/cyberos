# `campaign-plan-audit` — audit loop

This skill implements the **canonical 8-step audit-loop algorithm** documented at `cyberos/skill/docs/AUDIT_LOOP.md`. Do not duplicate the algorithm here. Customize only the artefact-specific aspects below.

## Artefact-specific bindings

| Field | Value |
|---|---|
| `artefact_extension` | `.md` (or skill-specific, e.g. `.json`, `.xml`) |
| `audit_extension` | `.audit.md` |
| `rubric_file` | `RUBRIC.md` (this bundle) |
| `report_format_file` | `REPORT_FORMAT.md` (this bundle) |
| `max_iterations` | 10 |
| `hitl_categories` | (list per skill — declared in CONTRACT_ECHO) |

## Termination policy override

Default termination is defined in `cyberos/skill/docs/AUDIT_LOOP.md` §7. This skill MAY override one rule's termination behaviour by documenting the override here:

| rule_id | override | reason |
|---|---|---|
| `STALE-001` | When fired, the loop terminates with `needs_human` even if other issues are still open, because STALE handling requires operator input before other rules become meaningful. | Avoids cascading re-fix attempts on stale source. |

(Other overrides documented here; remove the example row if the skill ships defaults only.)

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md` — the canonical algorithm.
- `RUBRIC.md` (sibling file) — the rules this loop walks.
- `REPORT_FORMAT.md` (sibling file) — the report shape this loop writes.
