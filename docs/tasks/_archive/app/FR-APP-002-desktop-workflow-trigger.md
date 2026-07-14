---
template: task@1
id: TASK-APP-002
title: "APP desktop workflow trigger - Tauri app to run CyberOS workflows"
author: "@stephen"
department: engineering
status: superseded
superseded_by: the React console (apps/web) - the static console tiles shipped, then were replaced by the SPA
priority: p2
created_at: "2026-06-22T11:00:00+07:00"
ai_authorship: assisted
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: 2026-Q4
client_visible: false
module: app
new_files:
  - apps/desktop/src-tauri/Cargo.toml
  - apps/desktop/src-tauri/src/main.rs
  - apps/desktop/src-tauri/src/auth.rs
  - apps/desktop/src-tauri/src/keychain.rs
  - apps/desktop/src-tauri/src/gateway_client.rs
  - apps/desktop/src-tauri/src/mcp_client.rs
  - apps/desktop/src-tauri/tests/auth_token_storage_test.rs
  - apps/desktop/src-tauri/tests/workflow_catalog_test.rs
  - apps/desktop/src/main.ts
  - apps/desktop/src/views/picker.ts
depends_on: [TASK-AUTH-004, TASK-AUTH-005, TASK-AI-105, TASK-AI-022, TASK-MCP-001, TASK-MCP-006, TASK-CUO-101, TASK-PORTAL-006]
---

# Task

> Turn Your Will Into Real.

## Summary

CyberOS needs a small desktop app that lets the operator trigger CyberOS workflows and skills directly from the desktop, without opening a browser or a terminal. The app is built with Tauri: a Rust backend matching the rest of the stack, a webview front-end, and a single small native binary per OS. It signs in through the existing auth service, stores the session token in the OS keychain, lists the workflows and skills that CUO and the mcp-gateway expose, and drives the existing gateway HTTP endpoints plus the MCP surface to start a run. It re-implements no workflow logic. CUO already orchestrates everything behind those endpoints; this app is a trigger and a status surface. macOS ships first because that is the operator's machine; Windows and Linux are follow-on targets from the same Rust codebase.

## Problem

Triggering a CyberOS workflow today means either calling the gateway HTTP endpoint by hand (curl, an HTTP client) or driving the CUO CLI in a terminal. Both work, but neither is something the operator wants in front of him every day. The CLI assumes a checked-out repo and a configured shell. The raw HTTP path means pasting a bearer token and remembering endpoint shapes. There is no first-party way to say "run this workflow" from the desktop, see which workflows and skills are even available, and watch the run move.

The control plane to back this already exists. The ai-gateway exposes an HTTP serving surface (TASK-AI-022, extended by TASK-AI-105). The mcp-gateway exposes the MCP tool and task surface (TASK-MCP-001, TASK-MCP-007). CUO already walks the skill chain and orchestrates the workflow (TASK-CUO-101). Auth already issues and validates the token (TASK-AUTH-004). What is missing is a thin, native, operator-facing trigger that ties them together and stores the token safely. The operator runs CyberOS unattended for long stretches; a desktop launcher that starts a workflow in two clicks and shows its state removes the terminal round-trip from the daily loop.

## Proposed Solution

A Tauri desktop app under `apps/desktop/`. The Rust backend (src-tauri) owns auth, token storage, and the two API clients (gateway HTTP, MCP); the webview front-end is the picker and the run view. User-visible behaviour: the operator opens the app, signs in once, sees a list of available workflows and skills, picks one, fills any required inputs, clicks run, and watches the status. The token lives in the OS keychain, never in a plaintext file. If a backend is unreachable, the app says so plainly and does not pretend a run started.

