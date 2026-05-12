#!/usr/bin/env python3
"""
canonical_sha — compute the canonical SHA-256 of an AGENTS.md per §0.5.

Canonical form (per AGENTS.md §0.5):
  - NFC Unicode normalisation
  - BOM stripped (start AND mid-file)
  - \\r\\n and lone \\r collapsed to \\n
  - trailing whitespace trimmed per line
  - trailing empty lines removed
  - single terminating \\n appended

Usage:
    python3 canonical_sha.py docs/memory/AGENTS.md
    python3 canonical_sha.py docs/memory/AGENTS.md --dump-canonical /tmp/canon.md
"""

from __future__ import annotations

import argparse
import hashlib
import sys
import unicodedata
from pathlib import Path


def canonicalise(text: str) -> bytes:
    text = unicodedata.normalize("NFC", text)
    text = text.replace("﻿", "")  # BOM anywhere
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    lines = [line.rstrip() for line in text.split("\n")]
    while lines and lines[-1] == "":
        lines.pop()
    return ("\n".join(lines) + "\n").encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(prog="canonical_sha")
    parser.add_argument("path")
    parser.add_argument("--dump-canonical",
                        help="Write the canonical bytes to this path")
    args = parser.parse_args()

    raw = Path(args.path).read_text(encoding="utf-8")
    canon = canonicalise(raw)
    digest = hashlib.sha256(canon).hexdigest()

    if args.dump_canonical:
        Path(args.dump_canonical).write_bytes(canon)

    print(f"sha256:{digest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
