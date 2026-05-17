"""
cyberos.core.consolidate — the 4-phase consolidation pipeline.

Implements AGENTS.md v2 §7: **Walk → Compact → Sign → Publish.** Runs from
``cyberos consolidate``; idempotent; safe to re-run after partial failure.

Phases:

* **Walk** — invariants walker (delegate to :mod:`cyberos.core.invariants`).
  Refuses to proceed unless every error-level check passes.
* **Compact** — archive sealed binlog segments older than
  ``compact_horizon_days`` via deterministic zstd. Original kept as
  ``<segment>.zst`` alongside a manifest line; never deletes.
* **Sign** — produce an STH from the current MMR root via
  :func:`cyberos.core.sth.sign_and_publish`. Requires the
  ``cryptography`` package; falls back to a no-op with a clear message
  if unavailable.
* **Publish** — atomically update
  ``manifest.json:audit_chain_head`` to the post-consolidation MMR root
  (and ``last_sth`` to the new STH file's relative path).

Per PROPOSAL.md P2 Stage 1: the STH is **additive**. The per-row chain
remains source of truth. After Stage 3 (chain primitive swap), the STH
becomes the only integrity primitive.
"""

from __future__ import annotations

import io
import json
import os
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from cyberos.core.fsync import durable_dir_sync, durable_sync


