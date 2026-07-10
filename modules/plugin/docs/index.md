---
title: PLUGIN — Cross-runtime distribution · Claude Code / Cursor / Cowork / Codex CLI · CyberOS
source: website/docs/modules/plugin/index.html
migrated: FR-DOCS-002
---

Status

Specced

8 FRs at 10/10 · runtime planned

FRs

8

FR-PLUGIN-001..008

Hours

~58h

3 slices · 7-9 eng-days

Targets

4 P1

\+ 3 deferred to P2

Tools

8 MCP

CUO 4 · memory 2 · SKILL 2

Playbooks

12

Anthropic Skills compliant

## Why a separate module — not "just an MCP gateway feature"

The [MCP gateway](<../mcp/index.html>) owns the _protocol_. PLUGIN owns the _distribution + packaging surface_. Keeping them separate matches CyberOS's "agentic-native + open-standard + audit-chained + regionally-localized" positioning without overloading the gateway with marketing-surface mechanics.

Concern| MCP gateway owns| PLUGIN owns  
---|---|---  
Protocol shape| Spec 2025-11-25, initialize, tools/list, tools/call, Tasks, Elicitation| Inherits — does not redefine  
Transport| stdio + HTTP/Streamable| Picks the right transport per target runtime  
Spans multiple runtimes| One protocol| 4 target adapters (claude-code, cursor, cowork, codex-cli)  
Auth lifecycle| Per-call JWT verify| Install-time OAuth-PKCE handshake + refresh-token rotation  
Audit emission| Transport-level| 6 plugin.* audit kinds; durable Postgres outbox  
Distribution channel| None| plugins.cyberskill.world OCI + agentskills.io mirror  
User-visible version| Internal| Surfaces as `cyberos@1.4.2` in host Customize tabs  
  
## Three host-facing layers — tools, commands, playbooks

Plugins expose three distinct surfaces to a host. Each addresses a different consumer; together they make the plugin both _capable_ and _usable_.

Layer| What it answers| Consumer| FR  
---|---|---|---  
**Tools**|  What the plugin can do| Host's tool-calling subsystem| FR-PLUGIN-002  
**Commands**|  How the user invokes it explicitly| Host UI affordances (/cyberos-run, /cyberos-route, etc.)| FR-PLUGIN-003  
**Playbooks**|  When the model should pick which tool| Host's skill router (description-match fingerprint)| FR-PLUGIN-004  
  
## Architecture — pack-once, emit-many

A plugin is described once in canonical form, then a per-runtime adapter emits the target-native bundle. Same canonical manifest → 4 different installables.
    
    
    modules/plugin/manifests/cyberos@1.0.0.plugin.json   (CANONICAL — JSONSchema 2020-12)
                                │
                                ▼
                      cyberos-plugin pack
                                │
                  ┌─────────────┼─────────────┬─────────────────┐
                  ▼             ▼             ▼                 ▼
           claude-code      cursor       cowork            codex-cli
           (.plugin)     (.mcp.json)   (Customize zip)   (SKILL.md folder)
                  │             │             │                 │
                  └─────────────┴──────┬──────┴─────────────────┘
                                       │
                                Sigstore Rekor anchor (per-target)
                                       │
                                       ▼
                           OAuth-PKCE handshake at install
                              (auth.cyberskill.world)
                                       │
                                       ▼
                         Bridge invoked via stdio/HTTP transport
                                       │
                    ┌──────────────────┼──────────────────┐
                    ▼                  ▼                  ▼
              CUO supervisor    memory HTTP REST    SKILL broker
              (4 tools)         (2 tools)          (2 tools)
                                       │
                                       ▼
                      Every call → memory plugin.invoked row
                      (audit-then-respond; durable outbox; 24h retry)

## Target runtimes — 4 P1, 3 deferred to P2

Runtime| Bundle format| Commands?| Playbooks?| Bridge bundled?| Status  
---|---|---|---|---|---  
**Claude Code**| .plugin (signed zip)| ✓ /cyberos-*| ✓ Skills router| No — Claude Code manages| P1  
**Cursor**| .mcp.json (single file)| —| —| ✓ multi-arch| P1  
**Anthropic Cowork**|  Customize slot (zip)| ✓| ✓| ✓ HTTP transport| P1  
**OpenAI Codex CLI**|  SKILL.md folder| (as supplementary skills)| ✓| ✓ env-var binding| P1  
Goose (Block)| TOML manifest| —| —| —| P2 — successor FR  
Amp (Sourcegraph)| Markdown + YAML| —| —| —| P2 — successor FR  
Continue.dev| .continue/config.json| —| —| —| P2 — successor FR  
  
## Auth + audit — every action audit-chained, every token short-lived

### OAuth-PKCE (FR-PLUGIN-005)

  * ✓ Audience-bound JWT RS256 (`aud: "plugin:<id>"`) — blocks cross-plugin token reuse
  * ✓ 1-hour access tokens; 24-hour rotating opaque refresh
  * ✓ OS-keychain storage (macOS Keychain · Windows Credential Manager · Linux Secret Service)
  * ✓ 7 locked scopes: cyberos:{cuo,memory,skill}:{list,read,write,execute,route,invoke}
  * ✓ Revocation propagates within 60 seconds via cache TTL



