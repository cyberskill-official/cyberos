# CyberOS release process

One page for how CyberOS ships across every surface. CyberOS carries ONE platform version (the root `VERSION` file, stamped into the init payload and every consumer repo's `.cyberos/VERSION`); module versions are internal. There are two release tracks, deliberately separate:

- Continuous delivery (web + services): every push to `main` updates the live site and the backend. No version, no tag, no manual step. This is where day-to-day work goes out.
- Versioned native release (desktop + mobile): cut on demand by pushing a `vX.Y.Z` tag. Only the native installers are versioned, because only they are downloaded and pinned to a machine.

The chat client is a single React app (`apps/web`). Desktop and mobile are thin wrappers around that same app, so there is one UI to maintain, and a web deploy updates every surface on next launch.

## The one client, four surfaces

- Web: `apps/web` (React 18 + TypeScript + Vite) built into `apps/console/web`, served by Caddy at the site root, `https://os.cyberskill.world/` (the legacy `/web/` prefix 308-redirects home). Each module owns a URL (`/chat`, `/dashboard`, `/<module>`), and the generated docs site is served at `/docs`.
- PWA: the same web app, installable from the browser (manifest + service worker + icons). No build.
- Desktop: a Tauri 2 window (`apps/desktop`) that loads the live site root. Same UI, native window + dock icon.
- Mobile: install the PWA today; ship a store build by wrapping the same `apps/web` bundle in Capacitor.

Because desktop and the PWA load the live web app, every web deploy updates them with nothing to rebuild. The Capacitor mobile build bundles the web assets, so it updates when you cut a new mobile release (or point it at the live URL later for over-the-air web updates).

## Staying up to date (check for update + auto-update)

Because all four surfaces load the same root bundle, one update mechanism at the web layer covers every platform:

- The check: each `npm run build` writes `/version.json` with a unique build id (the same id that stamps the service-worker cache). The running app records the id it loaded with, then re-checks `version.json` on an interval and whenever the tab regains focus or the network returns (`apps/web/src/lib/useUpdateCheck.ts`).
- The prompt: when a newer id is live, a small "A new version is available - Reload" banner appears (`UpdateBanner`). Reload applies it - the service worker is network-first, so the reload pulls the fresh index and hashed bundles and the new worker purges the old caches. This works identically in the browser, the installed PWA, the Tauri desktop window, and the Capacitor mobile shell.
- Desktop shell binary: the Tauri wrapper rarely changes (it just loads the site root), so the banner above already updates what the user sees. The `tauri-plugin-updater` is wired in and checks on launch (`apps/desktop/src-tauri/src/lib.rs`), staying a quiet no-op until it has a config. To switch it on, do the three key-dependent steps in the activation checklist: generate the signing keypair, add a `plugins.updater` block to `apps/desktop/src-tauri/tauri.conf.json` with your public key and the GitHub releases endpoint, and set `bundle.createUpdaterArtifacts: true`. `release.yml` then emits the signed update artifacts.
- Mobile binary: native app updates ship through the App Store / Play from a tagged release; the web-layer banner still covers the in-app content between store updates.
- Consumer repos (the init payload): compare installed vs available with `bash dist/cyberos/init.sh --check <repo>`, re-run `init.sh <repo>` to apply; the desktop app's CyberOS Ops tab and `tools/cyberos-init/rollout.sh` (fleet-wide) drive the same scripts.

## Track 1: continuous delivery (web + services)

Push to `main`. `.github/workflows/deploy.yml` runs the whole thing in GitHub Actions:

1. changes: detects whether `services/**` changed in the push.
2. gate: if services changed, runs `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` on a Linux runner. A failure here stops the deploy.
3. build-images: builds the service images and pushes them to GHCR (`cyberos-auth`, `cyberos-chat`, `cyberos-ai-gateway`, `cyberos-embed-sidecar`; eval + memory best-effort), tagged `:latest` and the short SHA, with a layer cache.
4. deploy: SSHes to the VPS and runs `deploy/vps/deploy.sh` (git pull the console bundle + Caddyfile + compose, `docker pull` the images, roll). A client-only push (apps/web, console, compose) skips the gate and image build and just rolls, because the React bundle ships via the VPS git pull.

Safe by construction: if the gate or the image build fails, the roll is skipped and the site stays on the current images. The local `.githooks/pre-push` runs the CI-equivalent checks before a push as an early tripwire (`scripts/local_verify.sh` when Docker is up), but it does not build images (CI owns that). Opt into a local image build with `BUILD_IMAGES_LOCALLY=1 git push`.

Local development and the production web build:

    scripts/dev/dev-up.sh     # builds + starts auth + chat + console, seeds a demo user, prints the URL
    scripts/dev/dev-down.sh   # stops it
    # see scripts/dev/README.md

    cd apps/web
    NODE_ENV=development npm ci     # the login shell exports production, which strips devDeps like vite
    npm run build                   # tsc --noEmit && vite build -> apps/console/web + a service-worker stamp

## Track 2: versioned native release (desktop + mobile)

Cut a release by pushing a tag:

    git tag v1.2.0
    git push origin v1.2.0

`.github/workflows/release.yml` then builds the native binaries and publishes a draft GitHub Release with the desktop installers attached:

- desktop: the official `tauri-action` builds an installer on each OS - `.dmg` (macOS), `.msi`/`.exe` (Windows), `.AppImage` (Linux). Signing is OPT-IN: the default build is UNSIGNED and always works. macOS signing turns on only when the repo variable `MACOS_SIGN=true` AND the `APPLE_*` secrets are set (the workflow forces the Apple env empty otherwise, so a stray or malformed certificate secret can never break the build - it did, on the first v1.0.0 tag: `security import: failed to import keychain certificate`). Updater signing is likewise gated on `DESKTOP_UPDATER_SIGN=true`.
- android: builds the web app, `npx cap sync android`, and assembles a signed `.aab`. Gated on the repo variable `MOBILE_RELEASE=true`.
- ios: builds the web app, `npx cap sync ios`, and (with fastlane) archives and uploads to TestFlight. Same gate.

### PWA (installable, works today, no build)

The web app ships a valid manifest (`apps/web/public/manifest.webmanifest`, `display: standalone`, scope `/`, 192 + 512 icons) and a service worker (`sw.js`), so it installs as an app with no build:

- Desktop Chrome / Edge: the Install icon in the address bar (or menu -> Install CyberOS).
- Android Chrome: menu -> Add to Home screen.
- iOS Safari: Share -> Add to Home Screen (also what enables iOS web push, which requires an installed PWA).

This is the fastest way to put CyberOS on a phone or a dock, before any store listing.

### Desktop notes

`apps/desktop` is a Tauri 2 app whose window loads `https://os.cyberskill.world/` (`apps/desktop/src/index.html`), so the desktop app IS the web app in a native window. Two settings make the webview behave: a Safari `userAgent` (Google blocks the default embedded-webview UA on OAuth) and camera + microphone usage strings (for WebRTC `getUserMedia`). Full first-build notes are in `apps/desktop/README.md`. `release.yml` builds it in CI, so you do not need a Mac/Windows/Linux box per target.

### Mobile one-time init (before the first mobile release)

The android/ and ios/ projects do not exist in the repo yet. Create them once, locally:

    cd apps/web
    npm i -D @capacitor/core @capacitor/cli @capacitor/ios @capacitor/android
    npx cap add ios
    npx cap add android
    git add android ios capacitor.config.ts package.json && git commit -m "chore: add Capacitor mobile shells"

`capacitor.config.ts` is already committed (appId `os.cyberskill.world`, webDir `../console/web`). After the shells are committed and the signing secrets below are set, flip the repo variable `MOBILE_RELEASE=true` so `release.yml` starts building the mobile apps. Push notifications are the one unfinished backend piece: the chat service registers devices and emits a push intent, but the actual APNS/FCM send is stubbed (`services/chat/src/push.rs`); wire real delivery before relying on mobile push.

## Activation checklist (secrets and accounts to procure)

Add secrets in the repo: Settings -> Secrets and variables -> Actions. All signing material is yours to generate and hold - never paste a private key, certificate, or password into a file or a chat; only into the GitHub secret box. Set the mobile toggle as a repo variable (the Variables tab, not Secrets).

Continuous delivery (Track 1) - required now:

- `VPS_HOST`, `VPS_USER`, `VPS_SSH_KEY` - the deploy target and a deploy SSH private key. The VPS also needs a read-only git deploy key (for `git pull`) and a `docker login ghcr.io` (for `docker pull`); see `deploy/vps/auto-deploy.md`. Image push uses the built-in `GITHUB_TOKEN`, so there is no registry secret.

Desktop signing (Track 2) - optional; unsigned builds work without these:

- macOS (Apple Developer account, USD 99/year): set the repo variable `MACOS_SIGN=true` AND the secrets `APPLE_CERTIFICATE` (base64 of the Developer ID .p12), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (e.g. "Developer ID Application: CyberSkill ..."), `APPLE_ID`, `APPLE_PASSWORD` (an app-specific password), `APPLE_TEAM_ID`. Until `MACOS_SIGN=true`, the macOS build ships unsigned regardless of what secrets exist (users clear it with right-click -> Open the first time).
- Tauri auto-updater (optional): the plugin is already wired and checks on launch; these steps switch it on. (1) `cargo tauri signer generate` - keep the private key and set `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as secrets. (2) In `apps/desktop/src-tauri/tauri.conf.json` add a `plugins.updater` block with your public key and `"endpoints": ["https://github.com/cyberskill-official/cyberos/releases/latest/download/latest.json"]`, and set `"createUpdaterArtifacts": true` under `bundle`. Then a tagged release publishes the update.
- Windows: `tauri-action` produces an installer but does not sign it by default. Signing needs a code-signing certificate (OV or EV, from a CA, roughly USD 100-400/year) and a `signCommand` in `tauri.conf.json`. Left as a follow-up; unsigned Windows installers show a SmartScreen warning until then.

Mobile signing (Track 2) - needed once you flip `MOBILE_RELEASE=true`:

- Android (Google Play, USD 25 one-time): create an upload keystore (`keytool -genkey -v -keystore release.keystore -alias cyberos -keyalg RSA -keysize 2048 -validity 10000`), then set `ANDROID_KEYSTORE_BASE64` (base64 of that file), `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`.
- iOS (Apple Developer account, the same USD 99/year): an App Store Connect API key - `APP_STORE_CONNECT_KEY_ID`, `APP_STORE_CONNECT_ISSUER_ID`, `APP_STORE_CONNECT_API_KEY` (the .p8 contents).

Repo variable:

- `MOBILE_RELEASE` = `true` - turns on the android + ios jobs in `release.yml`. Leave it unset until the Capacitor shells are committed and the mobile secrets are in, so tagged releases keep working in the meantime (they just build desktop only).

## Step by step

### Part A: first-time setup (do once, in this order)

1. Confirm continuous delivery is on. In GitHub -> Settings -> Secrets and variables -> Actions, check that `VPS_HOST`, `VPS_USER`, and `VPS_SSH_KEY` exist. Push any small change to `main` and confirm the `deploy` workflow goes green. This is already live for os.cyberskill.world, so it is a verification step, not new work.
2. Desktop signing (optional - skip to ship unsigned installers). Buy an Apple Developer account, export your Developer ID certificate, and add the six `APPLE_*` secrets listed above. Windows signing is a later follow-up (see the checklist).
3. Mobile (optional - skip until you want store apps). Run the one-time Capacitor init above and commit the generated projects, then add the Android and iOS secrets and set the repo variable `MOBILE_RELEASE` to `true`.
4. Do a first release with Part B to confirm the `release` workflow runs end to end.

### Part B: every release (repeat each time)

1. Land the work on `main` through PRs - the gates (services, awh-gate, docs-prerender-gate) must be green. It deploys to the web and service surface automatically; nothing else to do for web, PWA, or desktop content, since they all load the live site root.
2. Bump the platform version and the installer versions to the number you are about to tag:
   - `VERSION` at the repo root (the single platform version; the pre-commit `cyberos-payload-build` hook rebuilds `dist/cyberos` so the init payload always matches).
   - `version` in `apps/desktop/src-tauri/tauri.conf.json` and `version` in `apps/web/package.json` (what the installers report).
3. Record the release in `CHANGELOG.md` (repo level; per-module history lives in each module's `CHANGELOG.md`, rendered to the site's changelog pages).
4. Commit, tag, and push both:

       git commit -am "release vX.Y.Z"
       git tag vX.Y.Z
       git push origin main vX.Y.Z

5. Watch `deploy.yml` go green (images pushed, VPS rolled), then the `release` workflow build the native installers and open a draft GitHub Release with them attached (plus the Android bundle and a TestFlight upload once `MOBILE_RELEASE=true`).
6. Edit the draft release notes and publish. Hand out the installer links; with mobile on, the iOS build is in TestFlight and the Android `.aab` is the release artifact to upload to Play.
7. Distribute the payload to consumer repos: each project updates with `init.sh --check` (notify) and re-running `init.sh` (apply) - or from the desktop app's CyberOS Ops tab. Fleet-wide: `tools/cyberos-init/rollout.sh`.

## Docs are part of every release

The website is generated from the markdown single source of truth (FR-DOCS-002): module docs at `modules/<m>/docs/` or `services/<s>/docs/`, global docs under `docs/`. Three mechanisms keep it fresh, in order of defense:

1. Pre-commit `docs-site-build` (local, automatic).
2. `docs-prerender-gate` (CI, every PR touching doc sources): rebuilds the whole site and fails if it does not build clean.
3. Manual: `bash tools/docs-site/build.sh` (or `--docs` for the doctrine pages only).

Nothing generated is committed: the site renders into gitignored `dist/website`, so there is no generated HTML to edit by hand.

Hosting: a Vercel project connected to this repo builds the site on every push to `main` via `vercel.json` (`bash tools/docs-site/build.sh` + `tools/docs-site/stage-vercel.mjs`, output `.vercel-out`) and serves it at `cyberos.cyberskill.world/docs` (the domain root redirects to `/docs/`). The old hand-authored wiki deployment is retired.

## GHCR troubleshooting

A `403 Forbidden` pushing an image means that GHCR package exists without this repo granted write access (packages created by this workflow auto-link via the `org.opencontainers.image.source` label; older ones may not). Fix in GitHub: org -> Packages -> the failing package -> Package settings -> Manage Actions access -> add this repo with the Write role. Alternatively delete the stale package and let the workflow recreate it linked.

## Related runbooks

`go-live-guide.md` (first production bring-up), `cyberos-core-deploy.md` (VPS topology), `ci-and-local-checks.md` (what each gate runs), `local-dev-and-testing.md` (dev stack), `apps/desktop/README.md` (desktop first build), and `tools/cyberos-init/docs/index.md` (running CyberOS in other repos).
