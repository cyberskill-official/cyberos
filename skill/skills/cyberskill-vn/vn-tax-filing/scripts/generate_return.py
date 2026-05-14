"""Generate a Vietnamese VAT return (Mẫu 01/GTGT) XML from a JSON payload.

Schema target: Thông tư 80/2021/TT-BTC, Appendix II, Mẫu 01/GTGT
(sketch — see references/tt80-2021-schema.md).

Reads JSON from stdin, writes XML to stdout or to `--out <path>`.

Computation rules:
  - Output VAT per rate row: thue = round_half_up(taxable * rate)
  - Section A total: sum of per-rate thue (NOT round(sum * average_rate))
  - C1 = sum(A.thue) - B3
  - C4 = max(0, C1 + C2 - C3)        # tax payable
  - C5 = max(0, C3 - (C1 + C2))      # refund request
  - Exactly one of {C4, C5} is non-zero.
"""

from __future__ import annotations

import argparse
import decimal
import json
import re
import sys
from decimal import Decimal, ROUND_HALF_UP
from xml.etree.ElementTree import Element, SubElement, tostring
from xml.dom import minidom


NSMAP = {
    "ret": "http://kekhaithue.gdt.gov.vn/TIN/2021/Mau01GTGT",
    "ds":  "http://www.w3.org/2000/09/xmldsig#",
}

MST_ENTITY = re.compile(r"^\d{10}$")
MST_BRANCH = re.compile(r"^\d{10}-\d{3}$")

PERIOD_MONTHLY = re.compile(r"^\d{4}-(0[1-9]|1[0-2])$")
PERIOD_QUARTERLY = re.compile(r"^\d{4}-Q[1-4]$")

# Valid VAT rates accepted on Section A rows. 0/5/8/10 percent.
ALLOWED_RATES = {0, 5, 8, 10}
RATE_ROW_TAG = {
    0:  "A1_chiu_thue_0",
    5:  "A2_chiu_thue_5",
    8:  "A3_chiu_thue_8",
    10: "A4_chiu_thue_10",
}


def _round_vnd(x: Decimal) -> int:
    """Round a Decimal to integer VND, half-up (per GDT)."""
    return int(x.quantize(Decimal("1"), rounding=ROUND_HALF_UP))


def _mst_ok(s: str) -> bool:
    return bool(MST_ENTITY.fullmatch(s) or MST_BRANCH.fullmatch(s))


def _period_ok(s: str, freq: str) -> bool:
    if freq == "monthly":
        return bool(PERIOD_MONTHLY.fullmatch(s))
    if freq == "quarterly":
        return bool(PERIOD_QUARTERLY.fullmatch(s))
    return False


