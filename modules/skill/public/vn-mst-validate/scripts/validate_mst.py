"""Validate a Vietnamese tax code (Mã số thuế / MST).

Per General Department of Taxation regulations (decree 126/2020/NĐ-CP + circulars):
  - 10 digits  → legal entity (cá nhân kinh doanh / công ty)
  - 10 digits + '-' + 3 digits  → branch / dependent unit (chi nhánh / đơn vị phụ thuộc)

Reads stdin (a single line), prints a JSON result to stdout, exits 0 if valid.

Usage:
    echo '0312345678' | python scripts/validate_mst.py
    echo '0312345678-001' | python scripts/validate_mst.py
"""

from __future__ import annotations

import json
import re
import sys

PATTERN_ENTITY = re.compile(r"^\d{10}$")
PATTERN_BRANCH = re.compile(r"^\d{10}-\d{3}$")


def classify(raw: str) -> dict:
    """Return a {ok, kind?, reason?} dict for one MST string."""
    s = raw.strip()
    if not s:
        return {"ok": False, "reason": "MST is empty"}
    if PATTERN_ENTITY.fullmatch(s):
        return {"ok": True, "kind": "entity"}
    if PATTERN_BRANCH.fullmatch(s):
        return {"ok": True, "kind": "branch"}
    return {
        "ok": False,
        "reason": "MST must be 10 digits, optionally followed by '-NNN' (e.g. 0312345678 or 0312345678-001)",
    }


def main() -> int:
    raw = sys.stdin.read()
    result = classify(raw)
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
