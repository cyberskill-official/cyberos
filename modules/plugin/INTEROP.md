# Plugin — runtime-target compatibility matrix (INTEROP)

Spec status: Normative. Companion to `README.md` (informative). Maximum length: 6,000 chars (per AGENTS.md §14.1 — the file a non-ledger consumer reads to interop).

A CyberOS plugin bundle MUST be installable in at least the four P1 runtimes below. Each runtime has its own packaging format; the `cyberos-plugin pack` adapter (FR-PLUGIN-007) emits the right artefact for each.

## Target runtimes (P1)

| Runtime | Install format | Manifest path | Slash commands | MCP server | Skills | Status |
|---|---|---|---|---|---|---|
| **Claude Code** | `.plugin` (signed zip) | `plugin.json` at root | `commands/*.md` | `mcp_servers` array in manifest | `skills/*` directory | P1 |
| **Cursor** | `.mcp.json` (single file) | inline | n/a (Cursor renders MCP tool list) | `cyberos` entry with stdio transport | n/a | P1 |
| **Anthropic Cowork** | Customize slot (uploaded zip) | `manifest.json` | `commands/*.md` | `connectors[*].mcp_url` | `skills/*` directory | P1 |
| **OpenAI Codex CLI** | SKILL.md compat folder | top-level `SKILL.md` | n/a | external MCP via env var | nested skills | P1 |

## Deferred runtimes (P2)

| Runtime | Notes |
|---|---|
| **Goose (Block)** | TOML manifest format; MCP-native; ship adapter once Block stabilises 2026 Q3 spec |
| **Amp (Sourcegraph)** | Markdown + YAML; ship adapter once Amp adopts MCP 2025-11-25 spec |
| **Continue.dev** | `.continue/config.json` slot; ship adapter alongside Cursor parity work |

## Universal constraints (apply to every adapter)

1. **Bundle MUST be reproducible.** Same source → same hash. Adapters MUST NOT inject timestamps, machine names, or local paths.
2. **Bundle MUST be signed.** Sigstore Rekor anchor referenced from `plugin.json#/signature/rekor_uuid`. Unsigned bundles MUST be rejected at install by the host (when host supports verification) and flagged by `cyberos-plugin doctor` regardless.
3. **MCP server MUST authenticate via OAuth-PKCE** to `auth.cyberskill.world/v1/oauth/authorize`. No long-lived secrets in the bundle.
4. **Tool calls MUST emit memory audit rows.** Every plugin invocation produces one `plugin.invoked` row at `memory.cyberskill.world/v1/audit` with `body.tool`, `body.tenant_id`, `body.actor_id`, `body.trace_id`. Failure to reach memory MUST queue locally with idempotency key and retry; MUST NOT drop the audit row silently.
5. **Capability advertisement MUST be honest.** A plugin that declares `capability.write_memory: true` MUST actually require the corresponding scope. Host MUST surface the declared capabilities at install time so the user grants knowingly.
6. **Versioning MUST be SemVer 2.0.** Major bumps require an in-product changelog page; minor/patch bumps are silent except in `cyberos-plugin doctor`.
7. **Naming MUST follow SEP-986** — `cyberos.{module}.{verb}_{noun}` for every tool, per FR-MCP-003. Adapters that flatten the namespace (Cursor, Codex CLI) MUST preserve the SEP-986 string verbatim in the manifest even if the rendered name is shorter.
8. **No host-runtime escapes.** A plugin MUST NOT shell out, write to host filesystem outside its sandbox, or open arbitrary network connections. All effects flow through MCP tools.

## Conformance test

```bash
# Run on a packaged bundle, regardless of target:
cyberos-plugin doctor dist/cyberos-1.0.0.plugin
# → 8/8 invariants PASS
#    ✓ reproducible
#    ✓ signed (rekor uuid: a1b2c3...)
#    ✓ oauth-pkce declared
#    ✓ memory audit emission declared
#    ✓ capabilities honest (4 declared, 4 required)
#    ✓ semver 2.0
#    ✓ SEP-986 naming (8/8 tools)
#    ✓ no host-runtime escapes
```

If any invariant fails → bundle MUST NOT be published. Adapters MUST refuse to emit a non-conforming bundle. Marketplace publish MUST reject on upload.

## Glossary

- **adapter** — a per-runtime emitter under `modules/plugin/adapters/<runtime>/` that transforms the canonical CyberOS manifest into the runtime's install format
- **bundle** — the final artefact a user installs (`.plugin`, `.mcp.json`, etc.)
- **canonical manifest** — `plugin.json` in this module's `manifests/` folder, validated against `manifest.schema.json`
- **tool** — an MCP-callable function exposed by the plugin (e.g. `cyberos.cuo.execute_workflow`)
- **scope** — an OAuth scope granted to the plugin at install (e.g. `cyberos:memory:read`)
- **capability** — a high-level declaration in the manifest (e.g. `write_memory`) that implies one or more scopes

## Authority

This file is normative for plugin bundle conformance. Changes to it require a chat-turn protocol amendment per AGENTS.md §16.2.

*End of modules/plugin/INTEROP.md*
