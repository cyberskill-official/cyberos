# CyberOS — Feature Request Backlog

**Owner:** Stephen Cheng (CEO) · **Status:** v0.1.0 — initial index, 2026-05-15
**Source of truth:** the markdown files in this folder. This index is regenerated when FRs land or change status.
**Authoring playbook:** [`../FR_AUTHORING_WORKFLOW.md`](../FR_AUTHORING_WORKFLOW.md)
**Roadmap:** [`../../website/docs/architecture/milestones.html`](../../website/docs/architecture/milestones.html)

---

## §0 — How to read this backlog

This document is the **single source of truth** for what CyberOS is going to build, organised by **phase** (P0 → P4), then by **module**, then by **slice** within each module. Every row is one FR; one FR is one atomic, testable requirement.

- **Phase** maps to the milestone arc — `P0 Foundation` ships the cross-cutting infrastructure; `P1 Productivity` adds the internal-workflow modules; `P2 Operations` adds revenue + ops surfaces; `P3 SaaS-ready` adds the multi-tenant + employment-decision modules; `P4 Client-facing` adds external-customer surfaces.
- **Slice** is a coherent ship-unit within a module. Slice 1 is always the minimum viable surface for that module. Subsequent slices add depth, scale, compliance hardening, or persona surfaces.
- **Priority** uses BCP-14 keywords — `MUST` (release blocker) · `SHOULD` (release should-have) · `COULD` (release nice-to-have) · `MAY` (post-release).
- **Status** flows: `draft → audited → accepted → building → shipped` (or `deferred` / `rejected` / `superseded`).
- **Depends on** is the cross-FR dependency graph. An FR cannot start `building` until its `depends_on` rows are all `shipped`.
- **Effort** is a rough sizing in hours (1h = 30 min focused work + 30 min coordination/review). Treat as ±50%. Sized for one experienced engineer.

**Reading order for a planner:** scan §1 (totals) → pick the phase you're working in → read the per-module breakdown in that phase → drill into individual FR markdowns as you accept them.

**Reading order for an implementer:** find your assigned FR-ID in the per-module section → click through to the FR markdown → that file has the API contract, test harness, allowed-tools, implementation hints.

---

## §1 — Totals at a glance

| Phase | Modules in scope | FRs planned | Estimated effort (person-weeks) | Compliance gate at exit |
|---|---|---:|---:|---|
| **P0 — Foundation** | AI Gateway · OBS · AUTH (stub) · MCP Gateway · CHAT (dogfood) | ~37 | ~12 | SOC 2 readiness · CHAT decommission ≥ 0.95 |
| **P1 — Productivity** | BRAIN (auto-sync) · SKILL (packs) · PROJ · CRM · TIME · KB · EMAIL · HR · CUO (Phase 2 LLM) | ~58 | ~24 | EU AI Act Art. 12 ready · GDPR Art. 30 RoPA |
| **P2 — Operations** | INV · REW · ESOP · TEN (billing slice) · LEARN | ~32 | ~14 | PCI SAQ-A · Vietnamese hóa đơn Decree 123 compliant |
| **P3 — SaaS-ready** | TEN (full self-serve) · AUTH (full) · OKR · RES · HR (P3 extensions) | ~26 | ~12 | ISO 27001:2022 · PDPL Law 91/2025 full |
| **P4 — Client-facing** | DOC · PORTAL · TEN (external GA) · vertical-pack marketplace | ~22 | ~10 | eIDAS QTSP · AATL · Singapore HoldCo flip path |
| **Total** | 23 modules · 5 phases | **~175** | **~72 person-weeks** | 5 gated compliance milestones |

**Effort budget reality-check:** 175 FRs × 8h average = 1,400h ≈ 35 person-weeks of pure coding. The 72 person-weeks total accounts for design + review + integration + the inevitable surprise. That maps to roughly 18 months of one full-time engineer, or 9 months of two — which is consistent with the milestone arc on the docs site.

**Shipped state (excluded from the backlog count):** BRAIN Layer 1 (memory module — 6 ops, 15 invariants, 255 tests, MMR + Ed25519 STH); SKILL Phases 0–7 (open Agent Skills standard, Rust host, Bun toolchain); CUO Phase 1 (rule-based router, 6 core modules, 15 fixtures). These don't appear in this backlog because the work is done — their next-slice FRs (BRAIN auto-sync Stages 1–5; SKILL Phase 8 BRAIN integration; CUO Phase 2 LangGraph cascade) DO appear below.

---

## §2 — P0 · Foundation

**Phase goal:** stand up the cross-cutting infrastructure every other module depends on. By P0 exit (5 modules live), CyberSkill team members dogfood CHAT instead of Slack/Zalo, Genie answers route through CUO, every LLM call passes through the cost-of-everything gate, every action carries a BRAIN audit row, and the OBS plane gives one investigation surface for any incident.

**Compliance gate:** SOC 2 Type II readiness signal (RBAC + audit chain + retention policies). CHAT `decommission_signal ≥ 0.95` over 14-day rolling window at P0 exit (M+3 equivalent) — miss this and we hit the P0→P1 descope gate.

**Build order (locked):** AI Gateway → OBS → AUTH stub → MCP Gateway → CHAT.

### P0.1 — AI Gateway · the cost-of-everything gate

**Module page:** [`ai.html`](../../website/docs/modules/ai.html) · **Owner:** CTO · **Slice plan:** 5 slices, 22 FRs

#### Slice 1 — cost ledger + provider abstraction core

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-001** | AI Gateway cost-ledger pre-call check | MUST | accepted (10/10) | FR-AI-003, FR-AI-005 | 8h |
| **FR-AI-002** | AI Gateway cost-ledger post-call reconcile | MUST | accepted (10/10) | FR-AI-001, FR-AI-003 | 6h |
| **FR-AI-003** | BRAIN audit-row bridge (`ai.invocation` chained row per call) | MUST | accepted (10/10) | — | 5h |
| **FR-AI-004** | Cost-hold expiry cleanup job (Postgres scheduled) | MUST | accepted (10/10) | FR-AI-001, FR-AI-003 | 3h |
| **FR-AI-005** | Tenant-policy YAML loader (per-tenant cap + warn threshold + override) | MUST | accepted (10/10) | — | 5h |

#### Slice 2 — multi-provider router (Bedrock + Anthropic + OpenAI)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-006** | Model-alias resolution (`chat.smart` → `bedrock:claude-3.5-sonnet`) with per-tenant override | MUST | draft (10/10) | FR-AI-005 | 6h |
| **FR-AI-007** | Provider cost-table loader (YAML, hot-reload) | MUST | draft (10/10) | — | 4h |
| **FR-AI-008** | LiteLLM-derived router with retry + 30 s failover SLA | MUST | draft (10/10) | FR-AI-006, FR-AI-007 | 10h |
| **FR-AI-009** | Circuit-breaker per (provider, model) with half-open recovery probing | MUST | draft (10/10) | FR-AI-008 | 6h |
| **FR-AI-010** | Streaming SSE end-to-end (token-by-token to client) | SHOULD | draft (10/10) | FR-AI-008 | 8h |

