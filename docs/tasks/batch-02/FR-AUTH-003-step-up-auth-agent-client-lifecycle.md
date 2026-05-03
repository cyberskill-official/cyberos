---
title: "AUTH — step-up authentication for irreversible operations + agent OAuth client lifecycle (refresh, rotate, revoke)"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P0 / 2026-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Extend AUTH (FR-AUTH-001) with two P0-stabilisation capabilities: **step-up authentication for irreversible-adjacent operations** (a fresh passkey ceremony required before tool calls annotated `irreversible: true` or `destructive: true; sensitivity: high`, regardless of whether the user's session is otherwise authenticated), and the full **agent OAuth client lifecycle** (per-Member-per-MCP-client registration, refresh-token rotation with single-use semantics, manual + automatic revocation, agent-client inventory UI). Together these close the gaps the founder will encounter in the first weeks of dogfooding: the password-equivalent value of an authenticated session is too high if any tool call can succeed without re-auth on a stale tab; and the agent-parity invariant breaks down operationally without a clean way to register, rotate, and revoke per-client tokens.

## Problem

Two failure modes the founder will hit immediately without this FR:

- **Stale-session accidental destructive call.** The founder leaves a tab open at home overnight; a partner walks past and clicks an action that a CUO Notify card surfaced; the destructive-confirmation dialog appears; the partner clicks "yes". The session is the founder's; the audit row records the founder; the operation runs. The fix is step-up auth: any tool call annotated `irreversible: true` or `destructive: true; sensitivity: high` requires a fresh passkey gesture, regardless of session validity.
- **Agent client sprawl.** The founder authorises Claude.ai once; a month later he tries Cursor, Claude Desktop, an MCP-aware Raycast extension, and the embedded CUO client; if any of those is compromised or stale, the founder has no surface to revoke just that one client without invalidating his whole identity. The agent-parity invariant (FR-AUTH-001 §"Agent authentication") requires per-Member-per-client registration; this FR ships the lifecycle UI and the rotation policy.

The PRD §8.6 commits to MFA on irreversible operations (currently enforced only at login); this FR makes the enforcement per-operation. PRD §8.4.2's `destructive: true; requires_confirmation: true` annotation is the gateway-level surface; AUTH provides the proof.

## Proposed Solution

The shape of the answer is a small AUTH extension + UI surfaces in `/auth/account` + the "step-up confirmation token" plumbing used by the MCP gateway and the host shell.

**Step-up authentication.**

When a Member tries to perform an operation that requires step-up (defined below), the platform:

1. Returns `code: "STEP_UP_REQUIRED"` with `reason: "irreversible_operation" | "high_sensitivity" | "policy_threshold"`.
2. The host shell intercepts the response and shows a step-up dialog: "This action requires re-authentication. Tap your passkey to confirm." The dialog re-runs the WebAuthn `get()` ceremony (not `create()`; the credential already exists). The user's existing passkey is requested with `userVerification: "required"`.
3. On success, AUTH issues a **step-up token** with a 5-minute lifetime, narrowly scoped to the specific operation (`aud: <operation-id>`, `subject: <member-id>`, `bound_to_request: <request-id>`).
4. The original operation is retried with the step-up token in `X-Step-Up-Token` header. The operation's authorisation check requires both the regular access token *and* the step-up token; both must match the same Member.
5. On failure (passkey verification fails, or the user dismisses), the operation is rejected with `code: "STEP_UP_FAILED"` and an audit row in scope `auth.step_up.{tenant}`.

**Operations requiring step-up.**

- Every MCP tool call where `irreversible: true` (just `cyberos.cp.rtbe_request` in P0 from FR-CP-002).
- Every MCP tool call where `destructive: true; sensitivity: high`. P0 examples: `cyberos.auth.revoke_session` for *another* Member's session (a Member's own session revoke is `sensitivity: low`); `cyberos.genie.persona_pause` and `persona_resume`; `cyberos.genie.global_pause`; `cyberos.cp.dsar_request` initiated for someone else.
- Every UI action mapping to the above tools.
- Every direct GraphQL mutation marked with the `@stepUp` directive (composition-time enforced; modules declare which mutations require step-up).
- Optional per-Member preference: a Member can opt into "step-up always" — every destructive operation requires a fresh passkey.

**Step-up token semantics.**

