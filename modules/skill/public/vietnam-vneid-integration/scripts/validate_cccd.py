"""Validate a Vietnamese CCCD (Căn cước công dân) — 12 digits.

Structure: PPP G YY NNNNNN
  PPP    province code (001-096)
  G      gender + century digit (0=M/20thC, 1=F/20thC, 2=M/21stC, ...)
  YY     last 2 digits of birth year
  NNNNNN sequence number

Reads stdin, writes JSON. Exit 0 if valid.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

PATTERN = re.compile(r"^\d{12}$")


def _load_provinces() -> dict[str, str]:
    here = Path(__file__).resolve().parent.parent / "assets" / "province-codes.json"
    return json.loads(here.read_text(encoding="utf-8"))


def _ordinal(n: int) -> str:
    """Return 1->1st, 2->2nd, 3->3rd, 4->4th, etc."""
    if 10 <= n % 100 <= 20:
        suffix = "th"
    else:
        suffix = {1: "st", 2: "nd", 3: "rd"}.get(n % 10, "th")
    return f"{n}{suffix}"


def classify(raw: str) -> dict:
    s = raw.strip()
    if not PATTERN.fullmatch(s):
        return {"ok": False, "reason": "CCCD must be exactly 12 digits"}

    provinces = _load_provinces()
    p_code = s[0:3]
    if p_code not in provinces:
        return {"ok": False, "reason": f"unknown province code: {p_code}"}

    g_digit = int(s[3])
    yy = int(s[4:6])
    seq = s[6:]

    # Century mapping per Luật Căn cước. Even digit -> male; odd -> female.
    # 0/1 -> 1900s, 2/3 -> 2000s, 4/5 -> 2100s, 6/7 -> 2200s, 8/9 -> 2300s.
    century_map = {
        0: ("M", 1900), 1: ("F", 1900),
        2: ("M", 2000), 3: ("F", 2000),
        4: ("M", 2100), 5: ("F", 2100),
        6: ("M", 2200), 7: ("F", 2200),
        8: ("M", 2300), 9: ("F", 2300),
    }
    gender, century_base = century_map[g_digit]
    yob = century_base + yy
    century_label = _ordinal(century_base // 100 + 1)

    return {
        "ok": True,
        "province": p_code,
        "province_name": provinces[p_code],
        "gender": gender,
        "century": century_label,
        "year_of_birth": yob,
        "sequence": seq,
    }


def main() -> int:
    result = classify(sys.stdin.read())
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
