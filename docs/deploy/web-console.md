# The React web console (apps/web)

A Vite + React + TypeScript single-page app: Google or password sign-in, a module dashboard, and live chat
over the same `/v1/auth` and `/v1/chat` APIs the legacy static console uses. It exists to replace the
hand-written `apps/console/*.html` pages with one modern, maintainable UI - no token box, no service-URL
fields, real components.

## Where it runs

It is served additively at `https://<domain>/web/`, alongside the existing console at `/`. Shipping it does
not touch the live team flow at `/`; people opt in by visiting `/web/`. Once it is proven, flip the site root
to it (see "Promoting to root" below).

API calls inside the app are origin-relative (`/v1/auth/*`, `/v1/chat/*`, `/healthz`), so behind Caddy they
resolve to the same services as everything else. The Vite `base` is `/web/`, so its own assets load from
`/web/assets/*`; Caddy's `@web` handle strip-prefixes `/web` and file-serves `apps/web/dist`.

## Build and deploy

The VPS never compiles. Like the static console, the **built** app is served from the git checkout, so the
build output `apps/web/dist/` is committed to the repo. After any change under `apps/web/src`:

    cd apps/web
    npm install        # first time only
    npm run build      # type-checks, then writes apps/web/dist/
    cd ../..
    git add apps/web/dist apps/web/src && git commit -m "web: <change>" && git push

The push deploys: the VPS git-pulls, and Caddy serves the new `dist/` from the `../../apps/web/dist:/srv/web`
volume mount (deploy/vps/docker-compose.p0.images.yml). No image rebuild is needed for a UI-only change.

## Local development

    cd apps/web
    npm run dev        # http://localhost:5173/web/

The dev server proxies `/v1/auth` and `/.well-known` to auth (`127.0.0.1:7700`) and `/v1/chat` (including the
websocket) to chat (`127.0.0.1:7720`) - see `vite.config.ts`. Start the local stack first
(`scripts/dev/dev-up.sh`).

## Promoting to root

When the React console should become the default at `/`, point the site root at it instead of the static
console: change the catch-all `handle` in `deploy/vps/Caddyfile.p0` to `root * /srv/web` with a SPA fallback,
and set the Vite `base` back to `/`. Keep `apps/console` available at a path (for example `/classic`) until
every page has a React equivalent. This is a deliberate, separate change - not part of the additive `/web/`
rollout.

## Scope today

Login (Google + password), the dashboard, and chat (channel list, live messages over the websocket, send,
create channel, reachability dot) are built. Threads, attachments, search, DMs, and presence detail exist in
the chat backend and the legacy client and are the next slices to port.
