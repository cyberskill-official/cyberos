---
id: FR-BRAIN-107
title: "BRAIN capture daemon — Rust + notify crate FS watcher with rate-limit + content-dedup + backpressure + W3C trace propagation"
module: BRAIN
priority: MUST
status: building
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-BRAIN-101, FR-BRAIN-102, FR-BRAIN-104, FR-BRAIN-105, FR-BRAIN-108, FR-BRAIN-109, FR-BRAIN-110, FR-BRAIN-111]
depends_on: [FR-BRAIN-102, FR-BRAIN-105, FR-BRAIN-106]
blocks: [FR-BRAIN-108, FR-BRAIN-109, FR-BRAIN-110, FR-BRAIN-111]

source_pages:
  - website/docs/modules/brain.html#capture-daemon
  - website/docs/runbooks/brain-capture-runbook.html
source_decisions:
  - DEC-140 (FS watcher is the canonical capture path; never poll, always event-driven)
  - DEC-141 (content-hash dedup BEFORE emit — same byte stream = same memory; rename ≠ new memory)
  - DEC-142 (per-folder per-second rate limit; burst tolerated via bounded queue + drop-with-audit on overflow)
  - AGENTS.md §11 (FS event bodies are untrusted text; only the canonical-writer mutates the audit chain)

language: rust 1.81
service: cyberos/services/brain-capture/
new_files:
  - services/brain-capture/Cargo.toml
  - services/brain-capture/src/main.rs
  - services/brain-capture/src/lib.rs
  - services/brain-capture/src/watcher.rs
  - services/brain-capture/src/dedup.rs
  - services/brain-capture/src/rate_limit.rs
  - services/brain-capture/src/queue.rs
  - services/brain-capture/src/emit.rs
  - services/brain-capture/src/trace.rs
  - services/brain-capture/tests/watcher_test.rs
  - services/brain-capture/tests/dedup_test.rs
  - services/brain-capture/tests/rate_limit_test.rs
  - services/brain-capture/tests/end_to_end_test.rs
modified_files:
  - cyberos/Cargo.toml                                      # workspace member
  - services/brain/src/cli/watch.rs                         # `cyberos brain watch` registers folder with capture daemon
  - services/brain/manifest.json                            # add `capture_daemon` section
