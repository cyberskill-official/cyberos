# Plugin module — distribute CyberOS to every agentic runtime

> **What this module is.** The packaging + distribution layer that takes shipped CyberOS surfaces (CUO workflows, memory memory tools, SKILL playbooks) and bundles them as installable `.plugin` artifacts for **Claude Code**, **Cursor**, **Anthropic Cowork**, **OpenAI Codex CLI**, **Goose**, **Amp**, **Continue.dev**, and any other MCP-/Skills-compatible runtime. Strategy alignment: this is the runtime substrate behind CYBEROS_STRATEGY §4 Level 1 (open-source distribution) + Level 3 (marketplace).
>
> **What this module is NOT.** It is not the MCP protocol gateway — that's `modules/.../mcp` (FR-MCP-001..008 cover spec compliance, OAuth-PKCE, tasks primitive, elicitation, tool annotation gating). PLUGIN sits on top of MCP and consumes the gateway as a transport; it adds packaging, runtime adapters, capability advertisement, audit emission, and marketplace distribution.

**Repo layout doctrine (per root README §43–§59):** `modules/plugin/` is the catalog + Python reference packer + spec. The Rust runtime adapters that ship the plugin to each target IDE live in `services/plugin-host/` once FR-PLUGIN-007 lands. The split mirrors `modules/memory/` ↔ `services/memory/` and `modules/skill/` ↔ `services/skill-broker/`.

---

## §1 — Module scope

A **plugin** is a CyberOS-authored bundle that travels as one signed unit and contains:

| Layer | Bundle contents | Source of truth |
|---|---|---|
| **Manifest** | `plugin.json` declaring name, version, capabilities, target runtimes, auth scopes | `manifest.schema.json` in this folder |
| **MCP server bridge** | A single MCP-protocol-compliant server (`cyberos-mcp-bridge`) that exposes CUO + memory + SKILL tools | `services/plugin-host/` (planned, FR-PLUGIN-002) |
| **Slash-commands** | Markdown command files (`commands/cyberos-run.md`, `commands/cyberos-memory.md`, etc.) | `modules/plugin/commands/` |
| **Skill playbooks** | Anthropic-Agent-Skills-compliant SKILL.md files that *teach* the host model how to use the MCP tools | `modules/skill/feature-request-author/` catalogue extended with `plugin-*` skills |
| **Runtime adapters** | Per-target adapter that emits the right install artefact: `.plugin` for Cowork, `.mcp.json` for Cursor, manifest for Codex CLI, etc. | `modules/plugin/adapters/<target>/` |
| **Signature + audit** | Sigstore Rekor anchor + memory audit row at publish + every invocation | `services/plugin-host/` + memory |

A user who installs `cyberos.plugin` into Claude Code gets `/cyberos-run`, `/cyberos-memory`, `/cyberos-skill-list`, plus an MCP server that any of the host's tool-calling subsystems can reach.

---

## §2 — Why a separate module (not "just an MCP gateway feature")

The MCP gateway (`docs/feature-requests/mcp/FR-MCP-001..008`) owns the *protocol*: spec compliance, OAuth-PKCE, tasks primitive, elicitation, tool annotation gating, naming convention, heartbeat lifecycle. It is a single-tenant in-process gateway that CyberOS modules register their MCP servers against.

PLUGIN owns the *packaging + distribution surface* that ships outwards. It:

