---
id: TASK-MCP-003
title: "MCP SEP-986 naming convention validator — `cyberos.{module}.{verb}_{noun}` pattern enforced at skill registration + CI gate"
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
module: MCP
priority: p0
status: implementing
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MCP-001, TASK-MCP-002, TASK-MEMORY-111]
depends_on: [TASK-MCP-001]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#sep-986

source_decisions:
  - DEC-2360 2026-05-17 — SEP-986 naming: skill ID = `cyberos.{module}.{verb}_{noun}` where module ∈ approved enum, verb ∈ approved enum, noun = snake_case identifier
  - DEC-2361 2026-05-17 — Closed enum `sep986_verb` = {get, list, create, update, delete, send, fetch, sync, validate, generate, execute, search, replay, accept, reject}; cardinality 15
  - DEC-2362 2026-05-17 — Validation at registration (TASK-MCP-001) + CI gate scanning code for skill_id constants
  - DEC-2363 2026-05-17 — Module name validated against active tenant modules list (TEN/HR/REW/EMAIL/etc.); unknown module → reject
  - DEC-2364 2026-05-17 — memory audit kinds: mcp.skill_name_validated, mcp.skill_name_rejected, mcp.naming_ci_check_passed, mcp.naming_ci_check_failed

language: rust 1.81
service: cyberos/services/mcp/
new_files:
  - services/mcp/src/naming/mod.rs
  - services/mcp/src/naming/validator.rs
  - services/mcp/src/naming/module_registry.rs
  - services/mcp/src/audit/naming_events.rs
  - scripts/check_sep986_naming.sh
  - .github/workflows/mcp-sep986-check.yml
  - services/mcp/tests/sep986_verb_enum_cardinality_test.rs
  - services/mcp/tests/sep986_regex_test.rs
  - services/mcp/tests/sep986_module_validation_test.rs
  - services/mcp/tests/sep986_ci_grep_test.rs
  - services/mcp/tests/sep986_audit_emission_test.rs

modified_files:
  - services/mcp/src/lib.rs

