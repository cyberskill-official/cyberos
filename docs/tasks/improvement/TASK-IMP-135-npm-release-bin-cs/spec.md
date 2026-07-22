---
id: TASK-IMP-135
title: "Publish npm release shipping bin.cs for the rename"
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-130]
blocks: [TASK-IMP-133]
related_tasks: [TASK-IMP-069, TASK-IMP-071, TASK-IMP-134]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.9"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 2
service: .github/workflows/release.yml
new_files:
  - (none — operational cut of an existing audited pipeline)
modified_files:
  - "VERSION (via the audited version.yml / scripts/cyberos-version.mjs bump on main, or an explicit Release-As / --set if the auto-bump has not yet landed)"
  - "CHANGELOG.md (promote the Unreleased Breaking rename entry into the dated release section for the cut version)"
source_pages:
  - "docs/tasks/improvement/TASK-IMP-133-homebrew-tap-cs-rename-followup/audit.md ISS-005 (batch gap: no task owns cutting and publishing the npm release both IMP-133 and IMP-134's manual checklist depend on)"
  - "docs/tasks/improvement/TASK-IMP-133-homebrew-tap-cs-rename-followup/spec.md Edge cases (same gap named in prose) and Dependencies (requires a PUBLISHED npm release with bin.cs, not merely merged code)"
  - "docs/deploy/RELEASE.md §'npm (the npx-cli channel)' and §'npm: first publish, then trusted publishing' (OIDC trusted publishing via release.yml; no NPM_TOKEN; Workflow filename release.yml is part of the trust pin)"
  - "docs/deploy/RELEASE.md §'Versioning: auto-bump, manual release' and §'Part B: every release' (land on main via PR → VERSION bump → tag vX.Y.Z → release.yml publishes)"
  - ".github/workflows/release.yml npm job (lines ~160-217): builds payload via tools/install/build.sh, asserts name=@cyberskill/cyberos and repository.url, then npm publish --access public under id-token: write; skips if version already published"
  - "tools/install/build.sh:351-352 (current HEAD on the rename branch: bin field already emits \"cs\": \"cli/bin/cli.mjs\")"
  - "CHANGELOG.md:5-8 ([Unreleased] Breaking entry for the cyberos→cs rename, citing TASK-IMP-130 — not yet cut into a dated release section)"
  - "npm registry live query 2026-07-23: npm view @cyberskill/cyberos version → 1.0.9; npm view @cyberskill/cyberos@1.0.9 bin → { cyberos: 'cli/bin/cli.mjs' } — published package still ships the pre-rename bin"
  - "scripts/cyberos-version.mjs --check on the rename branch tip: current 1.0.9 → next 1.1.0 (minor) from feat(install) commits since the last VERSION touch"
source_decisions:
  - "2026-07-23 Stephen (operator judgment on the npm-release halt for IMP-133): 'do as your judgment' — do not leave the gap unowned; author a proper improvement task that owns cutting/publishing the npm release carrying bin.cs; then attempt the audited release path; HALT with a clear ask if publish is blocked (credentials, main not merged, etc.); never fake publish; never push to main directly; never mark IMP-133 done without real AC satisfaction."
  - "2026-07-23 authoring: the release path is the existing audited OIDC pipeline in release.yml, not a local npm publish with a long-lived token. docs/deploy/RELEASE.md explicitly documents that npm publishes via trusted publishing with no NPM_TOKEN. This task therefore owns driving that pipeline (merge → bump → tag → verify), not inventing an alternate publish channel."
  - "2026-07-23 authoring: projected version is 1.1.0 per scripts/cyberos-version.mjs --check on the rename branch (minor from feat commits). The normative target is 'the first published @cyberskill/cyberos version whose package.json bin field contains cs and does not contain cyberos' — not a hard-coded 1.1.0 string — so an explicit Release-As / --set that lands a different SemVer still satisfies this task if the bin contract holds."
  - "2026-07-23 authoring: TASK-IMP-130 remains the code-change owner; this task owns only the operational cut that turns that code into a registry artifact. Reciprocity: this task depends_on TASK-IMP-130 and blocks TASK-IMP-133; TASK-IMP-133's depends_on gains this task so the previously-unowned publish gap is a real status gate, not only an edge-case note."
  - "2026-07-23 self-audit revision (score_pre_revision 6/10 -> score_post_revision 10/10): first draft's AC 3 cited 'gh run view' prose without a concrete success filter an implementer could re-run identically; tightened to assert conclusion==success on the release.yml npm job for the cut tag. AC 4 originally allowed 'CHANGELOG mentions the rename somewhere' which a leftover Unreleased bullet would satisfy without ever cutting a dated section — required the dated ## [X.Y.Z] heading for the published version. Clause 1.5's 'must not fake publish' was only prose until AC 5 required the live npm view, not a local tarball, as the sole publish evidence. Added explicit non-goal: no local npm publish from a developer laptop (OIDC path only)."
