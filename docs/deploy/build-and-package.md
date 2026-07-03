# Build and package CyberOS (web, PWA, desktop, mobile)

One place that ties together how the app is built and shipped across every surface. The chat client is a
single React app (`apps/web`); desktop and mobile are thin wrappers around that same app, so there is one UI
to maintain. Details live in the linked docs; this page is the map.

## The one client, four surfaces

- Web: `apps/web` (React 18 + TypeScript + Vite) built into `apps/console/web`, served by Caddy at
  `https://os.cyberskill.world/web/`. The site root redirects there.
- PWA: the same web app, installable from the browser (manifest + service worker + icons). No build.
- Desktop: a Tauri 2 window that loads the live `/web/`. Same UI, native window + dock icon.
- Mobile: install the PWA today; wrap the same app in Capacitor for an App Store / Play listing later.

Because desktop and the PWA load the live web app, every web deploy updates them on next launch - nothing to
rebuild or keep in sync.

## Web app

Local development (one command):

    scripts/dev/dev-up.sh     # builds + starts auth + chat + console, seeds a demo user, prints the URL
    scripts/dev/dev-down.sh   # stops it
    # see scripts/dev/README.md

Production build (what the deploy runs for you):

    cd apps/web
    NODE_ENV=development npm ci     # the login shell exports production, which strips devDeps like vite
    npm run build                  # tsc --noEmit && vite build -> apps/console/web + a service-worker cache stamp

Deploy: push to `main`. The pre-push hook builds the service images and pushes them to GHCR; the VPS runs
`deploy/vps/deploy.sh` (pull, migrate, roll), and Caddy serves the rebuilt bundle from `apps/console/web`.
See `docs/deploy/p0-google-chat-runbook.md`.

## PWA (installable web app - works today)

The web app ships a valid manifest (`apps/web/public/manifest.webmanifest`, `display: standalone`, scope
`/web/`, 192 + 512 icons) and a service worker (`sw.js`), so it installs as an app with no build:

- Desktop Chrome / Edge: open the site, use the Install icon in the address bar (or menu -> Install CyberOS).
- Android Chrome: menu -> Add to Home screen.
- iOS Safari: Share -> Add to Home Screen (needed for iOS web push, which requires an installed PWA).

The installed PWA runs full-screen and is responsive (touch long-press message actions, the mobile channel
drawer, safe-area handling). It is the fastest way to put CyberOS on a phone or a dock right now.

## Desktop app (Tauri 2)

`apps/desktop` is a Tauri 2 app whose window loads `https://os.cyberskill.world/web/` (see
`apps/desktop/src/index.html`), so the desktop app IS the web app in a native window. It must be compiled on
each target OS (Tauri does not cross-build cleanly, and there is no Tauri toolchain in the authoring sandbox).

Prerequisites per build machine: Rust stable; macOS needs Xcode Command Line Tools, Windows needs MSVC build
tools + WebView2; then `cargo install tauri-cli --version "^2"`.

    cd apps/desktop/src-tauri
    cargo tauri icon <path-to-CyberSkill-logo-mark.png>   # one-time, writes icons/
    cargo tauri dev                                       # dev window on the live console
    cargo tauri build                                     # .dmg (macOS) / .msi/.exe (Windows) under target/release/bundle/

Two settings make the webview behave: a Safari `userAgent` (so Google OAuth loads - Google blocks the default
embedded-webview UA) and `Info.plist` camera + microphone strings (for WebRTC `getUserMedia` on macOS). Full
steps, signing/notarization notes, and first-build test points are in `apps/desktop/README.md`.

## Mobile app

Two paths, in the order I recommend:

1. PWA now (no store, no build): install to the home screen as above. Good enough for the team immediately.
2. Capacitor when you want App Store / Play (later): wrap the SAME `apps/web` build in Capacitor - it loads
   the web assets in a native shell and adds native push, camera, and share. Sketch:

       cd apps/web
       npm i -D @capacitor/core @capacitor/cli @capacitor/ios @capacitor/android
       npx cap init CyberOS world.cyberskill.cyberos --web-dir ../console/web
       npx cap add ios && npx cap add android
       npm run build && npx cap sync
       npx cap open ios      # build/sign in Xcode
       npx cap open android  # build/sign in Android Studio

   Capacitor reuses the React app unchanged, so there is one codebase for web + desktop + mobile. A React
   Native rewrite is the heavier alternative and is not recommended unless a fully native feel is required.

Push notifications - the one unfinished piece for mobile: the chat backend already registers devices
(`chat_devices`, `POST /v1/chat/devices`) and emits a push intent on a new message, but the actual APNS /
FCM send is stubbed (`services/chat/src/push.rs` logs the intent). Wiring real delivery is the work needed
before mobile push is live; web push (Android PWA, and iOS PWA on 16.4+) can be added on the same device
registration.

## Which surface when

- Try it on a phone or pin it to a dock this week: install the PWA.
- Hand employees a signed installer: build the Tauri desktop app per OS.
- List in the App Store / Play with native push: add Capacitor, then finish APNS/FCM delivery.
