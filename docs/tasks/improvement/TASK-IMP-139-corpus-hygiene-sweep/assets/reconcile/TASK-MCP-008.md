# Reconcile dossier — TASK-MCP-008 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-MCP-008 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

MCP Elicitation — server-initiated structured prompts for mid-call user input (clarifications, confirmations, missing args) with timeout and cancellation. Claims migration `0012_mcp_elicitations.sql`, a ten-file `services/mcp/src/elicitation/` module (`request`, `response`, `poll`, `cancel`, `timeout_job`, `validate`, `file_upload`, `nats_push`), handlers, audit events, and twelve `elicitation_*` integration tests; modifies `gating/elicit.rs` and `tasks/mod.rs`. `depends_on: [TASK-MCP-001, TASK-MCP-004]` (both `done`). Created 2026-05-17, `effort_hours: 6`, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- The live service `services/mcp-gateway/` carries `src/elicitation.rs` and `src/elicitation_pg.rs` — the elicitation feature with a Postgres persistence split, consolidated single-file layout (house style, same shape as `tasks.rs`/`tasks_pg.rs`).
- None of the twelve claimed tests exist by name; no claimed migration; no `audit/` module; the claimed cross-feature touchpoints (`gating/elicit.rs`, `tasks/mod.rs`) do not exist as paths, though their consolidated homes (`gating.rs`, `tasks.rs`) do.
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — path-literal: claimed module tree absent; committed equivalents exist. Timeout, re-elicit-after-invalid, cross-tenant-denial, and file-upload ACs have zero test evidence.
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Route back per §1.3; rework = task@1 modernization against the consolidated layout, audit, adopt `elicitation.rs`/`elicitation_pg.rs`, verify the interaction ACs (timeout/cancel/cross-tenant) with real tests. Same rationale pattern as MCP-006/007: the code has a home, the claims have no evidence, the spec cannot re-enter as-is. Not `resume`; not `on_hold`.

## Gate question

Elicitation exists in `mcp-gateway` as two consolidated files; the spec claims a ten-file module, a migration, and twelve tests — none present by name. Route back to modernize-audit-adopt (recommended), resume (blocked), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-MCP-008
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-MCP-008 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-008-elicitation/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/mcp/migrations/0012_mcp_elicitations.sql; absent at HEAD and on disk: services/mcp/src/elicitation/mod.rs; absent at HEAD and on disk: services/mcp/src/elicitation/request.rs; absent at HEAD and on disk: services/mcp/src/elicitation/response.rs; absent at HEAD and on disk: services/mcp/src/elicitation/poll.rs; absent at HEAD and on disk: services/mcp/src/elicitation/cancel.rs; absent at HEAD and on disk: services/mcp/src/elicitation/timeout_job.rs; absent at HEAD and on disk: services/mcp/src/elicitation/validate.rs; absent at HEAD and on disk: services/mcp/src/elicitation/file_upload.rs; absent at HEAD and on disk: services/mcp/src/elicitation/nats_push.rs; absent at HEAD and on disk: services/mcp/src/audit/elicitation_events.rs; absent at HEAD and on disk: services/mcp/src/handlers/elicitation_routes.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_request_response_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_timeout_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_cancellation_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_type_enum_cardinality_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_schema_validation_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_re_elicit_after_invalid_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_confirmation_integration_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_file_upload_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_idempotency_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_cross_tenant_denied_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_rate_limit_test.rs; absent at HEAD and on disk: services/mcp/tests/elicitation_audit_emission_test.rs; absent at HEAD and on disk: services/mcp/src/server_registry.rs; absent at HEAD and on disk: services/mcp/src/lib.rs; absent at HEAD and on disk: services/mcp/src/gating/elicit.rs; absent at HEAD and on disk: services/mcp/src/tasks/mod.rs

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-008-elicitation/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/mcp/TASK-MCP-008-elicitation)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/mcp/migrations/0012_mcp_elicitations.sql
- absent at HEAD and on disk: services/mcp/src/elicitation/mod.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/request.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/response.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/poll.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/cancel.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/timeout_job.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/validate.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/file_upload.rs
- absent at HEAD and on disk: services/mcp/src/elicitation/nats_push.rs
- absent at HEAD and on disk: services/mcp/src/audit/elicitation_events.rs
- absent at HEAD and on disk: services/mcp/src/handlers/elicitation_routes.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_request_response_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_timeout_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_cancellation_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_type_enum_cardinality_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_schema_validation_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_re_elicit_after_invalid_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_confirmation_integration_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_file_upload_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_idempotency_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_cross_tenant_denied_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_rate_limit_test.rs
- absent at HEAD and on disk: services/mcp/tests/elicitation_audit_emission_test.rs
- absent at HEAD and on disk: services/mcp/src/server_registry.rs
- absent at HEAD and on disk: services/mcp/src/lib.rs
- absent at HEAD and on disk: services/mcp/src/gating/elicit.rs
- absent at HEAD and on disk: services/mcp/src/tasks/mod.rs

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/mcp/TASK-MCP-008-elicitation
-rw-r--r--@ 1 stephencheng  staff  50217 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "MCP Elicitation — server-initiated structured prompts for mid-call user input (clarifications, confirmations, missing args) with timeout + cancellation"
10:created_at: 2026-05-17T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-MCP-001, TASK-MCP-004]
120:effort_hours: 6
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
608d95fb 2026-07-18 fix(docs/tasks): flatten build_envelope nested-map frontmatter - FM-001 0 (IMP-117 1.8/AC7)
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  services/mcp/migrations/0012_mcp_elicitations.sql | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/mod.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/request.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/response.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/poll.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/cancel.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/timeout_job.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/validate.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/file_upload.rs | last-commit: NONE | ABSENT
  services/mcp/src/elicitation/nats_push.rs | last-commit: NONE | ABSENT
  services/mcp/src/audit/elicitation_events.rs | last-commit: NONE | ABSENT
  services/mcp/src/handlers/elicitation_routes.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_request_response_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_timeout_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_cancellation_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_type_enum_cardinality_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_schema_validation_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_re_elicit_after_invalid_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_confirmation_integration_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_file_upload_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_idempotency_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_cross_tenant_denied_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_rate_limit_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/elicitation_audit_emission_test.rs | last-commit: NONE | ABSENT
  services/mcp/src/server_registry.rs | last-commit: NONE | ABSENT
  services/mcp/src/lib.rs | last-commit: NONE | ABSENT
  services/mcp/src/gating/elicit.rs | last-commit: NONE | ABSENT
  services/mcp/src/tasks/mod.rs | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