---

# TASK-IMP-135: Publish npm release shipping bin.cs for the rename

## Summary

Cut and publish the first `@cyberskill/cyberos` npm release whose `bin` field is `cs` (not `cyberos`), via this repo's existing OIDC `release.yml` pipeline — closing the plan-level gap TASK-IMP-133's audit ISS-005 named, so the Homebrew tap follow-up has a real registry artifact to pin.

## Problem

TASK-IMP-130's acceptance criteria prove a scratch build's `package.json` declares `bin.cs`. They do not require that build to reach the npm registry. TASK-IMP-133's entire premise — bump `Formula/cyberos-cli.rb`'s `url`/`sha256` and assert `bin/"cs"` — and TASK-IMP-134's manual release-time checklist both assume a published release with `bin.cs` already exists. As of 2026-07-23 the live registry still serves `@cyberskill/cyberos@1.0.9` with `bin.cyberos`, while the rename code lives only on the open PR branch. TASK-IMP-133's own audit (ISS-005) and Edge cases section named this as an unowned gap in the five-task batch; leaving it unowned means IMP-133 stays permanently blocked with no task whose `done` criterion is "the registry artifact exists."

## Proposed Solution

Own the operational cut through the audited release path already documented in `docs/deploy/RELEASE.md` and implemented by `.github/workflows/release.yml`:

1. Land the rename code (TASK-IMP-130 and siblings) on `main` via the normal PR merge — never a direct push to `main`.
2. Let `version.yml` / `scripts/cyberos-version.mjs` produce the next platform `VERSION` (projected `1.1.0` minor from the rename `feat` commits), or apply an explicit `Release-As:` / `--set` if the operator chooses a different SemVer; promote the `[Unreleased]` Breaking rename entry into that dated CHANGELOG section.
3. Tag `v$(cat VERSION)` at the bump commit and push the tag so `release.yml` fires natively (TASK-IMP-071).
4. Wait for the `npm` job (OIDC trusted publishing, `id-token: write`, no `NPM_TOKEN`) to publish `@cyberskill/cyberos@<version>`.
5. Verify live via `npm view` that the published version's `bin` contains `cs` and does not contain `cyberos`.

That live registry fact is the sole publish evidence this task accepts — a local `npm pack` / scratch tarball does not satisfy it.

## Alternatives Considered

- Leave the gap as an Edge case note on TASK-IMP-133 and wait for an ad-hoc operator release. Rejected: Stephen's 2026-07-23 judgment explicitly directed that the gap must not stay unowned; IMP-133 and IMP-134's manual checklist otherwise have no status-gating owner for their shared precondition.
- Publish from a developer laptop with `npm login` + `npm publish`. Rejected: `docs/deploy/RELEASE.md` documents that the package uses trusted publishing (OIDC) pinned to `release.yml`; a laptop publish either requires a token the docs say to revoke, or fights the "Require 2FA and disallow tokens" lock. Not the audited path.
- Fold publish verification into TASK-IMP-130 by amending its ACs. Rejected: IMP-130 is already `done` with scratch-build ACs; reopening it conflates "code correct in tree" with "artifact on registry," and would invalidate the TRACE-004 evidence already recorded for its offline tests.
- Cut the release from the open PR branch without merging to `main`. Rejected: `release.yml` checks out the tag ref; tags on a non-main tip that never merges leave `main` without the rename while the registry claims it shipped — the continuous-delivery and version-bump contracts both assume the bump commit lives on `main`.

