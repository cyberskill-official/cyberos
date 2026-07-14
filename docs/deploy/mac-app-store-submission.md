# Mac App Store submission — answer sheet + sandbox audit (TASK-APP-003)

Companion to `docs/deploy/RELEASE.md` (Developer ID channel) and `docs/deploy/play-store-submission.md` (Android pattern this file mirrors). The Mac App Store build is a second, independent Tauri target: sandboxed, signed with the `3rd Party Mac Developer *` certificate pair, packaged as `.pkg`, submitted via `release-mas.yml` behind the repo variable `MAS_RELEASE=true` (off by default — the workflow is inert until every "hard blocker" row below is resolved).

## Sandbox surface audit

Every `#[tauri::command]` handler and every direct filesystem / network / process call in `apps/desktop/src-tauri/src/` (audited 2026-07-13 against the full 310-line backend: `lib.rs`, `gateway_client.rs`, `mcp_client.rs`, `keychain.rs`, `main.rs`). Dispositions: `sandbox-native` (works inside the container with no extra entitlement), `entitlement:<key>` (needs the named entitlement, declared in `Entitlements.mas.plist`), or `gated-behind-MAS-feature-flag`.

| Symbol | File:line | Capability used | Disposition |
|---|---|---|---|
| (whole MAS target) | apps/desktop/src-tauri/Entitlements.mas.plist | App Sandbox master switch — mandatory for every Mac App Store submission (spec §1 #2); not tied to one symbol | entitlement:com.apple.security.app-sandbox |
| `health` | src/lib.rs:17 | outbound HTTPS GET `/healthz` (reqwest/rustls) | entitlement:com.apple.security.network.client |
| `chat` | src/lib.rs:23 | outbound HTTPS POST `/v1/chat` + keychain read | entitlement:com.apple.security.network.client (network); sandbox-native (keychain) |
| `save_token` | src/lib.rs:36 | macOS Keychain write via `keyring` crate | sandbox-native (app's own keychain items are sandbox-permitted; no entitlement needed for self-owned items) |
| `clear_token` | src/lib.rs:41 | macOS Keychain delete via `keyring` | sandbox-native (same basis as `save_token`) |
| `has_token` | src/lib.rs:46 | macOS Keychain read via `keyring` | sandbox-native (same basis as `save_token`) |
| `mcp_health` | src/lib.rs:52 | outbound HTTPS GET `/mcp/healthz` (reqwest) | entitlement:com.apple.security.network.client |
| `list_tools` | src/lib.rs:58 | outbound HTTPS POST MCP `tools/list` (reqwest) | entitlement:com.apple.security.network.client |
| `call_tool` | src/lib.rs:66 | outbound HTTPS POST MCP `tools/call` (reqwest) | entitlement:com.apple.security.network.client |
| `spawn_update_check` (not an IPC command; launch task) | src/lib.rs:76 | tauri-plugin-updater: downloads a signed bundle, **installs over the app, restarts** | gated-behind-MAS-feature-flag — self-update is both a sandbox violation (writes to the app bundle) and an App Store policy violation (App Store owns updates). See "Updater finding" below. |
| WKWebView content loads | tauri.conf.json `frontendDist` + remote gateway calls from the webview | in-process web content; network via the same client entitlement | sandbox-native + entitlement:com.apple.security.network.client |
| Child processes / sidecars | — (none: no `Command::new`, no `std::process`, no `externalBin`, no shell plugin anywhere in src/) | — | none required — `Entitlements.mas.inherit.plist` ships defensive-only (spec §9 open question resolved: inherit plist is NOT load-bearing today) |
| Filesystem access | — (none: no fs plugin, no dialog plugin, no `std::fs` call in src/) | — | none — which is why the spec §3 sketch's `com.apple.security.files.user-selected.read-write` is deliberately absent from the final plist (AC #2 minimality; an unused entitlement would fail Apple review scrutiny and our own lint) |

`tools/mas-entitlement-lint.sh` enforces this table ↔ plist correspondence: any plist key without a row above fails the build.

### Updater finding (the one gated capability)

`tauri-plugin-updater` is registered unconditionally on desktop (`src/lib.rs:100-103`) and checks on every launch. For the MAS target this MUST be excluded at compile time (a `mas` cargo feature that skips the plugin registration + the launch check), because:

1. **Sandbox:** `download_and_install` writes into the installed `.app` bundle — outside the sandbox container, no entitlement exists for it.
2. **Policy:** Mac App Store apps must receive updates through the App Store; a self-updater is grounds for rejection regardless of sandbox mechanics.

That compile-time exclusion is a small Rust change and is **out of scope for TASK-APP-003 by its own §11 boundary** ("any capability found to be sandbox-incompatible is a follow-up FR"). Consequence, recorded honestly: **`MAS_RELEASE=true` must not be flipped until the follow-up FR lands.** It is listed as a hard blocker below alongside Stephen's account prerequisites. Today the updater is a *quiet no-op only when* `plugins.updater` config is absent — but the base `tauri.conf.json` DOES configure it (pubkey + GitHub endpoint), so the MAS bundle would actively self-update without the exclusion. Do not rationalize this away with a config-overlay trick; unverified deep-merge null-out semantics are not a submission-safety mechanism.

## Hard blockers on `MAS_RELEASE=true` (in dependency order)

| # | Blocker | Owner | Status |
|---|---|---|---|
| 1 | `mas` cargo feature excluding the self-updater from the MAS build | engineering | **resolved by TASK-IMP-075** (registration + launch check cfg-gated out; release-mas.yml builds `--features mas`). Verify on first real toolchain run: `cargo check && cargo check --features mas`. Residual, non-blocking: the target-scoped dependency still compiles in as dead code — optional-dep shrink is a later follow-up only if App Review ever flags it |
| 2 | Apple Developer Program enrollment active | Stephen | pending-human |
| 3 | macOS platform added to the CyberOS App Store Connect app record (bundle-id decision: reuse `os.cyberskill.world.desktop` or mint a MAS-specific id — ASC account decision) | Stephen | pending-human |
| 4 | `3rd Party Mac Developer Application` + `3rd Party Mac Developer Installer` certs issued; secrets `MAS_APP_CERT_P12_BASE64`, `MAS_INSTALLER_CERT_P12_BASE64`, `MAS_CERT_PASSWORD`, `MAS_KEYCHAIN_PASSWORD`, `MAS_APP_SIGNING_IDENTITY`, `MAS_INSTALLER_SIGNING_IDENTITY` set | Stephen | pending-human |
| 5 | App Store Connect API key secrets `ASC_API_KEY`, `ASC_KEY_ID`, `ASC_ISSUER_ID` set | Stephen | pending-human |

## App Store Connect submission answer sheet (macOS)

Every ASC macOS submission field needing a human decision. `status` is `human-confirmed`, `pending-human` (recommended answer drafted, Stephen confirms), or `not-applicable` (+ reason).

| Field | Recommended answer | Status |
|---|---|---|
| Export compliance — uses encryption? | **Yes, exempt** — HTTPS/TLS only (rustls for gateway calls, WKWebView TLS); no proprietary crypto. Same declaration already made for the iOS app record | pending-human |
| Export compliance — France declaration | Not required for exempt-only encryption | pending-human |
| Content rights — third-party content | App displays the user's own CyberOS workspace content; no licensed third-party media shipped | pending-human |
| Age rating questionnaire | Same answers as the existing iOS CyberOS record (no objectionable content categories; unrestricted web access = No, the webview loads only the CyberOS console) — expect 4+ | pending-human |
| Privacy nutrition label | Mirrors iOS record: account identifiers (auth token) stored locally in Keychain; chat content transits to the user's own gateway; no tracking, no third-party analytics in the desktop shell | pending-human |
| macOS privacy usage strings (`NSCameraUsageDescription`, `NSMicrophoneUsageDescription`, …) | **not-applicable** — the AC #1 audit found no camera/mic/location/contacts API use and the entitlement set requests none; no usage string is required when no protected resource is touched | not-applicable (audit-grounded) |
| App Sandbox declaration in ASC | Automatic from the binary's entitlements (`app-sandbox=true`); nothing to fill manually | not-applicable (informational) |
| Pricing | Free (matches iOS record + product decision from the 2026-07 store push) | pending-human |
| Category | Productivity (primary) / Developer Tools (secondary) — matches the iOS listing | pending-human |
| Screenshots | macOS 2880×1800 or 2560×1600 set, reusing the desktop marketing captures; **do not fabricate** — capture from a real build | pending-human |

## Operational notes

- Trigger: `release-mas.yml` is `workflow_dispatch` (manual, Stephen-run) — deliberately NOT tag-coupled while the channel is being stood up, so a normal `vX.Y.Z` release can never accidentally attempt an ASC submission.
- The Developer ID channel (RELEASE.md) is unchanged and remains the primary macOS distribution; MAS is additive discoverability (TASK-APP-003 `risk_if_skipped`).
- Version comes from the base `tauri.conf.json` (stamped by `scripts/stamp-release-version.mjs`); `tauri.mas.conf.json` deliberately declares **no** `version`/`productName` keys so the overlay can never drift from the stamper (spec §10).
- AC #5 (`codesign --verify`, `spctl -a -t install`) can only run on macOS with real certs — recorded as expected-pending in the FR's review packet; first verified run happens on Stephen's Mac or the first `MAS_RELEASE=true` CI run after blockers clear.
