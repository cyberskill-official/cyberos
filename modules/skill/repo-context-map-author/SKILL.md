---
# ── Identity ─────────────────────────────────────────────────────────
name: repo-context-map-author
description: >-
  Deep-scan the repo before any code is generated for a given task, and emit a `repo-context-map@1` capturing: (a) existing patterns the new code must follow (DI containers, error type, state-management style, logging convention), (b) database schemas + type interfaces in the task's declared module, (c) files outside the task's immediate domain that the implementation would touch, (d) the task's blast-radius estimate (file count + module count + cross-module edges), and (e) a flag if the task appears to belong in a different module than its catalogue placement. Used by chief-technology-officer/ship-tasks as step 1. Use when user asks to "draft a repo context map" or "create the repo context map". Do NOT use for "audit existing repo context map" (use repo-context-map-audit instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: repo-context-map@1
  cyberos-rubric-target: repo_context_map_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:fr/{task_id}/repo-context-map
audit:
  row_kind: repo_context_map_authored
  required_fields: [task_id, files_in_immediate_domain, files_outside_immediate_domain, modules_touched, blast_radius_score, existing_patterns_count]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: fr,        format: task@1, required: true }
  - { name: repo_root, format: absolute path,     required: true }
outputs:
  - { name: context_map, format: repo-context-map@1 }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - any task moving from `accepted` → `building`
  - workflow `chief-technology-officer/ship-tasks` step 1
blockers:
  - "repo has uncommitted divergent state — must be resolved first"
  - "task's declared module does not exist on disk — escalate to chief-product-officer"
---

# repo-context-map-author

## 1. Purpose

Build a static, code-aware snapshot of the parts of the repo that the
incoming task will interact with — **before** any code is generated. The
map is the input to the implementation-plan-author and the trigger for
the optional architecture-decision-record auto-ADR when a task's blast
radius exceeds the in-module threshold.

## 2. Output schema

```yaml
# repo-context-map@1
task_id: task-<MODULE>-<NNN>
generated_at: <ISO-8601>
task_module: <module name from task frontmatter>
repo_root: <absolute path>

# A. Existing patterns the new code must respect
existing_patterns:
  - { kind: error_type,           value: "anyhow::Error / thiserror::Error / Result<T,E>",         pinned_in: "<file:line>" }
  - { kind: di_container,         value: "axum Extension / global static / dependency-injection crate", pinned_in: "<file:line>" }
  - { kind: state_management,     value: "Arc<Mutex<T>> / actor / tokio::sync::RwLock",            pinned_in: "<file:line>" }
  - { kind: logging,              value: "tracing / log / println!",                                pinned_in: "<file:line>" }
  - { kind: test_framework,       value: "cargo test / pytest / vitest",                           pinned_in: "<file:line>" }

# B. Database + type surface in this module
schemas:
  - { table_or_type: "...", defined_in: "<file:line>", consumed_by: ["..."] }

# C. Files outside the task's immediate domain that this task will likely touch
files_outside_immediate_domain:
  - { path: "<absolute>", reason: "<one-sentence reason>", risk: low | medium | high }

# D. Blast radius
blast_radius:
  files_in_immediate_domain: <int>
  files_outside_immediate_domain: <int>
  modules_touched: <int>
  cross_module_edges: <int>
  score: <int 0-100, weighted by risk>

# E. Module-placement sanity check
module_placement_warning: null | "task appears to belong in module <X>, not <Y>; rationale: ..."
```

## 3. Quality gates

- `existing_patterns` covers at least: error_type, logging, test_framework
  (the three patterns every cyberos service uses).
- `files_outside_immediate_domain.length > 3` MUST trigger the ADR
  branch in the workflow (steps 3-4).
- `schemas` is non-empty for any task that declares a `migrations` or
  `data:` block in its frontmatter.
- `module_placement_warning` is null OR is escalated to chief-product-officer
  before the chain continues.

## 4. Chains to

`repo-context-map-audit` then (conditionally) `architecture-decision-record-author`
when `files_outside_immediate_domain.length > 3`, then `edge-case-matrix-author`.

---

*End of repo-context-map-author SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `PIPELINE.md` (chain binding + HALT points), `INVARIANTS.md`, `envelopes/` (I/O schemas), `references/FAILURE_MODES.md`, `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