allowed_tools:
  - file_read: services/brain-capture/**, services/brain/**
  - file_write: services/brain-capture/{src,tests}/**
  - bash: cd services/brain-capture && cargo test
  - bash: cd services/brain-capture && cargo build --release
disallowed_tools:
  - poll-based watching (DEC-140 — always event-driven)
  - bypass dedup (DEC-141 — same content-hash = same memory, idempotently)
  - silently drop events on queue overflow (per §1 #7 — must emit `brain.capture_dropped` audit row)
  - write directly to audit chain (per AGENTS.md §14.1 — only canonical writer mutates chain)

effort_hours: 14
sub_tasks:
  - "0.5h: Cargo.toml — notify@6, tokio, ring, blake3, governor, anyhow, tracing-opentelemetry"
  - "0.5h: lib.rs — re-exports + CaptureDaemon struct skeleton"
  - "1.0h: watcher.rs — notify::recommended_watcher with debounce; emit raw FsEvent into bounded queue"
  - "1.5h: dedup.rs — blake3 content hash + LRU cache (10k entries, 5-minute TTL) for rename + churn detection"
  - "1.0h: rate_limit.rs — governor@0.6 per-folder + per-tenant token buckets (50 events/s sustained, 200 burst)"
  - "1.5h: queue.rs — tokio::sync::mpsc bounded(10000) with drop-with-audit semantics"
  - "1.0h: emit.rs — bridge to FR-BRAIN-101 brain_writer; carries W3C TraceContext; retries with exp backoff"
  - "0.5h: trace.rs — W3C trace_id generation per capture batch + per-event child span"
  - "0.5h: main.rs — daemon entry point; reads manifest; spawns one watcher task per watched folder"
  - "1.5h: watcher_test.rs — single-file write → 1 capture row; sub-second latency"
  - "1.0h: dedup_test.rs — same content hashes identically; rename ≠ new memory"
  - "1.0h: rate_limit_test.rs — 1000-event burst → ≤ 200 emitted in first second; overflow → `brain.capture_dropped` row"
  - "1.5h: end_to_end_test.rs — write 100 files to watched folder → 100 capture rows in BRAIN; chain_anchor verified"
  - "1.0h: integrate with FR-BRAIN-102's `brain watch` (registration writes manifest entry; daemon picks up via SIGHUP)"
risk_if_skipped: "Without an event-driven watcher, capture latency is bounded by polling interval (minutes) — by then the user has moved on. Without dedup, rapid auto-save (editor write-on-keystroke) creates thousands of duplicate rows per minute. Without rate-limit, a runaway file-generation script could flood the BRAIN chain and trigger backpressure across the system. Without W3C trace propagation, downstream consumers can't correlate a capture row with the upstream tool call that produced it (cowork session, Claude Code hook, terminal command). Without `brain.capture_dropped` rows on overflow, observability sees the event count drop and assumes the user stopped working — silent data loss. This FR is load-bearing for FR-BRAIN-108 (Cowork session capture), FR-BRAIN-109 (Claude Code hook capture), FR-BRAIN-110 (health daemon), FR-BRAIN-111 (pre-ingest PII)."
---

## §1 — Description (BCP-14 normative)

The BRAIN capture daemon **MUST** be an event-driven filesystem watcher per watched folder, emitting canonical BRAIN audit rows for every meaningful change. The daemon's contract:

1. **MUST** use `notify::recommended_watcher` (notify@6) — on macOS this resolves to FSEvents, on Linux to inotify, on Windows to ReadDirectoryChangesW. Polling MUST NOT be used (per DEC-140).
2. **MUST** debounce per-file events at 250ms — editors that perform multi-step atomic writes (write tmp + fsync + rename) generate 3–5 events per save; the debouncer coalesces them into one logical event.
3. **MUST** compute a `blake3` content hash on the post-debounce file body BEFORE emitting any audit row. The hash becomes the memory's content key (DEC-141): same hash = same memory.
4. **MUST** maintain an in-process LRU cache (10K entries, 5-minute TTL) mapping `content_hash → last_path_seen + last_seq`. A cache hit on (a) the same path = no-op (idempotent), (b) a different path = `brain.capture_renamed` audit row referencing the prior seq.
5. **MUST** enforce per-folder + per-tenant token-bucket rate limits via `governor@0.6`:
    - Per-folder: 50 events/sec sustained, 200 burst.
    - Per-tenant: 500 events/sec sustained, 2000 burst.
    - First exceeded limit wins (the stricter one applies).
6. **MUST** route post-debounce + post-dedup + post-rate-limit events through a bounded `tokio::sync::mpsc` queue of capacity 10000. The producer (watcher) backpressures when full; the consumer (emitter) drains as fast as `brain_writer` can absorb.
7. **MUST** emit a `brain.capture_dropped` BRAIN audit row when the queue overflows (producer cannot enqueue within 100ms). The dropped row carries `folder_path`, `event_kind` (Create | Modify | Delete | Rename), `content_hash`, `dropped_at_ns`, `reason: queue_overflow` so the operator can see "the daemon dropped N events in the last minute and here's why."
8. **MUST** propagate W3C TraceContext per FR-AI-022: every capture batch carries a `traceparent`; if the upstream tool (Cowork session, Claude Code hook) injects one via the environment, use it; otherwise generate a fresh trace_id at the daemon boundary. The trace_id appears in every emitted audit row's `payload.trace_id`.
9. **MUST** support per-folder include/exclude globs from `manifest.watched_folders[N].include` and `exclude` (default include: `**/*`; default exclude: `node_modules/**`, `target/**`, `.git/**`, `*.tmp`, `*.swp`, `.DS_Store`). Globs use the `globset@0.4` crate.
10. **MUST** emit one canonical row per surviving event:
    - File created → `brain.capture_created` with `{folder_path, relative_path, content_hash, byte_count, mtime_ns, trace_id}`.
    - File modified → `brain.capture_modified` with same fields plus `prior_content_hash`.
    - File renamed → `brain.capture_renamed` with `{folder_path, from_relative_path, to_relative_path, content_hash, trace_id}`.
    - File deleted → `brain.capture_deleted` with `{folder_path, relative_path, last_content_hash, last_seen_at, trace_id}`.
11. **MUST** be crash-safe: on daemon startup, perform a full scan of every watched folder, hash every file, compare to last-known state (recovered from the BRAIN chain via `brain_reader`), and emit catch-up rows for any divergence. The scan emits a single `brain.capture_resync_started` row at start, a `brain.capture_resync_completed` at end with `{files_scanned, captures_emitted, duration_ms}`.
12. **MUST** complete the startup resync in ≤ 60s for ≤ 100K files; assertion in `end_to_end_test::resync_latency`. Folders exceeding this size print a sev-2 warning at boot.
13. **MUST** integrate with FR-BRAIN-105's doctor invariants: `cyberos brain capture status` invokes `doctor --only watched-folders`; refuses to start if any error-severity invariant fails.
14. **MUST** emit OTel metrics:
    - `brain_capture_events_total{folder_id, kind, outcome}` (counter; outcome ∈ emitted | dedup_skip | rate_limited | dropped).
    - `brain_capture_emit_latency_seconds{folder_id}` (histogram; FR-OBS-003 buckets).
    - `brain_capture_queue_depth{folder_id}` (gauge).
    - `brain_capture_dedup_cache_hit_ratio` (gauge; cache hits / (hits + misses)).
    - `brain_capture_resync_files_total` (counter).
15. **MUST** receive SIGHUP to reload `manifest.watched_folders`; new folders begin watching, removed folders stop (event handles dropped, dedup cache entries pruned). No restart required for folder add/remove.
16. **SHOULD** support `cyberos brain capture --foreground` for ops debugging (no daemonisation; logs to stderr).
17. **SHOULD** support `cyberos brain capture --dry-run` (don't emit; just print what would emit) for operator-facing change preview.

---

## §2 — Why this design (rationale for humans)

**Why notify::recommended_watcher (§1 #1)?** Cross-platform consistency is critical — operators run cyberos on Mac (FSEvents), Linux servers (inotify), and occasionally Windows VMs (ReadDirectoryChangesW). The `recommended_watcher` API picks the right backend per OS; rolling our own would mean three platform implementations + edge cases per platform. notify@6 is the maintained-by-Rust-async-community crate and matches our tokio runtime.

**Why 250ms debounce (§1 #2)?** Empirical: VS Code's atomic-save sequence is `tmp.create → tmp.write → tmp.fsync → tmp.rename → final.fsync`. On macOS FSEvents that's 4 events within ~80ms. JetBrains IDEs are similar. 250ms safely catches all editor variants while keeping latency low (a human writes once per minute or so; 250ms is invisible).

**Why blake3 for content hash (§1 #3)?** blake3 is faster than sha256 (about 2x on commodity hardware), still cryptographically secure, and produces a 32-byte digest that fits naturally in `content_hash` columns. We don't need sha256's specific properties (FIPS-140) for this use; we just need a deterministic, fast, collision-resistant hash. blake3 wins.

**Why dedup BEFORE emit (§1 #3 + #4)?** Without dedup, "user opens file, types one character, save" creates a new row identical in content to the prior version (because of editor auto-save on-save). The BRAIN chain accumulates noise. With dedup, identical content = identical row = idempotent no-op. The rename case (cache hit on different path) is also load-bearing: `git stash` renames files; without rename detection the same content would emit two rows.

**Why bounded queue + drop-with-audit (§1 #6 + #7)?** Two non-negotiables: (a) the daemon must NOT silently lose events (DEC-021), (b) it must NOT OOM under a runaway file-generation script. The bounded queue + drop-with-audit pattern satisfies both: backpressure under normal load, drop with full audit-row evidence under overload. Operators see exactly when the queue was full and what was dropped.

**Why W3C TraceContext propagation (§1 #8)?** A capture row in BRAIN with `trace_id: 0af7...` is correlatable to a Cowork session that wrote the file. Without propagation, the row is orphaned — operators can see "BRAIN gained a row at 14:32:17.483 with content hash X" but can't ask "who wrote this?" With propagation, OBS dashboards can build "capture rows by source tool" pivots.

**Why include/exclude globs (§1 #9)?** Operators don't want every `node_modules/` change tracked (thousands of files churn per `npm install`). The default exclude list catches the obvious noise; per-folder include/exclude lets advanced users customise (e.g. exclude `**/*.log` for a server-log folder where log rotation is high-frequency-low-information).

**Why startup resync (§1 #11)?** The daemon may have been down (crash, planned maintenance, OS reboot). Files were created / modified / deleted while it was off. Without resync, those changes are silently lost from the BRAIN's perspective. The resync compares filesystem reality to last-known BRAIN state and emits catch-up rows. The 60-second budget for 100K files is the operator-friendly upper bound; exceeded → sev-2 alarm so the operator knows to investigate.

**Why per-folder per-tenant rate limit (§1 #5)?** Per-folder protects against one runaway folder (e.g. a build script writing to `tmp/`). Per-tenant protects against many folders aggregating to a flood. The "first exceeded wins" rule is simple to reason about: if either limit hits, the daemon waits.

**Why SIGHUP reload (§1 #15)?** Operators add and remove watched folders frequently. Restarting the daemon for each change would: (a) trigger a full resync (60s pause); (b) lose in-flight events; (c) be operationally hostile. SIGHUP is the standard Unix way to "reload config without restart" and our manifest change is exactly that.

**Why no `cyberos brain capture stop` command?** The daemon is managed by systemd / launchd (FR-BRAIN-110); stopping it is `systemctl stop` etc. Adding a CLI stop creates two ways to do the same thing — the systemd unit is authoritative.

---

## §3 — API contract

### Cargo.toml

```toml
# services/brain-capture/Cargo.toml
[package]
name        = "cyberos-brain-capture"
version     = "0.1.0"
edition     = "2021"
description = "Event-driven FS watcher → BRAIN capture daemon"