allowed_tools:
  - file_read: services/mcp/**
  - file_write: services/mcp/{src,tests}/**
  - bash: cd services/mcp && cargo test naming

disallowed_tools:
  - register without validation (per DEC-2362)
  - bypass CI gate (per DEC-2362)

effort_hours: 3
subtasks:
  - "0.3h: naming/mod.rs"
  - "0.4h: validator.rs"
  - "0.3h: module_registry.rs"
  - "0.3h: audit/naming_events.rs"
  - "0.4h: CI workflow + grep script"
  - "1.0h: tests — 5 test files"
  - "0.3h: docs"

risk_if_skipped: "Without naming validator, skill ID sprawl → discovery broken + collision risk. Without DEC-2362 CI gate, code drift introduces non-conforming names. Without DEC-2361 verb enum, every new skill invents its own pattern."
---

## §1 — Description (BCP-14 normative)

The MCP service **MUST** ship SEP-986 naming validator at `services/mcp/src/naming/` enforcing skill ID pattern at registration + CI gate, 4 memory audit kinds.

1. **MUST** validate `sep986_verb` against closed enum per DEC-2361.

2. **MUST** parse + validate at `validator.rs::validate(skill_id)` per DEC-2360:
   - Regex: `^cyberos\.([a-z][a-z0-9_]*)\.([a-z]+)_([a-z][a-z0-9_]*)$`
   - Extract module, verb, noun
   - Module ∈ TASK-MCP-002 registered modules
   - Verb ∈ closed enum
   - Noun: snake_case identifier

3. **MUST** hook into TASK-MCP-001 skill registration per DEC-2362 — reject with sev-2 audit if invalid.

4. **MUST** ship CI grep workflow per DEC-2362 — `.github/workflows/mcp-sep986-check.yml`:
   ```yaml
   - name: SEP-986 naming check
     run: bash scripts/check_sep986_naming.sh
   ```

5. **MUST** validate module per DEC-2363 at `module_registry.rs::is_valid_module(name)`:
   - Hardcoded list: ten, hr, rew, email, inv, crm, doc, kb, okr, res, learn, esop, cuo, mcp, auth, memory, ai, time, proj, chat, obs, portal, skill
   - Unknown → reject

6. **MUST** emit 4 memory audit kinds per DEC-2364. PII: skill IDs (public) ok.

7. **MUST** thread trace_id from validation → audit.

8. **MUST NOT** register without validation per DEC-2362.

9. **MUST NOT** bypass CI gate per DEC-2362.

---

## §2 — Why this design

**Why SEP-986 (DEC-2360)?** Consistent naming across modules; discovery by pattern; collision avoidance.

**Why verb enum (DEC-2361)?** Without bounded set, every developer invents synonyms (get/fetch/retrieve/read).

**Why CI gate (DEC-2362)?** Catches violations at PR review; runtime check is safety net.

---

## §3 — API contract

Sample validation request:
```json
POST /v1/mcp/naming/validate
{
  "skill_id": "cyberos.calendar.list_events"
}

{"valid": true, "module": "calendar", "verb": "list", "noun": "events"}
```

Invalid:
```json
{
  "skill_id": "cyberos.calendar.getEvents"   # camelCase noun + 'get' may be OK but format wrong
}

{"valid": false, "error": "noun must be snake_case"}
```

---

## §4 — Acceptance criteria
1. **sep986_verb enum cardinality 15**. 2. **Regex matches valid IDs**. 3. **Module ∈ registered list**. 4. **Verb ∈ closed enum**. 5. **Noun snake_case**. 6. **Registration gated**. 7. **CI grep catches violations**. 8. **4 memory audit kinds emitted**. 9. **PII: skill IDs (public) ok**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved**. 12. **Append-only validations log**. 13. **CI script in repo**. 14. **Bypass impossible (required check)**. 15. **Error messages specific**. 16. **Validation perf < 1ms**. 17. **Module list maintained in code**. 18. **New module addition documented**. 19. **Capitalization rejected**. 20. **Reserved verb additions need RFC**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn valid_skill_id_passes() {
    let ok = vec![
        "cyberos.calendar.list_events",
        "cyberos.email.send_message",
        "cyberos.inv.create_invoice",
    ];
    for id in ok {
        let r = validator::validate(id).await;
        assert!(r.is_ok());
    }
}

#[tokio::test]
async fn invalid_skill_id_rejected() {
    let bad = vec![
        "cyberos.calendar.listEvents",  // camelCase
        "calendar.list_events",  // missing prefix
        "cyberos.unknown.list_things",  // unknown module
        "cyberos.calendar.fetch_events",  // wait, fetch IS in enum
        "cyberos.calendar.retrieve_events",  // 'retrieve' not in enum
    ];
    let mut bad_count = 0;
    for id in bad {
        if validator::validate(id).await.is_err() { bad_count += 1; }
    }
    assert!(bad_count >= 3);
}

#[tokio::test]
async fn ci_grep_catches_non_conforming() {
    let test_dir = create_test_dir_with_code(r#"const SKILL: &str = "myskill.getStuff";"#);
    let r = run_shell("scripts/check_sep986_naming.sh", test_dir);
    assert!(!r.success());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-MCP-001.
**Cross-module:** TASK-MCP-002 (module registry), TASK-MEMORY-111 (audit).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Verb not in enum | regex + enum | reject; sev-2 audit | use valid verb |
| Module unknown | registry check | reject; sev-2 | add to registry |
| Camelcase noun | regex | reject | use snake_case |
| Missing prefix | regex | reject | add cyberos. |
| CI script bug | tests | CI fails | fix script |
| Validator perf | benchmark | inherent | inherent |
| Cross-tenant validation | RLS | inherent | inherent |
| Concurrent register | inherent | each isolated | inherent |
| Verb enum extension | RFC process | inherent | governance |
| Module list drift | maintained | inherent | code review |

## §11 — Implementation notes
- §11.1 Regex compiled once; reused for perf.
- §11.2 CI script greps `*.rs` and `*.toml` for skill_id constants; fails on non-conforming pattern.
- §11.3 memory audit body: skill_id, validation result; no PII (skill IDs public).
- §11.4 Module list reviewed quarterly; additions require module owner sign + RFC.
- §11.5 Verb enum additions require SEP RFC + community discussion.

---

*End of TASK-MCP-003 spec.*