- Lifetime: 5 minutes.
- Use: single-use; re-using the same token returns 409.
- Audience: the specific operation it was minted for; an attempt to reuse for a different operation is rejected.
- Storage: opaque server-side state in `auth.step_up_token` keyed by hash; the wire token is a 256-bit base64url string.
- Issuer: AUTH module.
- Audit: every issuance + every consumption + every failure logged in scope `auth.step_up.{tenant}`.

**Agent OAuth client lifecycle.**

Each Member can register multiple agent clients (MCP-aware tools). The lifecycle:

- **Register.** A Member, signed in via passkey, navigates to `/auth/account/agent-clients` and clicks "Register a new agent client". A wizard prompts for: client display name (e.g. "Claude on my Macbook Air"), client kind (Claude.ai, Cursor, Claude Desktop, Raycast, custom), expected residency region, and a **scope-limited authorisation** (the Member can opt into "this client can call only `cyberos.brain.*` and `cyberos.proj.*`" rather than the Member's full RBAC, narrowing the agent's effective rights below the human's). On submission, AUTH issues a one-time enrolment URL valid for 5 minutes; the Member opens the URL in the agent client; the client completes the OAuth 2.1 + PKCE flow with `aud: https://mcp.cyberos.world` (and the scope limitation applied as a custom claim).
- **Inspect.** `/auth/account/agent-clients` lists every active client with: display name, kind, last-used-at, residency region, scope limitation, refresh-token-rotation count.
- **Refresh-token rotation.** Refresh tokens are single-use with rotation: every refresh issues a new refresh token and invalidates the prior. A reused refresh token (e.g. a stolen leaked token replayed) signals theft and triggers immediate revocation of the entire client + a high-priority Notify to the Member.
- **Manual revoke.** A Member can revoke any of their clients with a single click; revocation is immediate (the access token is invalidated cluster-wide via NATS broadcast within 30 seconds; the refresh token is destroyed in storage; in-flight requests using the revoked token are rejected within the same window).
- **Automatic rotation.** Refresh tokens older than 30 days are auto-rotated on next refresh (forced single-use; no impact on UX). Refresh tokens unused for 14 days expire (Member must re-authorise).
- **Quarterly review prompt.** Every 90 days, the Member is shown a Notify card "Review your active agent clients" with a list and a single-button per-client revoke; this is the operational discipline for keeping the agent surface clean.

**Agent client storage.**

```sql
CREATE TABLE auth.agent_client (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  display_name TEXT NOT NULL,
  client_kind TEXT NOT NULL,
  residency_region TEXT NOT NULL,
  scope_limit_predicates TEXT[],          -- e.g. ["brain.*", "proj.*"]; null = full Member RBAC
  current_refresh_token_hash BYTEA,       -- single-use, sliding-window rotation
  last_refreshed_at TIMESTAMPTZ,
  last_used_at TIMESTAMPTZ,
  refresh_count INT NOT NULL DEFAULT 0,
  registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  revoked_at TIMESTAMPTZ,
  revocation_reason TEXT
);
```

**Audit integration.** Every register / refresh / revoke / step-up issuance / step-up consumption writes an audit row in `auth.agent_client.{tenant}` or `auth.step_up.{tenant}` scope.

**Genie panel surface.** When a stale agent client is detected (sustained 14-day inactivity), CUO/CTO surfaces a Notify card "You have unused agent clients (3)"; clicking opens the agent-client inventory UI. A new-client registration triggers a confirmation card in the founder's panel showing the client kind and scope limits — a sanity check before the client gains access.

**MCP tool surface.**

- `cyberos.auth.list_my_agent_clients` — read; returns the calling Member's own clients only (regardless of role; a Member cannot enumerate another Member's clients).
- `cyberos.auth.revoke_agent_client(client_id)` — `destructive: true; requires_confirmation: true`; only the owning Member or the Founder can revoke.
- `cyberos.auth.request_step_up(operation_id)` — internal; called by the host shell when intercepting `STEP_UP_REQUIRED`.
- `cyberos.auth.list_active_step_up_tokens(member_id?)` — read; founder + DPO + audit; informational.

**Step-up flow with MCP-driven destructive tool calls.**

The integration with FR-MCP-001 §"Tool annotations enforced at the proxy":

1. An agent calls `cyberos.cp.rtbe_request(...)` — the gateway sees `irreversible: true`.
2. Gateway returns `code: "STEP_UP_REQUIRED"` with the operation-ID embedded.
3. The agent's host (Claude.ai, Cursor) opens a browser tab to `https://auth.cyberos.world/step-up?op=<op_id>` (or surfaces a deep link the Member taps on a phone).
4. The Member completes the passkey ceremony; AUTH issues the step-up token bound to that operation-ID + Member.
5. The agent retries the tool call with the step-up token attached; the gateway validates and proxies the call.

