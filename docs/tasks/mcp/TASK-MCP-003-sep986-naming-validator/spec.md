---
id: TASK-MCP-003
title: "MCP SEP-986 naming validator + CI gate (cyberos.module.verb_noun)"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: mcp
priority: p0
status: reviewing
entered_via: rework
routed_back_count: 1
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
  - DEC-2363 2026-05-17 — Module name validated against the 23-module registry; unknown module → reject
  - DEC-2364 2026-05-17 — memory audit kinds: mcp.skill_name_validated, mcp.skill_name_rejected, mcp.naming_ci_check_passed, mcp.naming_ci_check_failed

language: rust 1.81
service: cyberos/services/mcp-gateway/
new_files:
  - services/mcp-gateway/src/naming/mod.rs
  - services/mcp-gateway/src/naming/validator.rs
  - services/mcp-gateway/src/naming/module_registry.rs
  - scripts/check_sep986_naming.sh
  - .github/workflows/mcp-sep986-check.yml
  - services/mcp-gateway/tests/sep986_verb_enum_cardinality_test.rs
  - services/mcp-gateway/tests/sep986_regex_test.rs
  - services/mcp-gateway/tests/sep986_module_validation_test.rs
  - services/mcp-gateway/tests/sep986_ci_grep_test.rs
  - services/mcp-gateway/tests/sep986_audit_emission_test.rs

modified_files:
  - services/mcp-gateway/src/lib.rs
  - services/mcp-gateway/src/federation/register.rs
  - services/mcp-gateway/src/router.rs
  - services/mcp-gateway/src/oauth/audit.rs

