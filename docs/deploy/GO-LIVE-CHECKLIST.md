# CyberOS 1.0.0 go-live checklist

Living document (2026-07-13). Everything code-side is committed and machine-verified. This is the full list of remaining steps to ship every channel, with a **Who** column on every row: **Agent task** means Claude can drive it now on request (drafting, running CLI commands, filling forms, clicking through up to any irreversible Submit); **Stephen** means it needs a password, a credential, a real device/toolchain, a payment, or an explicit go/no-go call only Stephen can make.

## 0. One push unblocks everything — Done

| # | Step | Who | How |
|---|---|---|---|
| 1 | Push pending local commits + re-cut the release | Done | Pushed and re-tagged `v1.0.0` → commit `2493164`. |

The re-tag re-ran release.yml (run #59) and every job went green: `payload`, `android` (AAB built, published to the Play internal track), `ios` (TestFlight upload), `docs`, and the full `desktop` matrix (macOS/Windows/Ubuntu). Both mobile blockers held clean on re-run: the 90717 icon defect (icon flattened, guard asserts alpha-free) and the Play versionCode collision (FR-IMP-078: release builds stamp `max(BUILD_NUMBER, minutes-since-epoch)`).

Note: an earlier manual run (#58, same tag) failed mid-`android` with "This edit has expired, please create a new Edit." — traced to a Play Edits-API collision (tester-list edits open in Play Console at the same time as that run's publish step). Not a code defect; run #59 confirms it. Also found and fixed in passing, unrelated to the failure: `r0adkll/upload-google-play`'s `track:` input is deprecated in favor of `tracks:` — renamed in local commit `461cc87` (not yet pushed; cosmetic, no rush).

## 1. Web + desktop

Nothing manual. `deploy.yml` ships both on every push to `main`; FR-IMP-081 (testing, gate 2 pending) closes the last drift gap by rebuilding `apps/console/web` in CI on real source changes.

## 2. iOS App Store

| # | Step | Who | How |
|---|---|---|---|
| 1 | EU DSA trader declaration | Done | Verified live in ASC → App Information: "This developer has identified itself as a trader for this app." Already complete — this row was stale, it is not a pending blocker |
| 2 | Add TestFlight testers | Done | Added 10 testers to the ASC "CyberOS External Testers" group. Build 10706 isn't assigned to that group yet — say the word if you want that done too |
| 3 | Submit for review | Agent task up to the Submit click, then Stephen | App Privacy: configured and Published for all 7 data types (Name, Email, Emails/Text Messages, Photos/Videos, Other User Content, User ID, Device ID — all App Functionality only, linked, not tracking). Export compliance: confirmed on build 29732976 — "App Uses Non-Exempt Encryption: No" (exempt, HTTPS only), already answered correctly, no action needed. Build 29732976 (1.0.0) is already attached to the 1.0 version. Remaining gaps before Stephen can click Add for Review: (a) App Review Information → Sign-In Information username/password are empty — needs a real Apple-ID demo account provisioned into a demo workspace, same pattern as `play-review@cyberskill.world` for Play; I will not fabricate credentials. (b) Screenshots/App Previews are still 0 of 10 / 0 of 3 — need real device captures, not fabricated. Once both are supplied, I can finish prepping the version page; Stephen still clicks Add for Review himself |

## 3. Android / Google Play

| # | Step | Who | How |
|---|---|---|---|
| 1 | Confirm publishing pipeline is green | Done | Verified live end-to-end — run #59's `android` job actually published to the internal track (not just a dry build); versionCode collision fixed in-repo (FR-IMP-078) |
| 2 | Add internal/closed testers | Done | Added 10 testers to Play Console's "CyberOS Testers" list; synced the pre-existing "Test" list to the same 10 for consistency |
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
| 4 | Draft the store listing copy (description, screenshots list, age rating prep) | Done | Drafted in `docs/deploy/microsoft-store-listing-copy.md` — description, short description, category, keywords, age-rating guidance, privacy URL, support contact. Real screenshots still needed (not fabricated) |
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
| 2 | Draft/refresh the Flathub manifest | Done (ongoing) | Confirmed current this pass — I keep it in sync on request whenever the app changes |
| 3 | Open the PR to the Flathub repo | Stephen approval required, then Agent task opens it | Say the word once §0 is done and the manifest is drafted — I never open an external PR without a fresh go each time |

## 8. Homebrew Cask + winget (manifests ready; prep jobs inert)

| # | Step | Who | How |
|---|---|---|---|
| 1 | §0 is done, so this is unblocked: `PKGMGR_CASK_RELEASE=true` + `PKGMGR_WINGET_RELEASE=true`, dispatch `release-pkgmgr-pr.yml` → download the rendered drafts (real hashes) | Stephen | Actions tab (release dispatch — Stephen only) |
| 2 | Cask, on your Mac: `brew audit --cask` + `brew style --cask` + `brew install --cask ./cyberos.rb` + `brew uninstall --zap` test (verifies the zap paths) | Stephen | terminal (needs your Mac) |
| 3 | winget, on a Windows box: `winget validate --manifest ...` + run the installer with `/S` to confirm the silent switch (AC #8) | Stephen | terminal (needs a Windows box) |
| 4 | Submit to homebrew-cask / winget-pkgs | Stephen approval required each time, then Agent task submits | Say the word per PR — every version bump is gated the same way |

## 9. MCP connector (transport ready)

| # | Step | Who | How |
|---|---|---|---|
| 1 | Route `https://os.cyberskill.world/mcp` → `localhost:8799` on the VPS with proxy auth | Stephen | checklist in `docs/deploy/mcp-connector.md` (VPS/server access — Stephen only) |
| 2 | Register the connector once the route is live | Agent task | Say the word once §9.1 is done — I can drive the connector registration flow |

## Current blockers snapshot

- iOS: build 29732976 (1.0.0) processed/validated and attached to the 1.0 version; 10 testers added to the ASC group (build not yet assigned to it — say the word). DSA declaration (§2.1) already done, checklist corrected. App Privacy configured + published; export compliance confirmed exempt on the build. Remaining before submit (§2.3): Stephen needs to supply real App Review sign-in credentials (demo account) and real device screenshots/previews — then Stephen clicks Add for Review.
- Android: §0 done — v1.0.0 re-tagged, `android` job published live on run #59; testers done (§3.2, 10 added). Remaining: promote internal → production when ready (§3.3, Stephen).
- MAS: §4.1 compile check + Apple account items (all Stephen).
- MS Store: §5.1 reservation (Stephen, blocks the rest of §5); listing copy (§5.4) already drafted in `docs/deploy/microsoft-store-listing-copy.md`.
- Snap: §6.1 registration (Stephen).
- Flathub: §7.1 app-id decision (Stephen); manifest (§7.2) confirmed current.
- Cask/winget: §0 is done, so the precondition for §8.1 is met — dispatching `release-pkgmgr-pr.yml` and the local Mac/Windows validation runs are all that's left (both Stephen — need real hardware).
- MCP connector: §9.1 VPS routing (Stephen); §9.2 registration is an agent task once routed.
- Housekeeping: unpushed local commit `461cc87` renames the deprecated `track:` → `tracks:` input on the Play publish step (CI hygiene, not required before promoting) — push whenever convenient.
