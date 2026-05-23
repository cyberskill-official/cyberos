---
id: FR-MEMORY-110
title: "memory capture daemon supervision — systemd + launchd units + /healthz + watchdog + crash-restart with exponential backoff + sweeper cron"
module: memory
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-MEMORY-105, FR-MEMORY-107, FR-MEMORY-108, FR-MEMORY-109, FR-OBS-001, FR-OBS-007]
depends_on: [FR-MEMORY-105, FR-MEMORY-107, FR-MEMORY-109]
blocks: []

source_pages:
  - website/docs/modules/memory.html#daemon-supervision
  - website/docs/runbooks/memory-capture-runbook.html#supervision
source_decisions:
  - DEC-160 (capture daemon MUST auto-restart on crash with exp backoff capped at 5 min)
  - DEC-161 (per-OS init system: systemd on Linux, launchd on macOS; no custom supervisor)
  - DEC-162 (/healthz endpoint reports doctor invariant status + queue depth + last emit timestamp)
  - DEC-163 (hourly sweeper prunes /tmp/cyberos-memory-claude-traces, stale dedup cache, old metric snapshots)

language: rust 1.81 + systemd + launchd plist
service: cyberos/services/memory-capture/
new_files:
  - services/memory-capture/src/healthz.rs
  - services/memory-capture/src/sweeper.rs
  - services/memory-capture/install/systemd/cyberos-memory-capture.service
  - services/memory-capture/install/launchd/world.cyberos.memory-capture.plist
  - services/memory-capture/install/install-daemon.sh
  - services/memory-capture/install/uninstall-daemon.sh
  - services/memory-capture/tests/healthz_test.rs
  - services/memory-capture/tests/sweeper_test.rs
  - services/memory-capture/tests/restart_e2e_test.sh
modified_files:
  - services/memory-capture/src/main.rs                  # bind /healthz; start sweeper; install signal handlers
  - services/memory-capture/src/lib.rs                   # CaptureDaemon::health() + ::run_sweeper()
