# BRAIN module — feature request index

_Generated 2026-05-17 — 11 FRs, 136 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-BRAIN-101](FR-BRAIN-101-layer2-ingest-pipeline.md) | MUST | 1 | 18 | Layer-2 ingest pipeline (binlog → pgvector + Apache AGE) — chain-anchor verification + 1s p95 lag +  |
| [FR-BRAIN-102](FR-BRAIN-102-layer2-rebuild-ci-gate.md) | MUST | 1 | 10 | Layer-2 rebuild-from-Layer-1 CI gate — deterministic rebuild + spot-check + 30min budget + mid-rebui |
| [FR-BRAIN-103](FR-BRAIN-103-multi-device-sync.md) | MUST | 1 | 18 | brain-sync daemon — laptop A ↔ Cloud BRAIN ↔ laptop B with sync_class gating + CRDT conflict + 10K o |
| [FR-BRAIN-104](FR-BRAIN-104-tauri-app.md) | SHOULD | 2 | 28 | Tauri 2.x desktop app — macOS + Windows + Linux signed/notarised + auto-update + tray + quick captur |
| [FR-BRAIN-105](FR-BRAIN-105-doctor-watched-folders-invariants.md) | MUST | 2 | 7 | cyberos doctor — watched-folders integrity invariants (manifest ↔ filesystem ↔ HEAD reconciliation;  |
| [FR-BRAIN-106](FR-BRAIN-106-sync-class-enforcement.md) | MUST | 1 | 6 | BRAIN sync_class enforcement — private vs shareable + ACL filtering + structural compensation exclus |
| [FR-BRAIN-107](FR-BRAIN-107-fs-watcher.md) | MUST | 2 | 14 | BRAIN capture daemon — Rust + notify crate FS watcher with rate-limit + content-dedup + backpressure |
| [FR-BRAIN-108](FR-BRAIN-108-search-api.md) | MUST | 2 | 12 | BRAIN search — vector + graph + full-text in parallel + RRF fusion + BGE-rerank + RLS + ACL + chain_ |
| [FR-BRAIN-109](FR-BRAIN-109-claude-code-hook-capture.md) | MUST | 2 | 8 | Claude Code hook capture — UserPromptSubmit + PostToolUse + Stop hooks emit BRAIN memories with prom |
| [FR-BRAIN-110](FR-BRAIN-110-capture-daemon-health-restart.md) | MUST | 2 | 6 | BRAIN capture daemon supervision — systemd + launchd units + /healthz + watchdog + crash-restart wit |
| [FR-BRAIN-111](FR-BRAIN-111-pre-ingest-pii-detection.md) | MUST | 2 | 9 | BRAIN pre-ingest PII detection — Presidio EN + custom VN recognisers; ≥ 99.5% held-back recall on la |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-BRAIN-101→FR-AI-019, FR-BRAIN-111→FR-AI-012
- **AUTH**: FR-BRAIN-101→FR-AUTH-003

**This module is depended on by:**

- **KB**: FR-KB-007→FR-BRAIN-108
- **PROJ**: FR-PROJ-008→FR-BRAIN-101
- **TEN**: FR-TEN-004→FR-BRAIN-111

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._