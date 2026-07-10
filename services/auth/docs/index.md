---
title: auth — identity and tenancy · CyberOS
migrated: FR-DOCS-002
---

auth owns who you are and which tenant you act in. Every service trusts auth's JWTs and Postgres row-level security; nothing else hands out identity.

## Current state

- Google SSO sign-in with JIT provisioning; sessions revocable account-wide (kick-by-revoke, sockets die inside a minute).
- Tenants and subjects live in auth's schema; the `cyberos_app` role and RLS policies enforce tenant isolation fail-closed — a missing tenant context returns nothing rather than everything.
- Admin surfaces (tenant create, subject create) are idempotent APIs with structured validation, bcrypt-cost-12 credentials, and p95 latency budgets enforced by tests.
- auth is also an OIDC provider for first-party apps (chat signs in through it), and sign-ins emit consent-gated interaction events into the memory audit chain.
- The suite (about 220 tests incl. the DB-backed set) runs in `scripts/local_verify.sh`.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
