# CyberOS memory — desktop app

Tauri 2.x desktop client for CyberOS memory. First-slice scaffold for [TASK-MEMORY-104](../../../docs/tasks/memory/TASK-MEMORY-104-tauri-app.md).

## What's wired (first slice)

- Tauri 2 builder with `tauri-plugin-shell` + `tauri-plugin-fs`.
- Three commands callable from the Svelte UI via `@tauri-apps/api/core::invoke`:
- `search_memory(query, limit)` — POSTs to `127.0.0.1:7901/v1/memory/search` (the Rust memory service from session 14).
- `write_quick_note(text, tags)` — writes a `.md` capture under `~/.cyberos/memory/store/default/captures/`.
- `get_sync_state()` — reads `~/.cyberos/memory/store/default/sync/last-status.json`.
- A `sync_supervisor` background task that spawns `python3 -m cyberos.core.memory_sync_daemon` with a 5-restarts-per-60s circuit breaker. ENOENT is handled gracefully (sleep + retry until the daemon ships).
- A system tray with Open / Force Sync / Quit. Close-button on the main window is intercepted and **minimises to tray**.
- Svelte 5 (runes) + Vite + Tailwind 3 frontend with a Dashboard / Search / Sync tab strip.

## What is **not** wired yet (deferred)

- Real auto-update signature verification (`tauri-plugin-updater` is wired in config but not activated; pubkey blank).
- macOS Full Disk Access prompt + signing/notarisation.
- Windows EV code-signing.
- Quick-capture global hotkey (Cmd+Shift+Space).
- Recent-memories tray submenu.
- Cloud memory connection state.
- Disputed-pair count badge.
- Headless mode (`--headless` flag).
- Sentry-rust crash reporting (opt-in).
- Multi-window search.

## Run

Prerequisites: Node 20+, pnpm 9+, Rust 1.81+, the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```bash
cd apps/memory
pnpm install
pnpm tauri dev
```

Vite serves the frontend on `http://localhost:1420` (port fixed + strict).

## Build a release (unsigned)

```bash
cd apps/memory
pnpm tauri build
```

Bundles land under `services/memory/desktop/src-tauri/target/release/bundle/`.

## Signing + notarisation

### Updater keypair (one-time, per release line)

```bash
cd apps/memory
./scripts/generate-updater-keys.sh ./out
```

Paste the printed public key into `services/memory/desktop/src-tauri/tauri.conf.json` → `plugins.updater.pubkey`. Keep `./out/tauri-updater.key` in your secrets vault — releases sign update manifests with it.

### macOS — Developer ID + notarisation

Required:

- Apple Developer Program membership.
- Developer ID Application certificate installed in Keychain.
- Apple ID with an app-specific password (issue at appleid.apple.com).
- Team ID (10 chars).

Set env vars + run:

```bash
export APPLE_DEVELOPER_ID="Developer ID Application: CYBERSKILL ... (TEAMID)"
export APPLE_NOTARIZE_APPLE_ID="you@example.com"
export APPLE_NOTARIZE_PASSWORD="app-specific-password"
export APPLE_NOTARIZE_TEAM_ID="ABCD1234EF"

./scripts/sign-and-notarize-macos.sh \
  src-tauri/target/release/bundle/dmg/CyberOS-memory.dmg
```

The script generates `src-tauri/entitlements.plist` on first run (hardened runtime + JIT + network client + bookmarked file access), signs the inner `.app` and outer `.dmg`, submits to Apple notarisation with `xcrun notarytool --wait`, staples the ticket, and verifies with `spctl`.

### Windows — EV code-signing

Required:

- EV code-signing certificate (recommended — bypasses SmartScreen warnings).
- `signtool.exe` on PATH (ships with Windows 10/11 SDK).

```bash
export WIN_SIGN_CERT_PATH="/path/to/cert.pfx"
export WIN_SIGN_CERT_PASS="..."
./scripts/sign-windows.sh src-tauri/target/release/bundle/msi/CyberOS-memory.msi
```

For hosted CI prefer Azure Code Signing rather than copying a `.pfx` into the runner image.

## Where the daemon lives

The supervisor invokes the system `python3` and expects the `cyberos-memory` package to be installed (provides `cyberos.core.memory_sync_daemon`). **The daemon module itself doesn't exist yet** — it ships in a follow-up slice. Until then, the supervisor sleeps + retries; the app stays usable.

## Caveat: not in the services/ workspace

This crate is **intentionally outside** the `services/Cargo.toml` workspace. Tauri builds are slow and OS-specific; keeping them in their own crate lets the rest of the Rust monorepo `cargo build` quickly. The trade-off is a separate `Cargo.lock` here.
