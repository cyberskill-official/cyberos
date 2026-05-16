---
id: FR-BRAIN-104
title: "Tauri 2.x desktop app — macOS + Windows + Linux signed/notarised + auto-update + tray + quick capture + Full Disk Access"
module: BRAIN
priority: SHOULD
status: accepted
verify: D
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CDO)
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-BRAIN-103, FR-BRAIN-105, FR-BRAIN-108]
depends_on: [FR-BRAIN-103]
blocks: [FR-BRAIN-105]   # placeholder — disputed-pair UI FR, not yet specified

source_pages:
  - website/docs/modules/brain.html#desktop-app
source_decisions:
  - DEC-190 (Tauri 2.x over Electron; smaller bundle, native performance, Rust-native)
  - DEC-191 (signed releases on Cloudflare R2; no third-party signing service)
  - DEC-192 (Full Disk Access required on macOS for ~/.cyberos-memory/ access)
  - DEC-193 (auto-update via Tauri updater; signed manifests; rollback on signature failure)

language: rust 1.81 + svelte 5 + tailwind
service: cyberos/apps/brain/
new_files:
  - apps/brain/src-tauri/Cargo.toml
  - apps/brain/src-tauri/tauri.conf.json
  - apps/brain/src-tauri/src/main.rs
  - apps/brain/src-tauri/src/commands.rs
  - apps/brain/src-tauri/src/sync_supervisor.rs
  - apps/brain/src-tauri/src/tray.rs
  - apps/brain/src-tauri/src/updater.rs
  - apps/brain/src-tauri/src/permissions.rs
  - apps/brain/src/App.svelte
  - apps/brain/src/routes/+page.svelte
  - apps/brain/src/routes/search/+page.svelte
  - apps/brain/src/routes/sync/+page.svelte
  - apps/brain/src/lib/quick_capture.ts
  - apps/brain/src-tauri/build.rs
  - apps/brain/src-tauri/icons/
  - apps/brain/.github/workflows/release.yml
  - apps/brain/scripts/sign-and-notarize-macos.sh
  - apps/brain/scripts/sign-windows.sh