#### Slice 3 — PII redaction + persona stamping

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-011** | Presidio EN-base PII redaction in-flight (every prompt) | MUST | draft (10/10) | FR-AI-008 | 6h |
| **FR-AI-012** | VN-PII Presidio plugin (CCCD · MST · VN phone · NĐD · VN address · bank account) | MUST | draft (10/10) | FR-AI-011 | 10h |
| **FR-AI-013** | VN-PII recall ≥ 99% CI gate on 200-sample test set | MUST | draft (10/10) | FR-AI-012 | 4h |
| **FR-AI-014** | Persona-version system-prompt injection from BRAIN `memories/personas/<version>.md` | MUST | draft (10/10) | FR-AI-003 | 5h |
| **FR-AI-015** | ZDR check — refuse non-ZDR provider when tenant policy requires it | MUST | draft (10/10) | FR-AI-006 | 3h |

#### Slice 4 — geographic residency + per-tenant cache

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-016** | Tenant residency pinning (`sg-1` / `eu-1` / `us-1` / `vn-1`) propagating to provider selection | MUST | draft (10/10) | FR-AI-006 | 5h |
| **FR-AI-017** | Cache (Redis) keyed by (tenant_id × prompt_hash × model); ≥ 30% hit rate P0 target | SHOULD | draft (10/10) | FR-AI-008 | 6h |
| **FR-AI-018** | Cross-tenant cache leak property-test (hard zero) | MUST | draft (10/10) | FR-AI-017 | 3h |
| **FR-AI-019** | Self-hosted BGE-M3 embeddings (single L4 GPU pod) + CPU fallback | SHOULD | draft (10/10) | — | 8h |
| **FR-AI-020** | BGE-rerank-v2-m3 cross-encoder for KB reranking | COULD | draft (10/10) | FR-AI-019 | 5h |

#### Slice 5 — operator surface + observability

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-021** | `cyberos-ai` operator CLI (usage · models list · policy set · failover drill · invoice export) | MUST | draft (10/10) | FR-AI-008, FR-AI-005 | 8h |
| **FR-AI-022** | OTel trace + span emission for every call (caller → router → provider → response) | MUST | draft (10/10) | FR-AI-008 | 4h |

---

### P0.2 — OBS · observability spine

**Module page:** [`obs.html`](../../website/docs/modules/obs.html) · **Owner:** CTO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — three pillars wired

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-OBS-001** | OTel collector deployed (Loki + Prometheus + Tempo backends) | MUST | planned | — | 6h |
| FR-OBS-002 | Grafana stood up with tenant-aware query proxy (Rust) injecting `tenant_id` label | MUST | planned | **FR-OBS-001** | 8h |
| FR-OBS-003 | Per-service RED metrics emitted (rate / errors / duration) via OTel SDK | MUST | planned | **FR-OBS-001** | 5h |

#### Slice 2 — AI traces + cross-pillar correlation

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-OBS-004 | LangSmith integration for AI traces (tied to operational trace_id) | MUST | planned | FR-AI-022 | 5h |
| FR-OBS-005 | Trace × log × metric × AI-trace correlation via W3C TraceContext propagation | MUST | planned | FR-OBS-003, FR-OBS-004 | 6h |
| FR-OBS-006 | Tail-based sampling (100% on errors, 10% normal) via OTel Collector | SHOULD | planned | **FR-OBS-001** | 4h |

#### Slice 3 — auto-runbook router + compliance surfaces

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-OBS-007 | Alert Manager → CUO `obs.triage-alert@1` skill routing (≥ 0.70 conf → CHAT; else PagerDuty) | MUST | planned | FR-OBS-003 | 8h |
| FR-OBS-008 | Compliance view scoping (EU AI Act / PDPL / SOC 2 / ISO 27001) over BRAIN audit chain | MUST | planned | FR-OBS-002 | 10h |
| FR-OBS-009 | Chain-of-custody manifest with Ed25519 signature on compliance exports | MUST | planned | FR-OBS-008 | 6h |

---

### P0.3 — AUTH (stub) · M+2-equivalent ship

**Module page:** [`auth.html`](../../website/docs/modules/auth.html) · **Owner:** CTO · **Slice plan:** 5 slices (P0 = slice 1; remainder defers to P3 full)

#### Slice 1 — stub (5 roles, password + WebAuthn, no MFA, no SSO)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-AUTH-001 | Tenant create (root-admin in tenant 0 calls `POST /v1/admin/tenants`) | MUST | planned | — | 6h |
| FR-AUTH-002 | Subject create (`POST /v1/admin/subjects`) with bcrypt hashed password | MUST | planned | FR-AUTH-001 | 5h |
| FR-AUTH-003 | RLS enforcement at every table (Postgres `current_setting('app.tenant')` predicate) | MUST | planned | FR-AUTH-001 | 8h |
| FR-AUTH-004 | JWT issuance + JWKS endpoint (RS256) with `tenant_id` + `agent_persona` + `scope_grants` claims | MUST | planned | FR-AUTH-002 | 6h |
| FR-AUTH-005 | Admin REST: list tenants + list subjects + revoke subject | MUST | planned | FR-AUTH-001, FR-AUTH-002 | 5h |
| FR-AUTH-006 | `cyberos-auth bootstrap` CLI for tenant-0 root-admin (no UI required) | MUST | planned | FR-AUTH-001 | 3h |

---

### P0.4 — MCP Gateway · external-agent door

**Module page:** [`mcp.html`](../../website/docs/modules/mcp.html) · **Owner:** CTO · **Slice plan:** 3 slices, 8 FRs

#### Slice 1 — federation core

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MCP-001 | MCP 2025-11-25 spec compliance — `tools/list`, `tools/call`, `capabilities` | MUST | planned | FR-AUTH-004 | 12h |
| FR-MCP-002 | Per-module server registration + heartbeat lifecycle (3-miss → unhealthy) | MUST | planned | FR-MCP-001 | 6h |
| FR-MCP-003 | SEP-986 naming convention validator (`cyberos.{module}.{verb}_{noun}`) | MUST | planned | FR-MCP-001 | 3h |

#### Slice 2 — OAuth 2.1 PKCE + audience binding

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MCP-004 | OAuth 2.1 PKCE authorization-code flow with audience-bound tokens | MUST | planned | FR-AUTH-004 | 10h |
| FR-MCP-005 | Protected Resource Metadata (PRM, RFC 9728) at `/.well-known/oauth-protected-resource` | MUST | planned | FR-MCP-004 | 3h |
| FR-MCP-006 | Tool-annotation gating (`destructive` requires explicit confirm or Elicitation flow) | MUST | planned | FR-MCP-001 | 6h |

#### Slice 3 — Tasks primitive + Elicitation

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MCP-007 | Tasks primitive (long-running work with status polling + resume on reconnect) | MUST | planned | FR-MCP-001 | 10h |
| FR-MCP-008 | Elicitation server-initiated request/response for mid-call user prompts | MUST | planned | FR-MCP-001 | 6h |

---

### P0.5 — CHAT · P0 dogfood gate

**Module page:** [`chat.html`](../../website/docs/modules/chat.html) · **Owner:** CPO · **Slice plan:** 4 slices, 12 FRs

