---
id: FR-MCP-006
title: "MCP tool-annotation gating — destructive / write / external-effect tools require explicit confirm or Elicitation pre-execution per MCP 2025-11-25 spec"
module: MCP
priority: MUST
status: draft
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-MCP-001, FR-MCP-002, FR-MCP-003, FR-MCP-004, FR-MCP-005, FR-MCP-007, FR-MCP-008, FR-AUTH-004, FR-AI-003, FR-MEMORY-111, FR-OBS-007]
depends_on: [FR-MCP-001, FR-MCP-004]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#tool-annotations
  - https://modelcontextprotocol.io/specification/2025-11-25/server/tools#tool-annotations
  - https://modelcontextprotocol.io/specification/2025-11-25/server/utilities/elicitation
  - https://datatracker.ietf.org/doc/html/rfc6750  # bearer-token authorisation

source_decisions:
  - DEC-1040 2026-05-17 — MCP 2025-11-25 spec defines 5 tool annotations: `title`, `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`; this FR enforces gating on the latter 4 hints
  - DEC-1041 2026-05-17 — Tools with `destructiveHint=true` OR `openWorldHint=true` (external effect) MUST require explicit user confirmation OR Elicitation BEFORE the underlying handler runs
  - DEC-1042 2026-05-17 — Tools with `readOnlyHint=true` AND `destructiveHint=false` AND `openWorldHint=false` are FAST-PATH allowed (no confirmation needed)
  - DEC-1043 2026-05-17 — Gating modes: `auto-confirm` (always proceed; for trusted callers), `confirm` (server-side ack required), `elicit` (server initiates Elicitation request to caller via FR-MCP-008)
  - DEC-1044 2026-05-17 — Per-tenant gating policy: tenant_admin configures `mcp_gating_policy_yaml` declaring per-tool / per-annotation gating mode; default per DEC-1041
  - DEC-1045 2026-05-17 — Bypass token: trusted internal callers (per FR-AUTH-004 JWT carrying `mcp_gating_bypass=true` scope claim) skip gating entirely; restricted to system-tenant callers only
  - DEC-1046 2026-05-17 — Gating decision logged in memory audit as `mcp.tool_gating_decision` — every gated call produces a row (decision + reason + confirmation_method + actor_id)
  - DEC-1047 2026-05-17 — Confirmation TTL: confirm-mode acks expire after 5 min if not consumed by the tool call; expired ack = `confirm_required` reload
  - DEC-1048 2026-05-17 — Tool annotations from FR-MCP-002 registration are STORED as part of the module-registration row; annotation drift between registry + tool schema is forensic event sev-1
  - DEC-1049 2026-05-17 — Closed enum `mcp_gating_decision` = {auto_allowed, confirmed, elicited, bypassed, rejected_no_confirmation, rejected_expired_ack}; CI cardinality test asserts 6
  - DEC-1050 2026-05-17 — Per-tenant audit-only mode: a tenant can set `mcp_gating_audit_only=true` for a transition period — gating decisions emitted but tools always proceed (used during policy bootstrap; sev-2 audit row marks audit-only events)
  - DEC-1051 2026-05-17 — memory audit kinds: mcp.tool_gating_decision, mcp.tool_gating_policy_updated, mcp.tool_gating_annotation_drift, mcp.tool_gating_bypass_used, mcp.tool_gating_audit_only_mode_set
  - DEC-1052 2026-05-17 — Gating is enforced AT THE MCP GATEWAY ENTRY (before tools/call dispatches to the per-module server) — single ingress point; per-module server never bypasses
  - DEC-1053 2026-05-17 — Annotation precedence at gating decision (most restrictive wins): destructiveHint=true OR openWorldHint=true → REQUIRES confirmation; readOnlyHint=true AND no destructive/openWorld → BYPASS; idempotentHint informational only
  - DEC-1054 2026-05-17 — Confirmation acks stored in `mcp_pending_confirmations` table keyed by (caller_id, tool_id, request_payload_sha256, expires_at); per DEC-1047 single-use + TTL
  - DEC-1055 2026-05-17 — Elicitation mode delegates to FR-MCP-008 (placeholder until FR-MCP-008 ships); at slice 1 of THIS FR, `elicit` gating mode returns `503 + elicitation_not_yet_supported` until FR-MCP-008 lands
  - DEC-1056 2026-05-17 — Rate limit on confirmation requests: 100 confirm-mode requests/min/caller (defense against confirmation-flood abuse)
  - DEC-1057 2026-05-17 — Bypass-token usage emits sev-2 row `mcp.tool_gating_bypass_used` for every invocation (forensic visibility — bypass is rare-by-design)
  - DEC-1058 2026-05-17 — Policy update requires `tenant_admin` role; emits sev-1 `mcp.tool_gating_policy_updated` (security-relevant config change)
  - DEC-1059 2026-05-17 — Annotation drift detection runs nightly: compares registered annotations vs current tool-schema declared annotations; mismatch = sev-1 `mcp.tool_gating_annotation_drift`
  - DEC-1060 2026-05-17 — Audit-row payload PII-scrubbed via FR-MEMORY-111: request_payload_sha256 only (raw payload retained in MCP gateway logs with RLS-scope, 7-day retention)

