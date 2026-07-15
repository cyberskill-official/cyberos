---
template: task@1
id: TASK-APP-007
title: "APP chat web client - first-party chat surface over cyberos-chat"
author: "@stephen"
department: engineering
status: superseded
superseded_by: the React console (apps/web) - the static console tiles shipped, then were replaced by the SPA
priority: p3
created_at: "2026-06-29T22:30:00+07:00"
ai_authorship: assisted
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: 2026-Q4
client_visible: false
module: app
new_files:
  - apps/console/chat.html
depends_on: [TASK-CHAT-101, TASK-AUTH-110, TASK-APP-001]
---

# Task

> Turn Your Will Into Real.

## Summary

CyberOS now has a first-party chat service (cyberos-chat, TASK-CHAT-101): channels, messages, threads, membership and roles, search, attachments, presence, typing, read receipts, and call signaling, all on the CyberOS identity, database, and audit chain. It has no human surface. This task adds one: a self-contained chat web client in the same console family as TASK-APP-001, styled with the CyberSkill Design System (Umber and Ochre, "Turn Your Will Into Real"). It is a static page that consumes only the chat service's existing HTTP and websocket surface and adds no backend. It signs the operator in with a CyberOS access token (TASK-AUTH-110), lists the channels they belong to, opens a channel, shows history, and from there is live: incoming messages, presence, and typing arrive over the websocket; sending posts a message; reading marks the channel read. It deploys as a static file behind the same Caddy front as the rest of the console.

## Problem

The chat server is feature-complete and proven by smoke tests, but every interaction so far has been a script minting a token and calling endpoints. There is nothing a person opens. Without a client, the chat module cannot be used, demoed, or reviewed by a human, and the live parts (websocket delivery, presence, typing, read state) cannot be seen working the way a user would see them.

The surface to build against already ships. cyberos-chat exposes REST for channels, messages, members, search, attachments, presence, and read/unread, and a websocket at `/v1/chat/ws` that carries message, presence, and typing events. Identity already ships too: TASK-AUTH-110 issues the access token the client presents. What is missing is the presentation layer that turns those into a usable chat window. Building it as a static page over the shipped surface keeps the no-new-backend rule of TASK-APP-001 and reuses the same deployment path.

## Proposed Solution

A self-contained page at `apps/console/chat.html`, styled with CDS tokens, served as a static file behind the existing Caddy front. User-visible behaviour: the operator pastes a CyberOS access token and connects; the sidebar lists their channels with unread badges; selecting a channel loads history and opens a websocket; new messages, presence changes, and typing indicators arrive live; typing in the composer relays a typing ping; sending posts a message; viewing a channel marks it read. The look is CDS, so the client is recognisably a CyberSkill surface.

### Section 1 - normative requirements (BCP-14)

1. The client MUST consume only the cyberos-chat surface that already ships (TASK-CHAT-101): the REST routes for channels, messages, members, presence, and read/unread, and the websocket at `/v1/chat/ws`. It MUST NOT introduce a new backend endpoint or server-side component; it is a front-end over the shipped surface.

2. The client MUST authenticate with a CyberOS access token (TASK-AUTH-110), presented as a bearer token on every REST call and as the `access_token` query parameter on the websocket. It MUST NOT define its own credential store, and the subject and tenant MUST come from the token, never from a separate field the user types.

3. The client MUST use CDS design tokens for layout, colour, and type (the Umber and Ochre palette), consistent with the operator console (TASK-APP-001). It MUST NOT introduce a second design language.

4. The client MUST render only channels the token's subject belongs to, as returned by `GET /v1/chat/channels`; it MUST NOT assume or fabricate membership.

5. On selecting a channel the client MUST load history from `GET .../messages` and then open the websocket for that channel; live message events MUST be appended without a reload, and a message already shown from history MUST NOT be duplicated when the same id arrives over the websocket.

6. The client MUST reflect presence and typing from the websocket event stream: a member coming online or going offline updates the presence display, and a typing event shows a transient indicator. It MUST NOT poll for these in a tight loop where the event stream already carries them.

7. The client MUST mark a channel read (`POST .../read` with the newest visible message id) when the channel is opened and when a live message arrives while it is open, and MUST reflect the resulting unread count. It MUST NOT mark messages read that were never shown.

8. The client MUST be a static page: HTML, CSS, and client JavaScript served as a file, with no server-side rendering and no application server of its own. State lives client-side and in the chat service it calls.

