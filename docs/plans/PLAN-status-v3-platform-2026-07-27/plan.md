# PLAN - Status v3 as the CyberOS platform standard

- Date: 2026-07-27
- Author: Claude (session with operator Stephen Cheng), for handoff to executing models
- Status: proposed - awaiting operator approval; NOTHING in this plan is implemented by the plan itself
- Baseline: cyberos @ `a7e0e212` (v1.10.0), branch `batch/ten-inv-host-e`
- Approved input: `docs/status-v3-preview/` (operator-reviewed preview, this session)
- Consumes into backlog via: `/cyberos:create-tasks` (task set in §7)

## 0. Intent and non-negotiables

Promote the approved v3 Status experience from a standalone preview to the CyberOS
platform standard: one generator, one data contract, one tabless page, one
traceability rule - shipped identically to the mothership, every consumer install,
and every distribution channel (online and offline), and rolled out to the existing
fleet under `/Users/stephencheng/Projects`.

Non-negotiables the executing models must preserve at every step:

1. THE RULE: every change introduced by a commit is linked to one or more tasks and
   reflected on the Status page and in the Release Notes. No exceptions. Exempt
   classes (release plumbing) are explicit, versioned, and visible on the page.
2. Git history is never rewritten. Retroactive link fixes go through the reviewed
   ledger (`commit-links.yaml`), never `rebase`/`filter-branch`.
3. Determinism: same inputs produce byte-identical page output; the `fp-` stamp
   covers every input that determined the page (existing render-status-hub law).
4. Offline-first: the page works over `file://`, zero network calls, zero runtime
   dependencies, and keeps a `<noscript>` fallback table.
5. Consumer repos are sovereign: their tasks, history, `.cyberos/config.yaml`, and
   uncommitted work are preserved through any upgrade; failures degrade loudly but
   never block unrelated work (matches existing hook posture).
6. Executing models follow the rule themselves: every commit in this program cites
   its task id from §7.

## 1. Evidence - current state (verified 2026-07-26/27)

Mothership pipeline:

