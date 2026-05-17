# MCP module — feature request index

_Generated 2026-05-17 — 8 FRs, 56 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-MCP-001](FR-MCP-001-spec-compliance.md) | MUST | 4 | 12 | MCP Gateway 2025-11-25 spec compliance — initialize + tools/list + tools/call + capabilities negotia |
| [FR-MCP-002](FR-MCP-002-server-heartbeat-lifecycle.md) | MUST | 2 | 6 | MCP per-module server registration + heartbeat lifecycle — 3-miss → unhealthy with automatic skill_u |
| [FR-MCP-003](FR-MCP-003-sep986-naming-validator.md) | MUST | 2 | 3 | MCP SEP-986 naming convention validator — `cyberos.{module}.{verb}_{noun}` pattern enforced at skill |
| [FR-MCP-004](FR-MCP-004-oauth-pkce.md) | MUST | 2 | 10 | OAuth 2.1 PKCE authorization-code flow with audience-bound tokens for MCP servers |
| [FR-MCP-005](FR-MCP-005-protected-resource-metadata.md) | MUST | 2 | 3 | MCP Protected Resource Metadata (RFC 9728) at `/.well-known/oauth-protected-resource` — closed audie |
| [FR-MCP-006](FR-MCP-006-tool-annotation-gating.md) | MUST | 2 | 6 | MCP tool-annotation gating — destructive / write / external-effect tools require explicit confirm or |
| [FR-MCP-007](FR-MCP-007-tasks-primitive.md) | MUST | 3 | 10 | MCP Tasks primitive — long-running tool calls with status polling + resume-on-reconnect + cancellati |
| [FR-MCP-008](FR-MCP-008-elicitation.md) | MUST | 3 | 6 | MCP Elicitation — server-initiated structured prompts for mid-call user input (clarifications, confi |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-MCP-001→FR-AUTH-004, FR-MCP-004→FR-AUTH-004

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._