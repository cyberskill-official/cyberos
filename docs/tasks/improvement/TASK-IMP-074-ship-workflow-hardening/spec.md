---
id: TASK-IMP-074
title: "Ship-workflow hardening — status-page auto-sync, batch/parallel task shipping with unlock rescan, rules-to-channels distribution sync"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: "Wave 6 - go-live (workflow engine)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-IMP-068, TASK-IMP-072, TASK-CUO-206]
depends_on: []
blocks: []
source_pages:
  - ".githooks/pre-commit (41 lines; regenerates dist/website on docs changes but NEVER calls migrate-tasks.sh --page nor stages docs/status/ - the root cause of the stale status page)"
  - "tools/install/migrate-tasks.sh line 13 (its own comment claims the pre-commit hook uses --page; the wiring does not exist - aspiration, not fact)"
  - ".cyberos/cuo/gates/run-gates.sh lines 82-88 (the only current docs/status regeneration point, best-effort, output left unstaged)"
  - "modules/cuo/chief-technology-officer/workflows/ship-tasks.md (workflow_version 2.4.0; one-task-at-a-time queue; no batch selection, no unlock rescan)"
  - "tools/install/build.sh manifest.yaml block (no rules fingerprint - channels cannot detect rule drift when VERSION is unchanged)"
  - ".github/workflows/deploy.yml paths (docs job triggers exclude modules/cuo/** and modules/skill/** rule sources)"
source_decisions:
  - "2026-07-13 Stephen: strengthen /ship-tasks with (1) status html page auto-update on changes, (2) batch shipping of parallel-safe tasks with auto batch detection + auto unlock detection, (3) workflow rules always synced to distributed channels (standalone .cyberos, AI plugins/connectors/MCPs) with auto hooks on build/release/deploy."
language: bash (hooks, build), YAML (CI), markdown (workflow spec)
service: modules/cuo + tools/install + .githooks
new_files: []
modified_files:
  - .githooks/pre-commit
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - tools/install/build.sh
  - tools/install/check-version-sync.sh
  - .github/workflows/deploy.yml
allowed_tools:
  - bash/sed/sha256sum (hook + deterministic rules fingerprint)
disallowed_tools:
  - Any weakening of the two HITL gates - batching changes HOW MANY tasks move per phase, never WHICH transitions need a human verdict
effort_hours: 6
subtasks:
  - "Wire migrate-tasks.sh --page into pre-commit on task/CHANGELOG/VERSION changes + auto-stage docs/status/ (1h)"
  - "ship-tasks.md v2.5.0: batch selection, batched phases/commits, unlock rescan, status-sync rule, distribution-sync section (2h)"
  - "build.sh: deterministic rules_sha in manifest.yaml; check-version-sync.sh asserts it (2h)"
  - "deploy.yml: rule-source paths trigger the docs job (1h)"
risk_if_skipped: "Status page silently lies about task state after every create/ship run (operators act on stale data); serial one-task-at-a-time shipping wastes sessions when cones are independent (this session shipped 5 tasks batch-style by hand - the workflow doc did not sanction it); distributed channels (self-hosted .cyberos, plugin, MCP) keep running old workflow rules with no way to detect drift between version bumps."
---

## §1 — Description

**Group A — status page auto-sync**

