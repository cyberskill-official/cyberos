# `srs-audit` self-audit invariants (scaffold)

Mirrors `cuo/cpo/prd-audit/INVARIANTS.md`'s 6-invariant pattern with SRS-specific phrasing:

- **INV-001** — verdict reproducibility on mechanical rules (LLM-judgement rules band-reproducible).
- **INV-002** — rubric coverage (every rule_id appears under passed/failed/skipped).
- **INV-003** — needs_human is precise (only fires on declared ambiguity criteria).
- **INV-004** — citation completeness (every fail cites rule_id + line + substring).
- **INV-005** — no false-pass on STALE.
- **INV-006** — no rubric drift mid-batch.

Severities + refinement templates mirror prd-audit's; rule_ids namespaced under `srs_rubric@1.0`.
