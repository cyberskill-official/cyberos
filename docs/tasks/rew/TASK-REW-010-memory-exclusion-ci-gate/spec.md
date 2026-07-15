---
id: TASK-REW-010
title: "REW memory structural exclusion CI gate — no comp fields appear in memory-ingest paths; static analysis + runtime check"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: REW
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CISO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-REW-001, TASK-MEMORY-111, TASK-MEMORY-101]
depends_on: [TASK-REW-001]
blocks: []

source_pages:
  - website/docs/modules/rew.html#memory-exclusion

source_decisions:
  - DEC-2240 2026-05-17 — Structural invariant: comp data (amounts, gross/net VND, deductions) NEVER reaches memory ingest paths — static analysis at CI + runtime check
  - DEC-2241 2026-05-17 — Closed enum `exclusion_check_kind` = {static_grep, runtime_scrubber_validation, schema_field_blocklist, payload_audit}; cardinality 4
  - DEC-2242 2026-05-17 — Static analysis: grep for forbidden patterns (decrypted_amount, gross_vnd, net_vnd, etc.) in memory-ingest code paths; CI blocks PR on match
  - DEC-2243 2026-05-17 — Runtime: memory ingest path validates payload against schema blocklist; reject + sev-1 audit if comp fields detected
  - DEC-2244 2026-05-17 — memory audit kinds: rew.exclusion_ci_passed, rew.exclusion_ci_failed, rew.exclusion_runtime_violation, rew.exclusion_field_blocked

build_envelope:
  language: rust 1.81 + ci scripts
  service: cyberos/services/rew/
  new_files:
    - services/rew/src/exclusion/mod.rs
    - services/rew/src/exclusion/runtime_check.rs
    - services/rew/src/exclusion/field_blocklist.rs
    - services/rew/src/audit/exclusion_events.rs
    - .github/workflows/rew-memory-exclusion.yml
    - scripts/check_rew_memory_exclusion.sh
    - services/rew/tests/exclusion_kind_enum_cardinality_test.rs
    - services/rew/tests/exclusion_runtime_blocks_comp_test.rs
    - services/rew/tests/exclusion_static_grep_test.rs
    - services/rew/tests/exclusion_audit_emission_test.rs

  modified_files:
    - services/rew/src/lib.rs

  allowed_tools:
    - file_read: services/**
    - file_write: services/rew/{src,tests,migrations}/**
    - bash: cd services/rew && cargo test exclusion

  disallowed_tools:
    - bypass static analysis (per DEC-2242)
    - skip runtime check (per DEC-2243)

effort_hours: 3
subtasks:
  - "0.3h: exclusion/mod.rs"
  - "0.4h: runtime_check.rs"
  - "0.3h: field_blocklist.rs"
  - "0.2h: audit/exclusion_events.rs"
  - "0.3h: CI workflow + grep script"
  - "1.2h: tests — 4 test files"
  - "0.3h: docs"

risk_if_skipped: "Without CI gate, code drift can introduce comp leak. Without DEC-2243 runtime check, ingest bug bypasses static analysis. Without DEC-2244 audit, leak undetectable."
---

## §1 — Description (BCP-14 normative)

The REW service **MUST** ship memory exclusion gate at `services/rew/src/exclusion/` with static CI grep + runtime payload check + sev-1 audit on violation, 4 memory audit kinds.

1. **MUST** validate `exclusion_check_kind` against closed enum per DEC-2241.

2. **MUST** run static grep CI at `.github/workflows/rew-memory-exclusion.yml` per DEC-2242:
   - Pattern list: `decrypted_amount`, `gross_vnd`, `net_vnd`, `deductions_total`, `payslip_pdf_bytes`, etc.
   - Search in: any file under `services/rew/src/audit/`, `services/*/src/memory/`, memory ingest paths
   - Match → CI fails

3. **MUST** check runtime at `runtime_check.rs::validate_payload(json)` per DEC-2243:
   - Walk JSON, check each key against field_blocklist
   - Match → reject ingest + emit sev-1 audit

4. **MUST** define field blocklist at `field_blocklist.rs` per DEC-2241 — exhaustive list of forbidden field names.

5. **MUST** emit 4 memory audit kinds per DEC-2244. Audit body itself MUST NOT carry comp data.

6. **MUST** thread trace_id from ingest attempt → check → audit.

7. **MUST NOT** bypass static analysis per DEC-2242.

8. **MUST NOT** skip runtime check per DEC-2243.

---

## §2 — Why this design

**Why CI + runtime (DEC-2240)?** Defense in depth — static catches at compile-time; runtime catches anything that slipped through.

**Why CI grep (DEC-2242)?** Cheap + fast; catches 95% of accidental introductions at PR review.

**Why runtime payload check (DEC-2243)?** Final guarantee — even if CI somehow misses, runtime aborts the request.

---

## §3 — API contract

Sample runtime violation response:
```json
{
  "error": "memory_exclusion_violation",
  "field": "gross_vnd",
  "message": "Comp field detected in memory ingest payload; rejected per TASK-REW-010."
}
```

CI grep script:
```bash
#!/bin/bash
# scripts/check_rew_memory_exclusion.sh
FORBIDDEN_PATTERNS="decrypted_amount|gross_vnd|net_vnd|deductions_total|payslip_pdf_bytes"
SCOPES="services/rew/src/audit services/*/src/memory"
matches=$(grep -rE "$FORBIDDEN_PATTERNS" $SCOPES || true)
if [ -n "$matches" ]; then
  echo "memory exclusion violation:"
  echo "$matches"
  exit 1
