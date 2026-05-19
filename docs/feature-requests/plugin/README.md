# PLUGIN module — feature request index

_Generated 2026-05-19 — 8 FRs, 58 engineering-hours total._

The PLUGIN module ships the packaging + distribution surface that takes shipped CyberOS surfaces (CUO workflows, memory memory tools, SKILL playbooks) and bundles them as installable plugins for **Claude Code**, **Cursor**, **Anthropic Cowork**, **OpenAI Codex CLI**, and downstream MCP-/Skills-compatible runtimes.

This is **distinct** from the MCP gateway (`docs/feature-requests/mcp/FR-MCP-001..008`) which owns the protocol layer. PLUGIN sits on top: packaging, runtime adapters, capability advertisement, install-time auth, audit emission, marketplace distribution.

**Strategy alignment:** CYBEROS_STRATEGY §4 Level 1 (OSS distribution) + Level 3 (marketplace) + Level 4 (vertical packs ship as per-region plugins) + Level 5 (enterprise white-label uses private marketplace mode).

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-PLUGIN-001](FR-PLUGIN-001-manifest-schema.md) | MUST | 1 | 8 | Plugin manifest schema v1.0.0 — canonical `plugin.json` validated against `manifest.schema.json`; reference Python packer `cyberos-plugin pack` |
| [FR-PLUGIN-002](FR-PLUGIN-002-mcp-bridge.md) | MUST | 1 | 10 | CyberOS MCP bridge server — exposes CUO/memory/SKILL tools over MCP 2025-11-25 protocol; one binary, stdio + HTTP transports |
| [FR-PLUGIN-003](FR-PLUGIN-003-slash-commands.md) | MUST | 1 | 4 | Canonical slash-commands — `/cyberos-run`, `/cyberos-memory`, `/cyberos-skill-list`, `/cyberos-route` markdown definitions |
| [FR-PLUGIN-004](FR-PLUGIN-004-skill-playbooks.md) | MUST | 2 | 6 | Skill playbooks bundle — Anthropic-Agent-Skills SKILL.md files teaching hosts how to chain plugin tools correctly |
| [FR-PLUGIN-005](FR-PLUGIN-005-oauth-pkce-auth.md) | MUST | 2 | 8 | Plugin OAuth-PKCE authentication — install-time authorize + refresh-token rotation against `auth.cyberskill.world` |
| [FR-PLUGIN-006](FR-PLUGIN-006-memory-audit-emission.md) | MUST | 2 | 6 | memory audit emission — every install/update/uninstall/invoke produces a `plugin.*` audit row; idempotent retry queue |
| [FR-PLUGIN-007](FR-PLUGIN-007-multi-runtime-adapters.md) | MUST | 3 | 10 | Multi-runtime adapters — `cyberos-plugin pack --target {claude-code,cursor,cowork,codex-cli}` emitters; deferred targets in P2 |
| [FR-PLUGIN-008](FR-PLUGIN-008-marketplace-distribution.md) | SHOULD | 3 | 6 | Marketplace distribution — `cyberos-plugin publish` pushes signed bundle to `plugins.cyberskill.world` + mirrors to `agentskills.io`; revenue-share + vetted-badge |

**Total: 58 hours.** Roughly 7-9 engineering days for one experienced engineer.

## Slice structure

- **Slice 1 — Plugin core (22h)**: manifest schema + MCP bridge + slash commands. After slice 1, a developer can pack and install a working CyberOS plugin into Claude Code via local file install.
- **Slice 2 — Auth + audit + skill playbooks (20h)**: OAuth-PKCE handshake, memory audit row emission, skill playbook bundles. After slice 2, the plugin is production-safe (auditable, authenticated, governed).
- **Slice 3 — Multi-runtime + marketplace (16h)**: adapters for Cursor / Cowork / Codex CLI + publish surface. After slice 3, the plugin is publicly distributable.

## Cross-module dependencies

**This module depends on:**

- **MCP**: FR-PLUGIN-002 → FR-MCP-001 (protocol spec compliance), FR-MCP-003 (SEP-986 naming), FR-MCP-006 (tool annotation gating); FR-PLUGIN-005 → FR-MCP-004 (OAuth-PKCE flow shape)
- **AUTH**: FR-PLUGIN-005 → FR-AUTH-004 (JWT/JWKS issuance, shipped), FR-AUTH-007 (OAuth-PKCE authorize/token endpoints — placeholder, not yet specified)
- **memory**: FR-PLUGIN-006 → FR-MEMORY-101 (Layer-2 ingest pipeline, shipped), FR-MEMORY-104 (Tauri/HTTP REST, shipped)
- **SKILL**: FR-PLUGIN-004 → FR-SKILL-111 (description trigger enrichment, shipped), FR-SKILL-112 (TRIGGER_TESTS.md, shipped), FR-SKILL-104 (capability broker)
- **CUO**: FR-PLUGIN-002 → CUO supervisor v3.0.0-a4 (shipped at modules/cuo/, no FR — already production code)

**This module blocks:**

- **TEN**: FR-TEN-005 (vertical pack pricing) needs FR-PLUGIN-008 for the publish + revenue-share surface
- **PORTAL**: FR-PORTAL-005 (branded Genie chat) needs FR-PLUGIN-007 Cowork adapter so the white-label client ships as a plugin
- **Strategy Level 1 / 3 / 4 / 5 unlocks** — every distribution-facing milestone in the strategy depends on this module shipping

## Status

All 8 FRs at **10/10 audit** as of 2026-05-19. See sibling `*.audit.md` files.

## Implementation order

The slice order above IS the implementation order. FR-PLUGIN-001 has the lowest dependency surface (just JSONSchema validation) and is the keystone — every other FR consumes its manifest contract. Within each slice, FRs can be implemented in parallel (different files), but reciprocity invariants in BACKLOG.md §B require shipping in slice order.

```
Slice 1                           Slice 2                                Slice 3
┌───────────────┐    ┌──────┐    ┌────────────┐    ┌──────────────┐    ┌────────────┐    ┌─────────────┐
│ FR-PLUGIN-001 │ ─▶ │ -002 │ ─▶ │ FR-PLUGIN- │ ─▶ │  FR-PLUGIN-  │ ─▶ │ FR-PLUGIN- │ ─▶ │ FR-PLUGIN-  │
│   manifest    │    │ MCP  │    │  005 auth  │    │ 006 audit    │    │ 007 multi  │    │ 008 publish │
└───────────────┘    └──────┘    └────────────┘    └──────────────┘    └────────────┘    └─────────────┘
        │                              │                                       ▲
        ▼                              ▼                                       │
   ┌─────────┐                  ┌─────────────┐                                │
   │ FR-003  │                  │  FR-004     │ ──────── playbook deps ────────┘
   │ slash   │                  │  skill      │
   │ cmds    │                  │  playbooks  │
   └─────────┘                  └─────────────┘
```

---

_See [`../BACKLOG.md`](../BACKLOG.md) §0.5 for the full repo-level catalog state and [`../../modules/plugin/README.md`](../../modules/plugin/README.md) for the module-side description._