def _build_return(data: dict) -> Element:
    """Construct the Mẫu 01/GTGT XML tree from the parsed JSON payload."""
    # Top-level required fields.
    for required in ("period", "filing_frequency", "taxpayer", "filing_date", "output_vat", "input_vat"):
        if required not in data:
            raise ValueError(f"missing required field: {required}")

    period = data["period"]
    freq = data["filing_frequency"]
    if freq not in ("monthly", "quarterly"):
        raise ValueError(f"filing_frequency must be 'monthly' or 'quarterly', got: {freq}")
    if not _period_ok(period, freq):
        raise ValueError(
            f"period {period!r} does not match filing_frequency {freq!r} "
            "(expected YYYY-MM for monthly or YYYY-QN for quarterly)"
        )

    tp = data["taxpayer"]
    for f in ("name", "mst"):
        if f not in tp or not tp[f]:
            raise ValueError(f"taxpayer.{f} is required")
    mst = tp["mst"].strip()
    if not _mst_ok(mst):
        raise ValueError(
            f"taxpayer.mst invalid: {mst!r} (expected 10 digits or 10-digit-NNN branch)"
        )

    out_lines = data["output_vat"]
    if not isinstance(out_lines, list):
        raise ValueError("output_vat must be an array of {rate, taxable} objects")

    # Build the document root.
    root = Element(
        "TKhaiThue",
        {
            "xmlns:ret": NSMAP["ret"],
            "xmlns:ds":  NSMAP["ds"],
            "mauTKhai":  "01/GTGT",
            "kyKKhai":   "thang" if freq == "monthly" else "quy",
        },
    )

    # Header (Thông tin chung).
    header = SubElement(root, "TTinChung")
    SubElement(header, "kyKKhai").text = period
    SubElement(header, "ngayLap").text = data["filing_date"]
    nnt = SubElement(header, "NNT")
    SubElement(nnt, "tenNNT").text = tp["name"]
    SubElement(nnt, "mst").text = mst
    SubElement(nnt, "diaChi").text = tp.get("address", "")

    # Section A — Output VAT (Thuế GTGT đầu ra).
    sec_a = SubElement(root, "ThueDauRa")
    total_a_thue = Decimal("0")
    total_a_gia_tri = Decimal("0")
    seen_rates = set()
    for ln in out_lines:
        if "rate" not in ln or "taxable" not in ln:
            raise ValueError("each output_vat row needs {rate, taxable}")
        rate_pct = int(ln["rate"])
        if rate_pct not in ALLOWED_RATES:
            raise ValueError(
                f"VAT rate {rate_pct} not allowed; must be one of {sorted(ALLOWED_RATES)}"
            )
        if rate_pct in seen_rates:
            raise ValueError(
                f"VAT rate {rate_pct}% appears more than once — combine before submission"
            )
        seen_rates.add(rate_pct)
        gia_tri = Decimal(str(ln["taxable"]))
        rate_dec = Decimal(rate_pct) / Decimal(100)
        thue = _round_vnd(gia_tri * rate_dec)
        total_a_gia_tri += gia_tri
        total_a_thue += Decimal(thue)

        row = SubElement(sec_a, RATE_ROW_TAG[rate_pct])
        SubElement(row, "giaTri").text = str(_round_vnd(gia_tri))
        SubElement(row, "thue").text = str(thue)

    SubElement(sec_a, "tongGiaTri").text = str(_round_vnd(total_a_gia_tri))
    SubElement(sec_a, "tongThue").text = str(int(total_a_thue))

    # Section B — Input VAT (Thuế GTGT đầu vào được khấu trừ).
    inp = data["input_vat"]
    b1 = Decimal(str(inp.get("total_purchases", 0)))
    b2 = Decimal(str(inp.get("deductible", inp.get("total_deductible", 0))))
    # B3: deductible allowed this period — defaults to B2 unless overridden.
    b3 = Decimal(str(inp.get("allowable_deduction", b2)))

    sec_b = SubElement(root, "ThueDauVao")
    SubElement(sec_b, "B1_tongGiaTriMua").text = str(_round_vnd(b1))
    SubElement(sec_b, "B2_tongThueDauVao").text = str(_round_vnd(b2))
    SubElement(sec_b, "B3_tongThueDuocKhauTru").text = str(_round_vnd(b3))

    # Section C — Net payable / refundable.
    c1 = total_a_thue - b3
    c2 = Decimal(str(data.get("previous_carry_over", 0)))
    c3 = Decimal(str(data.get("paid_this_period", 0)))
    net = c1 + c2 - c3
    if net >= 0:
        c4 = net
        c5 = Decimal("0")
    else:
        c4 = Decimal("0")
        c5 = -net

    sec_c = SubElement(root, "ThuePhaiNop")
    SubElement(sec_c, "C1_thuePhatSinh").text = str(_round_vnd(c1))
    SubElement(sec_c, "C2_thueKyTruocChuyenSang").text = str(_round_vnd(c2))
    SubElement(sec_c, "C3_thueDaNopTrongKy").text = str(_round_vnd(c3))
    SubElement(sec_c, "C4_thueConPhaiNop").text = str(_round_vnd(c4))
    SubElement(sec_c, "C5_thueDeNghiHoan").text = str(_round_vnd(c5))

    return root


def _prettify(elem: Element) -> str:
    raw = tostring(elem, encoding="utf-8", xml_declaration=True)
    return minidom.parseString(raw).toprettyxml(indent="  ", encoding="utf-8").decode("utf-8")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=None, help="output path (default: stdout)")
    args = ap.parse_args()

    try:
        data = json.loads(sys.stdin.read())
    except json.JSONDecodeError as exc:
        print(json.dumps({"ok": False, "reason": f"invalid JSON: {exc}"}), file=sys.stderr)
        return 2

    try:
        root = _build_return(data)
    except (ValueError, KeyError, decimal.InvalidOperation) as exc:
        print(json.dumps({"ok": False, "reason": str(exc)}), file=sys.stderr)
        return 2

    xml = _prettify(root)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as fh:
            fh.write(xml)
    else:
        sys.stdout.write(xml)
    return 0


if __name__ == "__main__":
    sys.exit(main())
