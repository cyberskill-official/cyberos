# API versioning and deprecation policy (CyberOS 1.x payload)

**Task:** TASK-IMP-056  
**Scope:** HTTP/MCP surfaces the **install payload** exposes — not every
`services/*/v1` route in the monorepo platform (see [ADR-003](../adrs/ADR-003-payload-vs-platform-scope.md)).

## Surfaces in scope

| Surface | Today | Compatibility unit |
|---|---|---|
| Install MCP server (`tools/install/mcp/cyberos-mcp.mjs`) | MCP protocol `2025-06-18`; tools `task_install`, `task_gates`, `task_status`, `ship_task` | Tool name + inputSchema |
| Install CLI / `cs` verbs (`tools/install/cli`, `help.sh`) | Verb names + flags documented in GUIDE | Verb name + flag grammar |
| Docs-tools Node helpers (`tools/install/docs-tools/*.mjs`) | `--help` contracts used by ship-tasks | CLI flags + exit codes |
| Payload machine docs (ship-tasks, STATUS-REFERENCE) | Workflow versions in frontmatter | `workflow_version` / documented status enum |

Out of scope for this policy: Axum `/v1` service routes, public website APIs,
store submission endpoints. Those follow their own service ADRs when authored.

## Versioning rules

1. **Additive by default.** New MCP tools, CLI verbs, or optional flags are
   minor/compatible changes. Existing names keep prior semantics.
2. **Breaking change = major.** Removing a tool/verb, renaming it, or changing
   required fields / exit-code meaning requires a major payload version bump
   (`VERSION`) and a deprecation window (below).
3. **Document before delete.** Breaking changes land in GUIDE / MCP README in
   the same PR that introduces the replacement.
4. **No silent behavioural forks.** A flag must not invert meaning across
   minors; introduce a new flag instead.

## Deprecation window

| Class | Minimum notice | Mechanism |
|---|---|---|
| MCP tool / CLI verb | One minor release marked deprecated + one further minor before removal | Help text + README "Deprecated" section; tool still works |
| Docs-tool exit code | Same as above | Changelog + `--help` |
| Workflow status enum | ADR + STATUS-REFERENCE revision; never remove a status agents may still hold mid-flight | Dual-read if needed |

Emergency security breaks may skip the window with an ADR and a loud release
note; still bump major when behaviour is incompatible.

## Deprecation checklist (authoring)

- [ ] Name the replacement surface.
- [ ] Mark old surface deprecated in help/README with the removal target version.
- [ ] Keep old surface functional through the window.
- [ ] Add/extend a suite assertion so removal cannot happen silently.
- [ ] Bump `VERSION` appropriately when removal lands.

## Consumer guidance

- Pin to a payload `VERSION` in fleet installs; read
  `bash .cyberos/version.sh` / update-check before assuming new tools exist.
- MCP clients should tolerate unknown tools in `tools/list` (ignore), and must
  not invent required arguments not present in `inputSchema`.
