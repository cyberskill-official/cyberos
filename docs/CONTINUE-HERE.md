# Continue here - CyberOS state and next steps (handoff 2026-06-30)

A self-contained brief so any session can pick up exactly where this left off.

## Where the project is

CyberOS is LIVE in production at https://os.cyberskill.world for the CyberSkill team. The P0 stack (cyberos-auth, cyberos-chat, cyberos-eval, Caddy) runs on a Vultr VPS against Supabase Postgres, with GHCR images and GitHub auto-deploy on push. Google Workspace sign-in, the dashboard, and team chat are live and in daily use.

The build and deploy loop is the Mac-gate loop: author on the repo, gate on the Mac (`cargo fmt --all`, `cargo clippy -p <crate> -- -D warnings`, `cargo test -p <crate>`) via Desktop Commander, commit, then push - the pre-push hook re-runs the full gate and builds the auth and chat images, and GitHub auto-deploys (git pull, migrate, compose up). See `docs/deploy/web-and-desktop-deploy.md` for the full web + desktop deploy story.

## The 2026-06-30 wave (shipped)

Two tracks, plan-first then build, all gated on the Mac and pushed:

- Chat client (live): DMs finished (presence, recent-activity sort), attachments polished (staging preview, drag/drop, paste, size guard), emoji reactions (live strip + picker), and inline Vietnamese/English translation.
- BRAIN and EVAL backend (pushed): the consent gate verified already-correct; the ai-gateway `POST /v1/embeddings` route; the EVAL-002 rubric built from the three signed employment documents (migration verified); the memory brain migrations enabled on deploy; and the EVAL-001 governance layer finished (retention sweeper, status endpoint, metrics).

What is live vs dormant right now:

- Reactions and the rest of chat are fully live.
- The embeddings route and chat translation ship but return a clean error until the ai-gateway is deployed (it is not in the P0 stack yet).
- Eval and memory code is in git and the memory migrations applied, but the running eval service stays on its current image until a `BUILD_EVAL=1` push. That is intentional: the evaluation half is disabled by default and waiting on Vietnamese counsel to clear the monitoring-and-evaluation notice (docs/legal/data-monitoring-and-evaluation-notice.md). The three signed employment contracts live in docs/legal/ but are gitignored (kept out of version control).

## Next steps (in priority order)

CURRENT FOCUS (2026-07-02): the chat module - make it full-featured, then overhaul the UI/UX. Full audit at
docs/feature-requests/chat/CHAT-AUDIT-2026-07-02.md. Operator chose the sequence: (1) split Chat.tsx into
components; (2) build ALL four feature clusters; (3) then the UI/UX overhaul.

