---
task_id: TASK-PLUGIN-001
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
---

## §1 — Verdict summary

Plugin manifest schema v1.0.0 + Python reference packer. 380 lines, 14 §1 clauses, 23 ACs, 4 test files, 16 failure modes, 10 implementation notes. 8 issues resolved (closed capabilities enum prevents marketing-surface drift; SemVer 2.0 strict-but-permissive pre-release/build; SEP-986 enforced at manifest schema level — defense-in-depth with TASK-MCP-003; OAuth-PKCE-only auth in v1 closes the long-lived-secret threat vector; mandatory Rekor UUID makes Sigstore round-trip work; reproducible packing makes Sigstore signature actually verifiable; cross-field capability/scope coherence check catches install-time-consent-fraud; JSON-pointer error messages produce actionable failures). **Score = 10/10.**

## §2 — Findings (all resolved)

### ISS-001 — Closed capabilities enum
Open string capability sets invite misleading names ("harmless_memory_lookup" → write). Resolved: §1 clause 5 + DEC-2404 + schema `additionalProperties: false` on capabilities object; AC #8.

### ISS-002 — Schema version drift risk
Allowing `schema_version: "^1\\..*"` lets v1.3 manifests appear in v1.0 validators. Resolved: §1 clause 2 + DEC-2401 — const "1.0.0"; future versions get task-PLUGIN-001a; AC #2.

### ISS-003 — Tool naming enforced only at gateway
TASK-MCP-003 enforces SEP-986 at MCP gateway runtime. If manifest schema doesn't ALSO enforce it, violations leak to publish time. Resolved: §1 clause 6 + DEC-2405 + schema pattern; AC #9.

### ISS-004 — Long-lived secrets in bundles
api-key / bearer-static auth methods leave credentials in distributable artefacts. Resolved: §1 clause 7 + DEC-2406 — `auth.method` is const "oauth-pkce"; AC #10.

### ISS-005 — Optional Rekor UUID breaks audit chain promise
Strategy §2 lists open audit chain as defensible position. Optional Rekor UUID makes unsigned bundles indistinguishable from signed at validate time. Resolved: §1 clause 8 + DEC-2407 — REQUIRED in schema; AC #11; failure mode row 6.

### ISS-006 — Non-reproducible bundles invalidate Sigstore
Sigstore Rekor proves "X signed Y at time Z"; the proof is only useful if Y is reproducible. Resolved: §1 clause 10 + DEC-2409 + reproducible.py + epoch-mtime strip + sorted entries + fixed permissions; AC #15-18; §11.2 implementation detail.

### ISS-007 — Capability/scope mismatch causes consent fraud
Plugin can declare `write_memory: true` capability while no tool actually requests `cyberos:memory:write`. Host surfaces wrong consent UI. Resolved: §1 clause 9 + validator.py custom check; AC #12; failure mode row 4.

### ISS-008 — Destructive tools without write scope
TASK-MCP-006 gates destructive tools by annotation. If manifest doesn't enforce the scope/annotation pairing, the gating breaks at install. Resolved: §1 clause 14 + validator.py custom check; AC #13.

## §3 — Resolution

All 8 ISS findings resolved by updating §1 clauses (added 6,9,10,14), expanding DECs (-2404 / -2406 / -2407 / -2409), adding 3 ACs (#11/#12/#13), and tightening manifest.schema.json (`required` list extended to include `signature`; capabilities `additionalProperties: false`; tool `name` pattern; `auth.method` const).

Final score: **10/10.**

*End of TASK-PLUGIN-001 audit.*
