"""Generate a VietQR / Napas247 payload string from a JSON request.

Reads JSON from stdin, prints the VietQR payload to stdout.

Input shape:
    {
        "bank": "VCB" | "BIDV" | ...,        # short code; resolved to BIN via assets/bank-bins.json
        "account": "0123456789",              # 6-19 digits
        "recipient": "NGUYEN VAN A",          # optional, <=25 chars ASCII recommended
        "amount": 250000,                     # optional, integer VND
        "memo": "Thanh toan ...",             # optional, <=25 chars ASCII recommended
        "dynamic": true                       # optional; static if false/omitted
    }
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def _tlv(tag: str, value: str) -> str:
    """Tag-Length-Value triplet. Length is 2-digit zero-padded decimal."""
    return f"{tag}{len(value):02d}{value}"


def _crc16_ccitt_false(data: bytes) -> str:
    """CRC16-CCITT-FALSE (polynomial 0x1021, init 0xFFFF, no xor-out)."""
    crc = 0xFFFF
    for byte in data:
        crc ^= byte << 8
        for _ in range(8):
            if crc & 0x8000:
                crc = (crc << 1) ^ 0x1021
            else:
                crc <<= 1
            crc &= 0xFFFF
    return f"{crc:04X}"


def _load_bank_bins() -> dict[str, str]:
    here = Path(__file__).resolve().parent.parent / "assets" / "bank-bins.json"
    return json.loads(here.read_text(encoding="utf-8"))


def build(req: dict) -> str:
    bins = _load_bank_bins()
    short = req.get("bank")
    if not short or short not in bins:
        raise ValueError(f"unknown bank short code: {short!r}. See assets/bank-bins.json")
    bin_code = bins[short]

    account = str(req.get("account", "")).strip()
    if not account.isdigit() or not (6 <= len(account) <= 19):
        raise ValueError("account must be 6-19 digits")

    # Build inner merchant account info (tag 38).
    benef_info = _tlv("00", bin_code) + _tlv("01", account)
    merchant_account = (
        _tlv("00", "A000000727")     # AID Napas
        + _tlv("01", benef_info)
        + _tlv("02", "QRIBFTTA")     # service code: inter-bank account transfer
    )

    # Top-level fields.
    payload = ""
    payload += _tlv("00", "01")  # Payload Format Indicator
    payload += _tlv("01", "12" if req.get("dynamic") else "11")  # Point of Init
    payload += _tlv("38", merchant_account)
    payload += _tlv("53", "704")
    if req.get("amount") is not None:
        amt = str(int(req["amount"]))
        payload += _tlv("54", amt)
    payload += _tlv("58", "VN")
    if req.get("recipient"):
        payload += _tlv("59", str(req["recipient"])[:25])
    if req.get("memo"):
        # Additional data: terminal label (sub-tag 08).
        add = _tlv("08", str(req["memo"])[:25])
        payload += _tlv("62", add)

    # CRC: append tag "63" + length "04", then compute over everything so far.
    payload += "6304"
    crc = _crc16_ccitt_false(payload.encode("ascii"))
    payload += crc
    return payload


def main() -> int:
    try:
        req = json.loads(sys.stdin.read())
    except json.JSONDecodeError as exc:
        print(f"invalid JSON: {exc}", file=sys.stderr)
        return 2
    try:
        out = build(req)
    except (ValueError, KeyError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    sys.stdout.write(out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