DONE + LIVE so far:
- (1) Chat.tsx split into components (task #159, commit 020184a).
- (2a) get-notified cluster (commit 11575fd, deployed to os.cyberskill.world): the per-user notification
  socket (services/chat/src/notify.rs Notifier + NotifyEvent + fanout + GET /v1/chat/notify), a one-query
  GET /v1/chat/unread summary (unread + unread-mentions per channel), and @-mentions (migration 0009
  chat_mentions, PostMessage.mentions validated to members, mention flag on the notify event). Client:
  useNotifySocket for live unread badges, desktop notifications when the tab is hidden (permission asked on
  first channel select), a "(N) CyberOS Chat" tab title, a Composer @-mention autocomplete, and a distinct
  Sidebar mention badge. This closed the audit's top gap (the ws was per-channel only).

- (2b) richer-messages cluster (four commits, one deploy): rich rendering (apps/web/src/lib/richtext.ts
  hand-written parser -> React nodes, XSS-safe by construction; code blocks/inline code/bold/italic/strike/
  links/autolink/quotes/lists + the in-body @-mention highlight deferred from 2a; contract pinned by
  apps/web/scripts/richtext-smoke.ts, run with plain `node`); the full emoji picker
  (components/EmojiPicker.tsx over unicode-emoji-json as a lazy chunk - search, categories, persisted skin
  tone, recents; opened from the reaction strip "+" and a composer emoji button); attachment storage
  (services/chat/src/storage.rs db|fs seam - prod compose mounts a chat-attachments volume, fs store, 50 MB
  cap; migration 0010 chat_message_attachments; raw-body POST /v1/chat/channels/:id/uploads; GET
  /v1/chat/config caps; message delete now purges attachment rows + fs bytes; found+fixed: axum's default
  2 MB body limit had silently capped the "5 MB" base64 route); and the multi-file client (staged File[],
  tiled attachment groups with server-folded metadata, image lightbox, config-driven caps, editable
  captions on attachment messages). Reaction emoji cap now 64 bytes for toned ZWJ sequences.

- (2c) find-and-organize cluster (three commits, one deploy): GET /v1/chat/search (tenant-wide over the
  caller's channels incl. DMs, trigram-backed) + list ?around= jump windows; migration 0011 (topic,
  visibility private|public - existing rows private, archived_at) with PATCH channel, browse + self-join,
  self-leave, PATCH member role, post-blocked-on-archived. Client: header search + Ctrl/Cmd+K is now
  global with clickable results (channel chip) -> jump-to-message (around window, scroll + flash, sticky
  "Jump to latest" pill), scroll-up load-older pagination (audit #11), ChannelSettings modal
  (rename/topic/visibility/roster with roles/leave/archive), BrowseChannels modal, visibility choice on
  create, archived sidebar section + read-only note. FOUND+FIXED: fetched pages rendered newest-first -
  reloaded channels and thread panels showed history REVERSED; all fetches now sort ascending.

- (2d) AI-native cluster (two commits, one deploy): services/chat/src/ai.rs on the translate.rs posture
  (server-side gateway calls, member-gated, graceful 502, audit counts only) - POST channels/:id/ai/
  summarize + /ai/actions (chat.smart, bullet markdown, bilingual prompts, 24 KB transcript cap) and
  /ai/replies (chat.fast, 3 line-parsed suggestions); speaker labels from a sanitized client-supplied
  name map. Client: header sparkle -> Assistant panel (Catch me up auto-runs, Action items, refresh,
  unavailable note), composer sparkle -> reply chips that fill the draft (never auto-send). NOTE: these
  return "AI is unavailable" in prod until the VPS is resized + the llm compose profile enabled (same
  gate as translation - docs/deploy/ai-gateway-and-embeddings.md); embeddings-only today.

REMAINING (task #182), still in the same sequence:
- (3) UI/UX overhaul: tokenize the umber/ochre palette by role + fix AA contrast, real type + spacing scales,
  redesign message rows/composer/empty-states/the hover action bar, in one hand-written styles.css. While
  here, bump the PWA service-worker cache version per build so deploys reach open clients on reload. No i18n
  exists yet on a bilingual team.

Each slice: gate on the Mac (fmt + clippy + tests; web tsc + vite; verify any new migration on a throwaway
DB), commit, push (deploys). Pushing rebuilds the auth + chat + ai-gateway images and applies new chat
migrations on deploy - treat the push as a confirm-with-the-operator action.

Deploy and infra follow-ups (still valid, lower priority than the chat focus):

1. DONE (2026-07-02): the ai-gateway + bge-m3 embed sidecar are in the prod compose as a best-effort AI
   group - embeddings live, gateway internal-only, chat wired via `AI_GATEWAY_URL`. Chat translation stays
   gracefully dormant until the VPS is resized to 8 GB and the `llm` compose profile is enabled (ollama +
   qwen2.5:3b-instruct) - exact steps in `docs/deploy/ai-gateway-and-embeddings.md`.
2. Flip translation on: resize the VPS, set `COMPOSE_PROFILES=llm` in `.env.p0`, redeploy (runbook above).
3. When counsel clears the notice, turn the evaluation half on: publish the monitoring notice, record acknowledgments, set retention policies, author and publish a rubric version, then deploy the eval service with `BUILD_EVAL=1` and set `CHAT_AUDIT_DATABASE_URL` + `CAPTURE_ENABLED`. Keep it human-in-the-loop and disabled-by-default until every governance precondition holds.
4. Stand up the memory recall service container (only its migrations are enabled today, not a running service; the embedding dependency it needs is now live).
5. Optional polish: split the 947-line apps/web/src/pages/Chat.tsx into components (deferred); the MCP and OBS modules per their build plans if still in scope.

## Pointers

- Deploy (web + desktop): `docs/deploy/web-and-desktop-deploy.md`.
- Roadmap tracker (browser): `docs/roadmap.html`.
- BRAIN/EVAL plan: `docs/strategy/cyberos-brain-evaluation-plan.md`; FR specs under `docs/feature-requests/`.
- Local dev + the dev Postgres/Redis: `docs/deploy/local-dev-and-testing.md` (services/dev docker compose).
- Agent memory (next Claude session): the space memory dir, especially `cyberos-mac-gate-loop.md` (the build loop + wave state), `cyberos-react-console.md`, and `cyberos-brain-evaluation-plan.md`.

## Working rules

- Gate before pushing: the pre-push hook runs `fmt --all --check`, `clippy --workspace -- -D warnings`, and `test --workspace`. Document every `pub` item; keep imports clean. The flaky ai-gateway Redis integration tests are `#[ignore]`d.
- Pushing deploys to production, so treat it as a confirm-with-the-operator action.
- Cloud-provider API keys are deferred; local inference is LM Studio (:1234) or Ollama (:11434) through the ai-gateway RouterBackend. Never author or enter secrets.
- The evaluation half stays governance-first: disabled by default, human-approved, counsel-gated.
