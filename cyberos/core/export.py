"""
cyberos.core.export — deterministic zip export of a `.cyberos-memory/` store.

Walks the tree in lexicographic order, re-serialises every JSON-encodable
file canonically via msgspec (sorted keys), and writes a zip with:

* ``ZIP_DEFLATED`` compression at level 6 (default; deterministic),
* fixed timestamp ``2000-01-01T00:00:00Z`` on every entry,
* fixed file mode ``0o644`` on data, ``0o755`` on directory markers,
* sorted file order — never relies on filesystem iteration order,
* no extra ZIP comments or extra fields.

Two calls with the same store produce byte-identical zip bytes. This is
checked by the CI determinism guard (audit report §5.6).

Skipped from the export:

* ``.lock`` — runtime coordination file, not portable;
* ``HEAD`` — derived from the audit ledger, not portable;
* ``audit/current.binlog`` — IS exported (it's the live source of truth);
* ``audit/*.tmp`` and any ``*.tmp`` — partially-written files;
* the derived SQLite database (lives outside the store anyway).
"""

from __future__ import annotations

import hashlib
import os
import stat
import zipfile
from pathlib import Path
from typing import Final, Iterable

import msgspec

_FIXED_TIMESTAMP: Final[tuple[int, int, int, int, int, int]] = (2000, 1, 1, 0, 0, 0)
_DATA_MODE: Final[int] = 0o644
_DIR_MODE: Final[int] = 0o755

# Files we never include in the export bundle.
_EXCLUDE_NAMES: Final[frozenset[str]] = frozenset({".lock", "HEAD", ".DS_Store", "Thumbs.db"})
_EXCLUDE_SUFFIXES: Final[tuple[str, ...]] = (".tmp", ".swp", "~")

# Directory names we never recurse into during the export walk. ``exports``
# is the output target of this function — including it in the input is a
# self-reference that breaks the deterministic-export invariant the moment
# a previous run has left a zip behind. The cache equivalents (``.cache/``,
# Python ``__pycache__/``) are similarly transient runtime state.
_EXCLUDE_DIRS: Final[frozenset[str]] = frozenset({"exports", "__pycache__", ".cache"})


def _is_excluded(rel_path: str, name: str) -> bool:
    if name in _EXCLUDE_NAMES:
        return True
    for suffix in _EXCLUDE_SUFFIXES:
        if name.endswith(suffix):
            return True
    return False


def _walk_sorted(root: Path) -> Iterable[Path]:
    """Lexicographic walk of all regular files under ``root``.

    os.walk uses readdir order, which is not portable. We sort entries
    at every level so the produced ordering is identical on macOS,
    Linux, and Windows.
    """
    stack: list[Path] = [root]
    while stack:
        current = stack.pop()
        try:
            entries = sorted(current.iterdir(), key=lambda p: p.name)
        except (FileNotFoundError, PermissionError):
            continue
        for entry in entries:
            try:
                st = entry.lstat()
            except FileNotFoundError:
                continue
            if stat.S_ISDIR(st.st_mode):
                if entry.name in _EXCLUDE_DIRS:
                    continue
                stack.append(entry)
            elif stat.S_ISREG(st.st_mode):
                yield entry


def _canonicalise(name: str, data: bytes) -> bytes:
    """Re-emit JSON files canonically so an export is byte-deterministic.

    Only JSON files are touched; binary files (binlog, images) are
    pass-through. The canonical form is msgspec's ``order='sorted'``
    output which is RFC 8785 JCS equivalent for our schemas.
    """
    if not name.endswith(".json"):
        return data
    try:
        value = msgspec.json.decode(data)
    except msgspec.DecodeError:
        # File is named .json but not valid JSON — pass through unchanged
        # so the export still captures it for forensic purposes.
        return data
    return msgspec.json.encode(value, order="sorted")


def export_zip(store: Path, out_path: Path) -> str:
    """Write a deterministic zip of ``store`` to ``out_path``.

    Returns the SHA-256 hex of the produced zip bytes — useful for
    sanity-checking that two consecutive exports really are byte-identical.
    """
    store = store.resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Build the zip in a tmp file then atomic-rename, so a crash mid-write
    # doesn't leave a half-zip in place.
    tmp = out_path.with_name(out_path.name + ".tmp")
    if tmp.exists():
        tmp.unlink()

    files: list[tuple[str, bytes]] = []
    for path in _walk_sorted(store):
        rel = path.relative_to(store)
        name = path.name
        if _is_excluded(str(rel), name):
            continue
        try:
            data = path.read_bytes()
        except (FileNotFoundError, PermissionError):
            continue
        data = _canonicalise(rel.as_posix(), data)
        files.append((rel.as_posix(), data))

    # Sort one more time — _walk_sorted is depth-first but we want a
    # purely lexicographic order over relative paths regardless of depth.
    files.sort(key=lambda kv: kv[0])

    with zipfile.ZipFile(tmp, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as zf:
        for rel_path, data in files:
            info = zipfile.ZipInfo(filename=rel_path, date_time=_FIXED_TIMESTAMP)
            info.external_attr = _DATA_MODE << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            # CREATE_SYSTEM=3 (Unix) is platform-invariant.
            info.create_system = 3
            zf.writestr(info, data)

    os.replace(tmp, out_path)

    # SHA-256 of the produced file for the determinism guard.
    h = hashlib.sha256()
    with open(out_path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


__all__ = ["export_zip"]
