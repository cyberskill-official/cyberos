# ADR-OBS-003-001: RED instrumentation via an axum middleware layer, not a per-handler proc-macro

- Status: Accepted
- Date: 2026-06-20
- Context FR: FR-OBS-003 (per-service RED metrics)
- Decision owner: CTO (self-approved architectural deviation, per ship-feature-requests §2 step 3-4)

## Context

FR-OBS-003 §1 #5 and #11 prescribe a `#[red_instrument(service, route)]` proc-macro applied to every axum handler, plus a CI `instrument_completeness_test` that AST-walks each service's handler files and fails if any axum-handler-shaped function lacks the macro. The intent is full RED coverage with no handler left uninstrumented.

Mapping the real services changed the picture:

- The instrumented set is three in-repo Rust services (`auth`, `memory`, `ai-gateway`), not four. `chat` ships as a pinned Docker image (`CHAT_IMAGE_TAG`) and has no Rust source in this workspace, so a source-level macro cannot apply to it here.
- The three services have different handler shapes and different tenant sources (`auth` and `memory` take `Extension<Claims>` injected by a `verify_jwt` middleware; `ai-gateway` resolves tenant elsewhere). A proc-macro that wraps an arbitrary handler and extracts `tenant_id` from its destructured extractors is fragile across these shapes and would touch every handler in three currently-green services.

## Decision

Instrument RED with a single axum middleware layer per service, provided by `cyberos-obs-sdk`, instead of a per-handler proc-macro.

- `cyberos_obs_sdk::layer::red_mw` (an `axum::middleware::from_fn_with_state` function) wraps every request: it reads the matched route template (`MatchedPath`), times the request, reads the request's tenant from a `TenantCtx` response extension, and calls `red::record_request` on the way out.
- Each service adds the layer once at its router (`.layer(from_fn_with_state(RedState::new("<service>"), red_mw))`), calls `red::init("<service>", version)` at boot, and inserts `TenantCtx` from its own claims so the `tenant_id` label is real (falling back to `"unknown"` where no auth context exists, e.g. `/healthz`).

## Consequences

- One touch point per service instead of one per handler; the middleware cannot miss a route, which removes the need for the AST-walk completeness lint (#11) - route coverage is structural, guaranteed by the router rather than checked after the fact.
- Lower blast radius on three production services: no rewrite of handler signatures.
- The metric contract is unchanged: the same `cyberos_requests_total` / `cyberos_errors_total` / `cyberos_duration_ms` names, the same labels (`service`, `route`, `tenant_id`, `status_class`, `error_class`), the same buckets, the same cardinality guard. FR-OBS-002 tenant filtering still gets its `tenant_id` label.
- Trade-off: a route label is the matched template (e.g. `/v1/auth/token`), which is exactly what bounded cardinality wants; non-HTTP code paths (background jobs) still call `record_request` by hand, as the FR already allows.
- `chat` is instrumented when/where its source lives; it is out of scope for the in-repo rollout.

This supersedes the literal mechanism in FR-OBS-003 §1 #5 and #11 while satisfying their intent (complete, consistent RED coverage). The FR text is annotated to point here.
