//! Debounced file watcher for persona hot reloads.

use std::path::PathBuf;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

const DEBOUNCE: Duration = Duration::from_millis(250);
const POLL: Duration = Duration::from_millis(25);

/// Keeps the notify watcher and debounce worker alive.
#[derive(Debug)]
pub struct PersonaWatcher {
    _watcher: RecommendedWatcher,
    _worker: JoinHandle<()>,
}

/// Watch a persona directory and reload the registry after a 250ms debounce.
pub fn start(persona_dir: PathBuf) -> notify::Result<PersonaWatcher> {
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if res.is_ok() {
            let _ = tx.send(());
        }
    })?;
    watcher.watch(&persona_dir, RecursiveMode::Recursive)?;

    let worker_dir = persona_dir.clone();
    let worker = std::thread::spawn(move || {
        let mut pending_since: Option<Instant> = None;
        loop {
            match rx.recv_timeout(POLL) {
                Ok(()) => pending_since = Some(Instant::now()),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
            }

            if pending_since
                .map(|started| started.elapsed() >= DEBOUNCE)
                .unwrap_or(false)
            {
                super::reload(&worker_dir);
                pending_since = None;
            }
        }
    });

    Ok(PersonaWatcher {
        _watcher: watcher,
        _worker: worker,
    })
}