1. Any commit whose staged changes touch `docs/tasks/**`, `CHANGELOG.md`, or `VERSION` **MUST** regenerate `docs/status/` (via the existing `migrate-tasks.sh --page` fast path) and stage the result in the same commit, wired into `.githooks/pre-commit`. The status page can never again lag the task frontmatter it renders.
2. The regeneration **MUST** be best-effort-loud: a render failure prints a warning naming the fix command but does not block the commit (matching run-gates.sh's posture; a docs-render bug must not dead-lock all task work).
3. `ship-tasks.md` **MUST** record this as a rule of the backlog-state-update steps: every status mutation rides with a fresh status page.

**Group B — batch / parallel shipping**

4. The workflow (bumped to v2.5.0) **MUST** define batch selection: the eligible set is every `ready_to_implement` task whose `depends_on` are all `done`; a batch is a maximal subset whose members are pairwise independent — no `depends_on`/`blocks` edge between members AND no overlap between their declared cones (`new_files` + `modified_files` + `service`). Overlapping tasks stay serial in priority order.
5. Batched execution **MUST** keep per-task artefacts, per-task manifests, and per-task HITL verdicts (a single human reply MAY record verdicts for many tasks at once, e.g. "approve all" - one utterance, N recorded verdicts), while phases MAY be executed and committed batch-wide instead of one-by-one.
6. **Unlock rescan:** whenever any task reaches `done`, the workflow **MUST** re-scan the backlog for tasks whose `depends_on` just became fully satisfied and append the newly-eligible, cone-independent ones to the running batch queue (no operator prompt - EXECUTION-DISCIPLINE §1 continuation).
7. Batching **MUST NOT** weaken HITL: the two human-acceptance gates still apply to every task individually; batch = fewer round-trips, identical guarantees.

**Group C — rules-to-channels distribution sync**

8. The payload's `manifest.yaml` **MUST** carry a deterministic `rules_sha` — a content fingerprint over the rule trees the payload distributes (`cuo/`, `plugin/`, `mcp/`, `cli/`, `memory/`) — so every channel (standalone/self-hosted `.cyberos`, Claude plugin, MCP server, npx CLI) can detect rule drift even when VERSION is unchanged. `check-version-sync.sh` **MUST** fail if `rules_sha` is missing/empty.
9. The auto-hook chain **MUST** cover build, release, and deploy: (build) the existing pre-commit payload rebuild on `modules/**`/`tools/install/**` changes + payload-gate on push; (release) the existing release.yml payload job publishing stamped payload assets per tag; (deploy) deploy.yml's docs job **MUST** additionally trigger on `modules/cuo/**` and `modules/skill/**` so rule changes refresh the published site without waiting for a release. The chain is documented in the workflow spec so it is discoverable, not tribal.
10. `cyberos update` / `install.sh --check` remain the pull side: they already compare payload versions; `rules_sha` gives them (and any plugin/MCP consumer) a finer-grained drift signal to compare against. Extending their comparison logic is explicitly a follow-up once this fingerprint exists in the wild.

*Length note: this task consciously invokes the sanctioned pure-infra profile — every clause is hook/CI/doc wiring over already-existing machinery, with the §5 checks runnable in-session; the fuller 300-line bar buys nothing here that §5 does not already prove.*

## §2 — Why this design

The status-page fix reuses the exact regeneration path run-gates.sh already trusts (`migrate-tasks.sh --page`) instead of inventing a second renderer — the bug was missing wiring, not missing tooling. Batch semantics are codified from this session's real 5-task run (queue → per-task artefacts → one "approve all"/"accept all" verdict pair), which worked but was improvised; v2.5.0 makes it the sanctioned default and adds the unlock rescan the manual run lacked. The rules fingerprint goes in `manifest.yaml` because that file already rides every channel and is already gate-checked — one new field, zero new distribution surface.

## §3 — API contract

- pre-commit block: trigger regex `^(docs/tasks/|CHANGELOG\.md$|VERSION$)`; call `"$root/.cyberos/migrate-tasks.sh" --page "$root"` (fallback `tools/install/migrate-tasks.sh` for this self-hosting repo); then `git add docs/status/`.
- manifest.yaml gains: `rules_sha: <64-hex>` computed as `sha256(sorted per-file sha256 list over cuo/ plugin/ mcp/ cli/ memory/ in $out)` — deterministic across runs/platforms.
- ship-tasks.md: `workflow_version: 2.5.0`; new §11a (batch selection + unlock rescan), §1 note on status sync, new "Distribution sync" subsection under Cross-references.
- deploy.yml docs paths += `modules/cuo/**`, `modules/skill/**`.

## §4 — Acceptance criteria

1. Committing a change under `docs/tasks/` regenerates and stages `docs/status/` in that same commit (hook run observed; `git show --stat` includes docs/status/index.html alongside the task change).
2. `bash -n .githooks/pre-commit` passes; a render failure path prints a warning and exits 0 (non-blocking).
3. Two consecutive payload builds produce identical `rules_sha`; editing any file under `modules/cuo/` changes it.
4. `check-version-sync.sh` fails against a payload whose manifest lacks `rules_sha`, passes against a fresh build.
5. `ship-tasks.md` parses as v2.5.0 with §11a present; batch rules restate both HITL gates unchanged.
6. deploy.yml YAML-parses with the two added path globs.

## §5 — Verification

```bash
bash -n .githooks/pre-commit
bash tools/install/build.sh /tmp/p1 >/dev/null && bash tools/install/build.sh /tmp/p2 >/dev/null
grep "^rules_sha:" /tmp/p1/manifest.yaml /tmp/p2/manifest.yaml   # identical, non-empty
bash tools/install/check-version-sync.sh /tmp/p1            # PASS incl. rules_sha assert
sed -i 's/^rules_sha:.*/rules_sha: ""/' /tmp/p1/manifest.yaml && ! bash tools/install/check-version-sync.sh /tmp/p1   # negative
grep -n "workflow_version: 2.5.0" modules/cuo/chief-technology-officer/workflows/ship-tasks.md
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy.yml'))"
```

## §6 — Implementation skeleton

Covered by §3 — five files, all edits; no new scripts beyond the hook block.

## §7 — Dependencies

Upstream none; downstream: a follow-up task may teach `cyberos update`/plugin/MCP clients to compare `rules_sha` (clause 10). Batched with TASK-IMP-075 (non-overlapping cones: workflow/hooks/CI-docs vs apps/desktop Rust) — the first sanctioned use of §11a.

## §8 — Example payloads

`rules_sha: 3f9c…64hex` in manifest.yaml; hook output: `cyberos: docs/status regenerated + staged (task sources changed)`.

## §9 — Open questions

- Client-side `rules_sha` comparison in `cyberos update`/plugin/MCP (clause 10) — follow-up task once the field ships.
- Whether batch commits should also batch across MODULES with distinct gate profiles — deferred; v2.5.0 scopes batching to cone-independence only.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Render script errors during commit | warning printed, commit proceeds (clause 2) | page stale for one commit | run-gates.sh next run heals; fix renderer |
| Two batched tasks secretly touch the same file despite declared cones | git conflict/second edit visible in review packet diff | reviewer rejects at gate 1 | route one back; cones corrected in frontmatter |
| Unlock rescan picks a task whose spec drifted since audit | ship-manifest task_sha256 staleness rule (TASK-CUO-206) triggers restart at step 1 | no stale-spec shipping | existing manifest machinery |
| rules_sha nondeterminism across OS (sort/locale) | AC #3 double-build check; LC_ALL=C forced in build.sh | none if caught | pinned locale in the hash pipeline |
| Channels ignore rules_sha (no client logic yet) | clause 10 documents it as pull-side follow-up | drift detectable, not yet auto-acted-on | follow-up task |
| deploy docs job now triggers more often | Actions usage visible | slightly more CI minutes | acceptable; docs job is minutes-cheap |

## §11 — Implementation notes

Batched with TASK-IMP-075 per §11a. Hash pipeline uses `LC_ALL=C sort` + `sha256sum` (ubuntu CI) with `shasum -a 256` fallback for macOS operators. HITL gates restated verbatim in §11a so batching can never be cited to skip a verdict.

*End of TASK-IMP-074.*