1. **Spans multiple runtimes.** A single CyberOS plugin must work in Claude Code (manifest format A), Cursor (`.mcp.json` format B), Cowork (Customize slot format C), Codex CLI (SKILL.md format D), Goose (TOML format E). The adapter set is the work — MCP is just one of several transports.
2. **Has its own auth lifecycle.** Plugin install is a one-time OAuth-PKCE handshake to CyberOS AUTH; subsequent tool calls reuse a long-lived refresh token. MCP gateway sees per-call JWTs that PLUGIN issues.
3. **Has its own audit story.** Every plugin install, update, and tool call MUST emit a memory audit row (`plugin.installed`, `plugin.invoked`, `plugin.uninstalled`). The MCP gateway emits transport-level audit only.
4. **Has its own distribution channel.** Plugins publish to `plugins.cyberskill.world` (private CyberSkill marketplace) AND `agentskills.io` (public Anthropic registry) under different policies. The MCP gateway has no publish surface.
5. **Has its own versioning.** Plugin bumps are user-visible (`cyberos@1.4.2` shows up in Claude Code's Customize tab). Gateway protocol versions are internal.

Keeping these concerns separate matches CYBEROS_STRATEGY §2 — agentic-native + open-standard + audit-chained + regionally-localized — without overloading the gateway with marketing-surface mechanics.

---

## §3 — File layout

```
modules/plugin/
├── README.md                       ← THIS FILE
├── AGENTS.md                       ← symlink to modules/memory/AGENTS.md (Layer-1 memory protocol)
├── CHANGELOG.md                    ← pointer to centralised root CHANGELOG.md
├── INTEROP.md                      ← runtime-target compatibility matrix (≤ 6,000 chars)
├── manifest.schema.json            ← JSONSchema for plugin.json — load-bearing contract
├── adapters/
│   ├── claude-code/                ← .plugin bundler + manifest generator
│   ├── cursor/                     ← .mcp.json bundler
│   ├── cowork/                     ← Customize slot manifest
│   ├── codex-cli/                  ← SKILL.md compat exporter
│   └── goose/                      ← TOML emitter (deferred to P2)
├── commands/                       ← canonical slash-command markdowns
│   ├── cyberos-run.md
│   ├── cyberos-memory.md
│   ├── cyberos-skill-list.md
│   └── cyberos-route.md
├── manifests/                      ← shipped sample manifests
│   ├── cyberos@1.0.0.plugin.json
│   ├── cyberos-vn@1.0.0.plugin.json  ← Vietnamese vertical pack (Level 4 in strategy)
│   └── cyberos-enterprise@1.0.0.plugin.json
└── examples/                       ← runnable end-to-end demos
    ├── claude-code-install.md
    └── cursor-install.md
```

Rust production runtime (FR-PLUGIN-007) lives at `services/plugin-host/`. Python reference packer (FR-PLUGIN-001) lives at `modules/plugin/cyberos_plugin/`.

---

## §4 — Status

| Item | Status |
|---|---|
| Manifest schema (FR-PLUGIN-001) | **draft** — schema file in this folder; reference packer pending |
| MCP bridge (FR-PLUGIN-002) | **draft** — depends on MCP gateway (FR-MCP-001) + CUO supervisor v3.0.0a4 (shipped) + memory HTTP REST (shipped) |
| Slash commands (FR-PLUGIN-003) | **draft** — 4 commands specced |
| Skill playbooks (FR-PLUGIN-004) | **draft** — extends modules/skill/ catalog |
| OAuth-PKCE auth (FR-PLUGIN-005) | **draft** — depends on FR-AUTH-004 (JWT/JWKS, shipped) + FR-MCP-004 (OAuth-PKCE flow) |
| memory audit emission (FR-PLUGIN-006) | **draft** — depends on FR-MEMORY-101 (Layer-2 ingest, shipped) |
| Multi-runtime adapters (FR-PLUGIN-007) | **draft** — Claude Code + Cursor + Cowork + Codex CLI in P1; Goose + Amp + Continue.dev in P2 |
| Marketplace distribution (FR-PLUGIN-008) | **draft** — publish to `plugins.cyberskill.world` + `agentskills.io` |

All 8 FRs at `docs/feature-requests/plugin/`. See [`docs/feature-requests/plugin/README.md`](../../docs/feature-requests/plugin/README.md) for the index.

---

## §5 — Quick start (developer, when runtime ships)

```bash
# 5.1 — pack a plugin manifest
cd modules/plugin
python -m cyberos_plugin pack \
    --manifest manifests/cyberos@1.0.0.plugin.json \
    --target claude-code \
    --out dist/cyberos-1.0.0.plugin

# 5.2 — install locally into Claude Code
claude-code plugin install dist/cyberos-1.0.0.plugin
claude-code plugin list                          # → cyberos@1.0.0 (8 tools, 4 commands, 12 skills)

# 5.3 — verify auth handshake to cyberos.cyberskill.world
claude-code /cyberos-route "What did I commit yesterday?"   # → executes CUO workflow via plugin

# 5.4 — verify memory audit emission
curl -fsS https://memory.cyberskill.world/v1/audit/recent?kind=plugin.invoked | jq '.[0]'
# → { kind: "plugin.invoked", body: { plugin_id: "cyberos@1.0.0", tool: "cyberos.cuo.execute_workflow", ... } }
```

---

## §6 — Cross-module dependencies

| This module needs | From | Status |
|---|---|---|
| MCP protocol implementation | `modules/.../mcp` (FR-MCP-001..008) | specced, runtime not built |
| JWT/JWKS issuance for plugin tokens | `services/auth/` (FR-AUTH-004) | shipped |
| OAuth-PKCE authorize+token flow | `services/auth/` (FR-AUTH-007 — planned) + FR-MCP-004 | partially specced |
| CUO workflow execution surface | `modules/cuo/` v3.0.0-a4 | shipped |
| memory audit emission | `services/memory/` HTTP REST | shipped |
| SKILL catalog for playbook skills | `modules/skill/` 104 pairs | shipped |

| This module enables | For |
|---|---|
| Public OSS distribution (Strategy Level 1) | external developers cloning + using CyberOS |
| Marketplace (Strategy Level 3) | third-party publishers, paid skills, vetted-by-CyberSkill badge |
| Vertical packs (Strategy Level 4) | cyberos-vn, cyberos-sg, cyberos-eu, etc. — each shipped as its own plugin |
| Ecosystem-as-a-Service (Strategy Level 5) | white-label enterprise deployments |

---

## §7 — Authoring discipline

This module follows the same FR authoring discipline as the rest of CyberOS:

- Every FR loops audit rounds to **10/10** before the next FR starts. See [`modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md`](../skill/feature-request-audit/AUTHORING_DISCIPLINE.md).
- Every FR has a sibling `*.audit.md` with ≥ 6 ISS findings, all resolved, score_post_revision = 10/10.
- Every FR is self-contained — a reader does not need to open a dependency FR to understand THIS FR's contract.
- Cross-module dependencies in `depends_on`/`blocks` MUST be reciprocal.

---

## §8 — Open questions (resolved at FR time)

| # | Question | Answer location |
|---|---|---|
| 1 | What manifest format? | FR-PLUGIN-001 §3 — JSONSchema in `manifest.schema.json` |
| 2 | Which runtimes ship in P1? | FR-PLUGIN-007 §1 — Claude Code, Cursor, Cowork, Codex CLI |
| 3 | How are tool calls authenticated? | FR-PLUGIN-005 §3 — OAuth-PKCE issues plugin-scoped JWT, refreshed every 24h |
| 4 | What gets audited? | FR-PLUGIN-006 §1 — install / update / uninstall / every tool call |
| 5 | How does marketplace publish work? | FR-PLUGIN-008 §3 — `cyberos-plugin publish` pushes signed bundle to `plugins.cyberskill.world` + mirrors to `agentskills.io` |

---

## §9 — Related strategy references

- **CYBEROS_STRATEGY §2** — open-standard + audit-chained positioning; this module is how that positioning becomes user-installable
- **CYBEROS_STRATEGY §4 Level 1** — OSS distribution; this module is the substrate
- **CYBEROS_STRATEGY §4 Level 3** — marketplace; this module's FR-PLUGIN-008 is the publish surface
- **CYBEROS_STRATEGY §4 Level 4** — vertical packs; each pack ships as its own plugin under `cyberos-{region}` naming
- **CYBEROS_STRATEGY §4 Level 5** — ecosystem-as-a-service; private marketplace is FR-PLUGIN-008 §11 (enterprise mode)

---

*End of modules/plugin/README.md*
