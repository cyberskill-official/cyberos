# AI gateway + bge-m3 embeddings in the P0 stack

How the AI group runs in production, what is live versus dormant, and the exact steps to turn the chat
LLM (translation) on. Companion to `deploy/vps/docker-compose.p0.images.yml` and
`deploy/vps/ai/tenants/org-cyberskill.yaml`.

## Shape

Three services join the P0 compose as a best-effort group - nothing in the core stack (auth, chat, caddy)
depends on them, and deploy.sh rolls them after the core so a failure there never blocks the team:

- `ai-gateway` (image `cyberos-ai-gateway`, internal :8080). Policy-routed AI front door. Reads the tenant
  policy from the bind mount `deploy/vps/ai/tenants` (hot-reloads edits in ~35s). INTERNAL ONLY: Caddy
  exposes no `/v1` route to it; chat calls it service-to-service via `AI_GATEWAY_URL=http://ai-gateway:8080`.
- `embed` (image `cyberos-embed-sidecar`, internal :7900). bge-m3 embeddings, CPU, fp32. Serves the OpenAI
  shape `POST /v1/embeddings` for the gateway's `local_openai` adapter plus the native `POST /embed` for the
  future memory service. Weights (~2.3 GB) download from Hugging Face on first boot into the named volume
  `embed-hf-cache` and survive redeploys; `GET /healthz` reports `warm: cold|warming|ready|failed`.
  Memory-capped (`mem_limit: 3g`, swap headroom to 6g) so a runaway load OOM-kills only this container.
- `ollama` (chat LLM for translation/assistant) - OFF by default behind the compose profile `llm`. The
  4 GB box cannot hold bge-m3 and a 3b chat model together; see flip-on below.

Alias routing in `deploy/vps/ai/tenants/org-cyberskill.yaml` (all providers local: no API keys, zero cost,
inherently ZDR, no region so the sg-1 pin holds):

- `embed.standard` -> `local-openai` adapter -> the embed sidecar (`OPENAI_COMPAT_ENDPOINT=http://embed:7900`).
- `chat.smart` / `chat.fast` -> `ollama` adapter -> `OLLAMA_ENDPOINT=http://ollama:11434`, model
  `qwen2.5:3b-instruct`.

## Live vs dormant after this ships

- LIVE: `POST /v1/embeddings` on the gateway (the brain's one embedding path, TASK-MEMORY-123).
- LIVE: health probes `https://os.cyberskill.world/status/ai` and `/status/embed`.
- DORMANT: chat translation - `/v1/chat/translate` returns its clean 502 because the ollama container is
  not running (the gateway maps a dead provider to a typed error; chat degrades gracefully, proven in the
  pre-ship smoke). Flips on with the llm profile below.
- DORMANT: brain ingest/capture - still governed by `CAPTURE_ENABLED=false` + the acknowledgment gate +
  counsel clearance (TASK-EVAL-001). This deploy only makes the embedding dependency available.

## Turn translation on (the VPS LLM), step by step

1. Resize the VPS. 3.3 GiB total RAM cannot hold bge-m3 (~3 GB RSS) plus a 3b chat model (~2.5 GB RSS).
   In the Vultr panel: Products -> the instance -> Settings -> Change Plan -> 8 GB. Expect a few minutes
   of downtime while it restarts; the stack comes back by itself (`restart: unless-stopped`).
2. Enable the llm profile in the deploy env:
   `ssh linuxuser@149.28.158.169`, then add this line to `~/cyberos/deploy/vps/.env.p0`:
   `COMPOSE_PROFILES=llm`
   The choice persists - every later deploy keeps ollama in the stack automatically.
3. Deploy: `bash ~/cyberos/deploy/vps/deploy.sh` (or push to main / run the deploy workflow). deploy.sh
   starts ollama and runs an idempotent `ollama pull qwen2.5:3b-instruct` (~2 GB, one-time; later deploys
   no-op). Override the model with `OLLAMA_CHAT_MODEL=...` but keep it identical to the alias map in
   `deploy/vps/ai/tenants/org-cyberskill.yaml`.
4. Verify: `curl https://os.cyberskill.world/status/ai` returns 200; in the chat UI, translate any
   message. First tokens on CPU take a few seconds; if it feels slow, that is the 2-vCPU trade until the
   plan grows.
5. Pin the ollama image. The compose references `ollama/ollama:latest` while the profile is off; after the
   first successful flip-on, pin it: `docker inspect --format '{{index .RepoDigests 0}}' ollama/ollama`
   and put that digest (or the version tag it maps to) into both compose files.

Swapping models later: edit the alias map in `deploy/vps/ai/tenants/org-cyberskill.yaml` (the gateway
hot-reloads it, no restart) and `ollama pull` the new id inside the container.

## Verifying embeddings from inside the network

The gateway is not publicly routed, so exercise it from the VPS:

```bash
ssh linuxuser@149.28.158.169
docker exec cyberos-p0-ai-gateway-1 curl -s -X POST \
  -H 'x-tenant-id: org:cyberskill' -H 'content-type: application/json' \
  -d '{"input":["xin chào"],"model":"bge-m3"}' http://127.0.0.1:8080/v1/embeddings | head -c 200
```

Expect a 1024-dim vector and `"model":"bge-m3"`. First call after a cold volume waits for the warmup
download; check `curl -s http://127.0.0.1:7900/healthz` inside the embed container for `warm: ready`.

## Memory budget on the 4 GB box (pre-resize)

auth + chat + caddy + eval-off ≈ well under 1 GB; embed capped at 3 GB with the 4.8 GB swapfile behind
it. That is deliberately snug: embedding calls are background work (ingest, search), so occasional swap
is acceptable where a user-facing LLM would not be.
