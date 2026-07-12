# CyberOS 1.0.0 go-live checklist — every platform, remaining manual steps

Living document (2026-07-13). Everything code-side is committed and machine-verified; what remains is listed per platform, in execution order, with the exact command or click path. Steps marked **[agent-assistable]** can be driven by Claude via the Chrome extension on request (per-action approval each time); steps marked **[Stephen-only]** involve credentials, payments, account settings, or pushes, which stay yours.

## 0. One push unblocks everything

```
git push                                   # 25 local commits; Release-As: 1.0.0 pin holds the version
git tag -f v1.0.0 && git push -f origin v1.0.0    # re-cut the release at the icon-fixed commit
```
**[Stephen-only]** The re-tag re-runs release.yml: desktop rebuild, android, and the iOS lane that failed - the 90717 icon defect is fixed in-repo (icon flattened, guard now asserts alpha-free).

## 1. Web + desktop (GitHub Releases) — already live

Nothing manual. The v1.0.0 re-tag refreshes installers; deploy.yml keeps the site current.

## 2. iOS App Store

| # | Step | How | Who |
|---|---|---|---|
| 1 | Re-run the iOS lane | The §0 re-tag does it; expect TestFlight processing email in ~15 min | Stephen (push) |
| 2 | EU DSA trader declaration | App Store Connect → Business → Digital Services Act compliance → declare trader status (blocks EU distribution until done) | **[Stephen-only]** |
| 3 | Add the 14 testers | ASC → Users and Access → add; then TestFlight → Internal Testing group → add testers | **[agent-assistable]** (Chrome, with your approval per invite) |
| 4 | Submit for review | ASC → CyberOS → select the processed build → answer export compliance (exempt - HTTPS only) → Submit | **[agent-assistable]** up to the Submit click, which is yours |

## 3. Android / Google Play

| # | Step | How | Who |
|---|---|---|---|
| 1 | Confirm the android lane is green on the re-tag; `.aab` appears as a run artifact | Actions tab | either |
| 2 | `PLAY_PUBLISH=true` repo variable (internal track auto-publish) or manual upload of the `.aab` to Play Console → Internal testing | Settings → Actions → Variables | **[Stephen-only]** (repo settings) |
| 3 | Add tester emails to the internal track list | Play Console → Testing → Internal testing → Testers | **[agent-assistable]** |
| 4 | Promote internal → production when ready | Play Console → Promote release | **[Stephen-only]** |

## 4. Mac App Store (channel built; inert behind `MAS_RELEASE`)

| # | Step | How | Who |
|---|---|---|---|
| 1 | Verify the updater exclusion compiles | On your Mac: `cd apps/desktop/src-tauri && cargo check && cargo check --features mas` | **[Stephen-only]** (needs your toolchain) |
| 2 | Add macOS platform to the ASC app record (bundle-id decision: reuse `os.cyberskill.world.desktop` or mint one) | ASC → App → add platform | **[Stephen-only]** |
| 3 | Issue `3rd Party Mac Developer Application` + `Installer` certs; set the 6 `MAS_*` secrets + 3 `ASC_*` secrets | developer.apple.com → Certificates; GitHub → Settings → Secrets | **[Stephen-only]** (credential entry) |
| 4 | Confirm the `pending-human` rows in `docs/deploy/mac-app-store-submission.md` | 10-field answer sheet | Stephen (5 min read) |
| 5 | Set `MAS_RELEASE=true`, dispatch `release-mas.yml`, submit the processed build in ASC | Actions tab → Run workflow | **[Stephen-only]** |

## 5. Microsoft Store (channel built; inert behind `MSSTORE_RELEASE`)

| # | Step | How | Who |
|---|---|---|---|
| 1 | Reserve "CyberOS" in Partner Center → paste me the Identity Name/Publisher values and I swap the CHANGEME placeholders in `AppxManifest.xml` | Partner Center → Apps → New product | **[Stephen-only]** reserve; agent swaps values |
| 2 | Keep store-managed signing (default - zero cert cost); or decide self-managed for Intune sideload later | none needed | decision only |
| 3 | Either Azure AD app registration + 4 `MSSTORE_*` secrets (CI submission), or skip and upload the CI-built MSIX manually in Partner Center | portal.azure.com | **[Stephen-only]** |
| 4 | `MSSTORE_RELEASE=true`, dispatch `release-msstore.yml`; IARC age questionnaire + listing; submit | Partner Center | **[Stephen-only]** submit; **[agent-assistable]** listing copy |

## 6. Snap Store (channel built; inert behind `SNAP_RELEASE`)

| # | Step | How | Who |
|---|---|---|---|
| 1 | Ubuntu One account + `snapcraft register cyberos` (free) | snapcraft.io | **[Stephen-only]** |
| 2 | `snapcraft export-login -` → paste output into secret `SNAPCRAFT_STORE_CREDENTIALS` | terminal + GitHub secrets | **[Stephen-only]** |
| 3 | `SNAP_RELEASE=true` → dispatch `release-snap.yml` (first run proves the two in-recipe caveats; fails loud if the deb layout differs - I fix) | Actions tab | **[Stephen-only]** flip; agent fixes fallout |
| 4 | Smoke test: `snap install --dangerous cyberos_*.snap` (webview renders, network, Wayland+X11) before promoting past `stable` listing metadata | your Linux box/VM | **[Stephen-only]** |

## 7. Flathub (manifest ready; PR is per-instance gated)

1. Decide the app-id / confirm `cyberskill.world` ownership per Flathub's current docs **[Stephen-only]**.
2. I then author the `.desktop` + AppStream metainfo + final source pinning, and validate with `flatpak-builder` (agent).
3. Your explicit go opens the PR to the Flathub repo (never automated).

## 8. Homebrew Cask + winget (manifests ready; prep jobs inert)

1. After the §0 re-tag: `PKGMGR_CASK_RELEASE=true` + `PKGMGR_WINGET_RELEASE=true`, dispatch `release-pkgmgr-pr.yml` → download the rendered drafts (real hashes) **[Stephen-only]** flips.
2. Cask, on your Mac: `brew audit --cask` + `brew style --cask` + `brew install --cask ./cyberos.rb` + `brew uninstall --zap` test (verifies the zap paths) **[Stephen-only]**.
3. winget, on a Windows box: `winget validate --manifest ...` + run the installer with `/S` to confirm the silent switch (AC #8) **[Stephen-only]**.
4. Your explicit go per PR → submit to homebrew-cask / winget-pkgs (every version bump is also gated).

## 9. MCP connector (transport ready)

1. Route `https://os.cyberskill.world/mcp` → `localhost:8799` on the VPS with proxy auth (checklist in `docs/deploy/mcp-connector.md`) **[Stephen-only]**.
2. Add the connector in Claude (Settings → Connectors) and Grok (Skills and Connectors → New Connector) - name + URL; verify tools/list shows the 4 workflow tools **[agent-assistable]** in Chrome.

## Current blockers snapshot

- iOS: fixed in-repo (this commit); needs §0 re-tag only.
- Android: needs §3.2 variable or manual upload.
- MAS: §4.1 compile check + Apple account items.
- MS Store: §5.1 reservation.
- Snap: §6.1 registration.
- Flathub: §7.1 app-id decision.
- Cask/winget: §0 re-tag first, then local validation runs.
