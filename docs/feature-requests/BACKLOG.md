# CyberOS — Feature Request Backlog

**Owner:** Stephen Cheng (CEO) · **Status:** v0.7.0 — **STATUS-WAVE-2026-05 lifecycle simplification** (2026-05-19). The previous "tag soup" status enum (`shipped + strict-audited`, `shipped + mocked-dependency`, `[BLOCKED: …]`, `[FAILED: …]`, `accepted`, `building`, `audited`, `planned`, etc.) is **retired**. The new canonical 10-state enum lives at [`STATUS-REFERENCE.md`](../../modules/skill/contracts/feature-request/STATUS-REFERENCE.md): `draft | ready_to_implement | implementing | ready_to_review | reviewing | ready_to_test | testing | done | on_hold | closed`. Migration applied: `planned/accepted/audited/in_review → ready_to_implement`, `building/in_progress → implementing`, `shipped + … → done`, `deferred → on_hold`, `rejected/superseded → closed`, `[BLOCKED: …]/[FAILED: …] → ready_to_implement` (failures route back to rework — see STATUS-REFERENCE §1.3). The CTO workflow that drives ship was also renamed `implement-backlog-frs → ship-feature-requests`. Plus a global rename `brain → memory` across all memory-module references (FR IDs, file names, code identifiers, audit row_kinds, CHANGELOG tags, prose). The CHAT/PROJ/EMAIL Layer-0/2 wave still applies: FR-CHAT-001 + FR-EMAIL-001 + FR-PROJ-001 all moved to `done` (slice 1) on 2026-05-19. Three new `services/<name>/` directories: chat, email, proj. v0.6.0 — PLUGIN module wave + v0.5.0 MEMORY Improvement Wave still apply.
**Source of truth:** the markdown files in this folder. This index is regenerated when FRs land or change status.
**Authoring playbook:** `feature-request-audit` skill (see feature-request skills) (moved 2026-05-18 — was `feature-request-audit` skill at the root of this folder; now co-located with the `feature-request-audit` skill that enforces it)
**Roadmap:** [`../../website/docs/architecture/milestones.html`](../../website/docs/architecture/milestones.html)
**Repo layout:** modules live under `../../modules/<name>/` (post-2026-05-18 refactor)

---

## §0.5 — Implementation-readiness state & deploy roadmap (as of 2026-05-18)

The spec corpus is **closed** and ready for implementation kickoff. Three production modules now ship working bootstraps that the FR-driven implementation phase will extend. The full content of the four generated reports (`CONTRACT_VERIFICATION_REPORT.md` · `IMPLEMENTATION_ORDER.md` · `SPRINT_PLAN.md` · `MIGRATION_AUDIT.md` — previously consolidated in `REPORTS.md`) now lives in the appendices §A through §D at the end of this file.

### Headline metrics

| Metric | Value |
|---|---:|
| Total FRs authored | **261** |
| FRs at 10/10 audit score | **261** (100%) |
| FRs missing audit file | **0** |
| Reciprocity errors in DAG | **0** |
| Total engineering-hours | **~2,056h** (+58h from FR-PLUGIN-001..008 cross-runtime distribution wave 2026-05-19; +120h from FR-MEMORY-112..120 MEMORY Improvement Wave 2026-Q3; +58h from earlier FR-SKILL-111..115 Anthropic Skills portability wave) |
| Modules with full spec coverage | **25** |
| Dependency layers (topo build sequence) | **13** |
| API endpoints declared in §3 contracts | **262** (+1 from FR-MEMORY-120 history endpoint) |
| Migration files declared | **327** across 23 modules |