### memory audit (FR-PLUGIN-006)

  * ✓ 6 audit kinds: plugin.installed / updated / uninstalled / invoked / auth_refreshed / scope_denied
  * ✓ Audit-then-respond ordering — no lost rows on crash
  * ✓ Durable Postgres outbox · 24h exponential backoff retry
  * ✓ SHA-256 idempotency key dedups retries server-side
  * ✓ Body strictly scrubbed — no tool input / output leak



## Marketplace — three visibility modes

Visibility| Discoverability| agentskills.io mirror| Revenue share| Strategy level  
---|---|---|---|---  
**public**|  any user, marketplace search| ✓ default-on| 70/30 author/CyberSkill on paid| Level 1 + 3  
**private**|  publishing tenant only| — blocked| n/a| Level 4 (vertical pack staging)  
**enterprise**|  per-enterprise origin (white-label)| — blocked| negotiated| Level 5  
  
**Vetted-by-CyberSkill badge** — JWT signed by CyberSkill marketplace key (`aud: "plugin:<id>:<version>"`), client-side verifiable. Persists per `(plugin_id, version)`. Manual review at first, automated where possible.

## FRs — 8 at 10/10

FR| Slice| Hours| Title  
---|---|---|---  
[FR-PLUGIN-001](<../../reference/fr-catalog.html#FR-PLUGIN-001>)| 1| 8h| Plugin manifest schema v1.0.0 + Python reference packer  
[FR-PLUGIN-002](<../../reference/fr-catalog.html#FR-PLUGIN-002>)| 1| 10h| MCP bridge Rust binary — stdio + HTTP · 8 tools · Tasks primitive  
[FR-PLUGIN-003](<../../reference/fr-catalog.html#FR-PLUGIN-003>)| 1| 4h| Canonical slash-commands (/cyberos-run, /cyberos-memory, /cyberos-skill-list, /cyberos-route)  
[FR-PLUGIN-004](<../../reference/fr-catalog.html#FR-PLUGIN-004>)| 2| 6h| 12 skill playbooks — when to chain plugin tools (SKB-020..023)  
[FR-PLUGIN-005](<../../reference/fr-catalog.html#FR-PLUGIN-005>)| 2| 8h| OAuth-PKCE auth — audience-bound JWT · OS-keychain · 60s revocation  
[FR-PLUGIN-006](<../../reference/fr-catalog.html#FR-PLUGIN-006>)| 2| 6h| memory audit emission — durable outbox · 24h retry · scrubbed body  
[FR-PLUGIN-007](<../../reference/fr-catalog.html#FR-PLUGIN-007>)| 3| 10h| Multi-runtime adapters — claude-code / cursor / cowork / codex-cli  
[FR-PLUGIN-008](<../../reference/fr-catalog.html#FR-PLUGIN-008>)| 3| 6h| Marketplace publish — OCI · agentskills.io mirror · 70/30 split · vetted JWT badge  
  
## Cross-module dependencies

This module depends on| FR| Status  
---|---|---  
MCP protocol spec compliance| FR-MCP-001| specced  
SEP-986 tool naming| FR-MCP-003| specced  
Tool annotation gating| FR-MCP-006| specced  
Tasks primitive| FR-MCP-007| specced  
JWT/JWKS issuance| FR-AUTH-004| shipped  
memory Layer-2 ingest| FR-MEMORY-101| shipped  
CUO supervisor v3.0.0-a4| (modules/cuo/)| shipped  
SKILL FR-SKILL-111..115 (trigger discipline)| FR-SKILL-111..115| shipped at spec-level; FR-115 sweep applied  
This module blocks| FR  
---|---  
Vertical pack pricing (TEN)| FR-TEN-005  
Branded Genie chat (PORTAL)| FR-PORTAL-005  
Strategy Level 1 OSS distribution| (strategy/CYBEROS_STRATEGY.md §4)  
Strategy Level 3 marketplace| (strategy/CYBEROS_STRATEGY.md §4)  
Strategy Level 4 vertical packs (cyberos-vn, etc.)| (strategy/CYBEROS_STRATEGY.md §4)  
Strategy Level 5 enterprise white-label| (strategy/CYBEROS_STRATEGY.md §4)  
  
## Strategy fit — the substrate for every distribution play

Without PLUGIN, CyberOS is only installable via git clone or direct download — viable for early adopters, fatal for ecosystem distribution. With PLUGIN, the same canonical CyberOS appears as a one-click install across every major agentic IDE.

Strategy level| What unlocks| This module ships  
---|---|---  
Level 1 — OSS distribution| Developers clone, install, use CyberOS without on-boarding from CyberSkill| Public visibility · agentskills.io mirror · Apache-2.0 default  
Level 3 — Marketplace| Third-party authors publish; 70/30 revenue; vetted badge; CyberSkill earns from ecosystem| FR-PLUGIN-008 publish CLI + OCI registry  
Level 4 — Vertical packs| `cyberos-vn`, `cyberos-sg`, `cyberos-eu`, `cyberos-us` each ship as their own plugin| Same manifest schema; per-region bundles  
Level 5 — Ecosystem-as-a-Service| Enterprise pays CyberSkill to run a private branded marketplace| Enterprise visibility mode · private origin pattern  
[← MCP Gateway (protocol layer below this module)](<../mcp/index.html>) [Module catalog →](<../index.html#catalog>)

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
