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

1. Stand up the ai-gateway and a bge-m3 embedding sidecar in the prod compose. This activates three already-built things at once: the embeddings route, chat translation, and (with capture on) brain ingest.
2. When counsel clears the notice, turn the evaluation half on: publish the monitoring notice, record acknowledgments, set retention policies, author and publish a rubric version, then deploy the eval service with `BUILD_EVAL=1` and set `CHAT_AUDIT_DATABASE_URL` + `CAPTURE_ENABLED`. Keep it human-in-the-loop and disabled-by-default until every governance precondition holds.
3. Stand up the memory recall service container (only its migrations are enabled today, not a running service).
4. Optional polish: split the 947-line apps/web/src/pages/Chat.tsx into components (deferred); the MCP and OBS modules per their build plans if still in scope.

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
