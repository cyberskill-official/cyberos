# Local auth + chat, one command

Bring up the auth service, the chat service, and the operator console locally, then sign in and test.

## Prerequisites

- The dev Postgres container `cyberos-postgres` is running (it holds the `cyberos` and `cyberos_chat` databases). Redis (`cyberos-redis`) running is fine but not required.
- A Rust toolchain, `docker`, `python3`, and `curl`. `htpasswd` (from the `apache2-utils` / `httpd-tools` package, and preinstalled on macOS) is used to hash the demo password; if it is missing, the seed falls back to python `bcrypt`.

## Use

```
scripts/dev/dev-up.sh      # build, start everything, seed the demo user, print the URL + login
# ... open the URL, test ...
scripts/dev/dev-down.sh    # stop everything
```

`dev-up.sh` prints the console URL and the demo sign-in. By default:

- Console: http://127.0.0.1:8090/app.html
- Workspace `cyberskill`, handle `@stephen`, password `CyberOS-Demo-2026!`

## What it wires

- Auth runs on :7700 with `AUTH_DEV_CORS=1` so the browser can call the token endpoint cross-origin.
- Chat runs on :7720 with `CHAT_DEV_CORS=1` and verifies access tokens against the auth JWKS URL (`CHAT_AUTH_JWKS_URL`), so there is no key file to manage.
- The console (`apps/console`) is served as static files. `app.html` signs in against auth (password grant), stores the token, and opens the Chat module, which auto-connects to chat. The access token lasts an hour; the client refreshes it from the stored refresh token, so a sign-in holds for the session.

## Overrides

Copy `dev.env.example` to `dev.env` (gitignored) to change ports, the demo password, or service URLs. Every value has a default, so `dev.env` is optional.

These flags and the seeded demo user are for local development only. In production, Caddy serves the console and proxies auth and chat under one origin, so the dev CORS flags stay off.

## MCP Registry tile (optional)

The dashboard's MCP Registry tile needs the MCP gateway and a module running. It is heavier and optional, so it has its own script:

```
scripts/dev/mcp-up.sh      # MCP Registry tile: the MCP gateway (:7730) + reference module, then lists the tools
scripts/dev/ai-up.sh       # Assistant + AI Ops tiles: the AI gateway (:8080) with the tenant config
```

`dev-down.sh` stops them along with everything else. The MCP gateway runs on 7730 so it does not collide with the console on 8090. The AI Ops tile reads the gateway's loaded policy and works with no model running; the Assistant tile needs a local model (LM Studio or Ollama) to answer.
