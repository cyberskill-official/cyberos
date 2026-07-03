# Releasing CyberOS

One page for how CyberOS ships across every surface. There are two release tracks, and they are deliberately
separate:

- Continuous delivery (web + services): every push to `main` updates the live site and the backend. No
  version, no tag, no manual step. This is where day-to-day work goes out.
- Versioned native release (desktop + mobile): you cut these on demand by pushing a `vX.Y.Z` tag. Only the
  native installers are versioned, because only they are downloaded and pinned to a machine.

The chat client is a single React app (`apps/web`). Desktop and mobile are thin wrappers around that same
app, so there is one UI to maintain, and a web deploy updates every surface on next launch.

## The one client, four surfaces

- Web: `apps/web` (React 18 + TypeScript + Vite) built into `apps/console/web`, served by Caddy at
  `https://os.cyberskill.world/web/`. The site root redirects there.
- PWA: the same web app, installable from the browser (manifest + service worker + icons). No build.
- Desktop: a Tauri 2 window (`apps/desktop`) that loads the live `/web/`. Same UI, native window + dock icon.
- Mobile: install the PWA today; ship a store build by wrapping the same `apps/web` bundle in Capacitor.

Because desktop and the PWA load the live web app, every web deploy updates them with nothing to rebuild. The
Capacitor mobile build bundles the web assets, so it updates when you cut a new mobile release (or you can
point it at the live URL later if you prefer over-the-air web updates).

## Track 1: continuous delivery (web + services)

Push to `main`. `.github/workflows/deploy.yml` runs the whole thing in GitHub Actions (moved off the local
machine on 2026-07-03):

1. changes: detects whether `services/**` changed in the push.
2. gate: if services changed, runs `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` on a Linux
   runner. A failure here stops the deploy.
3. build-images: builds the four service images and pushes them to GHCR (`cyberos-auth`, `cyberos-chat`,
   `cyberos-ai-gateway`, `cyberos-embed-sidecar`), tagged `:latest` and the short SHA, with a layer cache.
4. deploy: SSHes to the VPS and runs `deploy/vps/deploy.sh` (git pull the console bundle + Caddyfile +
   compose, `docker pull` the images, roll). A client-only push (apps/web, console, compose) skips the gate
   and image build and just rolls, because the React bundle ships via the VPS git pull.

Safe by construction: if the gate or the image build fails, the roll is skipped and the site stays on the
current images. The local `.githooks/pre-push` still runs the fast Rust checks before a push as an early
tripwire, but it no longer builds images (CI owns that). Opt back into a local image build with
`BUILD_IMAGES_LOCALLY=1 git push`.

Local development and the production web build:

    scripts/dev/dev-up.sh     # builds + starts auth + chat + console, seeds a demo user, prints the URL
    scripts/dev/dev-down.sh   # stops it
    # see scripts/dev/README.md

    cd apps/web
    NODE_ENV=development npm ci     # the login shell exports production, which strips devDeps like vite
    npm run build                  # tsc --noEmit && vite build -> apps/console/web + a service-worker stamp

## Track 2: versioned native release (desktop + mobile)

Cut a release by pushing a tag:

    git tag v1.2.0
    git push origin v1.2.0

`.github/workflows/release.yml` then builds the native binaries and publishes a draft GitHub Release with the
desktop installers attached:

- desktop: the official `tauri-action` builds a signed installer on each OS - `.dmg` (macOS, notarized),
  `.msi`/`.exe` (Windows), `.AppImage` (Linux). It runs unsigned until the signing secrets below are present,
  so the pipeline works from day one and signing slots in later.
- android: builds the web app, `npx cap sync android`, and assembles a signed `.aab`. Gated on the repo
  variable `MOBILE_RELEASE=true`.
- ios: builds the web app, `npx cap sync ios`, and (with fastlane) archives and uploads to TestFlight. Same
  gate.

Keep the version in one place: bump `apps/desktop/src-tauri/tauri.conf.json` `version` and
`apps/web/package.json` `version` to match the tag before you push it. The tag drives the GitHub Release name;
the two `version` fields drive what the installers report.

### PWA (installable, works today, no build)

The web app ships a valid manifest (`apps/web/public/manifest.webmanifest`, `display: standalone`, scope
`/web/`, 192 + 512 icons) and a service worker (`sw.js`), so it installs as an app with no build:

- Desktop Chrome / Edge: the Install icon in the address bar (or menu -> Install CyberOS).
- Android Chrome: menu -> Add to Home screen.
- iOS Safari: Share -> Add to Home Screen (also what enables iOS web push, which requires an installed PWA).