#### Slice 1 — Mattermost fork + AUTH bridge

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-001 | Mattermost v9.x fork at pinned MIT/Apache commit + license-drift watcher | MUST | planned | — | 8h |
| FR-CHAT-002 | `cyberos-chat-authbridge` plugin — Mattermost auth delegates to AUTH JWT | MUST | planned | FR-CHAT-001, FR-AUTH-004 | 10h |
| FR-CHAT-003 | Per-tenant deployment via Fargate + RDS Multi-AZ + Redis | MUST | planned | FR-CHAT-001 | 6h |

#### Slice 2 — VN search + BRAIN bridge

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-004 | PGroonga + TinySegmenter VN bigram tokeniser; recall ≥ 80% CI gate | MUST | planned | FR-CHAT-003 | 12h |
| FR-CHAT-005 | BRAIN bridge — logical-replication from Postgres → BRAIN Layer-3 ingest, p95 ≤ 5 s | MUST | planned | FR-CHAT-003 | 10h |

#### Slice 3 — Slack/Zalo migration + @lumi capture

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-006 | Slack import (`cyberos-chat import slack` — 8-step idempotent + checkpointed) | MUST | planned | FR-CHAT-005 | 12h |
| FR-CHAT-007 | Zalo manual export importer (`cyberos-chat import zalo --bundle.zip`) | SHOULD | planned | FR-CHAT-005 | 8h |
| FR-CHAT-008 | `@lumi` mention parser → CUO route → BRAIN capture row | MUST | planned | FR-CHAT-005, FR-AI-014 | 6h |
| FR-CHAT-009 | Retro-capture flow — `@lumi remember the last N messages` with per-message opt-in | SHOULD | planned | FR-CHAT-008 | 6h |

#### Slice 4 — decommission instrumentation + DSAR

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-010 | `decommission_signal := (chat msgs) / (chat + slack + zalo msgs) ≥ 0.95` over 14-day rolling window | MUST | planned | FR-CHAT-006 | 5h |
| FR-CHAT-011 | Mobile push delivery (privacy-preserving payload: title + sender only) | MUST | planned | FR-CHAT-003 | 6h |
| FR-CHAT-012 | DSAR export — every message a subject authored + chained BRAIN audit hashes | MUST | planned | FR-CHAT-005 | 6h |

---

### P0 Exit gate criteria

- All 5 P0 modules ship to internal tenant `org:cyberskill`
- `decommission_signal ≥ 0.95` over a 14-day rolling window (FR-CHAT-010)
- All FRs in this section either `shipped` or explicitly `deferred` with an ADR
- Cross-leak property test = 0 violations (FR-AI-018 + FR-AUTH-003)
- Persona-stamp coverage = 100% on `ai.invocation` audit rows (FR-AI-014)
- OBS auto-runbook router covers ≥ 30% of alerts (FR-OBS-007)
- SOC 2 readiness check: RBAC + audit chain + retention policies verifiable via Grafana compliance view (FR-OBS-008)

**Miss decommission_signal → P0→P1 descope gate fires:** 2-week sprint freeze on net-new modules, focused only on CHAT polish + migration tooling. If still &lt; 0.85 after 2 weeks, escalate to platform-thesis review.

---

## §3 — P1 · Productivity

**Phase goal:** make CyberOS the platform CyberSkill team members actively prefer to use over their previous tools. PROJ replaces Linear; KB replaces Notion; CRM replaces HubSpot; EMAIL replaces Gmail's shared-inbox add-ons; TIME replaces Toggl; HR replaces a spreadsheet. By P1 exit, the agency runs daily ops fully on CyberOS.

**Compliance gate:** EU AI Act Art. 12 logging ready · GDPR Art. 30 RoPA evidentiary surface · PDPL Art. 14 DSAR end-to-end across all P1 modules.

**Critical dependencies:** all P1 modules require P0 complete (AUTH stub, BRAIN audit, AI Gateway cost cap, OBS, MCP federation).

### P1.1 — BRAIN auto-sync · Stages 1–2 (Personal BRAIN + Capture daemon)

**Module page:** [`brain.html`](../../website/docs/modules/brain.html) · **Owner:** CDO · **Slice plan:** Stages 1–5 (this phase covers 1–2; Stage 3+ defers to P2)

#### Stage 1 — Personal BRAIN universal protocol (any folder, portable)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-BRAIN-101 | `cyberos brain init <path>` — any-folder bootstrap (manifest, audit/, memories/, HEAD) | MUST | planned | — | 6h |
| FR-BRAIN-102 | `cyberos brain watch / unwatch / status` — multi-folder watcher registry | MUST | planned | FR-BRAIN-101 | 6h |
| FR-BRAIN-103 | Privacy-floor flag in manifest (`sync_class_default: private`) + per-memory override | MUST | planned | FR-BRAIN-101 | 4h |
| FR-BRAIN-104 | Portable folder-copy round-trip (export to zip + import on another machine) | MUST | planned | FR-BRAIN-101 | 6h |
| FR-BRAIN-105 | Doctor invariants extended (+2 new for watched-folders integrity) | MUST | planned | FR-BRAIN-101 | 4h |
| FR-BRAIN-106 | `watched_folders` schema migration + idempotent re-watch on restart | MUST | planned | FR-BRAIN-102 | 4h |

#### Stage 2 — Capture daemon (FS watcher + Cowork hook + Claude Code hook)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-BRAIN-107 | FS watcher via Rust + `notify` crate; rate-limited + content-deduped | MUST | planned | FR-BRAIN-102 | 12h |
| FR-BRAIN-108 | Cowork session-hook capture (Claude Code Cowork mode emits memories) | MUST | planned | FR-BRAIN-107 | 6h |
| FR-BRAIN-109 | Claude Code hook capture (CLI / IDE plugin emits per-prompt memories) | MUST | planned | FR-BRAIN-107 | 5h |
| FR-BRAIN-110 | Capture daemon health check + restart on crash (systemd / launchd unit) | MUST | planned | FR-BRAIN-107 | 4h |
| FR-BRAIN-111 | Pre-ingest PII detection (Presidio EN + VN); held-back rate ≥ 99.5% | MUST | planned | FR-AI-012 | 6h |

---

### P1.2 — SKILL · Phase 8 (BRAIN integration) + vertical packs

**Module page:** [`skill.html`](../../website/docs/modules/skill.html) · **Owner:** CPO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — capability broker + BRAIN-aware SKILL.md

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-101 | SKILL.md frontmatter extension: `allowed_brain_scopes` + `allowed_tools` enforced by broker | MUST | planned | FR-BRAIN-101 | 5h |
| FR-SKILL-102 | Capability broker: subprocess sandbox enforces `allowed_tools` at invoke time | MUST | planned | FR-MCP-006 | 8h |
| FR-SKILL-103 | Pre + post audit rows on every skill invocation (BRAIN Writer dual-write) | MUST | planned | FR-SKILL-102 | 5h |

#### Slice 2 — universal-protocol skill bundles

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-104 | `brain-capture@1` skill bundle (canonical capture entry point) | MUST | planned | FR-BRAIN-107 | 6h |
| FR-SKILL-105 | `brain-sync@1` skill bundle (defers to Stage 4 sync orchestrator at P2) | SHOULD | planned | FR-SKILL-104 | 4h |
| FR-SKILL-106 | `synthesis-author@1` skill (multi-brain auto-evolve; runs nightly at P3) | COULD | planned | FR-SKILL-105 | 8h |

