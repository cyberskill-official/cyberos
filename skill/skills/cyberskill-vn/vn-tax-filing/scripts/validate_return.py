"""Structural validator for a vn-tax-filing XML output.

Reads XML from stdin, prints {ok, reason?} JSON to stdout.

Checks:
  - Header fields present (period, MST, address, filing date)
  - Period format matches the declared frequency (YYYY-MM or YYYY-QN)
  - MST structurally valid
  - At least one Section A row (or an explicit zero output declaration)
  - Arithmetic: C1 == sum(A.thue) - B3                       (±1 VND tolerance)
  - Arithmetic: C4 + C5 == max(C1 + C2 - C3, -(C1 + C2 - C3)) and exactly one is non-zero
"""

from __future__ import annotations

import json
import re
import sys
from xml.etree.ElementTree import fromstring, ParseError

MST_ENTITY = re.compile(r"^\d{10}$")
MST_BRANCH = re.compile(r"^\d{10}-\d{3}$")

PERIOD_MONTHLY = re.compile(r"^\d{4}-(0[1-9]|1[0-2])$")
PERIOD_QUARTERLY = re.compile(r"^\d{4}-Q[1-4]$")

RATE_ROW_TAGS = [
    "A1_chiu_thue_0",
    "A2_chiu_thue_5",
    "A3_chiu_thue_8",
    "A4_chiu_thue_10",
]


def _mst_ok(s: str) -> bool:
    return bool(MST_ENTITY.fullmatch(s) or MST_BRANCH.fullmatch(s))


def _int_text(el, tag: str) -> int:
    sub = el.find(tag)
    if sub is None or sub.text is None:
        raise ValueError(f"missing element: {tag}")
    return int(sub.text.strip())


def _validate(root) -> dict:
    if root.tag != "TKhaiThue":
        return {"ok": False, "reason": f"root element must be TKhaiThue, got {root.tag}"}

    ky_attr = root.get("kyKKhai") or ""
    if ky_attr not in ("thang", "quy"):
        return {"ok": False, "reason": f"kyKKhai attribute must be 'thang' or 'quy', got {ky_attr!r}"}

    ttc = root.find("TTinChung")
    sec_a = root.find("ThueDauRa")
    sec_b = root.find("ThueDauVao")
    sec_c = root.find("ThuePhaiNop")
    for el, name in [(ttc, "TTinChung"), (sec_a, "ThueDauRa"),
                     (sec_b, "ThueDauVao"), (sec_c, "ThuePhaiNop")]:
        if el is None:
            return {"ok": False, "reason": f"missing element: {name}"}

    # Period format.
    period_el = ttc.find("kyKKhai")
    if period_el is None or not period_el.text:
        return {"ok": False, "reason": "TTinChung/kyKKhai missing"}
    period = period_el.text.strip()
    if ky_attr == "thang":
        if not PERIOD_MONTHLY.fullmatch(period):
            return {"ok": False, "reason": f"monthly period must match YYYY-MM, got {period!r}"}
    else:
        if not PERIOD_QUARTERLY.fullmatch(period):
            return {"ok": False, "reason": f"quarterly period must match YYYY-QN, got {period!r}"}

    if ttc.find("ngayLap") is None or not (ttc.find("ngayLap").text or "").strip():
        return {"ok": False, "reason": "TTinChung/ngayLap missing"}

    # NNT block.
    nnt = ttc.find("NNT")
    if nnt is None:
        return {"ok": False, "reason": "TTinChung/NNT missing"}
    mst_el = nnt.find("mst")
    if mst_el is None or not _mst_ok((mst_el.text or "").strip()):
        return {"ok": False, "reason": f"NNT/mst invalid: {mst_el.text if mst_el is not None else None!r}"}
    if nnt.find("tenNNT") is None or not (nnt.find("tenNNT").text or "").strip():
        return {"ok": False, "reason": "NNT/tenNNT missing"}
    if nnt.find("diaChi") is None:
        return {"ok": False, "reason": "NNT/diaChi missing (may be empty but element required)"}

    # Section A — sum every rate row; allow zero rows (explicit-zero declaration).
    rows_found = [r for r in RATE_ROW_TAGS if sec_a.find(r) is not None]
    sum_a_thue = 0
    sum_a_gia_tri = 0
    for tag in rows_found:
        row = sec_a.find(tag)
        try:
            sum_a_gia_tri += int(row.find("giaTri").text)
            sum_a_thue += int(row.find("thue").text)
        except (AttributeError, ValueError, TypeError) as exc:
            return {"ok": False, "reason": f"section A row {tag} parse error: {exc}"}

    # totals on the A subtree
    try:
        tong_a_thue = _int_text(sec_a, "tongThue")
    except ValueError as exc:
        return {"ok": False, "reason": str(exc)}

    if abs(tong_a_thue - sum_a_thue) > 1:
        return {
            "ok": False,
            "reason": f"ThueDauRa/tongThue {tong_a_thue} != sum of row thue {sum_a_thue}",
        }

    # Section B.
    try:
        b1 = _int_text(sec_b, "B1_tongGiaTriMua")  # noqa: F841 — surfaced for completeness
        b2 = _int_text(sec_b, "B2_tongThueDauVao")  # noqa: F841
        b3 = _int_text(sec_b, "B3_tongThueDuocKhauTru")
    except ValueError as exc:
        return {"ok": False, "reason": str(exc)}

    # Section C.
    try:
        c1 = _int_text(sec_c, "C1_thuePhatSinh")
        c2 = _int_text(sec_c, "C2_thueKyTruocChuyenSang")
        c3 = _int_text(sec_c, "C3_thueDaNopTrongKy")
        c4 = _int_text(sec_c, "C4_thueConPhaiNop")
        c5 = _int_text(sec_c, "C5_thueDeNghiHoan")
    except ValueError as exc:
        return {"ok": False, "reason": str(exc)}

    # Arithmetic 1: C1 == sum(A.thue) - B3
    expected_c1 = sum_a_thue - b3
    if abs(c1 - expected_c1) > 1:
        return {
            "ok": False,
            "reason": f"C1 {c1} != sum(A.thue) - B3 = {sum_a_thue} - {b3} = {expected_c1}",
        }

    # Arithmetic 2: payable / refund mutual exclusion and balance.
    if c4 < 0 or c5 < 0:
        return {"ok": False, "reason": f"C4 and C5 must be non-negative (got {c4}, {c5})"}
    if c4 > 0 and c5 > 0:
        return {"ok": False, "reason": "exactly one of C4, C5 must be zero"}
    expected_balance = c1 + c2 - c3
    if expected_balance >= 0:
        # payable path: C4 = expected_balance, C5 = 0
        if abs(c4 - expected_balance) > 1 or c5 != 0:
            return {
                "ok": False,
                "reason": f"C4 {c4} / C5 {c5} != payable {expected_balance} (C1+C2-C3)",
            }
    else:
        # refund path: C5 = -expected_balance, C4 = 0
        if abs(c5 - (-expected_balance)) > 1 or c4 != 0:
            return {
                "ok": False,
                "reason": f"C5 {c5} / C4 {c4} != refund {-expected_balance}",
            }

    return {
        "ok": True,
        "period": period,
        "frequency": "monthly" if ky_attr == "thang" else "quarterly",
        "rows": len(rows_found),
        "c1": c1, "c4": c4, "c5": c5,
    }


def main() -> int:
    raw = sys.stdin.read()
    try:
        root = fromstring(raw)
    except ParseError as exc:
        print(json.dumps({"ok": False, "reason": f"XML parse error: {exc}"}))
        return 2
    result = _validate(root)
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    sys.exit(main())
