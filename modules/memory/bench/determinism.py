"""
bench/determinism.py — deterministic-export round-trip guard.

Audit report §3.C.5 invariant: two ``cyberos export`` calls on the same
store produce byte-identical zip bytes. The CI determinism guard runs
this on every PR; any regression fails the build.

Usage::

    python -m bench.determinism --store .cyberos/memory/store/

Exit codes:
  0 — bytes match
  1 — bytes differ; export is non-deterministic (regression)
"""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path


def main(argv: list[str] | None = None) -> int:
    from cyberos.core.export import export_zip

    ap = argparse.ArgumentParser()
    ap.add_argument("--store", required=True)
    args = ap.parse_args(argv)

    store = Path(args.store).resolve()
    if not store.is_dir():
        print(f"store not found: {store}", file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory(prefix="cyberos-det-") as td:
        td_path = Path(td)
        a = export_zip(store, td_path / "a.zip")
        b = export_zip(store, td_path / "b.zip")
        if a != b:
            print(f"FAIL: {a} != {b}")
            return 1
        print(f"OK sha256={a}")
        return 0


if __name__ == "__main__":
    sys.exit(main())