#### Slice 3 — vertical pack scaffolding (cyberskill-vn first)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-107 | `cyberskill-vn` pack — `vn-mst-validate` skill against GDT API | MUST | planned | FR-SKILL-102 | 6h |
| FR-SKILL-108 | `cyberskill-vn` pack — `vn-bank-transfer` skill (VietQR / Napas247 code generator) | MUST | planned | FR-SKILL-102 | 6h |
| FR-SKILL-109 | `cyberskill-vn` pack — `vn-vat-invoice` skill (hóa đơn Decree 123 XML emitter) | MUST | planned | FR-SKILL-102 | 10h |

---

### P1.3 — CUO · Phase 2 LLM cascade + Phase 3 multi-skill chains

**Module page:** [`cuo.html`](../../website/docs/modules/cuo.html) · **Owner:** CPO · **Slice plan:** 2 slices (P1) + Phases 3–4 deferred to P2/P3

#### Phase 2 — LangGraph + LiteLLM cascade

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CUO-101 | LangGraph supervisor wired to LiteLLM router (escalate when 0.10 ≤ conf ≤ 0.50) | MUST | planned | FR-AI-008 | 12h |
| FR-CUO-102 | Postgres checkpointer for LangGraph state (EU AI Act Art. 12 logging) | MUST | planned | FR-CUO-101 | 5h |
| FR-CUO-103 | Phase 2 trace rows include prompt + model + temperature + seed for replay | MUST | planned | FR-CUO-102 | 4h |

#### Phase 3 — multi-skill chains via `depends_on`

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CUO-104 | Topological walk of `depends_on` chain with composite audit row + sub-rows | MUST | planned | FR-CUO-101 | 10h |
| FR-CUO-105 | Per-step rollback on chain failure; partial-execution audit preserved | MUST | planned | FR-CUO-104 | 6h |

---

### P1.4 — PROJ · orchestration spine

**Module page:** [`proj.html`](../../website/docs/modules/proj.html) · **Owner:** COO/CPO · **Slice plan:** 5 slices, 18 FRs

#### Slice 1 — four primitives + sync engine

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-001 | Issue / Cycle / Project / Engagement Postgres schema with `engagement_id` FK | MUST | planned | FR-AUTH-003 | 8h |
| FR-PROJ-002 | WebSocket sync engine (axum + NATS JetStream) with optimistic client apply + server rebase | MUST | planned | FR-PROJ-001 | 16h |
| FR-PROJ-003 | Yjs CRDT for description + comment-body fields; LWW for scalars | MUST | planned | FR-PROJ-002 | 10h |
| FR-PROJ-004 | Issue lifecycle state machine (backlog → todo → in-progress → in-review → done / cancelled) | MUST | planned | FR-PROJ-001 | 5h |

#### Slice 2 — Engagement economics + billable rules

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-005 | Rate-card schema per Engagement (role × currency × hourly rate × billable_default) | MUST | planned | FR-PROJ-001 | 4h |
| FR-PROJ-006 | Billable cascade: Member override → task class → role default → fallback | MUST | planned | FR-PROJ-005 | 6h |
| FR-PROJ-007 | Three billing modes (T&M / fixed-fee / retainer) with mode-aware rollup | MUST | planned | FR-PROJ-005 | 6h |

#### Slice 3 — BRAIN integration + cross-module join contracts

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-008 | BRAIN audit row per Issue mutation (chained to PROJ `history_event` table) | MUST | planned | FR-BRAIN-101 | 5h |
| FR-PROJ-009 | `BRAIN_LINK` schema: Issue cites memory via (cites / implements / supersedes) | MUST | planned | FR-PROJ-001 | 5h |
| FR-PROJ-010 | Citation-drift detector (nightly sweep flags stale citations) | SHOULD | planned | FR-PROJ-009 | 4h |

#### Slice 4 — AI features (blocker detection + cycle review + calibration)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-011 | Blocker detector from comment stream (`blocked by` + dwell time → CUO Notify) | MUST | planned | FR-CUO-101 | 6h |
| FR-PROJ-012 | Cycle-review draft generator (CUO/COO persona) at cycle close | MUST | planned | FR-CUO-101 | 8h |
| FR-PROJ-013 | Estimate calibration snapshot (per Member per task class, nightly batch) | MUST | planned | FR-PROJ-002 | 6h |

#### Slice 5 — UI surfaces (Board · Timeline · Gantt · Brief)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-014 | Kanban Board (drag + drop, keyboard-first) | MUST | planned | FR-PROJ-002 | 12h |
| FR-PROJ-015 | Timeline view (cycle window × assignee) | MUST | planned | FR-PROJ-002 | 10h |
| FR-PROJ-016 | Gantt view with dependency arrows | SHOULD | planned | FR-PROJ-002 | 12h |
| FR-PROJ-017 | Brief modal (issue deep-view with Yjs description + comments + meta sidebar) | MUST | planned | FR-PROJ-003 | 8h |
| FR-PROJ-018 | Liquid-Glass design tokens (`tokens.proj.css`) + axe-core CI accessibility gate | MUST | planned | FR-PROJ-014 | 6h |

---

### P1.5 — CRM · sales-pipeline spine

**Module page:** [`crm.html`](../../website/docs/modules/crm.html) · **Owner:** CRO/CPO · **Slice plan:** 3 slices, 10 FRs

#### Slice 1 — three primitives + pipelines

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CRM-001 | Account / Contact / Deal Postgres schema with custom pipelines + stages | MUST | planned | FR-AUTH-003 | 6h |
| FR-CRM-002 | Activity feed auto-log from EMAIL/CHAT/Calendar via tracked-domain match | MUST | planned | FR-CRM-001 | 8h |
| FR-CRM-003 | VN-specific: account type → legal entity (Sole / LLC / JSC / FDI) + MST field | MUST | planned | FR-CRM-001 | 4h |

#### Slice 2 — Deal → Engagement bridge + AI features

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CRM-004 | Convert-to-Engagement workflow (deal.won → PROJ Engagement with rate card) | MUST | planned | FR-CRM-001, FR-PROJ-005 | 6h |
| FR-CRM-005 | CUO `crm.next-action@1` skill — top-3 ranked moves per open deal | MUST | planned | FR-CUO-101 | 6h |
| FR-CRM-006 | AI lead scoring at Contact creation + nightly refresh | SHOULD | planned | FR-CUO-101 | 5h |
| FR-CRM-007 | Win/loss analysis CUO draft at deal close; becomes BRAIN memory | SHOULD | planned | FR-CUO-101 | 5h |

#### Slice 3 — VN integrations

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CRM-008 | MST validation via `vn-mst-validate` skill on Account write | MUST | planned | FR-SKILL-107 | 3h |
| FR-CRM-009 | VietQR generation via `vn-bank-transfer` skill on Deal collection | MUST | planned | FR-SKILL-108 | 4h |
| FR-CRM-010 | Hóa đơn auto-emit via `vn-vat-invoice` on deal.stage=won | MUST | planned | FR-SKILL-109 | 5h |

