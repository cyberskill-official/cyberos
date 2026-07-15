---
title: chat - first-party team messaging · CyberOS
migrated: TASK-DOCS-002
---

chat is CyberSkill's own messaging service - Rust server, web client, no third-party fork. It signs in through auth (OIDC), stores per-tenant, and feeds the memory brain through consent-gated interaction events.

## Current state

- Channels, DMs, threads, reactions, mentions, attachments (multi-file), notify preferences, and channel management, shipped as versioned schema slices (13 migrations).
- Live delivery over WebSocket with hot-path indexes; the console tiles and the web client ride the same APIs.
- Deployed behind the platform's compose stack and rolled by `deploy.yml`; runs at os.cyberskill.world for the P0 login-to-chat path.
- Hardening backlog (rate limiting, JWT tightening, offline sync with idempotent outbox) is tracked as `task-CHAT-2xx` improvement tasks in the one backlog.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
