"""Generate a Vietnamese VAT e-invoice XML from a JSON line-item list.

Schema target: GDT v3.0 (sketch — see references/gdt-xml-schema-v3.md).

Reads JSON from stdin, writes XML to stdout or to `--out <path>`.

Per-line VAT rounding (half-up to integer VND) is mandated by GDT; do
NOT round at the document level.
"""

from __future__ import annotations

import argparse
import decimal
import json
import sys
from decimal import Decimal, ROUND_HALF_UP
from xml.etree.ElementTree import Element, SubElement, tostring
from xml.dom import minidom


NSMAP = {
    "inv": "http://kekhaithue.gdt.gov.vn/TIN/2014/04/01/HoaDonDienTu",
    "ds":  "http://www.w3.org/2000/09/xmldsig#",
}


def _round_vnd(x: Decimal) -> int:
    """Round a Decimal to integer VND, half-up (per GDT)."""
    return int(x.quantize(Decimal("1"), rounding=ROUND_HALF_UP))


def _build_invoice(data: dict) -> Element:
    """Construct the invoice Element tree from the parsed JSON payload."""
    # Validate top-level shape early — fail loud.
    for required in ("seller", "buyer", "invoice_no", "invoice_date", "lines"):
        if required not in data:
            raise ValueError(f"missing required field: {required}")
    if not data["lines"]:
        raise ValueError("invoice must have at least one line item")

    inv = Element("Invoice", {"xmlns:inv": NSMAP["inv"], "xmlns:ds": NSMAP["ds"]})
    header = SubElement(inv, "Header")
    SubElement(header, "InvoiceNo").text = data["invoice_no"]
    SubElement(header, "InvoiceDate").text = data["invoice_date"]
    SubElement(header, "Currency").text = data.get("currency", "VND")
    SubElement(header, "PaymentMethod").text = data.get("payment_method", "TM/CK")

    seller = SubElement(inv, "Seller")
    SubElement(seller, "Name").text = data["seller"]["name"]
    SubElement(seller, "MST").text = data["seller"]["mst"]
    SubElement(seller, "Address").text = data["seller"].get("address", "")

    buyer = SubElement(inv, "Buyer")
    SubElement(buyer, "Name").text = data["buyer"]["name"]
    SubElement(buyer, "MST").text = data["buyer"]["mst"]
    SubElement(buyer, "Address").text = data["buyer"].get("address", "")

    lines = SubElement(inv, "Lines")
    total_amount = Decimal("0")
    total_vat = Decimal("0")
    for i, ln in enumerate(data["lines"], start=1):
        qty = Decimal(str(ln["qty"]))
        unit_price = Decimal(str(ln["unit_price"]))
        rate = Decimal(str(ln["vat_rate"]))
        thanh_tien = _round_vnd(qty * unit_price)
        tien_thue = _round_vnd(Decimal(thanh_tien) * rate)
        total_amount += Decimal(thanh_tien)
        total_vat += Decimal(tien_thue)

        line_el = SubElement(lines, "Line", {"index": str(i)})
        SubElement(line_el, "Name").text = ln["name"]
        SubElement(line_el, "Unit").text = ln.get("unit", "")
        SubElement(line_el, "Qty").text = str(qty)
        SubElement(line_el, "UnitPrice").text = str(_round_vnd(unit_price))
        SubElement(line_el, "Amount").text = str(thanh_tien)
        SubElement(line_el, "VatRate").text = f"{int(rate * 100)}%"
        SubElement(line_el, "VatAmount").text = str(tien_thue)

    totals = SubElement(inv, "Totals")
    SubElement(totals, "TotalAmount").text = str(int(total_amount))
    SubElement(totals, "TotalVat").text = str(int(total_vat))
    SubElement(totals, "GrandTotal").text = str(int(total_amount + total_vat))
    return inv


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
        inv = _build_invoice(data)
    except (ValueError, KeyError, decimal.InvalidOperation) as exc:
        print(json.dumps({"ok": False, "reason": str(exc)}), file=sys.stderr)
        return 2

    xml = _prettify(inv)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as fh:
            fh.write(xml)
    else:
        sys.stdout.write(xml)
    return 0


if __name__ == "__main__":
    sys.exit(main())
