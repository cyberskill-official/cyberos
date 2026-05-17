# LEARN module — feature request index

_Generated 2026-05-17 — 7 FRs, 40 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-LEARN-001](FR-LEARN-001-skill-tree-mastery.md) | MUST | 7 | 6 | LEARN skill tree schema — 1-5 mastery levels per skill per Member with parent-child skill graph |
| [FR-LEARN-002](FR-LEARN-002-degrees-certifications.md) | MUST | 7 | 4 | LEARN bằng cấp + chứng chỉ — degree + certification evidence types with issuer + expiry + verificati |
| [FR-LEARN-003](FR-LEARN-003-vp-rollup.md) | MUST | 7 | 6 | LEARN VP (Voting Power) deterministic nightly roll-up — aggregates PROJ + TIME + KB contributions in |
| [FR-LEARN-004](FR-LEARN-004-specialist-council.md) | MUST | 7 | 10 | LEARN Hội đồng Chuyên môn (Specialist Council) — 3-5 judges + multi-dim scoring + per-judge anonymit |
| [FR-LEARN-005](FR-LEARN-005-per-judge-isolation.md) | MUST | 7 | 5 | LEARN per-judge score isolation — never exit LEARN boundary; HR receives only summary + recommendati |
| [FR-LEARN-006](FR-LEARN-006-promotion-approval.md) | MUST | 7 | 5 | LEARN promotion approval workflow — CEO + CHRO sign-off after council vote with cascade to HR + REW  |
| [FR-LEARN-007](FR-LEARN-007-vp-rew-handoff.md) | MUST | 7 | 4 | LEARN VP score → REW BP fund distribution handoff — quarter-close trigger emits aggregate VP shares  |

## Cross-module dependencies

**This module depends on:**

- **HR**: FR-LEARN-001→FR-HR-001
- **PROJ**: FR-LEARN-003→FR-PROJ-013
- **TIME**: FR-LEARN-003→FR-TIME-001

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._