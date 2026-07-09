"""
cyberos.core.backup — incremental snapshot tooling.

Different from :mod:`cyberos.core.export`:

* ``export`` produces a single deterministic ``.zip`` of the entire
  store — good for off-host portability, expensive to do daily.
* ``backup`` produces a *directory* snapshot under a configurable target
  with rsync-style hard-links to the previous snapshot. Unchanged files
  cost ~one inode each; only changed files are physically copied.

The snapshot lineage is recorded in ``audit/backups/manifest.json`` —
each snapshot includes its predecessor's path so an auditor can walk
the chain. Snapshots are read-only by convention (mode 0o555 on
directories, 0o444 on files).

Layout::

    <target>/
    ├── 2026-05-14T03-00-00Z/    one snapshot
    │   ├── .cyberos/memory/store/     full store at that time (hard-linked)
    │   └── snapshot.json        metadata (timestamp, predecessor, root hash)
    ├── 2026-05-15T03-00-00Z/
    └── manifest.json            lineage; ``[{snapshot, predecessor, ...}, ...]``

Usage::

    cyberos backup --target /Volumes/backup/cyberos
    cyberos backup --target ~/cyberos-backups --label "post-consolidation"
    cyberos backup --list --target ~/cyberos-backups
    cyberos backup --verify --target ~/cyberos-backups --snapshot 2026-05-14T03-00-00Z
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Optional

from cyberos.core.fsync import durable_dir_sync


@dataclass
class BackupReport:
    target: Path
    snapshot_name: str
    files_linked: int = 0
    files_copied: int = 0
    bytes_copied: int = 0
    predecessor: Optional[str] = None
    errors: list[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.errors


def _walk_store(store: Path) -> Iterable[Path]:
    """Walk every regular file under ``store`` (excluding runtime-only)."""
    skip = {".lock"}
    for path in store.rglob("*"):
        if not path.is_file():
            continue
        if path.name in skip:
            continue
        yield path


def _try_hardlink(src: Path, dst: Path) -> bool:
    """Hard-link src → dst; return True on success.

    Falls back gracefully on cross-device errors (separate filesystem)
    or filesystems that don't support hard links (most network FS).
    Caller MUST copy on failure.
    """
    dst.parent.mkdir(parents=True, exist_ok=True)
    try:
        os.link(src, dst)
        return True
    except OSError:
        return False


def _copy_file(src: Path, dst: Path) -> int:
    """Copy src → dst (preserving mtime); return bytes copied."""
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    return src.stat().st_size


def _snapshot_dir_name() -> str:
    return time.strftime("%Y-%m-%dT%H-%M-%SZ", time.gmtime())


def _root_hash(store: Path) -> str:
    """Hash the set of (rel_path, size, content_hash) tuples.

    Cheap forensic anchor — two snapshots with identical content produce
    the same root hash regardless of file system inode quirks.
    """
    h = hashlib.sha256()
    items = []
    for path in sorted(_walk_store(store)):
        rel = path.relative_to(store).as_posix()
        body = path.read_bytes()
        items.append((rel, len(body), hashlib.sha256(body).hexdigest()))
    for rel, size, sha in items:
        h.update(f"{rel}\t{size}\t{sha}\n".encode("utf-8"))
    return h.hexdigest()


def _load_manifest(target: Path) -> dict:
    mf = target / "manifest.json"
    if not mf.is_file():
        return {"snapshots": []}
    try:
        return json.loads(mf.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {"snapshots": []}


def _save_manifest(target: Path, payload: dict) -> None:
    mf = target / "manifest.json"
    target.mkdir(parents=True, exist_ok=True)
    body = json.dumps(payload, sort_keys=True, indent=2).encode("utf-8") + b"\n"
    tmp = mf.with_suffix(".json.tmp")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(tmp, flags, 0o644)
    try:
        os.write(fd, body)
    finally:
        os.close(fd)
    os.replace(tmp, mf)
    durable_dir_sync(target)


def backup(
    store: Path,
    target: Path,
    *,
    label: str | None = None,
) -> BackupReport:
    """Take an incremental snapshot of ``store`` under ``target``.

    Files that are byte-identical (same inode-relative size + content
    hash) to the previous snapshot are hard-linked; changed files are
    copied. The snapshot manifest tracks lineage so an auditor can walk
    backwards through the chain of snapshots.
    """
    target.mkdir(parents=True, exist_ok=True)
    snap_name = _snapshot_dir_name()
    snap_dir = target / snap_name
    if snap_dir.exists():
        # Same-second collision — append a counter.
        for i in range(1, 10):
            alt = target / f"{snap_name}-{i:02d}"
            if not alt.exists():
                snap_dir = alt
                snap_name = alt.name
                break
        else:
            raise RuntimeError(
                "could not pick a unique snapshot name; try again in 1s"
            )

    report = BackupReport(target=target, snapshot_name=snap_name)
    manifest = _load_manifest(target)

    predecessor: dict | None = None
    if manifest["snapshots"]:
        predecessor = manifest["snapshots"][-1]
        report.predecessor = predecessor["name"]
    pred_root = target / predecessor["name"] / ".cyberos/memory/store" if predecessor else None

    dst_root = snap_dir / ".cyberos/memory/store"
    for src in _walk_store(store):
        rel = src.relative_to(store)
        dst = dst_root / rel

        linked = False
        if pred_root is not None:
            pred_file = pred_root / rel
            if pred_file.is_file():
                # If size matches, the file hasn't changed in a way the
                # filesystem can detect cheaply. (mtime might differ
                # because cp -p preserves it but atomic-renames don't.)
                # The safe + cheap check: same size + same content hash.
                if src.stat().st_size == pred_file.stat().st_size:
                    if src.read_bytes() == pred_file.read_bytes():
                        linked = _try_hardlink(pred_file, dst)
                        if linked:
                            report.files_linked += 1

        if not linked:
            try:
                size = _copy_file(src, dst)
                report.files_copied += 1
                report.bytes_copied += size
            except OSError as exc:
                report.errors.append(f"copy {rel}: {exc}")

    snap_record = {
        "name": snap_name,
        "created_at_ns": time.time_ns(),
        "label": label,
        "predecessor": report.predecessor,
        "root_hash": _root_hash(store),
        "files_linked": report.files_linked,
        "files_copied": report.files_copied,
        "bytes_copied": report.bytes_copied,
    }
    snap_meta_path = snap_dir / "snapshot.json"
    snap_meta_path.write_text(
        json.dumps(snap_record, sort_keys=True, indent=2) + "\n",
        encoding="utf-8",
    )
    manifest["snapshots"].append(snap_record)
    _save_manifest(target, manifest)
    return report


def list_snapshots(target: Path) -> list[dict]:
    return _load_manifest(target)["snapshots"]


def verify_snapshot(target: Path, snapshot_name: str) -> tuple[bool, str]:
    """Recompute the snapshot's root_hash; compare against manifest."""
    manifest = _load_manifest(target)
    record = next(
        (s for s in manifest["snapshots"] if s["name"] == snapshot_name),
        None,
    )
    if record is None:
        return False, f"snapshot {snapshot_name!r} not in manifest"
    snap_store = target / snapshot_name / ".cyberos/memory/store"
    if not snap_store.is_dir():
        return False, f"snapshot data missing at {snap_store}"
    actual = _root_hash(snap_store)
    if actual != record["root_hash"]:
        return False, (
            f"root_hash mismatch: manifest={record['root_hash'][:16]}… "
            f"actual={actual[:16]}…"
        )
    return True, f"OK ({record['files_linked']} linked, {record['files_copied']} copied)"


def format_backup_report(report: BackupReport) -> str:
    lines = [f"cyberos backup → {report.target}/{report.snapshot_name}"]
    if report.predecessor:
        lines.append(f"  predecessor: {report.predecessor}")
    lines.append(
        f"  {report.files_linked} hard-linked, "
        f"{report.files_copied} copied ({report.bytes_copied} bytes)"
    )
    for err in report.errors:
        lines.append(f"  ERROR: {err}")
    return "\n".join(lines)


__all__ = [
    "BackupReport",
    "backup",
    "format_backup_report",
    "list_snapshots",
    "verify_snapshot",
]
