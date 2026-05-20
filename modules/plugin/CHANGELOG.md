# Changelog — PLUGIN Module

All notable changes to the `plugin` module will be documented in this file.

---

## 2026-05-19 — [PLUGIN] New module — cross-runtime distribution scaffold + 8 FRs at 10/10

CyberOS's distribution + packaging layer. PLUGIN sits on top of the MCP gateway (`docs/feature-requests/mcp/`) and bundles CUO workflows + memory memory tools + SKILL playbooks as installable `.plugin` artefacts for **Claude Code**, **Cursor**, **Anthropic Cowork**, and **OpenAI Codex CLI**. Strategy alignment: §4 Level 1 (OSS distribution) + Level 3 (marketplace) + Level 4 (vertical packs ship as per-region plugins) + Level 5 (private marketplace for enterprise white-label).

### What landed

**Module scaffold** at `modules/plugin/`:
- `README.md` — module charter, scope, layout doctrine, status, cross-module deps
- `INTEROP.md` — 4 P1 runtime target matrix + 8 universal bundle invariants
- `manifest.schema.json` — JSONSchema 2020-12 for canonical `plugin.json` (load-bearing contract)
- `AGENTS.md` — symlink to `modules/memory/AGENTS.md` (Layer-1 memory protocol)
- `CHANGELOG.md` — pointer to this root file per 2026-05-18 centralisation
- Empty scaffold folders: `adapters/` (per-runtime emitters), `commands/` (slash-commands), `manifests/` (sample shipped bundles), `examples/` (developer demos)

**FR catalog** at `docs/feature-requests/plugin/` — 8 FRs at **10/10**, 58 engineering-hours total:
- `FR-PLUGIN-001` — Plugin manifest schema v1.0.0 + Python reference packer (`cyberos-plugin pack`). 8h. Keystone — every other FR consumes this manifest contract. Reproducible bundles + Sigstore Rekor anchor mandatory.
- `FR-PLUGIN-002` — CyberOS MCP bridge Rust binary at `services/plugin-host/`. Supports both stdio + HTTP transports. Exposes 8 tools (CUO 4 + memory 2 + SKILL 2) over MCP 2025-11-25. Tasks primitive for long-running execute_workflow. 4-class error taxonomy. 10h.
- `FR-PLUGIN-003` — 4 canonical slash-commands (`/cyberos-run`, `/cyberos-memory`, `/cyberos-skill-list`, `/cyberos-route`). 4h.
- `FR-PLUGIN-004` — 12 skill playbooks teaching hosts WHEN to chain plugin tools. SKB-020..023 conformance + TRIGGER_TESTS.md fixtures. 6h.
- `FR-PLUGIN-005` — OAuth-PKCE auth. Audience-bound RS256 JWTs (1h), rotating opaque refresh (24h), OS-keychain storage, locked scope catalogue. 8h.
- `FR-PLUGIN-006` — memory audit emission. 6 plugin.* audit kinds. Audit-then-respond ordering. Durable Postgres outbox with 24h exponential-backoff retry. 6h.
- `FR-PLUGIN-007` — Multi-runtime adapters for 4 P1 targets (claude-code / cursor / cowork / codex-cli). Single canonical manifest → 4 target-native bundles. Per-target reproducibility + Sigstore. P2 targets (goose, amp, continue-dev) deferred. 10h.
- `FR-PLUGIN-008` — Marketplace publish CLI. OCI push to plugins.cyberskill.world + mirror to agentskills.io for public plugins. 70/30 revenue share. Vetted-by-CyberSkill JWT badge. Yank-not-delete semantics. Full marketplace server deferred to FR-PLUGIN-008a. 6h.

### Catalog totals after this wave

- **Total FRs:** 253 → **261**
- **Modules with full spec coverage:** 24 → **25**
- **Engineering-hours:** ~1,998h → **~2,056h**

### Runtime build-out (next session)

Implementation phase starts at `services/plugin-host/` — Rust workspace member (new). Build order matches slice order in `docs/feature-requests/plugin/README.md`:
- Slice 1 — Plugin core (FR-001 + 002 + 003) — 22h
- Slice 2 — Auth + audit + skill playbooks (FR-004 + 005 + 006) — 20h
- Slice 3 — Multi-runtime + marketplace (FR-007 + 008) — 16h

### Files touched

- New: `modules/plugin/{README.md,INTEROP.md,manifest.schema.json,CHANGELOG.md}` (4 files)
- New: `modules/plugin/AGENTS.md` (symlink → `../memory/AGENTS.md`)
- New: `docs/feature-requests/plugin/{README.md,FR-PLUGIN-001..008.{md,audit.md}}` (17 files)
- Modified: `README.md` (repo-layout + Modules table + Roadmap)
- Modified: `docs/feature-requests/BACKLOG.md` (header v0.5.0 → v0.6.0; headline metrics; new wave note; new production-status row)

---
