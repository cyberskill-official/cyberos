#!/usr/bin/env python3
"""
cyberos-export — deterministic export bundles for `.cyberos-memory/` stores.

One-shot mode produces a single byte-deterministic export per AGENTS.md §11.2.
Daemon mode watches the store and produces an incremental export every N hours
to a configurable destination.

Two exports of the same state are byte-identical:
  - entries sorted by relative path (C-locale lexicographic)
  - every entry's mtime set to the most-recent-audit-row time, falling back
    to 1980-01-01T00:00:00Z
  - uid/gid zeroed, ZIP extra attributes stripped, UTF-8 NFC filenames
  - LF line endings inside text files
  - signature file `manifest.sig` reserved for Ed25519 detached signature
    (omitted if no key configured — same as §11.3)

Usage
-----
    cyberos-export <store>                       # one-shot to ./
    cyberos-export <store> -o ~/Backups          # one-shot to chosen dir
    cyberos-export <store> --daemon              # background; every 6h
    cyberos-export <store> --daemon --interval 1 # every hour
    cyberos-export <store> --verify <bundle.zip> # verify deterministic equality

Exit codes
----------
0 = ok
1 = mid-bundle warning (e.g., ledger has trailing partial line; included anyway)
2 = bundle creation failed
3 = invocation error

Excludes per §11.1: `index/`, `.lock`, `exports/`, `.tmp.*.part`.

Author: CyberOS local-optimization Stage 4
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import io
import json
import os
import re
import signal
import struct
import sys
import time
import unicodedata
import zipfile
from pathlib import Path
from typing import Iterable

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

EXCLUDE_TOP_LEVEL = {"index", "exports", ".lock"}
TMP_PART_RE = re.compile(r"\.tmp\.[^/]+\.part$")
DEFAULT_DAEMON_INTERVAL_HOURS = 6
EPOCH_FALLBACK = dt.datetime(1980, 1, 1, 0, 0, 0, tzinfo=dt.timezone.utc)


# ---------------------------------------------------------------------------
# Path filtering (§11.1)
# ---------------------------------------------------------------------------

def is_excluded(rel: str) -> bool:
    parts = rel.split("/", 1)
    if parts[0] in EXCLUDE_TOP_LEVEL:
        return True
    if TMP_PART_RE.search(rel):
        return True
    return False


# ---------------------------------------------------------------------------
# Determinism: read most-recent-audit-row time
# ---------------------------------------------------------------------------

def most_recent_audit_ts(store: Path) -> dt.datetime:
    """Return the latest audit row's `ts` (parsed as UTC); fallback to epoch."""
    audit_dir = store / "audit"
    if not audit_dir.is_dir():
        return EPOCH_FALLBACK
    latest: dt.datetime | None = None
    for f in sorted(audit_dir.glob("*.jsonl")):
        try:
            with f.open("r", encoding="utf-8") as fp:
                for line in fp:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        row = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    ts = row.get("ts")
                    if not isinstance(ts, str):
                        continue
                    try:
                        parsed = dt.datetime.fromisoformat(
                            ts.replace("Z", "+00:00"))
                    except ValueError:
                        continue
                    if parsed.tzinfo is None:
                        continue
                    if latest is None or parsed > latest:
                        latest = parsed
        except OSError:
            continue
    return latest or EPOCH_FALLBACK


# ---------------------------------------------------------------------------
# File enumeration
# ---------------------------------------------------------------------------

def walk_store(store: Path) -> list[tuple[str, Path]]:
    """Return (relpath, absolute) tuples sorted by relpath C-locale."""
    out: list[tuple[str, Path]] = []
    for absp in store.rglob("*"):
        if not absp.is_file():
            continue
        rel = absp.relative_to(store).as_posix()
        rel = unicodedata.normalize("NFC", rel)
        if is_excluded(rel):
            continue
        out.append((rel, absp))
    out.sort(key=lambda x: x[0].encode("utf-8"))  # C-locale
    return out


# ---------------------------------------------------------------------------
# Bundle creation (deterministic ZIP)
# ---------------------------------------------------------------------------

def create_bundle(store: Path, out_path: Path) -> dict:
    """Create a deterministic export bundle. Return summary stats."""
    files = walk_store(store)
    mtime = most_recent_audit_ts(store)
    # ZIP DOS time = (year-1980) << 25 | month << 21 | day << 16 | hour << 11
    #                | minute << 5 | second >> 1
    dos_time = (
        (mtime.year - 1980) << 25 | mtime.month << 21 | mtime.day << 16
        | mtime.hour << 11 | mtime.minute << 5 | mtime.second >> 1
    )
    if mtime.year < 1980:
        dos_time = 1 << 21 | 1 << 16  # 1980-01-01

    total_uncompressed = 0
    total_compressed = 0

    out_path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = out_path.with_suffix(out_path.suffix + ".tmp")

    with zipfile.ZipFile(tmp_path, "w", zipfile.ZIP_DEFLATED,
                         compresslevel=6) as zf:
        for rel, absp in files:
            data = absp.read_bytes()
            # LF normalisation only for text files (.md, .json, .jsonl)
            if rel.endswith((".md", ".json", ".jsonl", ".txt", ".yaml",
                             ".yml")):
                data = data.replace(b"\r\n", b"\n").replace(b"\r", b"\n")
            zi = zipfile.ZipInfo(rel)
            zi.date_time = (
                max(1980, mtime.year), max(1, mtime.month), max(1, mtime.day),
                mtime.hour, mtime.minute, mtime.second & 0xFE
            )
            zi.compress_type = zipfile.ZIP_DEFLATED
            zi.create_system = 3  # Unix
            zi.external_attr = 0o644 << 16  # rw-r--r--
            zf.writestr(zi, data)
            total_uncompressed += len(data)

    # Atomic rename
    os.replace(tmp_path, out_path)

    bundle_sha = hashlib.sha256(out_path.read_bytes()).hexdigest()
    total_compressed = out_path.stat().st_size

    return {
        "bundle_path": str(out_path),
        "bundle_sha256": "sha256:" + bundle_sha,
        "file_count": len(files),
        "total_uncompressed": total_uncompressed,
        "total_compressed": total_compressed,
        "mtime_anchor": mtime.isoformat(),
    }