allowed_tools:
  - file_read: services/mcp-gateway/**
  - file_write: services/mcp-gateway/{src,tests}/**
  - bash: cd services && cargo test -p cyberos-mcp-gateway sep986

disallowed_tools:
  - register without validation (per DEC-2362)
  - bypass CI gate (per DEC-2362)
  - claim a standalone services/mcp/ tree as the live path

effort_hours: 3
subtasks:
  - "0.5h: naming/{mod,validator,module_registry}.rs"
  - "0.5h: registration + router audit emit"
  - "0.5h: CI workflow + bash-3.2-safe grep script"
  - "1.0h: sep986_* integration tests (regex/module/verb/ci/audit)"
  - "0.5h: batch/9a-mcp re-spec + audit"

risk_if_skipped: "Without naming validator, skill ID sprawl → discovery broken + collision risk. Without DEC-2362 CI gate, code drift introduces non-conforming names. Without DEC-2361 verb enum, every new skill invents its own pattern."
---

# TASK-MCP-003: MCP SEP-986 naming convention validator

## Summary

Enforce `cyberos.{module}.{verb}_{noun}` at skill registration and in CI. As-built surface lives under `services/mcp-gateway/src/naming/` (pure validator + 23-module registry + closed 15-verb enum), is wired into `federation::register::validate` / `router` register paths, emits the four DEC-2364 kinds via `oauth::audit`, and is guarded by `scripts/check_sep986_naming.sh` + `.github/workflows/mcp-sep986-check.yml`.

## Problem

The original engineering-spec claimed `services/mcp/` (never existed), a standalone `audit/naming_events.rs`, and five integration tests by name. The live crate is `mcp-gateway`; slices 1–3 already shipped the validator, registration gate, CI script, and audit helpers, but the process evidence failed FM-004 (task@1 frontmatter + `## §N` body) and two residual tests (`sep986_ci_grep_test`, `sep986_audit_emission_test`) were missing. The CI script also used bash-4-only `mapfile`, so it failed on macOS `/bin/bash` 3.2.

## Proposed Solution

Adopt the as-built layout:

- `naming/validator.rs` — `validate_sync`, `Sep986Verb` (15), precompiled regex
- `naming/module_registry.rs` — 23-module binary-search registry; `NAMING_EXEMPT_MODULES` for the demo fixture
- Registration reject path → `oauth::audit::skill_name_rejected`; success → `skill_name_validated`
- CI grep gate (bash 3.2+ portable) + Actions workflow
- Integration tests covering regex, modules, verb cardinality, CI grep (live + planted violation), and audit-kind surface pins

## Alternatives Considered

- **Resume the old engineering-spec as-is.** Rejected: FM-004 blocks re-entry; paths lie.
- **Standalone `services/mcp/src/audit/naming_events.rs`.** Rejected: house style puts MCP audit kinds on `oauth::audit` next to the other mcp.* kinds.
- **Drop the CI gate and rely on runtime only.** Rejected: DEC-2362 requires defense-in-depth at review time.

## Success Metrics

- Primary: non-conforming tool IDs cannot register; CI fails closed on planted violations; four DEC-2364 kind names remain present.
- Guardrail: `bash scripts/check_sep986_naming.sh` exits 0 on the live tree on macOS bash 3.2 and Ubuntu CI bash.

## Scope

In scope (as-built):

- `services/mcp-gateway/src/naming/**`
- Registration + router audit emit for validated/rejected
- `scripts/check_sep986_naming.sh` (bash 3.2+), `.github/workflows/mcp-sep986-check.yml`
- Five `sep986_*` integration test files under `services/mcp-gateway/tests/`

### Out of scope / Non-Goals

- A separate `services/mcp/` crate or `audit/naming_events.rs` module
- Emitting `naming_ci_check_passed/failed` from GitHub Actions into the live memory chain (helpers exist; CI exit code is the gate signal until CI can reach Postgres)
- Extending the verb enum or module registry without RFC / owner sign-off (governance tripwire already in cardinality tests)

## Dependencies

`depends_on: [TASK-MCP-001]`. Soft: TASK-MCP-002 module registry concepts; TASK-MEMORY-111 PII scrub (skill IDs are public identifiers).

## 1. Description (normative)

- 1.1 `Sep986Verb` MUST have exactly 15 variants matching DEC-2361; unknown verbs MUST fail validation.
- 1.2 `validate_sync(skill_id)` MUST enforce `^cyberos\.([a-z][a-z0-9_]*)\.([a-z]+)_([a-z][a-z0-9_]*)$` with module ∈ registry (or exempt), verb ∈ enum, noun snake_case.
- 1.3 Module registration MUST reject non-conforming tool IDs before they become callable (except `NAMING_EXEMPT_MODULES`).
- 1.4 CI MUST run `scripts/check_sep986_naming.sh` and fail closed on registry-module violations; the script MUST run on bash 3.2+.
- 1.5 `oauth::audit` MUST expose the four DEC-2364 kinds; router register success/reject MUST call validated/rejected emitters.
- 1.6 This adopt MUST NOT claim a `services/mcp/` tree or CI→BRAIN emission as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - verb enum cardinality 15 - test: `services/mcp-gateway/tests/sep986_verb_enum_cardinality_test.rs`
- [ ] AC 2 (traces_to: #1.2) - regex accepts valid / rejects camelCase, missing prefix, bad shape - test: `services/mcp-gateway/tests/sep986_regex_test.rs`
- [ ] AC 3 (traces_to: #1.2,#1.3) - module registry accepts 23 known modules, rejects unknown - test: `services/mcp-gateway/tests/sep986_module_validation_test.rs`
- [ ] AC 4 (traces_to: #1.3) - registration rejects non-conforming tool IDs - test: `services/mcp-gateway/src/federation/register.rs::validate_rejects_a_malformed_tool_name`
- [ ] AC 5 (traces_to: #1.4) - live tree passes CI grep; planted `cyberos.obs.triage` fails - test: `services/mcp-gateway/tests/sep986_ci_grep_test.rs`
- [ ] AC 6 (traces_to: #1.5) - four DEC-2364 kinds + router emit sites present - test: `services/mcp-gateway/tests/sep986_audit_emission_test.rs`
- [ ] AC 7 (traces_to: #1.4) - script is bash 3.2 portable (no `mapfile`) - verify: `scripts/check_sep986_naming.sh` header + `bash scripts/check_sep986_naming.sh` on macOS
- [ ] AC 8 (traces_to: #1.6) - Out of scope lists phantom `services/mcp/` paths; new_files cite mcp-gateway only - verify: this spec Scope / new_files

## Verification

```bash
bash scripts/check_sep986_naming.sh
cd services && cargo test -p cyberos-mcp-gateway --test sep986_regex_test --test sep986_module_validation_test --test sep986_verb_enum_cardinality_test --test sep986_ci_grep_test --test sep986_audit_emission_test
```

| Path | Covers |
|------|--------|
| `tests/sep986_regex_test.rs` | Shape / verb / error messages |
| `tests/sep986_module_validation_test.rs` | 23-module registry |
| `tests/sep986_verb_enum_cardinality_test.rs` | DEC-2361 cardinality tripwire |
| `tests/sep986_ci_grep_test.rs` | DEC-2362 CI gate live + planted |
| `tests/sep986_audit_emission_test.rs` | DEC-2364 kind surface + router wiring |
| `src/federation/register.rs` naming tests | Runtime registration gate |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9a-mcp`.
- **Scope:** Re-spec/adopt against as-built mcp-gateway; bash 3.2 CI fix; residual ci/audit tests; deferred CI→BRAIN emission ledgered.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9a-mcp adopt — TASK-MCP-003 re-spec against as-built mcp-gateway naming.*