build_envelope:
  language: rust 1.81
  service: cyberos/services/mcp/
  new_files:
    - services/mcp/migrations/0006_mcp_gating_policy.sql                # per-tenant policy
    - services/mcp/migrations/0007_mcp_pending_confirmations.sql        # confirm-mode ack store
    - services/mcp/migrations/0008_mcp_gating_decisions_log.sql         # full decision audit
    - services/mcp/src/gating/mod.rs                                    # gating orchestrator
    - services/mcp/src/gating/decision.rs                               # per-call decision logic
    - services/mcp/src/gating/policy.rs                                 # per-tenant policy loader
    - services/mcp/src/gating/confirm.rs                                # confirm-mode ack store
    - services/mcp/src/gating/elicit.rs                                 # elicit-mode delegator (FR-MCP-008)
    - services/mcp/src/gating/bypass.rs                                 # bypass-token check
    - services/mcp/src/gating/drift_detector.rs                         # nightly annotation drift job
    - services/mcp/src/handlers/tool_confirm.rs                         # POST /v1/mcp/tools/{tool_id}/confirm
    - services/mcp/src/handlers/gating_policy_admin.rs                  # tenant_admin policy CRUD
    - services/mcp/src/audit/gating_events.rs                           # 5 memory row builders
    - services/mcp/tests/gating_annotation_precedence_test.rs
    - services/mcp/tests/gating_destructive_requires_confirm_test.rs
    - services/mcp/tests/gating_readonly_fast_path_test.rs
    - services/mcp/tests/gating_confirm_ttl_test.rs
    - services/mcp/tests/gating_bypass_token_test.rs
    - services/mcp/tests/gating_audit_only_mode_test.rs
    - services/mcp/tests/gating_decision_enum_cardinality_test.rs
    - services/mcp/tests/gating_policy_tenant_admin_only_test.rs
    - services/mcp/tests/gating_drift_detection_test.rs
    - services/mcp/tests/gating_audit_emission_test.rs

  modified_files:
    - services/mcp/src/handlers/tools_call.rs                            # invoke gating before dispatch
    - services/mcp/src/server_registry.rs                                # store + serve annotations from registration
    - services/mcp/src/lib.rs                                            # mount confirm + policy admin routes

  allowed_tools:
    - file_read: services/mcp/**
    - file_write: services/mcp/{src,tests,migrations}/**
    - bash: cd services/mcp && cargo test gating

  disallowed_tools:
    - dispatch tool call without gating decision (per DEC-1052)
    - allow non-tenant_admin to update gating policy (per DEC-1058)
    - bypass-token without sev-2 audit (per DEC-1057)
    - skip annotation drift detection (per DEC-1059)
    - cache gating decisions cross-tenant (each tenant has independent policy)
    - allow elicit mode without FR-MCP-008 (per DEC-1055 — return 503 placeholder)

effort_hours: 6
sub_tasks:
  - "0.4h: 0006_mcp_gating_policy.sql + 0007 + 0008 migrations + RLS"
  - "0.3h: gating/policy.rs — per-tenant YAML loader + default policy"
  - "0.5h: gating/decision.rs — annotation precedence + mode resolution"
  - "0.4h: gating/confirm.rs — ack store + TTL + single-use"
  - "0.3h: gating/elicit.rs — FR-MCP-008 delegate (placeholder 503 at slice 2)"
  - "0.3h: gating/bypass.rs — scope claim check + sev-2 audit"
  - "0.3h: gating/drift_detector.rs — nightly compare"
  - "0.3h: handlers/tool_confirm.rs — POST ack endpoint"
  - "0.3h: handlers/gating_policy_admin.rs — tenant_admin CRUD"
  - "0.3h: audit/gating_events.rs — 5 builders"
  - "0.4h: wire-up — tools_call.rs invokes gating; server_registry.rs persists annotations"
  - "1.0h: tests — 10 test files covering precedence + confirm + bypass + drift + cardinality + audit"
  - "0.3h: integration smoke — exercise destructive tool through gating against real per-module server"

risk_if_skipped: "Without tool-annotation gating, every MCP tool call dispatches without confirmation — destructive tools (delete_project, drop_table, send_email_blast) fire on AI-driven hallucinated calls without user intent. MCP spec 2025-11-25 mandates client UX surfaces for destructive operations; gateway-side gating is the server-side safety net (clients may not implement). Without DEC-1041's defense-in-depth gating, a misbehaving AI agent deletes production data + leaves no audit trail of intent. Without DEC-1057's bypass-token audit, internal high-privilege callers go invisible to forensics. Without DEC-1059's drift detection, tools silently change from `readOnlyHint=true` to `destructiveHint=true` without operator notice. Without DEC-1044's per-tenant policy, all tenants get one-size-fits-all gating (regulated tenants need stricter; sandbox tenants need looser). The 6h effort lands the safety primitive that makes MCP operations forensically auditable + intent-confirmable."
---

## §1 — Description (BCP-14 normative)

The MCP service **MUST** ship tool-annotation gating at `services/mcp/src/gating/` enforcing MCP 2025-11-25 spec hint semantics (`readOnlyHint` / `destructiveHint` / `idempotentHint` / `openWorldHint`) at gateway entry, with per-tenant policy override, confirm-mode ack store, elicit-mode delegation to FR-MCP-008, bypass-token path, audit-only transition mode, nightly annotation drift detection, and 5 memory audit kinds.

1. **MUST** define the closed `mcp_gating_mode` enum at migration `0006`: `('auto_confirm','confirm','elicit')` per DEC-1043. CI cardinality test asserts 3.

2. **MUST** define the closed `mcp_gating_decision` enum at migration `0006`: `('auto_allowed','confirmed','elicited','bypassed','rejected_no_confirmation','rejected_expired_ack')` per DEC-1049. CI cardinality test asserts 6.

3. **MUST** define `mcp_gating_policy` table at migration `0006`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, policy_yaml TEXT NOT NULL, audit_only BOOLEAN NOT NULL DEFAULT false, version INT NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by_subject_id UUID NOT NULL, UNIQUE (tenant_id, version))`. Per-tenant versioned policy.

4. **MUST** define `mcp_pending_confirmations` table at migration `0007`: `(id BIGSERIAL PRIMARY KEY, caller_subject_id UUID NOT NULL, tool_id TEXT NOT NULL, tenant_id UUID NOT NULL, request_payload_sha256 CHAR(64) NOT NULL, expires_at TIMESTAMPTZ NOT NULL, consumed_at TIMESTAMPTZ, UNIQUE(caller_subject_id, tool_id, request_payload_sha256))`. Single-use ack store per DEC-1054.

5. **MUST** define `mcp_gating_decisions_log` table at migration `0008`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, caller_subject_id UUID NOT NULL, tool_id TEXT NOT NULL, decision mcp_gating_decision NOT NULL, annotations JSONB NOT NULL, gating_mode mcp_gating_mode NOT NULL, audit_only BOOLEAN NOT NULL, decided_at TIMESTAMPTZ NOT NULL DEFAULT now(), trace_id CHAR(32))`. Append-only per feature-request-audit skill rule 12.

6. **MUST** enforce RLS with both USING and WITH CHECK on all 3 gating tables. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

7. **MUST** intercept every `tools/call` MCP request at the gateway entry (FR-MCP-001 handler modification) BEFORE dispatching to the per-module server. The `tools_call` handler invokes `gating::decision::decide(caller, tool, payload)` first; on `rejected_*` decisions, returns 403 + `confirm_required` (with confirm endpoint URL) + emits `mcp.tool_gating_decision`. On `auto_allowed | confirmed | elicited | bypassed`, proceeds to dispatch.

8. **MUST** resolve gating mode per DEC-1053 annotation precedence (most restrictive wins):
    - If tool has `destructiveHint=true` OR `openWorldHint=true` → requires confirmation (mode from policy: `confirm` or `elicit`).
    - Else if `readOnlyHint=true` AND `destructiveHint=false` AND `openWorldHint=false` → `auto_confirm` (fast path).
    - Otherwise → policy-default (which itself defaults to `confirm`).
    - `idempotentHint` is informational only (does not change gating; surfaces in audit log).

9. **MUST** support per-tenant gating policy YAML per DEC-1044. Schema:
    ```yaml
    version: 1
    default_mode: confirm
    audit_only: false
    overrides:
      "cyberos.docs.delete_document":
        mode: elicit
      "cyberos.projects.read_issues":
        mode: auto_confirm
      # explicit override of annotation-derived default
    ```
   Policy stored in `mcp_gating_policy.policy_yaml`; loaded into memory at handler startup + hot-reloaded on NATS subject `cyberos.mcp.gating_policy.updated.<tenant_slug>`.

10. **MUST** expose `POST /v1/admin/tenants/{tenant_id}/mcp/gating-policy` per DEC-1058. Caller has `tenant_admin` role. Body: `{ policy_yaml }`. Handler:
    - Validates YAML schema (tool_id format `cyberos.<module>.<verb>_<noun>` per FR-MCP-003 + SEP-986).
    - INSERTs new policy row at version=max(version)+1.
    - Publishes NATS reload event.
    - Emits `mcp.tool_gating_policy_updated` sev-1.

11. **MUST** expose `POST /v1/mcp/tools/{tool_id}/confirm` for confirm-mode ack. Body: `{ tenant_id, tool_id, request_payload_sha256, signed_payload? }`. Handler:
    - Validates caller JWT (FR-AUTH-004).
    - Validates `request_payload_sha256` matches a recently-attempted tool call by this caller (matched within 60s — denial-of-future-action mitigation).
    - INSERTs `mcp_pending_confirmations` row with `expires_at = now() + 5 min` per DEC-1047.
    - Returns 201 + `{ confirm_token, expires_at }`. Caller re-issues tools/call within 5 min; confirm row consumed atomically.

12. **MUST** consume confirm token atomically per DEC-1054. The tools/call handler's gating decision:
    - Compute `request_payload_sha256`.
    - `SELECT ... FROM mcp_pending_confirmations WHERE caller_subject_id=$1 AND tool_id=$2 AND request_payload_sha256=$3 AND consumed_at IS NULL AND expires_at > now() FOR UPDATE`.
    - If row found: UPDATE `consumed_at=now()` + proceed.
    - If not found: return 403 + `confirm_required`.
    - `UNIQUE(caller_subject_id, tool_id, request_payload_sha256)` prevents double-use.

13. **MUST** delegate elicit mode to FR-MCP-008 per DEC-1055. At slice 2 of this FR (FR-MCP-008 not yet shipped), `elicit` mode returns `503 + { error: "elicitation_not_yet_supported", retry_when: "FR-MCP-008 ships in next slice" }`. When FR-MCP-008 lands, the elicit handler invokes `mcp::elicitation::request(caller, tool, prompt)` + awaits the response inline.

14. **MUST** support bypass token per DEC-1045 + DEC-1057. Callers with JWT `scope_grants` containing `mcp_gating_bypass` skip gating + record `bypassed` decision in audit log. The scope is granted ONLY to system-tenant callers (per FR-AUTH-004 + DEC-1045); cross-tenant grant attempt is rejected at JWT mint. Every bypass invocation emits `mcp.tool_gating_bypass_used` sev-2.

15. **MUST** support audit-only mode per DEC-1050. Tenant sets `mcp_gating_policy.audit_only=true`; gating decisions are computed + logged but tools always proceed (no 403). Surface mode via `decision='audit_only_allowed'` in audit log + sev-2 row `mcp.tool_gating_audit_only_mode_set` when policy is updated to enable audit-only.

16. **MUST** detect annotation drift nightly per DEC-1059. The `drift_detector.rs` scheduled job:
    - Loads each module's currently-registered annotations from `server_registry`.
    - Queries each module's `tools/list` endpoint to fetch the live annotations.
    - For each tool: compares declared annotations (registry) vs live annotations (server).
    - Drift detected → INSERTs `mcp_gating_drift_log` row + emits `mcp.tool_gating_annotation_drift` sev-1 with `(module, tool_id, registry_annotations, live_annotations, diff_fields)`.

17. **MUST** rate-limit confirmation requests at 100 confirm-mode requests/min/caller per DEC-1056. Excess returns `429 + Retry-After`.

18. **MUST** PII-scrub audit rows per DEC-1060 + feature-request-audit skill rule 18. `request_payload_sha256` only in memory chain; raw payload in MCP gateway logs (RLS-scoped, 7-day retention).

19. **MUST** thread W3C `traceparent` across `tools/call` → gating decision → confirm-ack → dispatch → response (feature-request-audit skill rule 22-24).

20. **MUST** emit 5 memory audit row kinds per DEC-1051:
    - `mcp.tool_gating_decision` (sev-3 high-volume; sampled at 1% via FR-OBS-006 tail-sampling)
    - `mcp.tool_gating_policy_updated` (sev-1)
    - `mcp.tool_gating_annotation_drift` (sev-1)
    - `mcp.tool_gating_bypass_used` (sev-2)
    - `mcp.tool_gating_audit_only_mode_set` (sev-2)

21. **MUST** preserve MCP-spec response shape on rejection. The 403 response body includes:
    ```json
    {
      "jsonrpc": "2.0", "error": {
        "code": -32000, "message": "confirm_required",
        "data": { "tool_id": "...", "confirm_endpoint": "https://...", "expires_at_after_confirm": "..." }
      },
      "id": <request_id>
    }
    ```

22. **MUST** support gating for FR-MCP-007 Tasks primitive — long-running tasks with `destructiveHint=true` require confirmation at task START; subsequent task status polls do NOT require re-confirmation (the consent is to the task as a unit). Documented as cross-FR contract.

23. **MUST NOT** cache gating decisions cross-tenant. Each `mcp_gating_policy` is per-tenant; per-tenant in-memory cache; tenant_id-scoped cache key.

24. **MUST NOT** allow bypass-token scope grant outside system-tenant per DEC-1045. FR-AUTH-004 JWT mint guard: if `tenant_id != system_tenant` AND `mcp_gating_bypass` in scope_grants → reject mint with `invalid_scope_for_tenant`.

25. **SHOULD** observe per-tenant gating-decision distribution via OTel histogram `mcp_gating_decision_total` labelled `(tenant_id, decision, tool_id)` for operator visibility on policy effectiveness.

---

## §2 — Why this design (rationale for humans)

**Why gateway-side gating (§1 #7, DEC-1052)?** Defense-in-depth. MCP clients (Claude Desktop, ChatGPT) MAY surface destructive-tool UX prompts, but they're not REQUIRED to. Server-side gating ensures the safety net exists regardless of client implementation quality. Single ingress point at the gateway means per-module servers don't each re-implement gating logic.

**Why annotation precedence (§1 #8, DEC-1053)?** MCP 2025-11-25 spec defines 5 hints; combining them ambiguously is bug-prone. Documenting "most restrictive wins" makes the decision algorithm deterministic. `readOnlyHint=true` AND `destructiveHint=true` together is contradictory (spec says don't); we err on the safe side (require confirmation).

**Why per-tenant policy (§1 #9, DEC-1044)?** Default conservative gating (confirm everything destructive) is right for most tenants. But sandbox tenants (development) need looser policy; regulated tenants (PII-heavy) need stricter (elicit instead of confirm). One global policy can't serve both. Per-tenant YAML lets each tenant_admin tune.

**Why confirm-token TTL of 5 min (§1 #12, DEC-1047)?** Confirmation is "user intends to do X now"; intent expires quickly. 5 min is long enough for the user to read the prompt + decide; short enough that an attacker who steals the confirm token has minimal window. Industry convention (Stripe payment intent confirm tokens: 5 min default).

**Why bypass-token forensic-loud (§1 #14, DEC-1057)?** Bypass is rare-by-design. Sev-2 audit on every bypass invocation makes "who is using the bypass and why" answerable in 30 seconds via dashboard. Silent bypass = forensic blindness.

**Why audit-only transition mode (§1 #15, DEC-1050)?** Switching from un-gated to gated breaks running workflows that didn't expect 403s. Audit-only mode runs the decision algorithm in shadow: produces audit rows showing what WOULD happen if gating were enforced. Tenant_admin reviews, tunes policy, then flips to enforced mode. Reduces rollout blast radius from "everything breaks" to "everything works, but now we know what would have been blocked".

**Why annotation drift detection nightly (§1 #16, DEC-1059)?** Tool annotations are forensic-critical (they drive gating decisions). A module that changes a tool from `readOnlyHint=true` to `destructiveHint=true` without updating the registry = silent removal of safety. Nightly comparison catches drift before it bites.

**Why confirm-token request_payload_sha256 matching (§1 #11)?** Without payload binding, an attacker who tricks the user into confirming "delete_project(123)" could re-use the confirm token for "delete_project(456)". Binding the ack to the payload hash means the confirmation is for THAT specific call, not for that tool generally.

**Why elicit-mode placeholder via 503 at slice 2 (§1 #13, DEC-1055)?** FR-MCP-008 (Elicitation) is on the roadmap but not yet shipped. Returning 503 instead of silently degrading to confirm mode (a) makes the missing capability visible, (b) doesn't violate the tenant_admin's policy choice (they picked elicit; we don't pretend confirm is equivalent), (c) gives operators a metric to track when FR-MCP-008 demand materialises.

**Why scope-claim approach for bypass (§1 #14, DEC-1045)?** Scope claims are issued at JWT mint (FR-AUTH-004 controlled); they're tamper-evident (signed); they're auditable (scope grants logged at mint time). Alternative (separate bypass-token endpoint) duplicates auth surface. Scope claim integrates with existing FR-AUTH-004 + FR-AUTH-101 RBAC.

**Why versioned policy (§1 #3)?** Same rationale as FR-PORTAL-002 brand pack versioning — tenant_admin wants rollback safety on commercial-grade config changes. Versioning is cheap (one row per save) + valuable (undo button).

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0006_mcp_gating_policy.sql
CREATE TYPE mcp_gating_mode AS ENUM ('auto_confirm','confirm','elicit');
CREATE TYPE mcp_gating_decision AS ENUM (
  'auto_allowed','confirmed','elicited','bypassed',
  'rejected_no_confirmation','rejected_expired_ack'
);

CREATE TABLE mcp_gating_policy (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  policy_yaml TEXT NOT NULL,
  audit_only BOOLEAN NOT NULL DEFAULT false,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by_subject_id UUID NOT NULL,
  UNIQUE (tenant_id, version)
);
CREATE TABLE mcp_gating_policy_active (
  tenant_id UUID PRIMARY KEY,
  active_policy_id BIGINT NOT NULL REFERENCES mcp_gating_policy(id),
  activated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE mcp_gating_policy ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_gating_policy_rls ON mcp_gating_policy
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
ALTER TABLE mcp_gating_policy_active ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_gating_policy_active_rls ON mcp_gating_policy_active
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON mcp_gating_policy FROM cyberos_app;
GRANT UPDATE (active_policy_id, activated_at) ON mcp_gating_policy_active TO cyberos_app;

-- 0007_mcp_pending_confirmations.sql
CREATE TABLE mcp_pending_confirmations (
  id BIGSERIAL PRIMARY KEY,
  caller_subject_id UUID NOT NULL,
  tool_id TEXT NOT NULL,
  tenant_id UUID NOT NULL,
  request_payload_sha256 CHAR(64) NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  consumed_at TIMESTAMPTZ,
  UNIQUE (caller_subject_id, tool_id, request_payload_sha256)
);
CREATE INDEX idx_mcp_confirm_lookup
  ON mcp_pending_confirmations(caller_subject_id, tool_id, request_payload_sha256)
  WHERE consumed_at IS NULL;
CREATE INDEX idx_mcp_confirm_expiry
  ON mcp_pending_confirmations(expires_at) WHERE consumed_at IS NULL;
ALTER TABLE mcp_pending_confirmations ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_pending_confirmations_rls ON mcp_pending_confirmations
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON mcp_pending_confirmations FROM cyberos_app;
GRANT UPDATE (consumed_at) ON mcp_pending_confirmations TO cyberos_app;
GRANT DELETE ON mcp_pending_confirmations TO cyberos_pruner;

-- 0008_mcp_gating_decisions_log.sql
CREATE TABLE mcp_gating_decisions_log (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  caller_subject_id UUID NOT NULL,
  tool_id TEXT NOT NULL,
  decision mcp_gating_decision NOT NULL,
  annotations JSONB NOT NULL,
  gating_mode mcp_gating_mode NOT NULL,
  audit_only BOOLEAN NOT NULL,
  decided_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id CHAR(32)
);
CREATE INDEX idx_gating_decisions_caller ON mcp_gating_decisions_log(caller_subject_id, decided_at DESC);
ALTER TABLE mcp_gating_decisions_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY mcp_gating_decisions_log_rls ON mcp_gating_decisions_log
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON mcp_gating_decisions_log FROM cyberos_app;
```

### 3.2 Rust types

```rust
// services/mcp/src/gating/mod.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type, serde::Serialize)]
#[sqlx(type_name = "mcp_gating_decision", rename_all = "snake_case")]
pub enum GatingDecision {
    AutoAllowed, Confirmed, Elicited, Bypassed,
    RejectedNoConfirmation, RejectedExpiredAck,
}

#[derive(Debug)]
pub struct ToolAnnotations {
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

pub async fn decide(ctx: &AppCtx, caller: &JwtClaims, tool_id: &str, payload: &[u8])
    -> Result<GatingDecision, GatingError>
{
    if caller.has_scope("mcp_gating_bypass") {
        audit_bypass(ctx, caller, tool_id).await;
        return Ok(GatingDecision::Bypassed);
    }
    let policy = ctx.gating.policy_for(caller.tenant_id).await?;
    let annotations = ctx.registry.annotations_for(tool_id).await?;
    let mode = policy.resolve_mode(tool_id, &annotations);
    let payload_sha = sha256_hex(payload);
    match mode {
        GatingMode::AutoConfirm => Ok(GatingDecision::AutoAllowed),
        GatingMode::Confirm => {
            if confirm_token_consume(ctx, caller, tool_id, &payload_sha).await? {
                Ok(GatingDecision::Confirmed)
            } else {
                Err(GatingError::ConfirmRequired { confirm_endpoint: confirm_url(tool_id) })
            }
        }
        GatingMode::Elicit => elicit::request(ctx, caller, tool_id, payload).await,
    }
}
```

### 3.3 REST endpoints

```text
POST   /v1/admin/tenants/{tenant_id}/mcp/gating-policy             (tenant_admin)
POST   /v1/admin/tenants/{tenant_id}/mcp/gating-policy/activate    (tenant_admin)
POST   /v1/mcp/tools/{tool_id}/confirm                              (any authenticated caller)
GET    /v1/admin/tenants/{tenant_id}/mcp/gating-decisions          (tenant_admin or system)
```

---

## §4 — Acceptance criteria

1. **Annotation precedence** — tool with `destructiveHint=true AND readOnlyHint=true` → requires confirm (most restrictive wins).
2. **Read-only fast path** — tool with `readOnlyHint=true, destructiveHint=false, openWorldHint=false` + policy default `confirm` → `auto_allowed` (annotation hint wins for read-only safety).
3. **Destructive requires confirm** — tool with `destructiveHint=true` invoked without prior ack → 403 + `confirm_required` + endpoint URL.
4. **Confirm ack consumed atomically** — same confirm token used twice → second invocation 403 `rejected_expired_ack`.
5. **Confirm TTL** — confirm with `expires_at=now() - 1min` (expired) → 403 + `rejected_expired_ack`.
6. **Bypass token** — caller with `mcp_gating_bypass` scope → `bypassed` decision; sev-2 memory row emitted.
7. **Bypass for non-system tenant rejected at mint** — JWT mint with `mcp_gating_bypass` for non-system tenant → mint rejected `invalid_scope_for_tenant`.
8. **Audit-only mode** — `audit_only=true` policy + destructive tool → call proceeds; audit row `decision='audit_only_allowed'` + `mcp.tool_gating_audit_only_mode_set` emitted on policy save.
9. **mcp_gating_decision enum cardinality** — 6 values exactly: `{auto_allowed, confirmed, elicited, bypassed, rejected_no_confirmation, rejected_expired_ack}`.
10. **mcp_gating_mode enum cardinality** — 3 values exactly: `{auto_confirm, confirm, elicit}`.
11. **Policy update tenant_admin only** — engagement_admin POST → 403; tenant_admin → 201 + sev-1 memory row.
12. **Elicit mode 503 placeholder** — policy with `mode: elicit` on a destructive tool → 503 `elicitation_not_yet_supported`.
13. **Rate limit on confirm endpoint** — 101st confirm in 60s → 429.
14. **Annotation drift detection** — registry says `readOnlyHint=true` but live tools/list says `destructiveHint=true` → drift row + sev-1 audit.
15. **JSON-RPC error shape** — 403 response body conforms to MCP JSON-RPC error shape with `confirm_endpoint` in `data`.
16. **Per-tenant policy isolation** — tenant A's policy doesn't affect tenant B's gating decisions.
17. **Bypass audit always emitted** — every bypass invocation → exactly one `mcp.tool_gating_bypass_used` row (no batching).
18. **PII scrub** — audit row carries `request_payload_sha256` only; raw payload not in memory chain.
19. **Trace_id threaded** — single trace_id present in tools/call span + gating decision row + downstream dispatch.
20. **Idempotent hint informational** — tool with `idempotentHint=true, destructiveHint=true` still requires confirm; idempotent hint surfaces in audit `annotations` JSONB but doesn't relax gating.

---

## §5 — Verification

### 5.1 `gating_annotation_precedence_test.rs`

```rust
#[tokio::test]
async fn destructive_overrides_readonly_when_both_true() {
    let ctx = TestContext::new().await;
    ctx.register_tool("cyberos.docs.weird_tool", ToolAnnotations {
        read_only_hint: true, destructive_hint: true, idempotent_hint: false, open_world_hint: false,
    }).await;
    let r = ctx.invoke_tool("cyberos.docs.weird_tool", json!({})).await;
    assert_eq!(r.status(), 403);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"]["message"], "confirm_required");
}
```

### 5.2 `gating_readonly_fast_path_test.rs`

```rust
#[tokio::test]
async fn readonly_tool_bypasses_confirmation() {
    let ctx = TestContext::with_default_policy_mode(GatingMode::Confirm).await;
    ctx.register_tool("cyberos.projects.list", ToolAnnotations {
        read_only_hint: true, destructive_hint: false, idempotent_hint: true, open_world_hint: false,
    }).await;
    let r = ctx.invoke_tool("cyberos.projects.list", json!({})).await;
    assert_eq!(r.status(), 200);
    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "mcp.tool_gating_decision"
        && r.payload["decision"] == "auto_allowed"));
}
```

### 5.3 `gating_destructive_requires_confirm_test.rs`

```rust
#[tokio::test]
async fn destructive_tool_requires_confirm_then_proceeds() {
    let ctx = TestContext::new().await;
    ctx.register_tool("cyberos.docs.delete", ToolAnnotations {
        read_only_hint: false, destructive_hint: true, idempotent_hint: false, open_world_hint: false,
    }).await;
    let payload = json!({"doc_id": "abc"});

    // First call: 403 + confirm endpoint
    let r1 = ctx.invoke_tool("cyberos.docs.delete", payload.clone()).await;
    assert_eq!(r1.status(), 403);

    // Confirm
    let payload_sha = sha256_hex(&serde_json::to_vec(&payload).unwrap());
    let confirm = ctx.post_confirm("cyberos.docs.delete", &payload_sha).await;
    assert_eq!(confirm.status(), 201);

    // Re-invoke: 200
    let r2 = ctx.invoke_tool("cyberos.docs.delete", payload.clone()).await;
    assert_eq!(r2.status(), 200);

    // Re-use ack: 403
    let r3 = ctx.invoke_tool("cyberos.docs.delete", payload).await;
    assert_eq!(r3.status(), 403);
}
```

### 5.4 `gating_confirm_ttl_test.rs`

```rust
#[tokio::test]
async fn expired_confirm_token_rejected() {
    let ctx = TestContext::new().await;
    ctx.register_tool("cyberos.docs.delete", destructive_annotations()).await;
    let payload = json!({"doc_id": "abc"});
    let payload_sha = sha256_hex(&serde_json::to_vec(&payload).unwrap());
    ctx.insert_pending_confirmation_expiring(
        ctx.caller_subject_id, "cyberos.docs.delete", &payload_sha,
        Utc::now() - Duration::minutes(1)
    ).await;

    let r = ctx.invoke_tool("cyberos.docs.delete", payload).await;
    assert_eq!(r.status(), 403);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"]["message"], "rejected_expired_ack");
}
```

### 5.5 `gating_bypass_token_test.rs`

```rust
#[tokio::test]
async fn bypass_scope_skips_gating_and_audits() {
    let ctx = TestContext::with_system_tenant_caller_scope("mcp_gating_bypass").await;
    ctx.register_tool("cyberos.docs.delete", destructive_annotations()).await;
    let r = ctx.invoke_tool("cyberos.docs.delete", json!({"doc_id": "x"})).await;
    assert_eq!(r.status(), 200);

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "mcp.tool_gating_bypass_used" && r.severity == 2));
}

#[tokio::test]
async fn non_system_tenant_cannot_grant_bypass_scope() {
    let ctx = TestContext::new().await;
    let result = ctx.auth.mint_jwt_for(ctx.regular_tenant, vec!["mcp_gating_bypass"]).await;
    assert!(matches!(result, Err(AuthError::InvalidScopeForTenant)));
}
```

### 5.6 `gating_audit_only_mode_test.rs`

```rust
#[tokio::test]
async fn audit_only_mode_allows_destructive_but_logs() {
    let ctx = TestContext::with_audit_only_policy().await;
    ctx.register_tool("cyberos.docs.delete", destructive_annotations()).await;
    let r = ctx.invoke_tool("cyberos.docs.delete", json!({"doc_id": "x"})).await;
    assert_eq!(r.status(), 200);

    let audit = ctx.memory_rows().await;
    let dec = audit.iter().find(|r| r.kind == "mcp.tool_gating_decision").unwrap();
    assert_eq!(dec.payload["audit_only"], true);
}
```

### 5.7 `gating_decision_enum_cardinality_test.rs`

```rust
#[tokio::test]
async fn mcp_gating_decision_has_6_values() {
    let ctx = TestContext::new().await;
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::mcp_gating_decision))::text"
    ).fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels; labels.sort();
    assert_eq!(labels, vec![
        "auto_allowed","bypassed","confirmed","elicited",
        "rejected_expired_ack","rejected_no_confirmation"
    ]);
}
```

### 5.8 `gating_drift_detection_test.rs`

```rust
#[tokio::test]
async fn drift_between_registry_and_live_detected() {
    let ctx = TestContext::new().await;
    ctx.register_tool_in_registry("cyberos.docs.weird", readonly_annotations()).await;
    ctx.set_live_tool_annotations("cyberos.docs.weird", destructive_annotations()).await;
    ctx.run_drift_detector().await;

    let audit = ctx.memory_rows().await;
    let drift = audit.iter().find(|r| r.kind == "mcp.tool_gating_annotation_drift").unwrap();
    assert_eq!(drift.severity, 1);
    assert!(drift.payload["diff_fields"].as_array().unwrap()
        .iter().any(|f| f == "destructiveHint"));
}
```

### 5.9 `gating_policy_tenant_admin_only_test.rs`

```rust
#[tokio::test]
async fn engagement_admin_cannot_set_gating_policy() {
    let ctx = TestContext::new().await;
    let eng_admin_token = ctx.mint_jwt_with_role(ctx.tenant, "engagement_admin");
    let r = ctx.post("/v1/admin/tenants/{tenant}/mcp/gating-policy").bearer_auth(eng_admin_token)
        .body("default_mode: confirm").send().await.unwrap();
    assert_eq!(r.status(), 403);
}
```

### 5.10 `gating_audit_emission_test.rs`

```rust
#[tokio::test]
async fn full_lifecycle_emits_5_kinds() {
    let ctx = TestContext::new().await;
    ctx.tenant_admin_set_policy_with_audit_only(true).await;     // audit_only_mode_set
    ctx.tenant_admin_set_policy_with_audit_only(false).await;    // policy_updated
    ctx.register_tool("cyberos.docs.weird", readonly_annotations()).await;
    ctx.set_live_tool_annotations("cyberos.docs.weird", destructive_annotations()).await;
    ctx.run_drift_detector().await;                              // annotation_drift
    ctx.invoke_tool_with_bypass("cyberos.docs.weird").await;     // bypass_used + decision
    let kinds: Vec<&str> = ctx.memory_rows().await.iter().map(|r| r.kind.as_str()).collect();
    assert!(kinds.contains(&"mcp.tool_gating_policy_updated"));
    assert!(kinds.contains(&"mcp.tool_gating_audit_only_mode_set"));
    assert!(kinds.contains(&"mcp.tool_gating_annotation_drift"));
    assert!(kinds.contains(&"mcp.tool_gating_bypass_used"));
    assert!(kinds.contains(&"mcp.tool_gating_decision"));
}
```

---

## §6 — Implementation skeleton

### 6.1 Policy mode resolver

```rust
// services/mcp/src/gating/policy.rs
impl GatingPolicy {
    pub fn resolve_mode(&self, tool_id: &str, ann: &ToolAnnotations) -> GatingMode {
        if let Some(override_mode) = self.overrides.get(tool_id) {
            return *override_mode;
        }
        if ann.destructive_hint || ann.open_world_hint {
            return self.default_destructive_mode.unwrap_or(GatingMode::Confirm);
        }
        if ann.read_only_hint && !ann.destructive_hint && !ann.open_world_hint {
            return GatingMode::AutoConfirm;
        }
        self.default_mode
    }
}
```

### 6.2 Confirm consumption (atomic)

```rust
// services/mcp/src/gating/confirm.rs
pub async fn consume_pending(
    pool: &PgPool, caller: Uuid, tool_id: &str, payload_sha: &str
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE mcp_pending_confirmations
         SET consumed_at = now()
         WHERE caller_subject_id = $1 AND tool_id = $2 AND request_payload_sha256 = $3
               AND consumed_at IS NULL AND expires_at > now()
         RETURNING id"
    ).bind(caller).bind(tool_id).bind(payload_sha).fetch_optional(pool).await?;
    Ok(result.is_some())
}
```

### 6.3 Drift detector

```rust
pub async fn detect_drift(ctx: &AppCtx) -> Result<Vec<DriftEvent>, DriftError> {
    let modules = ctx.registry.list_modules().await?;
    let mut drifts = Vec::new();
    for module in modules {
        let registered = ctx.registry.annotations_by_module(&module).await?;
        let live = ctx.fetch_live_tools_list(&module).await?;
        for tool in registered.tools {
            let live_ann = live.iter().find(|t| t.tool_id == tool.tool_id);
            if let Some(live_ann) = live_ann {
                let diff = compare_annotations(&tool.annotations, &live_ann.annotations);
                if !diff.is_empty() {
                    drifts.push(DriftEvent { module: module.clone(), tool_id: tool.tool_id.clone(), diff });
                }
            }
        }
    }
    for d in &drifts {
        emit_audit(ctx, "mcp.tool_gating_annotation_drift", json!(d)).await;
    }
    Ok(drifts)
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **FR-MCP-001** spec compliance — tools/call dispatch path; gating intercepts here.

**Cross-module (related_frs):**
- **FR-MCP-002** Per-module registration — annotations stored in registry.
- **FR-MCP-003** SEP-986 naming — tool_id format validated in policy.
- **FR-MCP-004** OAuth 2.1 PKCE — JWT bearer with scope_grants for bypass.
- **FR-MCP-005** PRM — gating-supported tool scopes appear in per-module PRM.
- **FR-MCP-007** Tasks primitive — destructive tasks confirm at start; subsequent polls don't re-confirm.
- **FR-MCP-008** Elicitation — `elicit` mode delegates here (slice-2 placeholder).
- **FR-AUTH-004** JWT mint — bypass-scope guard at mint.
- **FR-AI-003** memory audit-row bridge — 5 new kinds.
- **FR-MEMORY-111** PII scrubbing — payload SHA only in chain.
- **FR-OBS-007** Auto-runbook — sev-1 drift + policy events route to CHAT.

**Downstream (blocks):** None at this slice.

---

## §8 — Example payloads

### 8.1 `mcp.tool_gating_decision` memory row (rejected)

```json
{
  "kind": "mcp.tool_gating_decision",
  "severity": 3,
  "tenant_id": "8a2f...",
  "actor_id": "system.mcp.gating",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "caller_subject_id": "7c4e...",
    "tool_id": "cyberos.docs.delete_document",
    "decision": "rejected_no_confirmation",
    "annotations": {
      "destructiveHint": true, "openWorldHint": false,
      "readOnlyHint": false, "idempotentHint": false
    },
    "gating_mode": "confirm",
    "audit_only": false,
    "request_payload_sha256": "9c4e7a8b6d2f1e3a..."
  }
}
```

### 8.2 403 confirm-required response

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "confirm_required",
    "data": {
      "tool_id": "cyberos.docs.delete_document",
      "confirm_endpoint": "https://api.cyberos.world/v1/mcp/tools/cyberos.docs.delete_document/confirm",
      "expires_after_confirm_seconds": 300
    }
  },
  "id": 42
}
```

### 8.3 Policy YAML

```yaml
version: 1
default_mode: confirm
audit_only: false
overrides:
  "cyberos.docs.delete_document":
    mode: elicit
  "cyberos.projects.list_issues":
    mode: auto_confirm
  "cyberos.crm.bulk_email":
    mode: elicit
```

### 8.4 `mcp.tool_gating_annotation_drift` memory row

```json
{
  "kind": "mcp.tool_gating_annotation_drift",
  "severity": 1,
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "actor_id": "system.mcp.drift_detector",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T03:00:12.118Z",
  "payload": {
    "module": "docs",
    "tool_id": "cyberos.docs.export_audit",
    "registry_annotations": {
      "readOnlyHint": true, "destructiveHint": false,
      "idempotentHint": true, "openWorldHint": false
    },
    "live_annotations": {
      "readOnlyHint": false, "destructiveHint": false,
      "idempotentHint": true, "openWorldHint": true
    },
    "diff_fields": ["readOnlyHint", "openWorldHint"]
  }
}
```

---

## §9 — Open questions

All resolved for slice 2. Deferred:

- **Deferred:** Per-caller policy overrides (one user trusted, another not) — slice 3.
- **Deferred:** Time-bounded bypass scopes (bypass for next 1h) — slice 3.
- **Deferred:** Policy templates (compliance presets: HIPAA-strict, SOC2-balanced) — slice 3.
- **Deferred:** Confirmation via Out-Of-Band channel (Slack approval) — slice 3, FR-MCP-2xx.
- **Deferred:** Multi-actor confirmation (M-of-N approval for destructive ops) — slice 3.
- **Deferred:** Dry-run mode (show what gating WOULD do without actually invoking) — slice 3.
- **Deferred:** UI for policy editing (vs YAML) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Tool registered without annotations | registry validation | Reject registration with `missing_annotations` | Module fixes registration |
| Tool annotations malformed (non-boolean values) | schema validation | Reject with `invalid_annotation_value` | Module fixes |
| Confirm token consumed by different caller | UNIQUE constraint scoped to caller_subject_id | Original caller's confirm not affected; cross-caller confirm impossible | Inherent isolation |
| Pending confirmations table grows unbounded | scheduled prune at expires_at | TTL prune via cyberos_pruner role daily | Inherent — prune job |
| Policy YAML parse error | YAML parser | 400 + `policy_yaml_invalid` + line/column hint | tenant_admin fixes YAML |
| Policy YAML references unknown tool_id | validation against registry | 400 + `unknown_tool_id` + tool_id | Use valid tool_id from registry list |
| Bypass scope granted to non-system tenant | JWT mint guard | mint rejected `invalid_scope_for_tenant` | Caller's tenant cannot have bypass; admin re-mints without |
| Audit-only mode set + later policy change forgets to disable | activation pointer + audit | Tenant_admin sees `audit_only=true` indicator in dashboard | Tenant_admin reviews; flips to enforced |
| Elicit mode requested but FR-MCP-008 not shipped | mode resolver | 503 + `elicitation_not_yet_supported` | Tenant_admin temporarily falls back to confirm mode |
| Drift detector finds widespread drift (module re-registered) | drift event count threshold | sev-1 alert; ops review | Module-side fix + re-register |
| Tool removed from registry but cached gating decision references it | cache invalidation on registry change | Cached decision rejected on next use; re-resolved | Inherent — cache TTL |
| Confirm endpoint rate-limit hit | sliding-window | 429 + Retry-After | Caller backs off; investigate misbehaving client |
| JWT scope_grants tampered (impossible with signed JWTs) | signature verify | 401 at JWT validate | Inherent JWT security |
| Same tool registered with different annotations across module instances | drift detector | sev-1 with all instances' annotations | Module deployment process audit |
| `mcp_pending_confirmations` table missing index | slow query OBS alert | sev-3; query degraded | Migration adds index |
| Cross-tenant policy lookup attempt | RLS scope | Returns 0 rows; falls through to default global policy | Inherent isolation |
| Empty annotations on tool (all false) | resolver | Default to policy `confirm` mode (conservative) | Module should explicitly set hints |
| Tenant has no policy row | resolver | Falls through to platform-default `confirm` | tenant_admin creates policy |
| Policy version rollback to a previous version | activate handler | UPSERT active_policy_id; old version still in table | Inherent versioning |
| Audit log row insert fails post-decision | Postgres error | Sev-2 alert; decision proceeds (FAIL-OPEN on audit — alternative is FAIL-CLOSED which denies all tool calls on audit issue) | Operator investigates Postgres |

---

## §11 — Implementation notes

**§11.1** The decision algorithm is pure (no I/O); separable for testing without mocks. Side effects (audit emit, consume row) factored into outer orchestrator.

**§11.2** Pending confirmations are pruned by `cyberos_pruner` role's DELETE grant + scheduled job (same pattern as FR-TEN-003 `stripe_api_calls`).

**§11.3** Bypass scope is granted ONLY via FR-AUTH-004's admin mint endpoint with explicit operator approval; not a self-service grant.

**§11.4** Policy YAML validation includes tool_id existence check against current registry; future tools added after policy save are subject to default policy until policy updated.

**§11.5** Drift detector schedule: nightly at 03:00 UTC (low-traffic window); runs <2 min for 100 modules × 50 tools.

**§11.6** The `idempotentHint` annotation is informational only per spec. Documented in audit row's `annotations` JSONB so operators see the full picture.

**§11.7** Annotation-precedence algorithm: documented + tested + reviewed — the 4-line resolver in §6.1 is THE algorithm; no alternative paths.

**§11.8** Confirm token request_payload_sha256 binding uses SHA-256 of canonical-JSON of payload; ensures key-order-independent matching.

**§11.9** Per-tenant policy hot-reload via NATS subject `cyberos.mcp.gating_policy.updated.<tenant_slug>`; in-memory cache invalidated on receipt + reload from Postgres.

**§11.10** Decision audit at sev-3 sampled at 1% via FR-OBS-006 tail-sampling for routine flow; rejected decisions emitted at 100% (security-relevant).

**§11.11** The `confirm_token` returned by POST confirm is the row's `id`; opaque to callers; not interpretable.

**§11.12** Elicit mode at slice 2 returns 503 not 501 (501 "Not Implemented" would be lying — it IS implemented as a placeholder); 503 "Service Unavailable" matches the "elicitation server not yet running" semantic.

**§11.13** Per-tenant audit-only mode is a transition path; the audit-only setting itself has a `set_at` column (not added at slice 2; add at slice 3 if audit-only durations need tracking).

**§11.14** The drift detector compares JSONB annotations field-by-field; nullable fields (e.g., `idempotentHint` may be omitted in some clients) treated as `false` for diff purposes.

**§11.15** Bypass invocations are NOT rate-limited (system-tenant callers may legitimately invoke many gated tools); the audit row volume is the limit.

**§11.16** Policy YAML schema versioning: slice 2 `version: 1`; slice 3 may add `version: 2` with backward-compat parser.

**§11.17** The MCP JSON-RPC error code `-32000` is "server error" per JSON-RPC 2.0 spec §5.1; chosen for confirm_required because it's a server-side policy enforcement.

**§11.18** Annotation hint defaults per MCP spec: all 4 hints default `false`; omission of a hint = explicit `false`. Our resolver treats undefined as `false` consistently.

**§11.19** Per-tenant audit log retention: 90 days (matches FR-TEN-101 signup_sessions pattern); after 90d, summary statistics retained, individual rows pruned.

---

*End of FR-MCP-006 spec.*