This is the fastest way to put CyberOS on a phone or a dock right now, before any store listing.

### Desktop notes

`apps/desktop` is a Tauri 2 app whose window loads `https://os.cyberskill.world/web/`
(`apps/desktop/src/index.html`), so the desktop app IS the web app in a native window. Two settings make the
webview behave: a Safari `userAgent` (Google blocks the default embedded-webview UA on OAuth) and camera +
microphone usage strings (for WebRTC `getUserMedia`). Full first-build notes are in `apps/desktop/README.md`.
`release.yml` builds it in CI, so you do not need a Mac/Windows/Linux box per target yourself.

### Mobile one-time init (before the first mobile release)

The android/ and ios/ projects do not exist in the repo yet. Create them once, locally:

    cd apps/web
    npm i -D @capacitor/core @capacitor/cli @capacitor/ios @capacitor/android
    npx cap add ios
    npx cap add android
    git add android ios capacitor.config.ts package.json && git commit -m "chore: add Capacitor mobile shells"

`capacitor.config.ts` is already committed (appId `world.cyberskill.cyberos`, webDir `../console/web`). After
the shells are committed and the signing secrets below are set, flip the repo variable `MOBILE_RELEASE=true`
so `release.yml` starts building the mobile apps. Push notifications are the one unfinished backend piece: the
chat service registers devices and emits a push intent, but the actual APNS/FCM send is stubbed
(`services/chat/src/push.rs`); wire real delivery before relying on mobile push.

## Activation checklist (secrets and accounts to procure)

Add secrets in the repo: Settings -> Secrets and variables -> Actions. All signing material is yours to
generate and hold - never paste a private key, certificate, or password into a file or a chat; only into the
GitHub secret box. Set the mobile toggle as a repo variable (the Variables tab, not Secrets).

Continuous delivery (Track 1) - required now:

- `VPS_HOST`, `VPS_USER`, `VPS_SSH_KEY` - the deploy target and a deploy SSH private key. The VPS also needs a
  read-only git deploy key (for `git pull`) and a `docker login ghcr.io` (for `docker pull`); see
  `deploy/vps/auto-deploy.md`. Image push uses the built-in `GITHUB_TOKEN`, so there is no registry secret.

Desktop signing (Track 2) - optional; unsigned builds work without these:

- macOS (Apple Developer account, USD 99/year): `APPLE_CERTIFICATE` (base64 of the Developer ID .p12),
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (e.g. "Developer ID Application: CyberSkill ..."),
  `APPLE_ID`, `APPLE_PASSWORD` (an app-specific password), `APPLE_TEAM_ID`.
- Tauri auto-updater (optional): generate a keypair with `cargo tauri signer generate`, then set
  `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Auto-update also needs the updater
  plugin wired into the app and an endpoint - it is not enabled yet, so treat this as a later enhancement.
- Windows: `tauri-action` produces an installer but does not sign it by default. Signing needs a code-signing
  certificate (OV or EV, from a CA, roughly USD 100-400/year) and a `signCommand` in `tauri.conf.json`. Left
  as a follow-up; unsigned Windows installers show a SmartScreen warning until then.

Mobile signing (Track 2) - needed once you flip `MOBILE_RELEASE=true`:

- Android (Google Play, USD 25 one-time): create an upload keystore
  (`keytool -genkey -v -keystore release.keystore -alias cyberos -keyalg RSA -keysize 2048 -validity 10000`),
  then set `ANDROID_KEYSTORE_BASE64` (base64 of that file), `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`,
  `ANDROID_KEY_PASSWORD`.
- iOS (Apple Developer account, the same USD 99/year): an App Store Connect API key -
  `APP_STORE_CONNECT_KEY_ID`, `APP_STORE_CONNECT_ISSUER_ID`, `APP_STORE_CONNECT_API_KEY` (the .p8 contents).

Repo variable:

- `MOBILE_RELEASE` = `true` - turns on the android + ios jobs in `release.yml`. Leave it unset until the
  Capacitor shells are committed and the mobile secrets are in, so tagged releases keep working in the
  meantime (they just build desktop only).

## Cutting a release, end to end

1. Land the work on `main` (it deploys to the web/service surface automatically).
2. Bump `version` in `apps/desktop/src-tauri/tauri.conf.json` and `apps/web/package.json` to the new number.
3. `git commit`, then `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. Watch the `release` workflow. It creates a draft GitHub Release with the desktop installers (and, once
   `MOBILE_RELEASE=true`, the Android bundle and a TestFlight upload).
5. Edit the draft release notes and publish.
