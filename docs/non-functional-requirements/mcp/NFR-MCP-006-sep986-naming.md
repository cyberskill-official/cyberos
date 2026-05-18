---
id: NFR-MCP-006
title: "MCP SEP-986 naming compliance — all server/tool identifiers MUST match SEP-986 regex"
module: MCP
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of names pass SEP-986 validator; non-conforming names blocked at registration"
owner: CTO
created: 2026-05-18
related_frs: [FR-MCP-003]
---

## §1 — Statement (BCP-14 normative)

1. Every MCP server, tool, resource, and prompt name **MUST** conform to the SEP-986 naming pattern: `^[a-z][a-z0-9_-]{0,63}$` (lowercase ASCII, digits, underscore, hyphen, leading letter, max 64 chars).
2. Name registration (in the platform's MCP registry) **MUST** reject non-conforming names at gateway ingress.
3. Servers loading tools with non-conforming names at runtime **MUST** refuse to publish those tools and log a structured warning.
4. Names **MUST NOT** collide within a server's namespace; the gateway dedupe-checks at registration.
5. Renaming is a breaking change — the old name is retained as an alias for ≥ 30 days post-rename.

## §2 — Why this constraint

SEP-986 was adopted upstream specifically to remove ambiguity around case-insensitive matching, Unicode lookalikes, and ID-based attacks. By enforcing the pattern at ingress, we guarantee every name is unambiguous. The 30-day alias preserves backward compatibility through renames without permanent name pollution.

## §3 — Measurement

- Counter `mcp_name_reject_total{reason=pattern|collision|length}`.
- CI gate metric `mcp_non_conforming_names_in_catalog` — must be 0.

## §4 — Verification

- Unit test (T) — fixtures of valid + invalid names; assert validator decisions.
- CI gate — full registry scan; non-conforming = fail.
- Property test (T) — random Unicode-rich strings; assert rejection.

## §5 — Failure handling

- Single rejection → expected; surfaced to author via clear error.
- Conformance regression in catalog → CI block + remediation PR.
- Naming-collision → registration refused; author renames.

---

*End of NFR-MCP-006.*