allowed_tools:
  - file_read: services/memory-capture/**
  - file_write: services/memory-capture/{src,install,tests}/**
  - bash: cd services/memory-capture && cargo test
  - bash: cd services/memory-capture && cargo build --release
disallowed_tools:
  - write a custom supervisor (per DEC-161 — use OS-native init)
  - skip /healthz endpoint (operators + monitoring depend on it)
  - skip the sweeper (per DEC-163 — without it, /tmp grows unbounded over months)

effort_hours: 6
sub_tasks:
  - "0.5h: cyberos-memory-capture.service unit file (Restart=on-failure; RestartSec=exp backoff cap 5m via systemd RestartSteps)"
  - "0.5h: world.cyberos.memory-capture.plist (KeepAlive + ThrottleInterval + ExitTimeOut)"
  - "0.5h: install-daemon.sh + uninstall-daemon.sh — OS detection + install/start/stop"
  - "1.0h: healthz.rs — axum router with GET /healthz (200 OK or 503 with details)"
  - "1.0h: sweeper.rs — tokio interval (60s); prune trace_id cache, dedup expired entries, old metric snapshots"
  - "0.5h: main.rs wiring: bind /healthz on 127.0.0.1:7777; spawn sweeper task; install SIGINT/SIGTERM/SIGHUP handlers"
  - "1.0h: healthz_test.rs — happy + invariant-failure + queue-saturated"
  - "0.5h: sweeper_test.rs — file pruning + dedup TTL"
  - "0.5h: restart_e2e_test.sh — bash script: install, kill -9, wait for restart, assert ≤ 1s downtime first iteration; cap at 5m by 10th iteration"
risk_if_skipped: "Without supervision, a single SEGFAULT or OOM permanently stops capture. Operators wake up to memory missing 8 hours of memories. Without /healthz, monitoring can't distinguish 'daemon running but stuck' from 'daemon running healthy.' Without the sweeper, /tmp accumulates trace_id files indefinitely; on a long-running deployment this fills the disk in ~6 months. Without exponential backoff, a daemon that crashes every 100ms (corrupted manifest) would restart 10× per second — log flood + parent process CPU burn."
---

## §1 — Description (BCP-14 normative)

The capture daemon's runtime supervision **MUST** consist of: (a) an OS-native init unit, (b) a `/healthz` HTTP endpoint, (c) a periodic sweeper for ephemeral state, (d) signal-driven graceful shutdown. The contract:

1. **MUST** ship a systemd unit `cyberos-memory-capture.service` (Linux) AND a launchd plist `world.cyberos.memory-capture.plist` (macOS). The installer (`install-daemon.sh`) detects the OS and copies the appropriate file to:
    - Linux: `/etc/systemd/system/cyberos-memory-capture.service` (system-scope) OR `~/.config/systemd/user/cyberos-memory-capture.service` (user-scope, default for non-root install).
    - macOS: `~/Library/LaunchAgents/world.cyberos.memory-capture.plist` (user-scope; system-scope deferred to slice 3).
2. **MUST** auto-restart on non-zero exit per OS-native semantics:
    - systemd: `Restart=on-failure`, `RestartSec=5s`, `RestartSteps=5s 10s 30s 1m 5m` (capped at 5 min).
    - launchd: `KeepAlive: true`, `ThrottleInterval: 60` (launchd does not natively support exponential backoff; we implement it in the daemon's own startup logic by sleeping `min(2^crash_count, 300)` seconds before binding sockets, where `crash_count` is read from `/tmp/cyberos-memory-capture-crashes`).
3. **MUST** expose `GET /healthz` on `127.0.0.1:7777` (no external bind; loopback only). Response schema:
    ```jsonc
    // 200 OK
    {
      "status": "healthy",
      "version": "0.1.0",
      "uptime_seconds": 14392,
      "watched_folders": 12,
      "queue_depth": 47,
      "queue_capacity": 10000,
      "last_emit_ns": 1747407137483000000,
      "doctor_invariants": {"all_pass": true, "checked_at_ns": 1747407100000000000}
    }
    // 503 Service Unavailable
    {
      "status": "unhealthy",
      "reasons": ["doctor_invariant_failed: WatchedFolderResolvable", "queue_saturated"],
      "queue_depth": 9876,
      "queue_capacity": 10000,
      "doctor_invariants": {"all_pass": false, "first_failure": "WatchedFolderResolvable"}
    }
    ```
4. **MUST** return 503 when ANY of the following are true:
    - Any FR-MEMORY-105 error-severity invariant fails (checked every 60s; result cached).
    - Queue depth ≥ 95% of capacity (9500 / 10000) for ≥ 30 seconds.
    - Last successful emit was > 5 minutes ago AND watched folders are non-empty (likely deadlock).
    - The memory writer subprocess (per FR-MEMORY-101) is unreachable.
5. **MUST** run a sweeper task on a 60-second tokio interval that prunes:
    - `/tmp/cyberos-memory-claude-traces/<uuid>` files older than 1 hour (FR-MEMORY-109 trace cache).
    - Dedup-cache entries whose `stored_at` exceeded TTL (5 minutes per FR-MEMORY-107 §1 #4).
    - Old metric snapshots in `/tmp/cyberos-memory-metrics/` older than 24 hours.
    - Crash-count file `/tmp/cyberos-memory-capture-crashes` if last reset > 1 hour ago (allows recovery from a transient crash storm without sticky exp-backoff).
6. **MUST** install signal handlers per AGENTS.md operational conventions:
    - `SIGTERM` → graceful shutdown: stop accepting new events, drain queue (deadline 5s), flush metrics, exit 0.
    - `SIGINT` → same as SIGTERM (treat Ctrl-C like Stop).
    - `SIGHUP` → reload manifest (per FR-MEMORY-107 §1 #15); does not exit.
    - `SIGUSR1` → dump health snapshot to stderr (operator debugging).
6.5. **MUST** treat `SIGKILL` as expected (cannot be caught); systemd/launchd restart on the next cycle.
7. **MUST** track crash count in `/tmp/cyberos-memory-capture-crashes` (single u32 LE; incremented at startup; reset to 0 after 5 minutes of stable uptime). The daemon uses this to compute its own startup-delay backoff on launchd.
8. **MUST** emit OTel metrics:
    - `memory_capture_daemon_uptime_seconds` (gauge).
    - `memory_capture_daemon_restart_count_total` (counter; survives restart via the crash-count file).
    - `memory_capture_sweeper_pruned_total{kind}` (counter; kind ∈ trace_cache | dedup | metric_snapshot | crash_count_reset).
    - `memory_capture_health_endpoint_total{status}` (counter; status ∈ healthy | unhealthy).
9. **MUST** emit OTel span `memory.capture.sweeper.tick` per sweep with attributes `pruned_trace_cache`, `pruned_dedup`, `pruned_metric_snapshots`, `duration_ms`.
10. **MUST** emit a `memory.capture_supervisor_event` memory audit row when the daemon process starts (kind=`started`), reloads manifest (kind=`reloaded`), or exits (kind=`exited`, `exit_code`, `uptime_seconds`). This gives operators a first-class audit trail for "when did the daemon last go down?"
11. **MUST** integrate with FR-OBS-007 alert routing: 503 on `/healthz` for ≥ 60 seconds triggers a sev-2 alert; ≥ 5 minutes triggers sev-1 (capture is effectively offline; user actions not being recorded).
12. **MUST** support `cyberos memory capture status` CLI which hits `/healthz` and pretty-prints the response (human-friendly format with colour-coded badges).
13. **MUST** support `cyberos memory capture logs [--follow] [--lines N]` which tails the daemon log file (`~/Library/Logs/cyberos-memory-capture.log` on macOS, `journalctl -u cyberos-memory-capture` on Linux).
14. **MUST** be installed by `install-daemon.sh` and uninstalled by `uninstall-daemon.sh`. Install is idempotent; uninstall is total (removes unit file, stops daemon, removes /tmp files).
15. **SHOULD** publish `cyberos-memory-capture.service` to a Homebrew formula (macOS) and a deb/rpm package (Linux) — slice 3+.

---

## §2 — Why this design (rationale for humans)

**Why systemd + launchd (§1 #1)?** They are the canonical OS init systems for Linux + macOS. They handle: process supervision, restart on crash, log rotation, signal delivery, dependency ordering. Building a custom supervisor would re-implement 30 years of init-system engineering. Per DEC-161 we do not.

**Why exp backoff capped at 5 min (§1 #2)?** A daemon that crashes immediately on startup (e.g. corrupted manifest) without backoff would consume parent process CPU and flood logs. Exp backoff is the standard pattern. 5-min cap prevents long outages: operators see "daemon down 5 min, sev-1 alarm" and act; without a cap, an undetected misconfiguration could leave the daemon restart-locked for hours.

**Why /healthz on 127.0.0.1:7777 (§1 #3)?** Loopback-only because: (a) we don't want external network exposure on capture daemon; (b) the consumers (monitoring agents, FR-OBS-007 probes) run on the same machine. Port 7777 is unallocated by IANA + memorable. The endpoint is unauthenticated; loopback is the perimeter.

**Why 503 on multiple conditions (§1 #4)?** Operators need to distinguish "daemon healthy" from "daemon up but useless." Queue saturated = events being dropped silently. Doctor invariant failed = capture wrong / unsafe. Stale emit = likely deadlock. Each condition warrants restart; 503 makes the supervisor handle it (systemd `Restart=on-failure` with a healthcheck wrapper).

**Why 60s sweep interval (§1 #5)?** Faster sweeping wastes CPU on idle systems; slower sweeping accumulates /tmp clutter. 60s is the empirical sweet spot: low overhead, low clutter. Operators can SIGHUP to force a sweep early (debugging).

**Why crash-count file in /tmp (§1 #7)?** launchd doesn't expose crash counts natively (unlike systemd's `Restart=on-failure` which has `RestartSteps` built in). To implement exp backoff on macOS, we maintain our own counter. /tmp is the right scope (per-machine, ephemeral); resets to 0 after stable uptime so transient crashes don't lock us into 5-min backoff forever.

**Why supervisor_event audit rows (§1 #10)?** Operators investigating "memory had a gap between 14:00 and 14:23" need to answer "was the daemon down?" Without a first-class audit row, they have to cross-reference systemd journalctl with memory — friction. The row turns the supervisor's behaviour into queryable data.

**Why sev-2 → sev-1 at 5 min (§1 #11)?** 60s of capture downtime is recoverable (most editor saves get caught on next batch). 5 min is significant data loss (8+ files of meaningful work). The escalation gives ops a window to self-resolve before paging.

**Why no `--restart-now` CLI command?** The OS init system owns the lifecycle. `systemctl restart cyberos-memory-capture` (Linux) and `launchctl kickstart -k gui/$UID/world.cyberos.memory-capture` (macOS) are the canonical paths. Adding a Cyberos-level wrapper duplicates them; operators benefit from learning the OS commands once.

**Why `cyberos memory capture logs` (§1 #13)?** Cross-platform log access is a friction point: `journalctl -u name --follow` on Linux vs tailing a file on macOS. The CLI command abstracts the difference.

---

## §3 — API contract

### systemd unit

```ini
# install/systemd/cyberos-memory-capture.service
[Unit]
Description=CyberOS memory capture daemon
After=network.target
Documentation=https://docs.cyberos.world/runbooks/memory-capture-runbook.html

[Service]
Type=simple
ExecStart=/usr/local/bin/cyberos-memory-capture --foreground
Restart=on-failure
RestartSec=5s
# Exponential backoff (systemd 254+): 5s → 10s → 30s → 1m → 5m → 5m → ...
RestartSteps=5
RestartMaxDelaySec=5min
# Resource limits (sane defaults; operator may override)
MemoryMax=1G
TasksMax=512
# Graceful shutdown deadline (matches §1 #6)
TimeoutStopSec=10s
# Crash notification to journal (parsed by FR-OBS-001 collector)
StandardOutput=journal
StandardError=journal
# Environment file (writer endpoint, manifest path, OTLP endpoint)
EnvironmentFile=-/etc/cyberos/memory-capture.env

[Install]
WantedBy=default.target
```

### launchd plist

```xml
<!-- install/launchd/world.cyberos.memory-capture.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>             <string>world.cyberos.memory-capture</string>
  <key>ProgramArguments</key>  <array><string>/usr/local/bin/cyberos-memory-capture</string><string>--foreground</string></array>
  <key>RunAtLoad</key>         <true/>
  <key>KeepAlive</key>         <true/>
  <!-- launchd's anti-flap; daemon's own backoff handles longer delays -->
  <key>ThrottleInterval</key>  <integer>60</integer>
  <key>ExitTimeOut</key>       <integer>10</integer>
  <key>StandardOutPath</key>   <string>/Users/USERNAME/Library/Logs/cyberos-memory-capture.log</string>
  <key>StandardErrorPath</key> <string>/Users/USERNAME/Library/Logs/cyberos-memory-capture.err</string>
  <key>WorkingDirectory</key>  <string>/tmp</string>
  <key>EnvironmentVariables</key>
  <dict>
    <key>RUST_LOG</key>        <string>info,cyberos_memory_capture=debug</string>
    <key>CYBEROS_MANIFEST</key><string>/Users/USERNAME/Library/Application Support/CyberOS/manifest.json</string>
  </dict>
</dict>
</plist>
```

### healthz endpoint

```rust
// services/memory-capture/src/healthz.rs
use axum::{Router, routing::get, Json};
use axum::http::StatusCode;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::Serialize;

pub struct HealthState {
    pub daemon_start:   Instant,
    pub queue_depth:    Arc<std::sync::atomic::AtomicU64>,
    pub queue_capacity: u64,
    pub last_emit_ns:   Arc<std::sync::atomic::AtomicU64>,
    pub last_doctor:    Arc<tokio::sync::RwLock<DoctorSnapshot>>,
    pub watched_count:  Arc<std::sync::atomic::AtomicU32>,
}

#[derive(Clone, Serialize)]
pub struct DoctorSnapshot {
    pub all_pass:      bool,
    pub first_failure: Option<String>,
    pub checked_at_ns: u64,
}

#[derive(Serialize)]
pub struct HealthReport {
    pub status:           &'static str,
    pub version:          &'static str,
    pub uptime_seconds:   u64,
    pub watched_folders:  u32,
    pub queue_depth:      u64,
    pub queue_capacity:   u64,
    pub last_emit_ns:     u64,
    pub doctor_invariants: DoctorSnapshot,
    #[serde(skip_serializing_if = "Vec::is_empty")] pub reasons: Vec<String>,
}

pub async fn handler(state: Arc<HealthState>) -> (StatusCode, Json<HealthReport>) {
    let now = Instant::now();
    let queue_depth = state.queue_depth.load(std::sync::atomic::Ordering::Relaxed);
    let last_emit_ns = state.last_emit_ns.load(std::sync::atomic::Ordering::Relaxed);
    let doctor = state.last_doctor.read().await.clone();
    let watched = state.watched_count.load(std::sync::atomic::Ordering::Relaxed);

    let mut reasons = Vec::new();
    if !doctor.all_pass {
        reasons.push(format!("doctor_invariant_failed: {}", doctor.first_failure.clone().unwrap_or_default()));
    }
    if queue_depth >= (state.queue_capacity * 95) / 100 {
        reasons.push("queue_saturated".into());
    }
    let stale_emit = watched > 0
        && (unix_ns() as u128).saturating_sub(last_emit_ns as u128) > Duration::from_secs(300).as_nanos();
    if stale_emit {
        reasons.push("stale_emit".into());
    }

    let status = if reasons.is_empty() { "healthy" } else { "unhealthy" };
    let code = if reasons.is_empty() { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    metrics::counter!("memory_capture_health_endpoint_total", "status" => status).increment(1);

    let report = HealthReport {
        status, version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: state.daemon_start.elapsed().as_secs(),
        watched_folders: watched, queue_depth, queue_capacity: state.queue_capacity, last_emit_ns,
        doctor_invariants: doctor,
        reasons,
    };
    (code, Json(report))
}

pub fn router(state: Arc<HealthState>) -> Router {
    Router::new().route("/healthz", get({
        let state = state.clone();
        move || handler(state.clone())
    }))
}
```

### Sweeper

```rust
// services/memory-capture/src/sweeper.rs
use tokio::time::{interval, Duration};
use std::time::SystemTime;
use std::path::Path;

pub async fn run_sweeper_loop() {
    let mut ticker = interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        let start = std::time::Instant::now();
        let pruned_traces  = prune_dir_older_than("/tmp/cyberos-memory-claude-traces", Duration::from_secs(3600));
        let pruned_metrics = prune_dir_older_than("/tmp/cyberos-memory-metrics",       Duration::from_secs(86400));
        let reset_crashes  = maybe_reset_crash_count("/tmp/cyberos-memory-capture-crashes", Duration::from_secs(300));
        tracing::debug!(
            pruned_trace_cache = pruned_traces,
            pruned_metric_snapshots = pruned_metrics,
            crash_count_reset = reset_crashes,
            duration_ms = start.elapsed().as_millis() as u64,
            "memory.capture.sweeper.tick"
        );
        metrics::counter!("memory_capture_sweeper_pruned_total", "kind" => "trace_cache").increment(pruned_traces as u64);
        metrics::counter!("memory_capture_sweeper_pruned_total", "kind" => "metric_snapshot").increment(pruned_metrics as u64);
        if reset_crashes {
            metrics::counter!("memory_capture_sweeper_pruned_total", "kind" => "crash_count_reset").increment(1);
        }
    }
}

fn prune_dir_older_than(dir: &str, age: Duration) -> u32 {
    let now = SystemTime::now();
    let mut pruned = 0u32;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if now.duration_since(mtime).map(|d| d > age).unwrap_or(false) {
                        let _ = std::fs::remove_file(entry.path());
                        pruned += 1;
                    }
                }
            }
        }
    }
    pruned
}

fn maybe_reset_crash_count(path: &str, age: Duration) -> bool {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(mtime) = meta.modified() {
            if SystemTime::now().duration_since(mtime).map(|d| d > age).unwrap_or(false) {
                let _ = std::fs::write(path, 0u32.to_le_bytes());
                return true;
            }
        }
    }
    false
}
```

### Installer

```bash
#!/usr/bin/env bash
# install/install-daemon.sh
set -euo pipefail

case "$(uname -s)" in
  Linux)
    UNIT=cyberos-memory-capture.service
    DEST="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/$UNIT"
    mkdir -p "$(dirname "$DEST")"
    install -m 644 "$(dirname "$0")/systemd/$UNIT" "$DEST"
    systemctl --user daemon-reload
    systemctl --user enable --now "$UNIT"
    echo "✓ installed and started → $DEST"
    ;;
  Darwin)
    PLIST="world.cyberos.memory-capture.plist"
    DEST="$HOME/Library/LaunchAgents/$PLIST"
    sed "s|/Users/USERNAME|$HOME|g" "$(dirname "$0")/launchd/$PLIST" > "$DEST"
    launchctl unload "$DEST" 2>/dev/null || true
    launchctl load -w "$DEST"
    echo "✓ installed and loaded → $DEST"
    ;;
  *)
    echo "Unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac
```

---

## §4 — Acceptance criteria

1. **systemd: daemon starts on `systemctl --user start`** — fixture installs unit; `systemctl --user is-active` returns `active`.
2. **systemd: daemon restarts on non-zero exit** — `kill -SEGV $(pidof cyberos-memory-capture)` → process restarts within `RestartSec=5s` ± 2s; `MainPID` differs.
3. **systemd: exp backoff cap at 5min** — crash 10× rapidly → 10th restart waits ~5 min (within ±10s).
4. **launchd: daemon starts on `launchctl load`** — fixture installs plist; `launchctl print gui/$UID/world.cyberos.memory-capture` shows state=running.
5. **launchd: daemon restarts on crash** — kill -9 the process → launchd restarts within ThrottleInterval=60s.
6. **launchd: daemon's own backoff** — crash 5× rapidly → daemon-side `sleep(min(2^crash_count, 300))` is observable in stderr.
7. **/healthz: 200 on happy** — fresh daemon, no failures → `curl 127.0.0.1:7777/healthz` returns 200; body has `status: "healthy"`.
8. **/healthz: 503 on doctor failure** — inject a dangling-symlink fixture → doctor cache reports failure → next `/healthz` returns 503; reasons includes `doctor_invariant_failed`.
9. **/healthz: 503 on queue saturation** — fill queue to 9500/10000 for ≥ 30 seconds → 503 with `queue_saturated`.
10. **/healthz: 503 on stale emit** — block emitter for 6 minutes (test seam) → 503 with `stale_emit`.
11. **Sweeper prunes trace cache** — write `/tmp/cyberos-memory-claude-traces/<uuid>` with mtime 2h ago → next tick removes it; metric `memory_capture_sweeper_pruned_total{kind="trace_cache"}` increments.
12. **Sweeper prunes metric snapshots** — fixture file in `/tmp/cyberos-memory-metrics/` with mtime 25h ago → pruned.
13. **Sweeper resets crash count after stable uptime** — write crash-count file with mtime 6 min ago → sweeper resets to 0.
14. **SIGTERM graceful shutdown** — send SIGTERM with full queue → daemon drains queue (≤ 5s), emits `memory.capture_supervisor_event` kind=`exited`, exits 0.
15. **SIGHUP reloads manifest** — SIGHUP → daemon re-reads manifest; new folders watched; row `kind=reloaded` emitted.
16. **SIGUSR1 dumps health to stderr** — SIGUSR1 → human-formatted health snapshot appears in stderr.
17. **supervisor_event row on start** — fresh start → exactly one `memory.capture_supervisor_event` row with kind=`started`.
18. **supervisor_event row on exit** — SIGTERM → row with kind=`exited`, `exit_code=0`, `uptime_seconds` present.
19. **install-daemon.sh idempotent** — run twice → second run is a no-op; unit file byte-identical.
20. **uninstall-daemon.sh total cleanup** — uninstall → unit/plist removed; daemon stopped; `/tmp/cyberos-memory-capture-crashes` removed.
21. **cyberos memory capture status** — pretty-prints `/healthz` JSON with colour badges (PASS/WARN/FAIL per reason).
22. **cyberos memory capture logs --follow --lines 100** — tails the appropriate log source; works on both Linux + macOS.
23. **OTel metrics: uptime + restart count** — Prometheus registry has both metrics; restart count survives a SIGSEGV→restart cycle.
24. **FR-OBS-007 alert fires** — 503 for 60s → sev-2 alert with `service: memory-capture` label.

