# Changelog — subtask@1 contract

## 2026-05-12 — Introduced as part of skills-Stage-1 collapse

Promoted from informal "work-package" wording inside `task@1` and `tech_spec@1` to a first-class contract. Drives the new `solo` chain_profile that collapses `task-author` + `fr-to-tech-spec` into a single `task-with-subtasks` skill for small-team workflows (CyberSkill internal use today).

### Added

- 12 required fields: id, title, description, preconditions, deliverables, acceptance_test, sizing, dependencies, parallelisable, assignable_to, agent_profile, estimated_tokens, estimated_hours, status
- 6 optional fields: owner, sprint, linked_pr, notes, review_cohort, runbook_hint
- Task ID format `FR-NNN-T-MM` — addressable, greppable, PR-referenceable
- Validation rules (8) including ≥200-char description floor + acceptance_test exactly-one-of
- Lifecycle: draft → ready → in_progress → {done, blocked}

### Steward

- Persona: `cuo-cpo`
- Legal escalation: `cuo-clo`
- Security escalation: `cuo-cseco`
