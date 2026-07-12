# CyberOS 1.0.0 go-live checklist — what Stephen needs to do manually

Living document (2026-07-13). Everything code-side is committed and machine-verified. This list is trimmed to what only Stephen can do — credentials, payments, account settings, pushes, and go/no-go calls. Everything else (drafting listing copy, adding testers via the Chrome extension, prepping submissions up to the final click) Claude can drive on request and isn't listed here as a to-do.

## 0. One push unblocks everything

```
git push                                   # pending local commits; Release-As: 1.0.0 pin holds the version
git tag -f v1.0.0 && git push -f origin v1.0.0    # re-cut the release at the icon-fixed commit
```
The re-tag re-runs release.yml. Both mobile blockers are fixed in-repo: the 90717 icon defect (icon flattened, guard asserts alpha-free - proven: iOS lane went green and build 10706 is in TestFlight) and the Play versionCode collision (FR-IMP-078: release builds now stamp `max(BUILD_NUMBER, minutes-since-epoch)`, so re-tags always upload a strictly higher number to both stores).

## 1. iOS App Store

| # | Step | How |
|---|---|---|
| 1 | EU DSA trader declaration | App Store Connect → Business → Digital Services Act compliance → declare trader status (blocks EU distribution until done) |
| 2 | Submit for review (final click) | ASC → CyberOS → select the processed build → answer export compliance (exempt - HTTPS only) → Submit |

## 2. Android / Google Play

| # | Step | How |
|---|---|---|
| 1 | Promote internal → production when ready | Play Console → Promote release |

## 3. Mac App Store (channel built; inert behind `MAS_RELEASE`)

| # | Step | How |
|---|---|---|
| 1 | Verify the updater exclusion compiles (needs your toolchain) | On your Mac: `cd apps/desktop/src-tauri && cargo check && cargo check --features mas` |
| 2 | Add macOS platform to the ASC app record (bundle-id decision: reuse `os.cyberskill.world.desktop` or mint one) | ASC → App → add platform |
| 3 | Issue `3rd Party Mac Developer Application` + `Installer` certs; set the 6 `MAS_*` secrets + 3 `ASC_*` secrets | developer.apple.com → Certificates; GitHub → Settings → Secrets |
| 4 | Confirm the `pending-human` rows in `docs/deploy/mac-app-store-submission.md` | 10-field answer sheet, ~5 min read |
| 5 | Set `MAS_RELEASE=true`, dispatch `release-mas.yml`, submit the processed build in ASC | Actions tab → Run workflow |

## 4. Microsoft Store (channel built; inert behind `MSSTORE_RELEASE`)

| # | Step | How |
|---|---|---|
| 1 | Reserve "CyberOS" in Partner Center, then send me the Identity Name/Publisher values so I can swap the CHANGEME placeholders in `AppxManifest.xml` | Partner Center → Apps → New product |
| 2 | Decide: keep store-managed signing (default, zero cert cost) or go self-managed for Intune sideload later | decision only |
| 3 | Either Azure AD app registration + 4 `MSSTORE_*` secrets (CI submission), or skip and upload the CI-built MSIX manually in Partner Center | portal.azure.com |
| 4 | `MSSTORE_RELEASE=true`, dispatch `release-msstore.yml`; complete the IARC age questionnaire; submit | Partner Center |

## 5. Snap Store (channel built; inert behind `SNAP_RELEASE`)

| # | Step | How |
|---|---|---|
| 1 | Ubuntu One account + `snapcraft register cyberos` (free) | snapcraft.io |
| 2 | `snapcraft export-login -` → paste output into secret `SNAPCRAFT_STORE_CREDENTIALS` | terminal + GitHub secrets |
| 3 | `SNAP_RELEASE=true` → dispatch `release-snap.yml` (first run proves the two in-recipe caveats; I fix anything that fails loud) | Actions tab |
| 4 | Smoke test before promoting past `stable`: `snap install --dangerous cyberos_*.snap` (webview renders, network, Wayland+X11) | your Linux box/VM |

## 6. Flathub (manifest ready; PR is per-instance gated)

| # | Step | How |
|---|---|---|
| 1 | Decide the app-id / confirm `cyberskill.world` ownership per Flathub's current docs | flathub.org docs |
| 2 | Your explicit go opens the PR to the Flathub repo (never automated) | say the word once §0 is done and I've drafted the manifest |

## 7. Homebrew Cask + winget (manifests ready; prep jobs inert)

| # | Step | How |
|---|---|---|
| 1 | After the §0 re-tag: `PKGMGR_CASK_RELEASE=true` + `PKGMGR_WINGET_RELEASE=true`, dispatch `release-pkgmgr-pr.yml` → download the rendered drafts (real hashes) | Actions tab |
| 2 | Cask, on your Mac: `brew audit --cask` + `brew style --cask` + `brew install --cask ./cyberos.rb` + `brew uninstall --zap` test (verifies the zap paths) | terminal |
| 3 | winget, on a Windows box: `winget validate --manifest ...` + run the installer with `/S` to confirm the silent switch (AC #8) | terminal |
| 4 | Your explicit go per PR → submit to homebrew-cask / winget-pkgs (every version bump is also gated) | say the word |

## 8. MCP connector (transport ready)

| # | Step | How |
|---|---|---|
| 1 | Route `https://os.cyberskill.world/mcp` → `localhost:8799` on the VPS with proxy auth | checklist in `docs/deploy/mcp-connector.md` |

## Current blockers snapshot

- iOS: build 10706 in TestFlight processing. Remaining: DSA declaration (§1.1), submit (§1.2).
- Android: publishing pipeline proven live; versionCode collision fixed in-repo (FR-IMP-078) — needs §0 re-tag, then promote when ready (§2.1).
- MAS: §3.1 compile check + Apple account items.
- MS Store: §4.1 reservation.
- Snap: §5.1 registration.
- Flathub: §6.1 app-id decision.
- Cask/winget: §0 re-tag first, then local validation runs.