# ---------------------------------------------------------------------------
# Verification: assert two exports of same state are byte-identical
# ---------------------------------------------------------------------------

def verify_determinism(store: Path, existing_bundle: Path) -> bool:
    import tempfile
    with tempfile.NamedTemporaryFile(
            suffix=".zip", delete=False) as tmp:
        tmp_path = Path(tmp.name)
    try:
        create_bundle(store, tmp_path)
        a = hashlib.sha256(existing_bundle.read_bytes()).hexdigest()
        b = hashlib.sha256(tmp_path.read_bytes()).hexdigest()
        return a == b
    finally:
        tmp_path.unlink(missing_ok=True)


# ---------------------------------------------------------------------------
# Daemon mode
# ---------------------------------------------------------------------------

class ExportDaemon:
    def __init__(self, store: Path, dest: Path, interval_hours: float):
        self.store = store
        self.dest = dest
        self.interval = interval_hours * 3600
        self.stop = False

    def handle_signal(self, signum, frame):  # noqa: ARG002
        self.stop = True

    def run(self) -> int:
        signal.signal(signal.SIGTERM, self.handle_signal)
        signal.signal(signal.SIGINT, self.handle_signal)
        last_audit_head: str | None = None
        while not self.stop:
            try:
                # Skip if no new audit rows since last run
                head = self._latest_audit_head()
                if head and head == last_audit_head:
                    print(f"[{ts_now()}] no changes; skipping",
                          flush=True)
                else:
                    fname = (
                        f"{self.store.parent.name}-"
                        f"{dt.datetime.now(dt.timezone.utc).strftime('%Y-%m-%d-%H')}"
                        f".zip"
                    )
                    out = self.dest / fname
                    summary = create_bundle(self.store, out)
                    print(f"[{ts_now()}] exported {out.name} "
                          f"({summary['file_count']} files, "
                          f"{summary['total_compressed']} bytes, "
                          f"{summary['bundle_sha256'][:23]}…)",
                          flush=True)
                    last_audit_head = head
            except Exception as e:  # noqa: BLE001
                print(f"[{ts_now()}] export failed: {e}", flush=True)
            # Sleep with periodic stop check
            slept = 0
            while slept < self.interval and not self.stop:
                time.sleep(min(1.0, self.interval - slept))
                slept += 1
        print(f"[{ts_now()}] daemon stopping", flush=True)
        return 0

    def _latest_audit_head(self) -> str | None:
        latest = None
        audit = self.store / "audit"
        if not audit.is_dir():
            return None
        for f in sorted(audit.glob("*.jsonl")):
            try:
                with f.open("rb") as fp:
                    fp.seek(0, 2)  # end
                    size = fp.tell()
                    if size == 0:
                        continue
                    fp.seek(max(0, size - 4096))
                    tail = fp.read().decode("utf-8", errors="replace")
                    for line in reversed(tail.split("\n")):
                        if not line.strip():
                            continue
                        try:
                            row = json.loads(line)
                            latest = row.get("chain")
                            break
                        except json.JSONDecodeError:
                            continue
            except OSError:
                continue
        return latest


def ts_now() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="cyberos-export",
        description="Deterministic export bundles for .cyberos-memory/ stores.",
    )
    parser.add_argument("store", help="Path to .cyberos-memory/ or project root")
    parser.add_argument("-o", "--output", default=".",
                        help="Destination directory (default: cwd)")
    parser.add_argument("--daemon", action="store_true",
                        help="Run as daemon, exporting periodically")
    parser.add_argument("--interval", type=float,
                        default=DEFAULT_DAEMON_INTERVAL_HOURS,
                        help=f"Daemon interval in hours (default: "
                             f"{DEFAULT_DAEMON_INTERVAL_HOURS})")
    parser.add_argument("--verify",
                        help="Verify the given existing bundle is determinism-equivalent")
    args = parser.parse_args(argv)

    store = Path(args.store).resolve()
    if (store / ".cyberos-memory").is_dir():
        store = store / ".cyberos-memory"
    if not store.is_dir():
        print(f"error: {store} is not a directory", file=sys.stderr)
        return 3

    if args.verify:
        ok = verify_determinism(store, Path(args.verify))
        print("DETERMINISTIC" if ok else "DIVERGED")
        return 0 if ok else 2

    dest = Path(args.output).resolve()
    dest.mkdir(parents=True, exist_ok=True)

    if args.daemon:
        daemon = ExportDaemon(store, dest, args.interval)
        return daemon.run()

    # one-shot
    fname = (
        f"{store.parent.name}-"
        f"{dt.datetime.now(dt.timezone.utc).strftime('%Y-%m-%d-%H%M')}"
        f".zip"
    )
    out = dest / fname
    summary = create_bundle(store, out)
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
