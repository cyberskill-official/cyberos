"""FR-BRAIN-105 — Watched-folder invariants.

Each Personal BRAIN tracks a list of folders the capture daemon watches.
The list lives at ``<store>/watched_folders.json`` with shape::

    {
        "version": 1,
        "folders": [
            {
                "path": "/Users/stephen/Projects/cyberskill",
                "added_at_ns": 1747200611000000000,
                "include_globs": ["**/*.md", "**/*.py"],
                "exclude_globs": ["**/.git/**", "**/node_modules/**"],
                "sync_class_default": "private"
            },
            …
        ]
    }

The doctor invariants enforce:

* Every path is absolute and exists on this device.
* Every path is unique (no double-watch).
* No path is inside another watched path (no nested duplication).
* ``sync_class_default`` is a closed-enum value (matches FR-BRAIN-106).
* No path is in a known-toxic location (``/`` root, ``/etc``, ``/var``, etc.)
  that would either flood the daemon or capture system files.

The doctor surfaces invariant violations as
:class:`WatchedFolderError` instances; the existing
``cyberos.core.invariants`` runner wraps them as catastrophic vs
recoverable based on which check fires.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable

from cyberos.core.sync_class import SYNC_CLASS_ENUM


# Locations we refuse to watch — capturing these would flood BRAIN with
# system noise or system-sensitive data.
_TOXIC_PREFIXES: tuple[str, ...] = (
    "/",                # exact-match check below — never the literal root
    "/etc",
    "/var",
    "/usr",
    "/bin",
    "/sbin",
    "/dev",
    "/proc",
    "/sys",
    "/System",          # macOS
    "/Library",         # macOS — could be watched as subdir but not root
    "C:\\Windows",
    "C:\\Program Files",
)


class WatchedFolderError(ValueError):
    """A watched-folder invariant failed."""


@dataclass(frozen=True)
class WatchedFolder:
    path: Path
    added_at_ns: int
    include_globs: tuple[str, ...]
    exclude_globs: tuple[str, ...]
    sync_class_default: str = "private"


def load_watched_folders(store: Path) -> list[WatchedFolder]:
    """Read ``<store>/watched_folders.json``. Returns [] if the file is absent."""
    p = store / "watched_folders.json"
    if not p.exists():
        return []
    data = json.loads(p.read_text(encoding="utf-8"))
    if not isinstance(data, dict) or "folders" not in data:
        raise WatchedFolderError(f"{p}: malformed — missing 'folders' key")
    out = []
    for entry in data["folders"]:
        if not isinstance(entry, dict) or "path" not in entry:
            raise WatchedFolderError(f"{p}: entry missing 'path'")
        out.append(WatchedFolder(
            path=Path(entry["path"]),
            added_at_ns=int(entry.get("added_at_ns", 0)),
            include_globs=tuple(entry.get("include_globs", ["**/*"])),
            exclude_globs=tuple(entry.get("exclude_globs", [])),
            sync_class_default=str(entry.get("sync_class_default", "private")),
        ))
    return out


def check_invariants(folders: Iterable[WatchedFolder]) -> list[WatchedFolderError]:
    """Run every invariant. Returns a list of errors (empty = pass)."""
    folders_list = list(folders)
    errors: list[WatchedFolderError] = []

    # Invariant 1 — absolute paths.
    for f in folders_list:
        if not f.path.is_absolute():
            errors.append(WatchedFolderError(
                f"path {f.path} is not absolute — relative watched folders are forbidden"
            ))

    # Invariant 2 — paths exist on this device.
    # (Doctor MAY skip this on portable BRAINs; for now, flag.)
    for f in folders_list:
        if f.path.is_absolute() and not f.path.exists():
            errors.append(WatchedFolderError(
                f"path {f.path} does not exist on this device — "
                "remove it or sync the BRAIN from a device where it does"
            ))

    # Invariant 3 — uniqueness.
    seen: set[Path] = set()
    for f in folders_list:
        if f.path in seen:
            errors.append(WatchedFolderError(f"path {f.path} listed twice"))
        seen.add(f.path)

    # Invariant 4 — no nesting.
    paths = sorted([f.path for f in folders_list], key=lambda p: len(str(p)))
    for i, child in enumerate(paths):
        for ancestor in paths[:i]:
            try:
                child.relative_to(ancestor)
            except ValueError:
                continue
            if child != ancestor:
                errors.append(WatchedFolderError(
                    f"path {child} is nested inside {ancestor} — "
                    "remove one or the other"
                ))
                break

    # Invariant 5 — sync_class_default enum.
    for f in folders_list:
        if f.sync_class_default not in SYNC_CLASS_ENUM:
            errors.append(WatchedFolderError(
                f"path {f.path}: sync_class_default {f.sync_class_default!r} "
                f"not in closed enum {sorted(SYNC_CLASS_ENUM)}"
            ))

    # Invariant 6 — toxic-prefix gate.
    for f in folders_list:
        s = str(f.path)
        # Exact root.
        if s == "/":
            errors.append(WatchedFolderError(
                f"path {f.path} is the filesystem root — refuse to watch"
            ))
            continue
        for tox in _TOXIC_PREFIXES:
            if tox == "/":
                continue
            if s == tox or s.startswith(tox + "/") or s.startswith(tox + "\\"):
                errors.append(WatchedFolderError(
                    f"path {f.path} is in toxic location {tox!r} — refuse to watch"
                ))
                break

    return errors


# ---------------------------------------------------------------------------
# Public doctor hook — called by cyberos.core.invariants.run_all
# ---------------------------------------------------------------------------

def doctor_check(store: Path) -> list[WatchedFolderError]:
    """Top-level doctor hook. Returns every invariant violation."""
    try:
        folders = load_watched_folders(store)
    except WatchedFolderError as e:
        return [e]
    return check_invariants(folders)
