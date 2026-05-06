# `srs-author/acceptance/` — priority test scenarios (stub)

> Pending v0.3.0 harness.

## Priority scenarios

### sev-0
1. **INV-001 refuse non-pass PRD.** Input: PRD with audit verdict `fail`. Expected: `REFUSED_NON_PASS_PRD`.
2. **INV-002 zero llm-implicit on Architecture.** Authority-elevation pass enforces.
3. **Happy path:** passing PRD + 7 architectural answers → `SRS_COMPLETE`; all 10 required H2 sections populated.

### sev-1
4. **CSecO sign-off triggered.** Q5 answers contains "yes" on auth surface → `security_review_required: true` in output.
5. **NFR measurability (INV-005).** Quality bar without measurable threshold flagged as warning.
6. **Chained from prd-audit.** End-to-end: prd-audit pass → srs-author → SRS written; trace_id consistent.

### sev-2
7. **Empty prd_path.** Schema validation fails.

## Citations
- Pattern source — sibling skills' acceptance/README.md.
