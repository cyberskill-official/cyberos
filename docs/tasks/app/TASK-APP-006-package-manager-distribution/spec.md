---
id: TASK-APP-006
title: "Package manager distribution — Homebrew Cask (macOS) and winget (Windows) manifests, external-repo PR submission"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: app
priority: p1
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 4
owner: Stephen Cheng
created: 2026-07-12
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-APP-001, TASK-APP-003, TASK-APP-004, TASK-APP-005]
depends_on: []
blocks: []
source_pages:
  - ".github/workflows/release.yml (lines 60–66: gh release create/upload pattern establishing the stable download-URL shape; line 114–115: VERSION-stamping mechanism)"
  - apps/desktop/src-tauri/tauri.conf.json (updater endpoint confirms repo slug `cyberskill-official/cyberos` and the `.../releases/.../download/<file>` URL pattern already in production use)
source_decisions: []
language: Ruby (Homebrew Cask DSL), YAML (winget manifest schema v1.x), bash (CI automation)
service: apps/desktop/src-tauri
new_files:
  - homebrew-cask-manifest/cyberos.rb
  - winget-manifest/CyberSkill.CyberOS/CyberSkill.CyberOS.yaml
  - winget-manifest/CyberSkill.CyberOS/CyberSkill.CyberOS.installer.yaml
  - winget-manifest/CyberSkill.CyberOS/CyberSkill.CyberOS.locale.en-US.yaml
  - .github/workflows/release-pkgmgr-pr.yml
  - docs/deploy/package-manager-submission.md
modified_files: []
allowed_tools:
  - Homebrew's own audit tooling (`brew audit --cask`, `brew style --cask`) for local pre-submission validation only — no live `brew` installation is assumed to exist in this task's CI, only referenced as the tool a human/CI runs before proposing a PR
  - "`winget validate` / Microsoft's winget-pkgs manifest schema for local pre-submission validation only"
  - "`wingetcreate` and Homebrew's own PR-update tooling (`brew bump-cask-pr`-equivalent) for generating manifest update diffs — exact invocation confirmed against each tool's current CLI at implementation time (§9), not asserted here"
disallowed_tools:
  - Any tool that would open a pull request against the external `homebrew/homebrew-cask` or `microsoft/winget-pkgs` GitHub repositories without a fresh, explicit chat-turn confirmation from Stephen at submission time — mirroring TASK-APP-005's Flathub gate for the same underlying reason (this is a distinct, later, irreversible action from drafting a manifest, not covered by the overall PLAN approval)
  - Any tool that would enter GitHub personal-access-token material for `wingetcreate`/Homebrew PR automation into a non-secret-manager location
effort_hours: 16
# Effort is split roughly evenly across the two ecosystems (Cask + winget authoring,
# ~7h combined) plus shared CI/docs work (~9h); see subtasks below for the breakdown.
subtasks:
  - "Author homebrew-cask-manifest/cyberos.rb: version/sha256/url pointing at the GitHub Release .dmg asset, app stanza, livecheck block (3h)"
  - "Author the three-file winget manifest set (version, installer, locale) per Microsoft's current manifest schema, installer URL pointing at the GitHub Release .msi/.exe asset (4h)"
  - "Validate both manifests locally (brew audit --cask, winget manifest schema validation) before ever proposing either PR (3h)"
  - "Author .github/workflows/release-pkgmgr-pr.yml: prepares (but per this task's disallowed_tools does not submit) updated manifest diffs on each tagged release, gated on repo variables PKGMGR_CASK_RELEASE / PKGMGR_WINGET_RELEASE (3h)"
  - "Write docs/deploy/package-manager-submission.md answer sheet (Homebrew Cask quality-bar checklist, winget validation-pipeline checklist, PAT scope requirements for update automation) (3h)"
risk_if_skipped: "CyberOS macOS/Windows users can already obtain .dmg/.pkg (TASK-APP-003 territory) and .msi/.exe/.msix (TASK-APP-004 territory) installers from GitHub Releases today. This remains fully functional. Deferring costs the one-command power-user install path (`brew install --cask cyberos`, `winget install CyberSkill.CyberOS`) that many developers and IT-managed Windows fleets expect — a distinct audience from app-store browsers, not core functionality, and like TASK-APP-005, this costs zero paid-account friction to eventually pursue."
---

## §1 — Description