[dependencies]
tokio       = { version = "1.40", features = ["rt-multi-thread", "macros", "sync", "signal", "fs", "io-util", "time"] }
notify      = "6"
notify-debouncer-full = "0.3"
blake3      = "1.5"
governor    = "0.6"
globset     = "0.4"
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
anyhow      = "1"
thiserror   = "1"
tracing     = "0.1"
tracing-opentelemetry = "0.23"
opentelemetry = "0.22"
opentelemetry-otlp    = "0.15"
lru         = "0.12"

# Shared internal crates
cyberos-brain-writer  = { path = "../brain/brain-writer" }
cyberos-brain-reader  = { path = "../brain/brain-reader" }
cyberos-obs-sdk       = { path = "../obs-sdk" }
cyberos-cli-exit      = { path = "../../crates/cyberos-cli-exit" }

[dev-dependencies]
tempfile    = "3"
tokio       = { version = "1.40", features = ["test-util"] }
```

### Watcher core

```rust
// services/brain-capture/src/watcher.rs
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct FolderWatcher {
    pub folder_id:   String,                  // stable id from manifest (UUID)
    pub realpath:    PathBuf,                 // canonicalised
    pub tenant_id:   uuid::Uuid,
    pub include:     globset::GlobSet,
    pub exclude:     globset::GlobSet,
    pub event_tx:    mpsc::Sender<RawEvent>,  // bounded queue, see queue.rs
    _watcher:        RecommendedWatcher,      // owned; dropped on shutdown
}

