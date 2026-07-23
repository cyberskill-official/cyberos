---
id: TASK-LEARN-001
title: "LEARN skill tree schema — 1-5 mastery levels per skill per Member with parent-child skill graph"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: learn
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-HR-001, TASK-LEARN-002, TASK-LEARN-003, TASK-LEARN-004, TASK-MEMORY-111]
depends_on: [TASK-HR-001]
blocks: [TASK-LEARN-002, TASK-LEARN-004]

source_pages:
  - website/docs/modules/learn.html#skill-tree

source_decisions:
  - DEC-2080 2026-05-17 — Skill tree: hierarchical (parent → child) with up to 4 levels of nesting; closed enum for top-level domain
  - DEC-2081 2026-05-17 — Closed enum `skill_domain` = {engineering, design, product, sales, finance, ops, legal, general}; cardinality 8
  - DEC-2082 2026-05-17 — Closed enum `mastery_level` = {1, 2, 3, 4, 5}; cardinality 5 (1=beginner, 5=expert)
  - DEC-2083 2026-05-17 — Per-Member mastery per skill — append-only; corrections via new row (audit lineage)
  - DEC-2084 2026-05-17 — memory audit kinds: learn.skill_added, learn.mastery_set, learn.mastery_corrected

language: rust 1.81
service: cyberos/services/learn/
new_files:
  - services/learn/migrations/0001_skill_tree_mastery.sql
  - services/learn/src/skill_tree/mod.rs
  - services/learn/src/skill_tree/validator.rs
  - services/learn/src/handlers/skill_routes.rs
  - services/learn/src/audit/skill_events.rs
  - services/learn/tests/skill_domain_enum_cardinality_test.rs
  - services/learn/tests/skill_mastery_enum_cardinality_test.rs
  - services/learn/tests/skill_parent_child_depth_test.rs
  - services/learn/tests/skill_mastery_append_only_test.rs
  - services/learn/tests/skill_audit_emission_test.rs

modified_files:
  - services/learn/src/lib.rs

