# Package manager submission — Homebrew Cask + winget answer sheet (TASK-APP-006)

Package managers, not stores: both targets are community-maintained manifest monorepos (`homebrew/homebrew-cask`, `microsoft/winget-pkgs`) pointing at CyberOS's own GitHub-Releases artifacts (produced by the always-on `desktop` job — no dependency on the MAS/MSIX channels). **Submission is never automated**: `release-pkgmgr-pr.yml` only *prepares* re-derived manifest drafts behind `PKGMGR_CASK_RELEASE` / `PKGMGR_WINGET_RELEASE`; every PR against either external monorepo — first submission and every version bump — needs Stephen's fresh per-instance approval (stricter than the Flathub gate on purpose: shared-monorepo blast radius, spec §2). The workflow's standing guard fails CI if any submission command ever appears under `.github/` or `tools/`.

Review this sheet TOGETHER with `docs/deploy/linux-store-submission.md` before the first PR to any of the four external repos (spec §11) — the URL-shape, stamping, and hash-derivation answers trace to the same CI facts.

## Homebrew Cask — quality-bar checklist

| Item | Prepared answer | Status |
|---|---|---|
| Stable, versioned download URL | `https://github.com/cyberskill-official/cyberos/releases/download/v<ver>/CyberOS_<ver>_universal.dmg` — pattern already in production (tauri.conf.json updater endpoint) | human-confirmed (structural) |
| No interactive-only installer | `.dmg` drag-install via `app "CyberOS.app"` stanza — no installer UI at all | human-confirmed (structural) |
| No duplicate/name-collision cask | Search the cask monorepo for `cyberos` immediately before submitting (queue moves fast; check at submission time, not authoring time) | pending-human (at PR time) |
| Real sha256 | Re-derived by the prep job from the actual artifact (AC #5, render-assert enforced); placeholder never submittable — brew audit rejects it | human-confirmed (structural) |
| `livecheck` strategy vs tag format | `:github_latest` against `v<semver>` tags (release.yml confirms the `vX.Y.Z` convention) — verify Homebrew's livecheck accepts it in the audit run | pending-human (verify in audit) |
| `zap trash:` paths verified (spec §1 #9) | **Required real test:** install a release build, exercise it, `brew uninstall --zap --cask cyberos`, confirm the two candidate paths are exactly what the app wrote (`~/Library/Application Support/CyberOS`, `~/Library/Preferences/os.cyberskill.world.desktop.plist`) — audit/style CANNOT catch a wrong path | pending-human (real zap test) |
| End-to-end install test | `brew install --cask ./cyberos.rb` against a locally-downloaded `.dmg` — catches an `app` stanza / bundle-name mismatch metadata-only audits miss (spec §10) | pending-human (pre-PR) |
| `brew audit --cask` + `brew style --cask` exit 0 | Run on a Mac with Homebrew against the RENDERED draft (real hash), not the in-repo placeholder | pending-human (pre-PR) |

## winget — validation-pipeline checklist

| Item | Prepared answer | Status |
|---|---|---|
| Three-file manifest set schema-valid | `winget validate --manifest winget-manifest/CyberSkill.CyberOS` (exact current invocation: confirm against live winget-pkgs contribution docs — spec §6) against the RENDERED drafts | pending-human (pre-PR) |
| `ManifestVersion` + `InstallerSwitches` nesting | 1.6.0 / nested-switches form is best-available understanding, NOT independently re-verified — regenerate from the live winget-pkgs templates if validation complains (hedge recorded in-file) | open (engineering, pre-PR) |
| Silent-install switches confirmed (AC #8) | **Required real test:** run the actual release `CyberOS_<ver>_x64-setup.exe /S` on a Windows machine/VM and confirm a fully non-interactive install; record the result here. Wrong switch = winget's own pipeline rejects before human review | pending-human (real test) |
| Real InstallerSha256 | Re-derived by the prep job (AC #5, render-assert enforced) | human-confirmed (structural) |
| Package identity | `CyberSkill.CyberOS` (Publisher.Package convention); collision-check the winget-pkgs repo at submission time | pending-human (at PR time) |
| Installer signature | GitHub-Releases NSIS artifact is Developer-signed only when `MACOS_SIGN`-equivalent Windows signing lands; winget accepts hash-verified unsigned installers but flags SmartScreen reputation — note at review | pending-human |

## Shared operational rules

- **`--clobber` staleness rule:** the existing release pipeline re-uploads assets with `--clobber`; ALWAYS re-run the prep job against the final release state immediately before preparing a submission — a hash from an earlier run may be stale (spec §10).
- **PAT scopes (future automation only):** this task ships no PR automation and no PAT. If a future task adopts `wingetcreate`/Homebrew bump tooling, scope the PAT to the minimum fork-and-PR permission set that tool needs (never a full-`repo` classic token), stored like the other release secrets (`*_BASE64` pattern).
- **Version-bump PRs are also Stephen-gated** — not just the first submission (spec §1 #5/#6).
