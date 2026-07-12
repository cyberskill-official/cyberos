# memory module — feature request index

_Generated 2026-05-17 — 11 FRs, 136 engineering-hours total._
_Updated 2026-05-19 — +9 FRs (FR-MEMORY-112..120) from the MEMORY Improvement Wave 2026 Q3 (Anthropic Memory+Dreaming alignment + Ramakrushna agentic-memory taxonomy). Catalog: 20 FRs, ~266 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-MEMORY-101](FR-MEMORY-101-layer2-ingest-pipeline/spec.md) | MUST | 1 | 18 | Layer-2 ingest pipeline (binlog → pgvector + Apache AGE) — chain-anchor verification + 1s p95 lag +  |
| [FR-MEMORY-102](FR-MEMORY-102-layer2-rebuild-ci-gate/spec.md) | MUST | 1 | 10 | Layer-2 rebuild-from-Layer-1 CI gate — deterministic rebuild + spot-check + 30min budget + mid-rebui |
| [FR-MEMORY-103](FR-MEMORY-103-multi-device-sync/spec.md) | MUST | 1 | 18 | memory-sync daemon — laptop A ↔ Cloud memory ↔ laptop B with sync_class gating + CRDT conflict + 10K o |
| [FR-MEMORY-104](FR-MEMORY-104-tauri-app/spec.md) | SHOULD | 2 | 28 | Tauri 2.x desktop app — macOS + Windows + Linux signed/notarised + auto-update + tray + quick captur |
| [FR-MEMORY-105](FR-MEMORY-105-doctor-watched-folders-invariants/spec.md) | MUST | 2 | 7 | cyberos doctor — watched-folders integrity invariants (manifest ↔ filesystem ↔ HEAD reconciliation;  |
| [FR-MEMORY-106](FR-MEMORY-106-sync-class-enforcement/spec.md) | MUST | 1 | 6 | memory sync_class enforcement — private vs shareable + ACL filtering + structural compensation exclus |
| [FR-MEMORY-107](FR-MEMORY-107-fs-watcher/spec.md) | MUST | 2 | 14 | memory capture daemon — Rust + notify crate FS watcher with rate-limit + content-dedup + backpressure |
| [FR-MEMORY-108](FR-MEMORY-108-search-api/spec.md) | MUST | 2 | 12 | memory search — vector + graph + full-text in parallel + RRF fusion + BGE-rerank + RLS + ACL + chain_ |
| [FR-MEMORY-109](FR-MEMORY-109-claude-code-hook-capture/spec.md) | MUST | 2 | 8 | Claude Code hook capture — UserPromptSubmit + PostToolUse + Stop hooks emit memory memories with prom |
| [FR-MEMORY-110](FR-MEMORY-110-capture-daemon-health-restart/spec.md) | MUST | 2 | 6 | memory capture daemon supervision — systemd + launchd units + /healthz + watchdog + crash-restart wit |
| [FR-MEMORY-111](FR-MEMORY-111-pre-ingest-pii-detection/spec.md) | MUST | 2 | 9 | memory pre-ingest PII detection — Presidio EN + custom VN recognisers; ≥ 99.5% held-back recall on la |
| [FR-MEMORY-112](FR-MEMORY-112-episodic-memory/spec.md) | MUST | 3 | 12 | memory episodic memory — `kind: episode` + `cyberos recall-similar`; reflection-loop foundation (Wave |
| [FR-MEMORY-113](FR-MEMORY-113-recency-decay-recall/spec.md) | MUST | 3 | 8 | memory recall ranking — Park-et-al combined score (relevance · 0.4 + importance · 0.3 + recency · 0.3 |
| [FR-MEMORY-114](FR-MEMORY-114-write-time-importance/spec.md) | SHOULD | 3 | 8 | memory write-time importance scoring — Haiku-rated `meta.importance` filters noise at the source; rea |
| [FR-MEMORY-115](FR-MEMORY-115-cyberos-dream/spec.md) | SHOULD | 3 | 32 | memory dreaming — `cyberos dream` out-of-band batch reflection (4 detectors: duplicates / stale / new |
| [FR-MEMORY-116](FR-MEMORY-116-semantic-dedup-consolidate/spec.md) | SHOULD | 3 | 6 | memory consolidate — semantic-dedup phase (Walk → Compact → Sign → Publish → SemanticDedup); shares F |
| [FR-MEMORY-117](FR-MEMORY-117-per-store-acl/spec.md) | SHOULD | 3 | 24 | memory per-store ACL — `STORE.yaml` per top-level subtree; writer enforces ACL on writes; requires AG |
| [FR-MEMORY-118](FR-MEMORY-118-put-if-precondition/spec.md) | SHOULD | 3 | 8 | memory put_if — optimistic-concurrency primitive with content-hash preconditions; requires AGENTS.md  |
| [FR-MEMORY-119](FR-MEMORY-119-session-transcript-ledger/spec.md) | SHOULD | 3 | 24 | memory session transcript ledger — opt-in `cyberos session {start,append,end}`; default classificatio |
| [FR-MEMORY-120](FR-MEMORY-120-cyberos-history/spec.md) | SHOULD | 3 | 8 | memory history — `cyberos history <path>` surfaces per-file version + attribution from the audit chai |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-MEMORY-101→FR-AI-019, FR-MEMORY-111→FR-AI-012
- **AUTH**: FR-MEMORY-101→FR-AUTH-003

**This module is depended on by:**

- **KB**: FR-KB-007→FR-MEMORY-108
- **PROJ**: FR-PROJ-008→FR-MEMORY-101
- **TEN**: FR-TEN-004→FR-MEMORY-111

## Wave 2026 Q3 — Self-learning agents (MEMORY Improvement)

Authored 2026-05-19 in response to the Anthropic Memory+Dreaming talk + Ramakrushna agentic-memory article (source materials at `../../../playground/extracts/`). The 9-FR wave splits into three sub-waves with explicit ship sequencing:

- **Wave 1 (4 dev-days)** — high-leverage, low-protocol-impact: FR-MEMORY-112 (episodic) + FR-MEMORY-113 (recency decay) + FR-MEMORY-114 (importance scoring).
- **Wave 2 (9 dev-days, gated on `APPROVE protocol change P19 §7.7`)** — the headline `cyberos dream` out-of-band reflection pipeline: FR-MEMORY-115 + FR-MEMORY-116.
- **Wave 3 (8 dev-days, gated on `APPROVE protocol change P20 §14.4` / `P21 §3.1` / `P22 §18`)** — multi-agent scale: FR-MEMORY-117 (per-store ACL) + FR-MEMORY-118 (put_if precondition) + FR-MEMORY-119 (session transcript ledger) + FR-MEMORY-120 (`cyberos history`).

Each protocol-amendment-bearing FR carries its own approval chat-turn (per Stephen's 2026-05-19 decision: one-at-a-time, not bundled).

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._