---

### P1.6 — TIME · billable-hours engine

**Module page:** [`time.html`](../../website/docs/modules/time.html) · **Owner:** CFO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — TimeEntry primitive + 3 input modes

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TIME-001 | TimeEntry append-only schema with `correction_to` link semantics | MUST | planned | FR-AUTH-003 | 5h |
| FR-TIME-002 | Timer start/stop UI in SPA | MUST | planned | FR-TIME-001 | 5h |
| FR-TIME-003 | Manual entry form (retroactive logging) with VN Labour Code cap validation | MUST | planned | FR-TIME-001 | 6h |
| FR-TIME-004 | Auto-detect proposals from PROJ activity (status changes + comment patterns; Member-confirm) | SHOULD | planned | FR-PROJ-002 | 6h |

#### Slice 2 — billable cascade + approval flow

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TIME-005 | Billable flag computation via 4-step cascade (snapshot on row) | MUST | planned | FR-TIME-001, FR-PROJ-006 | 5h |
| FR-TIME-006 | Weekly approval flow (Member submit → AM review → CFO visibility) | MUST | planned | FR-TIME-001 | 6h |
| FR-TIME-007 | VN Labour Code Art. 107 OT cap hard-block at entry write | MUST | planned | FR-TIME-001 | 4h |

#### Slice 3 — receipt OCR + PROJ-INV bridge

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TIME-008 | Expense capture: photo → AWS Textract OCR → hóa đơn parse → Member confirm | MUST | planned | FR-CRM-010 | 8h |
| FR-TIME-009 | Per-cycle billable rollup emit to INV (per-Member × role × Engagement) | MUST | planned | FR-TIME-005 | 6h |

---

### P1.7 — KB · RAG corpus + BRAIN companion

**Module page:** [`kb.html`](../../website/docs/modules/kb.html) · **Owner:** CDO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — markdown source + versioning + render

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-KB-001 | Document schema (slug + markdown body + YAML frontmatter + category + ACL) + immutable versions | MUST | planned | FR-AUTH-003 | 6h |
| FR-KB-002 | Server-side renderer: markdown → sanitised HTML (ammonia) + sanitised plaintext for BRAIN | MUST | planned | FR-KB-001 | 5h |
| FR-KB-003 | Three permission tiers: public · org-only · role-restricted with share-link tokens | MUST | planned | FR-KB-001 | 5h |

#### Slice 2 — three-layer search + AI Q&A

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-KB-004 | FTS5 + PGroonga lexical search with VN bigram tokenisation | MUST | planned | FR-KB-001 | 6h |
| FR-KB-005 | BGE-M3 semantic search via BRAIN Layer 2 ingest | MUST | planned | FR-AI-019, FR-KB-001 | 6h |
| FR-KB-006 | BGE-rerank-v2-m3 cross-encoder reranker over top-K from layers 1+2 | MUST | planned | FR-AI-020, FR-KB-005 | 4h |
| FR-KB-007 | "Ask this page" Q&A grounded in current + linked docs with span-level citations | MUST | planned | FR-KB-006, FR-CUO-101 | 8h |

#### Slice 3 — runbook catalogue + dual-language

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-KB-008 | Runbook category with applicability tags (provider / region / severity) for OBS triage | MUST | planned | FR-KB-001, FR-OBS-007 | 5h |
| FR-KB-009 | Dual-language `translation_of` link + locale-aware reader display (vi/en) | SHOULD | planned | FR-KB-001 | 4h |

---

### P1.8 — EMAIL · capture surface + Genie draft

**Module page:** [`email.html`](../../website/docs/modules/email.html) · **Owner:** CCO/CPO · **Slice plan:** 3 slices, 11 FRs

#### Slice 1 — Stalwart core + shared inbox

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-EMAIL-001 | Stalwart Rust mail server deployed (JMAP/IMAP/SMTP/ManageSieve all transports) | MUST | planned | — | 12h |
| FR-EMAIL-002 | `cyberos-email-authbridge` plugin — Stalwart JMAP auth delegates to AUTH JWT | MUST | planned | FR-EMAIL-001, FR-AUTH-004 | 6h |
| FR-EMAIL-003 | Missive-style shared-inbox UX (assignment, internal comments, snooze, tag) | MUST | planned | FR-EMAIL-001 | 16h |
| FR-EMAIL-004 | DKIM signing + ARC chain forward + BIMI brand indicator | MUST | planned | FR-EMAIL-001 | 6h |

#### Slice 2 — CaMeL quarantine + capture

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-EMAIL-005 | CaMeL dual-LLM: quarantine LLM (no tools, no memory) extracts → privileged CUO consumes only sanitised | MUST | planned | FR-CUO-101 | 12h |
| FR-EMAIL-006 | Tracked-domain auto-log to CRM activity feed (per-tenant tracked-domain config) | MUST | planned | FR-CRM-002, FR-EMAIL-001 | 5h |
| FR-EMAIL-007 | "Convert to issue" — thread → PROJ Issue with body as description, replies as comments | MUST | planned | FR-PROJ-001, FR-EMAIL-001 | 6h |

#### Slice 3 — Genie draft + bulk send

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-EMAIL-008 | "Genie:" subject prefix → CUO draft grounded in thread + CRM + BRAIN + KB (sync_class permitting) | MUST | planned | FR-CUO-101, FR-KB-007 | 8h |
| FR-EMAIL-009 | Outbound 1:1 send (DKIM-signed, AM confirms) | MUST | planned | FR-EMAIL-004 | 4h |
| FR-EMAIL-010 | Bulk send (≥ 10 recipients) requires AM + CFO/marketing approval token | MUST | planned | FR-EMAIL-009 | 5h |
| FR-EMAIL-011 | DSAR export — every message a subject authored + chained BRAIN audit hashes | MUST | planned | FR-EMAIL-001 | 5h |

---

### P1.9 — HR · Member lifecycle + onboarding orchestrator

**Module page:** [`hr.html`](../../website/docs/modules/hr.html) · **Owner:** CHRO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — Member directory + contract types

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-HR-001 | Member schema (profile + role + level + contract type + leave balance + sabbatical accrual) | MUST | planned | FR-AUTH-003 | 6h |
| FR-HR-002 | 5 contract types: indefinite · fixed-term · probation · part-time · contractor | MUST | planned | FR-HR-001 | 4h |
| FR-HR-003 | CCCD photo separate KMS keyspace + sev-1 access audit | MUST | planned | FR-HR-001 | 5h |

#### Slice 2 — leave + statutory caps

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-HR-004 | 8 leave types (annual / sick / maternity / paternity / sabbatical / unpaid / bereavement / public-holiday) | MUST | planned | FR-HR-001 | 5h |
| FR-HR-005 | Decree 145/2020 working-hour caps + Decree 152/2020 SI rates (version-pinned) | MUST | planned | FR-HR-001 | 4h |
| FR-HR-006 | Annual-leave accrual nightly batch (Decree 145 formula) | MUST | planned | FR-HR-004 | 4h |

