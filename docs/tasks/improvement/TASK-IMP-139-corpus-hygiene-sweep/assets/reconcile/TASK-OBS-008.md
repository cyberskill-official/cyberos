# Reconcile dossier — TASK-OBS-008 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-OBS-008 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

Compliance view scoping: per-regime views (EU AI Act, PDPL, SOC 2, ISO 27001) over the audit chain with scoped access, exports, and a chain proof. Claims `services/obs-compliance-view/src/views/{eu_ai_act,pdpl,soc2,iso27001}.rs`, `export/{pdf,json}.rs`, `chain_proof.rs`, four tests (`eu_ai_act`, `pdpl`, `cross_tenant`, `chain_proof`), and `deploy/obs/grafana/dashboards/compliance.json`. `depends_on: [TASK-OBS-002]` (done). Created 2026-05-15, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- `services/obs-compliance-view/` EXISTS (first commit f1d16ca8 2026-06-20; 8 commits): `views.rs` (all four regimes referenced — 7 hits for eu_ai_act/pdpl/soc2/iso27001), `proof.rs` (the chain_proof counterpart), `auth.rs` (scoping), `query.rs`, `summary.rs`, `window.rs`, `pii_scan.rs`, plus `manifest.rs`/`manifest_signing.rs`/`bin/verify_manifest.rs` (TASK-OBS-009's surface).
- Consolidated single-file layout vs the claimed per-regime file tree — same house-style delta as the MCP set.
- No `tests/` directory in the service (zero of the four claimed tests); no `export/{pdf,json}.rs` by name (whether `summary.rs`/`query.rs` cover JSON export is unverified; PDF has no visible counterpart); no `compliance.json` dashboard under `deploy/obs/grafana/` (only `provisioning/` exists).
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — path-literal for the views/proof core (exists consolidated); substantive for exports (PDF unverified) and the dashboard (absent). Cross-tenant scoping — the security-relevant AC — has `auth.rs` as its home but no test evidence.
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Route back per §1.3; rework = modernize to the as-built consolidated layout, audit, adopt `views.rs`/`proof.rs`/`auth.rs`, then verify the ACs with no evidence: cross-tenant denial, export formats (JSON confirmed, PDF likely missing), and the Grafana dashboard. Not `resume`; not `on_hold` — compliance-facing claims deserve the verification pass.

## Gate question

The four compliance views and the chain proof exist consolidated in `obs-compliance-view`; exports, the dashboard, and every claimed test are absent by name. Route back to modernize-audit-adopt with cross-tenant + export verification as the remainder (recommended), resume (blocked), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-OBS-008
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-OBS-008 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-008-compliance-view-scoping/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/obs-compliance-view/src/views/mod.rs; absent at HEAD and on disk: services/obs-compliance-view/src/views/eu_ai_act.rs; absent at HEAD and on disk: services/obs-compliance-view/src/views/pdpl.rs; absent at HEAD and on disk: services/obs-compliance-view/src/views/soc2.rs; absent at HEAD and on disk: services/obs-compliance-view/src/views/iso27001.rs; absent at HEAD and on disk: services/obs-compliance-view/src/export/pdf.rs; absent at HEAD and on disk: services/obs-compliance-view/src/export/json.rs; absent at HEAD and on disk: services/obs-compliance-view/src/chain_proof.rs; absent at HEAD and on disk: services/obs-compliance-view/tests/eu_ai_act_test.rs; absent at HEAD and on disk: services/obs-compliance-view/tests/pdpl_test.rs; absent at HEAD and on disk: services/obs-compliance-view/tests/cross_tenant_test.rs; absent at HEAD and on disk: services/obs-compliance-view/tests/chain_proof_test.rs; absent at HEAD and on disk: deploy/obs/grafana/dashboards/compliance.json

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-008-compliance-view-scoping/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/obs/TASK-OBS-008-compliance-view-scoping)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/obs-compliance-view/src/views/mod.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/views/eu_ai_act.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/views/pdpl.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/views/soc2.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/views/iso27001.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/export/pdf.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/export/json.rs
- absent at HEAD and on disk: services/obs-compliance-view/src/chain_proof.rs
- absent at HEAD and on disk: services/obs-compliance-view/tests/eu_ai_act_test.rs
- absent at HEAD and on disk: services/obs-compliance-view/tests/pdpl_test.rs
- absent at HEAD and on disk: services/obs-compliance-view/tests/cross_tenant_test.rs
- absent at HEAD and on disk: services/obs-compliance-view/tests/chain_proof_test.rs
- absent at HEAD and on disk: deploy/obs/grafana/dashboards/compliance.json

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/obs/TASK-OBS-008-compliance-view-scoping
-rw-r--r--@ 1 stephencheng  staff  24736 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "obs-compliance-view: pre-built read-only views (EU AI Act / PDPL / SOC 2 / ISO 27001) over memory audit chain with Ed25519 chain-proof + tenant-scoped + PDF/JSON export"
10:created_at: 2026-05-15T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-OBS-002]
74:effort_hours: 14
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  services/obs-compliance-view/Cargo.toml | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-compliance-view/src/main.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-compliance-view/src/auth.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-compliance-view/src/views/mod.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/views/eu_ai_act.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/views/pdpl.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/views/soc2.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/views/iso27001.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/export/pdf.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/export/json.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/src/chain_proof.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/tests/eu_ai_act_test.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/tests/pdpl_test.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/tests/cross_tenant_test.rs | last-commit: NONE | ABSENT
  services/obs-compliance-view/tests/chain_proof_test.rs | last-commit: NONE | ABSENT
  deploy/obs/grafana/dashboards/compliance.json | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
