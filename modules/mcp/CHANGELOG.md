# Changelog — MCP

## 2026-05-19 — P0 implementation wave — MCP Gateway slice-1 scaffold shipped (TASK-MCP-001)

See [AI changelog](../ai/changelog.html) for AI Gateway and [OBS changelog](../obs/changelog.html) for OBS collector portions of this wave.

### What landed

**`services/mcp-gateway/`** — Rust workspace member, slice-1 of P0.4 MCP Gateway:

- **TASK-MCP-001 — MCP 2025-11-25 spec compliance — scaffold shipped (10/10), status flipped `planned → building`.** JSON-RPC 2.0 parser; closed JSON-RPC error code map per DEC-272; `initialize` handshake returning `protocolVersion` + `Capabilities` + `ServerInfo`; `tools/list` with base64 cursor pagination; `tools/call` with permission gate; `ToolAnnotations` per DEC-264; `ToolRegistry` in-memory cache; Axum router mounting `POST /mcp` + `GET /mcp/healthz`. `MCP_PROTOCOL_VERSION` pinned at `"2025-11-25"` per DEC-260.
- **Remaining for `shipped` status:** JWT verification per TASK-AUTH-004 + audience-bound token check + per-(tenant, tool) rate-limit + memory audit row pair + Streamable HTTP SSE transport + OTel span emission.

---

## 2026-05-15 — MCP Gateway module page rewritten to Gold (external-client federation + capability broker + tool-discovery surface)

Rewrote `website/docs/modules/mcp.html` to Gold by encoding three strategic roles: (1) external-client federation (22 modules → one MCP server for Claude/Cursor/Codex/Cline; SEP-986 naming + module registration sequence + 6-row client compatibility matrix), (2) capability broker (6-row tool-annotation gating + audience-bound OAuth JWT example + destructive-op Elicitation flow), (3) tool-discovery surface (6 discovery endpoints + Tasks primitive 8-field schema + 5 pre-canned prompt templates).

Changes by section:
- **`<title>` + `<meta>`** — reframed: "MCP Gateway — External-client federation · Capability broker · Tool-discovery surface".
- **Hero tagline + lede** — "the external-agent door" framing: 22 modules behind one MCP surface; Claude/Cursor/Codex see one server; federation invisible to external clients.
- **Hero fact-grid** — extended from 8 to 12 cards: added Strategic role + External clients (Claude · Cursor · Codex · Cline) + Destructive-op gating (Human-confirm) + Persona stamp coverage (100%). Renamed naming convention card with concrete pattern.
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout; federation Mermaid (5 external clients × MCP Gateway × 6 per-module servers × 4 platform deps); 9-row auto-vs-human matrix.
- **TOC** — added bigger-picture · client-federation · capability-broker · tool-discovery entries.
- **NEW §2.5 "External-client federation"** — SEP-986 naming convention with 8 tool-name patterns + per-module registration sequence Mermaid (heartbeat-based lifecycle) + 6-row external-client compatibility matrix (Claude Code, Claude Desktop, Cursor, Codex, Cline, older 2024-11-05 clients).
- **NEW §2.6 "Capability broker"** — 6-row tool-annotation gating table (readOnly / idempotent / destructive / openWorld / longRunning / elicits); audience-bound OAuth JWT shape with aud=mcp.cyberos.com + scope_grants array; destructive-op confirmation flow with full Elicitation JSON request/response example.
- **NEW §2.7 "Tool-discovery surface"** — 6 discovery endpoints (well-known/mcp, capabilities, tools/list, prompts/list, resources/list, resources/templates/list); 8-field Tasks primitive schema with memory_chain anchor; 5 pre-canned prompt templates (weekly_brief, decision_to_issues, draft_cycle_review, deal_to_engagement, find_memory_citations).
- **§12 Risks** — added 10 new (R-MCP-011..020): external agent token theft (Critical) · prompt injection in tool description · elicitation fatigue (High likelihood) · federation lag · task storm · resource leak via list_changed · heartbeat false-positive · DCR abuse · older-protocol-version security gap · SEP-986 naming collision.
- **§13 KPIs** — added 10 new: persona-stamp coverage (hard floor = 1.0) · elicitation acceptance rate · tasks completion rate · cross-tenant token-replay attempts · older-protocol session rate (→ 0 by P3 · exit) · list_changed push latency · destructive-op confirm fatigue · external-client tools coverage · SEP-986 compliance.
- **§17 References** — replaced stale PRD/SRS refs with 4 in-page sections + 8 cross-module links + AUDIT_AND_PLAN §3.3 (P0 · slice 3 placement) + RESEARCH_REVIEW §5 (9/10) + MEMORY_AUTOSYNC_DESIGN.md §5+§6 + task-audit skill + DPoP RFC 9449 + EU AI Act + PDPL citations.

The MCP Gateway page now reads as the complete answer to: (1) why 22 modules need one external door (federation Mermaid + N²→N+1 math), (2) how the broker prevents a compromised external agent from escaping scope (audience-bound JWT + tool-annotation gating + destructive-op Elicitation), (3) how external agents discover what CyberOS can do (6 discovery endpoints + 5 pre-canned prompts + Tasks primitive for long-running work), (4) what fails if MCP Gateway is missing (every external agent re-implements its own auth + tool catalogue + audit).
