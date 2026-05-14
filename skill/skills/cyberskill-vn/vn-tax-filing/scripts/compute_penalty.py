"""Estimate the late-filing penalty band for a VAT return.

Reads JSON from stdin:
    {"period": "2026-05" or "2026-Q2",
     "filing_date": "YYYY-MM-DD",
     "filing_frequency": "monthly" | "quarterly" (default: monthly)}

Writes JSON to stdout:
    {"days_late": int,
     "penalty_band": "on-time" | "warning" | "2-5M" | "5-8M" | "8-15M" | "15-25M",
     "penalty_vnd_estimate": [low, high]  (0 / 0 when on-time or warning),
     "recommended_action": str}

Bands per Nghị định 125/2020/NĐ-CP Article 13. The Article 59 interest
charge (0.03% per day on the underpaid tax) is NOT modelled here — pair
this with the taxpayer's known liability to estimate total exposure.
"""

from __future__ import annotations

import datetime as dt
import json
import sys
from pathlib import Path

# Defer to compute_deadline.py for the deadline calculation. Import as a
# sibling so the same holiday roll-forward logic applies.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import compute_deadline  # noqa: E402


# (max_days_inclusive, band_code, vnd_low, vnd_high, recommended_action)
BANDS = [
    (5,  "warning", 0,          0,          "File immediately. Document mitigating circumstances; many warnings are issued without monetary penalty if a clean compliance history."),
    (30, "2-5M",    2_000_000,  5_000_000,  "File now. Consider voluntary disclosure to reduce the assessment; engage tax advisor if assessment exceeds 5M VND."),
    (60, "5-8M",    5_000_000,  8_000_000,  "File now. Engage tax advisor for a response strategy; prepare supporting documents for any mitigating-circumstance argument."),
    (90, "8-15M",   8_000_000,  15_000_000, "File now. Prepare a formal written explanation (giải trình) for submission with the return."),
]
ABOVE_90 = ("15-25M", 15_000_000, 25_000_000, "File now. Criminal-evasion investigation possible under Article 200 of the Penal Code if intent to evade is shown. Engage counsel before filing.")


def compute(data: dict) -> dict:
    if "period" not in data:
        raise ValueError("missing required field: period")
    if "filing_date" not in data:
        raise ValueError("missing required field: filing_date")

    freq = data.get("filing_frequency")
    if freq is None:
        # Infer from period shape — monthly is YYYY-MM, quarterly is YYYY-QN.
        freq = "quarterly" if "Q" in data["period"] else "monthly"

    try:
        filing_date = dt.date.fromisoformat(data["filing_date"])
    except ValueError as exc:
        raise ValueError(f"filing_date: {exc}") from exc

    deadline_info = compute_deadline.compute(
        {"period": data["period"], "filing_frequency": freq},
        today=filing_date,
    )
    deadline = dt.date.fromisoformat(deadline_info["deadline"])
    days_late = (filing_date - deadline).days

    if days_late <= 0:
        return {
            "period": data["period"],
            "filing_date": filing_date.isoformat(),
            "deadline": deadline.isoformat(),
            "days_late": days_late,
            "penalty_band": "on-time",
            "penalty_vnd_estimate": [0, 0],
            "recommended_action": "No penalty — return is on time.",
            "note": "Interest charges (Art. 59, 0.03%/day on unpaid tax) still apply to any underpayment.",
        }

    for max_days, band, lo, hi, action in BANDS:
        if days_late <= max_days:
            return {
                "period": data["period"],
                "filing_date": filing_date.isoformat(),
                "deadline": deadline.isoformat(),
                "days_late": days_late,
                "penalty_band": band,
                "penalty_vnd_estimate": [lo, hi],
                "recommended_action": action,
                "note": "Article 59 interest (0.03%/day on unpaid tax) accrues in addition to this administrative penalty.",
            }

    band, lo, hi, action = ABOVE_90
    return {
        "period": data["period"],
        "filing_date": filing_date.isoformat(),
        "deadline": deadline.isoformat(),
        "days_late": days_late,
        "penalty_band": band,
        "penalty_vnd_estimate": [lo, hi],
        "recommended_action": action,
        "note": "Article 59 interest (0.03%/day on unpaid tax) accrues in addition to this administrative penalty.",
    }


def main() -> int:
    raw = sys.stdin.read()
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        print(json.dumps({"ok": False, "reason": f"invalid JSON: {exc}"}))
        return 2
    try:
        result = compute(data)
    except ValueError as exc:
        print(json.dumps({"ok": False, "reason": str(exc)}))
        return 2
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