## Success Metrics

- Primary: within one release cycle after TASK-IMP-130's code is on `main`, `npm view @cyberskill/cyberos version` returns a version strictly newer than `1.0.9` whose `bin` object has key `cs` and lacks key `cyberos`. Baseline today: `1.0.9` / `bin.cyberos`.
- Guardrail: that version was published by the `release.yml` `npm` job for tag `v<version>` (OIDC), not by a local `npm publish` — confirmed by a successful Actions run for that tag, not by trusting a developer machine's npm auth state.

## Scope

In scope: merging the rename PR to `main` (or confirming it is already merged), the VERSION/CHANGELOG cut for the release that carries `bin.cs`, pushing the `v*` tag that fires `release.yml`, and verifying the live npm package's `bin` field.

### Out of scope / Non-Goals

- Any further change to `tools/install/build.sh`'s `bin` field or CLI dispatch — that is TASK-IMP-130 (already done).
- Updating `Formula/cyberos-cli.rb` in `homebrew-tap` — that is TASK-IMP-133, which this task unblocks.
- Running TASK-IMP-134's manual clean-machine checklist — still a release-process step owned by that spec's Edge cases, not by this task's status transition.
- Publishing desktop/mobile native installers beyond whatever `release.yml` already does for the same tag — incidental; this task's acceptance criteria are npm-bin only.
- A local `npm publish` from a developer laptop, inventing an `NPM_TOKEN`, or renaming `release.yml` (the filename is part of the OIDC trust pin).

## Dependencies

Depends on TASK-IMP-130 — the code that emits `bin.cs` must exist in the tree that the release tag points at. Soft coordination with TASK-IMP-131/132/134: those are already done on the same PR branch and should land with 130; this task does not list them in `depends_on` because the publish contract is specifically "the built payload's bin is cs," which 130 alone defines.

Blocks TASK-IMP-133 — that task MUST NOT merge a Formula pin to a `cs`-bin tarball until this task's live `npm view` evidence exists.

**Relationship to TASK-IMP-069 / TASK-IMP-071.** IMP-069 added the payload release assets and the npm job shape; IMP-071 made `git push origin vX.Y.Z` fire `release.yml` natively (no `[skip ci]` brake). This task consumes both — it does not re-implement them.

## AI Authorship Disclosure

- **Tools used:** Composer (Cursor agent) continuing the CyberOS `task-author` discipline after Stephen's 2026-07-23 judgment call.
- **Scope:** every `source_pages` line was re-read or re-queried in this session (live `npm view`, `cyberos-version.mjs --check`, `release.yml` npm job, IMP-133 ISS-005) rather than carried forward from a prior agent's notes alone.
- **Human review:** scope authorized by Stephen's explicit "do as your judgment" on the npm-release halt; PLAN-style approval treated as given for this single gap-closing task under that instruction.

## 1. Description (normative)