Tauri is the recommended framework, for four concrete reasons. The backend is Rust, which matches the gateway, mcp-gateway, and the rest of the service code, so the API clients and types can share shape with the existing crates instead of being re-derived in another language. The shipped binary is small (single-digit to low-tens of MB) because Tauri uses the OS webview rather than bundling a browser engine, which matters for a tool the operator updates often. It targets macOS, Windows, and Linux from one codebase, so the macOS-first build extends to the other two without a second stack. And the macOS build is first-class, which fits the operator's machine. An Electron app would bundle Chromium (a much larger binary) and put the backend in Node, away from the Rust stack; that is the main alternative and it is rejected below.

### Section 1 - normative requirements (BCP-14)

1. The app MUST be a Tauri application with a Rust backend under `apps/desktop/src-tauri/` and a webview front-end under `apps/desktop/src/`. It MUST NOT bundle a separate browser engine; it uses the OS webview that Tauri provides.

2. The app MUST drive only the existing backend surfaces: the ai-gateway HTTP endpoints (TASK-AI-022 / TASK-AI-105) and the mcp-gateway MCP surface (TASK-MCP-001). It MUST NOT invent a new backend endpoint, and it MUST NOT re-implement any workflow or skill logic; CUO (TASK-CUO-101) orchestrates the run behind those endpoints.

3. The app MUST sign in through the existing auth service (TASK-AUTH-004 token issuance, TASK-AUTH-005 admin REST), reusing the same JWT the gateway and mcp-gateway already accept. It MUST NOT mint its own credential format or bypass the auth flow.

4. The session token MUST be stored in the OS keychain (macOS Keychain via the Security framework; the Windows Credential Manager and the Linux Secret Service are the equivalents on the follow-on targets). The token MUST NOT be written to a plaintext file, to app-local storage, or to a log line.

5. The app MUST present a workflow and skill picker that lists what CUO and the mcp-gateway actually expose at runtime: workflows from the gateway, tools and tasks from the MCP `tools/list` surface (TASK-MCP-001), filtered to what the signed-in subject is allowed to invoke per TASK-MCP-006 annotation gating. The picker MUST NOT show a hard-coded list that can drift from the backend.

6. The macOS build MUST be the first delivered target. Windows and Linux are follow-on targets built from the same Rust codebase; the code MUST NOT take a hard dependency on a single OS beyond the keychain backend, which is selected per platform behind one trait.

7. Triggering a run MUST call the gateway or MCP endpoint and surface the returned run or task id and status to the operator. The app MUST show a clear, non-fabricated error when the backend is unreachable or returns a non-2xx; it MUST NOT display a success state for a call that did not succeed.

8. Every outbound call MUST carry the keychain-stored bearer token. On a 401 the app MUST send the operator back to sign-in rather than retrying with a stale token.

9. The gateway-client and mcp-client request-build and response-parse functions MUST be pure and unit-tested without a live server. A live round trip against a running gateway is an owner-run integration check, not a unit-test dependency.

10. The app MUST treat the token and any run inputs as the operator's own data on the operator's machine. It MUST NOT send telemetry to any third party, and the only network destinations are the configured CyberOS gateway and mcp-gateway.

## Alternatives Considered

Electron instead of Tauri. Rejected on two counts. It bundles Chromium, so the binary is an order of magnitude larger for a tool the operator updates frequently, and it puts the backend in Node, away from the Rust stack that the gateway and mcp-gateway are written in. Tauri keeps the backend in Rust (shared types and clients with the existing crates) and uses the OS webview, so the binary stays small. The cross-OS story is comparable; the stack-match and size arguments decide it.

A terminal TUI (a Rust ratatui app) instead of a desktop app. Rejected because it still lives in a terminal, which is the round-trip this FR removes. A TUI would reuse the same Rust clients, but the operator wants a desktop launcher with a window and a clickable picker, not another shell program. The clients are factored so a TUI could be added later if needed.

Drive the existing portal client-initiated-workflow surface (TASK-PORTAL-006) from a desktop wrapper. Rejected because that surface is tenant-facing and engagement-scoped: it is for a client submitting a request into a CHAT thread, not for the CyberOS operator triggering an internal workflow. The operator surface is `app`, distinct from the tenant-facing `portal`; reusing the portal write path would conflate the two audiences and pull engagement scoping into an operator tool.