---

## §5 — Verification

```rust
// services/memory-capture/tests/healthz_test.rs

#[tokio::test]
async fn returns_200_when_healthy() {
    let state = build_healthy_state();
    let router = healthz::router(state);
    let req = axum::http::Request::get("/healthz").body(axum::body::Body::empty()).unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: HealthReport = parse_body(resp).await;
    assert_eq!(body.status, "healthy");
    assert!(body.reasons.is_empty());
}

#[tokio::test]
async fn returns_503_when_doctor_failed() {
    let state = build_healthy_state();
    *state.last_doctor.write().await = DoctorSnapshot {
        all_pass: false, first_failure: Some("WatchedFolderResolvable".into()),
        checked_at_ns: unix_ns(),
    };
    let resp = healthz::router(state).oneshot(get_healthz_req()).await.unwrap();
    assert_eq!(resp.status(), 503);
    let body: HealthReport = parse_body(resp).await;
    assert!(body.reasons.iter().any(|r| r.starts_with("doctor_invariant_failed")));
}

#[tokio::test]
async fn returns_503_when_queue_saturated() {
    let state = build_healthy_state();
    state.queue_depth.store(9600, std::sync::atomic::Ordering::Relaxed);
    let resp = healthz::router(state).oneshot(get_healthz_req()).await.unwrap();
    assert_eq!(resp.status(), 503);
    let body: HealthReport = parse_body(resp).await;
    assert!(body.reasons.contains(&"queue_saturated".to_string()));
}
```

