//! Spawns the Python memory-sync daemon as a child process and restarts it on
//! exit. Includes a 5-restarts-per-60s circuit breaker; once tripped, the
//! supervisor sleeps a long backoff before retrying.
//!
//! The daemon module (`cyberos.core.memory_sync_daemon`) does not exist yet in
//! `modules/memory/`. The supervisor therefore handles ENOENT-on-spawn (python3
//! missing) and rapid-exit (module-not-found) gracefully by sleeping and
//! retrying — until the daemon ships, the desktop app stays "online" but no
//! syncing happens.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use tauri::AppHandle;
use tokio::process::Command;

const DAEMON_PYMODULE: &str = "cyberos.core.memory_sync_daemon";

const CIRCUIT_WINDOW: Duration = Duration::from_secs(60);
const CIRCUIT_TRIP_RESTARTS: usize = 5;
const CIRCUIT_LONG_BACKOFF: Duration = Duration::from_secs(300);

const SHORT_BACKOFF_BASE: Duration = Duration::from_secs(1);
const SHORT_BACKOFF_MAX: Duration = Duration::from_secs(30);

/// Entry point invoked from `main.rs` setup. Loops forever, supervising the
/// child process. The `AppHandle` is kept for future tray-state updates; the
/// first slice doesn't wire that yet.
pub async fn start(_app: AppHandle) {
    tracing::info!(daemon = DAEMON_PYMODULE, "starting memory-sync supervisor");

    let mut restart_marks: VecDeque<Instant> = VecDeque::with_capacity(CIRCUIT_TRIP_RESTARTS + 1);
    let mut current_backoff = SHORT_BACKOFF_BASE;

    loop {
        // Trim window.
        let cutoff = Instant::now() - CIRCUIT_WINDOW;
        while restart_marks.front().map_or(false, |t| *t < cutoff) {
            restart_marks.pop_front();
        }

        if restart_marks.len() >= CIRCUIT_TRIP_RESTARTS {
            tracing::error!(
                restarts = restart_marks.len(),
                window_s = CIRCUIT_WINDOW.as_secs(),
                "circuit breaker tripped; sleeping {} s before retry",
                CIRCUIT_LONG_BACKOFF.as_secs(),
            );
            tokio::time::sleep(CIRCUIT_LONG_BACKOFF).await;
            restart_marks.clear();
            current_backoff = SHORT_BACKOFF_BASE;
            continue;
        }

        restart_marks.push_back(Instant::now());
        let exit = run_daemon_once().await;

        match exit {
            Ok(status) => {
                tracing::warn!(?status, "memory-sync daemon exited");
                // Exit-zero still counts as a restart event — daemons shouldn't exit cleanly.
                current_backoff = (current_backoff * 2).min(SHORT_BACKOFF_MAX);
            }
            Err(SpawnError::NotFound) => {
                tracing::warn!(
                    "python3 + {DAEMON_PYMODULE} not available; will retry after backoff. \
                     Install the cyberos-memory package to enable sync."
                );
                current_backoff = (current_backoff * 2).min(SHORT_BACKOFF_MAX);
            }
            Err(SpawnError::Other(e)) => {
                tracing::error!(error = %e, "spawn failed");
                current_backoff = (current_backoff * 2).min(SHORT_BACKOFF_MAX);
            }
        }

        tokio::time::sleep(current_backoff).await;
    }
}

#[derive(Debug)]
enum SpawnError {
    NotFound,
    Other(String),
}

/// Spawns `python3 -m cyberos.core.memory_sync_daemon` and waits for it to
/// exit. Returns the exit status on a clean exit, or `SpawnError` if we
/// couldn't even start the child.
async fn run_daemon_once() -> Result<std::process::ExitStatus, SpawnError> {
    let mut cmd = Command::new("python3");
    cmd.arg("-m").arg(DAEMON_PYMODULE);
    cmd.kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(SpawnError::NotFound),
        Err(e) => return Err(SpawnError::Other(e.to_string())),
    };

    tracing::info!(pid = ?child.id(), "memory-sync daemon spawned");

    child
        .wait()
        .await
        .map_err(|e| SpawnError::Other(format!("wait failed: {e}")))
}
