# Reconcile dossier — TASK-MCP-005 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-MCP-005 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

MCP Protected Resource Metadata (RFC 9728) at `/.well-known/oauth-protected-resource` — closed audience-binding advertisement for federated MCP clients. Claims a standalone PRM module: `services/mcp/src/prm/{mod,gateway,per_module,cache}.rs`, `handlers/prm_routes.rs`, `audit/prm_events.rs`, migration `0005_prm_drift_log.sql`, and eight `prm_*` integration tests. `depends_on: [TASK-MCP-004]` (status `done`). Created 2026-05-17, `effort_hours: 3`, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- `services/mcp/` never existed; the live service is `services/mcp-gateway/` (first commit 2026-05-19, 32 commits).
- PRM is implemented INSIDE the oauth module rather than as a standalone `prm/` tree: `services/mcp-gateway/src/oauth/prm.rs`, with `oauth/discovery.rs`, and PRM references in `router.rs` and `federation/registry.rs` (matches for `protected-resource` / RFC 9728 markers).
- None of the eight claimed `prm_*` tests exist anywhere; no migration matches `*prm_drift_log*`; no `audit/` module in the gateway source.
- Task folder holds only `spec.md`; folder history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 pre-discipline grammar; never audited. (FM-112 markers are the endemic Gate-1 debt.)
- R2 red — real: no phase artefacts.
- R3 absent — expected out-of-band.
- R4 red — path-literal AND architecture-literal: the claimed standalone-module layout was superseded by an oauth-module implementation. The functional core (PRM endpoint) exists at HEAD; the drift-log migration, routes-by-name, audit events, and all tests are unverified or absent.
- R5 — not executed (shared tree; Rust citations outside the ladder's executable set). None of the cited test files exist under any path.

## Recommended operator verdict: route_back

Weaker file-level equivalence than TASK-MCP-003 (one core file vs a claimed nine-file module), but the endpoint the task exists for is present. Route back per §1.3; rework = re-spec against the as-built `oauth/prm.rs` architecture, audit, adopt the code, then close the real delta: audience-binding and drift-detection ACs need tests that do not exist today. `resume` is blocked by the machine floor; `on_hold` hides a live endpoint behind a stale claim.

## Gate question

PRM shipped inside `mcp-gateway`'s oauth module; the spec claims a standalone module with eight tests, none of which exist. Route back to re-spec-and-adopt with the test delta as the remaining work (recommended), resume (blocked), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-MCP-005
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-MCP-005 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-005-protected-resource-metadata/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/mcp/migrations/0005_prm_drift_log.sql; absent at HEAD and on disk: services/mcp/src/prm/mod.rs; absent at HEAD and on disk: services/mcp/src/prm/gateway.rs; absent at HEAD and on disk: services/mcp/src/prm/per_module.rs; absent at HEAD and on disk: services/mcp/src/prm/cache.rs; absent at HEAD and on disk: services/mcp/src/handlers/prm_routes.rs; absent at HEAD and on disk: services/mcp/src/audit/prm_events.rs; absent at HEAD and on disk: services/mcp/tests/prm_gateway_test.rs; absent at HEAD and on disk: services/mcp/tests/prm_per_module_test.rs; absent at HEAD and on disk: services/mcp/tests/prm_etag_cache_test.rs; absent at HEAD and on disk: services/mcp/tests/prm_unknown_module_test.rs; absent at HEAD and on disk: services/mcp/tests/prm_unauth_public_test.rs; absent at HEAD and on disk: services/mcp/tests/prm_rate_limit_test.rs; absent at HEAD and on disk: services/mcp/tests/prm_drift_detection_test.rs; absent at HEAD and on disk: services/mcp/tests/prm_audit_emission_test.rs; absent at HEAD and on disk: services/mcp/src/lib.rs; absent at HEAD and on disk: services/mcp/src/server_registry.rs; absent at HEAD and on disk: services/mcp/Cargo.toml

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-005-protected-resource-metadata/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/mcp/TASK-MCP-005-protected-resource-metadata)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/mcp/migrations/0005_prm_drift_log.sql
- absent at HEAD and on disk: services/mcp/src/prm/mod.rs
- absent at HEAD and on disk: services/mcp/src/prm/gateway.rs
- absent at HEAD and on disk: services/mcp/src/prm/per_module.rs
- absent at HEAD and on disk: services/mcp/src/prm/cache.rs
- absent at HEAD and on disk: services/mcp/src/handlers/prm_routes.rs
- absent at HEAD and on disk: services/mcp/src/audit/prm_events.rs
- absent at HEAD and on disk: services/mcp/tests/prm_gateway_test.rs
- absent at HEAD and on disk: services/mcp/tests/prm_per_module_test.rs
- absent at HEAD and on disk: services/mcp/tests/prm_etag_cache_test.rs
- absent at HEAD and on disk: services/mcp/tests/prm_unknown_module_test.rs
- absent at HEAD and on disk: services/mcp/tests/prm_unauth_public_test.rs
- absent at HEAD and on disk: services/mcp/tests/prm_rate_limit_test.rs
- absent at HEAD and on disk: services/mcp/tests/prm_drift_detection_test.rs
- absent at HEAD and on disk: services/mcp/tests/prm_audit_emission_test.rs
- absent at HEAD and on disk: services/mcp/src/lib.rs
- absent at HEAD and on disk: services/mcp/src/server_registry.rs
- absent at HEAD and on disk: services/mcp/Cargo.toml

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/mcp/TASK-MCP-005-protected-resource-metadata
-rw-r--r--@ 1 stephencheng  staff  35519 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "MCP Protected Resource Metadata (RFC 9728) at `/.well-known/oauth-protected-resource` — closed audience-binding advertisement for federated MCP clients"
10:created_at: 2026-05-17T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-MCP-004]
99:effort_hours: 3
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
608d95fb 2026-07-18 fix(docs/tasks): flatten build_envelope nested-map frontmatter - FM-001 0 (IMP-117 1.8/AC7)
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  services/mcp/migrations/0005_prm_drift_log.sql | last-commit: NONE | ABSENT
  services/mcp/src/prm/mod.rs | last-commit: NONE | ABSENT
  services/mcp/src/prm/gateway.rs | last-commit: NONE | ABSENT
  services/mcp/src/prm/per_module.rs | last-commit: NONE | ABSENT
  services/mcp/src/prm/cache.rs | last-commit: NONE | ABSENT
  services/mcp/src/handlers/prm_routes.rs | last-commit: NONE | ABSENT
  services/mcp/src/audit/prm_events.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_gateway_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_per_module_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_etag_cache_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_unknown_module_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_unauth_public_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_rate_limit_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_drift_detection_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/prm_audit_emission_test.rs | last-commit: NONE | ABSENT
  services/mcp/src/lib.rs | last-commit: NONE | ABSENT
  services/mcp/src/server_registry.rs | last-commit: NONE | ABSENT
  services/mcp/Cargo.toml | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