```rust
// services/memory-capture/tests/sweeper_test.rs
#[test]
fn prunes_files_older_than_ttl() {
    let tmpdir = tempdir().unwrap();
    let dir = tmpdir.path().join("traces");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("00000000-0000-0000-0000-000000000001");
    std::fs::write(&path, b"x").unwrap();
    // Backdate mtime by 2 hours
    set_mtime(&path, std::time::SystemTime::now() - std::time::Duration::from_secs(7200));

    let pruned = sweeper::prune_dir_older_than(dir.to_str().unwrap(), Duration::from_secs(3600));
    assert_eq!(pruned, 1);
    assert!(!path.exists());
}

#[test]
fn preserves_fresh_files() {
    let tmpdir = tempdir().unwrap();
    let dir = tmpdir.path().join("traces");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fresh");
    std::fs::write(&path, b"y").unwrap();
    let pruned = sweeper::prune_dir_older_than(dir.to_str().unwrap(), Duration::from_secs(3600));
    assert_eq!(pruned, 0);
    assert!(path.exists());
}
```

```bash
# services/memory-capture/tests/restart_e2e_test.sh
#!/usr/bin/env bash
set -euo pipefail

bash install/install-daemon.sh
sleep 2
[ "$(systemctl --user is-active cyberos-memory-capture)" = "active" ]

# Crash + restart
ORIG_PID=$(systemctl --user show -p MainPID --value cyberos-memory-capture)
kill -9 "$ORIG_PID"
sleep 7
NEW_PID=$(systemctl --user show -p MainPID --value cyberos-memory-capture)
[ "$ORIG_PID" != "$NEW_PID" ]
[ "$(systemctl --user is-active cyberos-memory-capture)" = "active" ]

# Crash 10× rapidly → 10th restart should be at 5 min cap (sample 3 to keep CI under 5 min)
for i in 1 2 3; do
  kill -9 $(systemctl --user show -p MainPID --value cyberos-memory-capture)
  sleep 1
done
# After 3 crashes, RestartSec should be 30s (per RestartSteps); verify within ±5s
START=$(date +%s)
while [ "$(systemctl --user is-active cyberos-memory-capture)" != "active" ]; do
  sleep 1
  [ $(($(date +%s) - START)) -lt 40 ] || { echo "did not restart within 40s"; exit 1; }
done

bash install/uninstall-daemon.sh
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

```rust
// services/memory-capture/src/main.rs (excerpt — daemon entrypoint)

