# Reconcile dossier — TASK-MCP-007 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-MCP-007 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
- claimed status: `implementing` · created 2026-05-17 · module `mcp`
- rungs: r1: red, r2: red, r3: absent, r4: red, r5: skipped · drift score 3/5 · tool recommendation (mechanical): `route_back`
- **recommended operator verdict: route_back**

> HITL: this dossier RECOMMENDS and executes nothing. The verdict is the operator's
> (skill hard rule; ship-tasks Reconcile entry §; TASK-IMP-139 spec §1.5). No status
> changed in producing it.

Method notes:
- R1–R4 ran read-only against the working tree. R5 (cited tests) was deliberately NOT
  executed: this triage ran in a shared working tree with concurrent batch/8 workers
  (suite execution belongs to the final sequential pass), and the spec's `test:`
  citations name Rust test binaries, which R5's repo-tracked sh/py/mjs/js/ts allowlist
  would refuse regardless. Cited-path existence was checked without execution
  (Appendix B).
- R1's lint red includes the corpus-endemic `# UNREVIEWED` markers (FM-112): this file
  is one of the 167 in TASK-IMP-139's Gate-1 marker set. That half of the red is corpus
  debt dispositioned separately under Gate 1, NOT task-specific drift. Task-specific R1
  findings are named in the classification below.

## What the spec says

MCP Tasks primitive — long-running tool calls with status polling, resume-on-reconnect, cancellation, and a per-task memory audit chain. The largest spec of the MCP set (`effort_hours: 10`): three migrations, a ten-file `services/mcp/src/tasks/` module (`create`, `status`, `cancel`, `list`, `worker_pool`, `checkpoint`, `expiry_job`, `progress`, `idempotency`), audit events, and fifteen `task_*` integration tests. `depends_on: [TASK-MCP-001, TASK-MCP-004]` (both `done`). Created 2026-05-17, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- The live service `services/mcp-gateway/` carries `src/tasks.rs` and `src/tasks_pg.rs` — the tasks primitive with a Postgres persistence split, consolidated single-file layout (house style), plus `src/db_slice_test.rs` (an in-crate DB test file).
- None of the fifteen claimed `task_*` tests exist by name; no claimed migrations (`0009..0011_mcp_task*`); no `audit/` module.
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — path-literal: claimed module tree absent; committed equivalents `tasks.rs`/`tasks_pg.rs` exist. This spec has the biggest claimed-vs-verified AC surface of the set (worker isolation, checkpoint resume, cancellation races, TTL expiry — all uncovered by any existing test file).
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Route back per §1.3; rework = task@1 modernization against the as-built consolidated layout, audit, adopt `tasks.rs`/`tasks_pg.rs`, then spend the remaining effort where the risk is: the concurrency/lifecycle ACs (races, resume, expiry) have zero test evidence. Of the five MCP tasks this one has the widest verification gap relative to its claims — worth naming in the rework's scope. Not `resume` (machine floor); not `on_hold`.

## Gate question

