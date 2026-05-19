"""Tests for FR-MEMORY-107 fs-watcher polling scaffold."""

from __future__ import annotations

from pathlib import Path

import pytest

from cyberos.core.fs_watcher import (
    FsEvent,
    FsWatcher,
    WatchSpec,
    coalesce_events,
)


def test_initial_scan_yields_created(tmp_path: Path) -> None:
    (tmp_path / "a.md").write_text("hello")
    (tmp_path / "b.md").write_text("world")
    w = FsWatcher([WatchSpec(root=tmp_path)])
    events = w._scan_once()
    paths = sorted(str(e.path.name) for e in events)
    assert paths == ["a.md", "b.md"]
    for e in events:
        assert e.kind == "created"
        assert e.size_bytes == 5


def test_second_scan_after_modify(tmp_path: Path) -> None:
    f = tmp_path / "a.md"
    f.write_text("hello")
    w = FsWatcher([WatchSpec(root=tmp_path)])
    _initial = w._scan_once()  # noqa: F841 — establishes baseline
    # mtime granularity can be coarse on some filesystems; touch with explicit delta.
    import os
    import time
    time.sleep(0.01)
    f.write_text("hello world")
    os.utime(f, None)
    events = w._scan_once()
    kinds = sorted(e.kind for e in events)
    assert kinds == ["modified"]
    assert events[0].size_bytes == len("hello world")


def test_delete_yields_deleted(tmp_path: Path) -> None:
    f = tmp_path / "a.md"
    f.write_text("hi")
    w = FsWatcher([WatchSpec(root=tmp_path)])
    w._scan_once()
    f.unlink()
    events = w._scan_once()
    assert [(e.kind, e.path.name) for e in events] == [("deleted", "a.md")]


def test_exclude_globs_filter_out_git(tmp_path: Path) -> None:
    (tmp_path / ".git").mkdir()
    (tmp_path / ".git" / "HEAD").write_text("ref: x")
    (tmp_path / "real.md").write_text("hi")
    w = FsWatcher([WatchSpec(root=tmp_path)])
    events = w._scan_once()
    names = {e.path.name for e in events}
    assert names == {"real.md"}


def test_coalesce_dedupes_within_window() -> None:
    p = Path("/tmp/x")
    events = [
        FsEvent(p, "modified", 1_000_000_000),
        FsEvent(p, "modified", 1_100_000_000),  # 100ms later — within window
        FsEvent(p, "modified", 1_200_000_000),  # 200ms later — within window
    ]
    out = coalesce_events(events, window_ns=250_000_000)
    assert len(out) == 1
    # Latest ts wins
    assert out[0].ts_ns == 1_200_000_000


def test_coalesce_preserves_different_kinds() -> None:
    p = Path("/tmp/x")
    events = [
        FsEvent(p, "created", 1_000_000_000),
        FsEvent(p, "modified", 1_100_000_000),
    ]
    out = coalesce_events(events)
    kinds = sorted(e.kind for e in out)
    assert kinds == ["created", "modified"]
