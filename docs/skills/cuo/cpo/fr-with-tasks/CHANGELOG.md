# Changelog — fr-with-tasks

## v0.1.0 — 2026-05-12 — Introduced

First release. Collapses `fr-author` + `fr-to-tech-spec` for the `solo` chain_profile (default for CyberSkill internal workflows; the 2-stage chain stays available for client-facing work demanding persona separation).

### Behaviour

- Reads PRD, SRS, or natural-language spec; emits `feature_request@1` files with embedded `task@1` lists
- Each task includes: id (FR-NNN-T-MM), title, ≥200-char description, preconditions, deliverables, concrete acceptance_test (shell or assertion), sizing, dependencies, parallelisable, assignable_to, agent_profile/tokens or estimated_hours
- Self-audit: 14 invariants per INVARIANTS.md
- HITL gates: ambiguous acceptance_test, high_risk EU AI Act tier, XL sizing
- Standalone interview: 3 questions (sprint, AI-agent budget, risk tier)

### Contracts depended on

- `feature_request@1` (existing)
- `task@1` (introduced same day; lives at `cyberos/docs/contracts/task/`)

### Steward

- persona: `cuo-cpo`
- legal: `cuo-clo`
- security: `cuo-cseco`
