# `coverage-gate-author` - acceptance

How to verify this skill (TASK-SKILL-117 convention):

1. `TRIGGER_TESTS.md` in this directory - run the trigger cases against the classifier conventions (TASK-SKILL-111/112).
2. Structural: `bash tools/cyberos-init/check-pair-parity.sh modules/skill` reports no PARITY line for `coverage-gate-author`.
3. Contract: envelopes validate as JSON Schema; artefact `coverage-gate@1` fields per SKILL.md.
