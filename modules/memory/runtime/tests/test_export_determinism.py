#!/usr/bin/env python3
"""
test_export_determinism.py — verify §11.2 byte-identical export property.

Aspect 13.9 of the Layer-1 improvement catalog.

Per §11.2: two exports of the same state MUST be byte-identical. Catches
non-determinism from __pycache__/, mtime-dependent ordering, etc.

Usage:
    python3 runtime/tests/test_export_determinism.py
"""
from __future__ import annotations
import hashlib
import subprocess
import sys
import tempfile
import time
from pathlib import Path

def find_root() -> Path:
    cur = Path.cwd().resolve()
    while cur != cur.parent:
        if (cur / ".cyberos/memory/store").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos/memory/store/ found")

def main():
    root = find_root()
    exporter = root / "runtime" / "tools" / "cyberos_export.py"
    if not exporter.exists():
        print(f"ERROR: {exporter} not found", file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory() as tmp:
        zip1 = Path(tmp) / "export-1.zip"
        zip2 = Path(tmp) / "export-2.zip"

        # Run export twice with 2s gap (to detect mtime-based non-determinism)
        print(f"→ first export ...")
        r1 = subprocess.run(["python3", str(exporter), str(root), "-o", str(zip1)],
                            capture_output=True, text=True)
        if r1.returncode != 0:
            print(f"first export failed: {r1.stderr}")
            return 2
        time.sleep(2)
        print(f"→ second export ...")
        r2 = subprocess.run(["python3", str(exporter), str(root), "-o", str(zip2)],
                            capture_output=True, text=True)
        if r2.returncode != 0:
            print(f"second export failed: {r2.stderr}")
            return 2

        # Byte-compare
        h1 = hashlib.sha256(zip1.read_bytes()).hexdigest()
        h2 = hashlib.sha256(zip2.read_bytes()).hexdigest()

        if h1 == h2:
            print(f"\n✓ §11.2 byte-identical: SHA256={h1[:24]}...  ({zip1.stat().st_size} bytes)")
            return 0
        else:
            size_diff = zip2.stat().st_size - zip1.stat().st_size
            print(f"\n✗ §11.2 VIOLATION: exports differ")
            print(f"  zip1: {h1[:24]}... ({zip1.stat().st_size} bytes)")
            print(f"  zip2: {h2[:24]}... ({zip2.stat().st_size} bytes)")
            print(f"  size diff: {size_diff:+d} bytes")
            # Suggest common culprits
            print(f"\nCommon non-determinism sources to check:")
            print(f"  - __pycache__/ leakage in export")
            print(f"  - mtime not normalised to fixed epoch")
            print(f"  - entry order not C-locale lexicographic")
            print(f"  - export including index/ or exports/ (should be excluded per §17)")
            return 1

if __name__ == "__main__":
    sys.exit(main())