The tasks primitive exists in `mcp-gateway` as two consolidated files; the spec claims ten files, three migrations, and fifteen tests — none present by name. Route back to modernize-audit-adopt with concurrency-AC verification as the real remainder (recommended), resume (blocked), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-MCP-007
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-MCP-007 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-007-tasks-primitive/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/mcp/migrations/0009_mcp_tasks.sql; absent at HEAD and on disk: services/mcp/migrations/0010_mcp_task_checkpoints.sql; absent at HEAD and on disk: services/mcp/migrations/0011_mcp_task_progress_events.sql; absent at HEAD and on disk: services/mcp/src/tasks/mod.rs; absent at HEAD and on disk: services/mcp/src/tasks/create.rs; absent at HEAD and on disk: services/mcp/src/tasks/status.rs; absent at HEAD and on disk: services/mcp/src/tasks/cancel.rs; absent at HEAD and on disk: services/mcp/src/tasks/list.rs; absent at HEAD and on disk: services/mcp/src/tasks/worker_pool.rs; absent at HEAD and on disk: services/mcp/src/tasks/checkpoint.rs; absent at HEAD and on disk: services/mcp/src/tasks/expiry_job.rs; absent at HEAD and on disk: services/mcp/src/tasks/progress.rs; absent at HEAD and on disk: services/mcp/src/tasks/idempotency.rs; absent at HEAD and on disk: services/mcp/src/audit/task_events.rs; absent at HEAD and on disk: services/mcp/tests/task_create_async_test.rs; absent at HEAD and on disk: services/mcp/tests/task_status_poll_test.rs; absent at HEAD and on disk: services/mcp/tests/task_resume_after_reconnect_test.rs; absent at HEAD and on disk: services/mcp/tests/task_cancellation_test.rs; absent at HEAD and on disk: services/mcp/tests/task_cancellation_race_test.rs; absent at HEAD and on disk: services/mcp/tests/task_ttl_expiry_test.rs; absent at HEAD and on disk: services/mcp/tests/task_status_enum_cardinality_test.rs; absent at HEAD and on disk: services/mcp/tests/task_progress_unit_enum_cardinality_test.rs; absent at HEAD and on disk: services/mcp/tests/task_oversized_result_test.rs; absent at HEAD and on disk: services/mcp/tests/task_gating_integration_test.rs; absent at HEAD and on disk: services/mcp/tests/task_idempotency_test.rs; absent at HEAD and on disk: services/mcp/tests/task_worker_pool_isolation_test.rs; absent at HEAD and on disk: services/mcp/tests/task_checkpoint_resume_test.rs; absent at HEAD and on disk: services/mcp/tests/task_rate_limit_test.rs; absent at HEAD and on disk: services/mcp/tests/task_audit_emission_test.rs; absent at HEAD and on disk: services/mcp/src/handlers/tools_call.rs; absent at HEAD and on disk: services/mcp/src/server_registry.rs; absent at HEAD and on disk: services/mcp/src/lib.rs

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-007-tasks-primitive/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/mcp/TASK-MCP-007-tasks-primitive)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/mcp/migrations/0009_mcp_tasks.sql
- absent at HEAD and on disk: services/mcp/migrations/0010_mcp_task_checkpoints.sql
- absent at HEAD and on disk: services/mcp/migrations/0011_mcp_task_progress_events.sql
- absent at HEAD and on disk: services/mcp/src/tasks/mod.rs
- absent at HEAD and on disk: services/mcp/src/tasks/create.rs
- absent at HEAD and on disk: services/mcp/src/tasks/status.rs
- absent at HEAD and on disk: services/mcp/src/tasks/cancel.rs
- absent at HEAD and on disk: services/mcp/src/tasks/list.rs
- absent at HEAD and on disk: services/mcp/src/tasks/worker_pool.rs
- absent at HEAD and on disk: services/mcp/src/tasks/checkpoint.rs
- absent at HEAD and on disk: services/mcp/src/tasks/expiry_job.rs
- absent at HEAD and on disk: services/mcp/src/tasks/progress.rs
- absent at HEAD and on disk: services/mcp/src/tasks/idempotency.rs
- absent at HEAD and on disk: services/mcp/src/audit/task_events.rs
- absent at HEAD and on disk: services/mcp/tests/task_create_async_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_status_poll_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_resume_after_reconnect_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_cancellation_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_cancellation_race_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_ttl_expiry_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_status_enum_cardinality_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_progress_unit_enum_cardinality_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_oversized_result_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_gating_integration_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_idempotency_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_worker_pool_isolation_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_checkpoint_resume_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_rate_limit_test.rs
- absent at HEAD and on disk: services/mcp/tests/task_audit_emission_test.rs
- absent at HEAD and on disk: services/mcp/src/handlers/tools_call.rs
- absent at HEAD and on disk: services/mcp/src/server_registry.rs
- absent at HEAD and on disk: services/mcp/src/lib.rs

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/mcp/TASK-MCP-007-tasks-primitive
-rw-r--r--@ 1 stephencheng  staff  53068 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "MCP Tasks primitive — long-running tool calls with status polling + resume-on-reconnect + cancellation + per-task memory audit chain"
10:created_at: 2026-05-17T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-MCP-001, TASK-MCP-004]
131:effort_hours: 10
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
608d95fb 2026-07-18 fix(docs/tasks): flatten build_envelope nested-map frontmatter - FM-001 0 (IMP-117 1.8/AC7)
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  services/mcp/migrations/0009_mcp_tasks.sql | last-commit: NONE | ABSENT
  services/mcp/migrations/0010_mcp_task_checkpoints.sql | last-commit: NONE | ABSENT
  services/mcp/migrations/0011_mcp_task_progress_events.sql | last-commit: NONE | ABSENT
  services/mcp/src/tasks/mod.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/create.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/status.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/cancel.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/list.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/worker_pool.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/checkpoint.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/expiry_job.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/progress.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/idempotency.rs | last-commit: NONE | ABSENT
  services/mcp/src/audit/task_events.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_create_async_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_status_poll_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_resume_after_reconnect_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_cancellation_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_cancellation_race_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_ttl_expiry_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_status_enum_cardinality_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_progress_unit_enum_cardinality_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_oversized_result_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_gating_integration_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_idempotency_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_worker_pool_isolation_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_checkpoint_resume_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_rate_limit_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/task_audit_emission_test.rs | last-commit: NONE | ABSENT
  services/mcp/src/handlers/tools_call.rs | last-commit: NONE | ABSENT
  services/mcp/src/server_registry.rs | last-commit: NONE | ABSENT
  services/mcp/src/lib.rs | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