The step-up token cannot be replayed for a different operation, cannot be replayed by a different Member, cannot be replayed after consumption. This is the architectural enforcement of "the human authorises every irreversible".

## Alternatives Considered

- **A single fixed inactivity timeout that forces re-auth on every tool call after N minutes.** Rejected: too friction-heavy; the destructive-vs-non-destructive distinction is the right axis.
- **Use the regular access token for step-up rather than a separate token class.** Rejected: a fresh passkey ceremony plus a narrowly-scoped short-lived token is the architectural floor for proving the human is at the keyboard; reusing the existing token loses that proof.
- **Allow agents to obtain their own step-up tokens via "I am acting as the Member who pre-authorised me".** Rejected: violates the agent-parity invariant by introducing an asymmetry that human Members do not have.
- **Skip per-client lifecycle UX; let Members revoke at IdP level only.** Rejected: per-client granularity is what makes the inventory tractable; without it the founder revokes the wrong client when one is compromised.

## Success Metrics

- **Primary metric.** S0-6 demo passes: (1) the founder attempts a `cyberos.cp.rtbe_request` from an MCP client; the gateway returns `STEP_UP_REQUIRED`; the founder completes the passkey ceremony; the operation succeeds; (2) the founder's agent-client inventory shows ≥ 2 distinct clients (Claude.ai + Cursor at minimum); (3) a synthetic refresh-token replay (the same refresh token used twice) triggers immediate revocation of the synthetic client + a Notify card to the Member.
- **Adoption metric.** ≥ 2 agent clients registered per active employee by P0 → P1 exit (proves the BYO-key model works).
- **Latency metric.** Step-up token issuance + retry round-trip p95 ≤ 4 seconds for a passkey-equipped device.

## Scope

**In-scope (S0-5 + S0-6).**
- Step-up auth flow with the 5-minute single-use bound-to-operation token semantics.
- The list of operations requiring step-up.
- The `@stepUp` GraphQL directive composition rule.
- The host shell's step-up-dialog interception of `STEP_UP_REQUIRED` responses.
- The MCP gateway integration so destructive-tool calls return `STEP_UP_REQUIRED` when appropriate.
- The agent-client lifecycle: register / inspect / refresh-rotate / revoke / quarterly-review.
- The `/auth/account/agent-clients` UI.
- Refresh-token replay detection + automatic revocation.
- Notify card surfaces for stale-client detection and new-client registration.
- The four MCP tools listed above.
- Audit integration in scopes `auth.step_up.{tenant}` and `auth.agent_client.{tenant}`.

**Out-of-scope (deferred).**
- Step-up via biometric on mobile (P3 mobile).
- Continuous-trust scoring (the device's posture + network changes downgrade trust dynamically) — P3.
- SSO + agent-client federation across tenants for the same human (forbidden by design through P3; reconsidered at P4 PORTAL).
- DPoP / mTLS access tokens replacing bearer (P3).

## Dependencies

- FR-AUTH-001 / FR-AUTH-002.
- FR-MCP-001 (the gateway intercepts destructive-tool calls and signals `STEP_UP_REQUIRED`).
- FR-CP-002 (the only `irreversible: true` tool in P0 — RTBE — is the canonical step-up consumer).
- FR-GENIE-001 / FR-GENIE-002 (Notify cards for stale-client + replay detection).
- FR-OBS-001 (audit-row visibility on step-up issuance + replay events).
- HashiCorp Vault for the WebAuthn enrolment key custody; existing Vault deployment from FR-AUTH-001.
- Compliance: PDPL Decree 13 (the per-Member control surface for agent clients is the consent-record surface for personal-data processing through agents); SOC 2 CC6 (logical access control); SOC 2 CC7 (system operations: revocation pathway).
- Locked decisions referenced: DEC-074 (step-up token: 5 min, single-use, bound to operation), DEC-075 (agent-client lifecycle with refresh-rotation single-use), DEC-076 (replay detection auto-revokes).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The step-up + agent-client-lifecycle machinery is deterministic identity flow; no AI-derived behaviour in the path. Notify cards surfaced by CUO/CTO inherit FR-GENIE-001's risk classification.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
