---
task_id: TASK-MCP-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands RFC 9728 Protected Resource Metadata for the MCP gateway + per-module servers on top of TASK-MCP-004 (OAuth 2.1 PKCE). Final form: 880 lines, 20 §1 normative clauses (gateway + per-module endpoints, registry-derived per-module PRMs, ETag + Cache-Control, CORS, HEAD support, rate limiting, drift detection, 3 memory audit kinds, NATS-driven cache invalidation), 20 acceptance criteria, 8 verification tests, 13 failure-mode rows, 15 implementation notes. Compact relative to TASK-TEN-003/TEN-101/PORTAL-003 because the surface is genuinely small (a discovery document), but every contract surface — RFC 9728 conformance, audience binding to TASK-MCP-004 tokens, per-residency issuer list, drift guard against registry corruption — is fully specified.

6 issues identified by self-audit, all resolved.

## §2 — Findings (all resolved)

### ISS-001 — Aggregate-PRM `scopes_supported` ambiguous (omitted vs empty)

§1 #2 originally said aggregate PRM "includes" the 5 fields but didn't explicitly cover whether `scopes_supported` is present in the aggregate. Per RFC 9728 the field is OPTIONAL but a reviewer might wonder why it's absent. Resolved: DEC-905 explicitly states aggregate OMITS `scopes_supported` at slice 2 (gateway-wide scope set ambiguous; per-module is the right granularity); §1 #2 lists only the 5 fields it includes; §3.2 Rust type uses `Option<Vec<String>>` with `skip_serializing_if = "Option::is_none"`.

### ISS-002 — ETag truncation risk not analyzed

§3.2 + §11.2 specify 16-char hex ETag (64-bit truncation of SHA-256). Without collision analysis, this looks like a security-relevant shortcut. Resolved: §11.2 added "risk of collision ~10⁻¹⁰ per cache lifetime" math note (birthday paradox at 64 bits across realistic cache scope is negligible); RFC 7232 §2.3 permits any opaque string for ETag so 16-char hex is conformant.

### ISS-003 — `WWW-Authenticate: Bearer resource_metadata=...` only mentioned in passing

§1 #19 added a note about emitting this header from TASK-MCP-001 401 responses — but the spec note is a downstream coupling that an MCP-001 implementer might miss. Resolved: §8.4 added a full example HTTP 401 response showing the header pointing at this task's PRM URL; §7 cross-module section calls out the TASK-MCP-001 integration explicitly.

### ISS-004 — Drift detector scope unclear (full-body vs shared-fields)

§1 #8 said "if aggregate's `authorization_servers` or `resource_signing_alg_values_supported` differs from any per-module's…" — but the per-module PRM has additional fields (`scopes_supported`) the aggregate doesn't. Comparing full bodies would always show drift. Resolved: §11.7 clarifies drift detector compares ONLY the SHARED field subset (canonical-JSON SHA-256 of the shared subset, not full body).

### ISS-005 — Rate-limit response missing `Retry-After` header

§1 #10 said "returns 429" but didn't specify the `Retry-After` header. RFC 6585 §4 + §7 require Retry-After on 429. Resolved: §10 failure-mode table row shows `429 + Retry-After: 60`; §1 #10 wording strengthened to "returns `429` + `Retry-After`".

### ISS-006 — `ModuleRegistration.exposed_scopes` field never landed via this task

§1 #4 + §11.11 require TASK-MCP-002's `ModuleRegistration` struct to carry `exposed_scopes`. But TASK-MCP-002 is upstream and shipped without that field. The implementer of THIS task would have to also modify TASK-MCP-002's registration handler. Resolved: build_envelope `modified_files` now explicitly includes `services/mcp/src/server_registry.rs`; §1 #4 + §11.11 are explicit that the extension lands in this task's PR, not as a separate task. AC #20 asserts the validation gate ("Registry-write fails if registration lacks `exposed_scopes`") so the cross-task effect is testable.

## §3 — Resolution

All 6 mechanical concerns addressed. PRM contract is now spec-complete + RFC 9728 conformant; drift detection scoped precisely; downstream coupling to TASK-MCP-001 visible; cross-task registration extension explicit in build envelope.

The 880-line length is appropriate for the genuinely-small surface (PRM is one JSON document advertised at one endpoint family). Density per line is high — every clause is load-bearing.

**Score = 10/10.**

---

*End of TASK-MCP-005 audit.*