#### Slice 3 — onboarding orchestrator + performance signals

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-HR-007 | Onboarding saga — fires to AUTH / TIME / LEARN / KB / CHAT / REW on `member.active` transition | MUST | planned | FR-HR-001 | 10h |
| FR-HR-008 | Performance signal aggregator (read-only consumer of PROJ + TIME + LEARN signals) | MUST | planned | FR-PROJ-013, FR-TIME-001 | 6h |
| FR-HR-009 | Termination workflow with GL/BL branch (CFO + CEO co-sign required) | MUST | planned | FR-HR-001 | 8h |

---

### P1 Exit gate criteria

- All 9 P1 modules ship internally and are used daily
- DSAR fulfilment p95 ≤ 24 h across all P1 modules (PDPL Art. 14)
- EU AI Act Art. 12 audit trail covers every CUO Phase 2 + skill invocation
- 16 P1 modules total live (P0's 5 + P1's 9 + 2 already-shipped: SKILL Phase 8, CUO Phase 2)
- BRAIN auto-sync Stages 1 + 2 demonstrably running on every Member's laptop
- Cycle-review draft acceptance rate ≥ 60% across all Engagements (FR-PROJ-012)

---

## §4 — P2 · Operations

**Phase goal:** revenue + financial ops solidified. P2 ships INV, REW, ESOP, the TEN billing thin slice, and LEARN. Invoices issue from PROJ-TIME rollups. Payroll runs through REW deterministically. Vertical-pack pricing becomes possible. LEARN's promotion workflow (Hội đồng Chuyên môn) anchors the career path.

**Compliance gate:** Vietnamese hóa đơn Decree 123 emission validated end-to-end · PCI SAQ-A self-assessment passed · ISO 27017 cloud-services controls signed off.

### P2.1 — INV · billable rollup invoicing

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-INV-001 | Invoice draft from TIME per-cycle rollup with rate-card snapshot preservation | MUST | planned | FR-TIME-009 | 8h |
| FR-INV-002 | Multi-currency support (VND / USD / SGD / EUR) with daily SBV FX snapshot | MUST | planned | FR-INV-001 | 6h |
| FR-INV-003 | Stripe webhook handler (signature-verified, idempotent) | MUST | planned | — | 8h |
| FR-INV-004 | Wise webhook handler for multi-currency receipts | SHOULD | planned | — | 6h |
| FR-INV-005 | VietQR/Napas247 webhook handler for VND domestic | MUST | planned | — | 6h |
| FR-INV-006 | Cash application — match incoming receipts to outstanding invoices (amount + reference) | MUST | planned | FR-INV-003, FR-INV-005 | 8h |
| FR-INV-007 | `vn-vat-invoice` hóa đơn auto-emit on AM-send for VN tenants (Decree 123 GDT XML) | MUST | planned | FR-INV-001, FR-SKILL-109 | 6h |
| FR-INV-008 | Hóa đơn cancellation (Decree 123 Art. 19) with dual approval (AM + CFO) | MUST | planned | FR-INV-007 | 5h |
| FR-INV-009 | AR aging report (nightly) + 90+ rolling alert | MUST | planned | FR-INV-001 | 4h |
| FR-INV-010 | CUO dunning draft on overdue (30/60/90); never auto-sent | MUST | planned | FR-CUO-101 | 5h |
| FR-INV-011 | Revenue recognition to GL (accrual or cash basis per tenant policy) | MUST | planned | FR-INV-001 | 5h |

### P2.2 — REW · compensation engine

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-REW-001 | 3P income schema (P1 Base / P2 Allowance / P3 Performance) — encrypted comp keyspace separate from HR | MUST | planned | FR-HR-001 | 6h |
| FR-REW-002 | Parameter versioning (immutable; replay-equivalence ≥ 100% on prior payslips) | MUST | planned | FR-REW-001 | 6h |
| FR-REW-003 | P1 protection invariant — DB CHECK constraint forbids any P1 cash reduction | MUST | planned | FR-REW-001 | 4h |
| FR-REW-004 | Statutory deductions: BHXH 10.5% + BHYT 1.5% + BHTN 1% + PIT progressive (Decree 152/2020) | MUST | planned | FR-HR-005 | 6h |
| FR-REW-005 | Monthly payroll compute + CFO+CHRO co-sign commit gate | MUST | planned | FR-REW-001 | 8h |
| FR-REW-006 | Byte-identical PDF payslip render (Tectonic + pinned fonts) | MUST | planned | FR-REW-005 | 6h |
| FR-REW-007 | BP (Bonus Points) ledger with ACB-rate interest accrual nightly | MUST | planned | FR-REW-001 | 5h |
| FR-REW-008 | Quarterly P3 distribution from BP fund (CEO+CFO sign-off) | MUST | planned | FR-REW-007 | 6h |
| FR-REW-009 | VietQR bank payroll batch send (manual CFO confirm at submission) | MUST | planned | FR-INV-005 | 5h |
| FR-REW-010 | BRAIN structural exclusion CI gate (no comp fields in BRAIN-ingest paths) | MUST | planned | FR-REW-001 | 3h |

### P2.3 — ESOP · Phantom Stock

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-ESOP-001 | SP grant schema with vesting params (4-year + 12-month cliff default) | MUST | planned | FR-HR-001 | 5h |
| FR-ESOP-002 | Monthly vesting accrual deterministic batch | MUST | planned | FR-ESOP-001 | 4h |
| FR-ESOP-003 | Annual valuation (CFO base + Board multiplier sign-off) immutable rows | MUST | planned | FR-ESOP-001 | 5h |
| FR-ESOP-004 | Put-option exec flow (Year 3+, per-Member cap, CFO approve, wire) | MUST | planned | FR-ESOP-003, FR-INV-005 | 8h |
| FR-ESOP-005 | Good/Bad Leaver branch on HR offboarding (CFO + CEO co-sign) | MUST | planned | FR-HR-009 | 5h |
| FR-ESOP-006 | M&A acceleration trigger + Member notice within 5 business days | SHOULD | planned | FR-ESOP-001 | 5h |
| FR-ESOP-007 | Member ESOP dashboard (personal view only; cross-Member requires CFO audit) | SHOULD | planned | FR-ESOP-001 | 6h |

### P2.4 — TEN · billing thin slice (per research review §7.3)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TEN-001 | Tenant provisioning CLI (`cyberos-ten provision`) for ops-driven flow | MUST | planned | FR-AUTH-001 | 5h |
| FR-TEN-002 | 3 plan tiers (Starter/Team/Enterprise) hardcoded | MUST | planned | FR-TEN-001 | 4h |
| FR-TEN-003 | Stripe billing integration (USD/EUR/SGD invoicing for international tenants) | MUST | planned | FR-INV-003 | 8h |
| FR-TEN-004 | 4-axis metering: seats · API · AI tokens · storage (BRAIN audit emission per metric event) | MUST | planned | FR-AI-001 | 8h |
| FR-TEN-005 | Vertical-pack pricing add-on (per-pack monthly fee, not per-seat) | MUST | planned | FR-TEN-002, FR-SKILL-107 | 5h |

