---
title: memory — Universal Personal &amp; Shared Memory Protocol · CyberOS
source: website/docs/modules/memory/index.html
migrated: FR-DOCS-002
---

memory is an **append-only, audit-chained ledger** of structured memory files. It is local-first (the canonical state is on the laptop), portable (deterministic `zip` export), and cryptographically verifiable (Merkle Mountain Range + Ed25519 signed tree heads). Every byte written to a CyberOS memory store can be replayed, proved-included, or — if compliance demands it — purged with a tamper-evident audit trail. The protocol lives at `memory/docs/AGENTS.md`; the implementation lives at `memory/cyberos/core/`; the CLI is `cyberos`. 

Layer 1 status

Shipped

filesystem ledger · 7/7 phases · 12 audit proposals

Stages 1–5

Designed

universal protocol roadmap · see §Stages

Tests

233/235

2 pre-existing invariant-check bugs · non-blocking

Invariants

15/15

`cyberos doctor` · all PASS on live store

CLI subcommands

30 shipped + 8 designed

\+ `memory init/watch/status/sync/...`

Stores

Personal + Lumi's

offline-first laptop · cloud org-tenant

Used by

All 22 modules + any folder

universal — not just CyberOS

Schema

`memory.schema.json`

closed enum · BCP-14 normative · P13 extends

0

## The bigger picture — Personal memory, Lumi's memory, the protocol

The original memory scope — "the audit-chained substrate of CyberOS" — understated what the protocol is. As of v1.0.0 of [MEMORY_AUTOSYNC_DESIGN.md](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) (locked 2026-05-14), memory is reframed as **the universal personal-and-shared memory protocol**. CyberOS is the first consumer but does not own the protocol. Three concepts to hold simultaneously: 

Layer 1 · Personal

🧠

Personal memory

One per human. Lives at `~/.cyberos/memory/store/`. Watches whichever folders you opt in (any folder — not just `cyberos/`). Captures files, decisions, discussions. **Portable** : copy the folder = move your memory.

Offline-first · cryptographically chained · open protocol

Layer 2 · Sync

🔄

Two-way sync orchestrator

Background daemon. Personal memory **pushes** shareable memories up to Lumi's memory. Personal memory **pulls** shared-team memories down. Conflict resolution by chain-position (per AGENTS.md §14.2 — every import is a fresh local `put`).

5-min default window · configurable · privacy-class enforced

Layer 3 · Shared

☁️

Lumi's memory (cloud)

Cloud-hosted org-tenant memory. Same protocol, multi-tenant deployment. Three names for three audiences: **Lumi's memory** (user-facing), **CUO's memory** (technical), **CyberSkill's memory** (deployment-specific). Gated on AUTH + TEN + AI Gateway.

RLS-isolated Postgres · S3/R2 · per-tenant · ships P2

### What's automatic, what's manual

Activity| Captured automatically?| By which capture surface  
---|---|---  
File edit inside a _watched_ folder| ✅ yes| Filesystem watcher daemon  
`git commit` inside a watched folder| ✅ yes — emits `decisions`| Filesystem watcher + git hook  
Cowork session locks a decision| ✅ yes| Cowork session hook (Stage 2 week 3)  
Claude Code tool call mutating working folder| ✅ yes| Claude Code hook (Stage 2 week 4)  
Slack / Zalo message tagged `@lumi`| ✅ yes (when MCP feed enabled — deferred)| Slack/Zalo MCP bridge  
Granola / Meet transcript at meeting end| ✅ yes (when MCP feed enabled — deferred)| Granola/Meet MCP bridge  
Apple Notes / Obsidian / Notion save| ✅ yes (when MCP feed enabled — deferred)| Notes-app MCP bridge  
File edit in an _unwatched_ folder| ❌ no — privacy floor| —  
Random browsing (no `@memory`/`@lumi` tag)| ❌ no — privacy floor| —  
Manual `cyberos memory capture <text>`| n/a — explicit one-off| CLI  
  
**Initial scope (Stage 2 sprint):** Claude-only — Cowork sessions + Claude Code. Other MCP surfaces queue behind explicit team adoption.

### Strategic implication — this is the moat

A tenant six months into using Lumi's memory has accumulated context that no competitor can replicate. The longer the org uses the platform, the smarter Lumi becomes. **Switching cost = value of the org's accumulated memory.** This is the answer to the strategic GTM question — not the marketplace; the memory. 

Personal memory ships under the open memory protocol — Apache 2.0, no login, useful to anyone with a laptop. Lumi's memory is the commercial offering — sold per tenant, per seat, per storage tier, provisioned at tenant create-time via the TEN module. Personal memory is the OSS distribution surface; Lumi's memory is the SaaS product; multi-memory auto-evolve is the compounding moat. 

1

## Why memory exists

Every agentic system eventually trips on the same wall: _state lives nowhere persistent and replayable_. Chat sessions evaporate, vendor memory APIs become single points of failure and audit, institutional knowledge ends up trapped in screenshots and Slack threads, and **nobody can answer "what did we decide last quarter?" without spelunking through search logs**. memory starts from a different axiom: **the laptop filesystem is the canonical store for one human's memory** , every mutation is an append, every append is anchored in a Merkle chain so corruption is detectable and tampering is impossible — and then the same protocol extends to a **cloud-hosted org tenant** (Lumi's memory) that aggregates the team's shareable memories and lets the org's accumulated wisdom compound over time. 

🔒

Sovereign by design

No vendor lock-in. `~/.cyberos/memory/store/` is a complete, portable snapshot of one human's memory — copy the folder, take it anywhere, no DB rebuild.

⛓

Cryptographic provenance

Every audit row carries `prev_chain` \+ `chain`. MMR overlay produces logarithmic inclusion proofs · Ed25519 signs tree heads · the chain refuses to forget.

🔄

Universal capture

Watch any folder; the protocol captures file activity AND discussions (Cowork, Claude Code, MCP feeds). Explicit opt-in per folder. The "remember everything" guarantee only applies where you let it.

🧠×N

Multi-memory power

N personal memories → 1 Lumi's memory → synthesised cross-person wisdom. The longer the org uses the platform, the smarter Lumi becomes. **This is the moat.**

The bet is twofold: **(1)** pay the cost of a real audit chain _once_ , at the substrate layer, and every downstream consumer — Skill, CUO, EMAIL, PROJ, CRM, HR, INV, plus any future module or third-party tool — inherits provenance, replay, and erasure for free. Without memory, every module would re-invent its own audit table. **(2)** Pay the cost of a real _shareable_ memory layer once, and the org gains a compounding wisdom asset that grows monotonically: accumulated decisions, recurring patterns, dedup-ed answers to questions the team has asked before. A new hire joining six months in inherits the entire org's institutional memory from day one. 

Compliance becomes a property of the platform, not a policy bolt-on. Vietnam PDPL Art. 14 DSAR, Art. 16 erasure, Art. 7 no-data-sale; GDPR Art. 17 right-to-erasure, Art. 30 records-of-processing; EU AI Act Art. 12 logging and Art. 50 transparency — all expressible as ledger operations or sync-class enforcement. Audit is a property of the protocol; CyberOS modules inherit it; Lumi's memory inherits it; the synthesis sub-skill inherits it. Every memory carries its own evidence. 

2

## What it does — 5W1H2C5M

