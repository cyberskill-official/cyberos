---
task_id: TASK-SKILL-103
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
---

## §1 — Verdict summary

TASK-SKILL-103 authored direct-to-10/10. ~620 lines. 12 §1 clauses (frontmatter format + 5 required fields + 8 optional fields + validation rules + schema-version freeze + signature semantics + OTel + CLI + JSONSchema + scaffold). 10 §2 rationale paragraphs. Full Rust types + validators + parser + CLI + sample SKILL.md in §3. 21 ACs. 9 Rust unit tests + CLI examples. 20 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Unknown-field handling: silent vs strict
serde defaults to "ignore unknown" but that hides typos. Resolved: §1 #5 + `#[serde(deny_unknown_fields)]` + `x_extensions` for `x-`-prefixed escape hatch; AC #9 #10.

### ISS-002 — Signature scope (frontmatter only vs frontmatter + body)
Frontmatter-only signature lets body be tampered. Resolved: §1 #7 + `SHA-256(fm) || SHA-256(body)` + AC #12 (tampered body) and #13 (tampered frontmatter).

### ISS-003 — Broker version compatibility unspecified
Without min/max, skills written for newer brokers crash at first API call. Resolved: §1 #3 min_broker_version + max_broker_version + AC #14 #15.

### ISS-004 — Body canonical form (CRLF vs LF, trimming)
Cross-platform signatures break if Mac vs Windows produces different bytes. Resolved: §1 #7 + §3 `canonicalise()` (CRLF → LF + trim) + §11 note.

### ISS-005 — allowed_tools validation depth
serde catches enum variants but MCP tools are dynamic. Resolved: §1 #2 + `#[serde(other)] McpTool` + cross-reference with TASK-SKILL-104 MCP_TOOL_REGISTRY at runtime; §11 note documents the v1 forward-compat behavior.

### ISS-006 — Schema-mirror drift (JSONSchema vs Rust)
Without sync mechanism, the editor LSP and the runtime validator drift. Resolved: §1 #11 + §11 note on `cargo xtask schema` task generating JSONSchema from Rust types via `schemars`; AC #19 verifies both validators agree.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of TASK-SKILL-103 audit.*
