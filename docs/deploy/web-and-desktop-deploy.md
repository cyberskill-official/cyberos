# Deploying CyberOS to web and desktop

CyberOS ships to two surfaces from a single codebase and a single push:

- Web: the React console at https://os.cyberskill.world/web/ (installable as a PWA).
- Desktop: a Tauri app (apps/desktop) that loads that same live URL in a native window.

The desktop app is a thin native shell over the web app, so for everyday changes there is no separate desktop deploy: every web deploy updates the desktop app on its next launch. You only rebuild the desktop binary when you want to ship a new installable (a new version, a new icon, or a Tauri-config change), not when you change a feature.

## One push, both surfaces

The flow is the same for any change:

1. Author the change. Web client lives in apps/web; backend services in services/.
2. Run the gate, then push to main. The pre-push hook (.githooks/pre-push) runs the full gate before the push: `cargo fmt --all --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, then it builds and pushes the cyberos-auth and cyberos-chat images to GHCR (it skips the eval image unless you set `BUILD_EVAL=1`). A failed gate blocks the push, so nothing broken reaches the server.
3. GitHub auto-deploys. Pushing to main triggers the deploy workflow, which on the Vultr VPS runs: `git pull`, then `deploy/vps/migrate.sh` (applies any new migrations to Supabase), then `docker compose up -d` (rolls the services to the new images).

Skips for faster pushes: `SKIP_LOCAL_CHECKS=1` (skip the whole gate), `SKIP_IMAGE_BUILD=1` (skip only the image build). Use these only when you know the change does not need them - for example a docs-only or deploy-config-only change.

## Web

The web console is apps/web (Vite + React + TypeScript, base path `/web/`). It builds into apps/console/web, which is committed and served by Caddy as static files - there is no web image. So a web-only change reaches production through `git pull` alone, with no container rebuild.

To ship a web change:

1. Build the bundle: `cd apps/web && npm run build` (this is `tsc --noEmit && vite build`; output goes to apps/console/web).
2. Commit apps/web and apps/console, then push.
3. Verify after the deploy: fetch https://os.cyberskill.world/web/ and confirm the referenced `index-<hash>.js` changed to the hash your build produced.

PWA install: the console is installable from the browser (manifest.webmanifest + a service worker + the brand icons). Users install it from the browser's install affordance; no store submission is involved.

## Desktop

apps/desktop is a Tauri 2 app whose window loads https://os.cyberskill.world/web/. Because it points at the live web app, the desktop app always matches whatever is deployed to the web - you do not redeploy the desktop app to ship a feature. Two settings make the webview behave: a standard Safari `userAgent` in src-tauri/tauri.conf.json (so Google accepts OAuth inside the webview), and camera plus microphone usage strings in src-tauri/Info.plist (for the WebRTC calls).

You rebuild and redistribute the desktop binary only to ship a new installable. Build per target OS (there is no cross-compile):

1. Prerequisites on the build machine: Rust (stable), the Tauri CLI v2 (`cargo install tauri-cli --version "^2"`), and the OS toolchain (macOS: Xcode Command Line Tools; Windows: MSVC build tools + WebView2).
2. One-time, generate the app icons from the brand logo: `cd apps/desktop/src-tauri && cargo tauri icon ~/Projects/CyberSkill/design-system/packages/brand-assets/assets/logo-mark.png`.
3. Build: `cd apps/desktop/src-tauri && cargo tauri build` produces a `.dmg` on macOS or `.msi`/`.exe` on Windows under target/release/bundle/.
4. Distribute the bundle. For Gatekeeper-free installs on macOS, sign and notarize with an Apple Developer ID; an unsigned build still runs after a right-click then Open.

See apps/desktop/README.md for the full per-OS notes and the first-build test points (Google sign-in through the webview, the call camera/mic prompt, and that chat/profile/attachments behave exactly as the web app).

## Services, migrations, and the eval/memory note

The deployed P0 stack is cyberos-auth, cyberos-chat, cyberos-eval, and Caddy, all on one Supabase Postgres. `deploy/vps/migrate.sh` applies the migrations of the services in `MIGRATE_SERVICES` (auth, chat, eval, memory); auth and chat are baselined (recorded, not re-run) on their first pass, and any new migration file added afterward is applied on the next deploy.

Two deploy facts worth keeping in mind:

- The hook builds only the auth and chat images by default. Eval and memory code reaches git and their migrations apply, but the running eval service stays on its current image until you push with `BUILD_EVAL=1`. That is intentional while the evaluation half is disabled by default and waiting on counsel sign-off.
- The ai-gateway is not in the P0 stack yet. Features that call it (the embeddings route, chat translation) ship but return a clean error until the gateway is deployed.

## Configuration and secrets

Real values live in `.env.p0` on the VPS (never committed); `deploy/vps/.env.p0.example` is the template. It carries the Supabase connection URLs, the Google OAuth client, `CHAT_AUDIT_DATABASE_URL` (the chat-to-brain audit link), and `CAPTURE_ENABLED` (off by default). Set `CHAT_AUDIT_DATABASE_URL` to the Supabase URL to enable audit chaining; turning capture on additionally requires that link.

## Rollback

Because the web bundle and the services come from git plus tagged images, rollback is a revert: `git revert` the bad commit and push (the deploy re-pulls and rolls), or redeploy a previous image tag. The web bundle reverts with the commit; migrations are forward-only, so a schema change is rolled back with a new migration, not by reverting.