### P2.5 — LEARN · skills catalogue + VP + Council

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-LEARN-001 | Skill tree schema + 1-5 mastery levels per skill per Member | MUST | planned | FR-HR-001 | 6h |
| FR-LEARN-002 | Bằng cấp + chứng chỉ (degrees + certifications) evidence types | MUST | planned | FR-LEARN-001 | 4h |
| FR-LEARN-003 | VP (Voting Power) deterministic nightly roll-up (PROJ + TIME + KB inputs) | MUST | planned | FR-PROJ-013, FR-TIME-001 | 6h |
| FR-LEARN-004 | Hội đồng Chuyên môn (Specialist Council) workflow — 3-5 judges + multi-dim scoring | MUST | planned | FR-LEARN-001 | 10h |
| FR-LEARN-005 | Per-judge score isolation (NEVER exit LEARN boundary; HR receives summary + recommendation only) | MUST | planned | FR-LEARN-004 | 5h |
| FR-LEARN-006 | Promotion approval workflow (CEO + CHRO sign-off after council vote) | MUST | planned | FR-LEARN-004 | 5h |
| FR-LEARN-007 | VP score → REW BP fund distribution handoff at quarter close | MUST | planned | FR-LEARN-003, FR-REW-008 | 4h |

### P2 Exit gate criteria

- All 5 P2 modules ship; 17 of 22 total modules live
- First full payroll cycle issued through REW (immutable PDFs replay byte-identical)
- First hóa đơn (Decree 123) emitted and accepted by GDT
- TEN-billing slice live → first paid vertical-pack subscription (cyberskill-vn pilot)
- PCI SAQ-A self-assessment passed (Stripe carries the PCI scope)
- ISO 27017:2015 controls signed off

---

## §5 — P3 · SaaS-ready

**Phase goal:** the platform supports paying external tenants. AUTH grows from stub to full (SSO + MFA + 22 RBAC roles). TEN graduates to self-serve. OKR + RES land. By P3 exit, the agency is ready to serve its first external paying tenant.

**Compliance gate:** ISO 27001:2022 signed-off · PDPL Law 91/2025 audit-ready · external SOC 2 Type II report drafted.

### P3.1 — AUTH (full) · 22-role RBAC + SSO + MFA

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-AUTH-101 | 22-role RBAC catalogue (full bands: root-admin → tenant-member + 17 specialist roles) | MUST | planned | FR-AUTH-005 | 12h |
| FR-AUTH-102 | TOTP + WebAuthn MFA flows | MUST | planned | FR-AUTH-002 | 10h |
| FR-AUTH-103 | SAML 2.0 SSO (per-tenant IdP config) | MUST | planned | FR-AUTH-004 | 12h |
| FR-AUTH-104 | OIDC SSO with discovery + JWKS rotation | MUST | planned | FR-AUTH-004 | 10h |
| FR-AUTH-105 | Passkey enrolment + login | MUST | planned | FR-AUTH-102 | 8h |
| FR-AUTH-106 | Impossible-travel detection + adaptive challenge | SHOULD | planned | FR-AUTH-002 | 8h |
| FR-AUTH-107 | HIBP password breach check on signup + rotation | SHOULD | planned | FR-AUTH-002 | 4h |
| FR-AUTH-108 | Lumi tenant-identity JWT shape (`agent_persona` + `tenant_residency` claims) | MUST | planned | FR-AUTH-101 | 6h |
| FR-AUTH-109 | Stub → full migration path (existing tokens valid for grace window) | MUST | planned | FR-AUTH-101 | 5h |

### P3.2 — TEN (full self-serve)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TEN-101 | Self-serve signup form ≤ 30 s end-to-end | MUST | planned | FR-AUTH-104 | 10h |
| FR-TEN-102 | VnPay + Momo + ZaloPay billing rails for VND domestic | MUST | planned | FR-TEN-003 | 12h |
| FR-TEN-103 | 4-residency provisioning (sg-1 / eu-1 / us-1 / vn-1) | MUST | planned | FR-AI-016 | 10h |
| FR-TEN-104 | 90-day offboarding contract (Active → Terminating-A → Terminating-B → Terminated) | MUST | planned | FR-TEN-001 | 12h |
| FR-TEN-105 | Signed-bundle export (deterministic zip, Ed25519 signature, BRAIN audit anchor) | MUST | planned | FR-TEN-104 | 8h |
| FR-TEN-106 | Permanent-delete attestation row (CSO + CLO sign-off + chained audit) | MUST | planned | FR-TEN-104 | 5h |
| FR-TEN-107 | Tenant-admin SPA (seats / billing / audit / residency / retention) | SHOULD | planned | FR-TEN-101 | 16h |

### P3.3 — OKR · strategy cascade

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-OKR-001 | Objective × KR schema + Company → Team → Member cascade | MUST | planned | FR-AUTH-003 | 6h |
| FR-OKR-002 | 3 KR types (hit-target / improvement / milestone) | MUST | planned | FR-OKR-001 | 4h |
| FR-OKR-003 | KR `progress_source` DSL — query against PROJ / INV / HR / LEARN | MUST | planned | FR-OKR-001 | 10h |
| FR-OKR-004 | Auto-progress nightly batch | MUST | planned | FR-OKR-003 | 5h |
| FR-OKR-005 | Weekly check-in (1-10 confidence + rationale) | MUST | planned | FR-OKR-001 | 5h |
| FR-OKR-006 | Monday-morning CUO digest (auto-progress + check-ins → founder summary) | MUST | planned | FR-CUO-101, FR-OKR-005 | 6h |
| FR-OKR-007 | Quarterly retro draft with face-saving Vietnamese framing | SHOULD | planned | FR-CUO-101, FR-OKR-001 | 6h |

### P3.4 — RES · capacity-vs-forecast + hiring forecast

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-RES-001 | Capacity-vs-demand matrix (HR × PROJ × TIME × LEARN joins, nightly batch) | MUST | planned | FR-HR-001, FR-PROJ-001, FR-TIME-001 | 10h |
| FR-RES-002 | Allocation Gantt UI + drag-rebalance | MUST | planned | FR-RES-001 | 12h |
| FR-RES-003 | Over/under-allocation flags (110% / 60% thresholds) | MUST | planned | FR-RES-001 | 4h |
| FR-RES-004 | Hiring memo CUO draft (skill-gap × CRM pipeline → hire trigger) | MUST | planned | FR-CUO-101, FR-CRM-001 | 8h |
| FR-RES-005 | VN Labour Code Art. 107 OT cap hard-block at allocation propose | MUST | planned | FR-HR-005 | 4h |

### P3 Exit gate criteria

- 19 of 22 modules live
- First external paying tenant onboarded
- Stub → full AUTH migration completed (no impossible-travel exemptions remaining)
- TEN cross-leak property test = 0 over 200+ randomised attempts per release
- ISO 27001:2022 audit signed off
- PDPL Law 91/2025 audit-ready

---

## §6 — P4 · Client-facing

**Phase goal:** external GA. DOC ships for legally-binding signatures. PORTAL ships for client-facing branded surfaces. TEN graduates to full multi-tenant SaaS. Vertical packs publish to a marketplace. Singapore HoldCo flip path activates if ARR ≥ $1.5M.

**Compliance gate:** eIDAS QTSP partnerships in place · AATL CA contracts signed · Singapore ACRA flip path tested in sandbox.

