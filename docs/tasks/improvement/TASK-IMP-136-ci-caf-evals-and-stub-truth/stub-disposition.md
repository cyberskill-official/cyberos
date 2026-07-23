# TASK-IMP-136 — stub workflow disposition table (spec §1.4)

Nine always-green stub workflows (auto-generated 2026-05-17 from task `build_envelope`
references; single `echo` placeholder step each) were dispositioned on 2026-07-23.

Decision rule (spec §1.4): **DELETE by default**; **IMPLEMENT** only when the declaring
task's spec embeds complete canonical workflow YAML **and** its runtime dependencies
(secrets, services, data) exist today. Both conditions were checked first-hand against
the working tree; the per-file evidence is in the rightmost column. Deletion reason
(recorded per the never-delete-without-reason rule, spec `source_decisions`): an
always-green check is worse than no check — it manufactures false confidence under a
gate-shaped name. Every deletion names its declaring task so the gap stays discoverable;
re-authoring a real gate is the operator's call per gap (spec Non-Goals).

| # | Workflow file (deleted from `.github/workflows/`) | Declaring task | Disposition | Evidence for the judgment (measured 2026-07-23) |
|---|---|---|---|---|
| 1 | `cache-isolation-gate.yml` | TASK-AI-018 (done) | DELETE | Spec embeds a full workflow, but it is not runnable as embedded: `cargo test ... --format=json` requires nightly libtest (repo pins stable), and the gate step pipes through `tee` without pipefail (GitHub's default `bash -e` shell), so a failing `cargo test` would still pass the step. The real isolation suite exists in-repo (`services/ai-gateway/tests/cache_isolation*.rs`, 5 files) and runs via the module's awh/caf target-health path. |
| 2 | `memory-rebuild.yml` | TASK-MEMORY-102 (done) | DELETE | Embedded workflow requires `crates/cyberos-obs-sdk/**` (path does not exist in the tree) and a `cyberos/bge-m3-sidecar:latest` container image (no public registry copy; not buildable from this repo), plus `generate_test_fixture`/`rebuild_layer2` cargo bins not present in `services/memory`. Runtime dependencies do not exist today. |
| 3 | `obs-correlation-gate.yml` | TASK-OBS-005 (implementing) | DELETE | Embedded workflow composes `deploy/obs/docker-compose.yml` + `langsmith-docker-compose.yml` and queries Loki/Tempo/LangSmith/Prometheus; `crates/cyberos-obs-sdk` (a trigger path and the SDK under test) does not exist. The declaring task is still `implementing` — its real gate belongs to its own landing. |
| 4 | `proj-a11y-gate.yml` | TASK-PROJ-018 (done) | DELETE | The workflow's target `web/proj-client` does not exist in the tree; no Playwright/storybook surface to run axe-core against. |
| 5 | `proj-storybook-chromatic.yml` | TASK-PROJ-018 (done) | DELETE | Same missing `web/proj-client`, plus Chromatic requires a project + `CHROMATIC_PROJECT_TOKEN` secret that does not exist (named as missing infrastructure in TASK-IMP-136 Alternatives). |
| 6 | `rew-memory-exclusion.yml` | TASK-REW-010 (draft) | DELETE | Declaring spec embeds no workflow YAML at all (0 fenced yaml blocks), and its target `services/rew` does not exist. The declarer is still `draft`; the gate ships with the task, if and when it ships. |
| 7 | `vn-pii-quarterly-refresh.yml` | TASK-AI-013 (done) | DELETE | Declaring spec embeds no YAML for this workflow (only for the recall gate and fixtures). The quarterly data-refresh pipeline it would schedule does not exist (named as missing infrastructure in TASK-IMP-136 Alternatives). |
| 8 | `vn-pii-recall.yml` | TASK-AI-013 (done) | DELETE | Spec embeds a full workflow, but its pytest paths (`services/ai-gateway/pii/tests/test_recall_gate.py` etc.) do not exist — the shipped suite lives at `services/ai-gateway/pii/test_vn_recall_floor.py` and siblings under different names — and it installs `vi_core_news_lg`, which is not a published spaCy model. Per the spec's edge case, TASK-AI-013's REAL recall gate is that local test suite; only the stub workflow is swept. |
| 9 | `zdr-staleness-check.yml` | TASK-AI-015 (done) | DELETE | Embedded workflow runs `cargo run --bin zdr-staleness-check`; `services/ai-gateway` declares no such binary (bins: cyberos-ai, gen-schema, cost-hold-expiry, cyberos-gateway). The config it would read (`config/zdr_attestations.yaml`) exists, but the executable half does not. |

Disposition counts: **9 deleted, 0 implemented, 0 labeled.**

Regrowth guard: `scripts/tests/test_benchmark_ci_truth.sh` assert (c) fails the moment any
file under `.github/workflows/` carries the stub placeholder marker again — the honest
path for a future declarer is shipping the real YAML, or nothing.

## Operator steps (branch protection — spec §3, before merging this sweep)

1. Run: `gh api repos/cyberskill-official/cyberos/branches/main/protection --jq '.required_status_checks.contexts'`
   Expected output: no entry naming `cache-isolation-gate`, `memory-rebuild`,
   `obs-correlation-gate`, `proj-a11y-gate`, `proj-storybook-chromatic`,
   `rew-memory-exclusion`, `vn-pii-quarterly-refresh`, `vn-pii-recall`, or
   `zdr-staleness-check` (they were never wired as required — verify, don't assume).
2. If any of the nine IS required: remove it from protection first, then merge; note the
   removal in the PR.

## Suggested commit-body block (for the final sequential pass)

```
Stub workflow sweep (TASK-IMP-136 §1.4) — 9 deleted, 0 implemented:
  cache-isolation-gate.yml     (declared by TASK-AI-018)
  memory-rebuild.yml           (declared by TASK-MEMORY-102)
  obs-correlation-gate.yml     (declared by TASK-OBS-005)
  proj-a11y-gate.yml           (declared by TASK-PROJ-018)
  proj-storybook-chromatic.yml (declared by TASK-PROJ-018)
  rew-memory-exclusion.yml     (declared by TASK-REW-010)
  vn-pii-quarterly-refresh.yml (declared by TASK-AI-013)
  vn-pii-recall.yml            (declared by TASK-AI-013)
  zdr-staleness-check.yml      (declared by TASK-AI-015)
Reason: always-green placeholder checks (single echo step) manufacture false
confidence under gate-shaped names; per-file evidence in
docs/tasks/improvement/TASK-IMP-136-ci-caf-evals-and-stub-truth/stub-disposition.md.
.pre-commit-config.yaml removed (TASK-IMP-136 §1.3): dead mechanism — the repo's
hook path is core.hooksPath=.githooks, and every live claim (payload build, docs
build, awh gate) is now covered by .githooks/pre-commit directly.
```
