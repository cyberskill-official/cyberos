"""Compute the Vietnamese VAT filing deadline for a given period.

Reads JSON from stdin:
    {"period": "2026-05" or "2026-Q2", "filing_frequency": "monthly" or "quarterly"}

Writes JSON to stdout:
    {"deadline": "YYYY-MM-DD",
     "days_remaining": int,
     "is_overdue": bool,
     "rolled_forward": bool}

Rules (per Thông tư 80/2021/TT-BTC + Law on Tax Administration 38/2019/QH14):
  - Monthly:   20th day of the following month
  - Quarterly: last day of the first month of the following quarter
  - If the statutory date is a weekend (Sat/Sun) or VN public holiday,
    it rolls forward to the next working day (Article 86).
"""

from __future__ import annotations

import datetime as dt
import json
import os
import re
import sys
from pathlib import Path

PERIOD_MONTHLY = re.compile(r"^(\d{4})-(0[1-9]|1[0-2])$")
PERIOD_QUARTERLY = re.compile(r"^(\d{4})-Q([1-4])$")

QUARTER_LAST_MONTH = {1: 3, 2: 6, 3: 9, 4: 12}
QUARTER_FOLLOWING_MONTH = {1: 4, 2: 7, 3: 10, 4: 1}


def _load_holidays() -> set[dt.date]:
    """Load bundled VN public holidays. Resolves relative to this script."""
    here = Path(__file__).resolve().parent
    candidates = [
        here.parent / "assets" / "holidays.json",
        here / "holidays.json",
    ]
    env_path = os.environ.get("VN_HOLIDAYS_JSON")
    if env_path:
        candidates.insert(0, Path(env_path))
    for p in candidates:
        if p.is_file():
            raw = json.loads(p.read_text(encoding="utf-8"))
            out: set[dt.date] = set()
            for k, v in raw.items():
                if k.startswith("_"):
                    continue
                for s in v:
                    out.add(dt.date.fromisoformat(s))
            return out
    return set()


def _roll_forward(d: dt.date, holidays: set[dt.date]) -> tuple[dt.date, bool]:
    rolled = False
    while d.weekday() >= 5 or d in holidays:
        d = d + dt.timedelta(days=1)
        rolled = True
    return d, rolled


def _statutory_deadline(period: str, frequency: str) -> dt.date:
    if frequency == "monthly":
        m = PERIOD_MONTHLY.fullmatch(period)
        if not m:
            raise ValueError(f"monthly period must match YYYY-MM, got {period!r}")
        year = int(m.group(1))
        month = int(m.group(2))
        # 20th of the following month.
        if month == 12:
            return dt.date(year + 1, 1, 20)
        return dt.date(year, month + 1, 20)
    if frequency == "quarterly":
        m = PERIOD_QUARTERLY.fullmatch(period)
        if not m:
            raise ValueError(f"quarterly period must match YYYY-QN, got {period!r}")
        year = int(m.group(1))
        q = int(m.group(2))
        # Last day of the first month of the following quarter.
        following_m = QUARTER_FOLLOWING_MONTH[q]
        following_y = year + 1 if q == 4 else year
        # Last day of the *first* month of the following quarter — which is
        # itself the month numbered `following_m`. The "last day" wording in
        # TT80 means the last calendar day of that month.
        if following_m == 12:
            return dt.date(following_y, 12, 31)
        first_of_next = dt.date(following_y, following_m + 1, 1)
        return first_of_next - dt.timedelta(days=1)
    raise ValueError(f"filing_frequency must be 'monthly' or 'quarterly', got {frequency!r}")


def compute(data: dict, today: dt.date | None = None) -> dict:
    period = data.get("period")
    freq = data.get("filing_frequency", "monthly")
    if not period:
        raise ValueError("missing required field: period")

    base = _statutory_deadline(period, freq)
    holidays = _load_holidays()
    rolled, rolled_forward = _roll_forward(base, holidays)

    today = today or dt.date.today()
    days_remaining = (rolled - today).days
    return {
        "period": period,
        "frequency": freq,
        "statutory_deadline": base.isoformat(),
        "deadline": rolled.isoformat(),
        "rolled_forward": rolled_forward,
        "days_remaining": days_remaining,
        "is_overdue": days_remaining < 0,
    }


def main() -> int:
    raw = sys.stdin.read()
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        print(json.dumps({"ok": False, "reason": f"invalid JSON: {exc}"}))
        return 2
    today = None
    if "as_of" in data:
        try:
            today = dt.date.fromisoformat(data["as_of"])
        except ValueError as exc:
            print(json.dumps({"ok": False, "reason": f"as_of: {exc}"}))
            return 2
    try:
        result = compute(data, today=today)
    except ValueError as exc:
        print(json.dumps({"ok": False, "reason": str(exc)}))
        return 2
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