## Success Metrics

Primary metric - operator workflow triggers from the desktop app.
- Definition: fraction of the operator's workflow and skill runs in a week that are started from the desktop app rather than the CUO CLI or a raw HTTP call.
- Baseline: 0%. No desktop app exists today; every run starts from the CLI or by hand.
- Target: at least 60% of the operator's runs started from the app within four weeks of the macOS build landing.
- Measurement method: the gateway request log carries a client-id tag per caller; count runs tagged with the desktop app over total operator-started runs.
- Source: ai-gateway request log (TASK-AI-022) and the memory audit rows (TASK-AI-003) for the same runs.

Guardrail metric - token exposure outside the keychain.
- Definition: number of incidents where the session token is found written to a plaintext file, app-local storage, or a log line instead of the OS keychain.
- Baseline: not applicable; no desktop token store exists today.
- Target: zero. The token is read from and written to the keychain only, by construction.
- Measurement method: the keychain-only storage unit test plus a static check that no code path writes the token to a file or log, run in the app's CI.
- Source: the `auth_token_storage_test.rs` output and code review of the keychain trait's call sites.

## Scope

In scope: the Tauri shell (Rust src-tauri backend plus the webview front-end), sign-in through the existing auth service, OS-keychain token storage behind a per-platform trait, the gateway HTTP client and the MCP client, the workflow and skill picker driven by the live backend surfaces, the run-trigger and status view, the macOS build, and pure unit tests for the client build/parse paths plus the keychain store.

### Out of scope

- A full IDE or code editor. This is a launcher and a status surface, not a development environment.
- Offline workflow execution. The app triggers runs on the live backend; with no backend reachable it shows an error, it does not run anything locally.
- App-store distribution (Mac App Store, Microsoft Store) in v1. The macOS build ships as a directly distributed signed binary; store packaging is a later FR.
- Re-implementing any workflow or skill. The app only triggers existing ones through the existing endpoints; CUO and the skills own the logic.
- Windows and Linux builds as delivered artefacts in v1. The code stays portable and the keychain trait has the other backends sketched, but only macOS is built and shipped first.

## Dependencies

- TASK-AI-022 ai-gateway HTTP serving surface - the HTTP endpoints the app calls to start and observe a run.
- TASK-AI-105 local + external model providers - the same serving path the gateway exposes; a workflow the app triggers can resolve to a local or cloud model behind the gateway.
- TASK-MCP-001 MCP spec compliance - the `tools/list`, tool-call, and task surface the picker reads and the run-trigger drives.
- TASK-MCP-006 tool-annotation gating - filters the picker to tools the signed-in subject may invoke.
- TASK-CUO-101 LangGraph supervisor - orchestrates the workflow behind the gateway and MCP endpoints; the app does not duplicate this.
- TASK-AUTH-004 JWT and JWKS - the token the app obtains and presents.
- TASK-AUTH-005 admin REST - the sign-in and subject endpoints the auth flow uses.
- TASK-PORTAL-006 client-initiated workflows - referenced for contrast, not consumed: that is the tenant-facing write path; this app is the operator-facing one, and the boundary between `app` and `portal` is the reason this is a separate module.
- Cross-cutting: the OS keychain (macOS Keychain on the first target; Windows Credential Manager and Linux Secret Service on the follow-on targets), reached through one per-platform storage trait.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from Stephen's capability request and the existing gateway, mcp-gateway, CUO, and auth FRs.
- Scope: full draft of this specification, including the normative clauses, the Tauri recommendation and its rationale, the alternatives, the metrics, and the scope boundaries. No application code is written by this FR; the desktop app is built in a later session.
- Human review: Stephen reviews and approves before status moves past draft. The Tauri choice (Rust-native, small binary, multi-OS, macOS first) is operator-confirmed, and the paired audit (TASK-APP-002.audit.md) validates the format before merge.
