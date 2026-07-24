---
id: TASK-MCP-005
title: "MCP Protected Resource Metadata (RFC 9728) well-known endpoint"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: mcp
priority: p0
status: reviewing
entered_via: rework
routed_back_count: 1
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-MCP-001, TASK-MCP-004, TASK-AUTH-004]
depends_on: [TASK-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#oauth
  - https://datatracker.ietf.org/doc/html/rfc9728

source_decisions:
  - DEC-895 2026-05-17 — RFC 9728 PRM is the discovery document for audience-bound MCP tokens (TASK-MCP-004)
  - DEC-896 2026-05-17 — PRM served unauthenticated at `/.well-known/oauth-protected-resource`
  - DEC-897 2026-05-17 — Per-module PRM at `/.well-known/oauth-protected-resource/:module`; aggregate omits scopes_supported (DEC-905)
  - DEC-900 2026-05-17 — bearer_methods_supported = ["header"] only
  - "DEC-901 (as-built) — resource_signing_alg_values_supported = [\"RS256\"] only; EdDSA deferred until the verifier supports it"
  - DEC-903 2026-05-17 — Cache-Control + ETag revalidation
  - DEC-905 2026-05-17 — scopes_supported omitted on aggregate; present on per-module PRMs

language: rust 1.81
service: cyberos/services/mcp-gateway/
new_files:
  - services/mcp-gateway/src/oauth/prm.rs
modified_files:
  - services/mcp-gateway/src/router.rs
  - services/mcp-gateway/src/oauth/mod.rs
  - services/mcp-gateway/src/oauth/audit.rs

allowed_tools:
  - file_read: services/mcp-gateway/**
  - file_write: services/mcp-gateway/src/**
  - bash: cd services && cargo test -p cyberos-mcp-gateway prm

disallowed_tools:
  - advertise signing algs the jwt verifier cannot verify
  - claim multi-issuer residency PRM / drift table / rate limit as shipped

effort_hours: 3
subtasks:
  - "1.0h: oauth/prm.rs builders + etag + unit tests"
  - "1.0h: router aggregate + per-module routes + caching headers"
  - "1.0h: batch/9a-mcp re-spec + audit"

risk_if_skipped: "Without PRM, federated MCP clients that receive a 401 cannot discover where to authenticate or which audience to request."
---

# TASK-MCP-005: MCP Protected Resource Metadata (RFC 9728)

## Summary

Serve RFC 9728 Protected Resource Metadata so MCP clients discovering a 401 can learn the authorization server and audience. As-built: pure builders in `services/mcp-gateway/src/oauth/prm.rs`, public routes on `router.rs` for aggregate + per-module documents, RS256-only alg advertisement, ETag/Cache-Control, and router tests for public JSON / 304 / unknown-module 404.

## Problem

The engineering-spec claimed a standalone `services/mcp/src/prm/` tree, drift-log migration, eight `prm_*` integration tests, four-issuer residency list, EdDSA, rate limits, and OTel p95. None of that matches HEAD: PRM lives inside the oauth module, tests are in-crate / router tests, and the gateway is its own single issuer today.

## Proposed Solution

Adopt `oauth::prm`:

- `protected_resource_metadata` — aggregate (no `scopes_supported`)
- `protected_resource_metadata_for_module` — adds module scopes from the registry
- `etag` — SHA-256 prefix for If-None-Match 304
- Router: `GET /.well-known/oauth-protected-resource` and `/:module`; WWW-Authenticate `resource_metadata` points at the aggregate

## Alternatives Considered

- **Advertise EdDSA because AUTH-004 may support it later.** Rejected: verifier is RS256-only; advertising EdDSA invites unverifiable tokens.
- **Ship four-issuer residency list now.** Rejected: single-issuer matches current gateway-as-AS deployment; multi-issuer deferred.
- **Standalone prm/ module + drift SQL.** Rejected: house style keeps PRM next to oauth discovery.

## Success Metrics

- Primary: aggregate + per-module PRM match RFC 9728 shape; unknown module 404; ETag revalidation returns 304.
- Guardrail: signing algs never include EdDSA/HS256; cargo prm/router tests green.

## Scope

In scope:

- `src/oauth/prm.rs` builders + unit tests
- Router aggregate / per-module / OPTIONS / caching / WWW-Authenticate pointer
- Best-effort `mcp.prm_unknown_module_requested` audit on unknown module

### Out of scope / Non-Goals

- Drift table / detector / `mcp.prm_aggregate_drift`
- Rate limit (600/min), OTel p95 alarms, NATS cache invalidation
- Four-issuer residency `authorization_servers` list
- EdDSA (or any non-RS256) in `resource_signing_alg_values_supported`
- Standalone `services/mcp/src/prm/**` and phantom `prm_*` integration filenames

## Dependencies

`depends_on: [TASK-MCP-004]`. Soft: TASK-MCP-002 registry for per-module scopes; TASK-AUTH-004 JWKS/issuer alignment.

## 1. Description (normative)

- 1.1 Aggregate PRM MUST include `resource`, `authorization_servers`, `bearer_methods_supported=["header"]`, `resource_signing_alg_values_supported=["RS256"]`, `resource_documentation`, and MUST omit `scopes_supported`.
- 1.2 Per-module PRM MUST add `scopes_supported` (possibly empty) for registered modules and MUST 404 for unknown modules.
- 1.3 Responses MUST be public JSON with caching headers; matching If-None-Match MUST return 304.
- 1.4 WWW-Authenticate challenges for invalid tokens MUST point `resource_metadata` at the aggregate PRM URL.
- 1.5 This adopt MUST NOT claim Out-of-scope residency/drift/rate-limit/EdDSA surfaces as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - aggregate RFC9728 shape omits scopes - test: `services/mcp-gateway/src/oauth/prm.rs::aggregate_has_rfc9728_shape_and_omits_scopes`
- [ ] AC 2 (traces_to: #1.1) - signing algs never advertise EdDSA/HS256 - test: `services/mcp-gateway/src/oauth/prm.rs::signing_algs_never_advertise_eddsa_or_hs256`
- [ ] AC 3 (traces_to: #1.2) - per-module carries scopes - test: `services/mcp-gateway/src/oauth/prm.rs::per_module_carries_the_modules_scopes`
- [ ] AC 4 (traces_to: #1.2) - empty scopes present but empty - test: `services/mcp-gateway/src/oauth/prm.rs::per_module_empty_scopes_is_present_but_empty`
- [ ] AC 5 (traces_to: #1.3) - aggregate is public JSON with caching headers - test: `services/mcp-gateway/src/router.rs::prm_aggregate_is_public_json_with_caching_headers`
- [ ] AC 6 (traces_to: #1.3) - ETag revalidation returns 304 - test: `services/mcp-gateway/src/router.rs::prm_etag_revalidation_returns_304`
- [ ] AC 7 (traces_to: #1.2) - known module OK / unknown 404 - test: `services/mcp-gateway/src/router.rs::prm_per_module_known_ok_unknown_404`
- [ ] AC 8 (traces_to: #1.4) - WWW-Authenticate points at PRM - test: `services/mcp-gateway/src/router.rs::www_authenticate_challenge_points_at_the_prm`
- [ ] AC 9 (traces_to: #1.5) - Out of scope lists deferred drift/rate/residency/EdDSA - verify: `docs/tasks/mcp/TASK-MCP-005-protected-resource-metadata/spec.md` Scope → Out of scope

## Verification

```bash
cd services && cargo test -p cyberos-mcp-gateway prm
cd services && cargo test -p cyberos-mcp-gateway --lib router::tests::prm_
cd services && cargo test -p cyberos-mcp-gateway --lib router::tests::www_authenticate_challenge_points_at_the_prm
```

| Path | Covers |
|------|--------|
| `src/oauth/prm.rs` unit tests | Document shape, algs, scopes, etag |
| `src/router.rs` prm_* / www_authenticate tests | HTTP surface |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9a-mcp`.
- **Scope:** Re-spec/adopt against `oauth/prm.rs` + router; deferred residency/drift/rate/EdDSA ledgered.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9a-mcp adopt — TASK-MCP-005 re-spec against as-built oauth PRM.*
