"""
cyberos.core.prune — sweep original binlog segments after archival.

After ``cyberos consolidate`` produces a ``<segment>.binlog.zst`` archive,
the original ``.binlog`` can be removed after a soak window — typically
30 days. ``cyberos prune`` performs that sweep with safety checks:

1. The ``.zst`` MUST exist for every segment being pruned.
2. The ``.zst`` MUST decompress to bytes that hash to the same SHA-256
   as the original. (Defends against silent zstd-archive corruption.)
3. The original's mtime MUST be older than ``--soak-days`` (default 30).
4. Each prune emits a per-segment record under
   ``audit/prune-history/<timestamp>-<segment>.json`` so the event is
   auditable.

``cyberos prune --restore`` is the inverse: decompresses a ``.zst`` back
to its original ``.binlog`` and records that fact. The prune-history
records make this a clean two-way operation.

NEVER prunes ``current.binlog``. NEVER prunes segments whose ``.zst``
fails the decompression cross-check.
"""

from __future__ import annotations

import hashlib
import json
import os
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from cyberos.core.fsync import durable_dir_sync, durable_sync


@dataclass
class PruneReport:
    store: Path
    pruned: list[str] = field(default_factory=list)
    skipped: list[tuple[str, str]] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)
    bytes_freed: int = 0

    @property
    def ok(self) -> bool:
        return not self.errors


@dataclass
class RestoreReport:
    store: Path
    restored: list[str] = field(default_factory=list)
    skipped: list[tuple[str, str]] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.errors


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _atomic_write_json(path: Path, payload: dict) -> None:
    body = json.dumps(payload, sort_keys=True, indent=2).encode("utf-8") + b"\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + ".tmp")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(tmp, flags, 0o600)
    try:
        os.write(fd, body)
        durable_sync(fd)
    finally:
        os.close(fd)
    os.replace(tmp, path)
    durable_dir_sync(path.parent)


def _verify_archive(zst_path: Path, expected_sha: str) -> bool:
    """Decompress and confirm SHA-256 matches the original."""
    try:
        import zstandard as zstd  # type: ignore[import-not-found]
    except ImportError:
        raise RuntimeError(
            "prune requires 'zstandard'. Install: "
            "pip install zstandard --break-system-packages"
        )
    decompressor = zstd.ZstdDecompressor()
    decompressed = decompressor.decompress(zst_path.read_bytes())
    return _sha256(decompressed) == expected_sha


def prune(
    store: Path,
    *,
    soak_days: int = 30,
    dry_run: bool = False,
) -> PruneReport:
    """Sweep original binlog segments whose .zst exists and is verified.

    Skips ``current.binlog`` and any segment whose archive can't be
    decompressed to byte-equal bytes.
    """
    report = PruneReport(store=store)
    audit = store / "audit"
    if not audit.is_dir():
        report.errors.append(f"audit dir missing: {audit}")
        return report

    horizon_ns = (time.time() - soak_days * 86400) * 1e9

    for binlog in sorted(audit.glob("*.binlog")):
        if binlog.name == "current.binlog":
            continue
        archive = binlog.with_suffix(".binlog.zst")
        if not archive.is_file():
            report.skipped.append((binlog.name, "no .zst archive"))
            continue
        stat = binlog.stat()
        if stat.st_mtime_ns > horizon_ns:
            report.skipped.append(
                (binlog.name, f"within soak window ({soak_days}d)")
            )
            continue

        # Cross-check: decompressed archive bytes == original bytes.
        original_bytes = binlog.read_bytes()
        original_sha = _sha256(original_bytes)
        try:
            ok = _verify_archive(archive, original_sha)
        except Exception as exc:  # noqa: BLE001
            report.errors.append(
                f"{binlog.name}: archive verify error: {exc}"
            )
            continue
        if not ok:
            report.errors.append(
                f"{binlog.name}: archive sha mismatch — refusing to prune"
            )
            continue

        if dry_run:
            report.pruned.append(binlog.name)  # would-prune
            report.bytes_freed += stat.st_size
            continue

        # Emit a prune-history record BEFORE deleting the original.
        history_path = audit / "prune-history" / (
            f"{time.strftime('%Y-%m-%dT%H-%M-%SZ', time.gmtime())}-{binlog.name}.json"
        )
        _atomic_write_json(history_path, {
            "action": "prune",
            "segment": binlog.name,
            "archive": archive.name,
            "original_sha256": original_sha,
            "original_size": stat.st_size,
            "soak_days": soak_days,
            "pruned_at_ns": time.time_ns(),
        })
        binlog.unlink()
        durable_dir_sync(audit)
        report.pruned.append(binlog.name)
        report.bytes_freed += stat.st_size

    return report