allowed_tools:
  - file_read: services/learn/**
  - file_write: services/learn/{src,tests,migrations}/**
  - bash: cd services/learn && cargo test skill

disallowed_tools:
  - mutate prior mastery row (per DEC-2083)
  - nesting depth > 4 (per DEC-2080)

effort_hours: 6
subtasks:
  - "0.4h: 0001_skill_tree_mastery.sql"
  - "0.4h: skill_tree/mod.rs"
  - "0.5h: validator.rs"
  - "0.4h: handlers/skill_routes.rs"
  - "0.3h: audit/skill_events.rs"
  - "2.5h: tests — 5 test files"
  - "1.0h: skill tree UI"
  - "0.5h: docs"

risk_if_skipped: "Without skill tree, performance reviews + hiring rely on subjective judgment. Without DEC-2082 5-level scale, mastery scoring inconsistent. Without DEC-2083 append-only, mastery revisions break audit lineage."
---

## §1 — Description (BCP-14 normative)

The LEARN service **MUST** ship skill tree at `services/learn/src/skill_tree/` with hierarchical skill graph + per-member mastery + append-only audit, 3 memory audit kinds.

1. **MUST** validate `skill_domain` against closed enum per DEC-2081, `mastery_level` per DEC-2082.

2. **MUST** define tables at migration `0001`:
   ```sql
   CREATE TABLE learn_skills (
     skill_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     name TEXT NOT NULL,
     description TEXT,
     domain TEXT NOT NULL
       CHECK (domain IN ('engineering','design','product','sales','finance','ops','legal','general')),
     parent_skill_id UUID REFERENCES learn_skills(skill_id),
     depth INT NOT NULL DEFAULT 0 CHECK (depth >= 0 AND depth <= 4),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, name, parent_skill_id)
   );
   CREATE INDEX skills_parent_idx ON learn_skills(tenant_id, parent_skill_id) WHERE parent_skill_id IS NOT NULL;
   ALTER TABLE learn_skills ENABLE ROW LEVEL SECURITY;
   CREATE POLICY skills_rls ON learn_skills
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (name, description, parent_skill_id, depth) ON learn_skills TO cyberos_app;

   CREATE TABLE learn_member_mastery (
     mastery_row_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     member_id UUID NOT NULL,
     skill_id UUID NOT NULL REFERENCES learn_skills(skill_id),
     mastery_level INT NOT NULL CHECK (mastery_level >= 1 AND mastery_level <= 5),
     assessed_by UUID NOT NULL,
     assessment_kind TEXT NOT NULL,  -- 'self' | 'peer' | 'council' | 'system'
     valid_from DATE NOT NULL,
     valid_to DATE,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX mastery_member_skill_idx ON learn_member_mastery(tenant_id, member_id, skill_id, valid_from DESC);
   ALTER TABLE learn_member_mastery ENABLE ROW LEVEL SECURITY;
   CREATE POLICY mastery_rls ON learn_member_mastery
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_member_mastery FROM cyberos_app;
   ```

3. **MUST** enforce nesting depth ≤4 per DEC-2080 at `validator.rs::validate(parent_skill_id)`.

4. **MUST** be append-only per DEC-2083 — corrections via new row with new valid_from; prior row's valid_to set.

5. **MUST** expose endpoints:
   ```text
   POST /v1/learn/skills                 (CHRO)
   POST /v1/learn/members/{id}/mastery   body: {skill_id, mastery_level, assessment_kind}
   GET  /v1/learn/members/{id}/mastery   (current per-skill)
   GET  /v1/learn/skills/tree            (hierarchical view)
   ```

6. **MUST** emit 3 memory audit kinds per DEC-2084. PII per TASK-MEMORY-111: skill names + descriptions text SHA-256 hashed; member_id + level ok.

7. **MUST** thread trace_id from set → audit.

8. **MUST NOT** mutate prior mastery row per DEC-2083.

9. **MUST NOT** create cycles in parent_skill_id (validator check).

---

## §2 — Why this design

**Why 4-level depth (DEC-2080)?** Bounded to prevent infinite trees; covers domain → discipline → skill → subskill.

**Why 8 domains (DEC-2081)?** Top-level taxonomy covers CyberSkill business; closed enum prevents sprawl.

**Why 1-5 mastery (DEC-2082)?** Industry-standard scale (Bloom + similar).

**Why append-only (DEC-2083)?** Audit lineage — promotion decisions reference mastery history.

---

## §3 — API contract

Sample skill tree:
```json
[
  {"skill_id": "uuid", "name": "Rust", "domain": "engineering", "depth": 0,
   "children": [{"skill_id": "uuid", "name": "Async Rust", "depth": 1}]}
]
```

Sample mastery set:
```json
POST /v1/learn/members/{id}/mastery
{
  "skill_id": "uuid",
  "mastery_level": 3,
  "assessment_kind": "council",
  "valid_from": "2026-06-01"
}
```

---

## §4 — Acceptance criteria
1. **skill_domain enum cardinality 8**. 2. **mastery_level CHECK 1-5**. 3. **parent depth ≤4**. 4. **Cycle prevention**. 5. **Append-only mastery**. 6. **UNIQUE(tenant, name, parent) on skills**. 7. **3 memory audit kinds emitted**. 8. **PII scrubbed (skill text SHA256)**. 9. **RLS denies cross-tenant**. 10. **CHRO-only skill create**. 11. **assessment_kind tagged (self/peer/council/system)**. 12. **valid_from + valid_to range**. 13. **Trace_id preserved**. 14. **Tree query recursive CTE**. 15. **Current mastery = max valid_from**. 16. **Self-reference parent rejected**. 17. **Skill rename via UPDATE OK**. 18. **Parent change via UPDATE OK (depth recomputed)**. 19. **Cross-tenant parent FK rejected**. 20. **Mastery FK to skill enforced**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn mastery_level_1_to_5_enforced() {
    for level in 1..=5 {
        let r = ctx.set_mastery(ctx.member_id, ctx.skill_id, level).await;
        assert!(r.is_ok());
    }
    for bad in [0, 6, 100] {
        let r = ctx.set_mastery(ctx.member_id, ctx.skill_id, bad).await;
        assert!(r.is_err());
    }
}

#[tokio::test]
async fn parent_depth_5_rejected() {
    let chain = ctx.build_skill_chain(5).await;  // chain of 5 = depth 4
    let r = ctx.add_skill_under(chain.last(), "depth5").await;
    assert!(r.is_err());
}

#[tokio::test]
async fn append_only_no_update() {
    let ctx = TestContext::with_mastery_row().await;
    let r = ctx.try_update_mastery_row(ctx.mastery_row_id).await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-HR-001. **Downstream:** TASK-LEARN-002 (degrees+certs), TASK-LEARN-003 (VP rollup), TASK-LEARN-004 (Council). **Cross-module:** TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Domain not in enum | CHECK | 400 | use valid |
| Depth > 4 | validator | reject | restructure tree |
| Cycle | validator | reject | inherent |
| Mastery out of range | CHECK | 400 | use 1-5 |
| Duplicate skill name | UNIQUE | 409 | rename |
| Mastery on deleted skill | FK | 404 | reactivate skill |
| Cross-tenant parent | FK + RLS | 404 | inherent |
| Decimal precision | not applicable | inherent | inherent |
| Mastery row append race | inherent | both append | inherent |
| Self-ref parent | validator | reject | use different parent |

## §11 — Implementation notes
- §11.1 Validator recursive depth check: walk parent chain; reject if depth > 4 or cycle.
- §11.2 Current mastery query: `SELECT * FROM mastery WHERE member=$1 AND skill=$2 AND valid_from <= today AND (valid_to IS NULL OR valid_to > today) ORDER BY valid_from DESC LIMIT 1`.
- §11.3 memory audit body: member_id, skill_id, mastery_level, assessment_kind; skill text SHA256.
- §11.4 Tree view via recursive CTE.
- §11.5 Assessment_kind drives TASK-LEARN-004 council vs self vs peer flows.

---

*End of TASK-LEARN-001 spec.*
