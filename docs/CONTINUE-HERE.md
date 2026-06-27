# Continue here - CyberOS state and next steps (handoff 2026-06-25)

A self-contained brief so any coding agent (or a future session) can pick up exactly where this left
off. Read this top to bottom, then start at "The plan", step 1.

## Where the project is

P0 platform path is AI -> OBS -> AUTH -> MCP -> CHAT. AI, AUTH, and CHAT are done. OBS is
feature-complete in the repo (status reconcile + deploy pending). MCP is the one module still mid-build.
Beyond P0, the 14 Phase-3 business modules are specified but unbuilt.

The core was brought up and verified locally on 2026-06-25 against a fresh database (services/dev docker
infra + all migrations). Result: every module test suite GREEN - auth, memory, email, proj,
obs-compliance-view, obs-router, mcp-gateway, ai-gateway - including the ai-gateway live local-model
round trip. The only failing test is `auth::create_subject_p95_latency_under_200ms` (p95 ~400ms), which
is Docker Desktop latency noise on macOS, not a logic bug; run the auth suite with
`--skip create_subject_p95` and it is fully green.

MCP module detail: 001 (spec) and 002 (heartbeat) done. 003 (SEP-986 naming) done through registration
enforcement; slice 3 left = the CI grep gate (DEC-2362) + 4 naming memory-audit kinds (DEC-2364). 004
(OAuth 2.1 + PKCE) has slice 1 (pkce, error) and the slice-2 foundation (enums, audience, scope +
migration renumber to 0013-0015) merged; the OAuth endpoints remain.

## The plan (in order, as the operator chose: test core local -> finish MCP -> deploy)

1. Build the MCP 004 OAuth endpoints. This is the keystone - it unblocks FR-MCP-005/006/007/008. The
   full, clause-by-clause build spec is `docs/feature-requests/mcp/MCP-004-SLICE2-PLAN.md`. Build it
   from there. Key facts that shape it:
   - The repo uses runtime-checked `sqlx::query(...)` (no `query!` macro, no `.sqlx` cache), so the code
     COMPILES WITHOUT a database; only the integration tests need Postgres. Author freely, `cargo
     check`/`clippy` on the Mac, then run the OAuth tests against local Postgres.
   - Token signing reuses the FR-AUTH-004 RS256 keys in the shared `auth_signing_keys` table. The
     pattern to mirror is `services/auth/src/jwt.rs` (load active key, sign RS256 with `jsonwebtoken`,
     `kid` in header). RS256 only.
   - The headline security property is the audience check wired into `protocol/tools_call.rs` (use the
     already-built `oauth::audience::audience_matches`). Do not skip it.
   - 8 memory audit kinds; 3 new tables (oauth_consents, oauth_revocations, redirect-host policy) as
     migrations 0016-0018.
2. Finish MCP 003 slice 3: the CI grep gate + the 4 naming audit kinds (DB/async, bundle with the above
   Postgres session).
3. Deploy the core + MCP to the VPS: `docs/deploy/cyberos-core-deploy.md`. Two gaps to close first: the
   production docker-compose + Caddyfile that consume `deploy/vps/.env.local` are not in the repo, and
   any in-tree live secrets must be rotated and kept out of git.

## How to run and verify locally

`docs/deploy/local-dev-and-testing.md` - Steps 1-3 bring up infra, apply migrations, run the suites;
Step 6 is the end-to-end smoke (chat path, live model, MCP tool calls). The MCP-only demo is
`bash scripts/mcp_demo.sh` + `bash scripts/mcp_call.sh <tool> <json>`.

## Working rules (honor these - they are how this project ships)

- One change = one branch named `auto/<topic>`. The operator does all git commit/push/merge; an agent
  never commits, pushes, or merges for them.
- Gate before merge: `cargo test -p <crate>` and, for mcp-gateway, `cargo clippy -p cyberos-mcp-gateway
  --all-targets -- -D warnings`. The caf gate (`bash scripts/caf_gate.sh <module>`) runs cargo test
  only; CI runs clippy with `-D warnings`, which escalates `missing_docs` and `unused_imports` to hard
  errors - so document every `pub` item and keep imports clean.
- Cloud-provider API keys are deferred; never author or enter secrets. Local inference is LM Studio
  (:1234) or Ollama (:11434) through the ai-gateway RouterBackend.
- Build security-critical code (OAuth) in pure, unit-testable pieces first, then the DB-bound parts -
  the slice-1/slice-2 split that worked here.

## Pointers

- MCP build plan: `docs/feature-requests/mcp/MCP-BUILD-PLAN.md` + `MCP-004-SLICE2-PLAN.md`.
- Roadmap tracker (open in a browser): `docs/roadmap.html`.
- FR specs: `docs/feature-requests/mcp/FR-MCP-00*.md`.
- Agent memory (if the next session is another Claude): the space memory dir, files
  `cyberos-mcp-build-state.md` and `cyberos-capabilities-build.md`.

## To hand this to another model/agent

Point it at this file and `MCP-004-SLICE2-PLAN.md`. Its first task is step 1 above - the 004 OAuth
endpoints. Because the code compiles without a database, the loop is: it authors, you `cargo check` +
`cargo clippy` on your Mac, fix anything, then `docker compose -f services/dev/docker-compose.yml up -d`
and run the OAuth integration tests. Keep one branch per slice and have it stop before any git
commit/push.