> **2026-05-19 — PLUGIN module added (cross-runtime distribution wave).** Authored FR-PLUGIN-001 (manifest schema v1.0.0 + Python reference packer) + FR-PLUGIN-002 (MCP bridge Rust binary supporting stdio+HTTP transports, 8 tools across CUO/memory/SKILL with Tasks primitive for long-running execute_workflow) + FR-PLUGIN-003 (4 canonical slash-commands) + FR-PLUGIN-004 (12 skill playbooks teaching hosts when to chain tools) + FR-PLUGIN-005 (OAuth-PKCE auth with audience-bound JWTs + 24h rotating refresh + OS-keychain storage) + FR-PLUGIN-006 (memory audit emission with durable Postgres outbox + idempotent retry + 24h exponential backoff) + FR-PLUGIN-007 (multi-runtime adapters for claude-code / cursor / cowork / codex-cli, P2 targets deferred) + FR-PLUGIN-008 (marketplace publish to plugins.cyberskill.world + mirror to agentskills.io + 70/30 revenue share + vetted-by-CyberSkill JWT badge). All 8 at 10/10. **+58h.** Strategy alignment: §4 Level 1 (OSS distribution) + Level 3 (marketplace) + Level 4 (vertical packs as plugins) + Level 5 (private-marketplace white-label). See [`plugin/`](plugin/) for the FR catalog and the [Plugin docs page](https://cyberos-wiki.cyberskill.world/modules/plugin/) for the module-side description.
>
> **2026-05-19 — MEMORY Improvement Wave 2026 Q3.** Authored FR-MEMORY-112 (episodic memory + recall-similar) + FR-MEMORY-113 (Park-et-al recency/importance/relevance combined-score) + FR-MEMORY-114 (Haiku-rated write-time importance) + FR-MEMORY-115 (`cyberos dream` out-of-band reflection, **gated on `APPROVE protocol change P19 §7.7`**) + FR-MEMORY-116 (semantic-dedup consolidation phase) + FR-MEMORY-117 (per-store ACL via STORE.yaml, **gated on `APPROVE protocol change P20 §14.4`**) + FR-MEMORY-118 (`put_if` precondition-hash, **gated on `APPROVE protocol change P21 §3.1`**) + FR-MEMORY-119 (session-transcript ledger, **gated on `APPROVE protocol change P22 §18`**) + FR-MEMORY-120 (`cyberos history` projection). All 9 at 10/10. See [`memory/`](memory/) for the FR catalog. Sources: Anthropic Memory+Dreaming talk (`playground/extracts/memory-and-dreaming.transcript.txt`) + Ramakrushna agentic-memory article (`playground/extracts/agentic-memory.article.txt`). Total ~120h across 3 sub-waves (4d / 9d / 8d). Three protocol amendments live independently per Stephen's one-at-a-time decision.

> **2026-05-19 — Anthropic Skills portability wave.** Authored FR-SKILL-111 (description trigger enrichment) + FR-SKILL-112 (TRIGGER_TESTS.md) + FR-SKILL-113 (XML-free frontmatter, registry v0.2.5) + FR-SKILL-114 (BASELINE.md at v1.0 promotion) + FR-SKILL-115 (134-file placeholder sweep, queued for v0.2.6). All 5 at 10/10. See the [SKILL appendices](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) for the portability findings (Appendix J) and the 3-session implementation cut (Appendix K, 38-46h).

### Production module status

| Module | Layer | What's shipped (bootstrap) | What FRs cover next |
|---|---|---|---|
| `modules/memory/` | memory protocol + Python impl | 255 green tests; P1–P12 audit proposals + P2 Stage 3 MMR/STH; HTTP REST; cross-memory merge; deterministic export | `memory/FR-MEMORY-101…111` extend Layer-2 ingest, multi-device sync, search API, fs-watcher, capture daemon, pre-ingest PII. **2026 Q3 MEMORY Improvement Wave (`memory/FR-MEMORY-112…120`, all at 10/10)** adds episodic memory + recall-similar (112), Park-et-al recency-decay recall (113), Haiku-rated write-time importance (114), `cyberos dream` out-of-band reflection (115, **§7.7 amendment gated**), semantic-dedup consolidate (116), per-store STORE.yaml ACL (117, **§14.4 amendment gated**), put_if precondition-hash (118, **§3.1 amendment gated**), session transcript ledger (119, **§18 amendment gated**), `cyberos history` projection (120) |
| `modules/skill/` | Anthropic Agent Skills catalog | 104 author+audit pairs (208 bundles) + 108 contracts; all chain through SDP; zero `planned:` gaps; **registry v0.2.5 (2026-05-19): Anthropic Skills portability — FR-SKILL-111..114 shipped, FR-SKILL-115 queued for v0.2.6**; 209-file `wrap_in:` → `wrap_in_marker:` sweep complete; 3 backfill exemplars (FR-author / FR-audit / PRD-author) carry enriched descriptions + TRIGGER_TESTS.md fixtures; `cuo.trigger_tests` + `cuo.baseline` Python validators shipped | `skill/FR-SKILL-101..115` + `…201` add OCI registry, capability broker, memory-capture/sync system skills, VN-regulatory bundles, plus FR-115 (134-file stale-placeholder sweep). [SKILL_BUNDLE_RUBRIC.md](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) (Appendix L on the docs site) |
| `modules/cuo/` | Python supervisor v3.0.0-a4 | Catalog scanner + 2-stage router with domain-fallback + Invoker ABC (Mock/Subprocess/LLM) + execute_chain + memory bridge + Phase 4 special-case handlers (Linear / TimeCritical / PerInstance / MultiOutput / SequentialApproval / PersonaPair); **49/50 tests pass**; 47/48 personas + 221 workflows (Tier-C1 depth additions shipped) | Implementation phase: deploy modules per the §0.6 roadmap |
| `modules/plugin/` | Cross-runtime distribution scaffold (2026-05-19) | `README.md` + `INTEROP.md` + `manifest.schema.json` + `AGENTS.md` symlink + commands/ + adapters/ + manifests/ folders scaffolded | **8 FRs at 10/10** (`plugin/FR-PLUGIN-001..008`, 58h): manifest schema + MCP bridge + slash commands + skill playbooks + OAuth-PKCE auth + memory audit emission + 4 P1 runtime adapters (claude-code / cursor / cowork / codex-cli) + marketplace publish to plugins.cyberskill.world with agentskills.io mirror. Runtime lands at `services/plugin-host/` per the implementation order |
| `services/ai-gateway/` | Rust workspace member (NEW 2026-05-19) | **FR-AI-003 + FR-AI-005 shipped.** FR-AI-003 (memory audit-row bridge): subprocess writer · canonical-JSON serialisation per AGENTS.md §6.2 · NFC normalisation · SHA-256 chain verification · path-traversal guard · 5s timeout w/ SIGTERM-then-SIGKILL · 5 typed builders (precheck · invocation · invocation_failed · hold_expired · persona_loaded) · `check_writer_available` startup health check. FR-AI-005 (tenant-policy YAML loader): `TenantPolicy` schema (cap · warn · hard_stop · provider · fallback_chain · timeout · residency · zdr · emergency_override · allowed_personas) · `ArcSwap`-backed lock-free cache · `notify` file-watcher with NFS polling-mode detection · 10 ACs covered via integration tests · CI gate via `gen-schema` binary · `cyberos-ai policy validate \| list \| serve` CLI · `EXAMPLE.tenant.yaml` reference. Workspace wired in `services/Cargo.toml`. | FR-AI-001 (cost-ledger pre-call) + FR-AI-002 (post-call reconcile) + FR-AI-004 (expiry cleanup) — all unblocked by this ship (FR-AI-003 + FR-AI-005 were their dependencies); land in next session. FR-AI-006..022 (router · PII · residency · cache · operator CLI · OTel) follow per slice 2–5 in §2 P0.1. |
| `services/obs-collector/` | Rust workspace member (NEW 2026-05-19) | **FR-OBS-001 slice-1 scaffold shipped** (building → shipped pending live LGTM deploy). Canonical `otel-collector-config.yaml` matching FR-OBS-001 §3 (OTLP receivers · resource processor · attributes/pii_scrub processor with PROMPT_TEXT/RESPONSE_TEXT/USER_EMAIL/CCCD deletion + secret-pattern rule · batch 10s/1024 · loki + prometheusremotewrite + otlp/tempo exporters · file_storage + bearertokenauth + health_check extensions). `cyberos-obs validate-config` + `validate-tokens` pre-flight CI gates. Self-metric name + label constants (`obs_collector_received_spans_total`, `dropped_total{reason}`, `export_latency_ms{backend}`). Bearer-token file parser + `tokens.example` template. Workspace-wired Rust crate. | FR-OBS-002 (Grafana tenant-aware proxy) + FR-OBS-003 (RED metrics SDK) + FR-OBS-004 (LangSmith) + FR-OBS-005 (TraceContext correlation) + FR-OBS-006 (tail sampling) + FR-OBS-007 (Alertmanager → CUO routing) + FR-OBS-008 (compliance views) + FR-OBS-009 (chain-of-custody manifest). The live LGTM stack deployment (Helm + docker-compose for otelcol-contrib + Loki + Prometheus + Tempo + Grafana) lands at `deploy/obs/` next session. |
| `services/mcp-gateway/` | Rust workspace member (NEW 2026-05-19) | **FR-MCP-001 slice-1 scaffold shipped** (building → shipped pending JWT + SSE + audit wiring). JSON-RPC 2.0 parser (single + batch + notifications + empty-batch rejection) · closed error-code map (DEC-272 — -32700/-32600/-32601/-32602/-32603 + MCP-defined -32001/-32002/-32003/-32004/-32005) · `initialize` handshake with protocol-version-mismatch error · `Capabilities` advertisement (tools+prompts+resources+logging per DEC-266) · `tools/list` with base64 cursor pagination at PAGE_SIZE=100 · `tools/call` dispatch with permission gate returning `module_unreachable` until FR-MCP-002 lands · `ToolAnnotations` (destructive/readOnly/idempotent/openWorld) · `ToolRegistry` in-memory cache · Axum router mounting `POST /mcp` + `GET /mcp/healthz`. Tests: jsonrpc parsing + error codes + initialize match/mismatch + pagination + tools/call permission gate + router dispatch end-to-end. | FR-MCP-002 (per-module registration + heartbeat) + FR-MCP-003 (SEP-986 validator) + FR-MCP-004 (OAuth 2.1 PKCE) + FR-MCP-005 (PRM at well-known) + FR-MCP-006 (destructive-hint gating + Elicitation) + FR-MCP-007 (Tasks primitive) + FR-MCP-008 (Elicitation flow). JWT verification + per-(tenant,tool) rate-limit + audit emission + Streamable HTTP SSE transport land in follow-on slices. |

### §0.6 — Deploy roadmap → cyberos.cyberskill.world (Vercel)

The website (this docs site at `website/docs/`) deploys to **cyberos.cyberskill.world** via Vercel. The runtime modules ship on a separate cadence to their own production targets and are then linked back from this site. The user-locked production order is:

```
  ┌──────────┐    ┌────────┐    ┌────────┐    ┌─────────┐    ┌──────────────┐
  │  MEMORY  │ ─▶ │  AUTH  │ ─▶ │  CHAT  │ ─▶ │ PROJECT │ ─▶ │  CUO + SKILL │
  │ (memory)  │    │        │    │        │    │ (PROJ)  │    │              │
  └──────────┘    └────────┘    └────────┘    └─────────┘    └──────────────┘
```

| # | Wave | Modules | What lands | Spec basis | Deploy target |
|---:|---|---|---|---|---|
| 1 | **MEMORY** | `modules/memory/` (memory protocol) | Layer-2 ingest pipeline · multi-device sync · capture daemon · fs-watcher · pre-ingest PII · search API · Tauri client | `memory/FR-MEMORY-101…111` (Layer 1–2 in §B topo order) | `cyberos.cyberskill.world` docs page goes live with current state; runtime ships to per-engineer Tauri + Cloud memory on Fargate |
| 2 | **AUTH** | `modules/auth/` | Tenant + Subject create · RLS · JWT/JWKS · admin REST · MFA (TOTP/WebAuthn/Passkey) · SAML/OIDC SSO · 22-role RBAC catalogue | `auth/FR-AUTH-001…109` (Layer 0–4) | AWS Fargate per-region (sg-1 · vn-1) with TLS termination at Apollo Router; AUTH docs page refreshes with shipped status |
| 3 | **CHAT** | `modules/chat/` | Mattermost fork pinning · AUTH bridge · Fargate deployment · VN-search · memory bridge · Slack/Zalo import · mobile push · DSAR export · decommission signal | `chat/FR-CHAT-001…012` (Layer 0–8, depends on AUTH) | Per-tenant Mattermost on Fargate behind Apollo Router; CHAT docs page reflects dogfood status + `decommission_signal ≥ 0.95` gate |
| 4 | **PROJECT** | `modules/proj/` | Issue + Cycle + Engagement schema · Yjs CRDT · issue lifecycle FSM · billable cascade · rate-card · MEMORY_LINK · estimate calibration · Kanban/Timeline/Gantt views · design-tokens a11y CI | `proj/FR-PROJ-001…018` (Layer 2–5, depends on AUTH + memory) | Same Fargate fleet; PROJ docs page refreshes |
| 5 | **CUO + SKILL** | `modules/cuo/` + `modules/skill/` runtime | LangGraph productionization · PG checkpointer · trace replay · topological walk · per-step rollback · OCI registry · capability broker · memory-capture/sync system skills · VN-regulatory bundles | `cuo/FR-CUO-101…106` + `skill/FR-SKILL-101…201` (Layers 2–6) | CUO supervisor binary distributed via OCI; SKILL host as Rust workspace; docs site documents the full runtime |

**Deploy mechanics for the docs site (cyberos.cyberskill.world):**
- Static-site build → `vercel deploy --prod` from repo root.
- `vercel.json` + `.vercelignore` in place at repo root.
- Vercel project `cyberos-docs` under team `team_9SRH0b2jquntBO1gu2jDA5zP` (Stephen Cheng's projects).
- DNS: `CNAME cyberos → cname.vercel-dns.com`.
- Runbook: [`../../DEPLOY-VERCEL.md`](../../website/docs/DEPLOYMENT.md).
- Recurring publish: `git push` (when Git integration enabled) OR `vercel deploy --prod` from repo root.

**Gate criteria for advancing waves:**
1. Wave N modules pass all FR-level acceptance criteria (§4 of each FR) before Wave N+1 starts.
2. Each module's docs page on cyberos.cyberskill.world updates with the shipped state at wave-exit.
3. memory audit-row coverage = 100% across all wave-N shipped surfaces (see `feature-request-audit` skill (see feature-request skills) §10.1 invariant 1).
4. Tenant isolation cross-leak = 0 verified by property tests (see `feature-request-audit` skill (see feature-request skills) §10.1 invariant 2).

### What changed since v0.5.0 (2026-05-19 evening — AI / OBS / MCP wave)

- **P0 implementation phase kicks off — three new Rust workspace members shipped.** Per Stephen's "cyberos implement AI Gateway + OBS + MCP Gateway" directive (2026-05-19 evening session), three foundational P0 services now exist under `services/` as workspace members alongside the previously-shipped `auth`/`memory`/`skill-broker` crates:
  - `services/ai-gateway/` — **FR-AI-003 + FR-AI-005 shipped end-to-end (both 10/10)**. FR-AI-003 is the canonical Python memory Writer subprocess bridge with NFC normalisation + SHA-256 chain verification + path-traversal guard + 5s timeout w/ SIGTERM-then-SIGKILL + 5 typed builders for the slice-1 closed set (`ai.precheck` · `ai.invocation` · `ai.invocation_failed` · `ai.hold_expired` · `ai.persona_loaded`). FR-AI-005 is the per-tenant YAML policy loader with `ArcSwap`-backed lock-free cache + `notify` file-watcher + 10 ACs covered by integration tests + `gen-schema` CI gate + `cyberos-ai policy validate \| list \| serve` CLI. These two foundational FRs unblock FR-AI-001/002/004 (cost-ledger), all three of which now have green dependencies. Next session lands the cost-ledger trio.
  - `services/obs-collector/` — **FR-OBS-001 slice-1 scaffold shipped (10/10, status flipped to `building`).** Canonical `otel-collector-config.yaml` matching the FR-OBS-001 §3 contract (OTLP receivers + pii_scrub processor + LGTM exporters + bearertokenauth + file_storage + health_check extensions). `cyberos-obs validate-config` + `validate-tokens` pre-flight CI gates. Self-metric name + label constants. Bearer-token file parser. Workspace-wired Rust crate. The live LGTM deployment (Helm + docker-compose for otelcol-contrib + Loki + Prometheus + Tempo + Grafana) lands at `deploy/obs/` next session, at which point the status flips to `shipped`.
  - `services/mcp-gateway/` — **FR-MCP-001 slice-1 scaffold shipped (10/10, status flipped to `building`).** JSON-RPC 2.0 parser (single + batch + notifications + empty-batch rejection) · closed error-code map (DEC-272 — -32700/-32600/-32601/-32602/-32603 + MCP-defined -32001..-32005) · `initialize` handshake with protocol-version-mismatch error · `Capabilities` advertisement per DEC-266 · `tools/list` with base64 cursor pagination at PAGE_SIZE=100 · `tools/call` dispatch with permission gate returning `module_unreachable` until FR-MCP-002 lands · `ToolAnnotations` · `ToolRegistry` · Axum router mounting `POST /mcp` + `GET /mcp/healthz` with `MCP_PROTOCOL_VERSION` pinned at `"2025-11-25"`. Tests across jsonrpc + error codes + initialize + pagination + dispatch + router. JWT verification (FR-MCP-004) + per-(tenant,tool) rate-limit + audit emission + Streamable HTTP SSE transport land in follow-on slices.

The three new crates are wired in `services/Cargo.toml` as workspace members. **The Rust toolchain is not available in the agent sandbox; Stephen runs `cargo check -p cyberos-ai-gateway -p cyberos-obs-collector -p cyberos-mcp-gateway --tests` locally to validate the build.**

### What changed since v0.4.0 (2026-05-19)

- **MEMORY Improvement Wave 2026 Q3:** 8 FRs (FR-MEMORY-112..119) plus one ergonomic projection FR (FR-MEMORY-120) authored at 10/10. Aligns CyberOS MEMORY with Anthropic's Memory+Dreaming primitive + Ramakrushna's agentic-memory taxonomy. Adds episodic memory, recency-decay recall, write-time importance scoring, the out-of-band dream pipeline, semantic-dedup consolidation, per-store ACL, optimistic-concurrency put_if, session transcript ledger, and `cyberos history`. Four FRs carry protocol amendments (§7.7 / §14.4 / §3.1 / §18) gated by independent `APPROVE` chat-turns per Stephen's 2026-05-19 decision. Total +120h. FRs live at [`memory/FR-MEMORY-{112..120}.md`](memory/). Source materials: `playground/extracts/agentic-memory.article.txt` + `playground/extracts/memory-and-dreaming.transcript.txt`.

### What changed since v0.3.0 (2026-05-18)

- **Skill catalog rename:** all 104 author+audit pairs (+ contracts) renamed from short-form (e.g. `fr-audit`) to full-form (e.g. `feature-request-audit`). Tests still 49/50 green. Public-bundle vn-* renamed to vietnam-*.
- **feature-request-audit skill absorbed:** `feature-request-audit` skill folded into `feature-request-audit` skill so the discipline lives next to the auditor that enforces it.
- **REPORTS.md absorbed:** the four generated reports (contract verification · implementation order · sprint plan · migration audit) now live as §A/§B/§C/§D appendices in THIS file. `REPORTS.md` deleted. Regenerate-in-place workflow unchanged — the regen scripts now write into this file's appendices.
- **CUO supervisor Phase 4:** 5 special-case workflow handlers shipped. Version `3.0.0-a3 → 3.0.0-a4`. `FR-CUO-106` authored at 10/10 covering the handlers.

---

## §0 — How to read this backlog

This document is the **single source of truth** for what CyberOS is going to build, organised by **phase** (P0 → P4), then by **module**, then by **slice** within each module. Every row is one FR; one FR is one atomic, testable requirement.

- **Phase** maps to the milestone arc — `P0 Foundation` ships the cross-cutting infrastructure; `P1 Productivity` adds the internal-workflow modules; `P2 Operations` adds revenue + ops surfaces; `P3 SaaS-ready` adds the multi-tenant + employment-decision modules; `P4 Client-facing` adds external-customer surfaces.
- **Slice** is a coherent ship-unit within a module. Slice 1 is always the minimum viable surface for that module. Subsequent slices add depth, scale, compliance hardening, or persona surfaces.
- **Priority** uses BCP-14 keywords — `MUST` (release blocker) · `SHOULD` (release should-have) · `COULD` (release nice-to-have) · `MAY` (post-release).
- **Status**: The current state of the FR (see [`STATUS-REFERENCE.md`](../../modules/skill/contracts/feature-request/STATUS-REFERENCE.md) for details).
- **Depends on**: The cross-FR dependency list.
- **Effort** is a rough sizing in hours (1h = 30 min focused work + 30 min coordination/review). Treat as ±50%. Sized for one experienced engineer.

**Reading order for a planner:** scan §1 (totals) → pick the phase you're working in → read the per-module breakdown in that phase → drill into individual FR markdowns.

**Reading order for an implementer:** find the assigned FR-ID in the per-module section → click through to the FR markdown file for details.

---

## §1 — Totals at a glance

| Phase | Modules in scope | FRs planned | Estimated effort (person-weeks) | Compliance gate at exit |
|---|---|---:|---:|---|
| **P0 — Foundation** | AI Gateway · OBS · AUTH (stub) · MCP Gateway · CHAT (dogfood) | ~37 | ~12 | SOC 2 readiness · CHAT decommission ≥ 0.95 |
| **P1 — Productivity** | memory (auto-sync) · SKILL (packs) · PROJ · CRM · TIME · KB · EMAIL · HR · CUO (Phase 2 LLM) | ~58 | ~24 | EU AI Act Art. 12 ready · GDPR Art. 30 RoPA |
| **P2 — Operations** | INV · REW · ESOP · TEN (billing slice) · LEARN | ~32 | ~14 | PCI SAQ-A · Vietnamese hóa đơn Decree 123 compliant |
| **P3 — SaaS-ready** | TEN (full self-serve) · AUTH (full) · OKR · RES · HR (P3 extensions) | ~26 | ~12 | ISO 27001:2022 · PDPL Law 91/2025 full |
| **P4 — Client-facing** | DOC · PORTAL · TEN (external GA) · vertical-pack marketplace | ~22 | ~10 | eIDAS QTSP · AATL · Singapore HoldCo flip path |
| **Total** | 23 modules · 5 phases | **~175** | **~72 person-weeks** | 5 gated compliance milestones |

**Effort budget reality-check:** 175 FRs × 8h average = 1,400h ≈ 35 person-weeks of pure coding. The 72 person-weeks total accounts for design + review + integration + the inevitable surprise. That maps to roughly 18 months of one full-time engineer, or 9 months of two — which is consistent with the milestone arc on the docs site.

**Shipped state (excluded from the backlog count):** memory Layer 1 (memory module — 6 ops, 15 invariants, 255 tests, MMR + Ed25519 STH); SKILL Phases 0–7 (open Agent Skills standard, Rust host, Bun toolchain); CUO Phase 1 (rule-based router, 6 core modules, 15 fixtures). These don't appear in this backlog because the work is done — their next-slice FRs (memory auto-sync Stages 1–5; SKILL Phase 8 memory integration; CUO Phase 2 LangGraph cascade) DO appear below.

---

## §2 — P0 · Foundation

**Phase goal:** stand up the cross-cutting infrastructure every other module depends on. By P0 exit (5 modules live), CyberSkill team members dogfood CHAT instead of Slack/Zalo, Genie answers route through CUO, every LLM call passes through the cost-of-everything gate, every action carries a memory audit row, and the OBS plane gives one investigation surface for any incident.

**Compliance gate:** SOC 2 Type II readiness signal (RBAC + audit chain + retention policies). CHAT `decommission_signal ≥ 0.95` over 14-day rolling window at P0 exit (M+3 equivalent) — miss this and we hit the P0→P1 descope gate.

**Build order (locked):** AI Gateway → OBS → AUTH stub → MCP Gateway → CHAT.

### P0.1 — AI Gateway · the cost-of-everything gate

**Module page:** [`ai.html`](../../website/docs/modules/ai.html) · **Owner:** CTO · **Slice plan:** 5 slices, 22 FRs

#### Slice 1 — cost ledger + provider abstraction core

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-001** | AI Gateway cost-ledger pre-call check | MUST | ready_to_implement | FR-AI-003, FR-AI-005 | 8h |
| **FR-AI-002** | AI Gateway cost-ledger post-call reconcile | MUST | ready_to_implement | FR-AI-001, FR-AI-003 | 6h |
| **FR-AI-003** | memory audit-row bridge (`ai.invocation` chained row per call) | MUST | done | — | 5h |
| **FR-AI-004** | Cost-hold expiry cleanup job (Postgres scheduled) | MUST | ready_to_implement | FR-AI-001, FR-AI-003 | 3h |
| **FR-AI-005** | Tenant-policy YAML loader (per-tenant cap + warn threshold + override) | MUST | done | — | 5h |

#### Slice 2 — multi-provider router (Bedrock + Anthropic + OpenAI)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-006** | Model-alias resolution (`chat.smart` → `bedrock:claude-3.5-sonnet`) with per-tenant override | MUST | ready_to_implement | FR-AI-005 | 6h |
| **FR-AI-007** | Provider cost-table loader (YAML, hot-reload) | MUST | ready_to_implement | — | 4h |
| **FR-AI-008** | LiteLLM-derived router with retry + 30 s failover SLA | MUST | ready_to_implement | FR-AI-006, FR-AI-007 | 10h |
| **FR-AI-009** | Circuit-breaker per (provider, model) with half-open recovery probing | MUST | ready_to_implement | FR-AI-008 | 6h |
| **FR-AI-010** | Streaming SSE end-to-end (token-by-token to client) | SHOULD | ready_to_implement | FR-AI-008 | 8h |

#### Slice 3 — PII redaction + persona stamping

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-011** | Presidio EN-base PII redaction in-flight (every prompt) | MUST | ready_to_implement | FR-AI-008 | 6h |
| **FR-AI-012** | VN-PII Presidio plugin (CCCD · MST · VN phone · NĐD · VN address · bank account) | MUST | ready_to_implement | FR-AI-011 | 10h |
| **FR-AI-013** | VN-PII recall ≥ 99% CI gate on 200-sample test set | MUST | ready_to_implement | FR-AI-012 | 4h |
| **FR-AI-014** | Persona-version system-prompt injection from memory `memories/personas/<version>.md` | MUST | ready_to_implement | FR-AI-003 | 5h |
| **FR-AI-015** | ZDR check — refuse non-ZDR provider when tenant policy requires it | MUST | ready_to_implement | FR-AI-006 | 3h |

#### Slice 4 — geographic residency + per-tenant cache

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-016** | Tenant residency pinning (`sg-1` / `eu-1` / `us-1` / `vn-1`) propagating to provider selection | MUST | ready_to_implement | FR-AI-006 | 5h |
| **FR-AI-017** | Cache (Redis) keyed by (tenant_id × prompt_hash × model); ≥ 30% hit rate P0 target | SHOULD | ready_to_implement | FR-AI-008 | 6h |
| **FR-AI-018** | Cross-tenant cache leak property-test (hard zero) | MUST | ready_to_implement | FR-AI-017 | 3h |
| **FR-AI-019** | Self-hosted BGE-M3 embeddings (single L4 GPU pod) + CPU fallback | SHOULD | done | — | 8h |
| **FR-AI-020** | BGE-rerank-v2-m3 cross-encoder for KB reranking | COULD | ready_to_implement | FR-AI-019 | 5h |

#### Slice 5 — operator surface + observability

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-AI-021** | `cyberos-ai` operator CLI (usage · models list · policy set · failover drill · invoice export) | MUST | ready_to_implement | FR-AI-008, FR-AI-005 | 8h |
| **FR-AI-022** | OTel trace + span emission for every call (caller → router → provider → response) | MUST | ready_to_implement | FR-AI-008 | 4h |

---

### P0.2 — OBS · observability spine

**Module page:** [`obs.html`](../../website/docs/modules/obs.html) · **Owner:** CTO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — three pillars wired

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| **FR-OBS-001** | OTel collector deployed (Loki + Prometheus + Tempo backends) | MUST | implementing | — | 6h |
| FR-OBS-002 | Grafana stood up with tenant-aware query proxy (Rust) injecting `tenant_id` label | MUST | ready_to_implement | **FR-OBS-001** | 8h |
| FR-OBS-003 | Per-service RED metrics emitted (rate / errors / duration) via OTel SDK | MUST | ready_to_implement | **FR-OBS-001** | 5h |

#### Slice 2 — AI traces + cross-pillar correlation

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-OBS-004 | LangSmith integration for AI traces (tied to operational trace_id) | MUST | ready_to_implement | FR-AI-022 | 5h |
| FR-OBS-005 | Trace × log × metric × AI-trace correlation via W3C TraceContext propagation | MUST | ready_to_implement | FR-OBS-003, FR-OBS-004 | 6h |
| FR-OBS-006 | Tail-based sampling (100% on errors, 10% normal) via OTel Collector | SHOULD | ready_to_implement | **FR-OBS-001** | 4h |

#### Slice 3 — auto-runbook router + compliance surfaces

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-OBS-007 | Alert Manager → CUO `obs.triage-alert@1` skill routing (≥ 0.70 conf → CHAT; else PagerDuty) | MUST | ready_to_implement | FR-OBS-003 | 8h |
| FR-OBS-008 | Compliance view scoping (EU AI Act / PDPL / SOC 2 / ISO 27001) over memory audit chain | MUST | ready_to_implement | FR-OBS-002 | 10h |
| FR-OBS-009 | Chain-of-custody manifest with Ed25519 signature on compliance exports | MUST | ready_to_implement | FR-OBS-008 | 6h |

---

### P0.3 — AUTH (stub) · M+2-equivalent ship

**Module page:** [`auth.html`](../../website/docs/modules/auth.html) · **Owner:** CTO · **Slice plan:** 5 slices (P0 = slice 1; remainder defers to P3 full)

#### Slice 1 — stub (5 roles, password + WebAuthn, no MFA, no SSO)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-AUTH-001 | Tenant create (root-admin in tenant 0 calls `POST /v1/admin/tenants`) | MUST | done | — | 6h |
| FR-AUTH-002 | Subject create (`POST /v1/admin/subjects`) with bcrypt hashed password | MUST | done | FR-AUTH-001 | 5h |
| FR-AUTH-003 | RLS enforcement at every table (Postgres `current_setting('app.tenant')` predicate) | MUST | done | FR-AUTH-001 | 8h |
| FR-AUTH-004 | JWT issuance + JWKS endpoint (RS256) with `tenant_id` + `agent_persona` + `scope_grants` claims | MUST | done | FR-AUTH-002 | 6h |
| FR-AUTH-005 | Admin REST: list tenants + list subjects + revoke subject | MUST | done | FR-AUTH-001, FR-AUTH-002 | 5h |
| FR-AUTH-006 | `cyberos-auth bootstrap` CLI for tenant-0 root-admin (no UI required) | MUST | done | FR-AUTH-001 | 3h |

---

### P0.4 — MCP Gateway · external-agent door

**Module page:** [`mcp.html`](../../website/docs/modules/mcp.html) · **Owner:** CTO · **Slice plan:** 3 slices, 8 FRs

#### Slice 1 — federation core

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MCP-001 | MCP 2025-11-25 spec compliance — `tools/list`, `tools/call`, `capabilities` | MUST | implementing | FR-AUTH-004 | 12h |
| FR-MCP-002 | Per-module server registration + heartbeat lifecycle (3-miss → unhealthy) | MUST | ready_to_implement | FR-MCP-001 | 6h |
| FR-MCP-003 | SEP-986 naming convention validator (`cyberos.{module}.{verb}_{noun}`) | MUST | ready_to_implement | FR-MCP-001 | 3h |

#### Slice 2 — OAuth 2.1 PKCE + audience binding

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MCP-004 | OAuth 2.1 PKCE authorization-code flow with audience-bound tokens | MUST | ready_to_implement | FR-AUTH-004 | 10h |
| FR-MCP-005 | Protected Resource Metadata (PRM, RFC 9728) at `/.well-known/oauth-protected-resource` | MUST | ready_to_implement | FR-MCP-004 | 3h |
| FR-MCP-006 | Tool-annotation gating (`destructive` requires explicit confirm or Elicitation flow) | MUST | ready_to_implement | FR-MCP-001 | 6h |

#### Slice 3 — Tasks primitive + Elicitation

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MCP-007 | Tasks primitive (long-running work with status polling + resume on reconnect) | MUST | ready_to_implement | FR-MCP-001 | 10h |
| FR-MCP-008 | Elicitation server-initiated request/response for mid-call user prompts | MUST | ready_to_implement | FR-MCP-001 | 6h |

---

### P0.5 — CHAT · P0 dogfood gate

**Module page:** [`chat.html`](../../website/docs/modules/chat.html) · **Owner:** CPO · **Slice plan:** 4 slices, 12 FRs

#### Slice 1 — Mattermost fork + AUTH bridge

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-001 | Mattermost v9.x fork at pinned MIT/Apache commit + license-drift watcher | MUST | ready_to_implement | — | 8h |
| FR-CHAT-002 | `cyberos-chat-authbridge` plugin — Mattermost auth delegates to AUTH JWT | MUST | ready_to_implement | FR-CHAT-001, FR-AUTH-004 | 10h |
| FR-CHAT-003 | Per-tenant deployment via Fargate + RDS Multi-AZ + Redis | MUST | ready_to_implement | FR-CHAT-001 | 6h |

#### Slice 2 — VN search + memory bridge

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-004 | PGroonga + TinySegmenter VN bigram tokeniser; recall ≥ 80% CI gate | MUST | ready_to_implement | FR-CHAT-003 | 12h |
| FR-CHAT-005 | memory bridge — logical-replication from Postgres → memory Layer-3 ingest, p95 ≤ 5 s | MUST | ready_to_implement | FR-CHAT-003 | 10h |

#### Slice 3 — Slack/Zalo migration + @lumi capture

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-006 | Slack import (`cyberos-chat import slack` — 8-step idempotent + checkpointed) | MUST | ready_to_implement | FR-CHAT-005 | 12h |
| FR-CHAT-007 | Zalo manual export importer (`cyberos-chat import zalo --bundle.zip`) | SHOULD | ready_to_implement | FR-CHAT-005 | 8h |
| FR-CHAT-008 | `@lumi` mention parser → CUO route → memory capture row | MUST | ready_to_implement | FR-CHAT-005, FR-AI-014 | 6h |
| FR-CHAT-009 | Retro-capture flow — `@lumi remember the last N messages` with per-message opt-in | SHOULD | ready_to_implement | FR-CHAT-008 | 6h |

#### Slice 4 — decommission instrumentation + DSAR

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CHAT-010 | `decommission_signal := (chat msgs) / (chat + slack + zalo msgs) ≥ 0.95` over 14-day rolling window | MUST | ready_to_implement | FR-CHAT-006 | 5h |
| FR-CHAT-011 | Mobile push delivery (privacy-preserving payload: title + sender only) | MUST | ready_to_implement | FR-CHAT-003 | 6h |
| FR-CHAT-012 | DSAR export — every message a subject authored + chained memory audit hashes | MUST | ready_to_implement | FR-CHAT-005 | 6h |

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

**Critical dependencies:** all P1 modules require P0 complete (AUTH stub, memory audit, AI Gateway cost cap, OBS, MCP federation).

### P1.1 — memory auto-sync · Stages 1–2 (Personal memory + Capture daemon)

**Module page:** [`memory.html`](../../website/docs/modules/memory.html) · **Owner:** CDO · **Slice plan:** Stages 1–5 (this phase covers 1–2; Stage 3+ defers to P2)

#### Stage 1 — Personal memory universal protocol (any folder, portable)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MEMORY-101 | `cyberos memory init <path>` — any-folder bootstrap (manifest, audit/, memories/, HEAD) | MUST | ready_to_implement | — | 6h |
| FR-MEMORY-102 | `cyberos memory watch / unwatch / status` — multi-folder watcher registry | MUST | ready_to_implement | FR-MEMORY-101 | 6h |
| FR-MEMORY-103 | Privacy-floor flag in manifest (`sync_class_default: private`) + per-memory override | MUST | ready_to_implement | FR-MEMORY-101 | 4h |
| FR-MEMORY-104 | Portable folder-copy round-trip (export to zip + import on another machine) | MUST | ready_to_implement | FR-MEMORY-101 | 6h |
| FR-MEMORY-105 | Doctor invariants extended (+2 new for watched-folders integrity) | MUST | ready_to_implement | FR-MEMORY-101 | 4h |
| FR-MEMORY-106 | `watched_folders` schema migration + idempotent re-watch on restart | MUST | ready_to_implement | FR-MEMORY-102 | 4h |

#### Stage 2 — Capture daemon (FS watcher + Cowork hook + Claude Code hook)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-MEMORY-107 | FS watcher via Rust + `notify` crate; rate-limited + content-deduped | MUST | ready_to_implement | FR-MEMORY-102 | 12h |
| FR-MEMORY-108 | Cowork session-hook capture (Claude Code Cowork mode emits memories) | MUST | ready_to_implement | FR-MEMORY-107 | 6h |
| FR-MEMORY-109 | Claude Code hook capture (CLI / IDE plugin emits per-prompt memories) | MUST | ready_to_implement | FR-MEMORY-107 | 5h |
| FR-MEMORY-110 | Capture daemon health check + restart on crash (systemd / launchd unit) | MUST | ready_to_implement | FR-MEMORY-107 | 4h |
| FR-MEMORY-111 | Pre-ingest PII detection (Presidio EN + VN); held-back rate ≥ 99.5% | MUST | ready_to_implement | FR-AI-012 | 6h |

---

### P1.2 — SKILL · Phase 8 (memory integration) + vertical packs

**Module page:** [`skill.html`](../../website/docs/modules/skill.html) · **Owner:** CPO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — capability broker + memory-aware SKILL.md

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-101 | SKILL.md frontmatter extension: `allowed_memory_scopes` + `allowed_tools` enforced by broker | MUST | ready_to_implement | FR-MEMORY-101 | 5h |
| FR-SKILL-102 | Capability broker: subprocess sandbox enforces `allowed_tools` at invoke time | MUST | ready_to_implement | FR-MCP-006 | 8h |
| FR-SKILL-103 | Pre + post audit rows on every skill invocation (memory Writer dual-write) | MUST | ready_to_implement | FR-SKILL-102 | 5h |

#### Slice 2 — universal-protocol skill bundles

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-104 | `memory-capture@1` skill bundle (canonical capture entry point) | MUST | ready_to_implement | FR-MEMORY-107 | 6h |
| FR-SKILL-105 | `memory-sync@1` skill bundle (defers to Stage 4 sync orchestrator at P2) | SHOULD | ready_to_implement | FR-SKILL-104 | 4h |
| FR-SKILL-106 | `synthesis-author@1` skill (multi-memory auto-evolve; runs nightly at P3) | COULD | ready_to_implement | FR-SKILL-105 | 8h |

#### Slice 3 — vertical pack scaffolding (cyberskill-vn first)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-107 | `cyberskill-vn` pack — `vietnam-mst-validate` skill against GDT API | MUST | ready_to_implement | FR-SKILL-102 | 6h |
| FR-SKILL-108 | `cyberskill-vn` pack — `vietnam-bank-transfer` skill (VietQR / Napas247 code generator) | MUST | ready_to_implement | FR-SKILL-102 | 6h |
| FR-SKILL-109 | `cyberskill-vn` pack — `vietnam-vat-invoice` skill (hóa đơn Decree 123 XML emitter) | MUST | ready_to_implement | FR-SKILL-102 | 10h |

---

### P1.3 — CUO · Phase 2 LLM cascade + Phase 3 multi-skill chains

**Module page:** [`cuo.html`](../../website/docs/modules/cuo.html) · **Owner:** CPO · **Slice plan:** 2 slices (P1) + Phases 3–4 deferred to P2/P3

#### Phase 2 — LangGraph + LiteLLM cascade

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CUO-101 | LangGraph supervisor wired to LiteLLM router (escalate when 0.10 ≤ conf ≤ 0.50) | MUST | ready_to_implement | FR-AI-008 | 12h |
| FR-CUO-102 | Postgres checkpointer for LangGraph state (EU AI Act Art. 12 logging) | MUST | ready_to_implement | FR-CUO-101 | 5h |
| FR-CUO-103 | Phase 2 trace rows include prompt + model + temperature + seed for replay | MUST | ready_to_implement | FR-CUO-102 | 4h |

#### Phase 3 — multi-skill chains via `depends_on`

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CUO-104 | Topological walk of `depends_on` chain with composite audit row + sub-rows | MUST | ready_to_implement | FR-CUO-101 | 10h |
| FR-CUO-105 | Per-step rollback on chain failure; partial-execution audit preserved | MUST | ready_to_implement | FR-CUO-104 | 6h |

---

### P1.4 — PROJ · orchestration spine

**Module page:** [`proj.html`](../../website/docs/modules/proj.html) · **Owner:** COO/CPO · **Slice plan:** 5 slices, 18 FRs

#### Slice 1 — four primitives + sync engine

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-001 | Issue / Cycle / Project / Engagement Postgres schema with `engagement_id` FK | MUST | ready_to_implement | FR-AUTH-003 | 8h |
| FR-PROJ-002 | WebSocket sync engine (axum + NATS JetStream) with optimistic client apply + server rebase | MUST | ready_to_implement | FR-PROJ-001 | 16h |
| FR-PROJ-003 | Yjs CRDT for description + comment-body fields; LWW for scalars | MUST | ready_to_implement | FR-PROJ-002 | 10h |
| FR-PROJ-004 | Issue lifecycle state machine (backlog → todo → in-progress → in-review → done / cancelled) | MUST | ready_to_implement | FR-PROJ-001 | 5h |

#### Slice 2 — Engagement economics + billable rules

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-005 | Rate-card schema per Engagement (role × currency × hourly rate × billable_default) | MUST | ready_to_implement | FR-PROJ-001 | 4h |
| FR-PROJ-006 | Billable cascade: Member override → task class → role default → fallback | MUST | ready_to_implement | FR-PROJ-005 | 6h |
| FR-PROJ-007 | Three billing modes (T&M / fixed-fee / retainer) with mode-aware rollup | MUST | ready_to_implement | FR-PROJ-005 | 6h |

#### Slice 3 — memory integration + cross-module join contracts

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-008 | memory audit row per Issue mutation (chained to PROJ `history_event` table) | MUST | ready_to_implement | FR-MEMORY-101 | 5h |
| FR-PROJ-009 | `MEMORY_LINK` schema: Issue cites memory via (cites / implements / supersedes) | MUST | ready_to_implement | FR-PROJ-001 | 5h |
| FR-PROJ-010 | Citation-drift detector (nightly sweep flags stale citations) | SHOULD | ready_to_implement | FR-PROJ-009 | 4h |

#### Slice 4 — AI features (blocker detection + cycle review + calibration)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-011 | Blocker detector from comment stream (`blocked by` + dwell time → CUO Notify) | MUST | ready_to_implement | FR-CUO-101 | 6h |
| FR-PROJ-012 | Cycle-review draft generator (CUO/COO persona) at cycle close | MUST | ready_to_implement | FR-CUO-101 | 8h |
| FR-PROJ-013 | Estimate calibration snapshot (per Member per task class, nightly batch) | MUST | ready_to_implement | FR-PROJ-002 | 6h |

#### Slice 5 — UI surfaces (Board · Timeline · Gantt · Brief)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PROJ-014 | Kanban Board (drag + drop, keyboard-first) | MUST | ready_to_implement | FR-PROJ-002 | 12h |
| FR-PROJ-015 | Timeline view (cycle window × assignee) | MUST | ready_to_implement | FR-PROJ-002 | 10h |
| FR-PROJ-016 | Gantt view with dependency arrows | SHOULD | ready_to_implement | FR-PROJ-002 | 12h |
| FR-PROJ-017 | Brief modal (issue deep-view with Yjs description + comments + meta sidebar) | MUST | ready_to_implement | FR-PROJ-003 | 8h |
| FR-PROJ-018 | Liquid-Glass design tokens (`tokens.proj.css`) + axe-core CI accessibility gate | MUST | ready_to_implement | FR-PROJ-014 | 6h |

---

### P1.5 — CRM · sales-pipeline spine

**Module page:** [`crm.html`](../../website/docs/modules/crm.html) · **Owner:** CRO/CPO · **Slice plan:** 3 slices, 10 FRs

#### Slice 1 — three primitives + pipelines

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CRM-001 | Account / Contact / Deal Postgres schema with custom pipelines + stages | MUST | ready_to_implement | FR-AUTH-003 | 6h |
| FR-CRM-002 | Activity feed auto-log from EMAIL/CHAT/Calendar via tracked-domain match | MUST | ready_to_implement | FR-CRM-001 | 8h |
| FR-CRM-003 | VN-specific: account type → legal entity (Sole / LLC / JSC / FDI) + MST field | MUST | ready_to_implement | FR-CRM-001 | 4h |

#### Slice 2 — Deal → Engagement bridge + AI features

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CRM-004 | Convert-to-Engagement workflow (deal.won → PROJ Engagement with rate card) | MUST | ready_to_implement | FR-CRM-001, FR-PROJ-005 | 6h |
| FR-CRM-005 | CUO `crm.next-action@1` skill — top-3 ranked moves per open deal | MUST | ready_to_implement | FR-CUO-101 | 6h |
| FR-CRM-006 | AI lead scoring at Contact creation + nightly refresh | SHOULD | ready_to_implement | FR-CUO-101 | 5h |
| FR-CRM-007 | Win/loss analysis CUO draft at deal close; becomes memory memory | SHOULD | ready_to_implement | FR-CUO-101 | 5h |

#### Slice 3 — VN integrations

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-CRM-008 | MST validation via `vietnam-mst-validate` skill on Account write | MUST | ready_to_implement | FR-SKILL-107 | 3h |
| FR-CRM-009 | VietQR generation via `vietnam-bank-transfer` skill on Deal collection | MUST | ready_to_implement | FR-SKILL-108 | 4h |
| FR-CRM-010 | Hóa đơn auto-emit via `vietnam-vat-invoice` on deal.stage=won | MUST | ready_to_implement | FR-SKILL-109 | 5h |

---

### P1.6 — TIME · billable-hours engine

**Module page:** [`time.html`](../../website/docs/modules/time.html) · **Owner:** CFO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — TimeEntry primitive + 3 input modes

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TIME-001 | TimeEntry append-only schema with `correction_to` link semantics | MUST | ready_to_implement | FR-AUTH-003 | 5h |
| FR-TIME-002 | Timer start/stop UI in SPA | MUST | ready_to_implement | FR-TIME-001 | 5h |
| FR-TIME-003 | Manual entry form (retroactive logging) with VN Labour Code cap validation | MUST | ready_to_implement | FR-TIME-001 | 6h |
| FR-TIME-004 | Auto-detect proposals from PROJ activity (status changes + comment patterns; Member-confirm) | SHOULD | ready_to_implement | FR-PROJ-002 | 6h |

#### Slice 2 — billable cascade + approval flow

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TIME-005 | Billable flag computation via 4-step cascade (snapshot on row) | MUST | ready_to_implement | FR-TIME-001, FR-PROJ-006 | 5h |
| FR-TIME-006 | Weekly approval flow (Member submit → AM review → CFO visibility) | MUST | ready_to_implement | FR-TIME-001 | 6h |
| FR-TIME-007 | VN Labour Code Art. 107 OT cap hard-block at entry write | MUST | ready_to_implement | FR-TIME-001 | 4h |

#### Slice 3 — receipt OCR + PROJ-INV bridge

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TIME-008 | Expense capture: photo → AWS Textract OCR → hóa đơn parse → Member confirm | MUST | ready_to_implement | FR-CRM-010 | 8h |
| FR-TIME-009 | Per-cycle billable rollup emit to INV (per-Member × role × Engagement) | MUST | ready_to_implement | FR-TIME-005 | 6h |

---

### P1.7 — KB · RAG corpus + memory companion

**Module page:** [`kb.html`](../../website/docs/modules/kb.html) · **Owner:** CDO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — markdown source + versioning + render

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-KB-001 | Document schema (slug + markdown body + YAML frontmatter + category + ACL) + immutable versions | MUST | ready_to_implement | FR-AUTH-003 | 6h |
| FR-KB-002 | Server-side renderer: markdown → sanitised HTML (ammonia) + sanitised plaintext for memory | MUST | ready_to_implement | FR-KB-001 | 5h |
| FR-KB-003 | Three permission tiers: public · org-only · role-restricted with share-link tokens | MUST | ready_to_implement | FR-KB-001 | 5h |

#### Slice 2 — three-layer search + AI Q&A

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-KB-004 | FTS5 + PGroonga lexical search with VN bigram tokenisation | MUST | ready_to_implement | FR-KB-001 | 6h |
| FR-KB-005 | BGE-M3 semantic search via memory Layer 2 ingest | MUST | ready_to_implement | FR-AI-019, FR-KB-001 | 6h |
| FR-KB-006 | BGE-rerank-v2-m3 cross-encoder reranker over top-K from layers 1+2 | MUST | ready_to_implement | FR-AI-020, FR-KB-005 | 4h |
| FR-KB-007 | "Ask this page" Q&A grounded in current + linked docs with span-level citations | MUST | ready_to_implement | FR-KB-006, FR-CUO-101 | 8h |

#### Slice 3 — runbook catalogue + dual-language

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-KB-008 | Runbook category with applicability tags (provider / region / severity) for OBS triage | MUST | ready_to_implement | FR-KB-001, FR-OBS-007 | 5h |
| FR-KB-009 | Dual-language `translation_of` link + locale-aware reader display (vi/en) | SHOULD | ready_to_implement | FR-KB-001 | 4h |

---

### P1.8 — EMAIL · capture surface + Genie draft

**Module page:** [`email.html`](../../website/docs/modules/email.html) · **Owner:** CCO/CPO · **Slice plan:** 3 slices, 11 FRs

#### Slice 1 — Stalwart core + shared inbox

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-EMAIL-001 | Stalwart Rust mail server deployed (JMAP/IMAP/SMTP/ManageSieve all transports) | MUST | ready_to_implement | — | 12h |
| FR-EMAIL-002 | `cyberos-email-authbridge` plugin — Stalwart JMAP auth delegates to AUTH JWT | MUST | ready_to_implement | FR-EMAIL-001, FR-AUTH-004 | 6h |
| FR-EMAIL-003 | Missive-style shared-inbox UX (assignment, internal comments, snooze, tag) | MUST | ready_to_implement | FR-EMAIL-001 | 16h |
| FR-EMAIL-004 | DKIM signing + ARC chain forward + BIMI brand indicator | MUST | ready_to_implement | FR-EMAIL-001 | 6h |

#### Slice 2 — CaMeL quarantine + capture

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-EMAIL-005 | CaMeL dual-LLM: quarantine LLM (no tools, no memory) extracts → privileged CUO consumes only sanitised | MUST | ready_to_implement | FR-CUO-101 | 12h |
| FR-EMAIL-006 | Tracked-domain auto-log to CRM activity feed (per-tenant tracked-domain config) | MUST | ready_to_implement | FR-CRM-002, FR-EMAIL-001 | 5h |
| FR-EMAIL-007 | "Convert to issue" — thread → PROJ Issue with body as description, replies as comments | MUST | ready_to_implement | FR-PROJ-001, FR-EMAIL-001 | 6h |

#### Slice 3 — Genie draft + bulk send

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-EMAIL-008 | "Genie:" subject prefix → CUO draft grounded in thread + CRM + memory + KB (sync_class permitting) | MUST | ready_to_implement | FR-CUO-101, FR-KB-007 | 8h |
| FR-EMAIL-009 | Outbound 1:1 send (DKIM-signed, AM confirms) | MUST | ready_to_implement | FR-EMAIL-004 | 4h |
| FR-EMAIL-010 | Bulk send (≥ 10 recipients) requires AM + CFO/marketing approval token | MUST | ready_to_implement | FR-EMAIL-009 | 5h |
| FR-EMAIL-011 | DSAR export — every message a subject authored + chained memory audit hashes | MUST | ready_to_implement | FR-EMAIL-001 | 5h |

---

### P1.9 — HR · Member lifecycle + onboarding orchestrator

**Module page:** [`hr.html`](../../website/docs/modules/hr.html) · **Owner:** CHRO · **Slice plan:** 3 slices, 9 FRs

#### Slice 1 — Member directory + contract types

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-HR-001 | Member schema (profile + role + level + contract type + leave balance + sabbatical accrual) | MUST | ready_to_implement | FR-AUTH-003 | 6h |
| FR-HR-002 | 5 contract types: indefinite · fixed-term · probation · part-time · contractor | MUST | ready_to_implement | FR-HR-001 | 4h |
| FR-HR-003 | CCCD photo separate KMS keyspace + sev-1 access audit | MUST | ready_to_implement | FR-HR-001 | 5h |

#### Slice 2 — leave + statutory caps

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-HR-004 | 8 leave types (annual / sick / maternity / paternity / sabbatical / unpaid / bereavement / public-holiday) | MUST | ready_to_implement | FR-HR-001 | 5h |
| FR-HR-005 | Decree 145/2020 working-hour caps + Decree 152/2020 SI rates (version-pinned) | MUST | ready_to_implement | FR-HR-001 | 4h |
| FR-HR-006 | Annual-leave accrual nightly batch (Decree 145 formula) | MUST | ready_to_implement | FR-HR-004 | 4h |

#### Slice 3 — onboarding orchestrator + performance signals

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-HR-007 | Onboarding saga — fires to AUTH / TIME / LEARN / KB / CHAT / REW on `member.active` transition | MUST | ready_to_implement | FR-HR-001 | 10h |
| FR-HR-008 | Performance signal aggregator (read-only consumer of PROJ + TIME + LEARN signals) | MUST | ready_to_implement | FR-PROJ-013, FR-TIME-001 | 6h |
| FR-HR-009 | Termination workflow with GL/BL branch (CFO + CEO co-sign required) | MUST | ready_to_implement | FR-HR-001 | 8h |

---

### P1 Exit gate criteria

- All 9 P1 modules ship internally and are used daily
- DSAR fulfilment p95 ≤ 24 h across all P1 modules (PDPL Art. 14)
- EU AI Act Art. 12 audit trail covers every CUO Phase 2 + skill invocation
- 16 P1 modules total live (P0's 5 + P1's 9 + 2 already-shipped: SKILL Phase 8, CUO Phase 2)
- memory auto-sync Stages 1 + 2 demonstrably running on every Member's laptop
- Cycle-review draft acceptance rate ≥ 60% across all Engagements (FR-PROJ-012)

---

## §4 — P2 · Operations

**Phase goal:** revenue + financial ops solidified. P2 ships INV, REW, ESOP, the TEN billing thin slice, and LEARN. Invoices issue from PROJ-TIME rollups. Payroll runs through REW deterministically. Vertical-pack pricing becomes possible. LEARN's promotion workflow (Hội đồng Chuyên môn) anchors the career path.

**Compliance gate:** Vietnamese hóa đơn Decree 123 emission validated end-to-end · PCI SAQ-A self-assessment passed · ISO 27017 cloud-services controls signed off.

### P2.1 — INV · billable rollup invoicing

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-INV-001 | Invoice draft from TIME per-cycle rollup with rate-card snapshot preservation | MUST | ready_to_implement | FR-TIME-009 | 8h |
| FR-INV-002 | Multi-currency support (VND / USD / SGD / EUR) with daily SBV FX snapshot | MUST | ready_to_implement | FR-INV-001 | 6h |
| FR-INV-003 | Stripe webhook handler (signature-verified, idempotent) | MUST | ready_to_implement | — | 8h |
| FR-INV-004 | Wise webhook handler for multi-currency receipts | SHOULD | ready_to_implement | — | 6h |
| FR-INV-005 | VietQR/Napas247 webhook handler for VND domestic | MUST | ready_to_implement | — | 6h |
| FR-INV-006 | Cash application — match incoming receipts to outstanding invoices (amount + reference) | MUST | ready_to_implement | FR-INV-003, FR-INV-005 | 8h |
| FR-INV-007 | `vietnam-vat-invoice` hóa đơn auto-emit on AM-send for VN tenants (Decree 123 GDT XML) | MUST | ready_to_implement | FR-INV-001, FR-SKILL-109 | 6h |
| FR-INV-008 | Hóa đơn cancellation (Decree 123 Art. 19) with dual approval (AM + CFO) | MUST | ready_to_implement | FR-INV-007 | 5h |
| FR-INV-009 | AR aging report (nightly) + 90+ rolling alert | MUST | ready_to_implement | FR-INV-001 | 4h |
| FR-INV-010 | CUO dunning draft on overdue (30/60/90); never auto-sent | MUST | ready_to_implement | FR-CUO-101 | 5h |
| FR-INV-011 | Revenue recognition to GL (accrual or cash basis per tenant policy) | MUST | ready_to_implement | FR-INV-001 | 5h |

### P2.2 — REW · compensation engine

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-REW-001 | 3P income schema (P1 Base / P2 Allowance / P3 Performance) — encrypted comp keyspace separate from HR | MUST | ready_to_implement | FR-HR-001 | 6h |
| FR-REW-002 | Parameter versioning (immutable; replay-equivalence ≥ 100% on prior payslips) | MUST | ready_to_implement | FR-REW-001 | 6h |
| FR-REW-003 | P1 protection invariant — DB CHECK constraint forbids any P1 cash reduction | MUST | ready_to_implement | FR-REW-001 | 4h |
| FR-REW-004 | Statutory deductions: BHXH 10.5% + BHYT 1.5% + BHTN 1% + PIT progressive (Decree 152/2020) | MUST | ready_to_implement | FR-HR-005 | 6h |
| FR-REW-005 | Monthly payroll compute + CFO+CHRO co-sign commit gate | MUST | ready_to_implement | FR-REW-001 | 8h |
| FR-REW-006 | Byte-identical PDF payslip render (Tectonic + pinned fonts) | MUST | ready_to_implement | FR-REW-005 | 6h |
| FR-REW-007 | BP (Bonus Points) ledger with ACB-rate interest accrual nightly | MUST | ready_to_implement | FR-REW-001 | 5h |
| FR-REW-008 | Quarterly P3 distribution from BP fund (CEO+CFO sign-off) | MUST | ready_to_implement | FR-REW-007 | 6h |
| FR-REW-009 | VietQR bank payroll batch send (manual CFO confirm at submission) | MUST | ready_to_implement | FR-INV-005 | 5h |
| FR-REW-010 | memory structural exclusion CI gate (no comp fields in memory-ingest paths) | MUST | ready_to_implement | FR-REW-001 | 3h |

### P2.3 — ESOP · Phantom Stock

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-ESOP-001 | SP grant schema with vesting params (4-year + 12-month cliff default) | MUST | ready_to_implement | FR-HR-001 | 5h |
| FR-ESOP-002 | Monthly vesting accrual deterministic batch | MUST | ready_to_implement | FR-ESOP-001 | 4h |
| FR-ESOP-003 | Annual valuation (CFO base + Board multiplier sign-off) immutable rows | MUST | ready_to_implement | FR-ESOP-001 | 5h |
| FR-ESOP-004 | Put-option exec flow (Year 3+, per-Member cap, CFO approve, wire) | MUST | ready_to_implement | FR-ESOP-003, FR-INV-005 | 8h |
| FR-ESOP-005 | Good/Bad Leaver branch on HR offboarding (CFO + CEO co-sign) | MUST | ready_to_implement | FR-HR-009 | 5h |
| FR-ESOP-006 | M&A acceleration trigger + Member notice within 5 business days | SHOULD | ready_to_implement | FR-ESOP-001 | 5h |
| FR-ESOP-007 | Member ESOP dashboard (personal view only; cross-Member requires CFO audit) | SHOULD | ready_to_implement | FR-ESOP-001 | 6h |

### P2.4 — TEN · billing thin slice (per research review §7.3)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TEN-001 | Tenant provisioning CLI (`cyberos-ten provision`) for ops-driven flow | MUST | ready_to_implement | FR-AUTH-001 | 5h |
| FR-TEN-002 | 3 plan tiers (Starter/Team/Enterprise) hardcoded | MUST | ready_to_implement | FR-TEN-001 | 4h |
| FR-TEN-003 | Stripe billing integration (USD/EUR/SGD invoicing for international tenants) | MUST | ready_to_implement | FR-INV-003 | 8h |
| FR-TEN-004 | 4-axis metering: seats · API · AI tokens · storage (memory audit emission per metric event) | MUST | ready_to_implement | FR-AI-001 | 8h |
| FR-TEN-005 | Vertical-pack pricing add-on (per-pack monthly fee, not per-seat) | MUST | ready_to_implement | FR-TEN-002, FR-SKILL-107 | 5h |

### P2.5 — LEARN · skills catalogue + VP + Council

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-LEARN-001 | Skill tree schema + 1-5 mastery levels per skill per Member | MUST | ready_to_implement | FR-HR-001 | 6h |
| FR-LEARN-002 | Bằng cấp + chứng chỉ (degrees + certifications) evidence types | MUST | ready_to_implement | FR-LEARN-001 | 4h |
| FR-LEARN-003 | VP (Voting Power) deterministic nightly roll-up (PROJ + TIME + KB inputs) | MUST | ready_to_implement | FR-PROJ-013, FR-TIME-001 | 6h |
| FR-LEARN-004 | Hội đồng Chuyên môn (Specialist Council) workflow — 3-5 judges + multi-dim scoring | MUST | ready_to_implement | FR-LEARN-001 | 10h |
| FR-LEARN-005 | Per-judge score isolation (NEVER exit LEARN boundary; HR receives summary + recommendation only) | MUST | ready_to_implement | FR-LEARN-004 | 5h |
| FR-LEARN-006 | Promotion approval workflow (CEO + CHRO sign-off after council vote) | MUST | ready_to_implement | FR-LEARN-004 | 5h |
| FR-LEARN-007 | VP score → REW BP fund distribution handoff at quarter close | MUST | ready_to_implement | FR-LEARN-003, FR-REW-008 | 4h |

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
| FR-AUTH-101 | 22-role RBAC catalogue (full bands: root-admin → tenant-member + 17 specialist roles) | MUST | done | FR-AUTH-005 | 12h |
| FR-AUTH-102 | TOTP + WebAuthn MFA flows | MUST | done | FR-AUTH-002 | 10h |
| FR-AUTH-103 | SAML 2.0 SSO (per-tenant IdP config) | MUST | implementing | FR-AUTH-004 | 12h |
| FR-AUTH-104 | OIDC SSO with discovery + JWKS rotation | MUST | ready_to_implement | FR-AUTH-004 | 10h |
| FR-AUTH-105 | Passkey enrolment + login | MUST | ready_to_implement | FR-AUTH-102 | 8h |
| FR-AUTH-106 | Impossible-travel detection + adaptive challenge | SHOULD | implementing | FR-AUTH-002 | 8h |
| FR-AUTH-107 | HIBP password breach check on signup + rotation | SHOULD | ready_to_implement | FR-AUTH-002 | 4h |
| FR-AUTH-108 | Lumi tenant-identity JWT shape (`agent_persona` + `tenant_residency` claims) | MUST | ready_to_implement | FR-AUTH-101 | 6h |
| FR-AUTH-109 | Stub → full migration path (existing tokens valid for grace window) | MUST | ready_to_implement | FR-AUTH-101 | 5h |

### P3.2 — TEN (full self-serve)

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-TEN-101 | Self-serve signup form ≤ 30 s end-to-end | MUST | ready_to_implement | FR-AUTH-104 | 10h |
| FR-TEN-102 | VnPay + Momo + ZaloPay billing rails for VND domestic | MUST | ready_to_implement | FR-TEN-003 | 12h |
| FR-TEN-103 | 4-residency provisioning (sg-1 / eu-1 / us-1 / vn-1) | MUST | ready_to_implement | FR-AI-016 | 10h |
| FR-TEN-104 | 90-day offboarding contract (Active → Terminating-A → Terminating-B → Terminated) | MUST | ready_to_implement | FR-TEN-001 | 12h |
| FR-TEN-105 | Signed-bundle export (deterministic zip, Ed25519 signature, memory audit anchor) | MUST | ready_to_implement | FR-TEN-104 | 8h |
| FR-TEN-106 | Permanent-delete attestation row (CSO + CLO sign-off + chained audit) | MUST | ready_to_implement | FR-TEN-104 | 5h |
| FR-TEN-107 | Tenant-admin SPA (seats / billing / audit / residency / retention) | SHOULD | ready_to_implement | FR-TEN-101 | 16h |

### P3.3 — OKR · strategy cascade

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-OKR-001 | Objective × KR schema + Company → Team → Member cascade | MUST | ready_to_implement | FR-AUTH-003 | 6h |
| FR-OKR-002 | 3 KR types (hit-target / improvement / milestone) | MUST | ready_to_implement | FR-OKR-001 | 4h |
| FR-OKR-003 | KR `progress_source` DSL — query against PROJ / INV / HR / LEARN | MUST | ready_to_implement | FR-OKR-001 | 10h |
| FR-OKR-004 | Auto-progress nightly batch | MUST | ready_to_implement | FR-OKR-003 | 5h |
| FR-OKR-005 | Weekly check-in (1-10 confidence + rationale) | MUST | ready_to_implement | FR-OKR-001 | 5h |
| FR-OKR-006 | Monday-morning CUO digest (auto-progress + check-ins → founder summary) | MUST | ready_to_implement | FR-CUO-101, FR-OKR-005 | 6h |
| FR-OKR-007 | Quarterly retro draft with face-saving Vietnamese framing | SHOULD | ready_to_implement | FR-CUO-101, FR-OKR-001 | 6h |

### P3.4 — RES · capacity-vs-forecast + hiring forecast

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-RES-001 | Capacity-vs-demand matrix (HR × PROJ × TIME × LEARN joins, nightly batch) | MUST | ready_to_implement | FR-HR-001, FR-PROJ-001, FR-TIME-001 | 10h |
| FR-RES-002 | Allocation Gantt UI + drag-rebalance | MUST | ready_to_implement | FR-RES-001 | 12h |
| FR-RES-003 | Over/under-allocation flags (110% / 60% thresholds) | MUST | ready_to_implement | FR-RES-001 | 4h |
| FR-RES-004 | Hiring memo CUO draft (skill-gap × CRM pipeline → hire trigger) | MUST | ready_to_implement | FR-CUO-101, FR-CRM-001 | 8h |
| FR-RES-005 | VN Labour Code Art. 107 OT cap hard-block at allocation propose | MUST | ready_to_implement | FR-HR-005 | 4h |

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
| FR-DOC-001 | Document repository (versioned, ACL'd, 10-year retention) S3 Object-Lock Compliance bucket | MUST | ready_to_implement | FR-AUTH-101 | 8h |
| FR-DOC-002 | eIDAS QTSP partner integration (GlobalSign or Cryptomathic) for EU residency | MUST | ready_to_implement | FR-DOC-001 | 16h |
| FR-DOC-003 | AATL CA partner integration (Adobe-AATL listed) for US/non-EU | MUST | ready_to_implement | FR-DOC-001 | 12h |
| FR-DOC-004 | VNeID + VN CA chain (VnPay/MK Group/Viettel-CA) for VN tenants | MUST | ready_to_implement | FR-DOC-001 | 16h |
| FR-DOC-005 | Multi-party signing workflow (ordered + parallel + counter-sign) | MUST | ready_to_implement | FR-DOC-001 | 10h |
| FR-DOC-006 | Identity verification — WebAuthn / VNeID / SMS-OTP / email-link 4 methods | MUST | ready_to_implement | FR-AUTH-105 | 8h |
| FR-DOC-007 | Lifecycle metadata (parties / dates / renewal / expiry / parent contract) | MUST | ready_to_implement | FR-DOC-001 | 5h |
| FR-DOC-008 | Expiry alert cascade (90 / 30 / 7 days) | MUST | ready_to_implement | FR-DOC-007 | 4h |
| FR-DOC-009 | Renewal proposal CUO draft + AM approval | SHOULD | ready_to_implement | FR-CUO-101, FR-DOC-007 | 6h |
| FR-DOC-010 | DocuSign / Adobe Sign / HelloSign import (LTV preservation) | SHOULD | ready_to_implement | FR-DOC-001 | 10h |
| FR-DOC-011 | PAdES-B-LT format with year-9 LTV re-stamping | MUST | ready_to_implement | FR-DOC-002 | 8h |

### P4.2 — PORTAL · client-facing scoped views

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-PORTAL-001 | Scoped read-only view layer (PROJ / INV / DOC / CHAT) filtered by Engagement membership + sync_class=client-visible | MUST | ready_to_implement | FR-TEN-101 | 12h |
| FR-PORTAL-002 | Per-tenant brand pack (logo + colours + custom CNAME + email template overrides) | MUST | ready_to_implement | FR-TEN-101 | 8h |
| FR-PORTAL-003 | External IdP SAML 2.0 + OIDC support with JIT user provisioning | MUST | ready_to_implement | FR-AUTH-103, FR-AUTH-104 | 10h |
| FR-PORTAL-004 | SCIM 2.0 deprovision (session invalidation ≤ 30 s on IdP removal) | MUST | ready_to_implement | FR-PORTAL-003 | 8h |
| FR-PORTAL-005 | Branded Genie chat (CUO scope-narrowed by JWT scope_grants) | SHOULD | ready_to_implement | FR-PORTAL-003, FR-CUO-101 | 6h |
| FR-PORTAL-006 | Client-initiated workflows — new project request / billing inquiry / support ticket → CHAT thread | MUST | ready_to_implement | FR-CHAT-005 | 6h |
| FR-PORTAL-007 | PWA installable (mobile-first) | SHOULD | ready_to_implement | FR-PORTAL-001 | 6h |
| FR-PORTAL-008 | DSAR self-service (client requests their own data) | MUST | ready_to_implement | FR-PORTAL-001 | 5h |

### P4.3 — vertical-pack marketplace + HoldCo flip path

| FR-ID | Title | Pri | Status | Depends on | Effort |
|---|---|:-:|:-:|---|---:|
| FR-SKILL-201 | OCI registry deploy for `.skill` bundles (R3 distribution stage) | MUST | ready_to_implement | FR-SKILL-102 | 8h |
| FR-SKILL-202 | `cyberskill-sg` pack (Singapore: ACRA filings + GST e-invoice + CPF) | SHOULD | ready_to_implement | FR-SKILL-107 | 16h |
| FR-SKILL-203 | `cyberskill-id` pack (Indonesia: NPWP + e-Faktur) | COULD | ready_to_implement | FR-SKILL-107 | 16h |
| FR-TEN-201 | Singapore HoldCo flip CLI (`cyberos-ten holdco-flip`) ACRA filings | MUST | ready_to_implement | FR-ESOP-001 | 16h |
| FR-TEN-202 | Hostile-termination override (legal-trigger fast-track with CEO+CLO+CSO sign-off) | SHOULD | ready_to_implement | FR-TEN-104 | 5h |
| FR-TEN-203 | Margin watchdog for fixed-fee engagements (alarm < 30% projected) | SHOULD | ready_to_implement | FR-PROJ-007 | 5h |

### P4 Exit gate criteria

- All 22 modules live
- First external paying tenant on full multi-tenant SaaS (not pilot)
- Singapore HoldCo flip path tested end-to-end in sandbox (no real flip required for gate)
- eIDAS-conformant signature emitted + verified
- ≥ 5 external paying tenants by P4 · late
- ≥ 2 vertical packs (cyberskill-vn + one of sg/id/th) generating ≥ 30% of ARR

---

*End of backlog narrative — v0.5.1, 2026-05-20 (relocated backlog rules and invariants to feature-request-audit skill).*
*Status:* spec corpus closed; implementation phase begins with MEMORY → AUTH → CHAT → PROJECT → CUO+SKILL per §0.6. Appendices §A–§D below contain the four generated reports absorbed from former `REPORTS.md`.

---

# Appendix §A — Contract verification report

_Generated 2026-05-17 — 241 FRs scanned. Was §1 of the former `REPORTS.md`. Refresh by re-running the regen scripts in `modules/skill/project-cleanup/scripts/`._

## §1 source detail — Contract verification (full)



_Generated 2026-05-17 — 241 FRs scanned._

## Summary

- Endpoints declared (in §3 API contract): **261**
- Endpoint references across all FR text: **595**
- Endpoints with multiple declarers (potential conflict): **0**
- Orphan endpoint references (not declared anywhere): **333**

## Endpoints declared (per FR)

- **FR-AUTH-105**: POST /v1/auth/mfa/recovery-codes/regen
- **FR-CRM-004**: POST /v1/crm/deals/{}/convert-to-engagement
- **FR-CRM-005**: POST /v1/crm/next-action, POST /v1/crm/next-action/{}/dismiss, POST /v1/crm/next-action/{}/execute
- **FR-CRM-006**: GET /v1/crm/contacts/{}/score-history, GET /v1/crm/scoring/weights, POST /v1/crm/contacts/{}/rescore, PUT /v1/crm/scoring/weights
- **FR-CRM-007**: GET /v1/crm/win-loss/drafts, POST /v1/crm/win-loss/drafts/{}/approve, POST /v1/crm/win-loss/drafts/{}/dismiss
- **FR-CRM-008**: GET /v1/crm/accounts/{}/mst-validation, POST /v1/crm/accounts/{}/validate-mst
- **FR-CRM-009**: GET /v1/crm/bank-config, POST /v1/crm/skill/vn-bank-transfer, PUT /v1/crm/bank-config
- **FR-CRM-010**: GET /v1/crm/skill/vn-vat-invoice/emissions/{}, POST /v1/crm/skill/vn-vat-invoice
- **FR-CUO-104**: POST /v1/cuo/chains
- **FR-DOC-002**: GET /v1/doc/qtsp/signatures/{}, POST /v1/doc/qtsp/sign, PUT /v1/doc/qtsp/creds
- **FR-DOC-004**: GET /v1/doc/vn-ca/signatures/{}, POST /v1/doc/vn-ca/sign, POST /v1/doc/vn-ca/vneid-link, PUT /v1/doc/vn-ca/creds
- **FR-DOC-005**: GET /v1/doc/signing-workflows/{}, POST /v1/doc/documents/{}/signing-workflows, POST /v1/doc/signers/{}/decline, POST /v1/doc/signers/{}/sign, POST /v1/doc/signing-workflows/{}/withdraw
- **FR-DOC-006**: GET /v1/doc/documents/{}/verifications, POST /v1/doc/documents/{}/verify/complete, POST /v1/doc/documents/{}/verify/start
- **FR-DOC-008**: DELETE /v1/doc/documents/{}/snooze-alerts, GET /v1/doc/expiry-alerts, POST /v1/doc/documents/{}/snooze-alerts, POST /v1/doc/expiry-scan
- **FR-DOC-010**: GET /v1/doc/import/jobs/{}, POST /v1/doc/import/{}/start, PUT /v1/doc/third-party-creds
- **FR-DOC-011**: GET /v1/doc/documents/{}/ltv/operations, POST /v1/doc/documents/{}/ltv/extend, POST /v1/doc/documents/{}/ltv/restamp
- **FR-EMAIL-002**: POST /v1/email/auth
- **FR-EMAIL-003**: POST /v1/email/threads/{}/comments
- **FR-EMAIL-004**: POST /v1/admin/tenants/{}/email/bimi-enable, POST /v1/admin/tenants/{}/email/dns-setup, POST /v1/admin/tenants/{}/email/dns-verify
- **FR-EMAIL-005**: GET /v1/email/camel/audit-log, POST /v1/email/camel/execute, PUT /v1/email/camel/trust-list
- **FR-EMAIL-006**: POST /v1/email/tracked-domains
- **FR-EMAIL-007**: GET /v1/email/messages/{}/converted-issues, POST /v1/email/messages/{}/convert-to-issue
- **FR-EMAIL-008**: GET /v1/email/genie/sessions, GET /v1/email/genie/sessions/{}, POST /v1/email/genie/actions/{}/approve, POST /v1/email/genie/actions/{}/dismiss, PUT /v1/email/genie/config
- **FR-EMAIL-009**: GET /v1/email/outbound, POST /v1/admin/email/suppression/unsuppress, POST /v1/email/outbound/compose, POST /v1/email/outbound/send
- **FR-EMAIL-011**: GET /v1/email/dsar/jobs/{}, POST /v1/email/dsar/export
- **FR-ESOP-001**: POST /v1/esop/grants
- **FR-ESOP-003**: POST /v1/esop/valuations
- **FR-ESOP-004**: POST /v1/esop/puts
- **FR-ESOP-006**: POST /v1/esop/ma-events
- **FR-ESOP-007**: GET /v1/esop/members/{}/dashboard
- **FR-HR-002**: GET /v1/hr/members/{}/contract-history, PUT /v1/hr/members/{}/contract
- **FR-HR-003**: POST /v1/hr/members/{}/cccd-consent
- **FR-HR-004**: GET /v1/hr/members/{}/leave-balance, POST /v1/hr/leave-requests, POST /v1/hr/leave-requests/{}/approve, POST /v1/hr/leave-requests/{}/cancel, POST /v1/hr/leave-requests/{}/reject
- **FR-HR-005**: GET /v1/hr/policy
- **FR-HR-006**: POST /v1/hr/accrual/corrections
- **FR-HR-009**: GET /v1/hr/terminations/{}, POST /v1/hr/terminations, POST /v1/hr/terminations/{}/ceo-sign, POST /v1/hr/terminations/{}/cfo-sign, POST /v1/hr/terminations/{}/dispute
- **FR-INV-001**: GET /v1/inv/invoices, GET /v1/inv/invoices/{}, POST /v1/inv/invoices/draft, POST /v1/inv/invoices/{}/approve, POST /v1/inv/invoices/{}/lines/correction, POST /v1/inv/invoices/{}/send, POST /v1/inv/invoices/{}/void, POST /v1/inv/invoices/{}/write-off
- **FR-INV-002**: GET /v1/inv/fx/convert, GET /v1/inv/fx/rates, POST /v1/admin/inv/fx/override
- **FR-INV-007**: GET /v1/inv/hoadon/{}, POST /v1/inv/hoadon/emit, POST /v1/inv/hoadon/{}/resubmit
- **FR-INV-008**: GET /v1/inv/cancellation-forms, GET /v1/inv/hoadon/{}/cancellation, POST /v1/inv/hoadon/{}/cancel
- **FR-INV-009**: GET /v1/inv/reports/aging, POST /v1/inv/reports/aging
- **FR-INV-010**: GET /v1/inv/dunning/drafts, POST /v1/inv/dunning/drafts/{}/approve, POST /v1/inv/dunning/drafts/{}/dismiss, POST /v1/inv/dunning/scan
- **FR-INV-011**: GET /v1/inv/recognition/journal-entries, GET /v1/inv/recognition/schedules/{}, GET /v1/inv/recognition/snapshots/{}, POST /v1/inv/recognition/rollforward, POST /v1/inv/recognition/schedules
- **FR-KB-002**: GET /v1/kb/docs/{}/render, POST /v1/kb/docs/{}/render
- **FR-KB-003**: GET /v1/kb/docs/{}, POST /v1/kb/docs/{}/share-links, POST /v1/kb/share-links/{}/revoke, PUT /v1/kb/docs/{}/visibility
- **FR-KB-004**: POST /v1/kb/search/lexical
- **FR-KB-005**: POST /v1/kb/search/semantic
- **FR-KB-006**: POST /v1/kb/search/rerank
- **FR-KB-007**: POST /v1/kb/docs/{}/ask
- **FR-KB-008**: GET /v1/kb/runbooks/match, PUT /v1/kb/docs/{}/runbook-tags
- **FR-KB-009**: PUT /v1/kb/docs/{}/translation
- **FR-LEARN-001**: POST /v1/learn/members/{}/mastery
- **FR-LEARN-004**: POST /v1/learn/councils, POST /v1/learn/councils/{}/scores
- **FR-LEARN-005**: GET /v1/learn/councils/{}/disclosure
- **FR-MCP-003**: POST /v1/mcp/naming/validate
- **FR-MCP-006**: GET /v1/admin/tenants/{}/mcp/gating-decisions, POST /v1/admin/tenants/{}/mcp/gating-policy, POST /v1/admin/tenants/{}/mcp/gating-policy/activate, POST /v1/mcp/tools/{}/confirm
- **FR-MCP-007**: GET /v1/mcp/tasks, GET /v1/mcp/tasks/{}, POST /v1/mcp/tasks/{}/cancel, POST /v1/mcp/tools/{}/call
- **FR-MCP-008**: GET /v1/mcp/elicitations, POST /v1/mcp/elicitations/{}/cancel, POST /v1/mcp/elicitations/{}/respond
- **FR-OKR-003**: POST /v1/okr/krs/{}/custom-sql/ceo-sign, POST /v1/okr/krs/{}/custom-sql/cfo-sign, POST /v1/okr/krs/{}/custom-sql/request
- **FR-OKR-005**: POST /v1/okr/krs/{}/checkins
- **FR-OKR-006**: GET /v1/okr/digest/runs, POST /v1/okr/digest/trigger, PUT /v1/okr/digest/recipients/{}
- **FR-PORTAL-001**: GET /v1/portal/views/{}, GET /v1/portal/views/{}/export, GET /v1/portal/views/{}/{}, POST /v1/portal/views/{}/search
- **FR-PORTAL-002**: GET /v1/admin/tenants/{}/brand-pack/{}/export, POST /v1/admin/tenants/{}/brand-pack, POST /v1/admin/tenants/{}/brand-pack/rollback, POST /v1/admin/tenants/{}/brand-pack/{}/activate, POST /v1/admin/tenants/{}/cname, POST /v1/admin/tenants/{}/cname/{}/verify
- **FR-PORTAL-003**: GET /v1/portal/sign-in, PATCH /v1/admin/engagements/{}/idp, POST /v1/admin/engagements/{}/idp, POST /v1/admin/engagements/{}/idp/groups-map, POST /v1/admin/engagements/{}/scim-token/rotate
- **FR-PORTAL-004**: GET /v1/admin/tenants/{}/deprovision-log, POST /v1/admin/engagements/{}/subjects/{}/restore
- **FR-PORTAL-005**: GET /v1/portal/genie/sessions, GET /v1/portal/genie/sessions/{}/messages, POST /v1/portal/genie/query, POST /v1/portal/genie/sessions/{}/archive
- **FR-PORTAL-006**: GET /v1/portal/workflows, GET /v1/portal/workflows/{}, POST /v1/admin/tenants/{}/workflow-routes, POST /v1/portal/workflows/submit, POST /v1/portal/workflows/{}/reopen, POST /v1/portal/workflows/{}/reply
- **FR-PORTAL-007**: GET /v1/portal/pwa/subscriptions, PATCH /v1/portal/pwa/preferences, POST /v1/portal/pwa/subscribe, POST /v1/portal/pwa/unsubscribe
- **FR-PORTAL-008**: GET /v1/admin/tenants/{}/dsar, GET /v1/portal/dsar/{}, POST /v1/admin/dsar/{}/deny, POST /v1/portal/dsar/request
- **FR-RES-002**: POST /v1/res/allocations/propose
- **FR-RES-005**: POST /v1/res/ot-consent
- **FR-REW-001**: POST /v1/rew/comp
- **FR-REW-002**: GET /v1/rew/params/tax_bracket
- **FR-REW-003**: POST /v1/rew/p1-demotion-consents
- **FR-REW-006**: GET /v1/rew/payslips/{}/pdf, POST /v1/rew/payslips/{}/render
- **FR-REW-007**: POST /v1/rew/bp/credits
- **FR-REW-008**: POST /v1/rew/p3-distributions
- **FR-SKILL-201**: POST /v1/skill/oci/push
- **FR-TEN-003**: GET /v1/admin/tenants/{}/billing, POST /v1/admin/tenants/{}/billing/refund
- **FR-TEN-005**: GET /v1/admin/packs/catalog, GET /v1/admin/tenants/{}/packs, POST /v1/admin/tenants/{}/packs/install, POST /v1/admin/tenants/{}/packs/{}/override, POST /v1/admin/tenants/{}/packs/{}/reinstall, POST /v1/admin/tenants/{}/packs/{}/uninstall
- **FR-TEN-101**: GET /v1/signup/oidc-callback, GET /v1/signup/slug-available, POST /v1/signup/complete, POST /v1/signup/payment-intent, POST /v1/signup/start, POST /v1/signup/verify-otp
- **FR-TEN-102**: GET /v1/admin/tenants/{}/vnd/invoices, GET /v1/admin/tenants/{}/vnd/invoices/{}, GET /v1/signup/vnd/token-bind-return, POST /v1/admin/tenants/{}/vnd/refund, POST /v1/admin/tenants/{}/vnd/token/revoke, POST /v1/signup/vnd/token-bind-start
- **FR-TEN-105**: GET /v1/admin/tenants/{}/bundle, GET /v1/admin/tenants/{}/bundle/{}/download, GET /v1/admin/tenants/{}/bundle/{}/verify, POST /v1/admin/tenants/{}/bundle/export
- **FR-TEN-106**: GET /v1/admin/permanent-delete/{}/verify, POST /v1/admin/permanent-delete/{}/cancel, POST /v1/admin/permanent-delete/{}/execute, POST /v1/admin/permanent-delete/{}/retry-cascade/{}, POST /v1/admin/permanent-delete/{}/sign-clo, POST /v1/admin/permanent-delete/{}/sign-cso, POST /v1/admin/tenants/{}/permanent-delete/initiate
- **FR-TEN-107**: GET /v1/ten/admin/audit-events
- **FR-TEN-202**: POST /v1/ten/hostile-overrides
- **FR-TIME-002**: GET /v1/time/timer/current, POST /v1/time/timer/abandon, POST /v1/time/timer/heartbeat, POST /v1/time/timer/pause, POST /v1/time/timer/resume, POST /v1/time/timer/start, POST /v1/time/timer/stop
- **FR-TIME-003**: GET /v1/time/entries/manual/pending-approvals, POST /v1/time/entries/manual
- **FR-TIME-004**: GET /v1/time/proposals, POST /v1/time/proposals/{}/accept, POST /v1/time/proposals/{}/reject
- **FR-TIME-005**: PATCH /v1/admin/tenants/{}, PATCH /v1/engagements/{}, PATCH /v1/projects/{}
- **FR-TIME-006**: GET /v1/time/timesheets/mine, GET /v1/time/timesheets/pending, GET /v1/time/timesheets/{}/diff, POST /v1/time/timesheets/bulk-approve, POST /v1/time/timesheets/{}/approve, POST /v1/time/timesheets/{}/reject, POST /v1/time/timesheets/{}/submit
- **FR-TIME-007**: GET /v1/time/vn-ot/status, POST /v1/admin/members/{}/vn-ot-approval
- **FR-TIME-008**: GET /v1/time/expenses, POST /v1/admin/engagements/{}/expense-policy, POST /v1/time/expenses/upload, POST /v1/time/expenses/{}/approve, POST /v1/time/expenses/{}/attach-to-invoice, POST /v1/time/expenses/{}/confirm, POST /v1/time/expenses/{}/reject
- **FR-TIME-009**: POST /v1/time/rollup

## Orphan endpoint references

Endpoint paths referenced in FR text but not declared in any FR's §3 API contract. May be intentional (external API, future-FR, internal-only) — review below:

- `FR-AI-001` references `POST /v1/chat/completions` — no FR declares this in §3.
- `FR-AI-104` references `GET /v1/ai/vn-providers/health` — no FR declares this in §3.
- `FR-AI-104` references `PUT /v1/ai/vn-providers/{}/creds` — no FR declares this in §3.
- `FR-AUTH-001` references `PATCH /v1/admin/tenants/<id>` — no FR declares this in §3.
- `FR-AUTH-001` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-AUTH-002` references `POST /v1/admin/subjects` — no FR declares this in §3.
- `FR-AUTH-003` references `POST /v1/admin/subjects` — no FR declares this in §3.
- `FR-AUTH-004` references `POST /v1/auth/token` — no FR declares this in §3.
- `FR-AUTH-005` references `GET /v1/admin/subjects` — no FR declares this in §3.
- `FR-AUTH-005` references `GET /v1/admin/tenants` — no FR declares this in §3.
- `FR-AUTH-005` references `POST /v1/admin/subjects/abc-id/revoke` — no FR declares this in §3.
- `FR-AUTH-005` references `POST /v1/admin/subjects/{}/revoke` — no FR declares this in §3.
- `FR-AUTH-005` references `POST /v1/admin/subjects/{}/unrevoke` — no FR declares this in §3.
- `FR-AUTH-006` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-AUTH-101` references `DELETE /v1/admin/subjects/{}/roles` — no FR declares this in §3.
- `FR-AUTH-101` references `DELETE /v1/admin/subjects/{}/roles/{}` — no FR declares this in §3.
- `FR-AUTH-101` references `GET /v1/admin/roles` — no FR declares this in §3.
- `FR-AUTH-101` references `GET /v1/admin/roles**` — no FR declares this in §3.
- `FR-AUTH-101` references `POST /v1/admin/subjects/{}/roles` — no FR declares this in §3.
- `FR-AUTH-102` references `DELETE /v1/auth/mfa/factors/{}` — no FR declares this in §3.
- `FR-AUTH-102` references `GET /v1/auth/mfa/factors` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/challenges` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/totp/enrol` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/totp/enrol/finish` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/webauthn/enrol/begin` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/webauthn/enrol/finish` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/recovery-codes/consume` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/unlock` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/verify` — no FR declares this in §3.
- `FR-AUTH-103` references `GET /v1/auth/saml/idp-configs/{}/sp-metadata` — no FR declares this in §3.
- `FR-AUTH-103` references `GET /v1/auth/saml/initiate` — no FR declares this in §3.
- `FR-AUTH-103` references `POST /v1/auth/saml/acs` — no FR declares this in §3.
- `FR-AUTH-103` references `POST /v1/auth/saml/idp-configs` — no FR declares this in §3.
- `FR-AUTH-104` references `GET /v1/auth/oidc/callback` — no FR declares this in §3.
- `FR-AUTH-104` references `GET /v1/auth/oidc/initiate` — no FR declares this in §3.
- `FR-AUTH-104` references `PATCH /v1/auth/oidc/idp-configs/{}` — no FR declares this in §3.
- `FR-AUTH-104` references `POST /v1/auth/oidc/idp-configs` — no FR declares this in §3.
- `FR-AUTH-105` references `DELETE /v1/auth/passkey/factors/{}` — no FR declares this in §3.
- `FR-AUTH-105` references `GET /v1/auth/passkey/factors` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/autofill-options` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/downgrade-optout` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/enrol/begin` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/enrol/finish` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/login/begin` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/login/finish` — no FR declares this in §3.
- `FR-AUTH-106` references `POST /v1/auth/login` — no FR declares this in §3.
- `FR-AUTH-106` references `POST /v1/auth/mfa/challenge/{}/verify` — no FR declares this in §3.
- `FR-AUTH-106` references `PUT /v1/admin/tenants/{}/travel-policy` — no FR declares this in §3.
- `FR-AUTH-107` references `POST /v1/admin/subjects/{}/password` — no FR declares this in §3.
- `FR-AUTH-107` references `POST /v1/auth/password/rotate` — no FR declares this in §3.
- `FR-AUTH-107` references `POST /v1/auth/signup` — no FR declares this in §3.
- `FR-AUTH-107` references `PUT /v1/admin/tenants/{}/hibp-policy` — no FR declares this in §3.
- `FR-AUTH-108` references `GET /v1/auth/lumi/verify` — no FR declares this in §3.
- `FR-AUTH-108` references `POST /v1/auth/lumi/issue` — no FR declares this in §3.
- `FR-AUTH-109` references `GET /v1/auth/migration/preview` — no FR declares this in §3.
- `FR-AUTH-109` references `GET /v1/auth/migration/refresh-events` — no FR declares this in §3.
- `FR-AUTH-109` references `POST /v1/auth/migration/extend-grace` — no FR declares this in §3.
- `FR-MEMORY-108` references `GET /v1/memory/search` — no FR declares this in §3.
- `FR-CRM-001` references `DELETE /v1/crm/contacts/{}/memberships/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/accounts/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/contacts/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/deals/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/pipelines` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/accounts` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/accounts/{}` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/contacts` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/deals` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/deals/{}` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/accounts` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/contacts` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/contacts/{}/memberships` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/deals` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/deals/{}/stage` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/deals/{}/status` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/pipelines` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/pipelines/{}/stages` — no FR declares this in §3.
- `FR-CRM-002` references `GET /v1/crm/accounts/{}/activities` — no FR declares this in §3.
- `FR-CRM-002` references `GET /v1/crm/contacts/{}/activities` — no FR declares this in §3.
- `FR-CRM-002` references `POST /v1/crm/activities` — no FR declares this in §3.
- `FR-CRM-004` references `GET /v1/crm/deals/{}/conversion` — no FR declares this in §3.
- `FR-CUO-102` references `GET /v1/cuo/runs/{}/checkpoints` — no FR declares this in §3.
- `FR-CUO-103` references `GET /v1/cuo/runs/{}/trace` — no FR declares this in §3.
- `FR-CUO-103` references `POST /v1/cuo/trace/{}/replay` — no FR declares this in §3.
- `FR-CUO-104` references `GET /v1/cuo/chains/{}` — no FR declares this in §3.
- `FR-CUO-105` references `GET /v1/cuo/chains/{}/rollback-status` — no FR declares this in §3.
- `FR-CUO-105` references `POST /v1/cuo/chains/{}/rollback` — no FR declares this in §3.
- `FR-DOC-001` references `GET /v1/doc/documents` — no FR declares this in §3.
- `FR-DOC-001` references `GET /v1/doc/documents/{}` — no FR declares this in §3.
- `FR-DOC-001` references `PATCH /v1/doc/documents` — no FR declares this in §3.
- `FR-DOC-001` references `PATCH /v1/doc/documents/{}` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/archive` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/finalize` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/legal-hold` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/versions` — no FR declares this in §3.
- `FR-DOC-003` references `GET /v1/doc/aatl/signatures/{}` — no FR declares this in §3.
- `FR-DOC-003` references `POST /v1/doc/aatl/sign` — no FR declares this in §3.
- `FR-DOC-003` references `PUT /v1/doc/aatl/creds` — no FR declares this in §3.
- `FR-DOC-007` references `GET /v1/doc/documents/{}/lifecycle` — no FR declares this in §3.
- `FR-DOC-007` references `GET /v1/doc/documents/{}/parent-chain` — no FR declares this in §3.
- `FR-DOC-007` references `PUT /v1/doc/documents/{}/lifecycle` — no FR declares this in §3.
- `FR-DOC-009` references `GET /v1/doc/renewal-drafts` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/documents/{}/draft-renewal` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/renewal-drafts/{}/approve` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/renewal-drafts/{}/dismiss` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/renewal-drafts/{}/send` — no FR declares this in §3.
- `FR-DOC-010` references `GET /v1/doc/imports` — no FR declares this in §3.
- `FR-EMAIL-001` references `GET /v1/email/healthz` — no FR declares this in §3.
- `FR-EMAIL-001` references `GET /v1/email/messages` — no FR declares this in §3.
- `FR-EMAIL-001` references `GET /v1/email/messages/{}/status` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/assign` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/close` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/reopen` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/snooze` — no FR declares this in §3.
- `FR-EMAIL-006` references `DELETE /v1/email/tracked-domains/{}` — no FR declares this in §3.
- `FR-EMAIL-006` references `GET /v1/email/tracked-domains` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/draft` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/cancel` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/dispatch` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/sign-am` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/sign-cfo` — no FR declares this in §3.
- `FR-ESOP-001` references `GET /v1/esop/grants/{}` — no FR declares this in §3.
- `FR-ESOP-001` references `POST /v1/esop/grants/{}/cancel` — no FR declares this in §3.
- `FR-ESOP-001` references `POST /v1/esop/grants/{}/ceo-sign` — no FR declares this in §3.
- `FR-ESOP-001` references `POST /v1/esop/grants/{}/member-sign` — no FR declares this in §3.
- `FR-ESOP-002` references `GET /v1/esop/grants/{}/accruals` — no FR declares this in §3.
- `FR-ESOP-002` references `POST /v1/esop/vesting/run-batch` — no FR declares this in §3.
- `FR-ESOP-003` references `GET /v1/esop/valuations/{}` — no FR declares this in §3.
- `FR-ESOP-003` references `POST /v1/esop/valuations/{}/board-sign` — no FR declares this in §3.
- `FR-ESOP-003` references `POST /v1/esop/valuations/{}/dismiss` — no FR declares this in §3.
- `FR-ESOP-004` references `GET /v1/esop/members/{}/puts` — no FR declares this in §3.
- `FR-ESOP-004` references `GET /v1/esop/puts/{}` — no FR declares this in §3.
- `FR-ESOP-004` references `POST /v1/esop/puts/{}/approve` — no FR declares this in §3.
- `FR-ESOP-004` references `POST /v1/esop/puts/{}/reject` — no FR declares this in §3.
- `FR-ESOP-005` references `GET /v1/esop/leaver-outcomes/{}` — no FR declares this in §3.
- `FR-ESOP-005` references `POST /v1/esop/leaver-outcomes` — no FR declares this in §3.
- `FR-ESOP-005` references `POST /v1/esop/leaver-outcomes/{}/ceo-sign` — no FR declares this in §3.
- `FR-ESOP-005` references `POST /v1/esop/leaver-outcomes/{}/cfo-sign` — no FR declares this in §3.
- `FR-ESOP-006` references `GET /v1/esop/ma-events/{}` — no FR declares this in §3.
- `FR-ESOP-006` references `POST /v1/esop/ma-events/{}/accelerate` — no FR declares this in §3.
- `FR-ESOP-006` references `POST /v1/esop/ma-events/{}/board-sign` — no FR declares this in §3.
- `FR-HR-001` references `GET /v1/admin/members` — no FR declares this in §3.
- `FR-HR-001` references `GET /v1/admin/members/{}` — no FR declares this in §3.
- `FR-HR-001` references `PATCH /v1/admin/members` — no FR declares this in §3.
- `FR-HR-001` references `PATCH /v1/admin/members/{}` — no FR declares this in §3.
- `FR-HR-001` references `POST /v1/admin/members` — no FR declares this in §3.
- `FR-HR-001` references `POST /v1/admin/members/{}/transition` — no FR declares this in §3.
- `FR-HR-003` references `DELETE /v1/hr/members/{}/cccd-photo` — no FR declares this in §3.
- `FR-HR-003` references `GET /v1/hr/members/{}/cccd-photo` — no FR declares this in §3.
- `FR-HR-003` references `POST /v1/hr/members/{}/cccd-photo` — no FR declares this in §3.
- `FR-HR-003` references `POST /v1/hr/members/{}/cccd-photo/rotate` — no FR declares this in §3.
- `FR-HR-005` references `POST /v1/hr/policy-versions` — no FR declares this in §3.
- `FR-HR-005` references `PUT /v1/hr/tenant-policy-override` — no FR declares this in §3.
- `FR-HR-006` references `GET /v1/hr/members/{}/accrual-ledger` — no FR declares this in §3.
- `FR-HR-006` references `POST /v1/hr/accrual/run-batch` — no FR declares this in §3.
- `FR-HR-007` references `GET /v1/hr/onboarding/sagas/{}` — no FR declares this in §3.
- `FR-HR-007` references `POST /v1/hr/onboarding/start` — no FR declares this in §3.
- `FR-HR-007` references `POST /v1/hr/onboarding/{}/compensate` — no FR declares this in §3.
- `FR-HR-007` references `POST /v1/hr/onboarding/{}/retry` — no FR declares this in §3.
- `FR-HR-008` references `GET /v1/hr/members/{}/perf-history` — no FR declares this in §3.
- `FR-HR-008` references `POST /v1/hr/perf/snapshot` — no FR declares this in §3.
- `FR-INV-003` references `POST /v1/inv/stripe-secrets/rotate` — no FR declares this in §3.
- `FR-INV-003` references `POST /v1/inv/webhooks/stripe/{}` — no FR declares this in §3.
- `FR-INV-004` references `GET /v1/admin/unmatched-receipts` — no FR declares this in §3.
- `FR-INV-004` references `GET /v1/admin/wise-events` — no FR declares this in §3.
- `FR-INV-004` references `GET /v1/admin/wise-events/{}` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/admin/unmatched-receipts/{}/resolve` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/admin/wise-events/{}/restore` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/webhooks/wise/12345678` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/webhooks/wise/{}` — no FR declares this in §3.
- `FR-INV-005` references `POST /v1/inv/webhook-secrets/rotate` — no FR declares this in §3.
- `FR-INV-005` references `POST /v1/inv/webhooks/vietqr/acme-corp` — no FR declares this in §3.
- `FR-INV-005` references `POST /v1/inv/webhooks/vietqr/{}` — no FR declares this in §3.
- `FR-INV-006` references `GET /v1/inv/cash-app/allocations` — no FR declares this in §3.
- `FR-INV-006` references `GET /v1/inv/cash-app/unmatched` — no FR declares this in §3.
- `FR-INV-006` references `POST /v1/inv/cash-app/allocate-manual` — no FR declares this in §3.
- `FR-INV-006` references `POST /v1/inv/cash-app/dry-run` — no FR declares this in §3.
- `FR-INV-006` references `POST /v1/inv/cash-app/reverse` — no FR declares this in §3.
- `FR-KB-001` references `DELETE /v1/kb/documents` — no FR declares this in §3.
- `FR-KB-001` references `GET /v1/kb/documents` — no FR declares this in §3.
- `FR-KB-001` references `GET /v1/kb/documents/{}` — no FR declares this in §3.
- `FR-KB-001` references `PATCH /v1/kb/documents/{}` — no FR declares this in §3.
- `FR-KB-001` references `POST /v1/kb/documents` — no FR declares this in §3.
- `FR-KB-001` references `POST /v1/kb/documents/{}/archive` — no FR declares this in §3.
- `FR-KB-001` references `POST /v1/kb/documents/{}/versions` — no FR declares this in §3.
- `FR-KB-009` references `GET /v1/kb/docs/{}/translation-parity` — no FR declares this in §3.
- `FR-LEARN-001` references `GET /v1/learn/members/{}/mastery` — no FR declares this in §3.
- `FR-LEARN-001` references `GET /v1/learn/skills/tree` — no FR declares this in §3.
- `FR-LEARN-001` references `POST /v1/learn/skills` — no FR declares this in §3.
- `FR-LEARN-002` references `GET /v1/learn/members/{}/evidence` — no FR declares this in §3.
- `FR-LEARN-002` references `POST /v1/learn/evidence/{}/verify` — no FR declares this in §3.
- `FR-LEARN-002` references `POST /v1/learn/members/{}/evidence` — no FR declares this in §3.
- `FR-LEARN-003` references `GET /v1/learn/members/{}/vp` — no FR declares this in §3.
- `FR-LEARN-003` references `GET /v1/learn/vp/weights` — no FR declares this in §3.
- `FR-LEARN-003` references `POST /v1/learn/vp/rollup/trigger` — no FR declares this in §3.
- `FR-LEARN-003` references `POST /v1/learn/vp/weights` — no FR declares this in §3.
- `FR-LEARN-004` references `GET /v1/learn/councils/{}` — no FR declares this in §3.
- `FR-LEARN-004` references `POST /v1/learn/councils/{}/dismiss` — no FR declares this in §3.
- `FR-LEARN-004` references `POST /v1/learn/councils/{}/judges` — no FR declares this in §3.
- `FR-LEARN-006` references `GET /v1/learn/promotions/{}` — no FR declares this in §3.
- `FR-LEARN-006` references `POST /v1/learn/promotions/{}/ceo-sign` — no FR declares this in §3.
- `FR-LEARN-006` references `POST /v1/learn/promotions/{}/chro-sign` — no FR declares this in §3.
- `FR-LEARN-006` references `POST /v1/learn/promotions/{}/decline` — no FR declares this in §3.
- `FR-LEARN-007` references `GET /v1/learn/vp-rew/handoffs` — no FR declares this in §3.
- `FR-LEARN-007` references `POST /v1/learn/vp-rew/trigger` — no FR declares this in §3.
- `FR-MCP-001` references `POST /v1/mcp/register` — no FR declares this in §3.
- `FR-MCP-002` references `GET /v1/mcp/servers` — no FR declares this in §3.
- `FR-MCP-002` references `POST /v1/mcp/servers/deregister` — no FR declares this in §3.
- `FR-MCP-002` references `POST /v1/mcp/servers/heartbeat` — no FR declares this in §3.
- `FR-MCP-002` references `POST /v1/mcp/servers/register` — no FR declares this in §3.
- `FR-OBS-001` references `POST /v1/traces` — no FR declares this in §3.
- `FR-OKR-001` references `DELETE /v1/okr/objectives/{}/key_results/{}` — no FR declares this in §3.
- `FR-OKR-001` references `GET /v1/okr/cycles` — no FR declares this in §3.
- `FR-OKR-001` references `GET /v1/okr/objectives` — no FR declares this in §3.
- `FR-OKR-001` references `GET /v1/okr/teams` — no FR declares this in §3.
- `FR-OKR-001` references `PATCH /v1/okr/objectives/{}` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/cycles` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/cycles/{}/transition` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/key_results/{}/progress` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/objectives` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/objectives/{}/key_results` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/objectives/{}/transition` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/teams` — no FR declares this in §3.
- `FR-OKR-004` references `GET /v1/okr/auto-progress/runs` — no FR declares this in §3.
- `FR-OKR-004` references `GET /v1/okr/auto-progress/runs/{}` — no FR declares this in §3.
- `FR-OKR-004` references `POST /v1/okr/auto-progress/trigger` — no FR declares this in §3.
- `FR-OKR-005` references `GET /v1/okr/krs/{}/checkins` — no FR declares this in §3.
- `FR-OKR-005` references `GET /v1/okr/krs/{}/trend` — no FR declares this in §3.
- `FR-OKR-007` references `GET /v1/okr/retros` — no FR declares this in §3.
- `FR-OKR-007` references `POST /v1/okr/retros/{}/approve` — no FR declares this in §3.
- `FR-OKR-007` references `POST /v1/okr/retros/{}/dismiss` — no FR declares this in §3.
- `FR-OKR-007` references `POST /v1/okr/retros/{}/regenerate` — no FR declares this in §3.
- `FR-PROJ-001` references `DELETE /v1/proj/issues/{}` — no FR declares this in §3.
- `FR-PROJ-001` references `GET /v1/proj/issues` — no FR declares this in §3.
- `FR-PROJ-001` references `GET /v1/proj/issues/{}` — no FR declares this in §3.
- `FR-PROJ-001` references `PATCH /v1/proj/issues/issue-` — no FR declares this in §3.
- `FR-PROJ-001` references `PATCH /v1/proj/issues/{}` — no FR declares this in §3.
- `FR-PROJ-001` references `POST /v1/proj/issues` — no FR declares this in §3.
- `FR-PROJ-001` references `POST /v1/proj/issues/issue-1/links` — no FR declares this in §3.
- `FR-PROJ-001` references `POST /v1/proj/issues/{}/links` — no FR declares this in §3.
- `FR-PROJ-002` references `DELETE /v1/proj/decisions/<id>` — no FR declares this in §3.
- `FR-PROJ-002` references `GET /v1/memory/search` — no FR declares this in §3.
- `FR-PROJ-002` references `PATCH /v1/proj/decisions/<id>` — no FR declares this in §3.
- `FR-PROJ-002` references `PATCH /v1/proj/issues/issue-` — no FR declares this in §3.
- `FR-PROJ-002` references `POST /v1/proj/decisions/<id>/retract` — no FR declares this in §3.
- `FR-RES-001` references `GET /v1/res/matrix/runs/{}` — no FR declares this in §3.
- `FR-RES-001` references `GET /v1/res/members/{}/capacity` — no FR declares this in §3.
- `FR-RES-001` references `POST /v1/res/matrix/trigger` — no FR declares this in §3.
- `FR-RES-002` references `GET /v1/res/allocations/changes` — no FR declares this in §3.
- `FR-RES-002` references `POST /v1/res/allocations/{}/commit` — no FR declares this in §3.
- `FR-RES-003` references `GET /v1/res/flags/summary` — no FR declares this in §3.
- `FR-RES-003` references `GET /v1/res/weekly-digests` — no FR declares this in §3.
- `FR-RES-004` references `GET /v1/res/hiring-memos` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos/{}/ceo-sign` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos/{}/cfo-sign` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos/{}/dismiss` — no FR declares this in §3.
- `FR-RES-005` references `GET /v1/res/members/{}/ot-status` — no FR declares this in §3.
- `FR-REW-001` references `GET /v1/rew/comp/{}/decrypt` — no FR declares this in §3.
- `FR-REW-001` references `GET /v1/rew/members/{}/comp-history` — no FR declares this in §3.
- `FR-REW-002` references `GET /v1/rew/params/{}` — no FR declares this in §3.
- `FR-REW-002` references `POST /v1/rew/params` — no FR declares this in §3.
- `FR-REW-002` references `POST /v1/rew/replay-test/trigger` — no FR declares this in §3.
- `FR-REW-003` references `POST /v1/rew/p1-demotion-consents/{}/ceo-sign` — no FR declares this in §3.
- `FR-REW-003` references `POST /v1/rew/p1-demotion-consents/{}/cfo-sign` — no FR declares this in §3.
- `FR-REW-005` references `GET /v1/rew/payroll/runs/{}` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/cfo-sign` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/chro-sign` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/commit` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/compute` — no FR declares this in §3.
- `FR-REW-006` references `POST /v1/rew/payslips/{}/verify` — no FR declares this in §3.
- `FR-REW-007` references `GET /v1/rew/members/{}/bp-balance` — no FR declares this in §3.
- `FR-REW-007` references `GET /v1/rew/members/{}/bp-ledger` — no FR declares this in §3.
- `FR-REW-007` references `POST /v1/rew/bp/debits` — no FR declares this in §3.
- `FR-REW-007` references `POST /v1/rew/bp/interest-accrual/trigger` — no FR declares this in §3.
- `FR-REW-008` references `GET /v1/rew/p3-distributions/{}` — no FR declares this in §3.
- `FR-REW-008` references `POST /v1/rew/p3-distributions/{}/ceo-sign` — no FR declares this in §3.
- `FR-REW-008` references `POST /v1/rew/p3-distributions/{}/cfo-sign` — no FR declares this in §3.
- `FR-REW-008` references `POST /v1/rew/p3-distributions/{}/execute` — no FR declares this in §3.
- `FR-REW-009` references `GET /v1/rew/payroll/batches/{}` — no FR declares this in §3.
- `FR-REW-009` references `GET /v1/rew/payroll/batches/{}/file` — no FR declares this in §3.
- `FR-REW-009` references `POST /v1/rew/payroll/batches/{}/confirm` — no FR declares this in §3.
- `FR-REW-009` references `POST /v1/rew/payroll/runs/{}/batch` — no FR declares this in §3.
- `FR-SKILL-201` references `GET /v1/skill/oci/bundles` — no FR declares this in §3.
- `FR-SKILL-201` references `POST /v1/skill/oci/pull` — no FR declares this in §3.
- `FR-SKILL-201` references `POST /v1/skill/oci/yank/{}` — no FR declares this in §3.
- `FR-TEN-001` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-TEN-002` references `DELETE /v1/admin/tenants/{}/plan/scheduled` — no FR declares this in §3.
- `FR-TEN-002` references `GET /v1/tenants/{}/plan` — no FR declares this in §3.
- `FR-TEN-002` references `GET /v1/tenants/{}/plan/history` — no FR declares this in §3.
- `FR-TEN-002` references `POST /v1/admin/founder/.../plan/override` — no FR declares this in §3.
- `FR-TEN-002` references `POST /v1/admin/founder/tenants/{}/plan/override` — no FR declares this in §3.
- `FR-TEN-002` references `POST /v1/admin/tenants/{}/plan` — no FR declares this in §3.
- `FR-TEN-003` references `DELETE /v1/subscriptions/{}` — no FR declares this in §3.
- `FR-TEN-003` references `GET /v1/charges/{}` — no FR declares this in §3.
- `FR-TEN-003` references `GET /v1/prices` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/customers` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/prices` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/refunds` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscription_items/{}/usage_records` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscription_schedules` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscriptions` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscriptions/{}` — no FR declares this in §3.
- `FR-TEN-004` references `GET /v1/usage` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/documents/search` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/metering/internal/record` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/metering/period/close` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/usage/correction` — no FR declares this in §3.
- `FR-TEN-101` references `DELETE /v1/subscriptions/{}` — no FR declares this in §3.
- `FR-TEN-101` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-TEN-101` references `POST /v1/setup_intents/{}/confirm` — no FR declares this in §3.
- `FR-TEN-101` references `POST /v1/signup/oidc-init` — no FR declares this in §3.
- `FR-TEN-102` references `POST /v1/inv/webhooks/momo/{}` — no FR declares this in §3.
- `FR-TEN-102` references `POST /v1/inv/webhooks/zalopay/{}` — no FR declares this in §3.
- `FR-TEN-103` references `GET /v1/account` — no FR declares this in §3.
- `FR-TEN-104` references `GET /v1/ten/offboarding/state/acme-corp` — no FR declares this in §3.
- `FR-TEN-104` references `GET /v1/ten/offboarding/state/{}` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/cancel` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/extend` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/finalize-termination` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/force-advance` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/initiate` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/restore-from-dead-letter` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/ceo-sign` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/challenge` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/clo-sign` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/cso-sign` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/execute` — no FR declares this in §3.
- `FR-TIME-001` references `GET /v1/time/entries` — no FR declares this in §3.
- `FR-TIME-001` references `GET /v1/time/entries/{}` — no FR declares this in §3.
- `FR-TIME-001` references `POST /v1/time/entries` — no FR declares this in §3.
- `FR-TIME-001` references `POST /v1/time/entries/{}/correct` — no FR declares this in §3.

---

**Interpretation**: Orphans are normal for (a) future-FR placeholders, (b) external APIs (Stripe, ACRA, GDT), (c) internal-only endpoints not surfaced in §3. Review individually before flagging.

---

# Appendix §B — Implementation order (topological)

_Generated 2026-05-17 — 241 FRs in 13 dependency layers. Was §2 of the former `REPORTS.md`._

## §2 source detail — Implementation order (full)



_Generated 2026-05-17 — 241 FRs in 13 dependency layers._

Each **layer** can be built in parallel (no cross-dependencies inside a layer). Layers MUST be built in order.

Within a layer, FRs are sorted alphabetically — pick by module ownership.

## Layer 0 (9 FRs — buildable in parallel)

- **FR-AI-003** [MUST, 5h, slice 1] — memory audit-row bridge — canonical Writer for AI Gateway
- **FR-AI-005** [MUST, 5h, slice 1] — Tenant-policy YAML loader — per-tenant cap + warn + override + residency
- **FR-AI-007** [MUST, 4h, slice 2] — Provider cost-table loader — YAML-backed, hot-reloadable rate table
- **FR-AI-019** [SHOULD, 12h, slice 4] — Self-hosted BGE-M3 embeddings (single L4 GPU sidecar) + ONNX-CPU fallback + adap
- **FR-AUTH-001** [MUST, 8h, slice 1] — Tenant create — root-admin in tenant 0 calls POST /v1/admin/tenants with idempot
- **FR-CHAT-001** [MUST, 8h, slice 1] — Mattermost v9.x fork at pinned MIT-Apache commit + automated license-drift watch
- **FR-DOCS-001** [SHOULD, 14h, slice 1] — Server-render NFR catalog + Risk Register + FR catalog at build time — Pagefind-
- **FR-EMAIL-001** [MUST, 12h, slice 1] — EMAIL Stalwart Rust mail server deployment — JMAP + IMAP + SMTP + ManageSieve + 
- **FR-OBS-001** [MUST, 10h, slice 1] — OTel Collector + LGTM stack (Loki + Prometheus + Tempo + Grafana) with mTLS ingr

## Layer 1 (12 FRs — buildable in parallel)

- **FR-AI-001** [MUST, 8h, slice 1] — AI Gateway cost-ledger pre-call check
- **FR-AI-014** [MUST, 8h, slice 3] — Persona-version system-prompt injection from memory memories/personas/<handle>.md
- **FR-AI-020** [COULD, 8h, slice 4] — BGE-reranker-v2-m3 cross-encoder for KB reranking (per-region sidecar; CPU fallb
- **FR-AUTH-002** [MUST, 6h, slice 1] — Subject create — POST /v1/admin/subjects with bcrypt + role allow-list + idempot
- **FR-AUTH-003** [MUST, 12h, slice 1] — RLS enforcement at every tenant-scoped table — USING + WITH CHECK + per-connecti
- **FR-EMAIL-004** [MUST, 6h, slice 1] — EMAIL DKIM signing + ARC chain forward + BIMI brand indicator — RFC 6376 + RFC 8
- **FR-EMAIL-005** [MUST, 12h, slice 2] — EMAIL CaMeL dual-LLM security layer — Privileged-LLM plans, Quarantined-LLM pars
- **FR-EMAIL-011** [MUST, 5h, slice 2] — EMAIL DSAR message export — every message a subject authored or received + chain
- **FR-OBS-003** [MUST, 8h, slice 1] — Per-service RED metrics (rate/errors/duration) via cyberos-obs-sdk shared crate 
- **FR-OBS-006** [SHOULD, 6h, slice 2] — Tail-based sampling at OTel collector — 100% errors/5xx/slow/flagged + 10% norma
- **FR-SKILL-101** [MUST, 6h, slice 1] — Skill memory integration — skill.invoked_started + skill.invoked_completed audit 
- **FR-TEN-001** [MUST, 5h, slice 1] — TEN tenant provisioning CLI — `cyberos-ten provision` ops-driven flow with schem

## Layer 2 (11 FRs — buildable in parallel)

- **FR-AI-002** [MUST, 6h, slice 1] — AI Gateway cost-ledger post-call reconcile
- **FR-AI-004** [MUST, 3h, slice 1] — Cost-hold expiry cleanup job — refund unsettled holds + emit audit
- **FR-AUTH-004** [MUST, 12h, slice 1] — JWT issuance + JWKS endpoint (RS256) with tenant_id + agent_persona + scope_gran
- **FR-AUTH-102** [MUST, 10h, slice 1] — AUTH TOTP (RFC 6238) + WebAuthn Level 3 MFA — closed factor enum + enrolment FSM
- **FR-MEMORY-101** [MUST, 18h, slice 1] — Layer-2 ingest pipeline (binlog → pgvector + Apache AGE) — chain-anchor verifica
- **FR-EMAIL-009** [MUST, 4h, slice 1] — EMAIL outbound 1:1 send — DKIM-signed via FR-EMAIL-004 + AM confirm-before-send 
- **FR-PROJ-001** [MUST, 12h, slice 1] — PROJ Issue + Cycle + Engagement schema — RLS + cross-module linkable + status FS
- **FR-SKILL-102** [MUST, 10h, slice 1] — Self-hosted OCI registry for .skill bundles — cosign signing + tenant-scoped + i
- **FR-SKILL-103** [MUST, 7h, slice 1] — SKILL.md frontmatter extension — allowed_memory_scopes + allowed_tools + version 
- **FR-TEN-002** [MUST, 4h, slice 1] — 3 plan tiers (Starter / Team / Enterprise) hardcoded with per-tier caps
- **FR-TEN-104** [MUST, 12h, slice 1] — TEN 90-day offboarding contract — closed 4-state FSM (Active → Terminating-A → T

## Layer 3 (22 FRs — buildable in parallel)

- **FR-AI-006** [MUST, 6h, slice 2] — Model-alias resolution (chat.smart → bedrock:claude-3.5-sonnet) with per-tenant 
- **FR-AUTH-005** [MUST, 8h, slice 1] — Admin REST: list tenants + list subjects + revoke subject + unrevoke + cursor pa
- **FR-AUTH-006** [MUST, 6h, slice 1] — cyberos-auth bootstrap CLI: tenant 0 + root-admin + initial signing key + sweepe
- **FR-AUTH-103** [MUST, 12h, slice 1] — AUTH SAML 2.0 SSO — SP-initiated flow + per-tenant IdP config + XML signature ve
- **FR-AUTH-105** [MUST, 8h, slice 1] — AUTH Passkey enrolment + login — discoverable credentials (resident keys) + auto
- **FR-AUTH-106** [SHOULD, 8h, slice 1] — Impossible-travel detection + adaptive MFA challenge
- **FR-MEMORY-102** [MUST, 10h, slice 1] — Layer-2 rebuild-from-Layer-1 CI gate — deterministic rebuild + spot-check + 30mi
- **FR-MEMORY-103** [MUST, 18h, slice 1] — memory-sync daemon — laptop A ↔ Cloud memory ↔ laptop B with sync_class gating + C
- **FR-CHAT-002** [MUST, 10h, slice 1] — cyberos-chat-authbridge plugin — Mattermost auth delegates to FR-AUTH-004 JWT wi
- **FR-EMAIL-002** [MUST, 6h, slice 1] — EMAIL Stalwart authbridge plugin — JMAP/IMAP/SMTP auth delegates to AUTH JWT val
- **FR-EMAIL-003** [MUST, 16h, slice 2] — EMAIL Missive-style team UX — shared inbox, thread assignment, internal comments
- **FR-EMAIL-007** [SHOULD, 6h, slice 1] — EMAIL convert-to-issue — one-click create FR-PROJ issue from message with thread
- **FR-EMAIL-010** [MUST, 5h, slice 1] — EMAIL bulk send (≥ 10 recipients) — AM + CFO/marketing dual-approval token + sup
- **FR-MCP-001** [MUST, 12h, slice 4] — MCP Gateway 2025-11-25 spec compliance — initialize + tools/list + tools/call + 
- **FR-OBS-002** [MUST, 12h, slice 1] — Tenant-aware Grafana proxy (Rust) — AST-injects tenant_id into PromQL/LogQL/Trac
- **FR-PROJ-002** [MUST, 7h, slice 1] — memory-anchored proj.decision row per Issue state change — reason + prior_chain l
- **FR-PROJ-005** [MUST, 4h, slice 2] — Rate-card schema per Engagement — (role × currency × hourly_rate × billable_defa
- **FR-PROJ-009** [MUST, 5h, slice 2] — MEMORY_LINK schema — Issue ↔ memory memory linkage (cites | implements | supersede
- **FR-SKILL-104** [MUST, 12h, slice 1] — Capability broker — subprocess sandbox enforces allowed_tools + allowed_memory_sc
- **FR-SKILL-201** [MUST, 8h, slice 1] — SKILL OCI registry deploy for `.skill` bundles — R3 distribution stage with sign
- **FR-TEN-105** [MUST, 8h, slice 2] — TEN signed-bundle export — deterministic zip + Ed25519 signature + memory audit a
- **FR-TEN-202** [SHOULD, 5h, slice 1] — TEN hostile-termination override — legal-trigger fast-track with CEO+CLO+CSO tri

## Layer 4 (26 FRs — buildable in parallel)

- **FR-AI-008** [MUST, 10h, slice 2] — LiteLLM-derived multi-provider router with retry + 30s failover SLA
- **FR-AI-015** [MUST, 6h, slice 3] — ZDR (Zero Data Retention) attestation table + enforcement when tenant policy req
- **FR-AI-016** [MUST, 8h, slice 4] — Tenant residency pinning (sg-1 / eu-1 / us-1 / vn-1) propagating to provider reg
- **FR-AUTH-101** [MUST, 12h, slice 1] — AUTH 22-role RBAC catalogue — closed enum + permission matrix + role-assignment 
- **FR-AUTH-107** [SHOULD, 4h, slice 1] — HIBP password breach check (k-anonymity) on signup + rotation
- **FR-MEMORY-104** [SHOULD, 28h, slice 2] — Tauri 2.x desktop app — macOS + Windows + Linux signed/notarised + auto-update +
- **FR-MEMORY-106** [MUST, 6h, slice 1] — memory sync_class enforcement — private vs shareable + ACL filtering + structural
- **FR-CHAT-003** [MUST, 6h, slice 1] — Per-tenant CHAT deployment — AWS Fargate + RDS Multi-AZ + Redis ElastiCache with
- **FR-DOC-006** [MUST, 8h, slice 2] — DOC identity verification — 4 methods (WebAuthn / VNeID / SMS-OTP / email-link) 
- **FR-MCP-002** [MUST, 6h, slice 2] — MCP per-module server registration + heartbeat lifecycle — 3-miss → unhealthy wi
- **FR-MCP-003** [MUST, 3h, slice 2] — MCP SEP-986 naming convention validator — `cyberos.{module}.{verb}_{noun}` patte
- **FR-MCP-004** [MUST, 10h, slice 2] — OAuth 2.1 PKCE authorization-code flow with audience-bound tokens for MCP server
- **FR-OBS-007** [MUST, 10h, slice 3] — obs-router: Alertmanager → CUO obs.triage-alert@1 skill → CHAT (≥0.70 conf) OR P
- **FR-OBS-008** [MUST, 14h, slice 3] — obs-compliance-view: pre-built read-only views (EU AI Act / PDPL / SOC 2 / ISO 2
- **FR-PROJ-003** [MUST, 10h, slice 2] — Yjs CRDT for issue description + comment-body fields; LWW for scalar metadata; r
- **FR-PROJ-004** [MUST, 5h, slice 2] — Issue lifecycle FSM — backlog → todo → in-progress → in-review → done | cancelle
- **FR-PROJ-006** [MUST, 6h, slice 2] — Billable cascade — Member-override → task-class → role-default → fallback; resol
- **FR-PROJ-010** [SHOULD, 4h, slice 3] — Citation drift detector — nightly sweep flags stale MEMORY_LINKs (deleted target,
- **FR-PROJ-013** [MUST, 6h, slice 3] — Estimate calibration snapshot — per-member per-task-class nightly batch with Bay
- **FR-PROJ-014** [MUST, 10h, slice 3] — Kanban Board view — drag/drop status transition + keyboard-first navigation + 60
- **FR-PROJ-015** [MUST, 8h, slice 3] — Timeline view — cycle window × assignee swimlane with day-grid layout, drag-resi
- **FR-PROJ-016** [SHOULD, 10h, slice 3] — Gantt view with dependency arrows — issue-to-issue precedence + critical path hi
- **FR-SKILL-105** [MUST, 9h, slice 2] — memory-capture@1 skill bundle — canonical SDK-style entry point for emitting BRAI
- **FR-SKILL-108** [MUST, 7h, slice 3] — vn-mst-validate@1 skill — Vietnamese Tax ID (MST) validation against General Dep
- **FR-TEN-106** [MUST, 5h, slice 2] — TEN permanent-delete attestation — CSO + CLO dual-sign + chain-anchored evidence
- **FR-TIME-004** [SHOULD, 6h, slice 2] — TIME auto-detect proposals — Member-confirm suggestions from PROJ activity (stat

## Layer 5 (35 FRs — buildable in parallel)

- **FR-AI-009** [MUST, 6h, slice 2] — Circuit breaker per (provider, model) with half-open recovery probing
- **FR-AI-010** [SHOULD, 8h, slice 2] — Streaming SSE end-to-end (token-by-token to client)
- **FR-AI-011** [MUST, 6h, slice 3] — Presidio EN-base PII redaction in-flight (every prompt)
- **FR-AI-017** [SHOULD, 8h, slice 4] — Per-tenant Redis response cache keyed by (tenant × redacted-prompt × model × per
- **FR-AI-022** [MUST, 8h, slice 5] — OpenTelemetry trace + span emission for every call (caller → router → provider →
- **FR-AI-104** [SHOULD, 12h, slice 1] — AI VN provider integration — Viettel Cloud + FPT Cloud as Vn1-residency LLM/embe
- **FR-AUTH-104** [MUST, 10h, slice 1] — AUTH OIDC SSO — RFC 8414 discovery + RFC 7517 JWKS rotation + per-tenant IdP con
- **FR-AUTH-108** [MUST, 6h, slice 1] — AUTH Lumi tenant-identity JWT shape — agent_persona + tenant_residency + lumi_or
- **FR-AUTH-109** [MUST, 5h, slice 1] — AUTH stub → full migration enforcer — 30-day grace window + cutover timestamp + 
- **FR-MEMORY-105** [MUST, 7h, slice 2] — cyberos doctor — watched-folders integrity invariants (manifest ↔ filesystem ↔ H
- **FR-CHAT-004** [MUST, 12h, slice 1] — PGroonga + custom Vietnamese bigram tokeniser — VN message search with ≥ 80% rec
- **FR-CHAT-005** [MUST, 10h, slice 1] — memory bridge — Postgres logical replication from chat to memory Layer-3 ingest wi
- **FR-CHAT-011** [MUST, 6h, slice 2] — Mobile push delivery — APNS + FCM with privacy-preserving payload (title + sende
- **FR-CRM-001** [MUST, 6h, slice 1] — CRM Account/Contact/Deal Postgres schema — closed entity primitives + custom pip
- **FR-CUO-101** [MUST, 12h, slice 2] — CUO Phase 2 — LangGraph supervisor + LiteLLM cascade + confidence-band escalatio
- **FR-DOC-001** [MUST, 8h, slice 1] — DOC Document repository — S3 Object-Lock Compliance bucket + per-tenant residenc
- **FR-HR-001** [MUST, 6h, slice 1] — HR Member schema — profile + role + level + contract type + leave balance + sabb
- **FR-INV-003** [MUST, 8h, slice 2] — INV Stripe webhook handler — Stripe-Signature verify + closed event-type allowli
- **FR-INV-004** [SHOULD, 6h, slice 1] — Wise webhook handler for multi-currency receipts (USD / EUR / GBP / SGD / JPY)
- **FR-INV-005** [MUST, 6h, slice 2] — INV VietQR / Napas247 webhook handler — HMAC-SHA256 signature + idempotent recei
- **FR-KB-001** [MUST, 6h, slice 1] — KB Document schema — slug + markdown body + YAML frontmatter + closed category e
- **FR-MCP-005** [MUST, 3h, slice 2] — MCP Protected Resource Metadata (RFC 9728) at `/.well-known/oauth-protected-reso
- **FR-MCP-006** [MUST, 6h, slice 2] — MCP tool-annotation gating — destructive / write / external-effect tools require
- **FR-MCP-007** [MUST, 10h, slice 3] — MCP Tasks primitive — long-running tool calls with status polling + resume-on-re
- **FR-MCP-008** [MUST, 6h, slice 3] — MCP Elicitation — server-initiated structured prompts for mid-call user input (c
- **FR-OBS-009** [MUST, 8h, slice 3] — Chain-of-custody manifest with Ed25519 signature on every compliance export — PD
- **FR-OKR-001** [MUST, 6h, slice 1] — OKR Objective × Key Result schema — Company → Team → Member cascade + quarterly 
- **FR-PROJ-007** [MUST, 6h, slice 2] — Three billing modes — Time & Materials, Fixed-Fee, Retainer — with mode-aware ro
- **FR-PROJ-008** [MUST, 5h, slice 2] — memory audit row per issue mutation — chained to PROJ history_event table with fi
- **FR-PROJ-017** [MUST, 8h, slice 3] — Brief Modal — issue deep-view with Yjs description editor + threaded comments + 
- **FR-PROJ-018** [MUST, 8h, slice 3] — Liquid-Glass design tokens (tokens.proj.css) + axe-core CI accessibility gate + 
- **FR-SKILL-106** [SHOULD, 4h, slice 3] — memory-sync@1 skill bundle — operator-facing sync trigger that defers to Stage 4 
- **FR-SKILL-109** [MUST, 7h, slice 3] — vn-bank-transfer@1 skill — VietQR + Napas247 transfer-code generator with bank-p
- **FR-TEN-103** [MUST, 10h, slice 2] — 4-residency provisioning — sg-1 / eu-1 / us-1 / vn-1 region pinning across Postg
- **FR-TIME-001** [MUST, 5h, slice 1] — TIME TimeEntry append-only schema — correction_to link semantics + tenant-scoped

## Layer 6 (58 FRs — buildable in parallel)

- **FR-AI-012** [MUST, 10h, slice 3] — VN-PII Presidio plugin (CCCD · MST · VN phone · NĐD · VN address · bank account)
- **FR-AI-018** [MUST, 6h, slice 4] — Cross-tenant cache leak property-test (hard zero) — 200K random ops + 7 regressi
- **FR-AI-021** [MUST, 14h, slice 5] — cyberos-ai operator CLI (usage · models · policy · failover · invoice · breaker 
- **FR-MEMORY-107** [MUST, 14h, slice 2] — memory capture daemon — Rust + notify crate FS watcher with rate-limit + content-
- **FR-CHAT-006** [MUST, 12h, slice 2] — Slack import — `cyberos-chat import slack` with 8-step idempotent checkpoint-dri
- **FR-CHAT-008** [MUST, 6h, slice 2] — @lumi mention parser — message mentions trigger CUO routing + memory capture row 
- **FR-CHAT-012** [MUST, 6h, slice 2] — DSAR export — Data Subject Access Request: every message a subject authored + ch
- **FR-CRM-003** [MUST, 4h, slice 5] — CRM VN account types + MST — legal entity classification (Sole/LLC/JSC/FDI) + ta
- **FR-CRM-004** [MUST, 6h, slice 5] — CRM convert-to-engagement — deal.won → PROJ Engagement creation with rate card +
- **FR-CRM-005** [MUST, 6h, slice 6] — CRM CUO crm.next-action@1 skill — AI-ranked top-3 next moves per open deal with 
- **FR-CRM-006** [SHOULD, 5h, slice 6] — CRM AI lead scoring — contact-creation-time score + nightly refresh based on act
- **FR-CRM-007** [SHOULD, 5h, slice 6] — CRM win/loss analysis CUO draft — auto-generate analysis at deal close + memory m
- **FR-CRM-009** [MUST, 4h, slice 7] — CRM vn-bank-transfer skill — VietQR payment image generation for deal collection
- **FR-CUO-102** [MUST, 5h, slice 6] — CUO Postgres checkpointer for LangGraph state — persists supervisor graph state 
- **FR-CUO-104** [MUST, 10h, slice 6] — CUO topological walk of `depends_on` chain — orchestrates multi-step skill invoc
- **FR-DOC-002** [MUST, 16h, slice 3] — DOC eIDAS QTSP integration — GlobalSign or Cryptomathic partner for EU residency
- **FR-DOC-003** [MUST, 12h, slice 3] — DOC AATL CA integration — Adobe Approved Trust List CA partner (DigiCert / Entru
- **FR-DOC-004** [MUST, 16h, slice 3] — DOC VN CA chain — VNeID + VnPay/MK Group/Viettel-CA partners for VN-residency qu
- **FR-DOC-005** [MUST, 10h, slice 2] — DOC multi-party signing workflow — ordered + parallel + counter-sign with remind
- **FR-DOC-007** [MUST, 5h, slice 1] — DOC lifecycle metadata — parties + effective_date + expiry_date + renewal_terms 
- **FR-DOC-010** [SHOULD, 10h, slice 3] — DOC third-party import — DocuSign / Adobe Sign / HelloSign migration with LTV (l
- **FR-EMAIL-006** [SHOULD, 5h, slice 1] — EMAIL tracked-domain → CRM auto-link — inbound message from tenant-tracked domai
- **FR-ESOP-001** [MUST, 5h, slice 1] — ESOP SP grant schema — Stock Plan grant with 4-year vesting + 12-month cliff def
- **FR-HR-002** [MUST, 4h, slice 6] — HR 5 contract types — indefinite + fixed_term + probation + part_time + contract
- **FR-HR-003** [MUST, 5h, slice 6] — HR CCCD photo KMS — separate keyspace for VN citizen ID photos with sev-1 access
- **FR-HR-004** [MUST, 5h, slice 6] — HR 8 leave types — annual/sick/maternity/paternity/sabbatical/unpaid/bereavement
- **FR-HR-005** [MUST, 4h, slice 6] — HR Decree 145/2020 working-hour caps + Decree 152/2020 SI rates — version-pinned
- **FR-HR-007** [MUST, 10h, slice 6] — HR onboarding saga — orchestrates AUTH + TIME + LEARN + KB + CHAT + REW provisio
- **FR-HR-008** [MUST, 6h, slice 7] — HR performance signal aggregator — read-only consumer of PROJ + TIME + LEARN sig
- **FR-HR-009** [MUST, 8h, slice 7] — HR termination workflow — Good-Leaver / Bad-Leaver branch with CFO+CEO co-sign +
- **FR-INV-006** [MUST, 8h, slice 2] — INV cash application — closed 4-step matching cascade (exact-ref → amount+date →
- **FR-KB-002** [MUST, 5h, slice 4] — KB server-side renderer — markdown → sanitised HTML (ammonia) + sanitised plaint
- **FR-KB-003** [MUST, 5h, slice 4] — KB 3 permission tiers — public / org-only / role-restricted with share-link toke
- **FR-KB-005** [MUST, 6h, slice 5] — KB BGE-M3 semantic search — memory Layer 2 vector ingest + dense embedding query 
- **FR-KB-008** [MUST, 5h, slice 5] — KB runbook category — applicability tags (provider / region / severity) for OBS 
- **FR-KB-009** [SHOULD, 4h, slice 5] — KB dual-language `translation_of` link — vi/en pairing with locale-aware reader 
- **FR-LEARN-001** [MUST, 6h, slice 7] — LEARN skill tree schema — 1-5 mastery levels per skill per Member with parent-ch
- **FR-LEARN-003** [MUST, 6h, slice 7] — LEARN VP (Voting Power) deterministic nightly roll-up — aggregates PROJ + TIME +
- **FR-OBS-004** [MUST, 6h, slice 2] — LangSmith integration for AI traces — self-hosted + per-tenant opt-in + redacted
- **FR-OKR-002** [MUST, 4h, slice 3] — OKR 3 KR types — hit_target + improvement + milestone with type-specific progres
- **FR-OKR-003** [MUST, 10h, slice 3] — OKR KR progress_source DSL — declarative query against PROJ / INV / HR / LEARN m
- **FR-OKR-005** [MUST, 5h, slice 3] — OKR weekly check-in — 1-10 confidence + rationale per KR with rolling 4-week his
- **FR-OKR-007** [SHOULD, 6h, slice 3] — OKR quarterly retro CUO draft — auto-generated retro with face-saving Vietnamese
- **FR-PORTAL-003** [MUST, 10h, slice 1] — PORTAL external IdP — SAML 2.0 + OIDC sign-in for client-tenant users + SCIM 2.0
- **FR-PORTAL-006** [MUST, 6h, slice 2] — PORTAL client-initiated workflows — new project request / billing inquiry / supp
- **FR-PROJ-011** [MUST, 6h, slice 3] — Blocker detector from comment stream — `blocked by` parser + dwell-time monitor 
- **FR-PROJ-012** [MUST, 8h, slice 3] — Cycle-review draft generator — CUO/COO-persona LLM compose at cycle close with c
- **FR-RES-001** [MUST, 10h, slice 7] — RES capacity-vs-demand matrix — nightly join across HR + PROJ + TIME + LEARN pro
- **FR-RES-004** [MUST, 8h, slice 8] — RES hiring memo CUO draft — skill-gap × CRM pipeline trigger → CEO+CFO review qu
- **FR-REW-001** [MUST, 6h, slice 1] — REW 3P income schema — P1 Base + P2 Allowance + P3 Performance with separate enc
- **FR-REW-009** [MUST, 5h, slice 2] — REW VietQR bank payroll batch send — bulk transfer file generation with CFO manu
- **FR-SKILL-107** [COULD, 3h, slice 1] — synthesis-author@1 skill — nightly multi-memory auto-evolve composes derived memo
- **FR-SKILL-110** [MUST, 11h, slice 3] — vn-vat-invoice@1 skill — Vietnamese e-invoice (hóa đơn) Decree 123 XML emitter w
- **FR-TIME-002** [MUST, 5h, slice 1] — TIME timer start/stop — single-active-timer per Member + auto-stop on logout + ≤
- **FR-TIME-003** [MUST, 6h, slice 1] — TIME manual entry form — retroactive time logging with date validation + per-day
- **FR-TIME-005** [MUST, 5h, slice 1] — TIME billable flag cascade — 4-step resolver (entry override → project default →
- **FR-TIME-006** [MUST, 6h, slice 1] — TIME weekly approval flow — Member submit → AM (engagement_admin) review → CFO v
- **FR-TIME-007** [MUST, 4h, slice 1] — TIME VN Labour Code Art. 107 OT cap — hard-block at entry write when monthly OT 

## Layer 7 (41 FRs — buildable in parallel)

- **FR-AI-013** [MUST, 8h, slice 3] — VN-PII recall ≥ 99% per-recognizer CI gate on 200-sample fixture
- **FR-MEMORY-108** [MUST, 12h, slice 2] — memory search — vector + graph + full-text in parallel + RRF fusion + BGE-rerank 
- **FR-MEMORY-109** [MUST, 8h, slice 2] — Claude Code hook capture — UserPromptSubmit + PostToolUse + Stop hooks emit BRAI
- **FR-MEMORY-111** [MUST, 9h, slice 2] — memory pre-ingest PII detection — Presidio EN + custom VN recognisers; ≥ 99.5% he
- **FR-CHAT-007** [SHOULD, 8h, slice 2] — Zalo manual export importer — `cyberos-chat import zalo --bundle.zip` with VN-Un
- **FR-CHAT-009** [SHOULD, 6h, slice 2] — Retro-capture flow — `@lumi remember the last N messages` with per-message opt-i
- **FR-CRM-002** [MUST, 8h, slice 5] — CRM activity feed — auto-log inbound email + outbound send + chat mention + cale
- **FR-CRM-008** [MUST, 3h, slice 7] — CRM vn-mst-validate skill — synchronous GDT lookup on Account write to confirm M
- **FR-CUO-103** [MUST, 4h, slice 6] — CUO Phase 2 trace rows include prompt + model + temperature + seed for determini
- **FR-CUO-105** [MUST, 6h, slice 6] — CUO per-step rollback on chain failure — execute compensating actions in reverse
- **FR-DOC-008** [MUST, 4h, slice 1] — DOC expiry alert cascade — 90/30/7-day notifications to parties + CLO with dedup
- **FR-DOC-009** [SHOULD, 6h, slice 1] — DOC renewal proposal CUO draft — auto-generate renewal terms + price adjustment 
- **FR-DOC-011** [MUST, 8h, slice 3] — DOC PAdES-B-LT format + year-9 LTV re-stamping — extend B-T signatures with vali
- **FR-ESOP-002** [MUST, 4h, slice 1] — ESOP monthly vesting accrual deterministic batch — runs EOM tenant_tz computing 
- **FR-ESOP-003** [MUST, 5h, slice 1] — ESOP annual valuation — CFO base + Board multiplier sign-off with immutable shar
- **FR-ESOP-005** [MUST, 5h, slice 2] — ESOP Good/Bad Leaver branch on HR offboarding — CFO+CEO co-sign to apply forfeit
- **FR-ESOP-006** [SHOULD, 5h, slice 2] — ESOP M&A acceleration trigger — Board declares M&A event + 5-business-day Member
- **FR-ESOP-007** [SHOULD, 6h, slice 2] — ESOP Member dashboard — personal view only (own grants + vesting + estimated val
- **FR-HR-006** [MUST, 4h, slice 6] — HR annual leave accrual nightly batch — Decree 145 formula (1d/month + 1d/5yr se
- **FR-KB-004** [MUST, 6h, slice 5] — KB FTS5 + PGroonga lexical search — VN bigram tokenisation + English stemming + 
- **FR-KB-006** [MUST, 4h, slice 5] — KB BGE-rerank-v2-m3 cross-encoder — reranks top-K results from FR-KB-004 lexical
- **FR-LEARN-002** [MUST, 4h, slice 7] — LEARN bằng cấp + chứng chỉ — degree + certification evidence types with issuer +
- **FR-LEARN-004** [MUST, 10h, slice 7] — LEARN Hội đồng Chuyên môn (Specialist Council) — 3-5 judges + multi-dim scoring 
- **FR-LEARN-007** [MUST, 4h, slice 7] — LEARN VP score → REW BP fund distribution handoff — quarter-close trigger emits 
- **FR-OBS-005** [MUST, 8h, slice 2] — W3C TraceContext correlation across logs/metrics/traces/AI-traces — propagate, e
- **FR-OKR-004** [MUST, 5h, slice 3] — OKR auto-progress nightly batch — resolves all KR progress_sources + updates cur
- **FR-OKR-006** [MUST, 6h, slice 3] — OKR Monday-morning CUO digest — auto-progress + check-ins → founder summary deli
- **FR-PORTAL-004** [MUST, 8h, slice 2] — PORTAL SCIM deprovision — session invalidation ≤ 30 s on IdP user removal + grac
- **FR-PORTAL-005** [SHOULD, 6h, slice 2] — PORTAL branded Genie chat — CUO scope-narrowed by JWT scope_grants + per-Engagem
- **FR-RES-002** [MUST, 12h, slice 8] — RES allocation Gantt UI — drag-rebalance interface over capacity matrix with opt
- **FR-RES-003** [MUST, 4h, slice 8] — RES over/under-allocation flags — 110% warning / 60% under-utilization threshold
- **FR-RES-005** [MUST, 4h, slice 8] — RES VN Labour Code Art. 107 OT cap hard-block — propose-time validation gate pre
- **FR-REW-002** [MUST, 6h, slice 1] — REW parameter versioning — immutable versioned formula parameters with 100% repl
- **FR-REW-003** [MUST, 4h, slice 1] — REW P1 protection invariant — DB CHECK constraint + service-layer guard forbiddi
- **FR-REW-004** [MUST, 6h, slice 1] — REW statutory deductions — BHXH 10.5% + BHYT 1.5% + BHTN 1% + PIT progressive pe
- **FR-REW-005** [MUST, 8h, slice 2] — REW monthly payroll compute + CFO+CHRO co-sign commit gate — orchestrates 3P + d
- **FR-REW-007** [MUST, 5h, slice 2] — REW BP (Bonus Points) ledger with ACB-rate interest accrual nightly + per-Member
- **FR-REW-010** [MUST, 3h, slice 1] — REW memory structural exclusion CI gate — no comp fields appear in memory-ingest p
- **FR-TEN-005** [MUST, 5h, slice 2] — TEN vertical-pack pricing add-on — per-pack monthly fee (not per-seat) on top of
- **FR-TEN-201** [MUST, 16h, slice 1] — TEN Singapore HoldCo flip CLI — `cyberos-ten holdco-flip` orchestrates ACRA fili
- **FR-TIME-009** [MUST, 6h, slice 1] — TIME per-cycle billable rollup → INV — per-Member × role × Engagement aggregatio

## Layer 8 (11 FRs — buildable in parallel)

- **FR-MEMORY-110** [MUST, 6h, slice 2] — memory capture daemon supervision — systemd + launchd units + /healthz + watchdog
- **FR-CHAT-010** [MUST, 5h, slice 2] — Decommission signal — (chat msgs) / (chat + slack + zalo msgs) ≥ 0.95 over 14-da
- **FR-EMAIL-008** [SHOULD, 8h, slice 2] — EMAIL Genie prefix — inbound subject prefix routes message to Genie (Branded AI)
- **FR-ESOP-004** [MUST, 8h, slice 2] — ESOP put-option exec flow — Year 3+ eligibility + per-Member annual cap + CFO ap
- **FR-INV-001** [MUST, 8h, slice 1] — INV invoice substrate — draft invoices from TIME per-cycle rollup with rate-card
- **FR-KB-007** [MUST, 8h, slice 5] — KB Ask-this-page Q&A — CUO-grounded answer over current + linked docs with span-
- **FR-LEARN-005** [MUST, 5h, slice 7] — LEARN per-judge score isolation — never exit LEARN boundary; HR receives only su
- **FR-LEARN-006** [MUST, 5h, slice 7] — LEARN promotion approval workflow — CEO + CHRO sign-off after council vote with 
- **FR-REW-006** [MUST, 6h, slice 2] — REW byte-identical payslip PDF render — Tectonic + pinned fonts produces determi
- **FR-REW-008** [MUST, 6h, slice 2] — REW quarterly P3 distribution from BP fund — CEO+CFO sign-off + LEARN-007 VP sha
- **FR-TEN-004** [MUST, 8h, slice 1] — 4-axis metering — seats · API · AI tokens · storage (memory audit per metric even

## Layer 9 (5 FRs — buildable in parallel)

- **FR-INV-002** [MUST, 6h, slice 1] — INV multi-currency support — VND/USD/SGD/EUR/GBP with daily SBV FX snapshot + pe
- **FR-INV-007** [MUST, 6h, slice 2] — INV VN hóa đơn auto-emit on AM-send — Decree 123/2020 GDT XML signing + idempote
- **FR-INV-009** [MUST, 4h, slice 2] — INV AR aging report — current/30/60/90/120+ bucket rollup per customer + per eng
- **FR-INV-011** [MUST, 5h, slice 2] — INV revenue recognition — ASC 606 / IFRS 15 compliant deferred-revenue rollforwa
- **FR-TEN-003** [MUST, 8h, slice 2] — Stripe billing integration — USD/EUR/SGD/GBP customer + subscription + per-perio

## Layer 10 (5 FRs — buildable in parallel)

- **FR-CRM-010** [MUST, 5h, slice 7] — CRM vn-vat-invoice skill — Decree 123 hóa đơn auto-emit on deal.stage=won + invo
- **FR-INV-008** [MUST, 5h, slice 2] — INV VN hóa đơn cancellation flow — Decree 123 Art. 19 replacement-or-cancellatio
- **FR-INV-010** [MUST, 5h, slice 2] — INV CUO dunning draft — auto-generate polite/firm/legal-warning email drafts per
- **FR-TEN-101** [MUST, 10h, slice 1] — Self-serve signup form ≤ 30 s end-to-end — email OTP + slug + plan + currency + 
- **FR-TEN-102** [MUST, 12h, slice 2] — VND domestic billing rail — VnPay + Momo + ZaloPay subscription, recurring-charg

## Layer 11 (4 FRs — buildable in parallel)

- **FR-PORTAL-001** [MUST, 12h, slice 1] — PORTAL scoped read-only views — PROJ/INV/DOC/CHAT filtered by Engagement members
- **FR-PORTAL-002** [MUST, 8h, slice 1] — PORTAL per-tenant brand pack — logo + colour palette + custom CNAME + email temp
- **FR-TEN-107** [SHOULD, 16h, slice 3] — TEN tenant-admin SPA — seats + billing + audit + residency + retention dashboard
- **FR-TIME-008** [MUST, 8h, slice 2] — TIME expense capture — photo → AWS Textract OCR → hóa đơn parser → Member confir

## Layer 12 (2 FRs — buildable in parallel)

- **FR-PORTAL-007** [SHOULD, 6h, slice 2] — PORTAL PWA installable — mobile-first Progressive Web App with offline-capable v
- **FR-PORTAL-008** [MUST, 5h, slice 2] — PORTAL DSAR self-service — GDPR Art. 15 + PDPL Art. 17 client-initiated data sub

---

# Appendix §C — Sprint plan (effort rollup)

_Generated 2026-05-17 — 241 FRs, 1,791 total engineering-hours. Was §3 of the former `REPORTS.md`._

## §3 source detail — Sprint plan (full)



_Generated 2026-05-17 — 241 FRs, 1791 total engineering-hours._

## Headline numbers

- **Total scope:** 241 FRs, 1791h (224 engineer-days @ 8h/d, or 11.2 engineer-months @ 160h/m).
- **At 3 engineers (480h/sprint @ 2-week sprints):** 3.7 sprints (~7.5 weeks).
- **At 5 engineers (800h/sprint):** 2.2 sprints (~4.5 weeks).

## By module

| Module | FRs | Total hours | Slices |
|---|---:|---:|---|
| **AI** | 23 | 175 | 1, 2, 3, 4, 5 |
| **AUTH** | 15 | 127 | 1 |
| **memory** | 11 | 136 | 1, 2 |
| **CHAT** | 12 | 95 | 1, 2 |
| **CRM** | 10 | 52 | 1, 5, 6, 7 |
| **CUO** | 5 | 37 | 2, 6 |
| **DOC** | 11 | 103 | 1, 2, 3 |
| **DOCS** | 1 | 14 | 1 |
| **EMAIL** | 11 | 85 | 1, 2 |
| **ESOP** | 7 | 38 | 1, 2 |
| **HR** | 9 | 52 | 1, 6, 7 |
| **INV** | 11 | 67 | 1, 2 |
| **KB** | 9 | 49 | 1, 4, 5 |
| **LEARN** | 7 | 40 | 7 |
| **MCP** | 8 | 56 | 2, 3, 4 |
| **OBS** | 9 | 82 | 1, 2, 3 |
| **OKR** | 7 | 42 | 1, 3 |
| **PORTAL** | 8 | 61 | 1, 2 |
| **PROJ** | 18 | 128 | 1, 2, 3 |
| **RES** | 5 | 38 | 7, 8 |
| **REW** | 10 | 55 | 1, 2 |
| **SKILL** | 11 | 84 | 1, 2, 3 |
| **TEN** | 14 | 124 | 1, 2, 3 |
| **TIME** | 9 | 51 | 1, 2 |

## By module & slice (sprint chunks)

### AI

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 6 | 39 | FR-AI-001, FR-AI-002, FR-AI-003, FR-AI-004, FR-AI-005, FR-AI-104 |
| 2 | 5 | 34 | FR-AI-006, FR-AI-007, FR-AI-008, FR-AI-009, FR-AI-010 |
| 3 | 5 | 38 | FR-AI-011, FR-AI-012, FR-AI-013, FR-AI-014, FR-AI-015 |
| 4 | 5 | 42 | FR-AI-016, FR-AI-017, FR-AI-018, FR-AI-019, FR-AI-020 |
| 5 | 2 | 22 | FR-AI-021, FR-AI-022 |

### AUTH

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 15 | 127 | FR-AUTH-001, FR-AUTH-002, FR-AUTH-003, FR-AUTH-004, FR-AUTH-005, FR-AUTH-006, FR-AUTH-101, FR-AUTH-102, FR-AUTH-103, FR-AUTH-104, FR-AUTH-105, FR-AUTH-106, FR-AUTH-107, FR-AUTH-108, FR-AUTH-109 |

### memory

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 4 | 52 | FR-MEMORY-101, FR-MEMORY-102, FR-MEMORY-103, FR-MEMORY-106 |
| 2 | 7 | 84 | FR-MEMORY-104, FR-MEMORY-105, FR-MEMORY-107, FR-MEMORY-108, FR-MEMORY-109, FR-MEMORY-110, FR-MEMORY-111 |

### CHAT

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 5 | 46 | FR-CHAT-001, FR-CHAT-002, FR-CHAT-003, FR-CHAT-004, FR-CHAT-005 |
| 2 | 7 | 49 | FR-CHAT-006, FR-CHAT-007, FR-CHAT-008, FR-CHAT-009, FR-CHAT-010, FR-CHAT-011, FR-CHAT-012 |

### CRM

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-CRM-001 |
| 5 | 3 | 18 | FR-CRM-002, FR-CRM-003, FR-CRM-004 |
| 6 | 3 | 16 | FR-CRM-005, FR-CRM-006, FR-CRM-007 |
| 7 | 3 | 12 | FR-CRM-008, FR-CRM-009, FR-CRM-010 |

### CUO

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 2 | 1 | 12 | FR-CUO-101 |
| 6 | 4 | 25 | FR-CUO-102, FR-CUO-103, FR-CUO-104, FR-CUO-105 |

### DOC

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 4 | 23 | FR-DOC-001, FR-DOC-007, FR-DOC-008, FR-DOC-009 |
| 2 | 2 | 18 | FR-DOC-005, FR-DOC-006 |
| 3 | 5 | 62 | FR-DOC-002, FR-DOC-003, FR-DOC-004, FR-DOC-010, FR-DOC-011 |

### DOCS

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 14 | FR-DOCS-001 |

### EMAIL

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 7 | 44 | FR-EMAIL-001, FR-EMAIL-002, FR-EMAIL-004, FR-EMAIL-006, FR-EMAIL-007, FR-EMAIL-009, FR-EMAIL-010 |
| 2 | 4 | 41 | FR-EMAIL-003, FR-EMAIL-005, FR-EMAIL-008, FR-EMAIL-011 |

### ESOP

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 14 | FR-ESOP-001, FR-ESOP-002, FR-ESOP-003 |
| 2 | 4 | 24 | FR-ESOP-004, FR-ESOP-005, FR-ESOP-006, FR-ESOP-007 |

### HR

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-HR-001 |
| 6 | 6 | 32 | FR-HR-002, FR-HR-003, FR-HR-004, FR-HR-005, FR-HR-006, FR-HR-007 |
| 7 | 2 | 14 | FR-HR-008, FR-HR-009 |

### INV

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 20 | FR-INV-001, FR-INV-002, FR-INV-004 |
| 2 | 8 | 47 | FR-INV-003, FR-INV-005, FR-INV-006, FR-INV-007, FR-INV-008, FR-INV-009, FR-INV-010, FR-INV-011 |

### KB

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-KB-001 |
| 4 | 2 | 10 | FR-KB-002, FR-KB-003 |
| 5 | 6 | 33 | FR-KB-004, FR-KB-005, FR-KB-006, FR-KB-007, FR-KB-008, FR-KB-009 |

### LEARN

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 7 | 7 | 40 | FR-LEARN-001, FR-LEARN-002, FR-LEARN-003, FR-LEARN-004, FR-LEARN-005, FR-LEARN-006, FR-LEARN-007 |

### MCP

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 2 | 5 | 28 | FR-MCP-002, FR-MCP-003, FR-MCP-004, FR-MCP-005, FR-MCP-006 |
| 3 | 2 | 16 | FR-MCP-007, FR-MCP-008 |
| 4 | 1 | 12 | FR-MCP-001 |

### OBS

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 30 | FR-OBS-001, FR-OBS-002, FR-OBS-003 |
| 2 | 3 | 20 | FR-OBS-004, FR-OBS-005, FR-OBS-006 |
| 3 | 3 | 32 | FR-OBS-007, FR-OBS-008, FR-OBS-009 |

### OKR

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-OKR-001 |
| 3 | 6 | 36 | FR-OKR-002, FR-OKR-003, FR-OKR-004, FR-OKR-005, FR-OKR-006, FR-OKR-007 |

### PORTAL

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 30 | FR-PORTAL-001, FR-PORTAL-002, FR-PORTAL-003 |
| 2 | 5 | 31 | FR-PORTAL-004, FR-PORTAL-005, FR-PORTAL-006, FR-PORTAL-007, FR-PORTAL-008 |

### PROJ

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 2 | 19 | FR-PROJ-001, FR-PROJ-002 |
| 2 | 7 | 41 | FR-PROJ-003, FR-PROJ-004, FR-PROJ-005, FR-PROJ-006, FR-PROJ-007, FR-PROJ-008, FR-PROJ-009 |
| 3 | 9 | 68 | FR-PROJ-010, FR-PROJ-011, FR-PROJ-012, FR-PROJ-013, FR-PROJ-014, FR-PROJ-015, FR-PROJ-016, FR-PROJ-017, FR-PROJ-018 |

### RES

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 7 | 1 | 10 | FR-RES-001 |
| 8 | 4 | 28 | FR-RES-002, FR-RES-003, FR-RES-004, FR-RES-005 |

### REW

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 5 | 25 | FR-REW-001, FR-REW-002, FR-REW-003, FR-REW-004, FR-REW-010 |
| 2 | 5 | 30 | FR-REW-005, FR-REW-006, FR-REW-007, FR-REW-008, FR-REW-009 |

### SKILL

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 6 | 46 | FR-SKILL-101, FR-SKILL-102, FR-SKILL-103, FR-SKILL-104, FR-SKILL-107, FR-SKILL-201 |
| 2 | 1 | 9 | FR-SKILL-105 |
| 3 | 4 | 29 | FR-SKILL-106, FR-SKILL-108, FR-SKILL-109, FR-SKILL-110 |

### TEN

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 7 | 60 | FR-TEN-001, FR-TEN-002, FR-TEN-004, FR-TEN-101, FR-TEN-104, FR-TEN-201, FR-TEN-202 |
| 2 | 6 | 48 | FR-TEN-003, FR-TEN-005, FR-TEN-102, FR-TEN-103, FR-TEN-105, FR-TEN-106 |
| 3 | 1 | 16 | FR-TEN-107 |

### TIME

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 7 | 37 | FR-TIME-001, FR-TIME-002, FR-TIME-003, FR-TIME-005, FR-TIME-006, FR-TIME-007, FR-TIME-009 |
| 2 | 2 | 14 | FR-TIME-004, FR-TIME-008 |

---

# Appendix §D — Migration audit (per-module SQL)

_Generated 2026-05-17 — scanned all FR `build_envelope.new_files` for `services/<module>/migrations/<N>_<name>.sql` patterns. Was §4 of the former `REPORTS.md`._

## §4 source detail — Migration audit (full)



_Generated 2026-05-17 — scanned all FR `build_envelope.new_files` for `services/<module>/migrations/<N>_<name>.sql` patterns._

## Summary

- Total modules with migrations: **23**
- Total migration files declared: **327**

### `ai`

- Total unique migrations: **1**
- Sequence range: `0010` → `0010`
- ⚠️ **Gaps in sequence**: [1, 2, 3, 4, 5, 6, 7, 8, 9]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0010 | `vn_provider_creds` | FR-AI-104 |

### `auth`

- Total unique migrations: **29**
- Sequence range: `0001` → `0026`
- ⚠️ **Gaps in sequence**: [8, 9]
- ⚠️ **Duplicate seq with different names**:
  - `0005`: ['rls_enable_on_tables', 'roles_permissions']
  - `0006`: ['role_catalogue_version', 'signing_keys']
  - `0015`: ['auth_token_refresh_log', 'hibp_audit']
  - `0016`: ['login_history_geo', 'mfa_factors']
  - `0017`: ['mfa_factor_history', 'travel_audit']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `tenants` | FR-AUTH-001 |
| 0002 | `admin_idempotency` | FR-AUTH-001 |
| 0003 | `subjects` | FR-AUTH-002 |
| 0004 | `rls_roles` | FR-AUTH-003 |
| 0005 | `rls_enable_on_tables` | FR-AUTH-003 |
| 0005 | `roles_permissions` | FR-AUTH-101 |
| 0006 | `role_catalogue_version` | FR-AUTH-101 |
| 0006 | `signing_keys` | FR-AUTH-004 |
| 0007 | `sessions` | FR-AUTH-005 |
| 0010 | `oidc_idp_configs` | FR-AUTH-104 |
| 0011 | `oidc_login_history` | FR-AUTH-104 |
| 0012 | `oidc_subject_link` | FR-AUTH-104 |
| 0013 | `lumi_token_issuance_log` | FR-AUTH-108 |
| 0014 | `auth_migration_state` | FR-AUTH-109 |
| 0015 | `auth_token_refresh_log` | FR-AUTH-109 |
| 0015 | `hibp_audit` | FR-AUTH-107 |
| 0016 | `login_history_geo` | FR-AUTH-106 |
| 0016 | `mfa_factors` | FR-AUTH-102 |
| 0017 | `mfa_factor_history` | FR-AUTH-102 |
| 0017 | `travel_audit` | FR-AUTH-106 |
| 0018 | `mfa_challenge_log` | FR-AUTH-102 |
| 0019 | `mfa_recovery_codes` | FR-AUTH-102 |
| 0020 | `mfa_lockout_state` | FR-AUTH-102 |
| 0021 | `saml_idp_configs` | FR-AUTH-103 |
| 0022 | `saml_login_history` | FR-AUTH-103 |
| 0023 | `saml_authn_request_log` | FR-AUTH-103 |
| 0024 | `saml_subject_link` | FR-AUTH-103 |
| 0025 | `passkey_enrolment_state` | FR-AUTH-105 |
| 0026 | `passkey_lifecycle_log` | FR-AUTH-105 |

### `memory`

- Total unique migrations: **3**
- Sequence range: `0001` → `0003`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `layer2` | FR-MEMORY-101 |
| 0002 | `layer2_cursor` | FR-MEMORY-101 |
| 0003 | `pgroonga` | FR-MEMORY-108 |

### `crm`

- Total unique migrations: **15**
- Sequence range: `0001` → `0010`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['activity_feed', 'contacts']
  - `0003`: ['pipelines_stages', 'vn_account_fields']
  - `0004`: ['deal_conversion', 'deals']
  - `0005`: ['deal_status_history', 'next_action_suggestions']
  - `0006`: ['lead_scoring', 'seed_pipelines']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `accounts` | FR-CRM-001 |
| 0002 | `activity_feed` | FR-CRM-002 |
| 0002 | `contacts` | FR-CRM-001 |
| 0003 | `pipelines_stages` | FR-CRM-001 |
| 0003 | `vn_account_fields` | FR-CRM-003 |
| 0004 | `deal_conversion` | FR-CRM-004 |
| 0004 | `deals` | FR-CRM-001 |
| 0005 | `deal_status_history` | FR-CRM-001 |
| 0005 | `next_action_suggestions` | FR-CRM-005 |
| 0006 | `lead_scoring` | FR-CRM-006 |
| 0006 | `seed_pipelines` | FR-CRM-001 |
| 0007 | `win_loss_drafts` | FR-CRM-007 |
| 0008 | `mst_validation` | FR-CRM-008 |
| 0009 | `tenant_bank_config` | FR-CRM-009 |
| 0010 | `vat_invoice_emissions` | FR-CRM-010 |

### `cuo`

- Total unique migrations: **4**
- Sequence range: `0002` → `0005`
- ⚠️ **Gaps in sequence**: [1]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0002 | `langgraph_checkpoints` | FR-CUO-102 |
| 0003 | `trace_rows` | FR-CUO-103 |
| 0004 | `chain_walks` | FR-CUO-104 |
| 0005 | `chain_rollbacks` | FR-CUO-105 |

### `doc`

- Total unique migrations: **12**
- Sequence range: `0001` → `0011`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['document_audit_log', 'lifecycle_metadata']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `document_metadata` | FR-DOC-001 |
| 0002 | `document_audit_log` | FR-DOC-001 |
| 0002 | `lifecycle_metadata` | FR-DOC-007 |
| 0003 | `expiry_alerts` | FR-DOC-008 |
| 0004 | `renewal_drafts` | FR-DOC-009 |
| 0005 | `identity_verifications` | FR-DOC-006 |
| 0006 | `signing_workflows` | FR-DOC-005 |
| 0007 | `third_party_imports` | FR-DOC-010 |
| 0008 | `qtsp_signatures` | FR-DOC-002 |
| 0009 | `aatl_signatures` | FR-DOC-003 |
| 0010 | `vn_ca_signatures` | FR-DOC-004 |
| 0011 | `ltv_operations` | FR-DOC-011 |

### `email`

- Total unique migrations: **16**
- Sequence range: `0001` → `0012`
- ⚠️ **Duplicate seq with different names**:
  - `0001`: ['email_auth_log', 'messages']
  - `0002`: ['bounce_log', 'tenant_dkim_keys']
  - `0003`: ['dkim_keys', 'tenant_dns_setup']
  - `0004`: ['outbound_messages', 'residency_routing']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `email_auth_log` | FR-EMAIL-002 |
| 0001 | `messages` | FR-EMAIL-001 |
| 0002 | `bounce_log` | FR-EMAIL-001 |
| 0002 | `tenant_dkim_keys` | FR-EMAIL-004 |
| 0003 | `dkim_keys` | FR-EMAIL-001 |
| 0003 | `tenant_dns_setup` | FR-EMAIL-004 |
| 0004 | `outbound_messages` | FR-EMAIL-009 |
| 0004 | `residency_routing` | FR-EMAIL-001 |
| 0005 | `suppression_list` | FR-EMAIL-009 |
| 0006 | `bulk_sends` | FR-EMAIL-010 |
| 0007 | `dsar_export_jobs` | FR-EMAIL-011 |
| 0008 | `tracked_domains` | FR-EMAIL-006 |
| 0009 | `message_issue_link` | FR-EMAIL-007 |
| 0010 | `genie_sessions` | FR-EMAIL-008 |
| 0011 | `camel_audit` | FR-EMAIL-005 |
| 0012 | `thread_state` | FR-EMAIL-003 |

### `esop`

- Total unique migrations: **7**
- Sequence range: `0001` → `0007`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `sp_grants` | FR-ESOP-001 |
| 0002 | `vesting_accruals` | FR-ESOP-002 |
| 0003 | `annual_valuations` | FR-ESOP-003 |
| 0004 | `put_options` | FR-ESOP-004 |
| 0005 | `leaver_outcomes` | FR-ESOP-005 |
| 0006 | `ma_events` | FR-ESOP-006 |
| 0007 | `dashboard_access_log` | FR-ESOP-007 |

### `hr`

- Total unique migrations: **11**
- Sequence range: `0001` → `0009`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['contract_types', 'member_status_history']
  - `0003`: ['cccd_storage', 'member_view']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `members` | FR-HR-001 |
| 0002 | `contract_types` | FR-HR-002 |
| 0002 | `member_status_history` | FR-HR-001 |
| 0003 | `cccd_storage` | FR-HR-003 |
| 0003 | `member_view` | FR-HR-001 |
| 0004 | `leave_requests` | FR-HR-004 |
| 0005 | `policy_constants` | FR-HR-005 |
| 0006 | `leave_accrual_ledger` | FR-HR-006 |
| 0007 | `perf_snapshots` | FR-HR-008 |
| 0008 | `terminations` | FR-HR-009 |
| 0009 | `onboarding_sagas` | FR-HR-007 |

### `inv`

- Total unique migrations: **14**
- Sequence range: `0001` → `0021`
- ⚠️ **Gaps in sequence**: [7, 8, 9, 16, 17, 18, 19]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `invoices` | FR-INV-001 |
| 0002 | `invoice_lines` | FR-INV-001 |
| 0003 | `invoice_status_history` | FR-INV-001 |
| 0004 | `invoice_number_sequence` | FR-INV-001 |
| 0005 | `rate_card_snapshot` | FR-INV-001 |
| 0006 | `fx_rates` | FR-INV-002 |
| 0010 | `payment_receipts` | FR-INV-005 |
| 0011 | `webhook_secrets` | FR-INV-005 |
| 0012 | `stripe_event_log` | FR-INV-003 |
| 0013 | `stripe_webhook_secrets` | FR-INV-003 |
| 0014 | `payment_allocations` | FR-INV-006 |
| 0015 | `invoice_outstanding_view` | FR-INV-006 |
| 0020 | `wise_webhook_events` | FR-INV-004 |
| 0021 | `wise_unmatched_receipts` | FR-INV-004 |

### `invoicing`

- Total unique migrations: **4**
- Sequence range: `0007` → `0010`
- ⚠️ **Gaps in sequence**: [1, 2, 3, 4, 5, 6]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0007 | `vn_hoadon` | FR-INV-007 |
| 0008 | `vn_hoadon_cancellation` | FR-INV-008 |
| 0009 | `dunning_drafts` | FR-INV-010 |
| 0010 | `recognition` | FR-INV-011 |

### `kb`

- Total unique migrations: **10**
- Sequence range: `0001` → `0009`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['document_views', 'render_cache']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `documents` | FR-KB-001 |
| 0002 | `document_views` | FR-KB-001 |
| 0002 | `render_cache` | FR-KB-002 |
| 0003 | `permissions_share_links` | FR-KB-003 |
| 0004 | `pgroonga_fts5_index` | FR-KB-004 |
| 0005 | `semantic_chunks` | FR-KB-005 |
| 0006 | `rerank_cache` | FR-KB-006 |
| 0007 | `qa_questions` | FR-KB-007 |
| 0008 | `runbook_tags` | FR-KB-008 |
| 0009 | `translation_link` | FR-KB-009 |

### `learn`

- Total unique migrations: **7**
- Sequence range: `0001` → `0007`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `skill_tree_mastery` | FR-LEARN-001 |
| 0002 | `evidence` | FR-LEARN-002 |
| 0003 | `vp_snapshots` | FR-LEARN-003 |
| 0004 | `councils` | FR-LEARN-004 |
| 0005 | `disclosure_log` | FR-LEARN-005 |
| 0006 | `promotions` | FR-LEARN-006 |
| 0007 | `vp_rew_handoffs` | FR-LEARN-007 |

### `mcp`

- Total unique migrations: **9**
- Sequence range: `0002` → `0012`
- ⚠️ **Gaps in sequence**: [1, 3, 4]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0002 | `server_heartbeats` | FR-MCP-002 |
| 0005 | `prm_drift_log` | FR-MCP-005 |
| 0006 | `mcp_gating_policy` | FR-MCP-006 |
| 0007 | `mcp_pending_confirmations` | FR-MCP-006 |
| 0008 | `mcp_gating_decisions_log` | FR-MCP-006 |
| 0009 | `mcp_tasks` | FR-MCP-007 |
| 0010 | `mcp_task_checkpoints` | FR-MCP-007 |
| 0011 | `mcp_task_progress_events` | FR-MCP-007 |
| 0012 | `mcp_elicitations` | FR-MCP-008 |

### `metering`

- Total unique migrations: **4**
- Sequence range: `0001` → `0003`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['metering_holds_index', 'metering_periods']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `metering_events` | FR-TEN-004 |
| 0002 | `metering_holds_index` | FR-TEN-004 |
| 0002 | `metering_periods` | FR-TEN-004 |
| 0003 | `metering_aggregates_view` | FR-TEN-004 |

### `okr`

- Total unique migrations: **12**
- Sequence range: `0001` → `0007`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['kr_types', 'teams']
  - `0003`: ['objectives', 'progress_source']
  - `0004`: ['auto_progress_runs', 'key_results']
  - `0005`: ['progress_log', 'weekly_checkins']
  - `0006`: ['monday_digests', 'objective_status_history']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `cycles` | FR-OKR-001 |
| 0002 | `kr_types` | FR-OKR-002 |
| 0002 | `teams` | FR-OKR-001 |
| 0003 | `objectives` | FR-OKR-001 |
| 0003 | `progress_source` | FR-OKR-003 |
| 0004 | `auto_progress_runs` | FR-OKR-004 |
| 0004 | `key_results` | FR-OKR-001 |
| 0005 | `progress_log` | FR-OKR-001 |
| 0005 | `weekly_checkins` | FR-OKR-005 |
| 0006 | `monday_digests` | FR-OKR-006 |
| 0006 | `objective_status_history` | FR-OKR-001 |
| 0007 | `quarterly_retros` | FR-OKR-007 |

### `portal`

- Total unique migrations: **21**
- Sequence range: `0001` → `0021`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `portal_idp_configs` | FR-PORTAL-003 |
| 0002 | `portal_scim_audit_log` | FR-PORTAL-003 |
| 0003 | `portal_idp_groups_map` | FR-PORTAL-003 |
| 0004 | `portal_scim_tokens` | FR-PORTAL-003 |
| 0005 | `portal_brand_packs` | FR-PORTAL-002 |
| 0006 | `portal_brand_pack_active` | FR-PORTAL-002 |
| 0007 | `portal_brand_assets` | FR-PORTAL-002 |
| 0008 | `portal_cname_configs` | FR-PORTAL-002 |
| 0009 | `portal_deprovision_log` | FR-PORTAL-004 |
| 0010 | `portal_jwt_blacklist` | FR-PORTAL-004 |
| 0011 | `portal_restore_requests` | FR-PORTAL-004 |
| 0012 | `portal_genie_sessions` | FR-PORTAL-005 |
| 0013 | `portal_genie_messages` | FR-PORTAL-005 |
| 0014 | `portal_view_definitions` | FR-PORTAL-001 |
| 0015 | `portal_view_read_log` | FR-PORTAL-001 |
| 0016 | `portal_dsar_requests` | FR-PORTAL-008 |
| 0017 | `portal_dsar_denials` | FR-PORTAL-008 |
| 0018 | `portal_workflow_submissions` | FR-PORTAL-006 |
| 0019 | `portal_workflow_routing_rules` | FR-PORTAL-006 |
| 0020 | `portal_pwa_subscriptions` | FR-PORTAL-007 |
| 0021 | `portal_pwa_notifications_log` | FR-PORTAL-007 |

### `proj`

- Total unique migrations: **5**
- Sequence range: `0001` → `0010`
- ⚠️ **Gaps in sequence**: [5, 6, 7, 8, 9]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `engagements` | FR-PROJ-001 |
| 0002 | `cycles` | FR-PROJ-001 |
| 0003 | `issues` | FR-PROJ-001 |
| 0004 | `issue_links` | FR-PROJ-001 |
| 0010 | `issues_addendum` | FR-TIME-001 |

### `res`

- Total unique migrations: **5**
- Sequence range: `0001` → `0005`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `capacity_demand_matrix` | FR-RES-001 |
| 0002 | `allocation_changes` | FR-RES-002 |
| 0003 | `allocation_flags` | FR-RES-003 |
| 0004 | `hiring_memos` | FR-RES-004 |
| 0005 | `ot_consent` | FR-RES-005 |

### `rew`

- Total unique migrations: **9**
- Sequence range: `0001` → `0009`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `comp_schema` | FR-REW-001 |
| 0002 | `param_versions` | FR-REW-002 |
| 0003 | `p1_protection` | FR-REW-003 |
| 0004 | `deductions` | FR-REW-004 |
| 0005 | `payroll_runs` | FR-REW-005 |
| 0006 | `payslip_pdfs` | FR-REW-006 |
| 0007 | `bp_ledger` | FR-REW-007 |
| 0008 | `p3_distributions` | FR-REW-008 |
| 0009 | `payroll_batches` | FR-REW-009 |

### `skill`

- Total unique migrations: **1**
- Sequence range: `0010` → `0010`
- ⚠️ **Gaps in sequence**: [1, 2, 3, 4, 5, 6, 7, 8, 9]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0010 | `oci_bundles` | FR-SKILL-201 |

### `ten`

- Total unique migrations: **33**
- Sequence range: `0001` → `0029`
- ⚠️ **Duplicate seq with different names**:
  - `0004`: ['plan_tier', 'tenant_offboarding_state']
  - `0005`: ['plan_history', 'tenant_offboarding_log']
  - `0010`: ['holdco_flips', 'stripe_price_map']
  - `0011`: ['hostile_overrides', 'signup_sessions']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `tenants` | FR-TEN-001 |
| 0002 | `tenant_status_history` | FR-TEN-001 |
| 0003 | `tenant_residency_map` | FR-TEN-001 |
| 0004 | `plan_tier` | FR-TEN-002 |
| 0004 | `tenant_offboarding_state` | FR-TEN-104 |
| 0005 | `plan_history` | FR-TEN-002 |
| 0005 | `tenant_offboarding_log` | FR-TEN-104 |
| 0006 | `stripe_billing` | FR-TEN-003 |
| 0007 | `stripe_api_calls` | FR-TEN-003 |
| 0008 | `stripe_event_dispatch_log` | FR-TEN-003 |
| 0009 | `billing_currency_enum` | FR-TEN-003 |
| 0010 | `holdco_flips` | FR-TEN-201 |
| 0010 | `stripe_price_map` | FR-TEN-003 |
| 0011 | `hostile_overrides` | FR-TEN-202 |
| 0011 | `signup_sessions` | FR-TEN-101 |
| 0012 | `tenant_consents` | FR-TEN-101 |
| 0013 | `signup_rate_limits` | FR-TEN-101 |
| 0014 | `disposable_email_domains` | FR-TEN-101 |
| 0015 | `residency_enum` | FR-TEN-103 |
| 0016 | `residency_trip_wire` | FR-TEN-103 |
| 0017 | `residency_health_log` | FR-TEN-103 |
| 0018 | `vnd_payment_tokens` | FR-TEN-102 |
| 0019 | `vnd_psp_credentials` | FR-TEN-102 |
| 0020 | `vnd_invoices` | FR-TEN-102 |
| 0021 | `vnd_invoice_sequence` | FR-TEN-102 |
| 0022 | `vnd_event_dispatch_log` | FR-TEN-102 |
| 0023 | `vertical_pack_installs` | FR-TEN-005 |
| 0024 | `vertical_pack_price_catalog` | FR-TEN-005 |
| 0025 | `vertical_pack_overrides` | FR-TEN-005 |
| 0026 | `tenant_bundle_exports` | FR-TEN-105 |
| 0027 | `tenant_signing_keys` | FR-TEN-105 |
| 0028 | `permanent_delete_attestations` | FR-TEN-106 |
| 0029 | `permanent_delete_cascade_log` | FR-TEN-106 |

### `time`

- Total unique migrations: **11**
- Sequence range: `0001` → `0010`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['time_entries_view', 'timers']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `time_entries` | FR-TIME-001 |
| 0002 | `time_entries_view` | FR-TIME-001 |
| 0002 | `timers` | FR-TIME-002 |
| 0003 | `vn_ot_tracking` | FR-TIME-007 |
| 0004 | `billable_defaults` | FR-TIME-005 |
| 0005 | `timesheets` | FR-TIME-006 |
| 0006 | `timesheet_reviews` | FR-TIME-006 |
| 0007 | `rollup_cache` | FR-TIME-009 |
| 0008 | `time_proposals` | FR-TIME-004 |
| 0009 | `expenses` | FR-TIME-008 |
| 0010 | `expense_policies` | FR-TIME-008 |

---

**Total issues found:** 18

**Interpretation**: Gaps may indicate planned but un-numbered migrations; duplicates with different names indicate two FRs claim the same sequence (must reconcile).

---

*End of backlog v0.4.0 — 2026-05-18 (full corpus + deploy roadmap + four absorbed appendices with full source detail).*
