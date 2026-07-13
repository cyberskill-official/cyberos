# CyberOS 1.0.0 go-live checklist

Living document (2026-07-13). Everything code-side is committed and machine-verified. This is the full list of remaining steps to ship every channel, with a **Who** column on every row: **Agent task** means Claude can drive it now on request (drafting, running CLI commands, filling forms, clicking through up to any irreversible Submit); **Stephen** means it needs a password, a credential, a real device/toolchain, a payment, or an explicit go/no-go call only Stephen can make.

## 0. One push unblocks everything

| # | Step | Who | How |
|---|---|---|---|
| 1 | Push pending local commits + re-cut the release | Stephen | `git push` then `git tag -f v1.0.0 && git push -f origin v1.0.0`. Release-As: 1.0.0 pin holds the version. |

The re-tag re-runs release.yml. Both mobile blockers are fixed in-repo: the 90717 icon defect (icon flattened, guard asserts alpha-free — proven: iOS lane went green and build 10706 is in TestFlight) and the Play versionCode collision (FR-IMP-078: release builds now stamp `max(BUILD_NUMBER, minutes-since-epoch)`, so re-tags always upload a strictly higher number to both stores).

## 1. Web + desktop

Nothing manual. `deploy.yml` ships both on every push to `main`; FR-IMP-081 (testing, gate 2 pending) closes the last drift gap by rebuilding `apps/console/web` in CI on real source changes.

## 2. iOS App Store

| # | Step | Who | How |
|---|---|---|---|
| 1 | EU DSA trader declaration | Stephen | App Store Connect → Business → Digital Services Act compliance → declare trader status (blocks EU distribution until done) |
| 2 | Add TestFlight testers | Agent task | Say the word — I can add the 14 external testers via the App Store Connect web UI |
| 3 | Submit for review | Agent task up to the Submit click, then Stephen | ASC → CyberOS → select the processed build → answer export compliance (exempt — HTTPS only) → I prep everything through the final Submit button; Stephen clicks Submit himself per the irreversible-action rule |

## 3. Android / Google Play

| # | Step | Who | How |
|---|---|---|---|
| 1 | Confirm publishing pipeline is green | Done | Verified live; versionCode collision fixed in-repo (FR-IMP-078) |
| 2 | Add internal/closed testers | Agent task | Say the word — I can add tester email addresses in Play Console |
| 3 | Promote internal → production when ready | Stephen | Play Console → Promote release (go/no-go call) |

## 4. Mac App Store (channel built; inert behind `MAS_RELEASE`)

| # | Step | Who | How |
|---|---|---|---|
| 1 | Verify the updater exclusion compiles (needs your toolchain) | Stephen | On your Mac: `cd apps/desktop/src-tauri && cargo check && cargo check --features mas` |
| 2 | Add macOS platform to the ASC app record (bundle-id decision: reuse `os.cyberskill.world.desktop` or mint one) | Stephen | ASC → App → add platform |
| 3 | Issue `3rd Party Mac Developer Application` + `Installer` certs; set the 6 `MAS_*` secrets + 3 `ASC_*` secrets | Stephen | developer.apple.com → Certificates; GitHub → Settings → Secrets (credentials — never handled by me) |
| 4 | Confirm the `pending-human` rows in `docs/deploy/mac-app-store-submission.md` | Stephen | 10-field answer sheet, ~5 min read |
| 5 | Set `MAS_RELEASE=true`, dispatch `release-mas.yml`, submit the processed build in ASC | Stephen | Actions tab → Run workflow (release dispatch — Stephen only) |

## 5. Microsoft Store (channel built; inert behind `MSSTORE_RELEASE`)

| # | Step | Who | How |
|---|---|---|---|
| 1 | Reserve "CyberOS" in Partner Center, then send me the Identity Name/Publisher values so I can swap the CHANGEME placeholders in `AppxManifest.xml` | Stephen reserves; Agent task swaps the placeholders once you send the values | Partner Center → Apps → New product |
| 2 | Decide: keep store-managed signing (default, zero cert cost) or go self-managed for Intune sideload later | Stephen | decision only |
| 3 | Either Azure AD app registration + 4 `MSSTORE_*` secrets (CI submission), or skip and upload the CI-built MSIX manually in Partner Center | Stephen | portal.azure.com (credentials) |
| 4 | Draft the store listing copy (description, screenshots list, age rating prep) | Agent task | Say the word — I can draft the listing text for your review |
| 5 | `MSSTORE_RELEASE=true`, dispatch `release-msstore.yml`; complete the IARC age questionnaire; submit | Stephen | Partner Center (release dispatch + submission — Stephen only) |