9. The client MUST fail visibly when a call is rejected or the service is unreachable: the affected action shows a clear error, not a silent no-op or a fabricated success. A websocket that drops while a channel is open SHOULD attempt to reconnect.

10. Cross-origin access for local development MUST be an opt-in flag on the chat service (`CHAT_DEV_CORS`), off by default; in production the page and the service are served under one origin by Caddy, so no cross-origin allowance is enabled. The client MUST NOT depend on a permissive CORS posture being on in production.

## Alternatives Considered

Fold the chat surface into the TASK-APP-001 console as another panel. Reasonable, and the two share the CDS shell, but chat is its own interaction model (a persistent sidebar, a live transcript, a composer, presence and typing) rather than a read-only operator screen. Keeping it a sibling page in the same `apps/console/` family preserves the shared look without forcing chat into the operator-dashboard layout. The two can be linked or later merged behind one shell.

Build the client with a framework and a build step (the package.json / TypeScript path sketched in TASK-APP-001's file list). Rejected for this first slice. A single self-contained page has no build to operate, drops straight behind Caddy, and matches the shipped TASK-APP-001 `index.html`, which is also a self-contained page. A framework can come later if the client grows; it is not needed to make chat usable.

Add a thin backend for the client (session handling, message aggregation). Rejected for the same reason as TASK-APP-001: the chat service already issues nothing the client cannot get from the token and the existing routes, so a backend would duplicate auth and add a service to operate.

## Success Metrics

Primary metric - chat usable by a human end to end.
- Definition: an operator can, from the page, sign in with a token, see their channels, open one, see history, send a message, and see another member's message, presence, and typing arrive live without a reload.
- Baseline: 0. No client exists; chat is exercised only by scripts.
- Target: the full path works against the running service in a local check.
- Measurement method: an owner-run live check (the page served, the chat service running with `CHAT_DEV_CORS=1`), plus the existing server smokes that prove the endpoints the client calls.
- Source: the page at `apps/console/chat.html` and the cyberos-chat smokes (`services/chat/tests/smoke_slice*.py`).

Guardrail metric - new backend endpoints introduced by the client.
- Definition: number of new server-side endpoints the client requires.
- Baseline: 0. The client is specified as a pure front-end over the shipped chat surface.
- Target: zero. The one service change this task allows is the opt-in `CHAT_DEV_CORS` dev flag, which adds no route and ships off.
- Measurement method: review the client's calls against the existing chat route list; any call to a route that does not already exist fails the check.
- Source: the cyberos-chat router (`services/chat/src/lib.rs`) and code review of the page's fetch and websocket calls.

## Scope

In scope: the self-contained CDS-styled page (`apps/console/chat.html`) with token connect, channel list and unread badges, channel create, add-member, history load, live message and presence and typing over the websocket, read receipts, and a composer; and the opt-in `CHAT_DEV_CORS` dev flag on the chat service so a local browser may reach it.

### Out of scope

- Any new backend API. The client is a front-end only; new data needs go to the chat service's task.
- Attachments upload UI and search UI. The server ships both (TASK-CHAT-101 slice 3); surfacing them in the client is a later additive slice.
- Voice and video media. The server relays WebRTC signaling (slice 5); the media path, TURN, and a call UI are a separate task and depend on deferred infrastructure.
- Threads, edit, and delete UI. The server supports them (slice 2); the first client slice is the flat channel transcript, with threads and edit added later.
- A native or mobile client. This is a responsive web page; a native shell is a separate task.

## Dependencies

- TASK-CHAT-101 native chat module - the REST and websocket surface the client consumes; all six server slices are built and proven.
- TASK-AUTH-110 AUTH as OIDC provider - issues the CyberOS access token the client presents; the subject and tenant come from it.
- TASK-APP-001 CDS web console - the sibling console page and the CDS palette and self-contained-page pattern this client follows.
- Cross-cutting: the existing Caddy front that serves the console statically and proxies the chat service under one origin in production, removing the need for the dev CORS flag there.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this task and the client page from the shipped cyberos-chat surface and the TASK-APP-001 console pattern.
- Scope: the specification and the first client slice (`apps/console/chat.html`), plus the opt-in `CHAT_DEV_CORS` flag on the chat service. No new chat endpoint is added.
- Human review: Stephen reviews and approves before status moves past implementing, and does all git. The "front-end only, no new backend" boundary and the CDS-as-source-of-truth rule are operator-mandated; the paired audit (TASK-APP-007.audit.md) validates the format before merge.
