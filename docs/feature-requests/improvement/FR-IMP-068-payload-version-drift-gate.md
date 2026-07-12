---
id: FR-IMP-068
title: "Payload-version drift gate - CI and git hooks fail when any dist/plugin stamp differs from VERSION"
module: improvement
priority: MUST
status: implementing
class: improvement
verify: T
phase: Wave A - version coupling
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: null
memory_chain_hash: null
related_frs: [FR-IMP-069, FR-IMP-070, FR-APP-001, FR-SKILL-116]
depends_on: []
blocks: [FR-IMP-069]
source_pages:
  - tools/cyberos-init/build.sh
  - .github/workflows/version.yml
  - .pre-commit-hooks/cyberos-payload-build.sh
  - docs/deploy/RELEASE.md
source_decisions:
  - "2026-07-12 operator decision: the distributed Claude plugin version and CyberOS VERSION must always move together on release/deploy. Observed drift: VERSION 1.7.0 vs payload/plugin 1.2.0 (built 2026-07-11, fb8fb61)."
  - "2026-07-12 operator decision: dist/ stays gitignored; consistency is proven by rebuilding in CI, not by committing build output."
language: bash + GitHub Actions YAML
service: tools/cyberos-init/ + .github/workflows/ + .githooks/
new_files:
  - tools/cyberos-init/check-version-sync.sh
  - tools/cyberos-init/tests/test_check_version_sync.sh
  - .github/workflows/payload-gate.yml
  - .githooks/pre-commit
modified_files:
  - tools/cyberos-init/build.sh
  - .pre-commit-hooks/cyberos-payload-build.sh
  - .github/workflows/version.yml
  - docs/deploy/RELEASE.md
---

# FR-IMP-068: Payload-version drift gate

## §1 - Description

The root `VERSION` file is the single platform version, auto-bumped in CI by `version.yml`. The distributable payload (`dist/cyberos`) is stamped from `VERSION` by `tools/cyberos-init/build.sh`, but only when a human runs the build. Nothing compares the two, so the payload and every installed plugin silently lag (observed: 1.2.0 vs 1.7.0). This FR adds the missing comparison and makes it enforceable in CI and locally.

Normative clauses:

