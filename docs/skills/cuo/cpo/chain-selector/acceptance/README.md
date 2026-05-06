# `chain-selector/acceptance/` — priority test scenarios (stub)

> Pending v0.3.0 harness.

## sev-0
1. Brief with `eu_ai_act_risk_class: high` → `chain_profile: full` regardless of other fields.
2. Brief with `confidentiality: regulated` → `chain_profile: full`.
3. Brief with `project_kind: research_spike` + `budget_band: under_5k` → `chain_profile: lean`.
4. Brief with `project_kind: software_product` + `budget_band: 5k_to_25k` → `chain_profile: standard` (default).

## sev-1
5. User override from `standard` to `lean` → recorded with reasoning in chain-plan artefact + memories/projects/.
6. User override TO `lean` when `client_visible: true` → INV-003 fires (warning); user can confirm to proceed.
7. Same brief frontmatter twice → identical chain_plan (INV-001 deterministic selection).

## sev-2
8. Empty brief frontmatter → schema validation fails → BOOT-003.
