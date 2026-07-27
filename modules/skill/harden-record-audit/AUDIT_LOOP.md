# `harden-record-audit` — audit loop

Implements the canonical 8-step audit-loop algorithm at `cyberos/skill/docs/AUDIT_LOOP.md`.

## Artefact-specific bindings

| Field | Value |
|---|---|
| `artefact_extension` | `.json` or `.md` (hardening-record@1) |
| `audit_extension` | `.audit.md` |
| `rubric_file` | `RUBRIC.md` |
| `report_format_file` | `REPORT_FORMAT.md` |
| `max_iterations` | 10 |
| `hitl_categories` | scope_breach, verification_gap, fingerprint_drift, silent_close, safety_violation |

## Termination

One scope breach or silent close fails the whole record. No partial acceptance.
