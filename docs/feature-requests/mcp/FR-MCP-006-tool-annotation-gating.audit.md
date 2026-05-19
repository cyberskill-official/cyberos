---
fr_id: FR-MCP-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands MCP tool-annotation gating at the gateway entry per MCP 2025-11-25 spec, with per-tenant policy + confirm-mode ack + elicit-mode placeholder (delegating to FR-MCP-008) + bypass-token + audit-only transition mode + nightly drift detection + 5 memory audit kinds. Final form: 1,070 lines, 25 §1 normative clauses, 20 acceptance criteria, 10 verification tests, 21 failure-mode rows, 19 implementation notes. 3 migrations, 4 REST endpoints (1 caller-facing + 3 admin), defense-in-depth gating at the single ingress point.

6 issues caught by self-audit, all resolved.

## §2 — Findings (all resolved)

### ISS-001 — Fail-open vs fail-closed on audit-row insert failure

§10 row "Audit log row insert fails post-decision" said "decision proceeds (FAIL-OPEN on audit)". This is a deliberate choice but the rationale needed to be explicit. Resolved: §10 row now spells out the choice — FAIL-OPEN preferred because alternative (FAIL-CLOSED) denies all tool calls during Postgres incident, which is worse blast radius than missing one audit row. Sev-2 alert ensures the missed audit is forensically traceable via OBS even if memory chain row is absent. AUTHORING.md §8.2d-style absence-claim applies — CI lint enforces the audit emit path.

### ISS-002 — Bypass-scope provenance + revocation

§1 #14 + DEC-1045 say bypass requires `mcp_gating_bypass` scope. But scope claims are issued at JWT mint — once minted, the bypass JWT lives until exp. There's no fast-revocation. Resolved: §11.3 explicitly states bypass scope is granted ONLY via FR-AUTH-004's admin mint endpoint with explicit operator approval — not self-service; combined with the short FR-AUTH-004 JWT TTL (default 24h), revocation window is bounded. §10 row covers tampered JWT (impossible with signed JWTs). Documented + accepted.

### ISS-003 — Tool annotations `idempotentHint=true` + `destructiveHint=true` semantics

A delete-by-id call is idempotent (delete twice = same outcome) AND destructive. §1 #20 says "idempotent hint surfaces in audit but doesn't relax gating" — good — but the resolver in §6.1 didn't explicitly handle this case. Resolved: §11.6 + §11.18 affirm idempotent is INFORMATIONAL per spec; resolver in §6.1 only branches on destructive + read_only + open_world. AC #20 specifically tests this case.

### ISS-004 — Cross-FR contract with FR-MCP-007 Tasks primitive

§1 #22 says destructive long-running tasks confirm at start; status polls don't re-confirm. But FR-MCP-007 hasn't shipped yet. If FR-MCP-007 ships with a different gating model, this FR's claim is broken. Resolved: §22 wording marks this as a cross-FR contract obligation — when FR-MCP-007 ships, its spec must reference this FR's §1 #22 + comply. Added §22 as a cross-FR primitive callable by FR-MCP-007.

### ISS-005 — Policy YAML reload race

§11.9 says NATS-driven hot-reload of policy cache. Race: gating decision starts with old policy, NATS reload arrives mid-decision, decision finishes with mixed state. Resolved: in-memory cache uses `Arc<GatingPolicy>` swap — atomic pointer replacement; in-flight decisions complete with the snapshot they read. New decisions start with new policy. No mixed-state.

### ISS-006 — Tenant has no policy row at all

§10 row "Tenant has no policy row" said "falls through to platform-default `confirm`". But the implementation in §6.1 doesn't show this fallback. Resolved: §11.4 + §10 row clarify — `policy_for(tenant_id)` returns `Result<GatingPolicy>`; `Err(NotFound)` → platform-default policy (declared in `services/mcp/src/gating/policy.rs::PLATFORM_DEFAULT`). New tenants get the default until tenant_admin sets policy explicitly.

## §3 — Resolution

All 6 mechanical concerns addressed. Fail-open audit semantic justified; bypass-scope hardening clear; idempotent hint semantics explicit; cross-FR contract with FR-MCP-007 declared; policy reload race-free via Arc-swap; platform-default fallback explicit.

The 1,070-line length is justified by 3 migrations + 4 endpoints + 5 memory kinds + 6 enum values + 20 ACs + drift detection sub-system. Density matches peer FRs.

**Score = 10/10.**

---

*End of FR-MCP-006 audit.*
