# Reconcile dossier — TASK-OBS-009 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-OBS-009 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

Chain-of-custody manifest: exportable, verifiable manifests over the compliance views — claims `services/obs-compliance-view/src/manifest_pdf.rs`, three tests (`manifest`, `manifest_verify`, `manifest_interrupted`), and an auditor-facing format doc that was never authored (claimed path `services/obs-compliance-view/docs/manifest-format.md` — ABSENT; as-built surface is `src/manifest.rs` + `bin/verify_manifest.rs`); modifies the views and export files (brace-glob claims). `depends_on: [TASK-OBS-008]` — itself stuck `implementing` (see its dossier). Created 2026-05-15, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- The manifest surface EXISTS in `services/obs-compliance-view/src/`: `manifest.rs`, `manifest_signing.rs`, `proof.rs`, and a dedicated verifier binary `bin/verify_manifest.rs` — generation, signing, AND verification all have committed homes (first commit 2026-06-20 with the service).
- Absent by name: `manifest_pdf.rs` (no PDF path anywhere in the service), all three claimed tests (no `tests/` dir), and the auditor-facing format doc (never authored; the claim named `services/obs-compliance-view/docs/manifest-format.md` — ABSENT).
- The claimed modified_files use brace-glob shapes (`views/{eu_ai_act,pdpl,soc2,iso27001}.rs`) that never existed — the as-built layout is consolidated (see the OBS-008 dossier).
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — path-literal for the core (manifest+signing+verifier committed); substantive for the PDF export, the format doc, and all test evidence. The `verify_manifest` binary existing at all is notable positive evidence — verification was built, just never proven in CI.
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Route back per §1.3; rework = modernize, audit, adopt `manifest.rs`/`manifest_signing.rs`/`bin/verify_manifest.rs`, then close the gaps that matter for custody claims: the format document (auditors need the contract, not the code), interrupted-export behavior, and the PDF half if still wanted. Sequencing note: its dependency OBS-008 is reconciling in this same triage; route-back for both keeps the pair coherent, and the rework can legitimately merge the two into one modernized spec if the operator prefers (they share a service and a consolidated layout).

## Gate question

Manifest generation, signing, and a verifier binary are committed; the format doc, PDF export, and all tests are not. Route back to modernize-audit-adopt (optionally merged with OBS-008's rework) (recommended), resume (blocked), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-OBS-009
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-OBS-009 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-009-chain-of-custody-manifest/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/obs-compliance-view/src/manifest_pdf.rs; absent at HEAD and on disk: services/obs-compliance-view/tests/manifest_test.rs; absent at HEAD and on disk: services/obs-compliance-view/tests/manifest_verify_test.rs; absent at HEAD and on disk: services/obs-compliance-view/tests/manifest_interrupted_test.rs; absent at HEAD and on disk: services/obs-compliance-view/docs/manifest-format.md; absent at HEAD and on disk: services/obs-compliance-view/src/views/{eu_ai_act,pdpl,soc2,iso27001}.rs; absent at HEAD and on disk: services/obs-compliance-view/src/export/{pdf,json}.rs

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-009-chain-of-custody-manifest/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/obs/TASK-OBS-009-chain-of-custody-manifest)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/obs-compliance-view/src/manifest_pdf.rs
- absent at HEAD and on disk: services/obs-compliance-view/tests/manifest_test.rs
- absent at HEAD and on disk: services/obs-compliance-view/tests/manifest_verify_test.rs
- absent at HEAD and on disk: services/obs-compliance-view/tests/manifest_interrupted_test.rs
- absent at HEAD and on disk: services/obs-compliance-view/docs/manifest-format.md
- absent at HEAD and on disk: services/obs-compliance-view/src/views/{eu_ai_act,pdpl,soc2,iso27001}.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/export/{pdf,json}.rs

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/obs/TASK-OBS-009-chain-of-custody-manifest
-rw-r--r--@ 1 stephencheng  staff  28223 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "Chain-of-custody manifest with Ed25519 signature on every compliance export — PDF cover + JSON sidecar + audit row + verifier CLI"
10:created_at: 2026-05-15T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-OBS-008]
67:effort_hours: 8
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  services/obs-compliance-view/src/manifest.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-compliance-view/src/manifest_signing.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-compliance-view/src/manifest_pdf.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/bin/verify_manifest.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-compliance-view/tests/manifest_test.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/tests/manifest_verify_test.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/tests/manifest_interrupted_test.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/docs/manifest-format.md | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/views/{eu_ai_act,pdpl,soc2,iso27001}.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/export/{pdf,json}.rs | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