A structured decomposition of memory's scope. Every cell below traces back to + AGENTS.md.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is memory?| An open, normative _protocol_ for append-only audit-chained personal-and-shared memory. Two concrete instances: **Personal memory** (one per human, lives at `~/.cyberos/memory/store/`, captures activity across watched folders + discussion surfaces) and **Lumi's memory** (cloud-hosted per-org tenant, 2-way sync target). Every mutation is one of three canonical ops — `put`, `move`, `delete`. Closed schema, BCP-14 normative.  
**5W · Who**|  Who reads/writes?| **Personal memory writers:** the human owner (CLI), the capture daemon (filesystem watcher · Cowork hook · Claude Code hook · MCP feeds), CUO routing decisions on the user's behalf, every module's subgraph that the user authored. **Lumi's memory writers:** sync orchestrator pushing on each user's behalf; synthesis sub-skill nightly. **Readers:** any agent (Claude, Codex, Cursor) the user has authorized, Lumi/CUO during conversation, downstream automation. **Owner:** the human for Personal memory; the org (CDO seat at P1+) for Lumi's memory.  
**5W · When**|  When does write happen?| **Personal:** on every captured activity — file event from any watched folder, end-of-Cowork-turn lock, Claude Code tool call, manual `cyberos memory capture`. Group-commit window 5 ms; consolidation at >5 MB or >5,000 rows. **Sync to Lumi's:** default 5-min window for shareable+ memories; mode configurable (realtime / frequent / normal / infrequent / manual). **Synthesis on Lumi's:** nightly, walks prior 24 h, emits cluster + dedup + wisdom artefacts.  
**5W · Where**|  Where does it live?| **Personal memory:** resolved at `$HOME/.cyberos/memory/store/` by default (override via `$CYBEROS_MEMORY_HOME`). **Portable** — copy the folder between machines (laptop, desktop, USB, iCloud Drive, Dropbox) to move your memory. Symlinks resolved; sandbox paths rejected (`layout-no-sandbox-path` invariant). **Lumi's memory:** a CyberOS multi-tenant deployment (Postgres + RLS + S3/R2) provisioned at tenant create-time via the TEN module. Per-tenant subject `org:<slug>`.  
**5W · Why**|  Why this design?| Three failure modes memory solves simultaneously: (1) **vendor lock-in** — your memory walks with you on any laptop; (2) **state without replay** — every byte is replayable from the ledger; (3) **no team-level wisdom** — Lumi's memory aggregates across N personal memories to produce synthesised org-level memory that no single contributor could write alone. The append-only + MMR + Ed25519 STH + sync-class taxonomy is the smallest primitive set that delivers all three plus cryptographic provenance.  
**1H · How**|  How does it work end-to-end?| Capture surface emits → canonical op (`put`/`move`/`delete`) → two-phase atomic write → group-commit audit frame → MMR leaf → derived SQLite index → HEAD seqlock advance. Sync orchestrator tails the local chain, filters by `sync_class`, pushes shareable+ to Lumi's memory over JWT-authenticated HTTP. Pulls inbound via the same envelope; each pulled record becomes a fresh local `put` per AGENTS.md §14.2 — never a chain-mutation. See Key flows for sequence diagrams of capture, sync, and synthesis.  
**2C · Cost**|  Cost?| **Personal memory per-write:** ~120 µs hot path (group-committed), ~3 ms cold. **Read:** ~6 µs (mmap seqlock). **Storage overhead:** ~28% vs raw body. **STH signing:** ~0.4 ms Ed25519. **Sync push:** ~50 ms p95 over LAN, ~150 ms p95 over WAN per memory; batched. **Lumi's memory infra:** ~$2/seat/month per-tenant baseline (RDS micro tier + S3 + signed-URL CDN) at P2 launch scale. **Synthesis sub-skill:** nightly LLM-cost ≈ $0.05/user/day at BGE-M3 embeddings + Haiku synthesis.  
**2C · Constraints**|  Constraints?| (a) macOS durability: `F_BARRIERFSYNC` per-batch, `F_FULLFSYNC` at checkpoint. (b) Single canonical writer per Personal memory at a time (lease + `.lock`). (c) Sandbox paths protocol-rejected. (d) Sync requires AUTH-issued JWT — no anonymous push to Lumi's. (e) PII detected at capture surfaces with sync_class ≠ private is _held back from sync_ and prompts the user via the configured prompt UX (Cowork-inline / desktop-notif / `memory pending`). (f) Right-to-forget (PDPL Art. 16 / GDPR Art. 17) propagates across memories but the _fact_ of erasure is itself unerasable.  
**5M · Materials**|  What does it use?| POSIX filesystem · SHA-256 · Ed25519 (cryptography lib) · libyaml-backed msgspec for frontmatter · zstd for compaction · SQLite (WAL mode) · optional sentence-transformers (BGE-M3) for semantic search · **Stage 2+:** Rust binary using `notify` crate for filesystem watcher · launchd/systemd-user/Task Scheduler for daemon supervision · **Stage 3+ (Lumi's memory):** Postgres 17 + RLS · pgvector HNSW · Apache AGE 1.5 · S3/R2/MinIO · AUTH-issued JWT (RS256) · LiteLLM via AI Gateway for synthesis.  
**5M · Methods**|  Method choices?| Append-only ledger (no in-place mutation). Seqlock for lock-free reads. MMR Merkle proofs. RFC 8785 JCS canonical JSON. BCP-14 normative protocol. Deterministic export (sorted paths · fixed timestamps · fixed mode · zstd level 6). **Universal-protocol additions:** explicit opt-in per watched folder (privacy floor). PII detection at capture write (Presidio). Sync conflict resolution via chain-position-wins + fresh-local-`put` on import (per §14.2). Synthesis as a CUO sub-skill, not a separate module — wisdom artefacts follow the same protocol.  
**5M · Machines**|  Where does it run?| Personal memory: any laptop / desktop (macOS primary; Linux + WSL + Windows supported). Optional HTTP REST mode (`cyberos serve` — shipped) and mobile read-only static site (`cyberos publish` — shipped). No server-side database for Personal memory — the laptop _is_ the database. Lumi's memory: managed cloud deployment per tenant (AWS Singapore primary; Azure VN secondary; per-tenant region pinning via TEN). Synthesis runs nightly in tenant's compute window.  
**5M · Manpower**|  Who maintains?| Personal memory: 1 IC owner (Stephen Cheng / future CDO seat) — module is small (~5k LoC + capture daemon adds ~2k). Open-source contributions welcome. Lumi's memory: shared on-call with TEN module owner from P2 forward; eventually CDO seat at P1+ takes accountability; Cloud-DBA + Sync-SRE specialists join at P3+.  
**5M · Measurement**|  How measured?| 15 doctor invariants (chain continuity, op-enum closure, layout-canonical, MMR cross-check, STH signature, …) plus 4 universal-protocol invariants (`layout-watched-folders-exist`, `layout-watched-folders-permissions`, `capture-daemon-not-stuck`, `sync-orchestrator-running`). 5 new KPIs at Lumi's memory scale: capture-rate per user, sync-success rate, sync-conflict rate, synthesis-useful rate (per-user thumbs up/down), Lumi's memory seq counter. Continuous benchmark suite. CHANGELOG newest-first.  
  
3

## Architecture — three layers, six ops

memory is specified as three layers. **Layer 1** (the append-only filesystem ledger) is shipped — 233/235 tests pass (2 pre-existing invariant-check bugs, non-blocking). **Layer 2 ingest pipeline** shipped end-to-end in `services/memory/` on 2026-05-18: Cargo workspace + 2 migrations + chain_anchor + cursor + binlog_tail + entity_extract + pgvector upsert + full `ingest::run_batch` orchestrator + daemon main loop running a per-tenant tokio task on 200 ms poll + `/metrics` Prometheus endpoint + graceful shutdown + 4 integration tests in `services/memory/tests/ingest_test.rs`. The Apache AGE graph mirror and Layer 3 (archival corpus) remain planned for P1 / P2. The single canonical writer enforces the protocol; every other module's "memory bridge" routes through it. 

graph TB subgraph CLIENTS ["Clients (untrusted from protocol pov)"] AGENT["Claude / Codex / Cursor  
via MCP tool"] CLI["cyberos CLI"] SUBG["Module subgraphs  
(via memory bridge)"] CUO_M["CUO router  
(decision rows)"] end subgraph CORE ["memory canonical writer (cyberos/core/)"] OPS["ops.py  
put · move · delete"] LOCK["lock.py  
leased.lock"] WRITER["writer.py  
group-commit ledger"] WALKER["walker.py  
mmap ledger replay"] READER["reader.py  
seqlock lock-free reads"] INDEX["index.py  
WAL-mode SQLite"] MMR["mmr.py  
Merkle Mountain Range"] STH["sth.py  
Ed25519 tree heads"] end subgraph LAYER1 ["Layer 1 · Filesystem ledger (.cyberos/memory/store/)"] FILES["memories/{kind}/{hex}/{slug}.md"] META["sidecar.meta.json"] BINLOG["audit/*.binlog"] HEAD["HEAD seqlock"] MANI["manifest.json"] CHK["audit/checkpoints/"] end subgraph LAYER2 ["Layer 2 · ingest pipeline (Wave 1 shipped) + AGE graph (planned)"] INGEST["services/memory/  
cursor · binlog_tail · chain_anchor · entity_extract"] PGV["pgvector HNSW  
(upsert via ingest::run_batch)"] AGE["Apache AGE graph  
(planned · P1)"] end subgraph LAYER2S ["Layer 2 (slice) · Local embeddings (shipped)"] EMB["sentence-transformers  
BGE-M3 (optional dep)"] end subgraph LAYER3 ["Layer 3 · Archival corpus (planned)"] S3["S3 / R2 / MinIO  
cold tier"] end AGENT --> OPS CLI --> OPS SUBG --> OPS CUO_M --> OPS OPS --> LOCK OPS --> WRITER WRITER --> BINLOG WRITER --> FILES WRITER --> META WRITER --> MMR MMR --> STH STH --> CHK WRITER --> HEAD WRITER --> INDEX READER --> HEAD READER --> FILES WALKER --> BINLOG FILES -. semantic.-> EMB FILES -. P1+.-> PGV FILES -. P1+.-> AGE CHK -. P2+ archive.-> S3 classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#f0eee9,stroke:#9c9286,stroke-dasharray:4 3 class OPS,LOCK,WRITER,WALKER,READER,INDEX,MMR,STH,FILES,META,BINLOG,HEAD,MANI,CHK,EMB,AGENT,CLI shipped class PGV,AGE,S3,SUBG planned 

### Internal components — walkthrough

Component| File| Responsibility  
---|---|---  
`ops.py`| core/ops.py| Canonical operations `put`, `move`, `delete(mode)`. Path validation, denylist check, idempotency by content hash.  
`writer.py`| core/writer.py| Group-commit ledger. Two-phase atomic write (tmpfile + rename + dir fsync). Group window = 5 ms; per-batch `F_BARRIERFSYNC`.  
`reader.py`| core/reader.py| Lock-free seqlock reader. mmap the body, snapshot HEAD, re-stat; mismatch triggers retry. Stale-tolerant.  
`walker.py`| core/walker.py| mmap-based binlog replay. Verifies frame CRC + chain continuity; emits to invariant walker.  
`lock.py`| core/lock.py| Leased `.lock` with 10 s TTL + 3 s renew. Stale leases reaped via monotonic clock comparison.  
`mmr.py`| core/mmr.py| Pure-Python Merkle Mountain Range. Stage-1 ships peaks + leaf-hashes; STH-only mode replaces chained `prev_chain` when enabled.  
`sth.py`| core/sth.py| Ed25519 signed tree heads. Passphrase-wrapped private key on disk; checkpoints land in `audit/checkpoints/`.  
`index.py`| core/index.py| Derived WAL-mode SQLite index. Rebuildable from binlog; never authoritative — filesystem wins on conflict.  
`fsync.py`| core/fsync.py| Platform-correct durability. macOS `F_BARRIERFSYNC` / `F_FULLFSYNC`; Linux `fdatasync` \+ parent-dir fsync.  
`consolidate.py`| core/consolidate.py| Walk → Compact → Sign → Publish. Triggers on size (>5 MB) or row count (>5,000). zstd archives sealed monthly segments.  
`invariants.py`| core/invariants.py| 15 walker invariants (chain continuity, op-enum closure, layout-canonical, MMR cross-check, sidecar-body-hash, …). Drives `cyberos doctor`.  
`export.py`| core/export.py| Deterministic zip. Sorted paths · fixed timestamp `2000-01-01T00:00:00Z` · fixed mode `0o644` · ZIP_DEFLATED level 6. Excluded: `exports/`, `__pycache__/`, `.cache/`, `.lock`, `HEAD`.  
`import_.py`| core/import_.py| P6 — cross-memory merge. Foreign rows become fresh local `put` rows with `extra.imported_from` \+ `extra.foreign_chain`.  
`semantic.py`| core/semantic.py| Optional local embeddings (sentence-transformers BGE-M3). On-demand index, no daemon.  
`serve.py`| core/serve.py| HTTP REST mode. Exposes the same six ops over a local socket for IDE / mobile clients.  
`publish.py`| core/publish.py| Mobile read-only static site. Generated from the deterministic export.  
`digest.py`| core/digest.py| Daily LLM summary (via AI gateway). Produces one digest row per day.  
`conflicts.py`| core/conflicts.py| Sync-FS conflict detection (iCloud / Dropbox / OneDrive). Quarantines competing file copies to `conflicts/`.  
`backup.py``prune.py`| core/| Incremental snapshot + sealed-segment pruning. Both reversible via the consolidation log.  
  
3.5

## Stages 1–5 — universal protocol roadmap

The roadmap from "memory is CyberOS's audit ledger" (shipped today, Stage 0) to "Personal + Lumi's memory with multi-memory auto-evolve" (Stage 5, P3) is gated in five stages. Stages 1 and 2 are buildable today against the existing memory module; Stages 3–5 ride the P0+P2 critical path (AUTH + AI Gateway + TEN). The full design is normative in [MEMORY_AUTOSYNC_DESIGN.md](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) §3 and lands as Proposal P13 in `memory/docs/PROPOSAL.md`. 

flowchart LR subgraph S0 ["Stage 0 · shipped"] S0L1["Layer 1  
filesystem ledger  
MMR · STH · 15 invariants"] end subgraph S1 ["Stage 1 · ready"] S1A["Personal memory  
any folder · portable"] S1B["memory init / watch / status"] S1C["watched_folders schema"] end subgraph S2 ["Stage 2 · 2-4 weeks"] S2A["Capture daemon  
FS watcher · Rust + notify"] S2B["Cowork session hook"] S2C["Claude Code hook"] end subgraph S3 ["Stage 3 · P2"] S3A["Lumi's memory  
cloud per-tenant"] S3B["TEN · AUTH · AI Gateway"] end subgraph S4 ["Stage 4 · P2"] S4A["2-way sync orchestrator  
JWT push / pull"] S4B["sync_class enforcement"] end subgraph S5 ["Stage 5 · P3"] S5A["Multi-memory auto-evolve  
synthesis sub-skill"] S5B["nightly dedup · cluster · synthesize"] end S0 --> S1 S1 --> S2 S2 -. requires .-> S3 S3 --> S4 S4 --> S5 classDef shipped fill:#f5ede6,stroke:#45210e,stroke-width:2px classDef ready fill:#fde7b3,stroke:#9c750a,stroke-width:2px classDef designed fill:#f0eee9,stroke:#9c9286,stroke-dasharray:4 3 class S0L1 shipped class S1A,S1B,S1C ready class S2A,S2B,S2C,S3A,S3B,S4A,S4B,S5A,S5B designed 

### Stage gating

Stage| Deliverable| Gating dependency| Buildable when| Effort  
---|---|---|---|---  
**Stage 0**|  Layer 1 filesystem ledger · 6 ops · 15 doctor invariants · MMR · Ed25519 STH · deterministic export · sync-conflict awareness| —| Shipped — 233/235 tests (2 pre-existing invariant-check bugs, non-blocking), 13–15 doctor invariants on live memory| —  
**Stage 1**|  Personal memory — universal protocol: any folder, multi-folder watch, explicit-opt-in privacy floor, portable by folder copy. CLI: `memory init/watch/unwatch/status/capture`. Manifest schema + 2 new doctor invariants.| None — extends shipped memory module| **Now** — no external deps| ~1 week (1 IC)  
**Stage 2**|  Capture daemon: filesystem watcher (Rust + `notify`) · Cowork session hook · Claude Code hook · rate limiting + content dedup + PII detection. _Initial scope: Claude-only._ Slack/Granola/Notes/etc. deferred to explicit team adoption.| Stage 1| **2–4 weeks from Stage 1**|  ~3 weeks (1–2 ICs)  
**Stage 3**|  Lumi's memory — cloud-hosted org tenant deployment of the memory protocol. RLS-isolated Postgres + S3/R2. Per-user JWT issued by AUTH. Read scopes per AGENTS.md §3.6 / SKILL.md frontmatter.| AUTH + AI Gateway + TEN modules + thin-TEN-billing slice| **P2 (P2 · exit)** per reviewer's reorder| ~6–8 weeks (2 ICs)  
**Stage 4**|  2-way sync orchestrator: push state-machine (local-write → pending-push → pushed → confirmed) and pull state-machine (remote-write → pending-pull → imported). Conflict resolution = chain-position-wins + fresh-local-`put`. Configurable sync intervals.| Stage 3 + JWT wire protocol| **P2 (P2 · exit) — parallel to Stage 3**|  ~4 weeks (1 IC)  
**Stage 5**|  Multi-memory auto-evolve: synthesis-author CUO sub-skill runs nightly over Lumi's memory. Emits daily / weekly / decision-pending wisdom artefacts. Cross-person dedup, pattern recognition, accumulated org learning.| Stage 4 + synthesis sub-skill spec| **P3 (P3 · exit)**|  ~6 weeks (1 IC + 1 part-time CDO seat)  
  
### Personal memory sub-architecture

graph TB subgraph CAP ["Capture surfaces (Stage 2)"] FSW["Filesystem watcher  
Rust + notify"] COW["Cowork session hook"] CCH["Claude Code hook"] MCP_S["MCP feeds (deferred)  
Slack · Granola · Notes"] CLI_C["cyberos memory capture  
manual one-off"] end subgraph PROTO ["memory protocol (Stage 0 — shipped)"] OPS["canonical ops  
put · move · delete"] WRITER["writer  
group-commit ledger"] MMR_P["MMR + Ed25519 STH"] INV["15 doctor invariants"] end subgraph PB_STORE ["~/.cyberos/memory/store/ (Stage 1 — ready)"] MANI["manifest.json  
watched_folders[]"] MEMS["memories/{kind}/{hex}/{slug}.md"] AUD["audit/current.binlog"] HEAD_S["HEAD seqlock"] end subgraph SYNC_O ["Sync orchestrator (Stage 4 — P2)"] OUT["push queue  
shareable+ memories"] IN["pull queue  
shared team memories"] end FSW --> OPS COW --> OPS CCH --> OPS MCP_S -. when enabled .-> OPS CLI_C --> OPS OPS --> WRITER WRITER --> MMR_P WRITER --> AUD WRITER --> MEMS WRITER --> HEAD_S INV -. validates .-> AUD AUD --> OUT IN -.-> OPS classDef shipped fill:#f5ede6,stroke:#45210e classDef ready fill:#fde7b3,stroke:#9c750a classDef designed fill:#f0eee9,stroke:#9c9286,stroke-dasharray:4 3 classDef deferred fill:#faf8f6,stroke:#c5bdb0,stroke-dasharray:2 2 class OPS,WRITER,MMR_P,INV,MANI,MEMS,AUD,HEAD_S shipped class FSW,COW,CCH,CLI_C ready class OUT,IN designed class MCP_S deferred 

### Lumi's memory sub-architecture (Stages 3–5)

graph TB subgraph PEOPLE ["Personal memories (N humans per org)"] PB1["Stephen's  
~/.cyberos/memory/store/"] PB2["Teammate A's  
~/.cyberos/memory/store/"] PB3["Teammate B's  
~/.cyberos/memory/store/"] end subgraph SYNC ["Sync transport (Stage 4)"] PUSH["push pipeline  
shareable+ → Lumi"] PULL["pull pipeline  
shared → personal"] JWT["AUTH JWT  
RS256 · per-user"] end subgraph LUMI ["Lumi's memory — cloud tenant (Stage 3)"] LMANI["manifest.json  
tenant-scoped"] LAUD["audit/binlog  
per-tenant"] LMEMS["memories/{kind}/{hex}/.md  
RLS by org_id"] PGV["pgvector HNSW  
BGE-M3 embeddings"] AGE["Apache AGE  
org relationship graph"] end subgraph SYN ["Synthesis sub-skill (Stage 5)"] NIGHTLY["nightly walk  
last 24h memories"] CLUSTER["topic cluster  
BGE-M3 + DBSCAN"] DEDUP["cross-person dedup  
semantic + chain"] WISDOM["synthesis@1 artefacts  
daily · weekly · decisions-pending"] end subgraph LUMI_R ["Lumi (the Genie) reads here"] CUO["CUO router  
tenant-scoped reads"] end PB1 --> PUSH PB2 --> PUSH PB3 --> PUSH PUSH --> JWT JWT --> LAUD LAUD --> LMEMS LMEMS --> PGV LMEMS --> AGE LMEMS --> NIGHTLY NIGHTLY --> CLUSTER CLUSTER --> DEDUP DEDUP --> WISDOM WISDOM --> LMEMS LMEMS --> PULL PULL --> PB1 PULL --> PB2 PULL --> PB3 LMEMS --> CUO PGV --> CUO AGE --> CUO classDef shipped fill:#f5ede6,stroke:#45210e classDef designed fill:#f0eee9,stroke:#9c9286,stroke-dasharray:4 3 class PB1,PB2,PB3 shipped class PUSH,PULL,JWT,LMANI,LAUD,LMEMS,PGV,AGE,NIGHTLY,CLUSTER,DEDUP,WISDOM,CUO designed 

### Sync-class privacy model

Class| Visibility| Stays where| Default for  
---|---|---|---  
`private`| Personal memory only · never pushed| Owner's memory, all machines (via folder copy)| Random browser scraps, half-finished notes, tool-output snippets, anything PII-flagged  
`personal`| Owner's Personal memory · syncs across owner's own machines (laptop ↔ desktop) but NOT to Lumi| Owner's memories only (Stage 4+ multi-device sync)| Personal notes, drafts, exploratory work  
`shareable`| Eligible for push to Lumi's memory · subject to ACL filter at push time| Owner's memory + Lumi's tenant (sync push gated by PII + ACL)| Decisions, completed deliverables, meeting outcomes  
`team-public`| Pushed to Lumi's memory AND visible to all org members via shared scope| Owner + Lumi + all org members' Personal memories (via pull)| Locked policy decisions, RFC approvals, OKR commits  
`org-only`| Pushed to Lumi's memory but restricted by RBAC role| Owner + Lumi + RBAC-eligible members' Personal memories| Compensation decisions, hiring decisions, financial state  
  
**Default:** `private`. Per-folder override at `cyberos memory watch <path> --default-sync-class <class>`. Per-memory override via `cyberos memory reclass <memory-id> <class>`.

4

## Data model

Every entity in memory traces to one or more files on disk. The schema is closed (`memory.schema.json`); unknown `kind` values are rejected at write time. The diagram below shows the entity relationships as they manifest in the filesystem layer. 

erDiagram MANIFEST ||--o{ AUDIT_SEGMENT: "tracks segments" MANIFEST ||--o{ MEMORY_FILE: "indexes" MANIFEST ||--o{ CHECKPOINT: "tracks last_sth" AUDIT_SEGMENT ||--|{ AUDIT_FRAME: "contains" AUDIT_FRAME }o--|| MEMORY_FILE: "describes mutation of" AUDIT_FRAME ||--o| MMR_LEAF: "becomes" MMR_LEAF }o--o{ MMR_PEAK: "rolls up to" MMR_PEAK ||--o{ CHECKPOINT: "signed by" CHECKPOINT ||--|| STH_SIG: "carries Ed25519" MEMORY_FILE ||--o| SIDECAR_META: "has sidecar" MEMORY_FILE ||--o| ENCRYPTED_BODY: "may be ciphertext" MEMORY_FILE }o--o| TOMBSTONE: "may be tombstoned" MANIFEST { string store_id PK string layout_version int64 audit_chain_head int64 last_seq string last_sth_root string crypto_mode "chained or sth_only" obj imports "fingerprint to last_imported_seq" } AUDIT_SEGMENT { string filename PK "YYYY-MM dot binlog" bool sealed bool compacted "zstd" int64 first_seq int64 last_seq } AUDIT_FRAME { int64 seq PK int64 ts_ns string op "put or move or delete" string path string body_hash "SHA-256" string prev_chain string chain "SHA-256" string actor obj extra "imported_from, foreign_chain, etc" } MMR_LEAF { int64 leaf_index PK string leaf_hash "= chain" int64 seq FK } MMR_PEAK { int height string peak_hash int64 position } CHECKPOINT { string filename PK "timestamp-root json file" int64 tree_size string mmr_root int64 signed_at_ns } STH_SIG { string algorithm "Ed25519" string pubkey_id bytes signature } MEMORY_FILE { string path PK "memories-kind-hex-file.md" string kind "decisions or facts or people or projects or preferences or drift or refinements" string body_format "frontmatter or sidecar" string body_hash string state "active or tombstoned or purged" } SIDECAR_META { string path PK ".meta.json" string kind string sync_class "private or shareable" string classification "public or internal or confidential or restricted" obj cipher "envelope when encrypted" obj acl string body_hash } ENCRYPTED_BODY { string envelope_id PK string algo "AES-256-GCM" bytes ciphertext bytes iv bytes tag } TOMBSTONE { string path PK string reason int64 tombstoned_at_seq } 

### Universal-protocol entities (P13 — Stages 1–5)

Five new entities encode the universal-protocol scope. Three are managed by the Personal memory owner (WatchedFolder · CaptureEvent · SyncState); two are managed on Lumi's memory (SharedMemoryAcl · SynthesisArtefact). 

erDiagram MANIFEST ||--o{ WATCHED_FOLDER: "lists" WATCHED_FOLDER ||--o{ CAPTURE_EVENT: "produces" CAPTURE_EVENT ||--|| MEMORY_FILE: "emits" MEMORY_FILE ||--o| SYNC_STATE: "tracked by" SYNC_STATE }o--|| LUMI_ROW: "pushed to" LUMI_ROW ||--o{ SHARED_MEMORY_ACL: "scoped by" SHARED_MEMORY_ACL }o--o{ ORG_MEMBER: "grants to" LUMI_ROW ||--o{ SYNTHESIS_INPUT: "feeds" SYNTHESIS_INPUT }o--|| SYNTHESIS_ARTEFACT: "rolls up to" WATCHED_FOLDER { string path PK string default_sync_class string default_classification int64 added_at_seq string added_by_actor bool active } CAPTURE_EVENT { string event_id PK string capture_surface "filesystem | cowork | claude-code | mcp | cli" string source_path_or_url int64 captured_at_ns string capture_daemon_version string trigger_kind "create | modify | commit | session-end | tool-call" bool auto_capture "vs manual" obj pii_flags } SYNC_STATE { string memory_path PK string state "local-write | pending-push | pushed | confirmed | pending-pull | imported" int64 last_attempt_ns int64 last_success_ns string lumi_chain_hash "set on confirm" int retry_count } LUMI_ROW { int64 lumi_seq PK string tenant_id FK string user_id FK string source_memory_fingerprint int64 source_local_seq int64 imported_at_ns string sync_class } SHARED_MEMORY_ACL { string memory_path PK string subject "user:* | role:* | org:*" string rights "read | write | grant" bool inherited "from folder default" int64 granted_at_seq } ORG_MEMBER { string user_id PK string email string role "from AUTH RBAC catalogue" obj rbac_grants } SYNTHESIS_INPUT { string memory_path PK int64 cluster_id FK float relevance_score } SYNTHESIS_ARTEFACT { string artefact_path PK "memories-synthesis-daily-YYYY-MM-DD.md" string kind "daily | weekly | decisions-pending" int64 walked_from_lumi_seq int64 walked_to_lumi_seq obj source_memory_paths "array" string synthesis_skill_version obj cluster_meta } 

### Filesystem layout
    
    
    .cyberos/memory/store/
    ├── manifest.json # store metadata (§6)
    ├── HEAD # 8-byte LE u64 seq counter; atomic rename
    ├──.lock # leased coordination record (§4.2)
    ├── audit/
    │ ├── current.binlog # active segment
    │ ├── 2026-04.binlog # sealed monthly segment
    │ ├── 2026-04.binlog.zst # compacted (zstd, deterministic)
    │ ├── checkpoints/ # per-consolidation Ed25519 STH anchors
    │ │ └── 2026-05-14T03Z-{root}.json
    │ └── *.jsonl # legacy v1 ledger (read-only post-cutover)
    ├── memories/
    │ ├── decisions/<2hex>/<2hex>/.md[.meta.json]
    │ ├── facts/...
    │ ├── people/...
    │ ├── projects/...
    │ ├── preferences/...
    │ ├── drift/...
    │ └── refinements/...
    ├── meta/ company/ module/ member/ client/ project/ persona/
    ├── conflicts/ # soft-tombstone bodies + sync-FS quarantine
    ├── exports/ # deterministic zip targets (excluded from chain)
    └── index/manifest.json # SQLite index rebuild marker

5

## API surface

memory exposes three surfaces: a GraphQL subgraph (P0 federation gateway · planned), an MCP tool catalogue (Claude / Codex / Cursor — partial), and a CLI (`cyberos` — 30 subcommands shipped). 

### GraphQL subgraph (planned · P0+)

Federated via Apollo Router v2.5+. Types are entity-key'd by store path so other subgraphs can extend them.
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@external", "@shareable"])
    
    type Memory @key(fields: "path") {
     path: String!
     kind: MemoryKind!
     bodyHash: String!
     state: MemoryState!
     syncClass: SyncClass!
     classification: Classification
     acl: [ACLEntry!]
     body: String # null if encrypted + caller lacks key
     createdAt: DateTime!
     updatedAt: DateTime!
    }
    
    type AuditRow @key(fields: "seq") {
     seq: Int!
     tsNs: BigInt!
     op: MemoryOp!
     path: String!
     bodyHash: String!
     prevChain: String!
     chain: String!
     actor: String!
     extra: JSON
    }
    
    type Checkpoint @key(fields: "filename") {
     filename: String!
     treeSize: Int!
     mmrRoot: String!
     signedAtNs: BigInt!
     signature: STHSignature!
    }
    
    type STHSignature { algorithm: String! pubkeyId: String! signature: String! }
    
    enum MemoryKind { decisions facts people projects preferences drift refinements }
    enum MemoryOp { put move delete }
    enum MemoryState { active tombstoned purged }
    enum SyncClass { private shareable }
    enum Classification { public internal confidential restricted }
    
    type Query {
     memory(path: String!): Memory
     memories(kind: MemoryKind, since: BigInt, limit: Int = 50): [Memory!]!
     auditRow(seq: Int!): AuditRow
     auditRows(fromSeq: Int!, limit: Int = 100): [AuditRow!]!
     checkpoint(filename: String!): Checkpoint
     state: MemoryState!
     inclusionProof(seq: Int!): InclusionProof!
    }
    
    type Mutation {
     put(path: String!, body: String!, meta: JSON): PutResult!
     move(src: String!, dst: String!): MoveResult!
     delete(path: String!, mode: DeleteMode! = TOMBSTONE, reason: String): DeleteResult!
     consolidate: Checkpoint! # admin
     import(sourceFingerprint: String!, bundle: Upload!): ImportSession!
    }
    
    enum DeleteMode { TOMBSTONE PURGE }

### MCP tool catalogue

Exposed via the MCP Gateway (P0 infra) to Claude / Codex / Cursor / any 2025-11-25-spec client. Tool annotations include capability scope so the broker can enforce least-privilege.

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`memory.put`| path, body, meta?| `{seq, chain, body_hash}`| destructive=false · idempotent=true · scope=write  
`memory.view`| path| `{body, meta, hash}`| destructive=false · readonly · scope=read  
`memory.move`| src, dst| `{seq, chain}`| destructive=true · scope=write  
`memory.delete`| path, mode, reason?| `{seq, chain, mode}`| destructive=true · purge requires gate · scope=delete  
`memory.search`| query, kind?, limit?| `Memory`| readonly · semantic + lexical · scope=read  
`memory.audit_tail`| fromSeq, limit| `AuditRow`| readonly · scope=audit  
`memory.prove`| seq| `InclusionProof`| readonly · MMR proof · scope=read  
`memory.verify_proof`| seq, proof| `{valid, root}`| readonly · scope=read  
`memory.state`| —| `MemoryState`| readonly · scope=read  
`memory.doctor`| repair?| `{invariants}`| readonly by default · scope=admin  
  
### CLI surface — `cyberos`

30 subcommands. Every chain-touching operation is single-writer and lease-protected. Examples below; full reference at CLI usage.

Subcommand| Purpose| Example  
---|---|---  
`cyberos put`| Append a memory file| `cyberos put memories/facts/x.md -`  
`cyberos view`| Read a memory body| `cyberos view memories/facts/x.md`  
`cyberos move`| Rename within store| `cyberos move a.md b.md`  
`cyberos delete`| Tombstone (default) or purge| `cyberos delete x.md --mode tombstone`  
`cyberos doctor`| Verify all invariants| `cyberos doctor [--repair]`  
`cyberos state`| Chain head + agent state| `cyberos state`  
`cyberos consolidate`| Walk → Compact → Sign → Publish| `cyberos consolidate`  
`cyberos prove`| MMR inclusion proof| `cyberos prove --seq 12345`  
`cyberos verify-proof`| Verify proof bundle| `cyberos verify-proof bundle.json`  
`cyberos export`| Deterministic zip| `cyberos export out.zip`  
`cyberos import`| Cross-memory merge| `cyberos import teammate.zip`  
`cyberos prune`| Sweep zstd-archived segments| `cyberos prune --older-than 90d`  
`cyberos backup`| Incremental snapshot| `cyberos backup./backups/`  
`cyberos serve`| HTTP REST mode| `cyberos serve --port 7878`  
`cyberos publish`| Mobile static site| `cyberos publish./site/`  
`cyberos digest`| Daily LLM summary| `cyberos digest --date today`  
`cyberos search`| Lexical + semantic| `cyberos search "Singapore HoldCo"`  
`cyberos validate`| Schema-validate frontmatter| `cyberos validate x.md`  
`cyberos walk`| Replay binlog| `cyberos walk --from-seq 0`  
`cyberos session start/end`| Bracket import or batch| `cyberos session start`  
_\+ 10 more:`encrypt`, `decrypt`, `rotate-key`, `conflicts`, `tail`, `show-config`, `show-manifest`, `checkpoints`, `repair-index`, `profile`._  
  
### CLI surface — universal protocol (Stage 1+ designed)

Eight new subcommands extend the existing 30. Implements §15.1 / §15.4 of [MEMORY_AUTOSYNC_DESIGN.md](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — locked 2026-05-14. All chain-touching operations remain single-writer + lease-protected.

Subcommand| Purpose| Stage| Example  
---|---|---|---  
`cyberos memory init`| Create `$HOME/.cyberos/memory/store/`; explicit opt-in privacy floor (watches nothing yet)| 1| `cyberos memory init`  
`cyberos memory watch <path>`| Add folder to capture daemon's watch list. Per-folder `--default-sync-class` \+ `--default-classification`| 1| `cyberos memory watch ~/Projects/cyberos --default-sync-class shareable`  
`cyberos memory unwatch <path>`| Remove folder from watch list. Captured memories remain; new activity in that folder stops landing on the chain| 1| `cyberos memory unwatch ~/Documents/Personal`  
`cyberos memory status`| Full Personal-memory digest: watched-folder list, last sync time, pending-PII count, audit-row count, doctor invariant summary| 1| `cyberos memory status [--json]`  
`cyberos memory capture <text-or-path>`| One-off manual capture (CLI fallback when no daemon-observed surface). Emits a `facts` or `discussions` memory depending on input| 1| `cyberos memory capture "Locked: AI Gateway ships before AUTH"`  
`cyberos memory sync`| Trigger sync window now (force push pending + pull latest). Subject to AUTH JWT + Lumi's memory reachability| 4| `cyberos memory sync [--push-only|--pull-only]`  
`cyberos memory sync-mode <mode>`| Set sync cadence: `realtime` (10 s) · `frequent` (1 min) · `normal` (5 min · default) · `infrequent` (1 hr) · `manual`| 4| `cyberos memory sync-mode normal`  
`cyberos memory pending`| List items the daemon is holding back: PII-detected sync candidates, new-folder defaults awaiting confirmation, conflict-resolution prompts| 2 / 4| `cyberos memory pending [--resolve]`  
`cyberos memory reclass <memory-id> <class>`| Change a memory's `sync_class` (private / personal / shareable / team-public / org-only). Propagates to next sync window| 1 / 4| `cyberos memory reclass memories/decisions/holdco-flip.md team-public`  
  
6

## Key flows

### Flow 1 — Write path (the hot path)

sequenceDiagram autonumber participant C as Caller (agent / CLI / subgraph) participant O as ops.put participant L as lock.acquire participant W as writer.append participant F as fsync.barrier participant M as mmr.append_leaf participant S as sth.maybe_sign participant H as HEAD seqlock participant I as index.upsert C->>O: put(path, body, meta) O->>O: validate path (no traversal, schema-conformant) O->>O: hash body (SHA-256) O->>L: acquire lease (10s TTL) L-->>O: lease(pid, monotonic_ns) O->>W: build frame {seq, ts_ns, op:"put", body_hash, prev_chain} W->>W: chain = SHA-256(canonical(frame) || prev_chain) W->>F: write tmpfile + rename + parent-dir fsync Note over F: macOS F_BARRIERFSYNC per-batch  
F_FULLFSYNC at checkpoint W->>M: append_leaf(chain) M-->>W: leaf_index, peaks W->>S: check trigger (every consolidation OR explicit) alt consolidate trigger S->>S: sign tree head (Ed25519) S-->>W: checkpoint written end W->>H: advance HEAD (atomic 8-byte rename) W->>I: upsert SQLite index row L-->>O: release lease O-->>C: {seq, chain, body_hash} 

Hot-path latency budget: ~120 µs group-committed (10+ concurrent writes share one `fsync`). Cold-path single writer: ~3 ms. STH signing: amortised over the consolidation window.

### Flow 2 — Read path (lock-free seqlock)

sequenceDiagram autonumber participant C as Caller participant R as reader.view participant H as HEAD seqlock participant FS as filesystem (mmap) C->>R: view(path) R->>H: snapshot HEAD seq (read-only) H-->>R: seq_pre R->>FS: mmap body file R->>FS: stat body file (mtime, ino, size) R->>H: re-read HEAD seq H-->>R: seq_post alt seq_pre == seq_post AND stat unchanged R-->>C: {body, meta, hash} (~6 µs hot) else inconsistent — writer raced R->>R: retry (bounded, exponential backoff) Note over R: max 3 retries; then fall back to lock-guarded read end 

No `.lock` acquisition on the read path. Workers / agents read concurrently at memory bandwidth.

### Flow 3 — Consolidation (Walk → Compact → Sign → Publish)

sequenceDiagram autonumber participant T as Trigger (size above 5 MB OR rows above 5,000 OR manual) participant C as consolidate.run participant WA as walker (verify) participant CO as compact (zstd archive) participant SI as sign (Ed25519 STH) participant PU as publish (advance manifest) T->>C: consolidate C->>WA: walk binlog from last checkpoint WA->>WA: verify frame CRC, chain continuity, MMR cross-check WA-->>C: invariants OK C->>CO: seal current.binlog → 2026-MM.binlog CO->>CO: zstd compress (deterministic, level 6) CO-->>C: 2026-MM.binlog.zst C->>SI: sign(tree_size, mmr_root) SI-->>C: checkpoint JSON with Ed25519 sig C->>PU: atomically advance manifest.audit_chain_head + last_sth PU-->>C: published C-->>T: {checkpoint_filename, tree_size} 

### Flow 4 — Conflict resolution & recovery (FROZEN_RECOVERABLE)

sequenceDiagram autonumber participant A as Agent (READY) participant CHK as Pre-write check (§1) participant DOC as cyberos doctor participant USER as Operator participant REP as doctor --repair A->>CHK: about to write CHK->>CHK: verify chain tip vs ledger alt divergent CHK->>A: transition FROZEN_RECOVERABLE A->>DOC: report inconsistency DOC->>DOC: run 15 invariants DOC->>USER: surface specific failure (e.g. chain gap @ seq 12345) USER->>REP: cyberos doctor --repair --reason "iCloud merge artefact" REP->>REP: safe auto-fixes only (rebuild index, replay tail) REP-->>A: state → READY (if all 15 pass) else catastrophic CHK->>A: transition FROZEN_HUMAN A->>USER: refuse all writes; human intervention required end 

### Flow 5 — GDPR purge (Article 17 right to erasure)

sequenceDiagram autonumber participant SUB as Data subject (user) participant OP as Operator (DPO) participant C as Chat session participant D as delete(path, "purge", reason) participant W as writer.append (purge row) SUB->>OP: DSAR / erasure request OP->>C: explicit chat-turn approval (§3.6 gate) C->>D: cyberos delete <path> \--mode purge --reason "DSAR-2026-014" D->>D: validate reason non-empty D->>W: append purge row {op:"delete", path, body_hash, mode:"purge", reason} W-->>D: chain committed D->>D: redact body file (zero-fill + unlink) D->>D: tombstone sidecar (retain meta minus PII) Note over W: The FACT of purge is itself a ledger leaf  
and is not itself erasable. D-->>OP: {seq, chain, redacted_hash} 

Per AGENTS.md §17.1, the audit fact of erasure is unerasable. The body is gone; the chain remembers the redaction.

7

## Memory file lifecycle

A single memory file traverses six states from authorship to long-term archive. The agent state machine (READY · FROZEN_RECOVERABLE · FROZEN_HUMAN) gates every transition that involves a write. 

stateDiagram-v2 [*] --> Composed: agent / user authors body + frontmatter Composed --> Submitted: ops.put(path, body, meta) Submitted --> Chained: writer appends audit frame · prev_chain + chain Chained --> Indexed: SQLite index upsert · MMR leaf append Indexed --> Signed: consolidation window OR explicit cyberos consolidate Signed --> Consolidated: STH written · manifest advanced · segment sealed Consolidated --> Exported: cyberos export → deterministic zip Exported --> Imported: cyberos import on a teammate's memory Consolidated --> Tombstoned: cyberos delete --mode tombstone Tombstoned --> Purged: cyberos delete --mode purge (DSAR gate) Purged --> [*]: body redacted · chain row preserved as erasure fact Consolidated --> Archived: cyberos prune (zstd) · move to cold tier (planned) Indexed --> Conflicted: sync-FS race detected Conflicted --> Indexed: cyberos conflicts resolve 

### Agent state machine

stateDiagram-v2 [*] --> READY READY --> FROZEN_RECOVERABLE: invariant failure (chain gap, MMR mismatch) FROZEN_RECOVERABLE --> READY: cyberos doctor --repair FROZEN_RECOVERABLE --> FROZEN_HUMAN: catastrophic divergence READY --> FROZEN_HUMAN: chain corruption or manifest unparseable FROZEN_HUMAN --> READY: cyberos doctor repair with reason flag note right of FROZEN_RECOVERABLE Reads OK · writes refused Auto-repair available end note note right of FROZEN_HUMAN Reads OK · writes refused Human signoff required end note 

8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

FR| Title| Status  
---|---|---  
`FR-MEMORY-101`| Layer 2 ingest pipeline — cursor + binlog_tail + chain_anchor + entity_extract + pgvector upsert + daemon main loop + /metrics| shipped · 2026-05-18 (Wave 1)  
`FR-MEMORY-108`| Apache AGE graph mirror + search API| planned · P1  
  
Additional memory FRs land here as they are re-authored via the `feature-request-author` skill.

9

## Non-Functional Requirements

Latency, throughput, and durability budgets specific to memory. Cross-referenced at [nfr-catalog.html#memory](<../../reference/nfr-catalog.html#memory>).

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Write p50 (group-committed)| ≤ 150 µs| `bench/throughput.py` · nightly  
`N(FR pending)`| Write p95 (group-committed)| ≤ 500 µs| bench/throughput · alert on regression > 10%  
`N(FR pending)`| Write p99 (cold, single writer)| ≤ 10 ms| bench/cold_cli.py  
`N(FR pending)`| Read p50 (mmap seqlock)| ≤ 10 µs| bench/reader.py  
`N(FR pending)`| Search p95 (lexical)| ≤ 50 ms over 10k memories| bench/search.py  
`N(FR pending)`| Search p95 (semantic, local BGE-M3)| ≤ 200 ms over 10k memories| bench/semantic.py  
`N(FR pending)`| Sustained write throughput| ≥ 8,000 ops/sec single thread| bench/throughput.py  
`N(FR pending)`| Durability guarantee| fsync-per-batch · 0 lost frames at process kill| bench/crash_recovery.py  
`N(FR pending)`| Chain integrity rate| 100% (15/15 invariants)| `cyberos doctor` · CI nightly  
`N(FR pending)`| Export determinism| byte-identical across 3 platforms| bench/determinism.py · macOS + Linux + WSL  
`N(FR pending)`| Storage overhead vs raw body| ≤ 35% (chain + sidecar + index)| bench/storage.py  
`N(FR pending)`| Single-process availability| ≥ 99.99% (local FS dependent)| observed from `cyberos state` uptime  
  
10

## Dependencies

memory is the deepest dependency in the CyberOS graph. Today (P0 · start) it has no module dependencies — it is the foundation. Once P0 infra lands, it picks up AUTH (for actor identity), the AI Gateway (for digest / semantic search), and the MCP Gateway (for tool exposure). 

graph LR subgraph upstream ["memory depends on"] AUTH["🔐 AUTH  
actor identity"] AI["⚡ AI Gateway  
digest · semantic"] MCP["🔌 MCP Gateway  
tool exposure"] end memory["🧠 memory"] subgraph downstream ["Used by all 22 modules"] CUO["🎯 CUO"] SKILL["🛠 SKILL"] CHAT["💬 CHAT"] EMAIL["✉️ EMAIL"] PROJ["📋 PROJ"] REW["💎 REW"] OTHERS["…16 more"] end AUTH --> memory AI --> memory MCP --> memory memory --> CUO memory --> SKILL memory --> CHAT memory --> EMAIL memory --> PROJ memory --> REW memory --> OTHERS classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class memory shipped class AUTH,AI,MCP planned 

11

## Compliance scope

memory is the single most compliance-relevant module in the catalog. Its audit chain is the regulator-facing evidence base for every decree below.

Regulation / standard| Article / clause| memory feature that satisfies it  
---|---|---  
Vietnam PDPL (Law 91/2025)| Art. 14 — DSAR| `cyberos search` \+ `cyberos export` produce a complete data-subject extract in < 1 min.  
Vietnam PDPL| Art. 16 — Erasure| `cyberos delete --mode purge` with reason gate; audit fact remains.  
Vietnam Decree 13/2023| Art. 17 — Personal data processing log| Audit chain is the processing log; every op recorded with actor.  
Vietnam Decree 53/2022| Art. 26 — Cybersecurity data localisation| Local-first by design; `.cyberos/memory/store/` stays on Vietnamese soil unless explicitly exported.  
GDPR (EU 2016/679)| Art. 17 — Right to erasure| Purge mode (§3.6) with reason gate; chain row records the redaction.  
GDPR| Art. 30 — Records of processing| Audit chain is the records-of-processing artefact.  
EU AI Act (Reg. 2024/1689)| Art. 12 — Logging| Every routing decision (via CUO) is recorded in memory; replayable.  
EU AI Act| Art. 26 — Human oversight| FROZEN_HUMAN state requires explicit signoff to recover; defer-to-human escapes encoded in audit rows.  
Singapore PDPA| §§ 12–13 — Access & correction| Same DSAR + erasure mechanisms; satisfied for HoldCo flip at P3.  
ISO/IEC 27001:2022| A.8.10 — Information deletion| Purge mode plus retention manifest.  
ISO/IEC 27001:2022| A.8.15 — Logging| Audit chain · 100% integrity rate.  
ISO/IEC 42001 (AIMS)| § 8.2 — Data management| Sync-class taxonomy + acl + classification + Ed25519-anchored chain.  
SOC 2 Type II| CC7.2 · CC7.3 — Monitoring| Doctor invariants run nightly; alert on first failure.  
Universal-protocol compliance scope (Stages 1–5)  
Vietnam PDPL (Law 91/2025)| Art. 7 — No personal data sale| Lumi's memory is multi-tenant but tenant-isolated; no inter-tenant memory transfer mechanism exists. Synthesis sub-skill operates within tenant boundary only. ToS for Lumi's memory explicitly prohibits selling tenant data.  
Vietnam PDPL| Art. 20 — Cross-border transfer (60-day post-audit)| Lumi's memory cloud deployment defaults to AWS Singapore. EU-resident tenants opt into `eu-fra-1` shard at P3; impact assessment submitted to MoPS within 60 days of first transfer. Per-tenant residency pinning via TEN module.  
Vietnam PDPL| Art. 38 — SME grace period| 5-year window from 2026-01-01 — small enterprises and start-ups may defer Art. 21 (impact assessment), DPO appointment, and certain other provisions. CyberSkill JSC is in scope. Lumi's memory tenants whose orgs qualify can opt out via TEN tenant config flag `pdpl_sme_grace = true` until 2031-01-01.  
EU AI Act (Reg. 2024/1689)| Art. 12 — Logging for the synthesis sub-skill| Stage 5 synthesis is a CUO sub-skill making content-influencing decisions; logged per Art. 12 with persona-version stamp, source-memory citations, and reproducibility-via-replay. Synthesis output is itself a memory, so Art. 12's logging requirement is satisfied by the memory protocol itself, not bolted on.  
EU AI Act| Art. 50 — Transparency for AI-generated content| Every synthesis artefact carries `extra.synthesised_by = "cuo/personas/synthesis-author@<version>"` in its audit frame. End-user surfaces (Lumi UI) label synthesised wisdom with an "AI synthesis" badge.  
ISO/IEC 27018:2019| § A.5 — Customer agreement| Lumi's memory customer agreement explicitly enumerates: which capture surfaces, default sync-class semantics, the synthesis sub-skill scope. Customers can opt out of synthesis per tenant via TEN flag.  
  
12

## Risk entries

memory-specific risks tracked in the full [risk register](<../../reference/risk-register.html#memory>).

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-memory-001`| Audit chain corruption (frame CRC failure, MMR peak mismatch)| Low| Catastrophic| CTO| 15 walker invariants nightly · FROZEN_RECOVERABLE auto-stop · `cyberos backup` daily.  
`R-memory-002`| MMR implementation bug (replay produces wrong root)| Low| High| CTO| Cross-check invariant compares `chain` against MMR-derived hash at every consolidation. 1k / 10k / 100k leaf scale tests in CI.  
`R-memory-003`| STH signing-key compromise| Low| High| CSO| Passphrase-wrapped key on disk. P3+: KMS-managed rotation. Re-sign all open checkpoints on rotation.  
`R-memory-004`| iCloud / Dropbox sync race produces conflicting bodies| Medium| Medium| CDO| `conflicts.py` quarantines competing copies to `conflicts/`; operator resolves via `cyberos conflicts resolve`.  
`R-memory-005`| Cross-platform fsync semantic drift (macOS vs Linux vs Windows)| Medium| Medium| CTO| Per-platform fsync.py paths, crash-recovery benchmark on each platform in CI.  
`R-memory-006`| Schema drift breaks downstream consumers (e.g. CUO bridge)| Low| Medium| CTO| `memory.schema.json` regenerated from types at every release · schema-drift regression test gate.  
`R-memory-007`| Operator accidentally purges legally-required data| Low| High| CLO| Purge mode requires explicit chat-turn approval AND non-empty reason. Two-person sign-off in production tenants (P3+).  
`R-memory-008`| Encryption envelope key loss (KMS outage / rotation bug)| Low| Catastrophic| CSO| Key escrow + N-of-M reconstruction (P2+). Sidecar always plaintext so metadata survives body loss.  
`R-memory-009`| **Lumi's memory tenant compromise** — credential theft or tenant-isolation bypass exposes shared org memories across tenants| Low| Catastrophic| CSO| Stage 3+: Postgres RLS by `tenant_id` on every table · per-tenant S3 prefix · per-user JWT scoped to tenant · weekly chaos-test of cross-tenant leak surface (read another tenant's row returns 403) · AUTH module's RBAC predicate gates every cross-module call.  
`R-memory-010`| **Sync conflict storm** — N personal memories producing high-volume concurrent shareable writes overwhelm Lumi's chain extension or produce cascading import-failures| Medium| High| CTO| Stage 4: per-tenant push-rate limit (default 100 ops/min/user) · server-side idempotency via content-hash dedup · client-side retry with exponential backoff + jitter · sync orchestrator surfaces stuck-state via `sync-orchestrator-not-stuck` doctor invariant · synthesis pass dampens duplicate decisions.  
`R-memory-011`| **Synthesis sub-skill hallucination** — Stage 5 wisdom artefact emits a false "synthesised decision" that does not actually appear in any source memory| Medium| High| CDO| Every synthesis claim hash-anchored to the underlying memory rows (citation-mandatory) · synthesis output is itself a memory and follows the protocol (replayable) · weekly user-flag "useful / not useful / wrong" → if "wrong" rate > 5% in any tenant, synthesis pauses for review · LLM caller stamped with persona-version per AGENTS.md §11.  
`R-memory-012`| **Capture daemon crash recovery** — daemon dies mid-capture and resumes producing duplicate or out-of-order memories| Medium| Medium| CTO| Content-addressed puts are idempotent per AGENTS.md §3.4 (duplicate emits become no-ops at writer). Daemon checkpoint state in `~/.cyberos/memory/store/capture-daemon.state.json`. `capture-daemon-not-stuck` doctor invariant warns if no emit in > 24 h on watched folder. launchd / systemd-user auto-restart on crash.  
`R-memory-013`| **iCloud / Dropbox sibling explosion** — when Personal memory store is itself synced by a cloud-FS provider, "filename (Mac).md" / "filename (iPhone).md" siblings multiply and the audit chain partitions| Medium| High| CDO| Existing `layout-no-sync-conflict-siblings` invariant detects siblings · operator-driven resolver at `cyberos conflicts resolve` · Stage 4 documentation explicitly recommends Lumi's memory sync over iCloud/Dropbox folder-sync as the cross-machine path (less brittle).  
`R-memory-014`| **PII leak via auto-capture** — capture daemon emits a memory containing PII before the user can mark it private; PII reaches Lumi's memory unintended| Medium| High| DPO| Stage 2 Presidio at capture write time — every PII-flagged memory is _held back from sync_ until user confirms via prompt UX (Cowork-inline / desktop-notif / `memory pending`). Default `sync_class` remains `private` unless folder override. Right-to-forget propagates: a purge on Personal memory issues `purge_propagate` to Lumi.  
  
13

## KPIs

Module health is read off these 8 KPIs. Targets are normative budgets; current values are observed on the user's live memory.

KPI| Formula| Source| Target| Current  
---|---|---|---|---  
**Chain integrity rate**| `passing_invariants / 15`| `cyberos doctor`| 100%| 15/15 (100%)  
**MMR cross-check pass rate**| `matched_leaves / total_leaves`| walker invariant| 100%| 100%  
**Write p95**|  group-committed| bench/throughput| ≤ 500 µs| ~ 380 µs  
**Read p95**|  mmap seqlock| bench/reader| ≤ 20 µs| ~ 8 µs  
**Search p95**|  lexical · 10k memories| bench/search| ≤ 50 ms| ~ 32 ms  
**Export determinism rate**| `identical_exports / runs`| bench/determinism| 100%| 100% (3 platforms)  
**Storage overhead**| `(chain + sidecar + index) / raw_body`| bench/storage| ≤ 35%| ~ 28%  
**Test pass rate**| `green / total`| pytest CI| ≥ 99%| 233/235 (99.1% — 2 pre-existing invariant-check bugs, non-blocking)  
Universal-protocol KPIs (Stages 1–5 — measured once each stage ships)  
**Capture rate per user**| `captured_memories / day / user`| capture daemon ledger| ≥ 20 / day / active user (signal floor) · ≤ 500 / day (noise ceiling)| — (Stage 2)  
**Sync success rate**| `successful_pushes / total_push_attempts`| sync orchestrator| ≥ 99.5% p95 over 7-day window| — (Stage 4)  
**Sync conflict rate**| `conflicts / total_imports`| sync orchestrator| ≤ 1% (per AGENTS.md §14.2 fresh-put semantics; most "conflicts" are content-different)| — (Stage 4)  
**Synthesis useful-rate**| `thumbs_up / (thumbs_up + thumbs_down + thumbs_wrong)`| synthesis sub-skill feedback rows| ≥ 75% per user per week (kill-switch if < 50%)| — (Stage 5)  
**Lumi's memory seq counter**| `monotone seq on Lumi tenant chain`| `cyberos --store lumi audit head`| ≥ 10× the largest single user's Personal memory seq (org-aggregate signal)| — (Stage 3)  
**PII held-back rate**| `held_back_for_pii / shareable_candidates`| capture daemon + Presidio| 3–8% expected band · alert if > 15% (too noisy) or < 1% (under-detecting)| — (Stage 2)  
**Capture daemon health**| `1 - (down_minutes / total_minutes)` over 7 days| `capture-daemon-not-stuck` doctor invariant| ≥ 99.9%| — (Stage 2)  
**Cross-machine portability**| `identical_chain_head_after_rsync / total_rsyncs`| portability bench| 100% (folder copy is the source of truth)| — (Stage 1)  
  
14

## RACI matrix

memory is owned end-to-end by the CEO seat today (Stephen Cheng). As the team grows, ownership shifts to the CDO (data) and CTO (engineering).

Activity| CEO| CTO| CDO| CSO| CLO| DPO  
---|---|---|---|---|---|---  
Protocol design (AGENTS.md)| A| R| C| C| I| I  
Implementation (cyberos/core/)| A| R| C| I| I| I  
On-call rotation| I| A| R| I| I| I  
Security review| I| C| C| A/R| C| I  
Compliance review (PDPL, GDPR)| I| C| C| C| A/R| R  
DSAR fulfilment| I| C| R| C| C| A/R  
Purge approval| C| I| C| I| A/R| R  
Key rotation (STH Ed25519)| I| C| I| A/R| I| I  
Stages 1–5 — universal protocol RACI additions (Cloud-DBA and Sync-SRE roles join at P3+)  
Stage 1: `memory init/watch/...` CLI design| A| R| C| I| I| I  
Stage 2: Capture daemon implementation (Rust)| I| A/R| C| I| I| I  
Stage 3: Lumi's memory cloud deployment + per-tenant RLS| I| C| A| R| I| I  
Stage 4: 2-way sync orchestrator| I| A/R| C| C| I| I  
Stage 5: Synthesis sub-skill + auto-evolve cadence| C| C| A/R| I| I| I  
Personal-memory portability across user's machines| I| C| A/R| I| I| I  
PII detection at capture write (Presidio configuration)| I| C| R| C| C| A  
Cross-tenant isolation testing (chaos)| I| R| I| A| C| I  
Synthesis-skill output review (weekly thumb-poll)| C| I| A/R| I| I| I  
  
**R** = Responsible · **A** = Accountable · **C** = Consulted · **I** = Informed. Stage-3+ adds Cloud-DBA (under CTO) and Sync-SRE (under CTO) as specialists.

15

## CLI usage — real examples

What a user actually types and what they see. All examples assume `pip install -e.` from the `memory/` directory.

### 1\. Append a memory
    
    
    $ echo "Decided to flip Singapore HoldCo if ARR ≥ \$1.5M at P3 exit." | \
     cyberos --store./.cyberos/memory/store --actor stephen put memories/decisions/holdco-flip.md -
    
    {
     "seq": 14823,
     "chain": "9f3e2a1b...8d4c",
     "body_hash": "a17b2c...e9f0",
     "elapsed_us": 412
    }

### 2\. Verify the entire chain
    
    
    $ cyberos --store./.cyberos/memory/store doctor
    
    [memory doctor v2.0.0] store=./.cyberos/memory/store rows=14823
    
    invariant status detail
    ─────────────────────────────────────────────────────
    chain-continuity OK 14823/14823 rows linked
    chain-no-rewind OK monotonic seq verified
    frame-crc OK all CRCs match
    op-enum-closed OK no unknown ops
    layout-canonical OK no sandbox path
    layout-no-traversal OK no '..' segments
    sidecar-body-hash OK all sidecars match body
    mmr-leaf-count OK 14823 leaves
    mmr-peak-cross-check OK peaks match chain hash
    sth-signature OK Ed25519 signature valid
    manifest-head-coherent OK HEAD matches manifest
    index-coverage OK SQLite mirrors filesystem
    schema-validation OK all frontmatter validates
    lease-not-stale OK no orphaned lease
    exports-excluded OK no chain leak through exports
    
    state: READY (15/15 invariants pass)

### 3\. Produce an MMR inclusion proof
    
    
    $ cyberos --store./.cyberos/memory/store prove --seq 14823 > proof.json
    
    $ cat proof.json
    {
     "seq": 14823,
     "leaf_hash": "9f3e2a1b...8d4c",
     "leaf_index": 14822,
     "proof_path": ["a1b2...", "c3d4...", "e5f6..."],
     "tree_size": 14823,
     "mmr_root": "f7e8...d9c0",
     "sth": {
     "filename": "2026-05-14T03Z-f7e8d9c0.json",
     "signature": "ed25519:8c7d..."
     }
    }
    
    $ cyberos verify-proof proof.json
    {"valid": true, "root": "f7e8...d9c0", "verified_at": "2026-05-14T07:21:08Z"}

### 4\. Deterministic export
    
    
    $ cyberos --store./.cyberos/memory/store export memory-2026-05-14.zip
    
    [export] paths=2451 bytes=14.2MB zip_bytes=4.1MB level=6
    [export] excluded: exports/ __pycache__/.cache/.lock HEAD
    [export] determinism: sorted paths · ts=2000-01-01T00:00:00Z · mode=0o644
    [export] written: memory-2026-05-14.zip
    [export] sha256: a4f9...8c2d (run twice on different machines → same hash)

### 5\. GDPR purge (Art. 17)
    
    
    $ cyberos --store./.cyberos/memory/store delete \
     memories/people/clients/acme-contact.md \
     --mode purge \
     --reason "DSAR-2026-014 erasure request, signed off by DPO"
    
    [delete] mode=purge body redacted audit row preserved
    {
     "seq": 14831,
     "chain": "b8d4...e3f7",
     "mode": "purge",
     "body_hash_before": "a17b...e9f0",
     "redaction_reason": "DSAR-2026-014 erasure request, signed off by DPO",
     "audit_fact_unerasable": true
    }

### 6\. Cross-memory import (P6 — teammate merge)
    
    
    $ cyberos --store./.cyberos/memory/store import../teammate-memory.zip --filter sync_class=shareable
    
    [import] source_fingerprint: 2a4d8f...1b3c
    [import] session.start written seq=14832
    [import] 103 memories considered -- 41 shareable, 62 private (skipped)
    [import] 41 imported (foreign_chain preserved in extra)
    [import] session.end written seq=14874
    [import] manifest.imports[2a4d8f...1b3c].last_imported_seq = 47812

15.5

## Developer quick start

For developers working on the Python implementation. The website covers the protocol; this section covers the codebase.

### Install
    
    
    # From repo root
    pip install -r memory/cyberos/requirements.txt
    
    # Or install in editable mode (exposes `cyberos` CLI)
    cd memory && pip install -e .

### Python dependencies

Package| Required?| Purpose  
---|---|---  
`msgspec ≥ 0.18`| yes| Canonical-JSON encoding; the whole hot path  
`crc32c ≥ 2.4`| **strongly recommended**|  SSE 4.2 / ARM CRC32 hardware path. Without it, falls back to `zlib.crc32` (different polynomial)  
`rfc8785 ≥ 0.1.4`| recommended| Used by migration preflight's `--strict-legacy-verify` mode  
`PyYAML ≥ 6.0`| recommended| Read-only legacy YAML frontmatter during migration window  
`uring`| optional (Linux)| io_uring linked WRITEV+FSYNC fast path; falls back transparently  
  
### Check CRC implementation
    
    
    python -c "from cyberos.core.writer import crc_implementation; print(crc_implementation())"
    # hw           → hardware-accelerated CRC-32C (correct)
    # zlib-fallback → install crc32c

### Benchmarks
    
    
    cd memory
    python -m bench.frontmatter --compare --files 2000   # msgspec vs PyYAML
    python -m bench.append --producers 1 --records 50000  # group-commit throughput
    python -m bench.append --producers 8 --records 50000
    python -m bench.cold_cli                              # cold CLI start
    python -m bench.determinism --store ../.cyberos/memory/store

Metric| Target| Typical (M2 MacBook)  
---|---|---  
Frontmatter parse p50 (msgspec)| <100 µs| ~0.6 µs  
Frontmatter parse p99 (msgspec)| <300 µs| ~1.0 µs  
Append throughput, 1 producer| 6,000/s| varies by SSD  
Append throughput, 8 producers| 9,000/s| varies by SSD  
Cold `cyberos --help`| <30 ms| ~10–25 ms  
Full chain verify, 100k records| <2 s| <1 s  
  
16

## Phase status & code stats

Total LoC (Python)

~5,000

core/ + tools/ + tests/

Test count

233/235

2 pre-existing invariant-check bugs · non-blocking

Modules in core/

25

writer, reader, walker, mmr, sth, …

CLI subcommands

30

single entrypoint `cyberos`

Audit proposals shipped

12 + Stage 3

P1–P12 + P2 Stage 3

Schema

closed

unknown `kind` rejected at write

Phase / capability| Status  
---|---  
Core writer + reader + walker| shipped  
MMR + Ed25519 STH| shipped  
Crypto-mode (STH-only)| shipped (opt-in)  
Cross-platform automation (launchd · systemd · Task Scheduler)| shipped  
Semantic search (optional `sentence-transformers`)| shipped  
Sync-FS conflict awareness (iCloud · Dropbox · OneDrive)| shipped  
P1–P12 audit proposals + P2 Stage 3| shipped  
Cross-memory import (P6)| shipped  
HTTP REST mode (`cyberos serve`)| shipped  
Daily digest (`cyberos digest`)| shipped  
Mobile static publish (`cyberos publish`)| shipped  
GraphQL subgraph (P0 federation)| planned · P0+  
Layer 2 ingest pipeline (services/memory/ Cargo workspace · cursor · binlog_tail · chain_anchor · entity_extract · pgvector upsert · daemon main loop · /metrics · 4 integration tests)| shipped · 2026-05-18 (Wave 1)  
Apache AGE graph mirror (FR-MEMORY-108 search API)| planned · P1  
S3 / R2 archival Layer 3| planned · P2  
iOS companion app| planned · P3+  
Public STH anchoring (transparency log)| planned · P3+  
Universal-protocol roadmap (Proposal P13)  
**Stage 1** — Personal memory universal protocol (any folder, portable, explicit-opt-in)  
CLI: `memory init/watch/unwatch/status/capture` · manifest schema + 2 new invariants| design-locked · ready to code  
**Stage 2** — Capture daemon (FS watcher + Cowork hook + Claude Code hook)  
Initial scope: Claude-only. Slack/Granola/Notes deferred.| design-locked · 2–4 weeks from Stage 1  
**Stage 3** — Lumi's memory cloud deployment  
Gated on AUTH + AI Gateway + TEN-billing thin slice| designed · P2 (P2 · exit)  
**Stage 4** — 2-way sync orchestrator  
Push / pull state machines · conflict resolution via chain-position + §14.2 fresh-put| designed · P2 (P2 · exit) parallel to Stage 3  
**Stage 5** — Multi-memory auto-evolve · synthesis sub-skill  
Nightly walk · cluster · dedup · synthesis@1 wisdom artefacts| designed · P3 (P3 · exit)  
  
17

## References

  * **MEMORY_AUTOSYNC_DESIGN.md** (archived 2026-05-18 — original 2026-05-14 design lock) — [universal Personal memory + Lumi's memory architecture](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>). Original product vision; live implementation guidance has migrated to `modules/memory/AGENTS.md` (Layer 1 spec), `modules/memory/README.md` (runtime + tests), `services/memory/README.md` (Layer 2 Wave 1), and the per-stage FRs under `docs/feature-requests/memory/` (FR-MEMORY-101…111).
  * **AGENTS.md** (RFC v2.0.0, normative) — `cyberos/modules/memory/AGENTS.md`. §3 canonical ops · §11 trust model · §14 cross-agent interop and the basis for the 2-way sync semantics · §15 sync_class taxonomy.
  * **PROPOSAL.md** — `cyberos/modules/memory/README.md (Appendix D — Proposals)` — where **Proposal P13** (universal protocol Stages 1–5) lands as formal protocol extension. P1–P12 + P2-Stage-3 already shipped.
  * **SPEC.md** — formal contract summary — `cyberos/modules/memory/docs/SPEC.md`.
  * **EVOLUTION.md** — history and rejected designs (informative).
  * **INTEROP.md** — Cursor / Codex-compatible read-only subset (≤ 6,000 chars).
  * **memory.schema.json** — JSON Schema for frontmatter, audit rows, manifest. P13 extends with `watched_folders`, `capture_event`, `sync_state`, `shared_memory_acl`, `synthesis_artefact` entities.
  * **memory.invariants.yaml** — invariant list consumed by the walker. P13 adds 4 invariants: `layout-watched-folders-exist`, `layout-watched-folders-permissions`, `capture-daemon-not-stuck`, `sync-orchestrator-running`.
  * **AUTHORING_DISCIPLINE.md** — [how new FRs are authored via the feature-request-author + feature-request-audit Agent Skill pair](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>). Every memory FR (Stages 1–5) lands via this workflow.
  * **archive/2026-05-14/AUDIT_AND_PLAN.md** — `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) placing Stages 3–5 on the P0+P2 critical path.
  * **archive/2026-05-14/RESEARCH_REVIEW.md** — `archive/2026-05-14/RESEARCH_REVIEW.md` (archived; see `cyberos/CHANGELOG.md`). §2 confirms memory as the strategic moat; §7 confirms multi-memory as the GTM answer.
  * **Source code:** `cyberos/modules/memory/cyberos/core/` · `cyberos/modules/memory/tests/` · `cyberos/modules/memory/bench/`. Stage 2 capture daemon lands at `cyberos/modules/memory/daemon/` (planned, Rust).
  * **CHANGELOG:** `cyberos/CHANGELOG.md (entries tagged [MEMORY])` (newest-first).



★

## Personas & skill bundles that touch memory

Every persona writes to memory through its workflows' audit chain. The personas below have memory as a first-class concern of their role profile — they govern memory's shape, classification, and synthesis. The CUO supervisor itself emits one memory audit row per workflow step + one summary row per chain (see `modules/cuo/cuo/core/memory_bridge.py`).

Persona governance (6 of 47)

  * [chief-knowledge-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-knowledge-officer/workflows>) · memory graph governance + canonicalisation pipeline
  * [chief-data-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-data-officer/workflows>) · memory data quality + lineage + retention policy
  * [chief-privacy-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-privacy-officer/workflows>) · DSR fulfilment by memory sync-class enforcement
  * [chief-information-security-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-information-security-officer/workflows>) · memory audit-chain integrity + 72h breach readiness
  * [chief-trust-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-trust-officer/workflows>) · memory-backed transparency report + trust-incident updates
  * [chief-ai-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-ai-officer/workflows>) · LLMInvoker prompt + cost logging into memory



Skill bundles that emit memory rows

  * **Every author skill** — emits a `view` row per step output (the chain ledger receipt)
  * **Every audit skill** — emits a `session.end` row when PASS / HITL_PAUSE / EXHAUSTED / NO_PROGRESS reached
  * [knowledge-pipeline-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/knowledge-pipeline-author>) \+ audit · CKO's memory-canonical pipeline
  * [data-governance-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/data-governance-author>) \+ audit · CDO memory-classification policy
  * [breach-notification-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/breach-notification-author>) \+ audit · 72-hour GDPR Art. 33 / PDPL Art. 16 timer pulled from memory audit chain
  * **synthesis-author** (Stage 5 CUO sub-skill) · nightly walks Lumi's memory, emits `synthesis@1` wisdom artefacts



[← All modules](<../index.html#catalog>) [Next module: CUO →](<../cuo/index.html>)
