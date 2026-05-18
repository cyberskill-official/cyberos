"""Structural validator for a vietnam-vat-invoice XML output.

Reads XML from stdin, prints {ok, reason?} JSON to stdout.

Checks:
  - Header fields present
  - Seller and Buyer MSTs structurally valid (delegates to vietnam-mst-validate logic)
  - At least one line item
  - Totals match sum of line amounts/VAT within +/-1 VND rounding tolerance
"""

from __future__ import annotations

import json
import re
import sys
from xml.etree.ElementTree import fromstring, ParseError

MST_ENTITY = re.compile(r"^\d{10}$")
MST_BRANCH = re.compile(r"^\d{10}-\d{3}$")


def _mst_ok(s: str) -> bool:
    return bool(MST_ENTITY.fullmatch(s) or MST_BRANCH.fullmatch(s))


def _validate(root) -> dict:
    header = root.find("Header")
    seller = root.find("Seller")
    buyer = root.find("Buyer")
    lines_el = root.find("Lines")
    totals = root.find("Totals")
    for el, name in [(header, "Header"), (seller, "Seller"), (buyer, "Buyer"),
                      (lines_el, "Lines"), (totals, "Totals")]:
        if el is None:
            return {"ok": False, "reason": f"missing element: {name}"}

    seller_mst = (seller.find("MST").text or "").strip()
    buyer_mst = (buyer.find("MST").text or "").strip()
    if not _mst_ok(seller_mst):
        return {"ok": False, "reason": f"seller MST invalid: {seller_mst}"}
    if not _mst_ok(buyer_mst):
        return {"ok": False, "reason": f"buyer MST invalid: {buyer_mst}"}

    line_amounts = []
    line_vats = []
    for line in lines_el.findall("Line"):
        try:
            line_amounts.append(int(line.find("Amount").text))
            line_vats.append(int(line.find("VatAmount").text))
        except (AttributeError, ValueError) as exc:
            return {"ok": False, "reason": f"line parse error: {exc}"}

    if not line_amounts:
        return {"ok": False, "reason": "no line items"}

    sum_amt = sum(line_amounts)
    sum_vat = sum(line_vats)
    total_amt = int(totals.find("TotalAmount").text)
    total_vat = int(totals.find("TotalVat").text)
    grand = int(totals.find("GrandTotal").text)

    if abs(sum_amt - total_amt) > 1:
        return {"ok": False, "reason": f"TotalAmount {total_amt} != sum of lines {sum_amt}"}
    if abs(sum_vat - total_vat) > 1:
        return {"ok": False, "reason": f"TotalVat {total_vat} != sum of line VAT {sum_vat}"}
    if abs(grand - (sum_amt + sum_vat)) > 2:
        return {"ok": False, "reason": f"GrandTotal {grand} != TotalAmount + TotalVat ({sum_amt + sum_vat})"}

    return {"ok": True, "lines": len(line_amounts), "total": grand}


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
