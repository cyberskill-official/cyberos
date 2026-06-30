# The React web console (apps/web)

A Vite + React + TypeScript single-page app: Google or password sign-in, a module dashboard, and live chat
over the same `/v1/auth` and `/v1/chat` APIs the legacy static console uses. It replaces the hand-written
`apps/console/*.html` pages with one modern UI - no token box, no service-URL fields, real components.

## Where it runs

Served additively at `https://<domain>/web/`, alongside the existing console at `/`. Shipping it does not
touch the live team flow at `/`; people opt in by visiting `/web/`.

The source lives in `apps/web/src`, but it **builds into `apps/console/web/`** (Vite `outDir`), so the
existing `/srv/console` Caddy mount serves it - there is no separate volume. This matters: the repo's global
`.gitignore` has a `dist/` rule, so an `apps/web/dist` build was silently never committed and the VPS served
an empty folder (a 404). Building into `apps/console/web` (not named `dist`) keeps the output tracked and
served by a mount that already works.

Caddy's `@web` handle strip-prefixes `/web` and file-serves `/srv/console/web`; bare `/web` 308-redirects to
`/web/`. API calls inside the app are origin-relative (`/v1/auth/*`, `/v1/chat/*`, `/healthz`), so they hit
the same services.

## Build and deploy

The VPS never compiles; the built app is served from the git checkout. After any change under
`apps/web/src`:

    cd apps/web
    npm install        # first time only
    npm run build      # type-checks, then writes apps/console/web/ (emptyOutDir clears it first)
    cd ../..
    git add apps/console/web apps/web/src && git commit -m "web: <change>" && git push

The push auto-deploys (apps/console is in deploy.yml's paths): the VPS git-pulls and Caddy serves the new
`apps/console/web` from the `/srv/console` mount. No image rebuild for a UI-only change, and the pre-push
hook skips the Rust checks + image build because nothing under `services/` changed.

## Local development

    cd apps/web
    npm run dev        # http://localhost:5173/web/

The dev server proxies `/v1/auth` and `/.well-known` to auth (`127.0.0.1:7700`) and `/v1/chat` (including the
websocket) to chat (`127.0.0.1:7720`) - see `vite.config.ts`. Start the local stack first
(`scripts/dev/dev-up.sh`).

## Promoting to root

When the React console should become the default at `/`, point the catch-all `handle` in
`deploy/vps/Caddyfile.p0` at `/srv/console/web` with a SPA fallback and set the Vite `base` back to `/`. Keep
the legacy pages reachable at a path (for example `/classic`) until every page has a React equivalent. This
is a deliberate, separate change - not part of the additive `/web/` rollout.

## Scope

Login (Google + password), the dashboard, and chat (channels, DMs, group channels, attachments, threads,
search, presence, typing, edit/delete) are built. Read receipts and the WebRTC call UI are deferred.
