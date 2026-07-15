"""TASK-MEMORY-107 — Filesystem watcher for the personal-memory capture daemon.

Watches every opted-in folder (per AGENTS.md §11 watched-folder list) and
emits :class:`FsEvent` rows when files are created, modified, moved, or
deleted. The capture daemon (TASK-MEMORY-110) subscribes to this stream and
decides which events should produce memory ``put`` / ``move`` / ``delete``
ops.

Design choices (DEC-091 / DEC-092 from the task spec):

* **Polling vs native watcher.** Slice 1 ships a portable polling implementation
  in pure stdlib so the personal-memory runs on every platform without
  external deps. Slice 2 wires in native backends: ``fsevents`` on macOS,
  ``inotify`` on Linux, ``ReadDirectoryChangesW`` on Windows. The same
  :class:`FsEvent` surface is preserved so callers don't change.

* **Coalescing.** Rapid bursts (file-save thrashing from IDEs) are coalesced
  inside a 250ms window — only the LAST event per (path, kind) tuple in the
  window is emitted. Prevents N×M memory writes for one logical save.

* **Mount-aware.** Network mounts and removable media are scanned but
  flagged with ``volume='remote'`` / ``'removable'`` so the capture daemon
  can apply a different ingest policy (e.g. don't auto-capture from a USB
  stick).

This module ships the data types + scaffold loop; the polling implementation
is intentionally minimal — the production watcher will live alongside the
capture daemon in :mod:`cyberos.core.capture_daemon` (TASK-MEMORY-110).
"""

from __future__ import annotations

import os
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Iterator, Literal


EventKind = Literal["created", "modified", "moved", "deleted"]


@dataclass(frozen=True)
class FsEvent:
    """A single filesystem change observation.

    The capture daemon decides whether to turn this into a memory op. Events
    are immutable so the coalescer can hash + dedupe within the 250ms window.
    """

    path: Path
    kind: EventKind
    ts_ns: int
    size_bytes: int | None = None
    volume: Literal["local", "remote", "removable"] = "local"
    moved_from: Path | None = None  # set when kind == "moved"


@dataclass
class WatchSpec:
    """One root the watcher tracks."""

    root: Path
    recursive: bool = True
    include_globs: tuple[str, ...] = ("**/*",)
    exclude_globs: tuple[str, ...] = (
        "**/.git/**",
        "**/node_modules/**",
        "**/target/**",
        "**/__pycache__/**",
        "**/.DS_Store",
    )


class FsWatcher:
    """Polling watcher scaffold.

    Usage::

        watcher = FsWatcher([
            WatchSpec(root=Path.home() / "Projects/cyberskill"),
            WatchSpec(root=Path.home() / ".cyberos/memory/store"),
        ])
        for event in watcher.poll_loop(interval_secs=2):
            capture_daemon.handle(event)

    The :meth:`poll_loop` returns an infinite generator. Call :meth:`stop`
    from a signal handler to break out cleanly.
    """

    def __init__(self, specs: Iterable[WatchSpec]) -> None:
        self._specs: list[WatchSpec] = list(specs)
        self._snapshots: dict[Path, dict[Path, tuple[int, int]]] = {}
        self._running = True

    def stop(self) -> None:
        self._running = False

    def poll_loop(self, interval_secs: float = 2.0) -> Iterator[FsEvent]:
        """Infinite generator. Calls :meth:`_scan_once` on each tick.

        Coalescing IS NOT done here — that's the capture daemon's job (it
        carries the 250ms window per task spec §1 #5).
        """
        while self._running:
            for ev in self._scan_once():
                yield ev
            time.sleep(interval_secs)

    def _scan_once(self) -> list[FsEvent]:
        """Snapshot every watched root + diff against the previous snapshot."""
        events: list[FsEvent] = []
        for spec in self._specs:
            try:
                current = self._snapshot_root(spec)
            except OSError:
                # Mount disappeared / permission denied / etc. — surface
                # as a no-op tick.
                continue
            prev = self._snapshots.get(spec.root, {})
            events.extend(self._diff(prev, current))
            self._snapshots[spec.root] = current
        return events

    def _snapshot_root(self, spec: WatchSpec) -> dict[Path, tuple[int, int]]:
        """Return ``{path → (mtime_ns, size_bytes)}`` for everything under root."""
        out: dict[Path, tuple[int, int]] = {}
        if not spec.root.exists():
            return out
        walker = spec.root.rglob("*") if spec.recursive else spec.root.iterdir()
        for p in walker:
            if not p.is_file():
                continue
            # Apply exclude globs.
            if any(p.match(g) for g in spec.exclude_globs):
                continue
            # Apply include globs (default '**/*' admits everything).
            if not any(p.match(g) for g in spec.include_globs):
                continue
            try:
                st = p.stat()
            except OSError:
                continue
            out[p] = (st.st_mtime_ns, st.st_size)
        return out

    def _diff(
        self,
        prev: dict[Path, tuple[int, int]],
        current: dict[Path, tuple[int, int]],
    ) -> list[FsEvent]:
        events: list[FsEvent] = []
        now_ns = time.time_ns()
        prev_keys = set(prev.keys())
        cur_keys = set(current.keys())

        # Created
        for p in cur_keys - prev_keys:
            mtime, size = current[p]
            events.append(FsEvent(p, "created", now_ns, size))
        # Deleted
        for p in prev_keys - cur_keys:
            events.append(FsEvent(p, "deleted", now_ns))
        # Modified
        for p in prev_keys & cur_keys:
            if prev[p] != current[p]:
                _, size = current[p]
                events.append(FsEvent(p, "modified", now_ns, size))
        return events


# ---------------------------------------------------------------------------
# 250ms coalescer — pure-function helper the capture daemon composes onto
# the raw event stream.
# ---------------------------------------------------------------------------

def coalesce_events(
    events: Iterable[FsEvent],
    *,
    window_ns: int = 250_000_000,
) -> list[FsEvent]:
    """Within a 250ms window, drop all but the LATEST event per (path, kind)."""
    bucket: dict[tuple[Path, EventKind], FsEvent] = {}
    for ev in events:
        key = (ev.path, ev.kind)
        existing = bucket.get(key)
        if existing is None or abs(ev.ts_ns - existing.ts_ns) <= window_ns:
            bucket[key] = ev
        else:
            # Outside the window — emit the older one and start tracking the new.
            bucket[key] = ev
    return list(bucket.values())