#[derive(Clone, Debug)]
pub struct RawEvent {
    pub folder_id:     String,
    pub tenant_id:     uuid::Uuid,
    pub kind:          EventKind,
    pub path:          PathBuf,            // relative to folder root
    pub mtime_ns:      i64,
    pub byte_count:    u64,
    pub captured_at_ns: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind { Created, Modified, Renamed { from: PathBuf }, Deleted }

impl FolderWatcher {
    pub async fn spawn(folder: WatchedFolder, event_tx: mpsc::Sender<RawEvent>) -> anyhow::Result<Self> {
        let (debouncer_tx, mut debouncer_rx) = tokio::sync::mpsc::channel(2048);
        let realpath = std::fs::canonicalize(&folder.path)?;
        let realpath_for_handler = realpath.clone();
        let folder_id = folder.id.clone();
        let tenant_id = folder.tenant_id;

        let mut debouncer = new_debouncer(
            Duration::from_millis(250),
            None,
            move |res: Result<Vec<DebouncedEvent>, notify::Error>| {
                if let Ok(events) = res {
                    for ev in events {
                        let _ = debouncer_tx.blocking_send(ev);
                    }
                }
            },
        )?;
        debouncer.watcher().watch(&realpath, RecursiveMode::Recursive)?;

        let include = build_globset(&folder.include)?;
        let exclude = build_globset(&folder.exclude)?;

        // Spawn task that forwards debounced events into the shared bounded queue
        let event_tx_clone = event_tx.clone();
        let folder_id_clone = folder_id.clone();
        let include_clone   = include.clone();
        let exclude_clone   = exclude.clone();
        tokio::spawn(async move {
            while let Some(ev) = debouncer_rx.recv().await {
                for raw in classify_event(ev, &realpath_for_handler, &folder_id_clone, tenant_id) {
                    // include/exclude glob check
                    if exclude_clone.is_match(&raw.path) { continue; }
                    if !include_clone.is_match(&raw.path) { continue; }
                    // Backpressure on full queue (100ms timeout; then drop-with-audit)
                    if (event_tx_clone.send_timeout(raw.clone(), Duration::from_millis(100)).await).is_err() {
                        emit::dropped(&raw, "queue_overflow").await;
                    }
                }
            }
        });

        Ok(Self {
            folder_id, realpath, tenant_id,
            include, exclude, event_tx,
            _watcher: debouncer.watcher().clone(),  // keep alive
        })
    }
}

fn build_globset(patterns: &[String]) -> anyhow::Result<globset::GlobSet> {
    let mut builder = globset::GlobSetBuilder::new();
    for p in patterns {
        builder.add(globset::Glob::new(p)?);
    }
    Ok(builder.build()?)
}

fn classify_event(ev: DebouncedEvent, root: &Path, folder_id: &str, tenant_id: uuid::Uuid) -> Vec<RawEvent> {
    use notify::EventKind as NotifyEventKind;
    // map notify EventKind → our EventKind; one DebouncedEvent may yield 0..N RawEvents
    // (e.g. rename = 1 RawEvent of EventKind::Renamed)
    // Implementation handles macOS FSEvents quirks (which often coalesce create+modify).
    todo!("see implementation; ~50 lines of pattern-match per platform")
}
```

### Dedup cache

```rust
// services/brain-capture/src/dedup.rs
use blake3;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct DedupCache {
    inner: Mutex<LruCache<[u8; 32], CachedEntry>>,
    ttl:   Duration,
}

#[derive(Clone)]
pub struct CachedEntry {
    pub last_path:  PathBuf,
    pub last_seq:   u64,            // BRAIN chain seq at which we last emitted for this hash
    pub stored_at:  Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DedupVerdict {
    EmitCreated,                    // first sighting
    EmitModified { prior_hash: [u8; 32] },
    EmitRenamed { from: PathBuf },  // same hash, different path
    Idempotent,                     // same hash, same path → no-op
}

impl DedupCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(LruCache::new(NonZeroUsize::new(10_000).unwrap())),
            ttl:   Duration::from_secs(300),
        }
    }
    pub fn classify(&self, hash: [u8; 32], path: &Path, prior_path_hash: Option<[u8; 32]>) -> DedupVerdict {
        let mut g = self.inner.lock().unwrap();
        if let Some(e) = g.get(&hash) {
            if e.stored_at.elapsed() < self.ttl {
                if e.last_path == path {
                    return DedupVerdict::Idempotent;
                } else {
                    return DedupVerdict::EmitRenamed { from: e.last_path.clone() };
                }
            }
        }
        // No fresh cache hit → either first sighting OR modified
        match prior_path_hash {
            Some(prior) if prior != hash => DedupVerdict::EmitModified { prior_hash: prior },
            _                            => DedupVerdict::EmitCreated,
        }
    }
    pub fn record(&self, hash: [u8; 32], path: PathBuf, seq: u64) {
        self.inner.lock().unwrap().put(hash, CachedEntry {
            last_path: path, last_seq: seq, stored_at: Instant::now(),
        });
    }
    pub fn cache_hit_ratio(&self) -> f64 {
        // counters updated externally by the metrics layer; ratio computed there.
        unimplemented!("metric computed in obs-sdk")
    }
}
```

### Rate limiter

```rust
// services/brain-capture/src/rate_limit.rs
use governor::{Quota, RateLimiter as Governor};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct RateLimits {
    per_folder: Mutex<HashMap<String, Arc<Governor<NotKeyed, InMemoryState, DefaultClock>>>>,
    per_tenant: Mutex<HashMap<Uuid,   Arc<Governor<NotKeyed, InMemoryState, DefaultClock>>>>,
}

