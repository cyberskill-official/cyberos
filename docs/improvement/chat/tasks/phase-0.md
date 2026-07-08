# Phase 0 - safety rails (T-001..T-010)

Exit bar: rate limiter live, revoke closes sockets under 60 s, external probe green, staging exists.
Specs are static; status lives in BACKLOG.md. Report references: docs/strategy/
chat-enterprise-grade-plan-2026-07-06.md sections 2, 4F, 5I, 5Q.

## T-001 Rate limiting (subject + IP layers)

- C-refs C46 | P0/M | depends: none
- Touch: services/chat/src/lib.rs (middleware), new src/ratelimit.rs; deploy/vps/Caddyfile.p0.
- Spec: token-bucket per (subject, route-class) in-process (single replica today): send 10/10s burst 20,
  search 5/10s, upload 5/min, channel-create 10/h; 429 + Retry-After; counters exported for C69. Caddy
  adds a per-IP cap on /v1/chat/* as the outer layer. Config via env with sane defaults; document in
  deploy runbook.
- Accept: unit test proves 429 after burst then recovery; smoke sends a burst and sees 429; clippy/test
  green; limiter metrics visible on /metrics (after T-008, otherwise counter in logs).
- Review: check the classes/limits table makes daily use comfortable (no false positives typing fast).

## T-002 JWT hardening

- C-refs C47, C48 | P0/S | depends: none
- Touch: services/chat/src/auth.rs.
- Spec: make audience validation unconditional (config value required at boot, boot fails without it);
  pin expected issuer; on token with unknown kid, trigger one bounded JWKS refetch (with cooldown) before
  rejecting, so key rotation does not require a restart.
- Accept: unit tests for wrong-aud reject, wrong-iss reject, unknown-kid-then-refetch accept; existing
  auth tests stay green; boot fails clearly when CHAT_JWT_AUD unset.
- Review: confirm env names + values land in deploy/vps/.env docs and the p0 compose.

## T-003 Account-wide socket kill on revoke

- C-refs C49 | P0/S | depends: none
- Touch: services/chat/src/realtime.rs, notify.rs; services/auth revoke path (only if an event hook is
  missing - prefer consuming what AUTH already emits).
- Spec: extend the existing Kicked control-event mechanism from per-channel to per-subject: on subject
  revoke/tenant suspend, close every live socket of that subject within 60 s. Poll fallback acceptable
  now (periodic membership/session check in the ws loop) if no cross-service signal exists; document.
- Accept: integration test: open ws, revoke subject via AUTH admin API, socket closes < 60 s; smoke added.
- Review: verify the revoke path used is the one the console actually calls.

## T-004 BYTEA attachment closeout

- C-refs C28 | P0/S | depends: none
- Touch: services/chat/src/storage.rs, attachments.rs; one migration if a backfill flag helps.
- Spec: one-shot migration tool/route (admin, idempotent) moving residual db-backend rows to fs; prod
  config rejects CHAT_ATTACHMENT_STORE=db (boot warning in dev). Keep db backend for tests only.
- Accept: running the mover twice is a no-op; download of a migrated attachment byte-identical (hash
  compare in test); prod compose asserts fs.
- Review: confirm row counts moved on prod during the deploy window.

## T-005 Edit-history revisions table

- C-refs C29 | P0/S | depends: none
- Touch: migration 00NN_chat_message_revisions.sql; services/chat/src/messages.rs.
- Spec: chat_message_revisions(message_id, rev, body, edited_at, editor_subject_id), RLS mirroring
  chat_messages; write previous body on every edit in the same tx; no read API yet (viewer is C118/T-048).
- Accept: edit twice -> two revision rows with correct bodies; delete keeps revisions (retention handles
  purge later); smoke extended; migration applies clean.
- Review: sanity-check storage growth expectation (revisions only on edit, not per message).

## T-006 WS heartbeat + idle reaper

- C-refs C16 | P0/S | depends: none
- Spec: server ping every 30 s, close on missed pong deadline (2 misses); reap sockets idle past
  deadline; presence updated on reap. Client answers pongs (browser does natively; verify through proxy).
- Accept: test with a silent client -> closed within ~90 s and presence flips offline; no regressions in
  reconnect smoke.
- Review: watch prod ws gauge for sawtooth after deploy (stale sockets finally dying).

## T-007 External synthetic probe

- C-refs C74 | P0/S | depends: none
- Touch: new scripts/probe/chat-probe.(py|sh) + a scheduler outside the VPS (GitHub Actions cron is
  acceptable v1), secrets via repo secrets.
- Spec: every 1-5 min: password/refresh grant on a probe user, open ws, send to a probe channel, await
  echo, report latency; alert (email + chat webhook once T-054, email now) after 2 consecutive failures.
- Accept: probe green against prod; forced-failure drill alerts; runbook snippet for muting during
  maintenance.
- Review: confirm probe user is a locked-down dedicated tenant, not cyberskill.

## T-008 Metrics baseline + core alerts

- C-refs C69 | P0/M | depends: none
- Touch: services/chat (metrics middleware + /metrics), deploy/vps compose additions (prometheus,
  grafana, alertmanager, node exporter), Caddy route guard for dashboards.
- Spec: RED per route, ws connection gauge, fan-out latency histogram, limiter hits (T-001), push intent
  counts; alert rules: send 5xx rate, ws connect failure rate, DB pool saturation, disk. Start with the
  R29 layout from the platform plan; keep dashboards in-repo as JSON.
- Accept: /metrics scrapes; Grafana shows live traffic; test alert fires to email; compose healthy.
- Review: pick the 3 panels you want on your phone; we pin them.

## T-009 Staging compose profile + seeded tenant

- C-refs C139 | P0/S | depends: none
- Touch: deploy/vps/docker-compose.staging.yml (or profile), Caddyfile entry (staging subdomain), seed
  script reusing scripts/dev seeding.
- Spec: same images as prod, separate DB (Supabase branch/second project or local PG container), seeded
  demo tenant + probe user; e2e suites (T-021, T-046) point here.
- Accept: staging URL serves login + chat round trip; documented bring-up/teardown; zero shared state
  with prod.
- Review: DNS + TLS choice for the staging subdomain.

## T-010 Go-live checklist doc

- C-refs C147 | P0/S | depends: none
- Touch: docs/deploy/chat-go-live-checklist.md.
- Spec: table of every P0/P1 task in this backlog with owner + status + evidence link (ledger anchor);
  the phase exit bars from report section 6 as gate rows; update as part of each task's ledger step.
- Accept: file exists, linked from README here and from the report; CI does not gate on it (human doc).
- Review: this is your go/no-go instrument; shape it how you want to read it.