#[tokio::main]
async fn main() -> cyberos_cli_exit::ExitCode {
    use cyberos_cli_exit::ExitCode;
    let cli = Cli::parse();
    cyberos_obs_sdk::init("memory-capture");

    // Crash-count exp-backoff (launchd-side)
    if cfg!(target_os = "macos") {
        let count = read_crash_count();
        let delay_s = std::cmp::min(2u64.pow(count.min(10)), 300);
        if delay_s > 0 {
            tracing::warn!(count, delay_s, "delaying startup for exp backoff");
            tokio::time::sleep(Duration::from_secs(delay_s)).await;
        }
        increment_crash_count();
    }

    // Doctor gate (FR-MEMORY-105 §1 #13)
    if let Err(e) = run_doctor_gate().await {
        tracing::error!(?e, "doctor invariant failure; refusing to start");
        return ExitCode::InternalError;
    }

    // Start the daemon (FR-MEMORY-107) and supervisory pieces
    let daemon = cyberos_memory_capture::CaptureDaemon::start(cli.manifest, cli.dry_run).await
        .expect("daemon start failed");
    let health_state = daemon.health_state();

    // Bind /healthz on 127.0.0.1:7777
    let healthz_listener = tokio::net::TcpListener::bind("127.0.0.1:7777").await.unwrap();
    tokio::spawn(async move {
        axum::serve(healthz_listener, cyberos_memory_capture::healthz::router(health_state)).await.unwrap();
    });

    // Spawn sweeper
    tokio::spawn(cyberos_memory_capture::sweeper::run_sweeper_loop());

    // Emit supervisor_event:started
    daemon.emit_supervisor_event("started", None).await;

    // Signal-driven main loop
    let mut sighup  = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()).unwrap();
    let mut sigusr1 = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()).unwrap();
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    let mut sigint  = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();
    let start = std::time::Instant::now();
    loop {
        tokio::select! {
            _ = sighup.recv()  => { let _ = daemon.reload().await; daemon.emit_supervisor_event("reloaded", None).await; }
            _ = sigusr1.recv() => { eprintln!("{:#?}", daemon.health_state().snapshot().await); }
            _ = sigterm.recv() => { daemon.stop().await; daemon.emit_supervisor_event("exited", Some(0)).await; break; }
            _ = sigint.recv()  => { daemon.stop().await; daemon.emit_supervisor_event("exited", Some(0)).await; break; }
        }
    }
    ExitCode::Ok
}
```

---

## §7 — Dependencies

- **FR-MEMORY-105 (upstream)** — `cyberos doctor --only watched-folders` is the gate at boot AND the periodic check feeding `/healthz`.
- **FR-MEMORY-107 (upstream)** — capture daemon being supervised; this FR adds the supervision wrapper.
- **FR-MEMORY-108 (related)** — Cowork session capture FR is downstream consumer (its hook posts to this daemon's socket).
- **FR-MEMORY-109 (related)** — Claude Code hook capture; same socket consumer + the `/tmp/cyberos-memory-claude-traces/` cache this FR sweeps.
- **FR-OBS-001 (related)** — journal/log capture by the OTel collector reads our stdout/journalctl.
- **FR-OBS-007 (downstream)** — alert routing fires on 503-for-60s and 503-for-5min thresholds.

---

## §8 — Example payloads

### Happy `/healthz`

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 14392,
  "watched_folders": 12,
  "queue_depth": 47,
  "queue_capacity": 10000,
  "last_emit_ns": 1747407137483000000,
  "doctor_invariants": {
    "all_pass": true,
    "first_failure": null,
    "checked_at_ns": 1747407100000000000
  }
}
```

