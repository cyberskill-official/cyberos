# `inspection-report-audit` — audit loop

Implements the canonical 8-step audit-loop algorithm at `cyberos/skill/docs/AUDIT_LOOP.md`. Customise only artefact-specific bindings.

## Artefact-specific bindings

| Field | Value |
|---|---|
| `artefact_extension` | `.md` |
| `audit_extension` | `.audit.md` |
| `rubric_file` | `RUBRIC.md` |
| `report_format_file` | `REPORT_FORMAT.md` |
| `max_iterations` | 10 |
| `machine_floor` | `node tools/inspect-lint.mjs <report.md>` (exit 0 required) |

## Termination

Below 10/10 on the rubric, route back to `inspection-report-author` citing failing IRA-* rule ids verbatim. Machine-floor failure stops scoring.
