# Reconcile dossier — TASK-OBS-005 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-OBS-005 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

W3C TraceContext correlation end-to-end: claims `crates/cyberos-obs-sdk/src/{tracecontext,logging,exemplar}.rs`, three crate tests (`tracecontext`, `log_enrichment`, `end_to_end_correlation`), and propagation wiring across `services/ai-gateway/**`, `services/auth/**`, `services/chat/**`, `services/memory/**`; modifies `red.rs`. `depends_on: [TASK-OBS-001, TASK-OBS-003, TASK-OBS-004]` — the first two are themselves stuck `implementing` (see their dossiers) and OBS-004 is `ready_to_implement`. Created 2026-05-15, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- All three claimed source files EXIST by name in the as-built crate home `services/shared/cyberos-obs-sdk/src/`: `tracecontext.rs`, `logging.rs`, `exemplar.rs` (plus `red.rs`, the claimed modify-target).
- `traceparent` propagation evidence appears in `services/obs-proxy/src/auth.rs`; coverage across the four claimed service trees is unverified.
- Zero of the three claimed crate tests exist (no crate `tests/` directory at all).
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — path-literal for the crate half (files exist under `services/shared/`); the `services/**` glob claims are unverifiable as written and end-to-end correlation (the task's entire point) has no committed test evidence.
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Route back per §1.3; rework = re-path the crate claims, modernize, audit, adopt the three source files, then prove the end-to-end correlation AC — the one deliverable that distinguishes this task from OBS-003 and currently has zero evidence. Dependency note for sequencing: two of its three dependencies are being reconciled in this same triage and the third (OBS-004, LangSmith traces) has not started; the operator may want OBS-005's re-entry ordered after those verdicts land.

## Gate question

The tracecontext/logging/exemplar sources exist in the as-built SDK crate; end-to-end correlation has no test evidence and two dependencies are themselves stuck. Route back to re-path-audit-adopt with the correlation proof as the remainder (recommended), resume (blocked), or hold pending the dependency verdicts?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-OBS-005
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-OBS-005 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-005-tracecontext-correlation/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: crates/cyberos-obs-sdk/src/tracecontext.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/src/logging.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/src/exemplar.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/tracecontext_test.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/log_enrichment_test.rs; absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/end_to_end_correlation_test.rs; absent at HEAD and on disk: services/ai-gateway/**; absent at HEAD and on disk: services/auth/**; absent at HEAD and on disk: services/chat/**; absent at HEAD and on disk: services/memory/**; absent at HEAD and on disk: crates/cyberos-obs-sdk/src/red.rs

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-005-tracecontext-correlation/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/obs/TASK-OBS-005-tracecontext-correlation)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/tracecontext.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/logging.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/exemplar.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/tracecontext_test.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/log_enrichment_test.rs
- absent at HEAD and on disk: crates/cyberos-obs-sdk/tests/end_to_end_correlation_test.rs
- absent at HEAD and on disk: services/ai-gateway/**
- absent at HEAD and on disk: services/auth/**
- absent at HEAD and on disk: services/chat/**
- absent at HEAD and on disk: services/memory/**
- absent at HEAD and on disk: crates/cyberos-obs-sdk/src/red.rs

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/obs/TASK-OBS-005-tracecontext-correlation
-rw-r--r--@ 1 stephencheng  staff  27222 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "W3C TraceContext correlation across logs/metrics/traces/AI-traces — propagate, embed, exemplar, end-to-end CI test"
10:created_at: 2026-05-15T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-OBS-001, TASK-OBS-003, TASK-OBS-004]
68:effort_hours: 8
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  crates/cyberos-obs-sdk/src/tracecontext.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/src/logging.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/src/exemplar.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/tests/tracecontext_test.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/tests/log_enrichment_test.rs | last-commit: NONE | ABSENT
  crates/cyberos-obs-sdk/tests/end_to_end_correlation_test.rs | last-commit: NONE | ABSENT
  .github/workflows/obs-correlation-gate.yml | last-commit: 34b46d7c 2026-07-15 | on-disk
  services/ai-gateway/** | last-commit: 069d4dff 2026-07-20 | ABSENT
  services/auth/** | last-commit: 06334c94 2026-07-20 | ABSENT
  services/chat/** | last-commit: 34b46d7c 2026-07-15 | ABSENT
  services/memory/** | last-commit: 069d4dff 2026-07-20 | ABSENT
  crates/cyberos-obs-sdk/src/red.rs | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
