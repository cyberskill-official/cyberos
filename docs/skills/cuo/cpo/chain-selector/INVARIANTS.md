# `chain-selector` self-audit invariants (scaffold)

## Invariants

### INV-001 — deterministic selection from frontmatter

**Statement.** Same brief frontmatter (specifically: `project_kind`, `eu_ai_act_risk_class`, `confidentiality`, `budget_band`, `target_release`, `client_visible`) → same chain_profile. The selection function is pure.

**Severity.** `error`.

### INV-002 — chain_plan size matches profile

**Statement.** `lean` chain_plan has exactly 4 skills; `standard` has exactly 6; `full` has exactly 8. (Numbers per SKILL.md §"Chain plan per profile".) Other sizes = breach.

**Severity.** `error`.

### INV-003 — never skip prd-audit when client_visible

**Statement.** If `client_visible: true` AND profile is `lean` (which would skip prd-audit), the skill MUST escalate to user: "client-visible work usually warrants prd-audit; want to override to standard?" If user confirms lean, record the override + reasoning.

**Severity.** `warning` — soft signal; client work without audit is risky but not always wrong.

### INV-004 — user-override is recorded with reasoning

**Statement.** Every user override carries a `## Override Reasoning` block in the chain-plan artefact + a one-line reason in `memories/projects/<slug>.md`. No silent overrides.

**Severity.** `error`.
