# Reconcile dossier — TASK-OBS-001 (TASK-IMP-139 Gate-2 triage)

- prepared: 2026-07-23, branch `batch/8-audit-hardening`, worktree at `be89966b`, by the TASK-IMP-139 unblocked-half worker
- instrument: `node tools/install/docs-tools/task-reconcile.mjs TASK-OBS-001 --repo <repo-root>` — the machine floor per `modules/skill/task-reconcile/SKILL.md`
- claimed status: `implementing` · created 2026-05-15 · module `obs`
- rungs: r1: red, r2: red, r3: absent, r4: red, r5: skipped · drift score 3/5 · tool recommendation (mechanical): `route_back`
- **recommended operator verdict: route_back (spec superseded — re-spec or close)**

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

An OTel-collector-based observability ingest stack: claims a config-file deployment layout — `deploy/obs/{otel-collector-config,loki-config,prometheus-config,tempo-config}.yaml`, `grafana/datasources.yaml`, `auth/tokens.example`, `scripts/{rotate_tokens,healthcheck}.sh` — and three shell test suites (`smoke`, `auth_required`, `buffer_survives_restart`). `depends_on: []`. Created 2026-05-15, `verify: T`. Body in pre-task@1 `## §N` grammar. This is the root of the OBS dependency chain (OBS-003 and OBS-005 depend on it).

## What the tree and git history show