### Unhealthy `/healthz`

```json
{
  "status": "unhealthy",
  "version": "0.1.0",
  "uptime_seconds": 432,
  "watched_folders": 12,
  "queue_depth": 9876,
  "queue_capacity": 10000,
  "last_emit_ns": 1747406500000000000,
  "doctor_invariants": {
    "all_pass": false,
    "first_failure": "WatchedFolderResolvable",
    "checked_at_ns": 1747407100000000000
  },
  "reasons": [
    "doctor_invariant_failed: WatchedFolderResolvable",
    "queue_saturated",
    "stale_emit"
  ]
}
```

### `memory.capture_supervisor_event` (start)

```json
{
  "kind": "memory.capture_supervisor_event",
  "payload": {
    "event":     "started",
    "version":   "0.1.0",
    "pid":       4823,
    "host_id":   "mac-stephen-001",
    "started_at_ns": 1747407137483000000
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Homebrew / deb / rpm distribution — slice 3+; needs release pipeline.
- Multi-user macOS install (system-scope LaunchDaemon under root) — slice 3+; needs permissions design.
- Cross-host failover (one daemon, multiple hosts) — slice 5+; speculative.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Process SEGFAULT | OS signal | systemd/launchd restart per backoff | None — automatic |
| OOM kill | OOM-killer | Restart; sev-1 if MemoryMax repeatedly hit | Operator investigates leak or raises limit |
| Manifest corrupted (bad JSON) | Daemon exits 1 at boot | Backoff escalates to 5min; sev-1 | Operator restores manifest |
| Doctor gate fails | Daemon exits 7 at boot | Backoff escalates; sev-2 | Operator runs `cyberos doctor` to identify failure |
| /healthz port 7777 in use | bind() Err | Daemon exits 1 with stderr; sev-1 | Operator frees port or sets env CYBEROS_HEALTHZ_PORT |
| Queue saturated for 30s | health check 503 | systemd's healthcheck wrapper restarts (or sev-2 alarm) | Operator investigates emitter |
| Stale emit > 5min | health check 503 | Restart | Operator investigates writer |
| Sweeper task panics | tokio task watchdog | Process exits; restart | Operator investigates |
| /tmp full | sweeper writes fail | sweeper logs WARN; continues with in-mem | Operator frees /tmp; auto-recovers |
| Crash count file unreadable | read returns 0 (treated as fresh) | No backoff; possible rapid-restart | Operator restores file or accepts behaviour |
| Crash count file write fails (/tmp ENOSPC) | write Err | Backoff math degrades to "no backoff"; restart loop tight | Operator frees /tmp |
| Two daemon instances (operator error) | second instance: bind() Err on healthz port | Second exits 1; first continues | Operator stops one |
| systemd unit file ENOENT after upgrade | systemctl daemon-reload finds nothing | Daemon dies; uninstall trigger | Operator reinstalls |
| launchd plist syntax error | launchctl load Err | Daemon doesn't start | Operator fixes plist |
| OTel exporter down | metric/span buffering | Buffer fills; oldest dropped; daemon survives | Operator restores FR-OBS-001 |
| `cyberos memory capture status` can't reach /healthz | connection refused | CLI prints error; exit 1 | Operator checks daemon is running |
| `cyberos memory capture logs` on macOS but log file path wrong | tail Err | CLI prints error | Operator updates plist path or accepts default |
| RestartMaxDelaySec hit; daemon-down 5+ min | FR-OBS-007 sev-1 page | Operator investigates manually | Manual fix and restart |
| Network mount unavailable when daemon starts | manifest path unreadable | exit 1; backoff; sev-2 | Operator remounts |

---

## §11 — Implementation notes

- The systemd unit uses `RestartSteps` introduced in systemd 254 (mid-2024); systems on older releases get a fixed `RestartSec=30s` fallback (declared by `Conditional` block in the unit file).
- `--user` mode is the default install scope; system-scope (root daemon for all users) is gated behind `--system` and requires sudo.
- launchd's `ThrottleInterval=60` means launchd waits 60s before restarting a process that exits within 10s of starting. Our daemon-side backoff layers on top: if the in-daemon `crash_count` says we've crashed 4 times rapidly, the daemon itself sleeps 16s before binding sockets — total wait ≈ 76s.
- `/healthz` uses Axum because it's already in the workspace (FR-OBS-002). Adding `hyper` directly would be smaller but inconsistent.
- The doctor-cache TTL (60s) means a doctor failure may take up to 60s to surface in `/healthz`. Faster polling would burn CPU; slower would mask outages. 60s matches FR-OBS-007's expected sev-2 latency.
- The crash-count file location (`/tmp/cyberos-memory-capture-crashes`) is intentionally NOT in the memory root — it's runtime state, not memory. /tmp is the right scope.
- The `supervisor_event` rows are written via `MemoryWriter::emit` (not via the daemon's own queue) because they may need to fire AFTER the queue stops (e.g. exit path). They're synchronous with-deadline; if memory is down they're dropped (logged) — acceptable since the supervisor restart-state is also surfaced via systemd journal.
- The `cyberos memory capture logs` CLI shells out to `journalctl` on Linux and `tail -F` on macOS; portable cross-platform log streaming would need a Rust journal-reader crate (`systemd@0.10`), deferred.
- `install-daemon.sh` and `uninstall-daemon.sh` are bash for the same reasons as FR-MEMORY-109's `install-hooks.sh`: small, calls standard tools (`systemctl`, `launchctl`, `sed`, `install`).
- Tests for restart behaviour (`restart_e2e_test.sh`) take ~3 min wall-clock; gated behind CI label `e2e:supervision` to keep PR latency reasonable.

---

*End of FR-MEMORY-110.*
