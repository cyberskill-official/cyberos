# `repo-context-map-audit` - audit loop

This skill implements the **canonical 8-step audit-loop algorithm** documented at `cyberos/skill/docs/AUDIT_LOOP.md`. Do not duplicate the algorithm here.

## Artefact-specific bindings

| Field | Value |
|---|---|
| `artefact_extension` | `.md` (JSON payload pairs where SKILL.md says so) |
| `audit_extension` | `.audit.md` |
| `rubric_file` | `RUBRIC.md` (repo_context_map_rubric@1.0) |
| `report_format_file` | `REPORT_FORMAT.md` (this bundle) |
| `max_iterations` | 10 |
| `hitl_categories` | needs_human verdicts per RUBRIC.md |

## Termination policy override

Ships defaults (`cyberos/skill/docs/AUDIT_LOOP.md` §7); no per-rule overrides.
