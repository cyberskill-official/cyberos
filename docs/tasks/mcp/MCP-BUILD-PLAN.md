# MCP module build plan (TASK-MCP-001..008)

Written 2026-06-24. mcp is the live front of the locked P0 path (AI -> OBS -> AUTH -> MCP -> CHAT) now
that obs is feature-complete in-repo. This plan reconciles each FR's frontmatter status against the code
actually shipped in `services/mcp-gateway`, then sequences the rest. Implementation is a toolchain step
(cargo), run on your machine - this is the spec-to-code map, not the code.

## The status-lag to clear first (blocks everything)

The frontmatter lags reality. The root dependency `TASK-AUTH-004` (JWT/JWKS) is `done`, and two MCP FRs
are already implemented but still read `draft`:

- TASK-MCP-001 (spec-compliance) - the whole protocol surface is in `src/protocol/` (initialize,
  capabilities, jsonrpc, errors, tools/list, tools/call) plus `annotations.rs` (ToolAnnotations,
  surfaced in tools/list) and the per-tool `requires_scope` gate in `tools_call.rs`. Shipped, `draft`.
- TASK-MCP-002 (server heartbeat lifecycle) - `federation/health.rs` (ServerHealthStatus + classify),
  registry heartbeat / deregister / lazy-status, the `/v1/mcp/heartbeat` + `/v1/mcp/deregister` routes,
  and the tools/list withdrawal + tools/call skill_unavailable gate. Built this session, `draft`.

This matters because `ship-tasks` only picks an FR whose `depends_on` rows are all `done`,
and six of the eight MCP FRs depend on TASK-MCP-001. TASK-MCP-004 even reads `ready_to_implement` while its
dependency TASK-MCP-001 is `draft` - an inconsistency the workflow would refuse. So the first action is
not a new build: it is running the awh + caf gate on TASK-MCP-001 and TASK-MCP-002 on your Mac and flipping
them `draft -> done` on GREEN+CLEAN. That single reconciliation unblocks the rest of the module.

## Dependency order (within mcp)

```
TASK-MCP-001 spec-compliance            [TASK-AUTH-004 done]   SHIPPED -> gate-verify -> done
  ├─ TASK-MCP-002 heartbeat lifecycle   [001]                SHIPPED -> gate-verify -> done
  ├─ TASK-MCP-003 SEP-986 naming validator [001]             slices 1-2 SHIPPED -> slice 3 (CI grep + audit) left
  └─ TASK-MCP-004 OAuth 2.1 + PKCE      [TASK-AUTH-004 + 001]  NOT BUILT (the ready FR - buildable now)
       ├─ TASK-MCP-005 protected-resource-metadata [004]     NOT BUILT
       ├─ TASK-MCP-008 elicitation                 [001+004] NOT BUILT  (build before 006's gating)
       │    └─ TASK-MCP-006 tool-annotation-gating [001+004] PARTIAL   (needs 008 for destructive->elicit)
       └─ TASK-MCP-007 tasks primitive             [001+004] NOT BUILT
```

## Per-FR

### TASK-MCP-001 - spec compliance  (dep AUTH-004; SHIPPED, status-lag draft)
The MCP 2025-11-25 wire surface: `protocol/initialize.rs`, `capabilities.rs`, `jsonrpc.rs`, `errors.rs`,
`tools_list.rs`, `tools_call.rs`, plus `annotations.rs` (ToolAnnotations with read_only/destructive/
idempotent/open_world hints, surfaced in tools/list) and the `requires_scope` permission gate in
tools_call. Federation (register/registry/health) and a live module round-trip are proven (the reference
module + the obs triage module both register and answer tools/call through it). Remaining: gate-verify
and flip to done. No new code expected.

### TASK-MCP-002 - server heartbeat lifecycle  (dep 001; SHIPPED this session, status-lag draft)
DEC-2350 10s heartbeat cadence + DEC-2351 ServerHealthStatus {healthy/degraded/unhealthy/deregistered}
with a pure `classify(now)`; lazy-on-read status (no background reaper); tools/list withdraws an
unavailable module's tools; tools/call fast-fails SKILL_UNAVAILABLE (-32006) before any network when the
owning module is unhealthy. Routes `/v1/mcp/heartbeat` + `/v1/mcp/deregister` behind the same
`MCP_DEV_REGISTRATION` dev-gate; `/mcp/healthz` returns a per-module `servers` array. Remaining:
gate-verify and flip to done.