modified_files: []
allowed_tools:
  - file_read: apps/brain/**
  - file_write: apps/brain/**
  - bash: cd apps/brain && cargo tauri dev
  - bash: cd apps/brain && cargo tauri build
  - bash: codesign --verify Brain.app   # macOS
disallowed_tools:
  - ship unsigned releases (per §1 #2 — must be signed + notarised)
  - bypass Full Disk Access prompt on macOS (per §1 #7)
  - request more permissions than necessary (least-privilege per §1 #8)
  - update without signature verification (per §1 #2 — rollback on signature fail)

effort_hours: 28
sub_tasks:
  - "0.5h: Tauri 2.x scaffold + tauri.conf.json"
  - "1.0h: Svelte 5 + Tailwind UI scaffold"
  - "1.0h: src/main.rs entry + sync_supervisor (embeds FR-BRAIN-103 daemon as tokio task)"
  - "1.0h: tray.rs — system tray with quick actions (open, force sync, recent memories)"
  - "1.0h: updater.rs — Tauri updater integration; signed manifest verification"
  - "1.0h: permissions.rs — Full Disk Access prompt (macOS) + AppContainer (Windows)"
  - "1.0h: commands.rs — Tauri commands (search_brain, write_quick_note, get_sync_state)"
  - "1.0h: routes/+page.svelte — main dashboard (chain head + last sync + disputed count)"
  - "1.0h: routes/search/+page.svelte — local search UI (FR-BRAIN-108 API)"
  - "1.0h: routes/sync/+page.svelte — sync state + manual force-sync"
  - "1.0h: quick_capture.ts — textbox + auto-tag + write quick_note row"
  - "0.5h: macOS code-signing (Developer ID Application) + notarisation"
  - "0.5h: Windows code-signing (EV cert) + AppContainer manifest"
  - "0.5h: Linux .deb + AppImage packaging"
  - "0.5h: Cloudflare R2 release storage + signed update manifest"
  - "1.0h: .github/workflows/release.yml — multi-platform CI build"
  - "1.0h: Auto-update test (publish v1.1; v1.0 client picks it up within 24h)"
  - "1.0h: macOS entitlements + sandbox profile"
  - "1.0h: Windows AppContainer + capabilities"
  - "1.5h: Tests — install + auto-update + sync daemon survives restart + permissions prompt + quick-capture"
  - "1.0h: Notarisation script + DMG packaging"
  - "1.0h: Bundle size optimisation (target < 30MB Mac, < 25MB Windows)"
  - "0.5h: Update UX (in-app banner; restart-to-apply prompt)"
  - "1.0h: First-run onboarding (Full Disk Access prompt, Cloud BRAIN setup)"
  - "1.0h: Crash reporting (sentry-rust integration; opt-in)"
  - "1.0h: Multi-window support (search in separate window)"
  - "0.5h: Localisation infrastructure (slice 3+ enables i18n for VN tenants)"
risk_if_skipped: "Power users can't run BRAIN as a first-class app. Sync daemon requires running `brain-sync` from terminal — operationally fragile. No tray = users forget BRAIN is syncing. No quick-capture = friction for ad-hoc notes. Without auto-update, security patches roll out manually (slow + error-prone). Without signing, OSes prevent installation OR show scary warnings."
---

## §1 — Description (BCP-14 normative)

A Tauri 2.x desktop app **MUST** bundle the BRAIN sync daemon + a minimal UI for inspecting + searching local memory. Each component:

1. **MUST** ship for macOS (universal Apple Silicon + Intel; signed + notarised), Windows (x64; EV-cert signed), Linux (.deb + AppImage). Each platform's release verified by OS-native code-signing checks.
2. **MUST** auto-update via Tauri's built-in updater. Update manifests signed with a release-signing Ed25519 key; client verifies signature before applying. Signature failure → rollback + sev-2 alert; user notified via in-app banner.
3. **MUST** run brain-sync (FR-BRAIN-103) as an internal Tauri-managed process via `tauri::async_runtime::spawn`. The supervisor monitors health; auto-restart on panic with exponential backoff.
4. **MUST** expose a system tray icon with quick actions:
    - Open BRAIN main window.
    - Force sync now.
    - Show recent memories (last 10 from local Layer 1).
    - Toggle sync (pause/resume).
    - Quit.
5. **MUST** show on the main dashboard:
    - Current chain head (hex; copyable).
    - Last sync time + duration.
    - Disputed pair count (badge; clicks to FR-BRAIN-105 resolution UI).
    - sync_class breakdown (count of shareable vs private rows).
    - Total memory count + storage used.
    - Cloud BRAIN connection state (online/offline).
6. **MUST** support local search via FR-BRAIN-108 API. Search box on dashboard; results pane with memory previews; click → open memory file.
7. **MUST** request macOS Full Disk Access at first run (one-time; persisted by macOS). Without FDA, the app cannot read `~/.cyberos-memory/` (System Integrity Protection blocks). The prompt directs user to System Settings > Privacy & Security > Full Disk Access > [Brain.app toggle].
8. **MUST** sandbox per OS:
    - macOS: hardened runtime + entitlements (no JIT, no debug, FDA only).
    - Windows: AppContainer with capabilities (file system access for `%USERPROFILE%/.cyberos-memory/`, network for Cloud BRAIN).
    - Linux: AppArmor profile (where supported).
9. **MUST** support quick-capture: tray-accessible textbox; user types → app writes a `quick_note` BRAIN row with auto-tag (date, source: tray, originator_device).
10. **MUST** sign update manifests with the release-signing Ed25519 key (separate from BRAIN signing key). Public key embedded in app binary at compile time; rotation requires app rebuild + re-release.
11. **MUST** persist user settings (Cloud BRAIN URL, sync interval, opt-in to crash reporting) in OS-standard config dir (`~/Library/Application Support/cyberos/brain/` macOS; `%APPDATA%/cyberos/brain/` Windows; `~/.config/cyberos/brain/` Linux).
12. **MUST** support headless mode (Linux daemon only; no UI) via `--headless` flag for server installs.
13. **MUST** target bundle size ≤ 30MB on macOS, ≤ 25MB on Windows. Tauri produces small binaries by design; size budget catches accidental dependency bloat.
14. **MUST** report crashes via opt-in sentry-rust integration. Default opt-OUT (privacy-respecting); user can enable in Settings.
15. **SHOULD** support multi-window (search in separate window; main dashboard always primary).
16. **SHOULD** support localisation infrastructure for slice-3+ Vietnamese UI (i18n keys + translation files).

---

## §2 — Why this design (rationale for humans)

**Why Tauri 2.x over Electron (DEC-190)?** Tauri produces 5-10× smaller binaries (Electron ~150MB; Tauri ~25MB). Native performance (Rust shell + native WebView vs Chromium). Rust-native means embedding BRAIN daemon directly without IPC overhead. Trade-off: WebView2/WKWebView is platform-specific (vs Chromium's uniformity); mitigated by Svelte's compatibility.

**Why signed + notarised on macOS (§1 #1)?** macOS Gatekeeper blocks unsigned apps with scary warnings. Notarisation registers the app with Apple's malware-scan service. Without these, users can't install easily; with them, install is one-click.

**Why EV cert on Windows (§1 #1)?** Standard code-signing certs trigger SmartScreen warnings until reputation builds (months of installs). EV certs skip SmartScreen entirely. Cost is higher ($300-500/year vs $50) but worth it for first-impression.

**Why auto-update via Tauri (DEC-193)?** Manual updates miss security patches. Tauri's updater + signed manifests = secure, automatic. Rollback on signature failure prevents the "compromised update server pushes malicious binary" attack.

**Why Full Disk Access prompt (DEC-192)?** macOS SIP blocks app access to `~/.cyberos-memory/` by default. FDA is one-time per user; persists. The prompt is annoying once, then invisible. Without FDA, the app can't read its own data — broken UX.

**Why hardened sandbox per OS (§1 #8)?** Compromised app shouldn't be able to read other apps' data, install kernel extensions, etc. Hardened runtime + entitlements (macOS) + AppContainer (Windows) limits blast radius. Trade-off: more development friction; worth it for security posture.

**Why quick-capture in tray (§1 #9)?** Friction kills ad-hoc note-taking. Tray-accessible textbox = instant capture (Cmd+Shift+Space global hotkey, opens textbox). User types, presses Enter, row written. The auto-tag (date + source: tray) saves the manual taxonomy.

**Why bundle size budget ≤ 30MB (§1 #13)?** Download UX matters. 100MB+ binaries take minutes to download on slow connections; users abandon install. 30MB downloads in <30s on most connections.

**Why opt-in crash reporting (§1 #14)?** Default-on crash reporting violates privacy expectations (BRAIN is the user's personal memory app). Opt-in respects user agency; UX still benefits from crashes when users opt in.

**Why headless mode (§1 #12)?** Linux server installs (e.g., always-on home server) don't need UI. The `--headless` flag runs sync daemon only; no tray; no main window. Useful for ops infrastructure.

**Why update-manifest signing (§1 #10)?** Update server compromise (Cloudflare R2 breach) could push malicious binary. Signature on manifest + verification before applying = the user's app refuses tampered updates. Public key embedded in app binary means even a compromised update server can't trick the client.

**Why config in OS-standard dirs (§1 #11)?** Each OS has conventions; respecting them means standard backup tools work, OS migration tools handle config, IT admins know where to look.

---

## §3 — API contract

```rust
// apps/brain/src-tauri/src/commands.rs
use tauri::State;

#[tauri::command]
async fn search_brain(query: String, state: State<'_, AppState>) -> Result<Vec<SearchResult>, String> {
    state.brain_search.search(&query).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_quick_note(text: String, state: State<'_, AppState>) -> Result<String, String> {
    let row = QuickNoteRow {
        text, ts_ns: chrono::Utc::now().timestamp_nanos(),
        source: "tray".into(), device_id: state.device_id,
        auto_tags: vec!["quick_note".into(), today_str()],
    };
    state.local_brain.write(row).await.map_err(|e| e.to_string())?;
    Ok("written".into())
}

#[tauri::command]
async fn get_sync_state(state: State<'_, AppState>) -> Result<SyncState, String> {
    Ok(SyncState {
        chain_head: state.local_brain.chain_head().await,
        last_sync_at: state.sync_supervisor.last_sync_at().await,
        last_sync_duration_ms: state.sync_supervisor.last_sync_duration_ms().await,
        disputed_pair_count: state.local_brain.disputed_pair_count().await,
        sync_class_breakdown: state.local_brain.sync_class_counts().await,
        cloud_state: state.sync_supervisor.cloud_state().await,
    })
}

#[derive(Serialize)]
pub struct SyncState {
    pub chain_head: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_duration_ms: u32,
    pub disputed_pair_count: u32,
    pub sync_class_breakdown: HashMap<String, u32>,
    pub cloud_state: String,    // "online" | "offline" | "syncing"
}
```

```rust
// apps/brain/src-tauri/src/sync_supervisor.rs
pub struct SyncSupervisor {
    handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    last_sync_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    cloud_state: Arc<RwLock<CloudState>>,
}

impl SyncSupervisor {
    pub async fn start(&self, local: Arc<LocalBrain>, cloud: Arc<CloudBrainClient>) {
        let handle = tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            loop {
                match brain_sync::sync_loop(&local, &cloud).await {
                    Ok(()) => break,
                    Err(e) => {
                        tracing::warn!(error = %e, "sync_loop crashed; restarting after {backoff:?}");
                        tokio::time::sleep(backoff).await;
                        backoff = (backoff * 2).min(Duration::from_secs(300));
                    }
                }
            }
        });
        *self.handle.lock().await = Some(handle);
    }
}
```

```rust
// apps/brain/src-tauri/src/main.rs
#[derive(clap::Parser)]
struct Cli {
    #[arg(long)] headless: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let local_brain = Arc::new(LocalBrain::open(brain_dir()).await.unwrap());
    let cloud = Arc::new(CloudBrainClient::connect_from_config().await.unwrap());
    let supervisor = SyncSupervisor::new();
    supervisor.start(local_brain.clone(), cloud).await;

    if cli.headless {
        // Linux server mode: no UI; just sync.
        loop { tokio::time::sleep(Duration::from_secs(60)).await; }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            permissions::request_full_disk_access(app)?;
            tray::install(app)?;
            Ok(())
        })
        .manage(AppState {
            local_brain, sync_supervisor: supervisor,
            brain_search: Arc::new(BrainSearch::new()),
            device_id: load_device_id(),
        })
        .invoke_handler(tauri::generate_handler![search_brain, write_quick_note, get_sync_state])
        .run(tauri::generate_context!())
        .expect("error running BRAIN app");
}
```

```json
// apps/brain/src-tauri/tauri.conf.json (excerpt)
{
  "tauri": {
    "bundle": {
      "identifier": "world.cyberos.brain",
      "category": "Productivity",
      "macOS": {
        "frameworks": [],
        "providerShortName": "CyberSkill",
        "signingIdentity": "Developer ID Application: CyberSkill (TEAMID)",
        "entitlements": "entitlements.plist",
        "minimumSystemVersion": "12.0"
      },
      "windows": {
        "certificateThumbprint": "<EV cert thumbprint>",
        "digestAlgorithm": "sha256",
        "timestampUrl": "http://timestamp.digicert.com",
        "wix": { "language": "en-US" }
      }
    },
    "updater": {
      "active": true,
      "endpoints": ["https://releases.cyberos.world/brain/{{target}}/{{current_version}}"],
      "dialog": false,
      "pubkey": "ed25519-public-key-base64..."
    }
  }
}
```

```yaml
# apps/brain/.github/workflows/release.yml
name: Release Brain App
on:
  push: { tags: ['brain-v*'] }

jobs:
  build:
    strategy:
      matrix:
        platform: [macos-latest, windows-latest, ubuntu-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - uses: pnpm/action-setup@v3
      - run: pnpm install
      - name: Build
        working-directory: apps/brain
        run: cargo tauri build
      - name: Sign macOS
        if: matrix.platform == 'macos-latest'
        run: ./scripts/sign-and-notarize-macos.sh
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_NOTARIZE_PASSWORD }}
      - name: Sign Windows
        if: matrix.platform == 'windows-latest'
        run: ./scripts/sign-windows.sh
      - name: Upload to R2
        run: aws s3 cp ./target/release/bundle/ s3://releases-cyberos/brain/ --recursive
        env:
          AWS_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_KEY }}
          AWS_ENDPOINT_URL: https://<account>.r2.cloudflarestorage.com
      - name: Generate signed update manifest
        run: |
          ./scripts/generate-update-manifest.sh \
              --version $GITHUB_REF_NAME \
              --signing-key /tmp/release-signing-key.ed25519 \
              --output update-manifest.json
          aws s3 cp update-manifest.json s3://releases-cyberos/brain/latest.json
```

---

## §4 — Acceptance criteria

1. App installs on macOS (drag-to-Applications) without Gatekeeper warning.
2. App installs on Windows (.msi double-click) without SmartScreen warning.
3. App installs on Linux (.deb + AppImage both work).
4. Updater pulls new release within 24h of publish (synthetic test publishes v1.1; v1.0 client picks up).
5. Update signature failure → rollback + sev-2 alert + in-app banner.
6. Sync daemon visible in tray; toggle works (pause/resume).
7. Local search returns memories ≤ 250ms p95.
8. Disputed-pair count badge updates in real-time as new conflicts arise.
9. Quick-capture textbox writes a `quick_note` BRAIN row with auto-tags.
10. Signed release verifies on every OS — `codesign --verify Brain.app` passes; Windows EV cert valid; Linux .deb dpkg-sig valid.
11. macOS Full Disk Access prompt appears on first run; persists after grant.
12. Sync supervisor restarts daemon on crash with exponential backoff.
13. Bundle size ≤ 30MB macOS / ≤ 25MB Windows.
14. `--headless` flag runs sync daemon only; no UI.
15. Crash reporting opt-in default off; can be toggled in Settings.
16. Multi-window: search in separate window works.
17. Tray quick action "Recent memories" shows last 10 rows from local Layer 1.

---

## §5 — Verification

```bash
# Manual installation tests (D = demonstration)
# macOS
cd apps/brain && cargo tauri build --target universal-apple-darwin
codesign --verify --deep --strict ./src-tauri/target/release/bundle/macos/Brain.app
spctl --assess --type exec ./src-tauri/target/release/bundle/macos/Brain.app

# Windows
cd apps/brain && cargo tauri build
signtool verify /pa ./src-tauri/target/release/bundle/msi/Brain.msi

# Linux
cd apps/brain && cargo tauri build
dpkg-deb --info ./src-tauri/target/release/bundle/deb/brain_*.deb
```

```rust
// apps/brain/src-tauri/tests/sync_supervisor_test.rs
#[tokio::test]
async fn supervisor_restarts_daemon_on_crash() {
    let supervisor = SyncSupervisor::new();
    let local = Arc::new(test_local_brain());
    let cloud = Arc::new(MockCloud::start());

    test_helper::inject_sync_panic_after(Duration::from_secs(1));
    supervisor.start(local, cloud).await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(supervisor.is_running().await);   // restarted after panic
}
```

```typescript
// apps/brain/src/lib/__tests__/quick_capture.test.ts
test('quick_capture writes a row via Tauri command', async () => {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('write_quick_note', { text: 'test note' });
  const state = await invoke('get_sync_state');
  expect(state.sync_class_breakdown.private).toBeGreaterThan(0);
});
```

```bash
# Auto-update test (D)
# Publish v1.0; install on test machine
# Publish v1.1
# Wait ≤ 24h
# Verify client auto-updates to v1.1
```

```bash
# Headless mode test
brain --headless &
sleep 5
ps aux | grep brain   # should be running
ls ~/.cyberos-memory/audit/  # should have latest binlog
```

```bash
# Bundle size test
cd apps/brain && cargo tauri build
ls -lh ./src-tauri/target/release/bundle/macos/Brain.app
# Assert: <= 30MB
ls -lh ./src-tauri/target/release/bundle/msi/Brain.msi
# Assert: <= 25MB
```

---

## §6 — Implementation skeleton

See §3.

```rust
// apps/brain/src-tauri/src/tray.rs
pub fn install(app: &mut tauri::App) -> tauri::Result<()> {
    let tray_menu = Menu::new()
        .add_item(MenuItem::with_id("open", "Open BRAIN"))
        .add_item(MenuItem::with_id("quick_capture", "Quick Capture..."))
        .add_separator()
        .add_item(MenuItem::with_id("force_sync", "Force Sync Now"))
        .add_item(MenuItem::with_id("toggle_sync", "Pause Sync"))
        .add_separator()
        .add_item(MenuItem::with_id("recent", "Recent Memories"))
        .add_separator()
        .add_item(MenuItem::with_id("quit", "Quit"));
    let _tray = TrayIconBuilder::with_id("main")
        .menu(&tray_menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(handle_tray_event)
        .build(app)?;
    Ok(())
}
```

---

## §7 — Dependencies

- **FR-BRAIN-103** — sync daemon embedded.
- **FR-BRAIN-105 (downstream)** — disputed-pair UI link.
- **FR-BRAIN-108** — search API.
- Crates: `tauri@2.0`, `tauri-plugin-updater`, `tauri-plugin-dialog`, `tauri-plugin-notification`, `tokio`, `clap@4`, `sentry@0.32` (opt-in).
- Apple Developer Program membership (Apple ID + signing cert).
- Windows EV code-signing cert.
- Cloudflare R2 for releases.
- Svelte 5 + Tailwind 4.

---

## §8 — Example payloads

### Quick-capture row

```json
{
  "kind": "quick_note",
  "ts_ns": 1747526400000000000,
  "body": "Remember to follow up with Stephen about the Bedrock cost spike.",
  "meta": {
    "sync_class": "private",
    "auto_tags": ["quick_note", "2026-05-15"]
  },
  "extra": {
    "source": "tray",
    "originator_device_id": "device-mbp"
  }
}
```

### Sync state response

```json
{
  "chain_head": "a3f9c8d7e6b5a4f3...",
  "last_sync_at": "2026-05-15T14:00:30.123Z",
  "last_sync_duration_ms": 2150,
  "disputed_pair_count": 2,
  "sync_class_breakdown": { "shareable": 1247, "private": 384 },
  "cloud_state": "online"
}
```

### Update manifest (from R2)

```json
{
  "version": "1.1.0",
  "notes": "Bug fixes + perf",
  "pub_date": "2026-05-15T00:00:00Z",
  "platforms": {
    "darwin-x86_64":   { "url": "https://releases.cyberos.world/brain/1.1.0/Brain-darwin-x86_64.app.tar.gz", "signature": "..." },
    "darwin-aarch64":  { "url": "https://releases.cyberos.world/brain/1.1.0/Brain-darwin-aarch64.app.tar.gz", "signature": "..." },
    "windows-x86_64":  { "url": "https://releases.cyberos.world/brain/1.1.0/Brain-windows-x86_64.msi", "signature": "..." }
  }
}
```

### Update signature failure log

```text
ERROR update_signature_invalid version=1.1.0
      expected_pubkey=ed25519-... actual=...
      Rollback applied; user notified via in-app banner
sev-2 brain_app_update_signature_failures_total incremented
```

---

## §9 — Open questions

All resolved. Deferred:
- Mobile app (iOS + Android) — slice 4+; different runtime (Swift / Kotlin).
- Plugin system (third-party Tauri plugins) — slice 5+.
- BRAIN UI for Windows ARM — slice 5+ when demand.
- App Store distribution — slice 5+ (Apple Mac App Store gatekeeping).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Full Disk Access not granted | Brain can't read ~/.cyberos-memory/ | App shows banner asking user to grant | User clicks System Settings |
| Updater fails | network error | Sev-2 metric; manual update via "Check for Updates" | Operator action |
| Update signature invalid | ed25519 verify fails | Rollback applied; sev-2 alarm | Operator investigates manifest source |
| WebView2 missing (Windows) | Tauri prompts user to install | One-time install | User action |
| WKWebView crash (macOS) | Tauri watchdog | Restart WebView | Self-heals |
| Sync daemon crash | tokio task panic | Supervisor restarts with exponential backoff | Self-heals |
| macOS Gatekeeper blocks | Notarisation expired or invalid | User can't open app | Re-notarise + push update |
| Windows SmartScreen warning | EV cert reputation | "More info" workaround for users | Build cert reputation OR re-issue |
| Linux .deb missing dependency | apt install error | User installs missing | One-time |
| Bundle size > budget | CI check at build | Fail | Engineer optimises deps |
| Crash report sent without opt-in | privacy violation | Sev-1 incident | Investigate; force opt-out by default |
| Tray icon disappears | OS settings issue | User restarts app | OS-specific |
| Multi-window state inconsistent | known limitation | Refresh button | Slice 3+ fix |
| Headless mode + UI flag passed | clap error | Exit 1 | User fixes args |
| Auto-update during BRAIN write | race | Update applied at next launch | By design |
| Cloud BRAIN config wrong | sync daemon errors | Settings UI shows error | User fixes config |
| Device ID collision (extremely unlikely) | UUID randomness | N/A | N/A |
| Tauri 2.x API breaking change | compile error | PR blocked | Engineer pins version |
| Sentry-rust panic on disabled | catch | No-op | By design |
| Localisation file missing | fallback to English | Sev-3 alarm | Translation engineer |

---

## §11 — Notes

- Tauri 2.x produces 5-10× smaller binaries than Electron — critical for download UX.
- Apple notarisation can take 10-30 min; CI workflow waits for completion before tagging release as "ready."
- Windows EV cert ($300-500/year) is worth the SmartScreen-bypass.
- Cloudflare R2 chosen over S3 for cost (no egress fees) + global CDN.
- Update-manifest signing uses a SEPARATE Ed25519 key from BRAIN signing key. Compromise of one doesn't compromise the other.
- Full Disk Access on macOS is the hardest UX friction — first-run banner with screenshot directs user; once granted, persists across updates.
- AppContainer on Windows limits damage from compromised app — file system access scoped to user profile only.
- Headless mode covers Linux server installs (always-on home server). No UI overhead; just sync.
- Bundle size budget enforced in CI prevents accidental dependency bloat (e.g., adding a 50MB image library).
- Crash reporting opt-in respects user agency (BRAIN is personal memory; default-on telemetry feels invasive).
- Multi-window support is slice-2 work; multi-window state coordination is non-trivial (each window has own search; main window is primary).
- Localisation infrastructure (i18n keys + translation files) is slice-2 wiring; actual VN translations are slice-3+ when content stabilises.

---

*End of FR-BRAIN-104. Status: draft (10/10 target).*