fi
```

---

## §4 — Acceptance criteria
1. **exclusion_check_kind enum cardinality 4**. 2. **CI script greps forbidden patterns**. 3. **CI fails PR on match**. 4. **Runtime check rejects comp fields**. 5. **sev-1 audit on runtime violation**. 6. **4 memory audit kinds emitted (no comp data in them)**. 7. **Field blocklist documented + tested**. 8. **Audit body excludes comp by structure**. 9. **CI runs on every PR**. 10. **Tests verify CI script logic**. 11. **Tests verify runtime check**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Append-only audit (no UPDATE)**. 15. **Pattern list maintained + reviewed**. 16. **Runtime check perf < 1ms per payload**. 17. **Static check perf < 5s in CI**. 18. **False-positive handling (allowlist)**. 19. **CI gate cannot be skipped**. 20. **CHANGELOG.md entry on every pattern list update**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn runtime_blocks_gross_vnd() {
    let payload = json!({"member_id": "uuid", "gross_vnd": 30000000});
    let r = runtime_check::validate_payload(&payload);
    assert!(r.is_err());
    let audits = ctx.fetch_memory_audits("rew.exclusion_runtime_violation").await;
    assert!(!audits.is_empty());
}

#[tokio::test]
async fn ci_script_catches_pattern() {
    let test_dir = create_test_dir_with("services/audit/test.rs", "let x = decrypted_amount;");
    let r = run_shell("scripts/check_rew_memory_exclusion.sh", test_dir);
    assert!(!r.success());
}

#[tokio::test]
async fn audit_body_excludes_comp() {
    let audit = audit::emit("rew.payroll_committed", json!({"run_id": "uuid", "members_count": 30}), trace).await;
    let serialized = serde_json::to_string(&audit).unwrap();
    assert!(!serialized.contains("vnd"));  // no amount fields
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-REW-001.
**Cross-module:** TASK-MEMORY-111 (PII enforcement), TASK-MEMORY-101 (memory ingest paths to gate).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| CI grep misses pattern | manual review | sev-1 if leak | add pattern |
| Runtime check fails | catch | sev-1 + reject | inherent |
| False positive pattern | allowlist | inherent | refine |
| Pattern list outdated | quarterly review | sev-3 | update |
| Audit body leaks comp | tests catch | inherent | bug fix |
| Cross-tenant payload | RLS | inherent | inherent |
| Performance degradation | benchmark | tune | inherent |
| CI skipped (admin bypass) | sev-1 alert | inherent | governance |
| Runtime check disabled | config check | sev-1 | re-enable |
| Pattern injection via user input | escape | inherent | inherent |

## §11 — Implementation notes
- §11.1 Static grep pattern list maintained as constant in CI script; reviewed in every comp-related PR.
- §11.2 Runtime check uses serde_json::Value walk; recursive; sub-100µs typical.
- §11.3 Field blocklist constants documented in spec; tests verify list completeness.
- §11.4 memory audit body: kind, payload_id (no values); structurally safe.
- §11.5 CI integration: required check, cannot bypass.

---

*End of TASK-REW-010 spec.*
