# Changelog — AI

## 2026-05-19 — P0 implementation wave — AI Gateway slice-1 shipped (TASK-AI-003 + TASK-AI-005)

CyberOS's P0 build order is locked at AI Gateway → OBS → AUTH (stub) → MCP Gateway → CHAT. This entry covers the AI Gateway portion; see [OBS changelog](../obs/changelog.html) and [MCP changelog](../mcp/changelog.html) for their respective slices.

### What landed

**`services/ai-gateway/`** — Rust workspace member, slice-1 of P0.1 AI Gateway:

- **TASK-AI-003 — memory audit-row bridge (canonical Writer subprocess) shipped end-to-end (10/10).** Subprocess spawn of `python3 -m cyberos.writer put` with stdin/stdout/stderr piping; NFC normalisation; sorted-key JSON serialisation matching AGENTS.md §6.2; SHA-256 chain-hash recomputation + verification; path-traversal guard; 5s timeout with `kill_on_drop` + SIGTERM-then-SIGKILL; typed builders for slice-1 closed set; `check_writer_available` startup health check.
- **TASK-AI-005 — Tenant-policy YAML loader shipped end-to-end (10/10).** Closed `TenantPolicy` + `AiPolicy` + `Provider` + `Residency` + `EmergencyOverride` schema; `ArcSwap`-backed lock-free cache; `notify` file-watcher with eager startup load; invalid hot-reload preserves cache; path-traversal + charset validation; `validate_yaml` pure-function entry point.
- **`cyberos-ai` operator CLI** — slice-1 subcommands: `policy validate <file>` + `policy list` + `serve`.

### Catalog totals after this wave

- **tasks at `shipped + 10/10`:** prior 17 → **19** (+TASK-AI-003, +TASK-AI-005)
- **tasks flipped `planned → building`:** **+2** (TASK-OBS-001, TASK-MCP-001)

---

## 2026-05-15 — AI Gateway module page rewritten to Gold (P0 · slice 1 cost-of-everything gate + provider abstraction + compliance plane)

Rewrote `website/docs/modules/ai.html` to Gold by encoding three strategic roles: (1) cost-of-everything gate (per-tenant policy YAML + 7-step pre/post accounting + 7-dimension attribution), (2) provider-agnostic router (6-row model-alias table + 7-row failover semantics + residency × provider matrix), (3) compliance plane (4-link chain PII → persona → ZDR → audit + 14-field invocation row schema + VN-PII recogniser).

Changes by section:
- **`<title>` + `<meta>`** — reframed: "AI Gateway — Cost-of-everything gate · Provider-agnostic router · Compliance plane".
- **Hero tagline + lede** — explicit research review §2.4 citation: "ships at P0 · slice 1 BEFORE AUTH because if you can't account for and cap LLM spend, every other module bleeds money invisibly". Lists all 3 strategic roles.
- **Hero fact-grid** — extended from 8 to 12 cards: added Strategic role + Build placement (P0 · slice 1 P0 #1) + Cost-cap enforcement (hard-stop) + ZDR (required). Renamed dependency card to reflect P0 · slice 1 reality (memory + OBS at start; AUTH at P0 · slice 2).
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout with cross-module dependency Mermaid (6 callers × AI Gateway × 5 providers × 4 platform deps); 9-row auto-vs-human matrix covering failover, cost-cap override, ZDR refusal, cache hit, model alias resolution, image-gen.
- **TOC** — added bigger-picture · cost-gate · provider-abstraction · compliance-plane entries (4 new).
- **NEW §2.5 "Cost-of-everything gate"** — per-tenant policy YAML (caps, hard-stop, emergency override, per-model caps, per-persona attribution); 8-actor pre/post-call accounting sequence (Caller → Gateway → ledger → Provider → memory → INV); 7-dimension attribution table (tenant_id, agent_persona, module, cost_centre, route_class, cache_state, failover_path).
- **NEW §2.6 "Provider abstraction + failover"** — 6-row model-alias resolution (chat.smart / chat.fast / chat.reason / embed.standard / rerank.standard / image.standard); 7-row failover semantics (5xx retry / consecutive 5xx → mark degraded / 429 backoff / circuit breaker / recovery / both-down degraded mode / per-tenant SLA breach); residency × provider matrix (sg-1 / eu-1 / us-1 / vn-1).
- **NEW §2.7 "Compliance plane"** — 4-link chain table (PII → persona → ZDR → audit) with recall target + failure behaviour per link; full <code>ai.invocation</code> audit row schema (14 extra fields); VN-PII recogniser table (CCCD / MST / VN phone / NĐD / VN address / VN bank account) with patterns + redaction examples.
- **§12 Risks** — added 10 new (R-AI-011..020): P0 · slice 1 sequence slip → cost-overrun invisible (Critical) · persona prompt cache poisoning · provider DPA cancellation mid-quarter · cost-ledger hold leak · streaming SSE buffer leak · embedding model upgrade breaks memory search · image-gen budget flood at P2+ · geographic residency violation during failover (Critical) · VN-PII recogniser regression · BGE GPU pod OOM under load.
- **§13 KPIs** — added 9 new: per-persona cost share (alert on > 50% concentration) · cache savings rate (≥ 15% by P1) · hold-to-actual drift (≤ 5%) · residency-violation refusal rate · persona stamp coverage (hard floor = 1.0) · ZDR-compliant routing rate (hard floor = 1.0) · VN-PII recall on production sample (≥ 0.99) · provider-failover MTTR p95 (≤ 30s) · dogfooding LLM cost / Member (≤ $10/$5 trajectory).
- **§17 References** — replaced stale PRD/SRS refs with the 4 new in-page sections + MEMORY_AUTOSYNC_DESIGN.md §7 + task-audit skill + AUDIT_AND_PLAN §3.3 (P0 · slice 1 placement) + RESEARCH_REVIEW §2.4 (reorder citation) + 8 cross-module links + expanded EU AI Act citations (Art. 12/13/14/15/26/50) + OWASP Gen AI Top-10 + ISO/IEC 42001 + PDPL Art. 14/20/38.

The AI Gateway page now reads as the complete answer to: (1) why this module ships first in P0 (cost-control before everything), (2) how the cost ledger gates calls in real-time (pre-check + post-reconcile + 60s hold expiry), (3) how the same Python service abstracts across Bedrock/Anthropic/OpenAI/Vertex (model alias + residency × provider matrix), (4) how the 4-link compliance chain ensures no bytes leak unscrubbed/unstamped/un-ZDR'd/un-audited. A new engineer reading this page cold can pick up the P0 · slice 1 build sequence and ship the cost-gate slice.
