# `srs-audit/acceptance/` — priority test scenarios (stub)

> Pending v0.3.0 harness.

## sev-0
1. Mechanical-rule reproducibility (mirrors prd-audit's INV-001 test).
2. AUTH-001 — missing authority on Architecture claim → needs_human.
3. AUTH-002 — llm-implicit on Architecture claim → needs_human.
4. STALE-001 — SRS changed after audit.

## sev-1
5. NFR measurability flagged as warning when threshold missing.
6. Chained from srs-author end-to-end.

## sev-2
7. Empty srs_paths → schema validation fails.

## Citations
- Pattern source — `cuo/cpo/prd-audit/acceptance/README.md`.