const FOLDER_BURST:    NonZeroU32 = NonZeroU32::new(200).unwrap();
const FOLDER_SUSTAINED: NonZeroU32 = NonZeroU32::new(50).unwrap();
const TENANT_BURST:    NonZeroU32 = NonZeroU32::new(2000).unwrap();
const TENANT_SUSTAINED: NonZeroU32 = NonZeroU32::new(500).unwrap();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitVerdict { Allowed, RateLimited { which: &'static str } }

impl RateLimits {
    pub fn new() -> Self { Self {
        per_folder: Mutex::new(HashMap::new()),
        per_tenant: Mutex::new(HashMap::new()),
    }}
    pub fn check(&self, folder_id: &str, tenant_id: Uuid) -> LimitVerdict {
        let folder_rl = self.folder_governor(folder_id);
        let tenant_rl = self.tenant_governor(tenant_id);
        // First exceeded wins; check tenant first because it's the broader bucket
        if tenant_rl.check().is_err() { return LimitVerdict::RateLimited { which: "tenant" }; }
        if folder_rl.check().is_err() { return LimitVerdict::RateLimited { which: "folder" }; }
        LimitVerdict::Allowed
    }
    fn folder_governor(&self, folder_id: &str) -> Arc<Governor<NotKeyed, InMemoryState, DefaultClock>> {
        let mut g = self.per_folder.lock().unwrap();
        g.entry(folder_id.into()).or_insert_with(|| {
            Arc::new(Governor::direct(Quota::per_second(FOLDER_SUSTAINED).allow_burst(FOLDER_BURST)))
        }).clone()
    }
    fn tenant_governor(&self, tenant_id: Uuid) -> Arc<Governor<NotKeyed, InMemoryState, DefaultClock>> {
        let mut g = self.per_tenant.lock().unwrap();
        g.entry(tenant_id).or_insert_with(|| {
            Arc::new(Governor::direct(Quota::per_second(TENANT_SUSTAINED).allow_burst(TENANT_BURST)))
        }).clone()
    }
}
```

### Emit (BRAIN bridge)

```rust
// services/brain-capture/src/emit.rs
use crate::watcher::{RawEvent, EventKind};
use cyberos_brain_writer::{BrainWriter, AuditRow, canonical};
use opentelemetry::trace::TraceContextExt;
use opentelemetry::Context;
use tracing::Instrument;

pub async fn emit_capture(
    writer: &BrainWriter,
    raw: &RawEvent,
    content_hash: [u8; 32],
    dedup: crate::dedup::DedupVerdict,
    trace_id: String,
) -> anyhow::Result<()> {
    let span = tracing::info_span!(
        "brain.capture.emit",
        folder_id  = %raw.folder_id,
        kind       = ?raw.kind,
        bytes      = raw.byte_count,
        trace_id   = %trace_id,
    );
    async move {
        let kind_str = match (&raw.kind, dedup) {
            (EventKind::Created, _)             => "brain.capture_created",
            (EventKind::Modified, _)            => "brain.capture_modified",
            (EventKind::Renamed { .. }, _)      => "brain.capture_renamed",
            (EventKind::Deleted, _)             => "brain.capture_deleted",
        };
        let mut payload = serde_json::json!({
            "folder_id":     raw.folder_id,
            "relative_path": raw.path,
            "content_hash":  hex::encode(content_hash),
            "byte_count":    raw.byte_count,
            "mtime_ns":      raw.mtime_ns,
            "trace_id":      trace_id,
        });
        if let EventKind::Renamed { from } = &raw.kind {
            payload["from_relative_path"] = serde_json::json!(from);
        }
        if let crate::dedup::DedupVerdict::EmitModified { prior_hash } = dedup {
            payload["prior_content_hash"] = serde_json::json!(hex::encode(prior_hash));
        }
        writer.emit(AuditRow {
            kind: kind_str.into(),
            payload,
            tenant_id: raw.tenant_id,
        }).await
    }.instrument(span).await
}

pub async fn dropped(raw: &RawEvent, reason: &str) {
    // Best-effort emit (no retry); if the writer is also unavailable, structured log
    tracing::warn!(
        folder_id = %raw.folder_id,
        kind      = ?raw.kind,
        reason,
        "capture event dropped"
    );
}
```

### CLI

```rust
// services/brain-capture/src/main.rs
use clap::Parser;
use cyberos_cli_exit::ExitCode;

#[derive(Parser)]
#[command(name = "cyberos-brain-capture")]
struct Cli {
    /// Run in foreground (no fork; logs to stderr)
    #[arg(long)] foreground: bool,
    /// Print events without emitting (operator preview)
    #[arg(long)] dry_run: bool,
    /// Manifest path (defaults to <memory-root>/manifest.json)
    #[arg(long)] manifest: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    cyberos_obs_sdk::init("brain-capture");

    // Gate on doctor (§1 #13)
    if let Err(e) = run_doctor_gate().await {
        tracing::error!(?e, "doctor invariant failure; refusing to start");
        return ExitCode::InternalError;
    }

    let daemon = match cyberos_brain_capture::CaptureDaemon::start(cli.manifest, cli.dry_run).await {
        Ok(d) => d,
        Err(e) => { tracing::error!(?e, "daemon start failed"); return ExitCode::InternalError; }
    };

    // SIGHUP reload, SIGTERM graceful stop
    let mut sighup  = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()).unwrap();
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    loop {
        tokio::select! {
            _ = sighup.recv()  => { if let Err(e) = daemon.reload().await { tracing::error!(?e, "reload failed"); } }
            _ = sigterm.recv() => { daemon.stop().await; break; }
        }
    }
    ExitCode::Ok
}
```

---

## §4 — Acceptance criteria

1. **Single-file write captures one row** — write `foo.txt` to a watched folder → exactly one `brain.capture_created` row in BRAIN within 500ms; `content_hash` = blake3(file body); `trace_id` is a 32-char hex.
2. **Editor atomic-save coalesced** — VS Code save sequence (tmp create + rename) produces ONE `brain.capture_created` row, not 3.
3. **Idempotent re-save** — write same bytes to same path twice → only one row emitted; second save is `DedupVerdict::Idempotent`.
4. **Modify with content change emits prior_content_hash** — write file v1, then v2 → second row is `brain.capture_modified` carrying `prior_content_hash = hash(v1)`.
5. **Rename detected via dedup cache** — write `foo.txt`, then rename to `bar.txt` → row 2 is `brain.capture_renamed` with `from_relative_path: foo.txt` and same `content_hash` as row 1.
6. **Delete emits last_content_hash** — write + delete → second row is `brain.capture_deleted` with `last_content_hash` matching the prior row.
7. **Exclude glob respected** — write to `node_modules/foo.js` → zero rows; debouncer + watcher count it but classifier drops it.
8. **Include glob respected** — folder configured `include: ["**/*.md"]`; write `notes.md` → 1 row; write `script.sh` → 0 rows.
9. **Per-folder rate-limit triggers drop** — write 1000 files in 1 second → first 200 emitted (burst), remaining drop within seconds with `brain.capture_dropped` rows carrying `reason: rate_limited:folder`.
10. **Per-tenant rate-limit aggregates** — 10 folders each at 100 events/s → after 2 sec, `brain.capture_dropped` rows with `reason: rate_limited:tenant` (1000/sec exceeds 500/sec sustained).
11. **Queue overflow drops with audit row** — manually pause the emitter; producer fills 10K queue; next event drops within 100ms with `brain.capture_dropped` carrying `reason: queue_overflow`.
12. **W3C trace propagation from env** — set `TRACEPARENT=00-...-...-01` in daemon env; first capture row's `payload.trace_id` matches.
13. **W3C trace generated when env absent** — no `TRACEPARENT` → daemon generates a new 32-char hex trace_id at startup; all capture rows share it for that batch.
14. **Startup resync emits catch-up rows** — daemon down; user writes 5 new files; daemon starts → 5 catch-up rows + `brain.capture_resync_started` + `brain.capture_resync_completed`.
15. **Resync latency budget** — fixture with 100K files; resync completes in ≤ 60s.
16. **SIGHUP reload picks up new folder** — daemon running; operator adds folder to manifest; sends `kill -HUP <pid>` → new folder starts being watched within 1 second; capture rows for new folder use a fresh trace_id.
17. **SIGHUP reload removes folder** — daemon running; operator removes folder from manifest; sends SIGHUP → folder watcher dropped; cache entries for that folder pruned; no new rows from that folder.
18. **SIGTERM graceful stop** — daemon receives SIGTERM; drains queue (≤ 5s); exits 0.
19. **Doctor gate refuses to start** — manifest has dangling symlink (FR-BRAIN-105 `WatchedFolderResolvable` fails) → daemon exits 7 (`InternalError`) with stderr explaining the doctor failure.
20. **Metrics emit** — Prometheus registry contains all metrics from §1 #14; counters increment per emit.
21. **OTel span per emit** — exporter receives a `brain.capture.emit` span per emitted row with attributes `folder_id`, `kind`, `bytes`, `trace_id`.
22. **Dry-run prints without emitting** — `--dry-run` → events printed to stdout; BRAIN chain unchanged.

---

## §5 — Verification

```rust
// services/brain-capture/tests/end_to_end_test.rs

#[tokio::test]
async fn single_file_write_captures_one_row() {
    let env = TestEnv::new().await;
    env.write_file("foo.txt", b"hello").await;
    env.wait_for_capture("foo.txt", Duration::from_millis(500)).await;
    let rows = env.brain.rows_since(env.start_seq()).await;
    let cap = rows.iter().find(|r| r.kind == "brain.capture_created").unwrap();
    assert_eq!(cap.payload["relative_path"], "foo.txt");
    assert_eq!(cap.payload["content_hash"], hex::encode(blake3::hash(b"hello").as_bytes()));
    assert!(cap.payload["trace_id"].as_str().unwrap().len() == 32);
}

#[tokio::test]
async fn idempotent_resave() {
    let env = TestEnv::new().await;
    env.write_file("foo.txt", b"same").await;
    env.wait_for_capture("foo.txt", Duration::from_millis(500)).await;
    env.write_file("foo.txt", b"same").await;  // identical bytes
    tokio::time::sleep(Duration::from_millis(500)).await;
    let rows = env.brain.rows_since(env.start_seq()).await;
    let captures: Vec<_> = rows.iter().filter(|r| r.kind.starts_with("brain.capture_")).collect();
    assert_eq!(captures.len(), 1, "second save should be idempotent");
}

#[tokio::test]
async fn rename_detected_via_dedup() {
    let env = TestEnv::new().await;
    env.write_file("foo.txt", b"unique-content").await;
    env.wait_for_capture("foo.txt", Duration::from_millis(500)).await;
    env.rename("foo.txt", "bar.txt").await;
    env.wait_for_capture("bar.txt", Duration::from_millis(500)).await;
    let rows = env.brain.rows_since(env.start_seq()).await;
    let renamed = rows.iter().find(|r| r.kind == "brain.capture_renamed").unwrap();
    assert_eq!(renamed.payload["from_relative_path"], "foo.txt");
    assert_eq!(renamed.payload["relative_path"], "bar.txt");
}

#[tokio::test]
async fn per_folder_rate_limit_drops_with_audit() {
    let env = TestEnv::new().await;
    for i in 0..1000 {
        env.write_file(&format!("flood-{i}.txt"), format!("content-{i}").as_bytes()).await;
    }
    tokio::time::sleep(Duration::from_secs(2)).await;
    let rows = env.brain.rows_since(env.start_seq()).await;
    let emitted = rows.iter().filter(|r| r.kind == "brain.capture_created").count();
    let dropped = rows.iter().filter(|r| r.kind == "brain.capture_dropped").count();
    assert!(emitted <= 200 + (50 * 2),  "emitted {emitted} (burst+sustained); expected ≤ 300");
    assert!(dropped > 0,                "expected drops; got {dropped}");
    let first_dropped = rows.iter().find(|r| r.kind == "brain.capture_dropped").unwrap();
    assert_eq!(first_dropped.payload["reason"], "rate_limited:folder");
}

#[tokio::test]
async fn startup_resync_catches_up() {
    let env = TestEnv::new_without_daemon().await;
    env.write_file("offline-1.md", b"a").await;
    env.write_file("offline-2.md", b"b").await;
    env.write_file("offline-3.md", b"c").await;

    env.start_daemon().await;
    env.wait_for_event("brain.capture_resync_completed", Duration::from_secs(10)).await;

    let rows = env.brain.rows_since(env.start_seq()).await;
    assert!(rows.iter().any(|r| r.kind == "brain.capture_resync_started"));
    assert!(rows.iter().any(|r| r.kind == "brain.capture_resync_completed"));
    let captures: Vec<_> = rows.iter().filter(|r| r.kind == "brain.capture_created").collect();
    assert_eq!(captures.len(), 3);
}

#[tokio::test]
async fn sighup_picks_up_new_folder() {
    let env = TestEnv::new().await;
    let folder_b = env.add_folder_to_manifest("folder-b").await;
    env.send_sighup().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    env.write_file_in(&folder_b, "hello-b.txt", b"x").await;
    env.wait_for_capture_in(&folder_b, "hello-b.txt", Duration::from_millis(500)).await;
    // assertion: capture row exists with folder_id matching folder-b
}

#[tokio::test]
async fn doctor_gate_refuses_to_start() {
    let env = TestEnv::new_with_dangling_symlink().await;
    let res = env.spawn_daemon_and_await_exit().await;
    assert_eq!(res.exit_code, 7);  // InternalError
    assert!(res.stderr.contains("WatchedFolderResolvable"));
}
```

```rust
// services/brain-capture/tests/dedup_test.rs
#[test]
fn cache_hit_same_path_is_idempotent() {
    let cache = DedupCache::new();
    let h = [42u8; 32];
    cache.record(h, PathBuf::from("foo.txt"), 1);
    assert_eq!(cache.classify(h, Path::new("foo.txt"), Some(h)), DedupVerdict::Idempotent);
}

#[test]
fn cache_hit_different_path_is_rename() {
    let cache = DedupCache::new();
    let h = [99u8; 32];
    cache.record(h, PathBuf::from("foo.txt"), 1);
    assert!(matches!(cache.classify(h, Path::new("bar.txt"), Some(h)), DedupVerdict::EmitRenamed { .. }));
}

#[test]
fn cache_miss_with_prior_hash_is_modified() {
    let cache = DedupCache::new();
    let h1 = [1u8; 32]; let h2 = [2u8; 32];
    assert!(matches!(cache.classify(h2, Path::new("foo.txt"), Some(h1)),
                     DedupVerdict::EmitModified { prior_hash } if prior_hash == h1));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; orchestrator stitches the pieces together.)

```rust
// services/brain-capture/src/lib.rs
pub mod watcher;
pub mod dedup;
pub mod rate_limit;
pub mod queue;
pub mod emit;
pub mod trace;

use cyberos_brain_writer::BrainWriter;
use cyberos_brain_reader::BrainReader;

pub struct CaptureDaemon {
    writer:        BrainWriter,
    reader:        BrainReader,
    watchers:      Vec<watcher::FolderWatcher>,
    dedup:         std::sync::Arc<dedup::DedupCache>,
    rate:          std::sync::Arc<rate_limit::RateLimits>,
    event_rx:      tokio::sync::mpsc::Receiver<watcher::RawEvent>,
    manifest_path: std::path::PathBuf,
    dry_run:       bool,
}

impl CaptureDaemon {
    pub async fn start(manifest_path: Option<std::path::PathBuf>, dry_run: bool) -> anyhow::Result<Self> {
        let manifest = read_manifest(&manifest_path)?;
        let writer = BrainWriter::connect().await?;
        let reader = BrainReader::connect().await?;

        // STEP 1: resync each watched folder before turning on event-driven path
        let mut event_tx = None;
        for wf in &manifest.watched_folders {
            resync_folder(&reader, &writer, wf).await?;
        }

        // STEP 2: spawn one watcher per folder, all feeding a single bounded queue
        let (tx, rx) = tokio::sync::mpsc::channel(10_000);
        let mut watchers = Vec::new();
        for wf in manifest.watched_folders {
            watchers.push(watcher::FolderWatcher::spawn(wf, tx.clone()).await?);
        }
        // STEP 3: spawn the emitter that drains the queue
        let dedup = std::sync::Arc::new(dedup::DedupCache::new());
        let rate  = std::sync::Arc::new(rate_limit::RateLimits::new());
        tokio::spawn(drain_loop(rx, writer.clone(), dedup.clone(), rate.clone(), dry_run));

        Ok(Self { writer, reader, watchers, dedup, rate, event_rx: tokio::sync::mpsc::channel(1).1, manifest_path: manifest_path.unwrap_or_default(), dry_run })
    }

    pub async fn reload(&self) -> anyhow::Result<()> { /* re-read manifest; diff vs current; add/remove watchers */ Ok(()) }
    pub async fn stop(&self) { /* drop watchers; drain queue with 5s deadline */ }
}

async fn drain_loop(
    mut rx: tokio::sync::mpsc::Receiver<watcher::RawEvent>,
    writer: BrainWriter,
    dedup: std::sync::Arc<dedup::DedupCache>,
    rate:  std::sync::Arc<rate_limit::RateLimits>,
    dry_run: bool,
) {
    while let Some(raw) = rx.recv().await {
        match rate.check(&raw.folder_id, raw.tenant_id) {
            rate_limit::LimitVerdict::RateLimited { which } => {
                emit::dropped(&raw, &format!("rate_limited:{which}")).await;
                continue;
            }
            rate_limit::LimitVerdict::Allowed => {}
        }
        let body = match tokio::fs::read(&raw.path).await { Ok(b) => b, Err(_) => continue };
        let content_hash = *blake3::hash(&body).as_bytes();
        let dedup_verdict = dedup.classify(content_hash, &raw.path, None);
        if matches!(dedup_verdict, dedup::DedupVerdict::Idempotent) { continue; }

        if dry_run {
            println!("WOULD EMIT: {:?} {:?} {}", raw.kind, raw.path, hex::encode(content_hash));
            continue;
        }
        if let Err(e) = emit::emit_capture(&writer, &raw, content_hash, dedup_verdict, trace::current_trace_id()).await {
            tracing::error!(?e, "emit failed; will retry next event");
        } else {
            dedup.record(content_hash, raw.path.clone(), writer.current_seq().await);
        }
    }
}
```

---

## §7 — Dependencies

- **FR-BRAIN-101** — `BrainWriter` + `BrainReader` are the interface for chain mutations + reads.
- **FR-BRAIN-102** — `cyberos brain watch/unwatch` is the registration UX; this daemon picks up via SIGHUP.
- **FR-BRAIN-105** — `cyberos doctor --only watched-folders` is the boot gate.
- **FR-BRAIN-108** (downstream) — Cowork session-hook capture FR uses this daemon as its emit pathway.
- **FR-BRAIN-109** (downstream) — Claude Code hook capture FR likewise.
- **FR-BRAIN-110** (downstream) — health-check daemon supervises this process.
- **FR-BRAIN-111** (downstream) — pre-ingest PII detection runs on `body` BEFORE `blake3::hash` is invoked.
- **FR-AI-022** — W3C TraceContext extract/inject pattern reused.
- **FR-OBS-003, FR-OBS-005** — metric + trace conventions.
- **`cyberos-cli-exit`** — shared exit code enum.

---

## §8 — Example payloads

### `brain.capture_created`

```json
{
  "kind": "brain.capture_created",
  "tenant_id": "7e57c0de-1234-5678-9abc-def012345678",
  "payload": {
    "folder_id":     "01HZK9R8M3X5C8Q4-notes",
    "relative_path": "design/auth-flow.md",
    "content_hash":  "9b0e8c5...",
    "byte_count":    2418,
    "mtime_ns":      1747407137483000000,
    "trace_id":      "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### `brain.capture_dropped`

```json
{
  "kind": "brain.capture_dropped",
  "payload": {
    "folder_id":      "01HZK9R8M3X5C8Q4-notes",
    "event_kind":     "Modified",
    "content_hash":   "ab12...",
    "dropped_at_ns":  1747407138105000000,
    "reason":         "queue_overflow"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Coalesced burst-emit (one row per N identical events within a window) — slice 3+; clashes with per-event audit principle, needs research.
- Per-file-type debounce tuning — slice 3+; current 250ms is uniform.
- Encrypted-at-rest body hashing — slice 3+; need FR-BRAIN-111 PII detection first to know what to encrypt.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| File unreadable (perms) | `tokio::fs::read` returns Err | Event silently skipped; metric `brain_capture_events_total{outcome="read_failed"}` increments | Operator fixes perms |
| File deleted between debounce and read | `tokio::fs::read` returns ENOENT | Skipped; treated as if Delete event will follow | None — Delete event handles it |
| `blake3` failure | None observed; impl is infallible | N/A | N/A |
| Queue full + 100ms timeout | `event_tx.send_timeout` returns Err | `brain.capture_dropped` row emitted | Operator scales emitter or reduces burst |
| `brain_writer` unavailable | emit returns Err | Logged; row not emitted; dedup cache NOT updated | Retry on next event; operator investigates writer |
| Rate-limit (folder) | `governor::check` returns Err | `brain.capture_dropped` with `reason: rate_limited:folder` | Operator raises folder quota in code (governed) |
| Rate-limit (tenant) | `governor::check` returns Err | `brain.capture_dropped` with `reason: rate_limited:tenant` | Operator raises tenant quota |
| Glob pattern compile failure | `globset::Glob::new` returns Err at startup | Daemon exits 1 with stderr explanation | Operator fixes manifest |
| Symlink loop | `canonicalize` returns ELOOP | Watcher for that folder errors at spawn; daemon continues with others; sev-1 alarm | Operator removes loop |
| Manifest reload fails (bad JSON) | `read_manifest` Err on SIGHUP | Old manifest kept; sev-1 alarm; logged | Operator fixes manifest; resends SIGHUP |
| Resync exceeds 60s budget | timer | sev-2 alarm; daemon continues; resync completes when done | Operator reviews folder size |
| 100K+ files in one folder | scan duration exceeds budget | sev-2 alarm; operator advised to split folder | Operator splits OR raises budget |
| Daemon OOMs | OOM-killer | systemd restarts (FR-BRAIN-110); resync replays | Operator investigates leak |
| FSEvents drops events (macOS) | mtime mismatch on next event | Caught by next resync OR next event for that file | Periodic resync (FR-BRAIN-110 schedules hourly) |
| Filesystem switches readonly mid-run | write of HEAD file fails | sev-1 alarm; daemon exits cleanly | Operator restores RW; restarts daemon |
| Disk fills | writer returns ENOSPC | `brain.capture_dropped` rows; sev-1 alarm | Operator frees space; daemon recovers |
| Editor renames during write (atomic) | Debounce coalesces; single Created/Modified emitted | By design | N/A |
| Hidden files / dotfiles | Default exclude `.git/**`; others pass through | Captured unless excluded | Operator adjusts exclude list |

---

## §11 — Implementation notes

- `notify-debouncer-full` handles the multi-step atomic-save coalescing automatically; we don't roll our own debounce.
- `blake3::hash` is internally parallelised on modern CPUs; benchmarks show ~3 GB/s on M1 — well below our event rate.
- The dedup cache is a single Mutex<LruCache>; under heavy contention this could become a bottleneck. v2 may switch to a sharded cache (one per 16 hash buckets).
- `governor@0.6` is in-process; no Redis dependency. The token buckets are recreated on daemon restart — fine since rate-limits are per-process.
- The 10K queue capacity is tuned for one-watcher-per-folder × 50 folders × 200 burst events ≈ 10K theoretical max simultaneous events. Operators with > 50 folders may need to raise.
- `tokio::fs::read` is async file IO — does not block the executor; for very large files (> 10MB) the read happens cooperatively.
- W3C trace_id generation uses `opentelemetry::trace::TraceId::from_bytes(rand_bytes())` — 16 random bytes per process, rendered as 32 lowercase hex chars per W3C spec.
- SIGHUP reload re-reads `manifest.watched_folders` and computes a diff: new entries get a new watcher; removed entries have their watcher dropped (which closes the notify handle); changed entries are dropped + re-spawned.
- The `dry_run` mode prints events to stdout in the format `WOULD EMIT: <kind> <path> <content_hash>` — operator-friendly and grep-able.
- Tests use `tokio::time::pause()` + `tokio::time::advance()` for deterministic time control, except where real-time behaviour is being tested (rate limit, resync).

---

*End of FR-BRAIN-107.*
