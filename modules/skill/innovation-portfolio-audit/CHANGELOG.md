# Changelog

All notable changes to this skill SHALL be documented here. Format follows Keep-a-Changelog. Versioning is SemVer.

## [1.0.0] — YYYY-MM-DD

### Added
- Initial audit skill scaffold copied from `_template/audit/`.
- Rubric version `innovation-portfolio_rubric@1.0`.
- 8-step audit loop per `cyberos/skill/docs/AUDIT_LOOP.md`.
- HITL halt-batch policy + aggregation discipline.
- NATS event emission for `audit_written`, `audit_batch_complete`, `hitl_pause`.
- `deterministic_drift` self-audit invariant (catastrophic — pauses immediately).

### Acceptance
- Golden fixture: `acceptance/golden-<flow-id>-input.json` + `acceptance/golden-<flow-id>-output.md` + sample audit report.
- Byte-stable on the same artefact + rubric version.
