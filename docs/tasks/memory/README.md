# memory module — task index

_Generated 2026-05-17 — 11 FRs, 136 engineering-hours total._
_Updated 2026-05-19 — +9 FRs (TASK-MEMORY-112..120) from the MEMORY Improvement Wave 2026 Q3 (Anthropic Memory+Dreaming alignment + Ramakrushna agentic-memory taxonomy). Catalog: 20 FRs, ~266 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-MEMORY-101](TASK-MEMORY-101-layer2-ingest-pipeline/spec.md) | MUST | 1 | 18 | Layer-2 ingest pipeline (binlog → pgvector + Apache AGE) — chain-anchor verification + 1s p95 lag +  |
| [TASK-MEMORY-102](TASK-MEMORY-102-layer2-rebuild-ci-gate/spec.md) | MUST | 1 | 10 | Layer-2 rebuild-from-Layer-1 CI gate — deterministic rebuild + spot-check + 30min budget + mid-rebui |
| [TASK-MEMORY-103](TASK-MEMORY-103-multi-device-sync/spec.md) | MUST | 1 | 18 | memory-sync daemon — laptop A ↔ Cloud memory ↔ laptop B with sync_class gating + CRDT conflict + 10K o |
| [TASK-MEMORY-104](TASK-MEMORY-104-tauri-app/spec.md) | SHOULD | 2 | 28 | Tauri 2.x desktop app — macOS + Windows + Linux signed/notarised + auto-update + tray + quick captur |
| [TASK-MEMORY-105](TASK-MEMORY-105-doctor-watched-folders-invariants/spec.md) | MUST | 2 | 7 | cyberos doctor — watched-folders integrity invariants (manifest ↔ filesystem ↔ HEAD reconciliation;  |
| [TASK-MEMORY-106](TASK-MEMORY-106-sync-class-enforcement/spec.md) | MUST | 1 | 6 | memory sync_class enforcement — private vs shareable + ACL filtering + structural compensation exclus |
| [TASK-MEMORY-107](TASK-MEMORY-107-fs-watcher/spec.md) | MUST | 2 | 14 | memory capture daemon — Rust + notify crate FS watcher with rate-limit + content-dedup + backpressure |
| [TASK-MEMORY-108](TASK-MEMORY-108-search-api/spec.md) | MUST | 2 | 12 | memory search — vector + graph + full-text in parallel + RRF fusion + BGE-rerank + RLS + ACL + chain_ |
| [TASK-MEMORY-109](TASK-MEMORY-109-claude-code-hook-capture/spec.md) | MUST | 2 | 8 | Claude Code hook capture — UserPromptSubmit + PostToolUse + Stop hooks emit memory memories with prom |
| [TASK-MEMORY-110](TASK-MEMORY-110-capture-daemon-health-restart/spec.md) | MUST | 2 | 6 | memory capture daemon supervision — systemd + launchd units + /healthz + watchdog + crash-restart wit |
| [TASK-MEMORY-111](TASK-MEMORY-111-pre-ingest-pii-detection/spec.md) | MUST | 2 | 9 | memory pre-ingest PII detection — Presidio EN + custom VN recognisers; ≥ 99.5% held-back recall on la |
| [TASK-MEMORY-112](TASK-MEMORY-112-episodic-memory/spec.md) | MUST | 3 | 12 | memory episodic memory — `kind: episode` + `cyberos recall-similar`; reflection-loop foundation (Wave |
| [TASK-MEMORY-113](TASK-MEMORY-113-recency-decay-recall/spec.md) | MUST | 3 | 8 | memory recall ranking — Park-et-al combined score (relevance · 0.4 + importance · 0.3 + recency · 0.3 |
| [TASK-MEMORY-114](TASK-MEMORY-114-write-time-importance/spec.md) | SHOULD | 3 | 8 | memory write-time importance scoring — Haiku-rated `meta.importance` filters noise at the source; rea |
| [TASK-MEMORY-115](TASK-MEMORY-115-cyberos-dream/spec.md) | SHOULD | 3 | 32 | memory dreaming — `cyberos dream` out-of-band batch reflection (4 detectors: duplicates / stale / new |
| [TASK-MEMORY-116](TASK-MEMORY-116-semantic-dedup-consolidate/spec.md) | SHOULD | 3 | 6 | memory consolidate — semantic-dedup phase (Walk → Compact → Sign → Publish → SemanticDedup); shares F |
| [TASK-MEMORY-117](TASK-MEMORY-117-per-store-acl/spec.md) | SHOULD | 3 | 24 | memory per-store ACL — `STORE.yaml` per top-level subtree; writer enforces ACL on writes; requires AG |
| [TASK-MEMORY-118](TASK-MEMORY-118-put-if-precondition/spec.md) | SHOULD | 3 | 8 | memory put_if — optimistic-concurrency primitive with content-hash preconditions; requires AGENTS.md  |
| [TASK-MEMORY-119](TASK-MEMORY-119-session-transcript-ledger/spec.md) | SHOULD | 3 | 24 | memory session transcript ledger — opt-in `cyberos session {start,append,end}`; default classificatio |
| [TASK-MEMORY-120](TASK-MEMORY-120-cyberos-history/spec.md) | SHOULD | 3 | 8 | memory history — `cyberos history <path>` surfaces per-file version + attribution from the audit chai |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-MEMORY-101→TASK-AI-019, TASK-MEMORY-111→TASK-AI-012
- **AUTH**: TASK-MEMORY-101→TASK-AUTH-003

**This module is depended on by:**

- **KB**: TASK-KB-007→TASK-MEMORY-108
- **PROJ**: TASK-PROJ-008→TASK-MEMORY-101
- **TEN**: TASK-TEN-004→TASK-MEMORY-111

## Wave 2026 Q3 — Self-learning agents (MEMORY Improvement)

Authored 2026-05-19 in response to the Anthropic Memory+Dreaming talk + Ramakrushna agentic-memory article (source materials at `../../../playground/extracts/`). The 9-FR wave splits into three sub-waves with explicit ship sequencing:

- **Wave 1 (4 dev-days)** — high-leverage, low-protocol-impact: TASK-MEMORY-112 (episodic) + TASK-MEMORY-113 (recency decay) + TASK-MEMORY-114 (importance scoring).
- **Wave 2 (9 dev-days, gated on `APPROVE protocol change P19 §7.7`)** — the headline `cyberos dream` out-of-band reflection pipeline: TASK-MEMORY-115 + TASK-MEMORY-116.
- **Wave 3 (8 dev-days, gated on `APPROVE protocol change P20 §14.4` / `P21 §3.1` / `P22 §18`)** — multi-agent scale: TASK-MEMORY-117 (per-store ACL) + TASK-MEMORY-118 (put_if precondition) + TASK-MEMORY-119 (session transcript ledger) + TASK-MEMORY-120 (`cyberos history`).

Each protocol-amendment-bearing FR carries its own approval chat-turn (per Stephen's 2026-05-19 decision: one-at-a-time, not bundled).

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._