- 1.1 The rename code from TASK-IMP-130 MUST be present on `main` at the commit the release tag points to — landed via PR merge, never a direct push to `main`.
- 1.2 Platform `VERSION` MUST equal the SemVer that will be tagged and published, and `CHANGELOG.md` MUST contain a dated `## [<VERSION>]` section that includes the Breaking `cyberos`→`cs` rename entry (promoted out of `[Unreleased]`).
- 1.3 A git tag `v$(cat VERSION)` MUST exist and MUST point at the bump commit that carries that VERSION; pushing that tag MUST be what triggers `.github/workflows/release.yml` (native `push: tags` path per TASK-IMP-071).
- 1.4 The `release.yml` `npm` job for that tag MUST conclude `success`, publishing `@cyberskill/cyberos@<VERSION>` via OIDC trusted publishing (`id-token: write`) — not via a long-lived npm token and not via a laptop `npm publish`.
- 1.5 After the job succeeds, a live `npm view @cyberskill/cyberos@<VERSION> bin` MUST show a `cs` key and MUST NOT show a `cyberos` key. A local scratch build or `npm pack` artifact MUST NOT be accepted as substitute evidence for this clause.
- 1.6 This task MUST NOT mark itself `done` until clause 1.5's live registry evidence exists. It MUST NOT mark TASK-IMP-133 `done`.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - `git merge-base --is-ancestor <IMP-130-landing-commit> origin/main` exits 0, and `git show origin/main:tools/install/build.sh` contains the literal `"cs": "cli/bin/cli.mjs"` bin entry - test: shell: `git fetch origin main && git merge-base --is-ancestor "$(git log origin/main --grep='TASK-IMP-130' --format=%H | head -1)" origin/main` exits 0 AND `git show origin/main:tools/install/build.sh | grep -F '"cs": "cli/bin/cli.mjs"'` exits 0
- [ ] AC 2 (traces_to: #1.2) - for `V=$(git show origin/main:VERSION | tr -d '[:space:]')`, `CHANGELOG.md` on `origin/main` contains a heading line matching `## [$V]` and that section (until the next `## ` heading) contains both the substring `cyberos` and the substring `` `cs` `` in the rename Breaking bullet - test: shell: extract the dated section for `$V` and `grep -F` for the rename markers; fail if the bullet remains only under `## [Unreleased]`
- [ ] AC 3 (traces_to: #1.3, #1.4) - tag `v$V` exists on `origin`, `git rev-list -n1 v$V` equals the commit where `VERSION` became `$V`, and the GitHub Actions run of workflow `release.yml` for that tag has job `npm` with `conclusion=success` - test: `gh api "/repos/cyberskill-official/cyberos/actions/runs?event=push&per_page=20" --jq ...` (or `gh run list --workflow=release.yml --branch "v$V"`) filters to that tag's run and asserts the npm job success
- [ ] AC 4 (traces_to: #1.5) - `npm view @cyberskill/cyberos@"$V" version` prints `$V`, and `npm view @cyberskill/cyberos@"$V" bin` JSON-parses to an object with key `cs` and without key `cyberos` - test: `node -e 'const b=JSON.parse(require("child_process").execSync("npm view @cyberskill/cyberos@'"$V"' bin --json","utf8")); if(!b.cs||b.cyberos) process.exit(1)'`
- [ ] AC 5 (traces_to: #1.5, #1.6) - AC 4's evidence is recorded against the live registry (command output from `npm view`, not from a local `dist/` or `.tgz`), and this task's frontmatter `status` remains non-`done` until that output is captured; TASK-IMP-133's frontmatter `status` is unchanged by this task - test: verify: the ship-tasks / final-acceptance note for this task pastes the `npm view` stdout, and `git diff` for this task's landing commit does not flip TASK-IMP-133's status

## 3. Edge cases

- If PR #109 (or its successor) is not yet mergeable (failing gates, review block, merge conflict), this task HALTs at clause 1.1 — authoring and prep are done; the operator must unblock the merge. Do not force-merge past red gates.
- If `version.yml`'s Deploy Key / ruleset bypass is missing, the auto-bump may compute `1.1.0` in the run summary without pushing — then the operator (or this task's implementer with an explicit, reviewed bump commit on a PR) must land `VERSION`/`CHANGELOG` via PR before tagging. Still no direct push to `main`.
- If `@cyberskill/cyberos@$V` is already on the registry when the npm job runs, `release.yml` intentionally no-ops the publish (`already published — nothing to do`). That satisfies clause 1.4's job-success requirement only if AC 4 still shows `bin.cs` for that version — a previously published wrong-bin version under the same SemVer is a hard failure requiring a new SemVer, not a re-tag.
- If OIDC trusted-publishing config on npmjs.com drifts (wrong workflow filename, missing `id-token: write`, Node too old), the npm job fails with `ENEEDAUTH` / similar — this task HALTs and surfaces the exact Actions log; do not fall back to a token publish.
- Security-class: this task publishes a public npm package. The audited path uses short-lived OIDC; introducing a long-lived `NPM_TOKEN` to "just get it out" is explicitly forbidden by this task's non-goals and by `docs/deploy/RELEASE.md`.

---

*End of TASK-IMP-135.*