## 6. Snap Store (channel built; inert behind `SNAP_RELEASE`)

| # | Step | Who | How |
|---|---|---|---|
| 1 | Ubuntu One account + `snapcraft register cyberos` (free) | Stephen | snapcraft.io (account creation — Stephen only) |
| 2 | `snapcraft export-login -` → paste output into secret `SNAPCRAFT_STORE_CREDENTIALS` | Stephen | terminal + GitHub secrets (credentials) |
| 3 | `SNAP_RELEASE=true` → dispatch `release-snap.yml` (first run proves the two in-recipe caveats; I fix anything that fails loud) | Stephen dispatches; Agent task fixes any failures | Actions tab |
| 4 | Smoke test before promoting past `stable`: `snap install --dangerous cyberos_*.snap` (webview renders, network, Wayland+X11) | Stephen | your Linux box/VM |

## 7. Flathub (manifest ready; PR is per-instance gated)

| # | Step | Who | How |
|---|---|---|---|
| 1 | Decide the app-id / confirm `cyberskill.world` ownership per Flathub's current docs | Stephen | flathub.org docs (decision only) |
| 2 | Draft/refresh the Flathub manifest | Agent task | I keep the manifest current on request |
| 3 | Open the PR to the Flathub repo | Stephen approval required, then Agent task opens it | Say the word once §0 is done and the manifest is drafted — I never open an external PR without a fresh go each time |

## 8. Homebrew Cask + winget (manifests ready; prep jobs inert)

| # | Step | Who | How |
|---|---|---|---|
| 1 | After the §0 re-tag: `PKGMGR_CASK_RELEASE=true` + `PKGMGR_WINGET_RELEASE=true`, dispatch `release-pkgmgr-pr.yml` → download the rendered drafts (real hashes) | Stephen | Actions tab (release dispatch — Stephen only) |
| 2 | Cask, on your Mac: `brew audit --cask` + `brew style --cask` + `brew install --cask ./cyberos.rb` + `brew uninstall --zap` test (verifies the zap paths) | Stephen | terminal (needs your Mac) |
| 3 | winget, on a Windows box: `winget validate --manifest ...` + run the installer with `/S` to confirm the silent switch (AC #8) | Stephen | terminal (needs a Windows box) |
| 4 | Submit to homebrew-cask / winget-pkgs | Stephen approval required each time, then Agent task submits | Say the word per PR — every version bump is gated the same way |

## 9. MCP connector (transport ready)

| # | Step | Who | How |
|---|---|---|---|
| 1 | Route `https://os.cyberskill.world/mcp` → `localhost:8799` on the VPS with proxy auth | Stephen | checklist in `docs/deploy/mcp-connector.md` (VPS/server access — Stephen only) |
| 2 | Register the connector once the route is live | Agent task | Say the word once §9.1 is done — I can drive the connector registration flow |

## Current blockers snapshot

- iOS: build 10706 in TestFlight processing. Remaining: DSA declaration (§2.1, Stephen), testers (§2.2, agent task on request), submit (§2.3, agent-prepped / Stephen-clicked).
- Android: publishing pipeline proven live; versionCode collision fixed in-repo (FR-IMP-078) — needs §0 re-tag, testers (§3.2, agent task on request), then promote when ready (§3.3, Stephen).
- MAS: §4.1 compile check + Apple account items (all Stephen).
- MS Store: §5.1 reservation (Stephen); listing copy (§5.4) available as an agent task once reserved.
- Snap: §6.1 registration (Stephen).
- Flathub: §7.1 app-id decision (Stephen); manifest refresh (§7.2) is an agent task any time.
- Cask/winget: §0 re-tag first, then local validation runs (both Stephen — need real Mac/Windows hardware).
- MCP connector: §9.1 VPS routing (Stephen); §9.2 registration is an agent task once routed.