1. CyberOS **MUST** gain two independent package-manager listing paths — Homebrew Cask (macOS) and winget (Windows) — each pointing at the existing GitHub Releases installer artifacts (the `.dmg` for Cask, matching TASK-APP-003's Developer-ID-signed channel not the Mac-App-Store `.pkg` channel; the `.msi`/NSIS `.exe` for winget, matching TASK-APP-004's GitHub-Releases NSIS channel not the MSIX Store channel) — not producing new build artifacts of their own.

2. This task **MUST** be treated as explicitly out-of-scope-for "store" semantics, consistent with its task-list framing ("package managers, not stores"): neither Homebrew Cask nor winget performs app review, sandboxing enforcement, or discovery/search placement — both are community-maintained manifest registries that point at CyberOS's own already-built artifacts. Precisely: the `.dmg` and NSIS `.exe` this task's manifests point at are produced by the pre-existing, always-on `desktop` job in `release.yml` (confirmed lines 69–139, no opt-in gate), which runs independent of and predates both TASK-APP-003 and TASK-APP-004. TASK-APP-003 and TASK-APP-004 add *separate*, additionally-gated artifacts (a MAS `.pkg`, an MSIX package) this task does **NOT** consume — this task does not duplicate either task's signing decisions, and its manifests' correctness has no dependency on TASK-APP-003/004 landing at all.