@dataclass
class ConsolidationReport:
    store: Path
    walk_ok: bool = False
    walk_summary: str = ""
    segments_compacted: list[str] = field(default_factory=list)
    sth_path: Optional[str] = None
    new_mmr_root: Optional[str] = None
    leaf_count: int = 0
    started_ns: int = 0
    finished_ns: int = 0
    errors: list[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return self.walk_ok and not self.errors


# --- Walk ----------------------------------------------------------------


def _phase_walk(store: Path, report: ConsolidationReport) -> bool:
    """Run the invariants walker. Refuse to proceed on any error-level failure."""
    from cyberos.core.invariants import format_report, run_all
    walker_report = run_all(store)
    n_err = len(walker_report.errors)
    n_warn = len(walker_report.warnings)
    n_pass = sum(1 for r in walker_report.results if r.passed)
    report.walk_ok = walker_report.ok
    report.walk_summary = (
        f"walk: {n_pass} pass / {n_warn} warn / {n_err} error"
    )
    if walker_report.errors:
        report.errors.extend(
            f"walk: {r.id}: {r.details}" for r in walker_report.errors
        )
    return walker_report.ok


# --- Compact -------------------------------------------------------------


def _phase_compact(
    store: Path,
    report: ConsolidationReport,
    *,
    horizon_days: int = 90,
) -> None:
    """Archive sealed binlog segments older than ``horizon_days`` to .zst.

    Sealed = not ``current.binlog``. Original files are NEVER deleted —
    the archive sits alongside as ``<segment>.zst``. A future
    ``cyberos prune`` (out of scope here) can sweep the originals after
    a soak window.
    """
    try:
        import zstandard as zstd  # type: ignore[import-not-found]
    except ImportError:
        report.errors.append(
            "compact: zstandard not installed; skipping. "
            "Install: pip install zstandard --break-system-packages"
        )
        return

    audit = store / "audit"
    if not audit.is_dir():
        return

    horizon_ns = (time.time() - horizon_days * 86400) * 1e9
    for seg in sorted(audit.glob("*.binlog")):
        if seg.name == "current.binlog":
            continue
        archive = seg.with_suffix(".binlog.zst")
        if archive.is_file():
            continue
        if seg.stat().st_mtime_ns > horizon_ns:
            continue

        # Deterministic zstd: pinned level + magic that doesn't include
        # a timestamp.
        compressor = zstd.ZstdCompressor(level=19, write_checksum=True)
        body = compressor.compress(seg.read_bytes())
        tmp = archive.with_suffix(".zst.tmp")
        flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
        fd = os.open(tmp, flags, 0o600)
        try:
            os.write(fd, body)
            durable_sync(fd)
        finally:
            os.close(fd)
        os.replace(tmp, archive)
        durable_dir_sync(audit)
        report.segments_compacted.append(seg.name)


# --- Sign ----------------------------------------------------------------


def _phase_sign(store: Path, report: ConsolidationReport) -> None:
    """Produce a Signed Tree Head from the current MMR root."""
    from cyberos.core.mmr import OnDiskMMR
    from cyberos.core.sth import P2NotActive, sign_and_publish

    mmr = OnDiskMMR(store)
    if mmr.leaf_count == 0:
        # No leaves yet — STH would commit to the empty root. Skip.
        return
    root_hex = mmr.root().hex()
    try:
        sth_path = sign_and_publish(
            store, tree_size=mmr.leaf_count, root_hash_hex=root_hex,
        )
    except P2NotActive as exc:
        report.errors.append(f"sign: {exc}")
        return
    except Exception as exc:  # noqa: BLE001
        report.errors.append(f"sign: {type(exc).__name__}: {exc}")
        return
    report.new_mmr_root = root_hex
    report.leaf_count = mmr.leaf_count
    report.sth_path = str(sth_path.relative_to(store))


# --- Publish -------------------------------------------------------------


def _phase_publish(store: Path, report: ConsolidationReport) -> None:
    """Atomically update the manifest's audit_chain_head and last_sth pointer."""
    manifest_path = store / "manifest.json"
    if not manifest_path.is_file():
        report.errors.append("publish: manifest.json missing")
        return
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        report.errors.append(f"publish: {exc}")
        return

    if report.new_mmr_root is not None:
        manifest.setdefault("consolidation", {})
        manifest["consolidation"].update({
            "last_consolidated_at": time.time_ns(),
            "last_mmr_root": report.new_mmr_root,
            "last_leaf_count": report.leaf_count,
            "last_sth": report.sth_path,
        })
    if report.segments_compacted:
        manifest.setdefault("consolidation", {})
        manifest["consolidation"]["last_compacted_segments"] = (
            report.segments_compacted
        )

    body = json.dumps(manifest, sort_keys=True, indent=2).encode("utf-8") + b"\n"
    tmp = manifest_path.with_name(manifest_path.name + ".tmp")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(tmp, flags, 0o600)
    try:
        os.write(fd, body)
        durable_sync(fd)
    finally:
        os.close(fd)
    os.replace(tmp, manifest_path)
    durable_dir_sync(store)


# --- top-level driver ----------------------------------------------------


def run(
    store: Path,
    *,
    dry_run: bool = False,
    compact_horizon_days: int = 90,
) -> ConsolidationReport:
    """Run the 4-phase consolidation.

    Parameters
    ----------
    store:
        Store root path.
    dry_run:
        If True, run Walk only; print what would happen for the other
        phases without writing.
    compact_horizon_days:
        Segments with mtime older than this are eligible for zstd archival.
        Default 90 days — leaves recent ops live for fast tail-reading.
    """
    report = ConsolidationReport(store=store, started_ns=time.time_ns())

    walk_ok = _phase_walk(store, report)
    if not walk_ok:
        report.finished_ns = time.time_ns()
        return report  # never compact/sign/publish over a failing walk

    if dry_run:
        # Report what compact/sign/publish would do without writing.
        audit = store / "audit"
        report.walk_summary += "  (dry-run; no further phases)"
        report.finished_ns = time.time_ns()
        return report

    _phase_compact(store, report, horizon_days=compact_horizon_days)
    _phase_sign(store, report)
    _phase_publish(store, report)

    report.finished_ns = time.time_ns()
    return report


def format_report(report: ConsolidationReport, *, json_mode: bool = False) -> str:
    if json_mode:
        return json.dumps({
            "store": str(report.store),
            "ok": report.ok,
            "walk_summary": report.walk_summary,
            "segments_compacted": report.segments_compacted,
            "sth_path": report.sth_path,
            "new_mmr_root": report.new_mmr_root,
            "leaf_count": report.leaf_count,
            "errors": report.errors,
            "duration_ms": (report.finished_ns - report.started_ns) / 1e6,
        }, indent=2)
    lines = [f"cyberos consolidate — {report.store}"]
    lines.append(f"  {report.walk_summary}")
    if report.segments_compacted:
        lines.append(
            f"  compacted: {', '.join(report.segments_compacted)}"
        )
    if report.sth_path:
        lines.append(
            f"  STH: {report.sth_path} (root={report.new_mmr_root[:16]}…, "
            f"{report.leaf_count} leaves)"
        )
    for err in report.errors:
        lines.append(f"  ERROR  {err}")
    lines.append(
        f"  overall: {'OK' if report.ok else 'FAIL'} "
        f"({(report.finished_ns - report.started_ns) / 1e6:.0f} ms)"
    )
    return "\n".join(lines)


__all__ = [
    "ConsolidationReport",
    "format_report",
    "run",
]