- Generator: `tools/docs-site/render-status-hub.mjs` (status-hub@2, "one corpus,
  three lenses"). Client source `modules/templates/html/status-app.js`, shell
  `modules/templates/html/status-hub.html`. Emits `reference/status.html`,
  `reference/data/task/<ID>.js` spec chunks, assets, and a `roadmap.html` redirect.
- Committed page: `.githooks/pre-commit` (TASK-IMP-074 group A) regenerates
  `docs/status/` via `.cyberos/lib/status-page.sh` -> `lib/task-migrate.sh`
  ("migrate 3/4") whenever staged paths match `^(docs/tasks/|CHANGELOG\.md$|VERSION$)`,
  and stages the result in the same commit. `/cyberos:status` opens
  `docs/status/index.html`.
- Online: `tools/docs-site/build.sh` -> gitignored `dist/website` -> `ship.sh` to
  VPS `/srv/console/docs` (deploy.yml paths filter + release.yml `docs` job);
  `stage-vercel.mjs` + `vercel.json` is the second online channel.
- Payload: `tools/install/build.sh` -> `dist/cyberos`; `.pre-commit-hooks/
  cyberos-payload-build.sh` rebuilds it when payload sources are staged;
  `check-version-sync.sh` enforces stamp = VERSION. `tools/install/install.sh`
  vendors payload into consumer `.cyberos/` (docs-tools, lib incl. status-page.sh,
  plugin, status.sh, AGENT-ENTRY.md scaffolding).
- Mirrors: `tools/docs-site/render-status-hub.mjs` and
  `tools/install/docs-tools/render-status-hub.mjs` are a checked pair
  (`check-pair-parity.sh`); `.cyberos/docs-tools/` and `dist/cyberos/docs-tools/`
  are derived copies.
- Versioning: `scripts/cyberos-version.mjs` (conventional commits; `!` or
  `BREAKING CHANGE:` -> major; `Release-As: X.Y.Z` trailer as escape hatch);
  version.yml commits `chore(release): vX.Y.Z`, rebuilds `apps/web`, proves payload;
  tagging is manual (`git tag vX.Y.Z`) and fires release.yml (jobs: payload,
  channels, npm, desktop, updater-manifest, android, ios, docs) plus
  release-snap / release-mas / release-msstore / release-pkgmgr-pr / notarize.
- Fleet tooling already exists: `tools/install/rollout-fleet.sh` (re-install roots
  from a payload, migrate repos with tasks or a status page), `audit-fleet.sh`
  (deep post-install audit incl. rules_sha), `fleet-install-test.sh`,
  `commit-fleet.sh`, `check-latest.sh`.
- Tests: `scripts/tests/run_all.sh` is the pre-commit-gated shell suite
  (test_render_stamp.sh already covers status-page stamping);
  `tools/docs-site/tests/` exists; suite-gate.yml runs the suite in CI.

Approved v3 assets produced this session (UNCOMMITTED, working tree of
`batch/ten-inv-host-e` - Phase 0 lands them):

- `docs/status-v3-preview/` - build.mjs (snapshot extractor), app.js, app.css,
  index.html (generated), commit-links.yaml (backfill ledger seed), README.md.
- `scripts/check_task_link.sh` - shared traceability checker (`--msg`, `--range`,
  cutoff constant `a7e0e212...`, env override `CYBEROS_TRACE_CUTOFF`).
- `.githooks/commit-msg` - now also runs the checker (advisory; strict via
  `CYBEROS_REQUIRE_TASK_LINK=1` or `CYBEROS_STRICT_COMMITS=1`).
- `.github/workflows/traceability.yml` - hard CI gate on PRs and pushes to main.

Verified data facts the executing models rely on:

- 586 live tasks, 588 dependency edges (`depends_on`/`blocks` in frontmatter are
  DROPPED by status-hub@2 - restoring them is a core deliverable), 82 cross-module
  edges, 29 modules.
- Version epoch reset: `537d3739` "chore(release): roll back to 0.1.0" (2026-07-12);
  1.x version strings exist in two epochs; commit bucketing MUST be by marker
  occurrence with everything older than the rollback folded into a first-epoch
  entry. Tags are sparse (14) - release links fall back to marker commits.
- Link recovery ladder proven on real data: canonical id (149) + shorthand resolved
  against the corpus (306) + reviewed ledger => coverage since the reset 68%, with
  148 true violations remaining (+296 first-epoch).
- Fleet snapshot (see §6): 22 consumer installs, versions 1.0.0/1.1.0/1.5.1,
  21 with a published status page, one flat-layout task corpus (sachviet), most
  worktrees dirty.

## 2. Target architecture

### 2.1 Data contract - `status-feed@1`

One versioned JSON payload embedded in the page (and emitted beside it as
`data/status-feed.json` for tooling), produced only by the generator. Extends the
current `cs-data` corpus; field names follow the preview's proven schema
(`docs/status-v3-preview/build.mjs` is the reference implementation).

| Key | Content | New vs status-hub@2 |
| --- | --- | --- |
| `project, version, snapshot, head, branch, repoUrl, tags, fp` | identity + provenance; `repoUrl` derived from `origin` (ssh and https forms), empty when remote absent | branch/repoUrl/tags new |
| `tasks[]` | id, folder key, title, module, type, priority, status, bucket, phase, phase group, owner, created, shipped, effort, `d[] bl[] rl[]` dependency edges, summary | dependency edges, shipped, phase group new |
| `modules[]`, `medges[]` | per-module counts + deterministic layout coords; cross-module dependency edges with weights and sample pairs | new |
| `phases[]` | P0..P5 / PRE / TRACK / POST lanes with date spans and counts | new |
| `releases[]` | notes mapped Added->features, Fixed->fixes, else->improvements; per-release `cov` (classified commits); `rc` marker hash; `lg` flag for the folded first epoch | cov, rc, lg, three-type mapping new |
| `unreleased`, `burnup` | staged notes + pending commits; cumulative shipped-by-day | new |

Commit classification ladder (exact order): canonical `TASK-[A-Z]+-\d+` in
subject+body -> shorthand `PREFIX-NNN` (incl. `TEN-002/004` lists) resolved only
when the task exists -> reviewed ledger `docs/tasks/_state/commit-links.yaml` ->
exempt (`chore(release)`, `chore(web): rebuild`, merges, reverts, fixups,
`[skip ci]`) -> unlinked (violation). `via` recorded for non-canonical links.

Build-time validation (fail the build, matching status-hub's honest-failure law):
unknown status values, dangling `depends_on` targets not in corpus and not in
`_archive` (report as ghosts, fail above a configurable threshold), ledger entries
citing unknown tasks, malformed changelog release headers. The `fp-` stamp must
newly cover: git commit set hash, ledger file, manifest.yaml, and the client/shell
template bytes.

### 2.2 Generator - render-status-hub v3 (status-hub@3)

Fold `docs/status-v3-preview/build.mjs` extraction INTO
`tools/docs-site/render-status-hub.mjs` (do not keep two extractors). Keep from
v2: frontmatter parser, spec chunk emission (`data/task/<ID>.js`), batch economics
ingestion, deterministic stamp, `CYBEROS_*` env knobs, LENIENT mode, noscript
table. Add: the §2.1 feed, git ingestion (`git log` with `\x01/\x02` separators;
tolerate absence of git -> empty cov, page still renders), epoch bucketing by
marker occurrence, GitHub URL derivation, tag list. Git queries are the only new
input class; in LENIENT/offline mode absence degrades to empty coverage with a
visible "no git history available" note, never a crash.

### 2.3 Client - one tabless canvas (platform UI)

The generator reads its client from templates (`const CSS = tpl('cds','tokens.css')
+ tpl('cds','status.css')`, `const APP = tpl('html','status-app.js')`, resolved
from `modules/templates/` with a `tools/docs-site/templates/` fallback). Replace
`modules/templates/html/status-app.js` with the preview's `app.js` and
`modules/templates/cds/status.css` with the preview's `app.css` recut against
`tokens.css` (keep brand tokens single-sourced in `tokens.css`; the night theme
joins as a `[data-theme]` layer, consistent with the existing token comment in
`docs/status/assets/status.css`). Adapt the client to read `status-feed@1` from
the embedded corpus and keep chunked spec loading. Locked-in behaviors,
all already proven in the preview and its 47-assertion DOM suite:

- Bands 01-06 (Pulse, Roadmap, System map, Flow, Releases and traceability, Index)
  with one shared selection state `{kind, id}`.
- Sync without jumps: selecting a module/phase/release updates bands 02-06 in
  place; nothing scrolls on selection; second click or the top-bar selection chip
  navigates; deep links (`#t/ #m/ #p/ #r/`) scroll on arrival.
- Legacy hash compatibility: `#board`, `#table`, `#timeline`, `#roadmap`,
  `#backlog`, `#changelog`, and `#task/<ID>` map to the new routes (bookmarks from
  status-hub@2 stay alive - same rule v2 applied to v1 tabs).
- Compact 05/06: featured cards only for releases with notes (fold after 6),
  plumbing releases as one-line rows, first-epoch folded; index defaults to the
  Open filter, 30-row group caps, five filter chips.
- Complete GitHub linking: commit hashes -> `/commit`, releases -> tag when it
  exists else marker commit, `#N` -> `/pull`, snapshot branch -> `/tree/<branch>`,
  head -> `/commit`. Non-GitHub or absent remotes render the same text unlinked.
- Traceability band states THE RULE verbatim, shows per-release coverage bars,
  the trend chart, violation flags, and the enforcement mode actually active.
- Brand paper theme default + night theme; `?theme=` override; localStorage
  persistence wrapped in try/catch (file:// safety); reduced-motion respected;
  staleness chip when snapshot age > 1 day.
- Drawer: full task record, dependency chips, citing releases/commits, spec link,
  "Trace in graph".

### 2.4 Enforcement - one script, three surfaces

- `scripts/check_task_link.sh` stays the single implementation (already written).
  Change from session state: cutoff moves OUT of the constant into config -
  resolution order `CYBEROS_TRACE_CUTOFF` env -> `.cyberos/config.yaml:
  traceability.cutoff` -> repo-local default written at install/upgrade time.
- Surface 1, local: `.githooks/commit-msg` (mothership) and the consumer hook the
  installer manages - advisory by default, strict via env or
  `config.yaml: traceability.strict: true`.
- Surface 2, CI: `.github/workflows/traceability.yml` (mothership: land as a
  required check). Installer scaffolds the consumer equivalent only when the repo
  has `.github/workflows/` or the operator opts in (consumer repos may be offline
  or non-GitHub; the hook is the universal layer).
- Surface 3, release: release tagging blocked while the release range contains
  unlinked commits - implemented as a preflight in the release runbook +
  `release.yml` payload job step calling `check_task_link.sh --range
  <last-tag>..HEAD` (soft-fail grace window during Phase 6, hard after v2.0.0).

## 3. Phase plan

Every phase ends at a gate; the executing model stops at each HITL gate and waits
for operator approval. Sequence: 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8; 3/4/5
may interleave after 2 passes its gate.

### Phase 0 - land the session work as reviewable history

Goal: nothing valuable exists only in a working tree.

1. Run `/cyberos:create-tasks` with §7 to mint the task set; land BACKLOG rows.
2. Commit the session artifacts in coherent, RULE-COMPLIANT commits on a fresh
   branch `feat/status-v3-platform` cut from `main` (do not entangle with
   `batch/ten-inv-host-e` work): preview folder; enforcement trio
   (`scripts/check_task_link.sh`, `.githooks/commit-msg`, workflow); each commit
   cites its §7 task id.
3. Pre-commit will run `scripts/tests/run_all.sh` (scripts/ touched) - must pass.
4. PR to main. traceability.yml goes live with cutoff = merge commit of this PR
   (update the script default; record the same sha in `.cyberos/config.yaml`).
   GitHub branch protection: add `traceability-gate` and `suite-gate` to required
   checks (operator action, needs repo admin).

Gate P0 (HITL): PR merged; CI green including the new gate on its own commits.
Rollback: revert the PR; the gate is additive and nothing else depends on it yet.

### Phase 1 - status-feed@1 in the generator

Goal: render-status-hub v3 emits the feed; old page still ships.

1. Implement §2.1/§2.2 in `tools/docs-site/render-status-hub.mjs`. Mirror to
   `tools/install/docs-tools/render-status-hub.mjs` (pair parity gate enforces).
2. Move the ledger: `git mv docs/status-v3-preview/commit-links.yaml
   docs/tasks/_state/commit-links.yaml`; generator + `task-lint.mjs` validate its
   entries (unknown hash prefix = warn, unknown task = fail).
3. New unit tests under `tools/docs-site/tests/` with committed fixtures:
   epoch-duplicate markers (modeled on `537d3739`), shorthand and slash-list
   resolution, ledger precedence, exempt classes, dangling deps, no-git degrade,
   ssh and https remote parsing, sparse tags. Extend
   `scripts/tests/test_render_stamp.sh` for the widened fp coverage.
4. Emit BOTH during this phase: current status-hub@2 HTML (unchanged) AND
   `data/status-feed.json` (new). No visual change yet.

Gate P1: suite green; `node tools/docs-site/render-status-hub.mjs` output byte-
identical on double-run; feed validates against a JSON schema checked into
`tools/docs-site/tests/status-feed.schema.json`.
Rollback: feed emission is additive - revert the commit.

### Phase 2 - page swap with legacy fallback

Goal: the v3 canvas becomes the emitted page everywhere the generator runs.

1. Replace `modules/templates/html/status-app.js` + the hub CSS source with the
   preview client adapted per §2.3; `status-hub.html` becomes the v3 shell; keep
   spec-chunk loading and `<noscript>` (regenerate the noscript table from the
   feed).
2. Legacy escape hatch for exactly one minor cycle: generator also emits
   `status-legacy.html` (the v2 page) next to the v3 page; v3 footer links it;
   `CYBEROS_STATUS_LEGACY=1` makes legacy the primary emission (rollback lever).
3. Promote the preview's DOM suite: port `smoke.mjs` (47 assertions) to
   `tools/docs-site/tests/test_status_dom.mjs` running against the EMITTED page
   with jsdom pinned as a devDependency of the docs-site tests only (never a
   runtime dep). Wire into `run_all.sh`.
4. Delete `docs/status-v3-preview/` in the same PR that lands the swap (its
   README's integration section moves to docs, §Phase 8). Redirect stub optional.
5. Regenerate `docs/status/` via `.cyberos/lib/status-page.sh` so the committed
   mothership page IS the v3 page; verify `/cyberos:status` opens it.

Gate P2 (HITL): operator reviews the emitted page (paper + night, file:// and
served); legacy page reachable; DOM suite green.
Rollback: `CYBEROS_STATUS_LEGACY=1` env in the hook/workflow callers, or revert.

### Phase 3 - regeneration on every relevant change

Goal: the page can never lag its inputs, without manual steps.

1. Local: widen `.githooks/pre-commit` `status_trigger` from
   `^(docs/tasks/|CHANGELOG\.md$|VERSION$)` to every commit for the coverage band
   with a cheap short-circuit: regenerate fully on the current trigger paths;
   otherwise run the generator in `--coverage-only` mode (new flag: refresh
   commit/coverage data + stamp, skip task/spec re-render) so hook cost stays
   sub-second on code-only commits. Keep the loud-warn-never-block posture.
2. Truth window disclosure: a commit cannot include its own hash in the page it
   carries. The page states its coverage boundary ("coverage as of parent
   <short-sha>"); CI closes the gap.
3. CI/publish: deploy.yml paths already rebuild the site on docs inputs; ADD
   `docs/tasks/**`, `.githooks/**`, `scripts/check_task_link.sh` to its trigger
   list; the deploy job re-runs the generator so the SERVED page includes the
   pushed commits themselves. release.yml `docs` job unchanged in shape.
4. Post-bump freshness: version.yml's bump commit matches the pre-commit trigger
   (VERSION), so the committed page carries the new version in the same commit -
   verify with a test in `scripts/tests/`.

Gate P3: three scripted scenarios pass in a scratch clone: task edit commit,
code-only commit, version bump - each leaves `docs/status/` consistent with HEAD
per the disclosure rule.
Rollback: restore the narrow trigger regex (one-line revert); page correctness
does not depend on the wide trigger.

### Phase 4 - enforcement as platform default

1. Cutoff/config: implement §2.4 resolution order in `check_task_link.sh`; add a
   commented `traceability:` block to the `config.yaml` scaffold in
   `tools/install/install.sh` (keys: `cutoff`, `strict`, `scaffold_ci`).
2. Installer behavior: on install/upgrade, if no cutoff recorded, write
   `cutoff: <HEAD at install>` - new installs enforce from day one, upgrades from
   upgrade day, no repo starts with retroactive violations.
3. Consumer hooks: installer already manages `core.hooksPath`/hook files for
   consumer repos - extend the vendored commit-msg hook to call the vendored
   checker from `.cyberos/lib/` (ship the script in the payload under `lib/`).
4. Consumer CI template: ship under `tools/install/ci/` following the existing
   `github-action` layout there; installer scaffolds it into the consumer repo
   when `.github/` exists and `traceability.scaffold_ci` is not false.
5. Release-range block: add the `--range "<last marker>..HEAD"` preflight to the
   release runbook and as a non-blocking warning step in release.yml now; flip to
   blocking in Phase 6 alongside v2.0.0.
6. Mothership backfill workflow: triage the 148 current-epoch violations - for
   each, either add a reviewed ledger entry or accept as pre-cutoff history (they
   are all pre-cutoff by construction; the ledger work is optional truth-recovery,
   list generated by the gate script and attached to the tracking task).

Gate P4: `fleet-install-test.sh` proves a fresh scratch install carries hook +
config + (when applicable) CI template; strict mode blocks an unlinked commit in
the scratch repo; advisory mode warns.
Rollback: `traceability.strict: false` + remove required-check status; hook stays
advisory (informational only).

### Phase 5 - packaging into every distribution

1. Payload: confirm `tools/install/build.sh` picks up new/changed files
   (docs-tools generator, `lib/check_task_link.sh`, hook templates, CI template,
   templates/html). Rebuild `dist/cyberos`; `check-version-sync.sh`,
   `check-pair-parity.sh`, `emit-payload-sbom.sh` green.
2. Channels to verify one by one (executing model must check each actually
   carries the new bytes, not assume):

| Channel | Artifact | Verify |
| --- | --- | --- |
| Git installer (`create.sh` / `bootstrap.sh` / `install.sh`) | `.cyberos/` vendored tree | scratch install -> `audit-fleet.sh <ver> <dir>` |
| npm (release.yml `npm` job) | published package | `npm pack` contents include docs-tools + lib |
| CLI (`tools/install/cli`) | `cyberos` CLI | `cli.mjs` paths unchanged or updated |
| MCP server (`tools/install/mcp`) | install MCP | starts; status-affecting tools list updated |
| Plugin (`tools/install/plugin`, `marketplace/`) | `/cyberos:status`, `/help` command text | commands reference the tabless page, not lenses |
| Docs site online (VPS + vercel) | served status page | post-deploy curl shows v3 markup + fp |
| Desktop/store channels (desktop, MAS, MS Store, Snap, flathub/homebrew/winget manifests, updater-manifest, android, ios) | version-stamped apps | stamp check only - these do not embed the status page; confirm and record that fact in the release notes |
| GHCR service images (deploy.yml) | services | unaffected; version stamp only |

3. Offline guarantee: from the payload alone, in a network-less scratch repo:
   install, create a task, commit, open `docs/status/index.html` over file:// -
   full v3 experience minus GitHub links (which degrade to plain text) and with
   empty-remote coverage still computed from local git.

Gate P5: matrix above executed and recorded in the PR description; payload-gate,
suite-gate, npm-supply-chain workflows green.
Rollback: payload is rebuilt from source on revert; no channel pins v3
independently.

### Phase 6 - v2.0.0

Major because: the emitted page format and asset layout change (consumers of
`reference/status.html`, `docs/status/data/`, and the v2 lens hashes), the
contribution contract changes (traceability gate), and the installer scaffold
changes. Mechanics:

1. Land the final PR with a `feat(docs)!:` subject AND `Release-As: 2.0.0`
   trailer (deterministic against the classifier); version.yml commits
   `chore(release): v2.0.0`, rebuilds `apps/web`, proves the payload.
2. CHANGELOG 2.0.0 section written in the three types with task chips -
   Features (v3 status experience, status-feed@1), Improvements (regeneration
   pipeline, packaging), Fixes (none expected), plus a BREAKING notes block:
   old lens URLs redirect, v2 page available one cycle at `status-legacy.html`,
   traceability gate active with per-repo cutoffs.
3. Operator tags `v2.0.0` -> release.yml full artifact run (payload, channels,
   npm, desktop, updater-manifest, android, ios, docs) + store workflows; flip the
   release-range traceability preflight to blocking in the same PR.
4. `BUILD_NUMBER` and `stamp-release-version.mjs --check` pass (pre-commit
   enforces when VERSION staged).

Gate P6 (HITL): operator cuts the tag. Rollback: do not tag; or tag v2.0.1 with
`CYBEROS_STATUS_LEGACY` default if the page must revert while keeping the gate.

### Phase 7 - fleet discovery and migration (`/Users/stephencheng/Projects`)

Use the existing fleet tooling; do not write a new orchestrator.

1. Discovery (read-only):
   `find /Users/stephencheng/Projects -maxdepth 4 -type d -name .cyberos` plus
   `docs/tasks` scan for installs the first query misses. Classify each repo:
   installed version (`.cyberos/VERSION`), status page present, task corpus size,
   flat-layout remnants, dirty worktree, clone/worktree duplicates (skip
   `*-wt-*` worktrees and known clones - upgrade the primary checkout only; the
   2026-07-27 snapshot is §6 and must be re-run at execution time).
2. Per-repo protocol, in order, one repo at a time (small -> large: start with a
   zero-task repo, end with strategem/landing-page/shopass):
   a. Preflight: `git status` - if dirty, create safety commit on a branch
      `pre-cyberos-2.0-backup` or stash with a recorded name; tag
      `pre-cyberos-2.0` at HEAD. Record repo state row in the migration report.
   b. Upgrade: `bash tools/install/install.sh <repo>` from the v2.0.0 payload
      (or `rollout-fleet.sh <payload> <root>` for batch mode after the first
      three repos succeed manually). Installer preserves `config.yaml`, tasks,
      BRAIN/memory, and writes the traceability cutoff = repo HEAD.
   c. Migrate data: `task-migrate.sh` runs inside install (flat TASK-*.md ->
      spec.md layout for sachviet-class repos); status page regenerated as v3.
   d. Validate: `audit-fleet.sh 2.0.0 <repo>`; open the page headless (the DOM
      suite binary runs against any emitted page: `node
      tools/docs-site/tests/test_status_dom.mjs <repo>/docs/status/index.html`);
      counts on the page equal `find docs/tasks -name spec.md | wc -l`; commit
      hook fires advisory on a test commit (then reset).
   e. Commit: `commit-fleet.sh` posture - one commit per repo,
      `chore(cyberos): upgrade to 2.0.0 - status v3 + traceability (TASK-...)`,
      citing the §7 fleet task. Do not push repos the operator has not cleared
      for pushing.
3. Exceptions: any repo failing b/c/d is REVERTED (`git reset --hard
   pre-cyberos-2.0` + restore stash) and recorded, never left half-migrated.
4. Report: `docs/reviews/fleet-status-v3-migration-<date>.md` - one row per repo:
   before-version, after-version, tasks, page fp, validation results, action
   taken, exception detail if any. Template in §6.

Gate P7 (HITL): operator reviews the report; exceptions get follow-up tasks.
Rollback per repo: the `pre-cyberos-2.0` tag; fleet-wide: re-run rollout with the
v1.10.0 payload (kept at `dist/` from the pre-bump commit or rebuilt from the tag).

### Phase 8 - documentation, monitoring, close-out

1. Docs: `docs/reference/status-feed.md` (contract spec, versioning policy);
   traceability runbook (`docs/runbooks/`: what to do when the gate fails, ledger
   procedure, cutoff policy); update `tools/install/README.md`, plugin command
   help, `AGENT-ENTRY.md` scaffold text, module CHANGELOGs; retire stale
   references to lenses/tabs (`docs/status/reference/` redirects).
2. Post-rollout watch (one week): deploy.yml/suite-gate stay green; traceability
   trend on the mothership page moves toward 100% for new commits; file follow-up
   tasks for any hook-cost complaint (target: <1s on code-only commits).
3. Run `/cyberos:improve` after the first 10 gated tasks complete, per house
   process, to fold lessons back into skills.

## 4. Test plan (consolidated)

| Layer | Location | Covers |
| --- | --- | --- |
| Generator unit + fixtures | `tools/docs-site/tests/` | feed schema, epoch duplicates, shorthand/lists, ledger precedence, exempts, ghosts, no-git, remote parsing, determinism (double-run byte equality) |
| Render stamp | `scripts/tests/test_render_stamp.sh` (extend) | fp covers all inputs incl. templates + ledger + commit set |
| DOM behavior | `tools/docs-site/tests/test_status_dom.mjs` (ported 47-assertion suite, jsdom) | all six bands, sync-without-jumps, second-click navigation, compact 05/06, GitHub links, filters, drawer, deep links, legacy hash redirects (new assertions) |
| Hooks | `scripts/tests/` new `test_task_link_gate.sh` | msg mode advisory/strict/exempt/trailer; range mode cutoff, offenders, zero-sha fallback |
| Pipeline scenarios | scratch-clone script in `tools/install/tests/` | Phase 3 gate scenarios; offline file:// scenario |
| Fleet | `fleet-install-test.sh` + `audit-fleet.sh` | scratch install completeness; per-repo post-migration audit |
| CI wiring | suite-gate.yml, payload-gate.yml, traceability.yml | everything above runs on PR |

## 5. Rollback matrix (summary)

| Change | Lever | Blast radius |
| --- | --- | --- |
| New page | `CYBEROS_STATUS_LEGACY=1` or revert Phase 2 PR | page only |
| Wide regen trigger | restore narrow regex | hook cost only |
| CI gate | remove required check; keep advisory hook | contribution flow |
| Installer defaults | `traceability.strict:false`, `scaffold_ci:false` | consumers |
| v2.0.0 | tag not cut / v2.0.1 with legacy default | release channels |
| Fleet repo | `pre-cyberos-2.0` tag reset | that repo |

## 6. Fleet inventory snapshot (2026-07-27, re-verify at execution)

22 consumer installs; scan commands in Phase 7.1. Worktrees (`strategem-wt-*`),
`cyberos-12c-pr153`, `practice` are excluded duplicates; `cyberos-12g-clone`
(v1.5.1, 579 specs) is a mothership clone - confirm with operator, expected
action: skip or delete, never migrate independently.

| Repo | Ver | Page | Specs | Dirty | Class |
| --- | --- | --- | --- | --- | --- |
| CyberSkill/strategem | 1.0.0 | yes | 136 | 4 | large corpus |
| CyberSkill/landing-page | 1.0.0 | yes | 111 | 3 | large corpus |
| CyberSkill/shopass | 1.0.0 | yes | 91 | 0 | large corpus |
| CyberSkill/sachviet | 1.1.0 | yes | 59 + 20 flat | 2 | LAYOUT MIGRATION |
| CyberSkill/tamagochi | 1.0.0 | yes | 53 | 6 | medium |
| Personal/kristen-calendar | 1.0.0 | yes | 28 | 6 | medium |
| CyberSkill/cyber-click | 1.0.0 | yes | 17 | 6 | medium |
| Personal/dom-defender | 1.0.0 | yes | 13 | 6 | medium |
| CyberSkill/ssl | 1.0.0 | yes | 8 | 0 | small |
| Personal/3d-preriodic-table | 1.0.0 | yes | 6 | 6 | small |
| CyberSkill/token-saver | 1.1.0 | no | 0 | 3 | no page - verify why |
| 9 further zero-task repos (wife-cv, issue-hunter, my-cv, styx, shared, code-audit-framework, code-audit-field-data, gam, design-system-audit-framework, quote-mind) | 1.0.0 | yes | 0 | 0-8 | trivial |

Exception report row format:
`repo | before-ver | after-ver | specs before/after | page fp | audit result |
dom suite | action | exception (verbatim error) | follow-up task id`.

## 7. Proposed task set (input for /cyberos:create-tasks)

Module `docs` unless noted; ids indicative - task-author assigns finals.
`->` = depends_on.

| # | Task | Phase | Deps | Acceptance core |
| --- | --- | --- | --- | --- |
| T1 | Land preview + enforcement trio on main behind PR | 0 | - | P0 gate |
| T2 | traceability required check + cutoff registry (config.yaml) | 0/4 | T1 | strict blocks in scratch repo |
| T3 | status-feed@1 emission + validation in render-status-hub (pair-mirrored) | 1 | T1 | P1 gate, schema test |
| T4 | Epoch/shorthand/ledger commit classification in generator | 1 | T3 | fixtures green, 68% reproduced on mothership |
| T5 | Ledger relocation + task-lint validation | 1 | T3 | unknown-task entry fails lint |
| T6 | v3 client into modules/templates + shell swap + legacy emission | 2 | T3 | P2 gate |
| T7 | Legacy hash redirect map + noscript regeneration | 2 | T6 | new DOM assertions |
| T8 | Port DOM suite to tools/docs-site/tests | 2 | T6 | 47+ assertions in run_all |
| T9 | Delete preview folder, docs redirect | 2 | T6-T8 | no dangling links (check_doc_anchors) |
| T10 | Wide regen trigger + --coverage-only fast path | 3 | T6 | P3 scenarios, <1s code-only |
| T11 | deploy.yml trigger widening + served-page freshness check | 3 | T6 | post-deploy curl assert |
| T12 | Installer: hook + checker in payload lib, config scaffold, CI template | 4 | T2 | fleet-install-test green |
| T13 | Release-range preflight (warn now, block at 2.0.0) | 4/6 | T2 | tag dry-run behavior |
| T14 | Mothership violation triage into ledger (148 rows, optional truth recovery) | 4 | T5 | list disposition recorded |
| T15 | Payload/channel verification matrix execution | 5 | T12 | §Phase 5 table recorded |
| T16 | Offline scratch-repo certification | 5 | T12 | file:// scenario script |
| T17 | v2.0.0 release: changelog, Release-As PR, tag runbook, artifacts | 6 | T13, T15 | P6 gate |
| T18 | Fleet discovery re-scan + classification report | 7 | T17 | report committed |
| T19 | Pilot migrations (one zero-task, one medium, sachviet layout case) | 7 | T18 | 3 clean audits |
| T20 | Fleet rollout remaining repos + exception handling | 7 | T19 | P7 gate report |
| T21 | Docs: status-feed spec, traceability runbook, help/AGENT-ENTRY text | 8 | T6, T12 | doc anchors green |
| T22 | Post-rollout watch + /cyberos:improve pass | 8 | T20 | follow-ups filed |

## 8. Risks

1. Hook cost on every commit (Phase 3) - mitigated by `--coverage-only` fast path
   and the loud-warn-never-block posture; measured gate (<1s) before widening.
2. Pair/copy drift between `tools/docs-site`, `tools/install/docs-tools`,
   `.cyberos`, `dist/cyberos` - rely on check-pair-parity + payload build hook;
   executing models must never hand-edit derived copies.
3. Consumer repos with non-GitHub or no remotes - links and CI degrade by design;
   the hook is the universal enforcement layer; validated in the offline cert.
4. jsdom as a test dependency - confined to docs-site tests; npm-supply-chain
   workflow must pass; never shipped in the payload.
5. Large-corpus render cost (strategem 136+, mothership 586) - the generator is
   already O(corpus) and proven at 586; DOM suite runs against the largest fleet
   page during Phase 7.
6. `cyberos-12g-clone` and worktrees double-counting or getting divergent
   upgrades - explicit exclusion list confirmed with operator at Phase 7 gate.
7. Version classifier surprises - `Release-As: 2.0.0` trailer pins the outcome.

## 9. Decisions taken in this plan and open questions for the operator

Decided (change requires operator override): fold extractor into
render-status-hub rather than a second tool; ledger at `docs/tasks/_state/`;
per-repo cutoff = install/upgrade HEAD; legacy page kept exactly one minor cycle;
fleet upgraded from the v2.0.0 payload, not from git main.

Open (answer at the P0 gate): (1) confirm `cyberos-12g-clone` disposition;
(2) may fleet repos be auto-committed by `commit-fleet.sh`, and which may be
pushed; (3) should consumer CI scaffolding default on or opt-in
(`traceability.scaffold_ci` default true or false); (4) is one minor cycle the
right legacy-page window.
