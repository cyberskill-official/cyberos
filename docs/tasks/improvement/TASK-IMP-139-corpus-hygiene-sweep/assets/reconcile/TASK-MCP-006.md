# Reconcile dossier — TASK-MCP-006 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-MCP-006 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

Tool-annotation gating: destructive / write / external-effect MCP tools require explicit confirm or Elicitation pre-execution (per MCP 2025-11-25). Claims three migrations (gating policy, pending confirmations, decisions log), a seven-file `services/mcp/src/gating/` module (`decision`, `policy`, `confirm`, `elicit`, `bypass`, `drift_detector`), two handlers, audit events, and ten `gating_*` integration tests. `depends_on: [TASK-MCP-001, TASK-MCP-004]` (both `done`). Created 2026-05-17, `effort_hours: 6`, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- The live service `services/mcp-gateway/` carries `src/gating.rs` and `src/annotations.rs` — a consolidated two-file implementation of the claimed seven-file module tree (annotation model + gating decisions in single-file layout, consistent with the gateway's house style: `elicitation.rs`, `tasks.rs` are likewise single files).
- None of the ten claimed `gating_*` tests exist; no gating migrations by claimed name; no `audit/` module.
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 markers endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — path-literal: the claimed module tree does not exist, but the functionality has a committed home (`gating.rs` + `annotations.rs`). Whether the confirm-TTL, bypass-token, audit-only-mode, and precedence ACs are actually satisfied is unverified — zero of the claimed tests exist.
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Same class as the rest of the MCP set: code present under as-built layout, process evidence absent, spec unable to re-enter the chain. Route back per §1.3; rework = task@1 modernization with paths matching the consolidated layout, audit, adopt, then verify the gating ACs with real tests (the largest visible gap — this is safety-relevant behavior with zero test evidence). Not `resume` (machine floor fails); not `on_hold` (guardrail code deserves verification, not parking).

## Gate question

Gating/annotations code exists consolidated in `mcp-gateway`; the spec claims a seven-file module and ten tests, none present. Route back to modernize-audit-adopt with gating-AC tests as the substantive remainder (recommended), resume (blocked), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-MCP-006
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-MCP-006 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-006-tool-annotation-gating/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/mcp/migrations/0006_mcp_gating_policy.sql; absent at HEAD and on disk: services/mcp/migrations/0007_mcp_pending_confirmations.sql; absent at HEAD and on disk: services/mcp/migrations/0008_mcp_gating_decisions_log.sql; absent at HEAD and on disk: services/mcp/src/gating/mod.rs; absent at HEAD and on disk: services/mcp/src/gating/decision.rs; absent at HEAD and on disk: services/mcp/src/gating/policy.rs; absent at HEAD and on disk: services/mcp/src/gating/confirm.rs; absent at HEAD and on disk: services/mcp/src/gating/elicit.rs; absent at HEAD and on disk: services/mcp/src/gating/bypass.rs; absent at HEAD and on disk: services/mcp/src/gating/drift_detector.rs; absent at HEAD and on disk: services/mcp/src/handlers/tool_confirm.rs; absent at HEAD and on disk: services/mcp/src/handlers/gating_policy_admin.rs; absent at HEAD and on disk: services/mcp/src/audit/gating_events.rs; absent at HEAD and on disk: services/mcp/tests/gating_annotation_precedence_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_destructive_requires_confirm_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_readonly_fast_path_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_confirm_ttl_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_bypass_token_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_audit_only_mode_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_decision_enum_cardinality_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_policy_tenant_admin_only_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_drift_detection_test.rs; absent at HEAD and on disk: services/mcp/tests/gating_audit_emission_test.rs; absent at HEAD and on disk: services/mcp/src/handlers/tools_call.rs; absent at HEAD and on disk: services/mcp/src/server_registry.rs; absent at HEAD and on disk: services/mcp/src/lib.rs

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-006-tool-annotation-gating/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/mcp/TASK-MCP-006-tool-annotation-gating)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/mcp/migrations/0006_mcp_gating_policy.sql
- absent at HEAD and on disk: services/mcp/migrations/0007_mcp_pending_confirmations.sql
- absent at HEAD and on disk: services/mcp/migrations/0008_mcp_gating_decisions_log.sql
- absent at HEAD and on disk: services/mcp/src/gating/mod.rs
- absent at HEAD and on disk: services/mcp/src/gating/decision.rs
- absent at HEAD and on disk: services/mcp/src/gating/policy.rs
- absent at HEAD and on disk: services/mcp/src/gating/confirm.rs
- absent at HEAD and on disk: services/mcp/src/gating/elicit.rs
- absent at HEAD and on disk: services/mcp/src/gating/bypass.rs
- absent at HEAD and on disk: services/mcp/src/gating/drift_detector.rs
- absent at HEAD and on disk: services/mcp/src/handlers/tool_confirm.rs
- absent at HEAD and on disk: services/mcp/src/handlers/gating_policy_admin.rs
- absent at HEAD and on disk: services/mcp/src/audit/gating_events.rs
- absent at HEAD and on disk: services/mcp/tests/gating_annotation_precedence_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_destructive_requires_confirm_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_readonly_fast_path_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_confirm_ttl_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_bypass_token_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_audit_only_mode_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_decision_enum_cardinality_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_policy_tenant_admin_only_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_drift_detection_test.rs
- absent at HEAD and on disk: services/mcp/tests/gating_audit_emission_test.rs
- absent at HEAD and on disk: services/mcp/src/handlers/tools_call.rs
- absent at HEAD and on disk: services/mcp/src/server_registry.rs
- absent at HEAD and on disk: services/mcp/src/lib.rs

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/mcp/TASK-MCP-006-tool-annotation-gating
-rw-r--r--@ 1 stephencheng  staff  49733 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "MCP tool-annotation gating — destructive / write / external-effect tools require explicit confirm or Elicitation pre-execution per MCP 2025-11-25 spec"
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
  services/mcp/migrations/0006_mcp_gating_policy.sql | last-commit: NONE | ABSENT
  services/mcp/migrations/0007_mcp_pending_confirmations.sql | last-commit: NONE | ABSENT
  services/mcp/migrations/0008_mcp_gating_decisions_log.sql | last-commit: NONE | ABSENT
  services/mcp/src/gating/mod.rs | last-commit: NONE | ABSENT
  services/mcp/src/gating/decision.rs | last-commit: NONE | ABSENT
  services/mcp/src/gating/policy.rs | last-commit: NONE | ABSENT
  services/mcp/src/gating/confirm.rs | last-commit: NONE | ABSENT
  services/mcp/src/gating/elicit.rs | last-commit: NONE | ABSENT
  services/mcp/src/gating/bypass.rs | last-commit: NONE | ABSENT
  services/mcp/src/gating/drift_detector.rs | last-commit: NONE | ABSENT
  services/mcp/src/handlers/tool_confirm.rs | last-commit: NONE | ABSENT
  services/mcp/src/handlers/gating_policy_admin.rs | last-commit: NONE | ABSENT
  services/mcp/src/audit/gating_events.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_annotation_precedence_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_destructive_requires_confirm_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_readonly_fast_path_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_confirm_ttl_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_bypass_token_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_audit_only_mode_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_decision_enum_cardinality_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_policy_tenant_admin_only_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_drift_detection_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/gating_audit_emission_test.rs | last-commit: NONE | ABSENT
  services/mcp/src/handlers/tools_call.rs | last-commit: NONE | ABSENT
  services/mcp/src/server_registry.rs | last-commit: NONE | ABSENT
  services/mcp/src/lib.rs | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
