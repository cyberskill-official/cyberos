# Reconcile dossier — TASK-OBS-003 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-OBS-003 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
- claimed status: `implementing` · created 2026-05-15 · module `obs`
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

A RED-metrics SDK crate plus service instrumentation: claims `crates/cyberos-obs-sdk/{Cargo.toml, src/{lib,red,macros,cardinality_guard}.rs}` with four crate tests (`red`, `macro`, `cardinality`, `instrument_completeness`), and instrumentation across `services/ai-gateway`, `services/auth`, `services/chat`, `services/memory`. `depends_on: [TASK-OBS-001]` (itself stuck `implementing`; see its dossier). Created 2026-05-15, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- The crate EXISTS — at `services/shared/cyberos-obs-sdk/`, not the claimed `crates/` workspace (a `crates/` tree never existed in this repo). Source inventory: `lib.rs`, `red.rs`, `cardinality_guard.rs`, `logging.rs`, `exemplar.rs`, `tracecontext.rs`, `layer.rs` — every claimed source file by name except `macros.rs` (`layer.rs` is the plausible tower-layer equivalent).
- No crate `tests/` directory — zero of the four claimed test files exist.
- Instrumentation signal is present in at least `services/ai-gateway` (metrics/histogram references across router, server, cache modules); completeness across the four claimed services is unverified.
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — path-literal: the crate shipped under `services/shared/`; the four glob-shaped service claims (`services/chat/src/*` etc.) are unverifiable as written. Core deliverable exists at HEAD; test evidence absent.
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Route back per §1.3; rework = correct the crate path to `services/shared/cyberos-obs-sdk`, modernize to task@1, audit, adopt the crate, then verify the two ACs that matter and have no evidence: cardinality-guard behavior and per-service instrumentation completeness. Note the dependency wrinkle: this task's `depends_on` (OBS-001) is itself being reconciled — sequencing of the two verdicts belongs to the operator, though route-back for both is compatible in either order.

## Gate question

The SDK crate exists (different workspace path) with all claimed sources except macros; no tests exist. Route back to re-path, audit, adopt, and add the missing test evidence (recommended), resume (blocked by the machine floor), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-OBS-003
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-OBS-003 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-003-red-metrics/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: crates/cyberos-obs-sdk/Cargo.toml; absent at HEAD and on disk: crates/cyberos-obs-sdk/src/lib.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/src/red.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/src/macros.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/src/cardinality_guard.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/red_test.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/macro_test.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/cardinality_test.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/instrument_completeness_test.rs; absent at HEAD and on disk: services/ai-gateway/src/handlers/*.rs; absent at HEAD and on disk: services/auth/src/admin/*.rs; absent at HEAD and on disk: services/chat/src/*; absent at HEAD and on disk: services/memory/src/*

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-003-red-metrics/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/obs/TASK-OBS-003-red-metrics)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: crates/cyberos-obs-sdk/Cargo.toml
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/lib.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/red.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/macros.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/cardinality_guard.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/red_test.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/macro_test.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/cardinality_test.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/instrument_completeness_test.rs
- absent at HEAD and on disk: services/ai-gateway/src/handlers/*.rs
- absent at HEAD and on disk: services/auth/src/admin/*.rs
- absent at HEAD and on disk: services/chat/src/*
- absent at HEAD and on disk: services/memory/src/*

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/obs/TASK-OBS-003-red-metrics
-rw-r--r--@ 1 stephencheng  staff  27939 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "Per-service RED metrics (rate/errors/duration) via cyberos-obs-sdk shared crate with macro + CI lint + standardised buckets"
10:created_at: 2026-05-15T00:00:00+07:00
16:status: implementing
17:verify: T
27:depends_on: [TASK-OBS-001]
73:effort_hours: 8
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  crates/cyberos-obs-sdk/Cargo.toml | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/src/lib.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/src/red.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/src/macros.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/src/cardinality_guard.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/tests/red_test.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/tests/macro_test.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/tests/cardinality_test.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/tests/instrument_completeness_test.rs | last-commit: NONE | ABSENT
  services/ai-gateway/src/handlers/*.rs | last-commit: NONE | ABSENT
  services/auth/src/admin/*.rs | last-commit: NONE | ABSENT
  services/chat/src/* | last-commit: 34b46d7c 2026-07-15 | ABSENT
  services/memory/src/* | last-commit: 34b46d7c 2026-07-15 | ABSENT
  services/ai-gateway/Cargo.toml | last-commit: 11628138 2026-07-14 | on-disk
  services/auth/Cargo.toml | last-commit: 34b46d7c 2026-07-15 | on-disk
  services/chat/Cargo.toml | last-commit: 11628138 2026-07-14 | on-disk
  services/memory/Cargo.toml | last-commit: 11628138 2026-07-14 | on-disk
----- cited test suites in spec (existence only, NOT executed)
```
