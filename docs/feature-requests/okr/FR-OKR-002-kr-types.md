---
id: FR-OKR-002
title: "OKR 3 KR types — hit_target + improvement + milestone with type-specific progress calculation"
module: OKR
priority: MUST
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-OKR-001, FR-OKR-003, FR-OKR-004, FR-MEMORY-111]
depends_on: [FR-OKR-001]
blocks: []

source_pages:
  - website/docs/modules/okr.html#kr-types

source_decisions:
  - DEC-1970 2026-05-17 — 3 KR types: hit_target (one-shot binary), improvement (start/current/target), milestone (ordered checkpoints)
  - DEC-1971 2026-05-17 — Closed enum `kr_type` = {hit_target, improvement, milestone}; cardinality 3
  - DEC-1972 2026-05-17 — Progress calc per type: hit_target → 0 or 100; improvement → (current - start) / (target - start); milestone → completed / total checkpoints
  - DEC-1973 2026-05-17 — Per-type validation: hit_target requires target_value; improvement requires start/target; milestone requires checkpoint array
  - DEC-1974 2026-05-17 — memory audit kinds: okr.kr_type_set, okr.kr_progress_calculated, okr.kr_validation_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/okr/
  new_files:
    - services/okr/migrations/0002_kr_types.sql
    - services/okr/src/kr_type/mod.rs
    - services/okr/src/kr_type/progress_calc.rs
    - services/okr/src/kr_type/validator.rs
    - services/okr/src/audit/kr_type_events.rs
    - services/okr/tests/kr_type_enum_cardinality_test.rs
    - services/okr/tests/kr_hit_target_progress_test.rs
    - services/okr/tests/kr_improvement_progress_test.rs
    - services/okr/tests/kr_milestone_progress_test.rs
    - services/okr/tests/kr_type_validation_test.rs
    - services/okr/tests/kr_audit_emission_test.rs

  modified_files:
    - services/okr/src/krs.rs

  allowed_tools:
    - file_read: services/okr/**
    - file_write: services/okr/{src,tests,migrations}/**
    - bash: cd services/okr && cargo test kr_type

  disallowed_tools:
    - skip type validation (per DEC-1973)
    - non-deterministic progress calc (per DEC-1972)

effort_hours: 4
sub_tasks:
  - "0.3h: 0002_kr_types.sql"
  - "0.3h: kr_type/mod.rs"
  - "0.5h: progress_calc.rs"
  - "0.5h: validator.rs"
  - "0.3h: audit/kr_type_events.rs"
  - "1.8h: tests — 6 test files"
  - "0.3h: docs"

risk_if_skipped: "Without type-specific calc, all KRs use ad-hoc formulas → progress drift. Without DEC-1973 validation, milestone KRs without checkpoints fail at progress time. Without DEC-1972 deterministic calc, weekly check-in numbers vary by minute."
---

## §1 — Description (BCP-14 normative)

The OKR service **MUST** extend KR schema with 3 types at `services/okr/src/kr_type/` enforcing per-type validation + deterministic progress, 3 memory audit kinds.

1. **MUST** validate `kr_type` against closed enum per DEC-1971.

2. **MUST** define schema extension at migration `0002`:
   ```sql
   ALTER TABLE okr_krs ADD COLUMN kr_type TEXT NOT NULL DEFAULT 'improvement'
     CHECK (kr_type IN ('hit_target','improvement','milestone'));
   ALTER TABLE okr_krs ADD COLUMN start_value NUMERIC(18,4);
   ALTER TABLE okr_krs ADD COLUMN target_value NUMERIC(18,4);
   ALTER TABLE okr_krs ADD COLUMN current_value NUMERIC(18,4);
   ALTER TABLE okr_krs ADD COLUMN milestone_checkpoints JSONB;
   ALTER TABLE okr_krs ADD COLUMN computed_progress_pct NUMERIC(5,2)
     CHECK (computed_progress_pct IS NULL OR (computed_progress_pct >= 0 AND computed_progress_pct <= 100));
   GRANT UPDATE (kr_type, start_value, target_value, current_value,
                 milestone_checkpoints, computed_progress_pct) ON okr_krs TO cyberos_app;
   ```

3. **MUST** validate per-type at `validator.rs::validate(kr)` per DEC-1973:
   - hit_target → target_value REQUIRED; others NULL
   - improvement → start_value + target_value REQUIRED; checkpoints NULL
   - milestone → checkpoints REQUIRED (array of `{name, completed: bool}`); start/target NULL

4. **MUST** compute progress at `progress_calc.rs::compute(kr) → NUMERIC(5,2)` per DEC-1972:
   - hit_target: `current_value >= target_value ? 100 : 0`
   - improvement: `((current - start) / (target - start)) * 100`, capped 0-100
   - milestone: `(completed_count / total_count) * 100`

5. **MUST** be deterministic per DEC-1972 — pure function, same inputs → same output.

6. **MUST** emit 3 memory audit kinds per DEC-1974. PII per FR-MEMORY-111: KR text hashed; type + progress ok.

7. **MUST** thread trace_id from set/compute → audit.

8. **MUST NOT** skip validation per DEC-1973 (reject 400 on wrong-type config).

9. **MUST NOT** use non-deterministic progress (no now(), no random).

---

## §2 — Why this design

**Why 3 types (DEC-1970)?** Covers all KR patterns; closed set prevents type sprawl. Industry standard (Doerr OKR formulation).

**Why per-type fields (DEC-1973)?** Type-specific schema prevents confusion (no checkpoints on improvement KRs).

**Why deterministic (DEC-1972)?** Weekly check-in numbers must be reproducible — auditors expect "same data = same number".

**Why JSONB for milestones (DEC-1973)?** Variable-length array; simpler than separate table.

---

## §3 — API contract

Sample KR with type:
```json
{
  "kr_id": "uuid",
  "title": "Increase MRR from $100k to $150k",
  "kr_type": "improvement",
  "start_value": 100000,
  "target_value": 150000,
  "current_value": 125000,
  "computed_progress_pct": 50.00
}
```

Sample milestone KR:
```json
{
  "kr_type": "milestone",
  "milestone_checkpoints": [
    {"name": "Sign 3 design partners", "completed": true},
    {"name": "Ship MVP", "completed": true},
    {"name": "First $10k revenue", "completed": false}
  ],
  "computed_progress_pct": 66.67
}
```

---

## §4 — Acceptance criteria
1. **kr_type enum cardinality 3**. 2. **hit_target requires target_value**. 3. **improvement requires start+target**. 4. **milestone requires checkpoints**. 5. **Wrong-type fields rejected (sev-3 audit)**. 6. **hit_target progress 0 or 100**. 7. **improvement progress capped 0-100**. 8. **milestone progress = completed/total ratio**. 9. **3 memory audit kinds emitted**. 10. **PII scrubbed (KR text SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Deterministic (pure function)**. 14. **rust_decimal precision**. 15. **CHECK constraint on progress 0-100**. 16. **Append-only via REVOKE except 6 cols**. 17. **Type change requires new KR (immutable type)**. 18. **Empty checkpoints array → progress=0**. 19. **Start = Target rejected (improvement)**. 20. **Negative progress impossible (clamp)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn improvement_progress_50pct() {
    let kr = mk_kr_improvement(100, 200, 150);
    assert_eq!(progress_calc::compute(&kr), dec!(50.00));
}

#[tokio::test]
async fn hit_target_binary() {
    let kr = mk_kr_hit_target(100, 99);
    assert_eq!(progress_calc::compute(&kr), dec!(0));
    let kr2 = mk_kr_hit_target(100, 100);
    assert_eq!(progress_calc::compute(&kr2), dec!(100));
}

#[tokio::test]
async fn milestone_validation_requires_checkpoints() {
    let r = create_kr_milestone_without_checkpoints().await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-OKR-001.
**Downstream:** FR-OKR-003 (auto-progress reads computed_progress_pct).
**Cross-module:** FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Wrong-type fields set | validator | 400 | use right type |
| Invalid kr_type | CHECK | 400 | use valid enum |
| Progress > 100 | clamp | 100 max | inherent |
| Progress < 0 | clamp | 0 min | inherent |
| Improvement start == target | reject | 400 | non-degenerate |
| Milestone empty checkpoints | progress=0 | inherent | inherent |
| Decimal precision drift | rust_decimal | inherent | inherent |
| Type change | reject (immutable) | 409 | new KR |
| Cross-tenant write | RLS | 403 | inherent |
| Concurrent progress update | last-writer-wins | inherent | inherent |

## §11 — Implementation notes
- §11.1 progress_calc pure: `(kr_type, fields) → Decimal`.
- §11.2 Computed_progress_pct cached on row; updated on current_value change.
- §11.3 memory audit body: kr_id, kr_type, progress; title SHA256.
- §11.4 Milestone JSONB schema: `[{name: str, completed: bool, completed_at?: timestamp}]`.
- §11.5 Future kr types via enum addition (e.g. "burndown") + new calc branch.

---

*End of FR-OKR-002 spec.*
