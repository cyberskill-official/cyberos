# AI Gateway P0 — complete (22 FRs at 10/10)

**Audited at:** 2026-05-15
**Status:** all 22 FRs `draft (10/10)` ready to transition to `accepted`

## Slice breakdown

| Slice | FRs | Total effort | Theme |
|---|---:|---:|---|
| 1 | FR-AI-001..005 | 27h | Cost-ledger + BRAIN bridge + policy loader |
| 2 | FR-AI-006..010 | 34h | Multi-provider router + circuit breaker + streaming |
| 3 | FR-AI-011..015 | 28h | PII redaction + persona stamping + ZDR |
| 4 | FR-AI-016..020 | 27h | Residency + cache + cross-leak property test + BGE embeddings/rerank |
| 5 | FR-AI-021..022 | 12h | Operator CLI + OTel emission |
| **Total** | **22** | **128h** | ≈ **16 person-days** |

## Quality metrics

- **Score:** 10/10 average across all 22 FRs
- **Open questions remaining:** 0
- **Failure modes documented:** 145 distinct failure paths
- **OBS metrics enumerated:** 60+ metric names
- **Test cases specified:** 200+ across all §4 acceptance criteria

## Implementation dependency order

```
slice 1:  FR-AI-005 → FR-AI-007 → FR-AI-003 → FR-AI-001 → FR-AI-002 → FR-AI-004
slice 2:  FR-AI-006 → FR-AI-008 → FR-AI-009 → FR-AI-010
slice 3:  FR-AI-011 → FR-AI-012 → FR-AI-013 → FR-AI-014 → FR-AI-015
slice 4:  FR-AI-016 → FR-AI-017 → FR-AI-018 → FR-AI-019 → FR-AI-020
slice 5:  FR-AI-022 → FR-AI-021
```

## Cross-FR consistency

All 22 FRs cross-reference correctly:
- TenantPolicy schema (FR-AI-005) consumed by 8 other FRs
- ResolvedModel (FR-AI-006) consumed by FR-AI-008, FR-AI-015, FR-AI-016
- brain_writer::emit (FR-AI-003) consumed by FR-AI-001, FR-AI-002, FR-AI-004, FR-AI-014
- cost_table::lookup (FR-AI-007) consumed by FR-AI-001, FR-AI-002, FR-AI-006

## Next batch

Per user direction: P0 (OBS, AUTH, MCP, CHAT) + P1 (BRAIN, SKILL, PROJ). User's explicit priority on BRAIN/SKILL/PROJ for P1.