- `deploy/obs/` EXISTS (first commit 1744f2d3 2026-06-19, 6 commits) but with a materially different architecture than the spec claims: `docker-compose.yml`, `Dockerfile.obs-proxy`, `prometheus/prometheus.yml`, `tempo/tempo.yaml`, `grafana/provisioning/`, `auth/{collector.token.live,tokens.live}`, `scripts/flag_tenant.sh`, `tests/sampling_test.sh`, `README.md`.
- The ingest path shipped as CUSTOM Rust services, not a vendored otel-collector config: `services/obs-collector` (first 2026-05-19, 10 commits; `auth.rs`, `config.rs`, `metrics.rs`) plus `services/obs-proxy`.
- No otel-collector config, and no Loki anywhere under `deploy/obs/` — the logs half of the claimed stack has no visible as-built counterpart.
- None of the three claimed test suites exist (the one present, `sampling_test.sh`, belongs to TASK-OBS-006, status `done`).
- Adjacent hygiene note (out of this triage's scope): `deploy/obs/auth/collector.token.live` and `tokens.live` are **local runtime secrets**, correctly untracked and double-gitignored (root `deploy/**/*.live` + directory rules), mode 0600; all-branch history only ever committed `.example` templates. Not a leak — hygiene note only; the spec's claimed `tokens.example` remains the committed shape.

## Evidence classification

- R1 red — real: FM-004 grammar + never audited (FM-112 endemic, Gate-1 scope).
- R2 red — real: no phase artefacts.
- R3 absent — expected.
- R4 red — and here the red is SUBSTANTIVE, not merely path-literal: the delivered architecture (custom collector + proxy services, compose-based deploy) contradicts the claimed one (vendored otel-collector + Loki + config files). This is spec obsolescence, not path drift.
- R5 — not executed; the claimed shell suites do not exist to run.

## Recommended operator verdict: route_back

Route back per §1.3 with a `spec_rejected` flavor: the tree evidences a deliberate architectural departure, so resuming or adopting against THIS spec would manufacture drift. The rework decision is genuinely the operator's: either re-spec the as-built collector/proxy stack (and answer the Loki/logs question explicitly) or close this task as superseded and let the as-built architecture get its own spec. `on_hold` is defensible if the obs stack is not currently a priority, but it leaves the root of the OBS dependency chain ambiguous while OBS-003/005 reconcile against it — a recorded route-back with reasons is cleaner.

## Gate question

The spec claims an otel-collector+Loki config stack; the tree shipped custom collector/proxy services with no Loki, plus live-looking token files where an example was claimed. Route back as spec-superseded and decide re-spec vs close (recommended), or hold? Resume is not credible against a contradicted architecture.

## Appendix A — verbatim reconcile-report@1 (tool output, unedited)

---
artefact: reconcile-report@1
task: TASK-OBS-001
claimed_status: implementing
rungs: { r1: red, r2: red, r3: absent, r4: red, r5: skipped }
drift_score: 3
recommendation: route_back
hitl: required
---

# Reconcile report - TASK-OBS-001 (claims `implementing`)

**Recommendation: route_back** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

- R1 spec integrity: task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-001-otel-collector/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file; audit.md absent - the spec was never audited
- R4 committed object: absent at HEAD and on disk: deploy/obs/otel-collector-config.yaml; absent at HEAD and on disk: deploy/obs/loki-config.yaml; absent at HEAD and on disk: deploy/obs/prometheus-config.yaml; absent at HEAD and on disk: deploy/obs/tempo-config.yaml; absent at HEAD and on disk: deploy/obs/grafana/datasources.yaml; absent at HEAD and on disk: deploy/obs/grafana/provisioning/dashboards/.keep; absent at HEAD and on disk: deploy/obs/auth/tokens.example; absent at HEAD and on disk: deploy/obs/scripts/rotate_tokens.sh; absent at HEAD and on disk: deploy/obs/scripts/healthcheck.sh; absent at HEAD and on disk: deploy/obs/tests/smoke_test.sh; absent at HEAD and on disk: deploy/obs/tests/auth_required_test.sh; absent at HEAD and on disk: deploy/obs/tests/buffer_survives_restart_test.sh

## Evidence ladder

### R1 spec integrity - **red**
- task-lint FAILED: error FM-004 /Users/stephencheng/Projects/CyberSkill/cyberos/docs/tasks/obs/TASK-OBS-001-otel-collector/spec.md:13 template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file
- audit.md absent - the spec was never audited

### R2 artefact set vs claimed phase - **red**
- missing for claimed status 'implementing': context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md (searched docs/tasks/obs/TASK-OBS-001-otel-collector)

### R3 run manifest - **absent**
- no ship-manifest (out-of-band work has none - a finding, not a failure)

### R4 committed-object presence - **red**
- absent at HEAD and on disk: deploy/obs/otel-collector-config.yaml
- absent at HEAD and on disk: deploy/obs/loki-config.yaml
- absent at HEAD and on disk: deploy/obs/prometheus-config.yaml
- absent at HEAD and on disk: deploy/obs/tempo-config.yaml
- absent at HEAD and on disk: deploy/obs/grafana/datasources.yaml
- absent at HEAD and on disk: deploy/obs/grafana/provisioning/dashboards/.keep
- absent at HEAD and on disk: deploy/obs/auth/tokens.example
- absent at HEAD and on disk: deploy/obs/scripts/rotate_tokens.sh
- absent at HEAD and on disk: deploy/obs/scripts/healthcheck.sh
- absent at HEAD and on disk: deploy/obs/tests/smoke_test.sh
- absent at HEAD and on disk: deploy/obs/tests/auth_required_test.sh
- absent at HEAD and on disk: deploy/obs/tests/buffer_survives_restart_test.sh

### R5 cited tests now - **skipped**
- --run-tests not given

## Appendix B — gathered read-only evidence (folder, spec head, git history, claimed paths, cited suites)

```
----- folder: docs/tasks/obs/TASK-OBS-001-otel-collector
-rw-r--r--@ 1 stephencheng  staff  28756 Jul 23 10:52 spec.md
----- spec head (status/created/verify lines)
3:title: "OTel Collector + LGTM stack (Loki + Prometheus + Tempo + Grafana) with mTLS ingress + per-service tokens + retention + file-buffer"
10:created_at: 2026-05-15T00:00:00+07:00
16:status: implementing
17:verify: T
26:depends_on: []
71:effort_hours: 10
----- git log — task folder
069d4dff 2026-07-20 docs: unwrap hard-wrapped markdown to one line per paragraph
4c02b556 2026-07-18 IMP-117 §1.6: migrate 497 non-conformant specs — move trailing frontmatter comments to own-line (FM-001)
f3e17e9f 2026-07-15 fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator
34b46d7c 2026-07-15 feat: updates
11628138 2026-07-14 refactor(rename): feature-request -> task, task -> subtask
----- claimed paths (new_files + modified_files): last commit + on-disk
  deploy/obs/docker-compose.yml | last-commit: 11628138 2026-07-14 | on-disk
  deploy/obs/otel-collector-config.yaml | last-commit: NONE | ABSENT
  deploy/obs/loki-config.yaml | last-commit: NONE | ABSENT
  deploy/obs/prometheus-config.yaml | last-commit: NONE | ABSENT
  deploy/obs/tempo-config.yaml | last-commit: NONE | ABSENT
  deploy/obs/grafana/datasources.yaml | last-commit: NONE | ABSENT
  deploy/obs/grafana/provisioning/dashboards/.keep | last-commit: NONE | ABSENT
  deploy/obs/auth/tokens.example | last-commit: NONE | ABSENT
  deploy/obs/scripts/rotate_tokens.sh | last-commit: NONE | ABSENT
  deploy/obs/scripts/healthcheck.sh | last-commit: NONE | ABSENT
  deploy/obs/README.md | last-commit: 069d4dff 2026-07-20 | on-disk
  deploy/obs/tests/smoke_test.sh | last-commit: NONE | ABSENT
  deploy/obs/tests/auth_required_test.sh | last-commit: NONE | ABSENT
  deploy/obs/tests/buffer_survives_restart_test.sh | last-commit: NONE | ABSENT
----- cited test suites in spec (existence only, NOT executed)
```
