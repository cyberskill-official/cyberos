# Reconcile dossier — TASK-MCP-003 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-MCP-003 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

SEP-986 naming-convention validator (`cyberos.{module}.{verb}_{noun}`) enforced at skill registration plus a CI grep gate. Claims a Rust module `services/mcp/src/naming/{mod,validator,module_registry}.rs`, audit emission `services/mcp/src/audit/naming_events.rs`, five `sep986_*` integration tests, `scripts/check_sep986_naming.sh`, and `.github/workflows/mcp-sep986-check.yml`. `depends_on: [TASK-MCP-001]` (status `done`). Created 2026-05-17, `effort_hours: 3`, `verify: T`. The body is written in pre-task@1 engineering-spec `## §N` grammar.

## What the tree and git history show

- `services/mcp/` does not exist and never did (no commit touches it). The live MCP service is `services/mcp-gateway/` — first commit 2026-05-19, two days after this spec was created; 32 commits since.
- The claimed source trio exists file-for-file under the as-built path: `services/mcp-gateway/src/naming/{mod,validator,module_registry}.rs`.
- Three of the five claimed tests exist by exact name: `services/mcp-gateway/tests/sep986_{regex,module_validation,verb_enum_cardinality}_test.rs`. Missing by name: `sep986_ci_grep_test.rs`, `sep986_audit_emission_test.rs`.
- The CI half is committed at HEAD: `scripts/check_sep986_naming.sh` (78 lines; last touched 73bc6d9d 2026-07-17 in an executable-bit sweep) and `.github/workflows/mcp-sep986-check.yml` (24 lines; last touched 5f9f8526 2026-07-16 in an Actions-upgrade sweep) — both landed via unrelated sweeps, meaning they predate those commits and have survived maintenance.
- No `audit/` module under `services/mcp-gateway/src/` — the `naming_events` audit-emission half is unverified.
- The task folder contains only `spec.md`; its git history is exclusively corpus-wide migration sweeps (rename, FM-001 conformance, unwrap) — no task-specific commit ever touched it.

## Evidence classification

- R1 red — real, two-part: (a) FM-004 `template_ambiguous` — the body's `## §N` grammar predates the task@1 discipline, so the machine floor stops the file; (b) `audit.md` absent — the spec never passed the draft gate that today's lifecycle requires. (The additional FM-112 marker findings are the corpus-endemic Gate-1 debt, not task-specific drift.)
- R2 red — real: no phase artefacts in either home.
- R3 absent — expected for out-of-band work.
- R4 red — path-literal, materially misleading here: 10/12 claimed paths are absent because the spec says `services/mcp/` while the code shipped in `services/mcp-gateway/`; the other 2 claimed paths (CI gate) are committed at HEAD.
- R5 — not executed (see Method notes); cited suites are Rust test binaries, three of which exist under as-built paths.

## Recommended operator verdict: route_back

The work substantially exists; the PROCESS evidence does not, and the spec itself cannot re-enter the chain (it fails the machine floor on its own grammar and was never audited). Route back per STATUS-REFERENCE §1.3 with reasons from this report, then rework as a spec-modernization: task@1 grammar, paths corrected to `services/mcp-gateway/`, audit to 10/10 — and at re-entry ADOPT the existing code (the reconcile `adopt_candidate` path), closing the visible residual gap: `sep986_ci_grep_test.rs`, `sep986_audit_emission_test.rs`, and the audit-emission module. `resume` is blocked by the failing machine floor; `on_hold` would re-hide code that is already shipped and CI-guarded.

## Gate question

TASK-MCP-003 has claimed `implementing` for ~10 weeks. The validator and its CI gate exist and are committed under `services/mcp-gateway/` (as-built paths differ from the spec); the spec fails lint on pre-discipline grammar and was never audited. Route back for a modernize-audit-adopt pass (recommended, ~small: 3h original estimate and most code exists), resume as-is (blocked by the machine floor), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-MCP-003
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-MCP-003 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-003-sep986-naming-validator/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/mcp/src/naming/mod.rs; absent at HEAD and on disk: services/mcp/src/naming/validator.rs; absent at HEAD and on disk: services/mcp/src/naming/module_registry.rs; absent at HEAD and on disk: services/mcp/src/audit/naming_events.rs; absent at HEAD and on disk: services/mcp/tests/sep986_verb_enum_cardinality_test.rs; absent at HEAD and on disk: services/mcp/tests/sep986_regex_test.rs; absent at HEAD and on disk: services/mcp/tests/sep986_module_validation_test.rs; absent at HEAD and on disk: services/mcp/tests/sep986_ci_grep_test.rs; absent at HEAD and on disk: services/mcp/tests/sep986_audit_emission_test.rs; absent at HEAD and on disk: services/mcp/src/lib.rs

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/mcp/TASK-MCP-003-sep986-naming-validator/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/mcp/TASK-MCP-003-sep986-naming-validator)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/mcp/src/naming/mod.rs
- absent at HEAD and on disk: services/mcp/src/naming/validator.rs
- absent at HEAD and on disk: services/mcp/src/naming/module_registry.rs
- absent at HEAD and on disk: services/mcp/src/audit/naming_events.rs
- absent at HEAD and on disk: services/mcp/tests/sep986_verb_enum_cardinality_test.rs
- absent at HEAD and on disk: services/mcp/tests/sep986_regex_test.rs
- absent at HEAD and on disk: services/mcp/tests/sep986_module_validation_test.rs
- absent at HEAD and on disk: services/mcp/tests/sep986_ci_grep_test.rs
- absent at HEAD and on disk: services/mcp/tests/sep986_audit_emission_test.rs
- absent at HEAD and on disk: services/mcp/src/lib.rs

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/mcp/TASK-MCP-003-sep986-naming-validator
-rw-r--r--@ 1 stephencheng  staff  8252 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "MCP SEP-986 naming convention validator — `cyberos.{module}.{verb}_{noun}` pattern enforced at skill registration + CI gate"
10:created_at: 2026-05-17T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-MCP-001]
66:effort_hours: 3
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
608d95fb 2026-07-18 fix(docs/tasks): flatten build_envelope nested-map frontmatter - FM-001 0 (IMP-117 1.8/AC7)
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  services/mcp/src/naming/mod.rs | last-commit: NONE | ABSENT
  services/mcp/src/naming/validator.rs | last-commit: NONE | ABSENT
  services/mcp/src/naming/module_registry.rs | last-commit: NONE | ABSENT
  services/mcp/src/audit/naming_events.rs | last-commit: NONE | ABSENT
  scripts/check_sep986_naming.sh | last-commit: 73bc6d9d 2026-07-17 | on-disk
  .github/workflows/mcp-sep986-check.yml | last-commit: 5f9f8526 2026-07-16 | on-disk
  services/mcp/tests/sep986_verb_enum_cardinality_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/sep986_regex_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/sep986_module_validation_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/sep986_ci_grep_test.rs | last-commit: NONE | ABSENT
  services/mcp/tests/sep986_audit_emission_test.rs | last-commit: NONE | ABSENT
  services/mcp/src/lib.rs | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
