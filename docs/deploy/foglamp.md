# Foglamp (AI tracing) — handoff

Status as of 2026-07-24. **Local / Vite-dev instrumentation is done and on `main`.** Production ingest is **not** wired yet — resume from § Production when ready.

Related PRs:

- [#135](https://github.com/cyberskill-official/cyberos/pull/135) — install Vercel AI SDK (`ai@7`) in `apps/web`
- [#136](https://github.com/cyberskill-official/cyberos/pull/136) — Foglamp + agents + Vite middleware + HUD

External docs: [Instrument with an AI agent](https://docs.foglamp.dev/ai-instrument.md), [HUD](https://docs.foglamp.dev/sdk/hud), index [llms.txt](https://docs.foglamp.dev/llms.txt).

---

## What works today (local)

| Piece | Where |
| --- | --- |
| Packages | `apps/web`: `ai@^7`, `foglamp@^0.8` |
| Collector | `apps/web/server/fog.ts` — `foglamp({ hud: true })` (lazy) |
| Gateway model | `apps/web/server/gateway-model.ts` — `LanguageModelV3` → CyberOS `POST {AI_GATEWAY_URL}/v1/chat` (`alias` + `messages` + `x-tenant-id`) |
| Agents | `apps/web/server/agents.ts` — `generateText` + `fog.integration(...)` (AI SDK **v7** path) |
| Dev intercept | `apps/web/vite-plugin-foglamp-ai.ts` + `server/ai-dev-middleware.ts` — Vite `configureServer` handles AI routes **before** the chat proxy |
| HUD | `<FoglampHUD />` in `apps/web/src/main.tsx` (inert without local broker) |

### Agent mapping (static names only)

| Agent name | Route / action | Session | Customer |
| --- | --- | --- | --- |
| `chat-summarizer` | `POST /v1/chat/channels/:id/ai/summarize` | `sessionId` = channel id | `customer.id` = JWT `tenant_id` |
| `chat-action-items` | `.../ai/actions` | channel id | tenant id |
| `chat-reply-suggest` | `.../ai/replies` | channel id | tenant id |
| `chat-translator` | `POST /v1/chat/translate` | _(none — one-off)_ | tenant id |

Dynamic ids go in `sessionId` / `customer.id` / `metadata` — never in `agentName`.

### Local env

Prefer **repo root** `.env` (Vite `envDir: "../.."` from `apps/web`):

```bash
FOGLAMP_API_KEY=fl_…          # npx foglamp login
AI_GATEWAY_URL=http://127.0.0.1:8080
# optional:
# CHAT_URL=http://127.0.0.1:7720
# FOGLAMP_INGEST_URL=…        # only if self-hosting ingest
```

### How to exercise a trace locally

1. Auth `:7700`, chat `:7720`, ai-gateway on `AI_GATEWAY_URL`.
2. `cd apps/web && npm run dev` → http://localhost:5173/
3. Sign in → channel with messages → **AI panel** (Catch me up / Action items), **reply chips**, or **translate**.
4. Watch the floating **Foglamp HUD** and [foglamp.dev](https://foglamp.dev) → Overview / Traces.

If AI returns 502 / “unavailable”, fix gateway/`AI_GATEWAY_URL` first — no model call means no span.

---

## Why production does not emit Foglamp traces yet

```
Local:   browser → Vite middleware → AI SDK + Foglamp → ai-gateway
Prod:    browser → Caddy → Rust chat → ai-gateway
```

Production never enters `generateText` / `fog.integration`. Foglamp only observes **Vercel AI SDK** calls. Rust chat’s `call_gateway` path is invisible to Foglamp. HUD stays a no-op in prod (expected).

CyberOS already has broader OBS / LangSmith on the gateway plane (`services/obs-*`, ai-gateway LangSmith) — Foglamp is a separate, AI-SDK-specific product surface.

---

## Production — resume checklist (not implemented)

Pick one architecture, then implement:

### Option A — Node sidecar / BFF (smallest new surface)

- Deploy a Node service that reuses `apps/web/server/*` (agents + gateway model + `foglamp()` **without** requiring Vite).
- Route prod AI HTTP (`/v1/chat/channels/*/ai/*`, `/v1/chat/translate`) to that service (Caddy), **or** have Rust chat call it for the LLM step only.
- Env on that process: `FOGLAMP_API_KEY`, `AI_GATEWAY_URL`, chat/auth URLs as needed.
- Flush: long-lived Node = default interval; serverless = Foglamp auto / `waitUntil`.

### Option B — Rust keeps auth + transcript; Node does LLM only

- Chat loads membership + messages, then HTTP-calls a Node worker that runs `generateText` + Foglamp against ai-gateway.
- Keeps secrets and Foglamp off the SPA; clearest separation.

### Option C — Do not use Foglamp in prod

- Keep LangSmith / OTel for production AI; leave Foglamp as **dev-only** (current behavior). Document that choice and stop here.

### Explicit non-goals for a production pass

- Do not invent demo/smoke endpoints “just to get a first prod trace”.
- Do not put provider keys or `FOGLAMP_API_KEY` in the browser bundle.
- Do not put dynamic ids in `agentName` / `workflowName` / `traceName`.
- Prefer reusing the existing agent names above so dashboards stay continuous with local.

### Suggested verification when shipped

1. Set `FOGLAMP_API_KEY` on the prod Node process (secret store / compose env — never commit).
2. Hit Catch me up / translate on production (or staging).
3. Confirm spans on Foglamp dashboard with the agent names above and `customer.id` = tenant.
4. Confirm HUD still inert on the public SPA.

---

## Code map (quick)

```
apps/web/
  package.json                 # ai, foglamp
  vite.config.ts               # envDir + foglampAiPlugin()
  vite-plugin-foglamp-ai.ts    # loads .env, configureServer
  src/main.tsx                 # <FoglampHUD />
  server/
    fog.ts                     # collector
    gateway-model.ts           # ai-gateway LanguageModelV3
    agents.ts                  # generateText + fog.integration
    ai-dev-middleware.ts       # Vite-only route handlers
```

When production work starts, either extract `server/` into a deployable package/service or import it from a new `services/` / `apps/` Node entry that is not Vite-bound.