### TASK-MCP-003 - SEP-986 naming validator  (dep 001; slices 1-2 SHIPPED, slice 3 left)
Slice 1 shipped the pure validator in `src/naming/` (closed 15-verb `Sep986Verb` enum, the
`^cyberos\.(module)\.(verb)_(noun)$` regex compiled once, the 23-module binary-search registry,
`validate_sync`). Slice 2 wired it into registration: `register::validate` now rejects any real
module's non-conforming tool ID with `RegisterError::NonConformingToolName` before it can become
callable, and the one pre-existing non-conforming production tool was migrated
(`cyberos.obs.triage` -> `cyberos.obs.execute_triage`). The dev/reference fixture
(`cyberos.demo.echo` / `cyberos.demo.now`) is exempt via `NAMING_EXEMPT_MODULES` - it predates the
convention and never ships in production. Both slices are pure Rust (cargo test + clippy, no DB).
Slice 3 left (DB/CI-bound, bundled with the next Postgres session): the CI grep gate
(`scripts/check_sep986_naming.sh` + `.github/workflows/mcp-sep986-check.yml`, DEC-2362) and the four
memory-audit kinds (`mcp.skill_name_validated` / `_rejected` / `naming_ci_check_passed` / `_failed`,
DEC-2364). Independent of OAuth.

### TASK-MCP-004 - OAuth 2.1 + PKCE  (dep AUTH-004 + 001; NOT BUILT - the ready FR)
The authorization-code + PKCE flow that lets a real MCP client authenticate to the gateway, building on
TASK-AUTH-004's JWT/JWKS. New module under `src/` (e.g. `auth/` with the authorize + token + PKCE
verifier). Test plan: a valid code+verifier exchanges for a token; a wrong verifier is refused; an
expired or replayed code is refused; the issued token validates against the AUTH-004 JWKS. This is the
gate that turns off `MCP_DEV_REGISTRATION` for production (registration + control-plane move behind real
auth), so it is the highest-value of the unbuilt set.

### TASK-MCP-005 - protected resource metadata  (dep 004; NOT BUILT)
The `/.well-known/oauth-protected-resource` document so clients discover the gateway's auth server and
scopes. Test plan: the metadata document matches the spec shape and points at the TASK-MCP-004 endpoints.

### TASK-MCP-008 - elicitation  (dep 001 + 004; NOT BUILT - build before 006)
The server-initiated elicitation primitive (the gateway asks the client to confirm or supply input
mid-call). Build before TASK-MCP-006 because annotation gating uses it: a `destructive_hint` tool should
require an elicited confirmation. Test plan: an elicitation request round-trips; a declined elicitation
aborts the call cleanly.

### TASK-MCP-006 - tool-annotation gating  (dep 001 + 004; PARTIAL)
The ToolAnnotations struct exists and is surfaced in tools/list (the `destructive()` constructor already
notes "requires Elicitation per TASK-MCP-006"), but the gate is not wired: a destructive tools/call is not
yet held for confirmation. Wire the gate in tools_call - a `destructive_hint` tool requires an elicited
confirmation (TASK-MCP-008) before forwarding. Test plan: a read-only tool calls straight through; a
destructive tool without confirmation is held; with confirmation it forwards.

### TASK-MCP-007 - tasks primitive  (dep 001 + 004; NOT BUILT)
The long-running `tasks/` primitive (start a task, poll status, fetch result) so a tool call that
outlives a request can be tracked. Test plan: a task starts and returns a handle; status moves
working -> complete; the result is fetchable; an unknown task id errors cleanly.

## Keep the gate in step

The mcp goldenset (`modules/mcp/.awh/goldenset.yaml`) and caf profile already cover
`cyberos-mcp-gateway`. Everything above lands in that one crate, so the existing golden-set task
(`cd services && cargo test -p cyberos-mcp-gateway`) covers the new code automatically - no new task per
FR. Re-seal the baseline once after the first FR adds behaviour
(`awh eval modules/mcp/.awh/goldenset.yaml --base-dir . --seeds 1 --out modules/mcp/.awh/eval-baseline.json`),
then it holds for the rest.

## How to ship

1. Reconcile first: run awh + caf on the gateway as it stands, and flip TASK-MCP-001 and TASK-MCP-002
   `draft -> done` on GREEN+CLEAN. This is the unblock - do it before anything else.
2. Ship TASK-MCP-003 (small, independent) alongside, then TASK-MCP-004 (OAuth, the ready FR).
3. With 004 done, ship TASK-MCP-005, then TASK-MCP-008, then TASK-MCP-006 (which needs 008), then TASK-MCP-007.
4. Each FR flows through `ship-tasks` to step 28 (awh rerun) + step 29 (caf target health +
   audit); `testing -> done` flips only on awh GREEN and caf CLEAN.
