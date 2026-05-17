# Changelog

All notable changes to this skill SHALL be documented here. Format follows Keep-a-Changelog. Versioning is SemVer.

## [1.0.0] — YYYY-MM-DD

### Added
- Initial author skill scaffold copied from `_template/author/`.
- PLAN / WORKER / RESUME phase machine.
- HITL halt-batch policy.
- NATS event emission for `pen-test-report_written`, `batch_complete`, `hitl_pause`.
- Anti-fabrication + untrusted-content discipline.

### Acceptance
- Golden fixture: `acceptance/golden-<flow-id>-input.json` + `acceptance/golden-<flow-id>-output.md`.
- Self-audit passes on its own input set at 10/10 by `pen-test-report-audit`.