3. `homebrew-cask-manifest/cyberos.rb` **MUST** declare `version`, `sha256`, `url` (pointing at a GitHub Releases `.dmg` asset URL, per the URL pattern already confirmed in production use by `tauri.conf.json`'s updater `endpoints` field: `https://github.com/cyberskill-official/cyberos/releases/.../download/<file>`), `name`, `desc`, `homepage`, and an `app "CyberOS.app"` stanza, plus a `livecheck` block pointing Homebrew's own automated version-checking bot at the GitHub Releases API so Homebrew's infrastructure can detect new CyberOS releases independent of CyberOS's own CI.

4. The winget manifest set (`CyberSkill.CyberOS.yaml`, `CyberSkill.CyberOS.installer.yaml`, `CyberSkill.CyberOS.locale.en-US.yaml`) **MUST** follow Microsoft's current three-file manifest schema (`PackageIdentifier: CyberSkill.CyberOS`, `PackageVersion`, `InstallerType`, `InstallerUrl`, `InstallerSha256`) with the installer URL pointing at the equivalent GitHub Releases Windows artifact.

5. Neither manifest **MUST** be submitted (i.e. no PR opened against `homebrew/homebrew-cask` or `microsoft/winget-pkgs`) as a direct or automatic consequence of this task landing — submission is a distinct, later, Stephen-gated action per §1's `disallowed_tools`, exactly mirroring the TASK-APP-005 Flathub-PR gate.

6. CI **MAY** prepare (stage, locally validate, diff-generate) updated manifests on each tagged release behind two independent repo variables, `PKGMGR_CASK_RELEASE=true` and `PKGMGR_WINGET_RELEASE=true`, but **MUST NOT** call `gh pr create`, `brew bump-cask-pr`, `wingetcreate submit`, or any equivalent submission command against either external repo under any CI-flag state — the flags gate manifest *preparation*, never *submission*, which stays a manual, explicit, per-instance human action regardless of flag state.

7. Both manifests' version/URL/hash fields **MUST** be kept re-derivable from the existing GitHub Releases artifacts and the existing VERSION-stamping mechanism (confirmed in `release.yml`: `node scripts/stamp-release-version.mjs --apply`) rather than hand-maintained as a second source of truth that can drift from the actual shipped version.

8. A `docs/deploy/package-manager-submission.md` answer sheet **MUST** document Homebrew Cask's quality-bar checklist (stable URL, no interactive-only installer, no existing duplicate cask) and winget's validation-pipeline checklist (silent-install flag support, installer signature/hash correctness), plus the exact GitHub PAT scopes needed if update automation via `wingetcreate`/Homebrew tooling is later adopted.

9. The Cask manifest's `zap trash:` uninstall-cleanup paths **MUST** be verified against what CyberOS's actual macOS build writes to disk (application-support directory, preferences plist) before the manifest is treated as final — §3's skeleton lists plausible candidates derived from the confirmed `os.cyberskill.world.desktop` Tauri identifier, not confirmed-observed paths, and an incorrect `zap` stanza either leaves stale files behind on uninstall or, worse, deletes paths CyberOS doesn't actually own.

## §2 — Why this design

**Why is the "never submit automatically" rule (§1 #5, #6) even stricter than TASK-APP-005's Flathub gate, extending to update PRs and not just the initial submission?** Flathub's manifest lives in a repo CyberOS effectively owns once accepted (a per-app repo under the `flathub` GitHub org that Stephen would have write access to after initial approval), so *update* automation there is plausible future work with a narrower blast radius. Homebrew Cask and winget-pkgs are both large, shared, single monorepos (`homebrew/homebrew-cask`, `microsoft/winget-pkgs`) serving thousands of unrelated packages — an automation bug that fires against the wrong branch, includes malformed YAML, or double-submits has a shared-repo blast radius that a single-app-repo mistake on Flathub doesn't. Treating every PR — first submission and every subsequent version-bump — as requiring a fresh human go-ahead is the more conservative posture warranted by that difference, not an arbitrary inconsistency with TASK-APP-005.

**Why isn't this task just "TASK-APP-003 and TASK-APP-004's Homebrew/winget sections," given it reuses their artifacts?** Package-manager listing is a genuinely separate audience and submission process from app-store review — a developer typing `brew install --cask cyberos` never interacts with the Mac App Store review pipeline TASK-APP-003 builds, and the two have independent quality bars, independent external repos, and independent (community-maintained, not Apple/Microsoft-controlled) review processes. Folding this into TASK-APP-003/004 would conflate "get CyberOS into Apple's/Microsoft's curated stores" with "get CyberOS listed in a community package index," which are different enough asks that a shared task would either under-serve one or bloat past a coherent single scope — this is the same reasoning that kept TASK-APP-005's Snap Store and Flathub sub-tracks inside one task (both are Linux "stores" in the task list's own framing) while keeping this task (explicitly "package managers, not stores") separate.

**Why bundle Homebrew Cask (macOS) and winget (Windows) into one task rather than splitting per-OS, mirroring how TASK-APP-005 bundled Snap Store and Flathub into one Linux task?** Both mechanisms share an identical shape — a community-maintained manifest registry, external to CyberOS's own CI, pointing at an already-built GitHub Release artifact, gated by the identical "prepare but never auto-submit" policy (§1 #5, #6) — so drafting them as two separate tasks would be near-total duplication of §1's submission-safety language, §2's blast-radius reasoning, and §7's human-gate structure, with no meaningfully separable acceptance criteria or verification surface between the two. This mirrors TASK-APP-005's own reasoning for bundling Snap and Flathub rather than splitting by mechanism, and the same test applies in reverse to justify *not* also bundling this task with TASK-APP-005: Linux's two mechanisms and this task's two mechanisms serve genuinely different OS audiences with no shared CI job, shared manifest format, or shared external-repo relationship, so a four-way bundle would conflate unrelated platforms for no corresponding reduction in duplicated language.

**Why does the winget manifest set split into three separate files (version, installer, locale) while the Cask manifest is a single file?** This is Microsoft's own current `winget-pkgs` schema requirement, not a CyberOS design choice — the three-file split (a version manifest, one or more installer manifests, one or more locale manifests) lets a single package identity carry multiple installer architectures and multiple locale descriptions without duplicating shared fields, a structure Homebrew Cask's single-file DSL doesn't need because Cask has no equivalent multi-locale requirement. §3 shows only the installer file's skeleton because it's the file carrying the fields most likely to need re-derivation on every release (`InstallerUrl`, `InstallerSha256`); the version and locale files change far less often and are generated directly from Microsoft's current manifest templates at WORKER-phase time rather than hand-drafted here against a schema this task's authoring hasn't independently re-verified (§9).

**Why keep version/URL/hash fields re-derivable rather than hand-maintained (§1 #7)?** Both Homebrew Cask and winget manifests encode a SHA256 hash of the exact installer artifact — a hand-maintained hash is a classic source of silent staleness (a new release ships, the manifest isn't updated, and either the old version stays listed or, worse, the hash mismatches the actual current download and both package managers' own validation rejects the install). Deriving these fields from the same VERSION-stamping mechanism and `gh release` artifacts the existing pipeline already produces (rather than a parallel manually-tracked value) keeps the eventual automation (§9) mechanically simple: compute the hash of the artifact `gh release download` just fetched, don't ask a human to type it in.

## §3 — API contract

`homebrew-cask-manifest/cyberos.rb` (structural skeleton):

```ruby
cask "cyberos" do
  version "1.0.0"
  sha256 "REPLACE_WITH_SHA256_OF_RELEASE_DMG"  # computed per §6, not hand-typed

  url "https://github.com/cyberskill-official/cyberos/releases/download/v#{version}/CyberOS_#{version}_universal.dmg"
  name "CyberOS"
  desc "CyberSkill's desktop client — Turn Your Will Into Real"
  homepage "https://os.cyberskill.world/"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "CyberOS.app"

  zap trash: [
    "~/Library/Application Support/CyberOS",
    "~/Library/Preferences/os.cyberskill.world.desktop.plist",
  ]
end
```

`winget-manifest/CyberSkill.CyberOS/CyberSkill.CyberOS.installer.yaml` (structural skeleton — the version and locale manifest siblings follow winget's standard three-file split, omitted here to avoid duplicating boilerplate the WORKER phase generates directly from Microsoft's current manifest templates):

```yaml
PackageIdentifier: CyberSkill.CyberOS
PackageVersion: 1.0.0
InstallerType: nullsoft   # matches the existing NSIS output already confirmed in release.yml's desktop job (windows-latest leg, tauri-action default Windows target)
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/cyberskill-official/cyberos/releases/download/v1.0.0/CyberOS_1.0.0_x64-setup.exe
    InstallerSha256: REPLACE_WITH_SHA256_OF_RELEASE_INSTALLER   # computed per §6, not hand-typed
    InstallerSwitches:
      Silent: "/S"          # NSIS's standard silent-install flag; MUST be confirmed against
      SilentWithProgress: "/S"  # CyberOS's actual NSIS installer config at implementation time,
                                 # not assumed — see §9.
ManifestType: installer
ManifestVersion: 1.6.0   # NOTE: this manifest schema version number and the exact
                          # InstallerSwitches key structure/nesting shown above are this task's
                          # best-available understanding, not independently re-verified against
                          # Microsoft's live current schema during authoring — §9 requires
                          # confirming both against winget-pkgs' current manifest templates
                          # before this file is treated as final, matching the same discipline
                          # applied to the uncertain Snapcraft architectures: form in TASK-APP-005.
```

`.github/workflows/release-pkgmgr-pr.yml` (preparation-only skeleton — never submits, per §1 #5/#6):

```yaml
name: release-pkgmgr-pr
on:
  workflow_dispatch:
jobs:
  prepare-cask-manifest:
    if: vars.PKGMGR_CASK_RELEASE == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Compute SHA256 of the release .dmg
        env: { GH_TOKEN: "${{ github.token }}" }
        run: |
          TAG="${{ github.event.inputs.tag || github.ref_name }}"
          gh release download "$TAG" --repo cyberskill-official/cyberos --pattern "*universal.dmg" --output cyberos.dmg
          sha256sum cyberos.dmg
      - name: Render updated cyberos.rb (staged locally, NOT submitted)
        run: |
          echo "manifest diff generation happens here; artifact is uploaded for Stephen's review,
                no gh pr create / brew bump-cask-pr call exists in this job — see §1 #5/#6."
      - uses: actions/upload-artifact@v4
        with: { name: cyberos-cask-manifest-draft, path: homebrew-cask-manifest/cyberos.rb }
  prepare-winget-manifest:
    if: vars.PKGMGR_WINGET_RELEASE == 'true'
    runs-on: windows-2022
    steps:
      - uses: actions/checkout@v4
      - name: Compute SHA256 of the release installer
        shell: pwsh
        env: { GH_TOKEN: "${{ github.token }}" }
        run: |
          $tag = "${{ github.event.inputs.tag || github.ref_name }}"
          gh release download $tag --repo cyberskill-official/cyberos --pattern "*x64-setup.exe" --output cyberos-setup.exe
          Get-FileHash cyberos-setup.exe -Algorithm SHA256
      - name: Render updated winget manifest set (staged locally, NOT submitted)
        run: |
          Write-Host "manifest diff generation happens here; artifact uploaded for Stephen's review, no wingetcreate submit call exists in this job — see §1 #5/#6."
      - uses: actions/upload-artifact@v4
        with: { name: cyberos-winget-manifest-draft, path: winget-manifest/CyberSkill.CyberOS/ }
```

## §4 — Acceptance criteria

1. **Cask manifest is Ruby-syntax-valid and passes local audit** — `brew audit --cask homebrew-cask-manifest/cyberos.rb` and `brew style --cask homebrew-cask-manifest/cyberos.rb` both exit 0 against a version of the manifest with a real, non-placeholder `sha256` (computed against an actual downloaded artifact, not the literal placeholder string).
2. **winget manifest set is schema-valid** — Microsoft's manifest validation (exact current invocation confirmed at implementation time, §9) passes against all three files with a real, non-placeholder `InstallerSha256`.
3. **No submission command exists anywhere in this task's tooling** — structurally guaranteed, not just documented, by two independent checks so a submission command isn't missed merely for not literally mentioning a target-repo string near it: (a) `grep -rE "gh pr create|brew bump-cask-pr|wingetcreate submit" .github/ tools/` across the *entire* CI/tooling surface, not gated on any repo-name precondition, returns zero matches; (b) a secondary, narrower check confirms no file mentioning `homebrew-cask` or `winget-pkgs` contains any of those same submission-command patterns, for defense in depth. Check (a) is the actual guarantee; check (b) exists only to catch the specific "someone spelled the repo name and the submission call in the same file" case with a clearer error message.
4. **Both CI prep jobs are inert by default** — with `PKGMGR_CASK_RELEASE`/`PKGMGR_WINGET_RELEASE` unset or `false`, both `release-pkgmgr-pr.yml` jobs are skipped (verified the same way as TASK-APP-003/004/005 — unconditional anchor jobs plus `gh run view` conclusion checks, one per job since the two flags are independent).
5. **Version/hash fields are re-derived, never hand-typed in the shipped workflow output** — the `prepare-cask-manifest`/`prepare-winget-manifest` jobs' uploaded draft artifacts contain a `sha256`/`InstallerSha256` value matching the actual `sha256sum`/`Get-FileHash` output computed in the same job run, not a value carried over from a prior manual edit.
6. **Answer sheet is complete** — `docs/deploy/package-manager-submission.md` has a filled-in row for every Homebrew Cask quality-bar item and every winget validation-pipeline item, each marked `human-confirmed` or `not-applicable` with a reason.
7. **No credential material committed** — the repo's existing secret-scan gate passes against every file this task adds.
8. **NSIS silent-install switches are confirmed, not assumed** — `docs/deploy/package-manager-submission.md` records the result of an actual test invocation of CyberOS's real NSIS installer with `/S` (or whatever flag is confirmed correct) before the winget manifest's `InstallerSwitches` block is treated as final, since an incorrect silent-install flag causes winget's own automated validation pipeline to fail installs that require interactive confirmation.

## §5 — Verification

```bash
# AC #1 — Cask manifest local audit (requires a local Homebrew installation; this task does not
# assume one exists in every CI environment, so this is documented as a local/manual verification
# step alongside any CI equivalent implemented at WORKER-phase time)
brew audit --cask homebrew-cask-manifest/cyberos.rb
echo "exit: $?"  # MUST be 0
brew style --cask homebrew-cask-manifest/cyberos.rb
echo "exit: $?"  # MUST be 0

# AC #3 — no submission command anywhere in the repo's tooling.
# Check (a): the real guarantee — scan ALL of .github/ and tools/ for the submission
# command patterns themselves, with no repo-name precondition that a submission call
# could evade by simply not mentioning "homebrew-cask"/"winget-pkgs" nearby.
if grep -rE "gh pr create|brew bump-cask-pr|wingetcreate submit" .github/ tools/ 2>/dev/null; then
  echo "VIOLATION: submission command found (see matches above)"
  exit 1
fi
# Check (b): defense-in-depth — same pattern, scoped to files that also mention either
# target repo by name, purely for a clearer violation message in that specific case.
for f in $(grep -rl "homebrew-cask\|winget-pkgs" .github/ tools/ 2>/dev/null); do
  if grep -qE "gh pr create|brew bump-cask-pr|wingetcreate submit" "$f"; then
    echo "VIOLATION: submission command found in $f (repo-name-adjacent)"
    exit 1
  fi
done
echo "no submission commands found"
```

```yaml
# AC #4 — both CI prep jobs inert-by-default, mirroring TASK-APP-003/004/005's pattern
- name: Assert Cask prep job skipped when PKGMGR_CASK_RELEASE unset
  run: |
    gh run view ${{ github.run_id }} --json jobs -q \
      '.jobs[] | select(.name=="prepare-cask-manifest") | .conclusion' | grep -q skipped
- name: Assert winget prep job skipped when PKGMGR_WINGET_RELEASE unset
  run: |
    gh run view ${{ github.run_id }} --json jobs -q \
      '.jobs[] | select(.name=="prepare-winget-manifest") | .conclusion' | grep -q skipped
```

## §6 — Implementation skeleton

The API contract in §3 covers both manifest skeletons and the preparation-only CI workflow. Two pieces are intentionally deferred rather than fully specified here:

- **The exact winget manifest validation command** (referenced in AC #2) — Microsoft ships this capability as part of the `winget-pkgs` repo's own CI tooling and/or the `winget validate` client subcommand; the precise current invocation should be confirmed against Microsoft's live `winget-pkgs` contribution documentation at implementation time rather than guessed, per the same discipline TASK-APP-005 applied to `flatpak-builder`'s uncertain flag.
- **The manifest-diff-rendering logic** in both `release-pkgmgr-pr.yml` jobs (currently a placeholder `echo`/`Write-Host` in §3) — templating a Ruby Cask file and three YAML winget files from the computed version/hash values is mechanical string substitution best implemented directly against the final manifest structure once §9's NSIS-switch and Homebrew `livecheck` questions resolve, rather than duplicated here against values that may still change.

## §7 — Dependencies

- **Upstream:** none. Consumes GitHub Release artifacts produced by TASK-APP-003 (macOS `.dmg`) and TASK-APP-004/the existing `desktop` job (Windows NSIS `.exe`) without depending on either task's own gating flags (`MAS_RELEASE`, `MSSTORE_RELEASE`) — the GitHub-Releases artifacts this task points at are produced by the pre-existing, always-on `desktop` job, not the store-specific opt-in jobs those two tasks add.
- **Downstream:** none currently drafted.
  - If a future task adopts `wingetcreate`/Homebrew PR-automation tooling (§9, §11's PAT-scope note), it would depend on this task's manifest files and CI prep jobs existing first.
  - No such task is drafted, scheduled, or otherwise planned as of this writing — the dependency is noted here purely so a future author knows to add `depends_on: [TASK-APP-006]` rather than discovering the relationship after the fact.
- **Cross-module:** none.
- **Human/account prerequisites:** none for manifest preparation (no paid account, no Homebrew/Microsoft account needed to author or locally validate either manifest). **Hard blocker on ever opening either PR:** Stephen's explicit, fresh, per-instance chat-turn approval (§1 #5, §4 AC #3 — structurally guaranteed). If update automation via `wingetcreate`/Homebrew PR tooling is adopted later (§9), a scoped GitHub PAT becomes a prerequisite at that time, not before.

## §8 — Example payloads

`brew audit --cask` failure output shape (illustrative — a plausible, not fabricated-as-real, Homebrew audit error for a placeholder hash):

```
Error: cyberos
  sha256 "REPLACE_WITH_SHA256_OF_RELEASE_DMG" is not a valid SHA256 digest
```

Cask manifest submission-guard violation shape (AC #3's local check, illustrative):

```
VIOLATION: submission command found in .github/workflows/release-pkgmgr-pr.yml
```

`winget validate`-style schema failure shape (illustrative — exact tool/invocation confirmed at implementation time per §6, not asserted here as the real Microsoft error text):

```
Manifest Validation Failed.
Field: InstallerSha256
Error: Value does not match a valid SHA256 hash
```

## §9 — Open questions

Deferred:
- **CyberOS's actual NSIS silent-install switch** — §3's winget manifest skeleton assumes the standard NSIS `/S` flag, which is the conventional default but depends on how CyberOS's specific NSIS installer script (generated by `tauri-action`/Tauri's NSIS templating) is configured; AC #8 requires this be confirmed by an actual test run before the manifest is treated as final, not assumed from NSIS convention alone.
- **Winget manifest validation invocation** — deferred per §6; the exact current command/CLI needs confirming against Microsoft's live `winget-pkgs` contribution docs.
- **Whether to adopt `wingetcreate submit`/Homebrew PR-automation tooling for ongoing version-bump PRs, vs. Stephen manually running the equivalent commands per release** — this task deliberately leaves the *submission* step manual regardless of the answer (§1 #6); the open question is only whether the *manifest-diff generation* step (§6) is eventually automated further, which is an implementation-convenience question, not a scope question this task needs to resolve now.
- **Homebrew Cask `zap trash:` paths** — §3's skeleton lists plausible uninstall-cleanup paths (`~/Library/Application Support/CyberOS`, a preferences plist matching the Tauri `identifier`) based on the confirmed `os.cyberskill.world.desktop` identifier convention, but these should be verified against what CyberOS's macOS build actually writes to disk before the cask is finalized, not assumed from the identifier string alone.
- **`related_tasks` references this task as the last item in the same batch as TASK-APP-003/004/005, all now landed** — unlike TASK-APP-003/004/005's own §9 forward-reference notes, this task is authored last in the batch, so all its `related_tasks` entries resolve to already-existing files; no forward-reference disclosure is needed here, and this is noted for symmetry with the prior three tasks' §9 sections rather than because a gap exists.
- **Whether `docs/deploy/package-manager-submission.md` should also catalog Homebrew Cask's and winget-pkgs' publicly documented rejection reasons for past duplicate-package or naming-collision submissions from other developers**, to pre-empt known failure patterns before Stephen's first submission attempt — a nice-to-have research task with no scope-blocking dependency on this task landing, but worth doing before the first PR to reduce rejection/resubmission cycles against two external review queues CyberOS doesn't control the pace of.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Assumed NSIS `/S` silent-install flag is actually wrong for CyberOS's specific installer configuration | winget's own automated validation pipeline fails the submission, reporting an installer that requires interactive input | Submission rejected by winget-pkgs' CI before human review even begins | AC #8 requires confirming the real flag via an actual test run before the manifest is treated as final — this failure mode is the reason that AC exists rather than trusting NSIS convention |
| Homebrew Cask's `livecheck` block strategy (`:github_latest`) doesn't correctly detect CyberOS's release tagging convention (e.g. if tags aren't semver-clean) | Homebrew's own automated livecheck bot reports a livecheck failure on the cask, visible in Homebrew's CI dashboards, not CyberOS's | Cask stays technically installable but Homebrew's staleness-detection stops working for it, risking eventual flagging as outdated | Documented as a manual verification step in the answer sheet (AC #6) — confirm the release-tag format (`v1.0.0`, confirmed in the existing `release.yml`'s `github.event.inputs.tag \|\| github.ref_name` pattern) is compatible with the `:github_latest` livecheck strategy before finalizing |
| `sha256sum`/`Get-FileHash` computed in the CI prep job (§3) doesn't match what a human manually re-downloading the same release artifact would get, due to a mid-release asset re-upload (`gh release upload --clobber`, confirmed used in the existing pipeline) changing the artifact after the prep job already ran | A subsequent PR submission (manual, by Stephen) using a stale hash from an earlier prep-job run gets rejected by Homebrew/winget's own hash-verification | Submission rejected, not silently accepted with a wrong hash (both ecosystems verify the hash against the actual downloaded bytes) | Prep jobs should be re-run against the final release artifacts immediately before Stephen prepares a submission, not relied on from an earlier CI run if the release was `--clobber`-updated since; documented in the answer sheet |
| Cask `zap trash:` paths (§1 #9, §9) are wrong — either incomplete (stale files survive uninstall) or, worse, overbroad (deletes a path CyberOS doesn't actually own, e.g. if the guessed preferences-plist name doesn't match what macOS actually wrote) | Not caught by `brew audit --cask`/`brew style --cask` (AC #1), which validate manifest syntax and metadata conventions, not runtime filesystem behavior; only caught by an actual `brew uninstall --zap --cask cyberos` test against a real installed copy | Silent data-cleanup bug shipped to end users — the failure mode is invisible until someone actually zap-uninstalls | §1 #9 makes this an explicit MUST-verify requirement precisely because AC #1's automated checks cannot catch it; the answer sheet (AC #6) records the result of a real zap-uninstall test as a required manual step, not an assumed pass |
| Two independent CI flags (`PKGMGR_CASK_RELEASE`, `PKGMGR_WINGET_RELEASE`) get accidentally coupled in a future `release-pkgmgr-pr.yml` refactor, so flipping one silently also runs the other | Regression caught only if a future PR's tests specifically re-verify both gates independently | Wrong preparation job runs unexpectedly (still not a submission, per §1 #6, so blast radius stays bounded even in this failure mode) | Same mitigation pattern as prior tasks in this batch: each gate's inert-by-default assertion (AC #4, both flags checked separately) is a standing regression test, not a one-time check |
| A future contributor, unaware of §1 #5/#6's stricter-than-TASK-APP-005 rule, adds a `wingetcreate submit`/`brew bump-cask-pr` call directly into `release-pkgmgr-pr.yml`, believing the existing `PKGMGR_*_RELEASE` flags make this safe to gate the same way TASK-APP-003/004's signing steps are gated | AC #3's structural grep-based guard starts failing the moment such a call appears, regardless of which flag or condition wraps it | Would silently convert a preparation-only job into an actual external-repo submission pipeline if merged unnoticed | AC #3 makes this a standing, automatically-enforced regression test rather than relying on institutional memory of §2's shared-monorepo blast-radius reasoning |
| Cask `app "CyberOS.app"` stanza assumes the `.dmg`'s internal app-bundle name matches exactly; a rename or Tauri config change could silently break this | `brew install --cask cyberos` fails at the `app` stanza's copy step, caught by AC #1's local audit only if the audit actually mounts and inspects the `.dmg` (some `brew audit` checks are metadata-only and would not catch this) | Cask technically passes lint but fails to actually install for end users | Documented in the answer sheet as a required manual end-to-end install test (`brew install --cask ./cyberos.rb` against a locally-built `.dmg`) before ever proposing the PR, not relied on from `brew audit`/`brew style` alone |
| A future task that adopts `wingetcreate`/Homebrew PR-automation tooling (§9) provisions an overly broad GitHub PAT (e.g. full `repo` scope instead of the minimum fork-and-PR permission set the chosen tool actually needs) | Not caught by anything in this task's own CI, since this task ships no such automation itself — would only surface in a future security review of whatever token that later task actually provisions | An unnecessarily powerful credential sits in CI secrets, widening blast radius if ever leaked, disproportionate to what version-bump PR automation actually requires | §11 documents this as a requirement for whichever future task adds the automation: scope the PAT to the minimum permission set the chosen tool needs, not the broadest available token type |

## §11 — Implementation notes

- **This task's "never auto-submit" rule is deliberately stricter than TASK-APP-005's Flathub gate** (§2 explains why: shared-monorepo blast radius vs. a per-app repo) — implementers extending this pattern to future package-manager integrations should re-derive the right posture from each ecosystem's actual repo structure, not copy this task's specific rule mechanically without re-checking whether the same reasoning applies.
- **Both manifest skeletons in §3 use placeholder hash strings (`REPLACE_WITH_SHA256_OF_RELEASE_DMG`, `REPLACE_WITH_SHA256_OF_RELEASE_INSTALLER`) rather than a plausible-looking fabricated hash** — consistent with TASK-APP-004's `CHANGEME-PENDING-PARTNER-CENTER-RESERVATION` placeholder pattern, an unambiguous placeholder is safer than a real-looking-but-fake value that could be mistaken for actual data.
- **The NSIS silent-switch and winget-validation-command open questions (§9) are both flagged for the same underlying reason**: this task's authoring did not execute either tool against a real CyberOS build, so asserting exact behavior would cross from "documented design" into "fabricated verification," which the authoring discipline forbids regardless of how conventional the assumed defaults are.
- **If update automation via `wingetcreate`/Homebrew PR tooling is adopted later** (§1 #8, §9, and the PAT-scope failure-mode row above), the resulting GitHub PAT MUST be stored using the same secret-manager pattern already established elsewhere in this batch (e.g. `MSSTORE_CERT_BASE64`, `MAS_CERT_BASE64`) — this task adds no such secret itself, consistent with §1 #6's stricter never-automate-submission posture.
- **This task's answer sheet (`docs/deploy/package-manager-submission.md`) and TASK-APP-005's Linux submission answer sheet should be reviewed together, not independently**, before Stephen's first actual PR to any of the four external repos (Snap Store, Flathub, `homebrew/homebrew-cask`, `microsoft/winget-pkgs`) — several of the human-confirmed items across both documents (stable download URL shape, VERSION-stamping mechanism, hash re-derivation discipline) trace back to the same underlying CI facts, and reviewing them together reduces the chance of an inconsistent answer between the two.

*End of TASK-APP-006.*
