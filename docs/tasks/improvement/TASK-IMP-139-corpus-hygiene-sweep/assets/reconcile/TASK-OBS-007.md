# Reconcile dossier — TASK-OBS-007 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-OBS-007 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
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

Alertmanager → CUO runbook routing: alerts webhook into a router that triages via a CUO skill against a runbooks corpus, with PagerDuty fallback and sev1-always-pages semantics. Claims `services/obs-router/src/ack_handler.rs`, four named tests (`triage`, `pagerduty_fallback`, `sev1_always_pages`, `cuo_skill_failure`), `skills/obs.triage-alert/SKILL.md` + `runbooks-corpus/`, and `deploy/obs/alertmanager-config.yaml`. `depends_on: [TASK-OBS-002 (done), TASK-OBS-003 (stuck implementing)]`. Created 2026-05-15, `verify: T`. Body in pre-task@1 `## §N` grammar.

## What the tree and git history show

- `services/obs-router/` EXISTS (first commit 069add8b 2026-06-20; 12 commits) and reads as a full implementation of the described pipeline: `alertmanager_webhook.rs`, `severity.rs`, `dedup.rs`, `route.rs`, `triage.rs`, `cuo_triage.rs`, `runbook.rs`, `pagerduty.rs`, `notify.rs`, `chat_post.rs`, `audit.rs`, `handle.rs`, `config.rs`, `error.rs` — the strongest functional equivalence in the OBS set.
- One test exists: `services/obs-router/tests/route_decision_test.rs` (not one of the four claimed names, but squarely in the claimed behavior space).
- Absent by name: `ack_handler.rs` (`handle.rs` is the plausible as-built counterpart), the `skills/obs.triage-alert/` tree (a root `skills/` directory does not exist in this repo), and `deploy/obs/alertmanager-config.yaml` (deploy/obs shipped compose-based, see the OBS-001 dossier).
- Task folder holds only `spec.md`; history is migration sweeps only.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — largely path-literal: the router shipped file-for-concept, one committed test covers route decisions; the alertmanager config and the CUO-skill artefact are the visible functional gaps.
- R5 — not executed (shared tree; Rust citations).

## Recommended operator verdict: route_back

Route back per §1.3; rework = modernize the spec to the as-built router layout, audit, adopt the service, and close the named gaps: the Alertmanager config that actually wires alerts into the webhook, the CUO-skill/runbooks-corpus artefact (or its as-built equivalent, `runbook.rs` + `cuo_triage.rs`, documented as such), and tests for the fallback/sev1 ACs. Not `resume` (machine floor fails); not `on_hold` (a paging pipeline with partial evidence is exactly what should not sit ambiguous).

## Gate question

The alert-routing service exists with 14 source files and a route-decision test; the wiring config and CUO-skill artefact claimed by the spec do not. Route back to modernize-audit-adopt with the wiring + fallback tests as the remainder (recommended), resume (blocked), or hold?

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-OBS-007
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-OBS-007 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-007-alertmanager-cuo-runbook-routing/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: services/obs-router/src/ack_handler.rs; absent at HEAD and on disk: services/obs-router/tests/triage_test.rs; absent at HEAD and on disk: services/obs-router/tests/pagerduty_fallback_test.rs; absent at HEAD and on disk: services/obs-router/tests/sev1_always_pages_test.rs; absent at HEAD and on disk: services/obs-router/tests/cuo_skill_failure_test.rs; absent at HEAD and on disk: skills/obs.triage-alert/SKILL.md; absent at HEAD and on disk: skills/obs.triage-alert/runbooks-corpus/.keep; absent at HEAD and on disk: deploy/obs/alertmanager-config.yaml

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-007-alertmanager-cuo-runbook-routing/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/obs/TASK-OBS-007-alertmanager-cuo-runbook-routing)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: services/obs-router/src/ack_handler.rs
- absent at HEAD and on disk: services/obs-router/tests/triage_test.rs
- absent at HEAD and on disk: services/obs-router/tests/pagerduty_fallback_test.rs
- absent at HEAD and on disk: services/obs-router/tests/sev1_always_pages_test.rs
- absent at HEAD and on disk: services/obs-router/tests/cuo_skill_failure_test.rs
- absent at HEAD and on disk: skills/obs.triage-alert/SKILL.md
- absent at HEAD and on disk: skills/obs.triage-alert/runbooks-corpus/.keep
- absent at HEAD and on disk: deploy/obs/alertmanager-config.yaml

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/obs/TASK-OBS-007-alertmanager-cuo-runbook-routing
-rw-r--r--@ 1 stephencheng  staff  24932 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "obs-router: Alertmanager → CUO obs.triage-alert@1 skill → CHAT (≥0.70 conf) OR PagerDuty + sev-1 always pages + ack-button + audit"
10:created_at: 2026-05-15T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: [TASK-OBS-002, TASK-OBS-003]
72:effort_hours: 10
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  services/obs-router/Cargo.toml | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-router/src/main.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-router/src/alertmanager_webhook.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-router/src/cuo_triage.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-router/src/chat_post.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-router/src/pagerduty.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-router/src/severity.rs | last-commit: 11628138 2026-07-14 | on-disk
  services/obs-router/src/ack_handler.rs | last-commit: NONE | ABSENT
  services/obs-router/tests/triage_test.rs | last-commit: NONE | ABSENT
  services/obs-router/tests/pagerduty_fallback_test.rs | last-commit: NONE | ABSENT
  services/obs-router/tests/sev1_always_pages_test.rs | last-commit: NONE | ABSENT
  services/obs-router/tests/cuo_skill_failure_test.rs | last-commit: NONE | ABSENT
  skills/obs.triage-alert/SKILL.md | last-commit: NONE | ABSENT
  skills/obs.triage-alert/runbooks-corpus/.keep | last-commit: NONE | ABSENT
  deploy/obs/alertmanager-config.yaml | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