1. A script `tools/cyberos-init/check-version-sync.sh <payload-dir>` MUST compare root `VERSION` against every stamped artifact in the payload: `<payload>/VERSION`, `<payload>/plugin/.claude-plugin/plugin.json` `.version`, `<payload>/.claude-plugin/marketplace.json` `metadata.version`, `<payload>/mcp/package.json` `.version`, `<payload>/manifest.yaml` `cyberos_version`, and the `plugin.json` sealed inside `<payload>/cyberos.plugin` (read via `unzip -p`, no extraction to disk). It MUST exit 0 when all six match, exit 10 on any mismatch printing one line per drifted artifact in the form `DRIFT <relative-path>: <found> != <expected>`, and exit 2 when root `VERSION` is missing or not `X.Y.Z` semver.
2. A workflow `.github/workflows/payload-gate.yml` MUST run on push and pull_request to `main` when any of `tools/cyberos-init/**`, `modules/skill/**`, `modules/cuo/**`, or `VERSION` changes. It MUST build the payload into a temporary directory with `build.sh <tmpdir>` and run `check-version-sync.sh <tmpdir>`; a non-zero exit from either MUST fail the workflow.
3. `build.sh` MUST exit non-zero with an explicit error when root `VERSION` is missing or not `X.Y.Z` semver. The current silent fallback (`|| echo 0.0.0`) MUST be removed; a payload stamped `0.0.0` MUST be impossible to produce.
4. A `.githooks/pre-commit` hook MUST be added (the repo's `core.hooksPath` is `.githooks`, so hooks placed only in the pre-commit framework never fire for contributors who skipped `pre-commit install`). When staged paths match the existing trigger list in `.pre-commit-hooks/cyberos-payload-build.sh` (`modules/cuo/ | modules/skill/ | tools/cyberos-init/ | VERSION`), the hook MUST invoke that script to refresh `dist/cyberos` and then run `check-version-sync.sh dist/cyberos`. Rebuild or check failure MUST abort the commit; a clean run MUST NOT block it. Non-matching commits MUST be a no-op.
5. `docs/deploy/RELEASE.md` MUST describe the invariant as enforced (CI gate + wired hook), replacing the current aspirational claim that the pre-commit hook keeps the payload matched.
6. The CI gate MUST NOT require network access beyond checkout and MUST complete its build+check steps in under 3 minutes (the build is file copies plus sed).
7. Because the bot's bump commit carries `[skip ci]` (so `payload-gate.yml` never sees it), `version.yml` MUST run the build+check inline in its own job, immediately after `cyberos-version.mjs --apply` and before pushing - the bump and the proof that a payload builds clean at the new version land together.

## §2 - Why this design

The stamping logic in `build.sh` is already correct and single-source; duplicating version propagation elsewhere would create a second drift surface. The gate therefore re-uses the real build and only adds a read-only comparator. Building into a temp dir in CI keeps `dist/` gitignored (operator decision) while still proving that a build at this commit yields stamps equal to `VERSION`. The comparator is a standalone script so the CI gate, the git hook, FR-IMP-069's release job, and the desktop Ops tab (FR-APP-001) all share one implementation.

## §3 - Contract

### check-version-sync.sh

```
usage: check-version-sync.sh [payload-dir]   # default: <repo>/dist/cyberos
exit 0   in sync (prints "sync OK <version> across 6 artifacts")
exit 10  drift (one "DRIFT <path>: <found> != <expected>" line per artifact)
exit 2   root VERSION missing/invalid, payload dir missing, or artifact unreadable
```

Artifact readers: `VERSION` = trimmed file; JSON fields via `node -p` (node is already a build dependency); `manifest.yaml` via grep of the `cyberos_version:` line; sealed plugin.json via `unzip -p <payload>/cyberos.plugin .claude-plugin/plugin.json`.

### payload-gate.yml

```yaml
name: payload-gate
on:
  push: { branches: [main], paths: [tools/cyberos-init/**, modules/skill/**, modules/cuo/**, VERSION] }
  pull_request: { paths: [tools/cyberos-init/**, modules/skill/**, modules/cuo/**, VERSION] }
jobs:
  build-and-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: bash tools/cyberos-init/build.sh "$RUNNER_TEMP/payload"
      - run: bash tools/cyberos-init/check-version-sync.sh "$RUNNER_TEMP/payload"
```

### .githooks/pre-commit

Delegates to `.pre-commit-hooks/cyberos-payload-build.sh` (trigger-list match) then `check-version-sync.sh dist/cyberos`; both under `set -euo pipefail` so failure aborts the commit.

## §4 - Acceptance criteria

1. **Sync passes on a fresh build** (§1 #1) - immediately after `build.sh`, `check-version-sync.sh` exits 0 and reports the version and artifact count.
2. **Each artifact is individually guarded** (§1 #1) - tampering any one of the six stamps (payload VERSION, plugin.json, marketplace.json, mcp/package.json, manifest.yaml, sealed plugin.json) makes the check exit 10 and name exactly that artifact.
3. **The sealed bundle is checked without extraction** (§1 #1) - re-zipping `cyberos.plugin` with a stale inner plugin.json while all on-disk files match still exits 10.
4. **Invalid VERSION cannot stamp** (§1 #3) - with `VERSION` containing `banana`, `build.sh` exits non-zero and writes no payload; with `VERSION` deleted, same.
5. **No 0.0.0 escape hatch remains** (§1 #3) - `grep -n "0\.0\.0" tools/cyberos-init/build.sh` shows only the placeholder-substitution seds, no fallback echo.
6. **CI gate is wired** (§1 #2, #6) - `payload-gate.yml` exists with the four path filters on both push and pull_request, parses as valid YAML, and its steps are exactly build-into-temp + check.
7. **Hook fires on trigger paths** (§1 #4) - in a fixture repo with `core.hooksPath=.githooks`, committing a change under `modules/skill/` runs the rebuild and check; committing a change under `docs/` runs neither.
8. **Hook failure blocks the commit** (§1 #4) - with `build.sh` forced to fail, the commit aborts with the build error visible.
9. **RELEASE.md tells the truth** (§1 #5) - the doc names `payload-gate.yml` and `.githooks/pre-commit` as the enforcement points; the old aspirational sentence is gone.
10. **Bump commits are self-proving** (§1 #7) - `version.yml` contains the inline build+check steps between apply and push; removing them makes t10's structural assertion fail.

## §5 - Verification

```bash
# tools/cyberos-init/tests/test_check_version_sync.sh
# Harness: builds a scratch payload into $TMP/payload from a scratch VERSION file,
# then mutates artifacts one at a time. Run: bash tools/cyberos-init/tests/test_check_version_sync.sh

t01_fresh_build_syncs()          # AC 1
t02_each_artifact_guarded()      # AC 2  (loop over the 6 artifacts, expect exit 10 + the right DRIFT line)
t03_sealed_zip_checked()         # AC 3  (zip -j a stale plugin.json into cyberos.plugin)
t04_invalid_version_refused()    # AC 4  (VERSION=banana and VERSION absent -> build.sh non-zero)
t05_no_fallback_left()           # AC 5  (static grep assertion)
t06_workflow_shape()             # AC 6  (node yaml-less structural greps: name, both triggers, 4 path filters)
t07_hook_trigger_matrix()        # AC 7  (git init fixture, core.hooksPath=.githooks, two commits)
t08_hook_blocks_on_failure()     # AC 8  (PATH-shadowed failing build.sh -> commit exits non-zero)
t09_release_md_updated()         # AC 9  (grep RELEASE.md for payload-gate.yml + .githooks/pre-commit)
t10_version_yml_inline_check()   # AC 10 (structural greps: build.sh + check-version-sync.sh steps between apply and push)
```

All nine cases green = §5 pass. The suite is plain bash with `set -euo pipefail`, no framework, runnable in CI and locally.

## §6 - Implementation skeleton

`check-version-sync.sh`: read+validate root VERSION; declare the six readers; accumulate drift lines; print and exit per contract. `build.sh` diff: replace `cyver="$(tr -d ' \n\r' < "$repo/VERSION" 2>/dev/null || echo 0.0.0)"` with an existence+regex guard that errors out. Hook: 15-line bash wrapper as in §3.

## §7 - Dependencies

None upstream. Blocks FR-IMP-069 (the release publisher reuses `check-version-sync.sh` as its pre-upload gate). FR-APP-001's desktop Ops tab can surface the same check verbatim.

## §8 - Example payloads

```
$ bash tools/cyberos-init/check-version-sync.sh dist/cyberos
DRIFT dist/cyberos/VERSION: 1.2.0 != 1.7.0
DRIFT dist/cyberos/plugin/.claude-plugin/plugin.json: 1.2.0 != 1.7.0
DRIFT dist/cyberos/cyberos.plugin!.claude-plugin/plugin.json: 1.2.0 != 1.7.0
$ echo $?
10
```

## §9 - Open questions

None blocking. Whether `payload-gate` becomes a required check in the branch ruleset is an operator toggle after one week of green runs.

## §10 - Failure modes inventory

1. CI runner lacks `zip`/`unzip` - `build.sh` already requires zip; the gate installs nothing, so the check script MUST fail with exit 2 and a "unzip missing" message rather than a false pass. Covered by t02 harness precondition.
2. Commit bypasses the hook (`--no-verify`) - accepted; `payload-gate.yml` is the backstop on push/PR.
3. Bot bump commit is invisible to path-triggered workflows (`[skip ci]`) - closed by §1 #7: the bump job itself proves the build before pushing, so no commit lands on main with an unprovable payload state.
4. Interrupted local build leaves a half-written dist - the check reads six artifacts; any missing file is exit 2 (unreadable), never a false 0.
5. manifest.yaml format change breaks the grep reader - t02 pins the reader against the generated manifest; a format change fails the suite at the same commit that changes the generator.

## §11 - Implementation notes

Keep the job name `payload-gate / build-and-check` stable so it can be added to the ruleset's required checks. The check script must not import from `build.sh` (read-only comparator; zero side effects). Reuse in FR-IMP-069 and FR-APP-001 is by invocation, not by copy.

*End of FR-IMP-068.*