### P4.1 — DOC · legally-binding e-sign

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-DOC-001 | Document repository (versioned, ACL'd, 10-year retention) S3 Object-Lock Compliance bucket | MUST | planned | FR-AUTH-101 | 8h |
| FR-DOC-002 | eIDAS QTSP partner integration (GlobalSign or Cryptomathic) for EU residency | MUST | planned | FR-DOC-001 | 16h |
| FR-DOC-003 | AATL CA partner integration (Adobe-AATL listed) for US/non-EU | MUST | planned | FR-DOC-001 | 12h |
| FR-DOC-004 | VNeID + VN CA chain (VnPay/MK Group/Viettel-CA) for VN tenants | MUST | planned | FR-DOC-001 | 16h |
| FR-DOC-005 | Multi-party signing workflow (ordered + parallel + counter-sign) | MUST | planned | FR-DOC-001 | 10h |
| FR-DOC-006 | Identity verification — WebAuthn / VNeID / SMS-OTP / email-link 4 methods | MUST | planned | FR-AUTH-105 | 8h |
| FR-DOC-007 | Lifecycle metadata (parties / dates / renewal / expiry / parent contract) | MUST | planned | FR-DOC-001 | 5h |
| FR-DOC-008 | Expiry alert cascade (90 / 30 / 7 days) | MUST | planned | FR-DOC-007 | 4h |
| FR-DOC-009 | Renewal proposal CUO draft + AM approval | SHOULD | planned | FR-CUO-101, FR-DOC-007 | 6h |
| FR-DOC-010 | DocuSign / Adobe Sign / HelloSign import (LTV preservation) | SHOULD | planned | FR-DOC-001 | 10h |
| FR-DOC-011 | PAdES-B-LT format with year-9 LTV re-stamping | MUST | planned | FR-DOC-002 | 8h |

### P4.2 — PORTAL · client-facing scoped views

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PORTAL-001 | Scoped read-only view layer (PROJ / INV / DOC / CHAT) filtered by Engagement membership + sync_class=client-visible | MUST | planned | FR-TEN-101 | 12h |
| FR-PORTAL-002 | Per-tenant brand pack (logo + colours + custom CNAME + email template overrides) | MUST | planned | FR-TEN-101 | 8h |
| FR-PORTAL-003 | External IdP SAML 2.0 + OIDC support with JIT user provisioning | MUST | planned | FR-AUTH-103, FR-AUTH-104 | 10h |
| FR-PORTAL-004 | SCIM 2.0 deprovision (session invalidation ≤ 30 s on IdP removal) | MUST | planned | FR-PORTAL-003 | 8h |
| FR-PORTAL-005 | Branded Genie chat (CUO scope-narrowed by JWT scope_grants) | SHOULD | planned | FR-PORTAL-003, FR-CUO-101 | 6h |
| FR-PORTAL-006 | Client-initiated workflows — new project request / billing inquiry / support ticket → CHAT thread | MUST | planned | FR-CHAT-005 | 6h |
| FR-PORTAL-007 | PWA installable (mobile-first) | SHOULD | planned | FR-PORTAL-001 | 6h |
| FR-PORTAL-008 | DSAR self-service (client requests their own data) | MUST | planned | FR-PORTAL-001 | 5h |

### P4.3 — vertical-pack marketplace + HoldCo flip path

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-201 | OCI registry deploy for `.skill` bundles (R3 distribution stage) | MUST | planned | FR-SKILL-102 | 8h |
| FR-SKILL-202 | `cyberskill-sg` pack (Singapore: ACRA filings + GST e-invoice + CPF) | SHOULD | planned | FR-SKILL-107 | 16h |
| FR-SKILL-203 | `cyberskill-id` pack (Indonesia: NPWP + e-Faktur) | COULD | planned | FR-SKILL-107 | 16h |
| FR-TEN-201 | Singapore HoldCo flip CLI (`cyberos-ten holdco-flip`) ACRA filings | MUST | planned | FR-ESOP-001 | 16h |
| FR-TEN-202 | Hostile-termination override (legal-trigger fast-track with CEO+CLO+CSO sign-off) | SHOULD | planned | FR-TEN-104 | 5h |
| FR-TEN-203 | Margin watchdog for fixed-fee engagements (alarm < 30% projected) | SHOULD | planned | FR-PROJ-007 | 5h |

### P4 Exit gate criteria

- All 22 modules live
- First external paying tenant on full multi-tenant SaaS (not pilot)
- Singapore HoldCo flip path tested end-to-end in sandbox (no real flip required for gate)
- eIDAS-conformant signature emitted + verified
- ≥ 5 external paying tenants by P4 · late
- ≥ 2 vertical packs (cyberskill-vn + one of sg/id/th) generating ≥ 30% of ARR

---

## §7 — Cross-phase invariants (NOT FR-level — protocol-level)

These do not have individual FRs because they apply to **every** FR. Auditors of this backlog should check that no FR violates these.

1. **BRAIN audit-row coverage = 100%** — every state-changing operation in every module emits a chained BRAIN audit row before returning success. CI gate per module.
2. **Tenant isolation cross-leak = 0** — property-based test runs per release on every tenant-aware code path. Zero cross-tenant data reads under any randomised query, JWT, label, or ID manipulation.
3. **Compensation never enters BRAIN** — DEC-036 structural exclusion. CI gate rejects any schema PR that lets comp fields appear in BRAIN-ingested paths.
4. **Sensitive PII never enters BRAIN raw** — Presidio + VN-PII recall ≥ 99% gate at every ingest point.
5. **Audit-before-action invariant** — for any action with persistent effect (DB write, network send, file write), the BRAIN audit row MUST land before the effect. CI test asserts ordering on every code path.
6. **Persona-version stamp on every AI call** — `ai.invocation` audit row carries `agent_persona` claim; 100% coverage hard floor.
7. **MUST destructive operations require human confirm** — no LLM-driven loop can auto-invoke a destructive tool. EU AI Act Art. 14 + Anthropic policy floor.

---

## §8 — How this backlog grows

- **New FRs:** authored via the `fr-author` skill per [`FR_AUTHORING_WORKFLOW.md`](../FR_AUTHORING_WORKFLOW.md). The skill writes the markdown into `docs/feature-requests/{module}/` and updates `MANIFEST.json`. This backlog is regenerated from those files.
- **FR status flow:** `draft → audited (fr-audit ran) → accepted (you signed off) → building (in-flight) → shipped (PR merged)`. Or `deferred / rejected / superseded` at any point.
- **Re-prioritising:** edit `priority` in the FR's frontmatter, then re-generate this backlog. Don't edit this index directly — it's a derived view.
- **Re-phasing:** if a P1 FR becomes urgent for P0, edit `phase: P0` in the FR's frontmatter. The phase exit gate criteria above don't change — just move the FR.
- **Deferring a phase:** if a slice can't ship in its planned phase, mark its FRs `deferred` and add a follow-up FR in the next phase with the same scope.

---

*End of backlog v0.1.1 — 2026-05-15.*
*Status:* AI Gateway slice 1 (FR-AI-001..005) now `draft` (5 of ~175 FRs). Next: run `fr-audit` on the batch, user accepts, then implement in dependency order: FR-AI-005 → FR-AI-003 → FR-AI-001 → FR-AI-002 → FR-AI-004.
