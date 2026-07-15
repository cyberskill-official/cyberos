# MCP module — task index

_Generated 2026-05-17 — 8 tasks, 56 engineering-hours total._

## tasks

| Task | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-MCP-001](TASK-MCP-001-spec-compliance/spec.md) | MUST | 4 | 12 | MCP Gateway 2025-11-25 spec compliance — initialize + tools/list + tools/call + capabilities negotia |
| [TASK-MCP-002](TASK-MCP-002-server-heartbeat-lifecycle/spec.md) | MUST | 2 | 6 | MCP per-module server registration + heartbeat lifecycle — 3-miss → unhealthy with automatic skill_u |
| [TASK-MCP-003](TASK-MCP-003-sep986-naming-validator/spec.md) | MUST | 2 | 3 | MCP SEP-986 naming convention validator — `cyberos.{module}.{verb}_{noun}` pattern enforced at skill |
| [TASK-MCP-004](TASK-MCP-004-oauth-pkce/spec.md) | MUST | 2 | 10 | OAuth 2.1 PKCE authorization-code flow with audience-bound tokens for MCP servers |
| [TASK-MCP-005](TASK-MCP-005-protected-resource-metadata/spec.md) | MUST | 2 | 3 | MCP Protected Resource Metadata (RFC 9728) at `/.well-known/oauth-protected-resource` — closed audie |
| [TASK-MCP-006](TASK-MCP-006-tool-annotation-gating/spec.md) | MUST | 2 | 6 | MCP tool-annotation gating — destructive / write / external-effect tools require explicit confirm or |
| [TASK-MCP-007](TASK-MCP-007-tasks-primitive/spec.md) | MUST | 3 | 10 | MCP Tasks primitive — long-running tool calls with status polling + resume-on-reconnect + cancellati |
| [TASK-MCP-008](TASK-MCP-008-elicitation/spec.md) | MUST | 3 | 6 | MCP Elicitation — server-initiated structured prompts for mid-call user input (clarifications, confi |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-MCP-001→TASK-AUTH-004, TASK-MCP-004→TASK-AUTH-004

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._