def restore(
    store: Path,
    *,
    segment_names: list[str] | None = None,
) -> RestoreReport:
    """Decompress ``.zst`` archives back to ``.binlog`` for the named segments.

    Inverse of :func:`prune`. If ``segment_names`` is None, restores
    every archive whose corresponding ``.binlog`` is missing.
    """
    report = RestoreReport(store=store)
    audit = store / "audit"
    if not audit.is_dir():
        report.errors.append(f"audit dir missing: {audit}")
        return report

    try:
        import zstandard as zstd  # type: ignore[import-not-found]
    except ImportError:
        report.errors.append(
            "restore requires 'zstandard'. Install: "
            "pip install zstandard --break-system-packages"
        )
        return report

    targets: list[Path]
    if segment_names:
        targets = [audit / s for s in segment_names]
    else:
        targets = []
        for zst in sorted(audit.glob("*.binlog.zst")):
            original = zst.with_suffix("")  # strip .zst → .binlog
            if not original.is_file():
                targets.append(original)

    for binlog in targets:
        zst = binlog.with_suffix(".binlog.zst")
        if not zst.is_file():
            report.skipped.append((binlog.name, "no .zst archive"))
            continue
        if binlog.is_file():
            report.skipped.append((binlog.name, "already present"))
            continue
        try:
            decompressor = zstd.ZstdDecompressor()
            data = decompressor.decompress(zst.read_bytes())
        except Exception as exc:  # noqa: BLE001
            report.errors.append(f"{binlog.name}: decompress failed: {exc}")
            continue

        # Atomic write.
        tmp = binlog.with_suffix(".binlog.tmp")
        flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
        fd = os.open(tmp, flags, 0o600)
        try:
            os.write(fd, data)
            durable_sync(fd)
        finally:
            os.close(fd)
        os.replace(tmp, binlog)
        durable_dir_sync(audit)

        # Audit the restore.
        history_path = audit / "prune-history" / (
            f"{time.strftime('%Y-%m-%dT%H-%M-%SZ', time.gmtime())}-{binlog.name}-restore.json"
        )
        _atomic_write_json(history_path, {
            "action": "restore",
            "segment": binlog.name,
            "restored_at_ns": time.time_ns(),
            "decompressed_sha256": _sha256(data),
        })
        report.restored.append(binlog.name)

    return report


def format_prune_report(report: PruneReport, *, dry_run: bool = False) -> str:
    lines = [f"cyberos prune — {report.store}"]
    label = "would prune" if dry_run else "pruned"
    for seg in report.pruned:
        lines.append(f"  {label}: {seg}")
    for seg, reason in report.skipped:
        lines.append(f"  skip: {seg} ({reason})")
    for err in report.errors:
        lines.append(f"  ERROR: {err}")
    lines.append(
        f"  total: {len(report.pruned)} segment(s), {report.bytes_freed} bytes "
        f"{'would be freed' if dry_run else 'freed'}"
    )
    return "\n".join(lines)


def format_restore_report(report: RestoreReport) -> str:
    lines = [f"cyberos prune --restore — {report.store}"]
    for seg in report.restored:
        lines.append(f"  restored: {seg}")
    for seg, reason in report.skipped:
        lines.append(f"  skip: {seg} ({reason})")
    for err in report.errors:
        lines.append(f"  ERROR: {err}")
    lines.append(f"  total: {len(report.restored)} segment(s) restored")
    return "\n".join(lines)


__all__ = [
    "PruneReport",
    "RestoreReport",
    "format_prune_report",
    "format_restore_report",
    "prune",
    "restore",
]
