"""Parse a VietQR / Napas247 payload string into structured JSON.

Reads payload from stdin, prints JSON to stdout. Verifies CRC16.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def _crc16_ccitt_false(data: bytes) -> str:
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


def _walk_tlv(s: str) -> list[tuple[str, str]]:
    out, i = [], 0
    while i < len(s):
        if i + 4 > len(s):
            raise ValueError(f"truncated TLV at offset {i}")
        tag = s[i:i+2]
        length = int(s[i+2:i+4])
        value = s[i+4:i+4+length]
        if len(value) != length:
            raise ValueError(f"TLV length mismatch for tag {tag}")
        out.append((tag, value))
        i += 4 + length
    return out


def parse(payload: str) -> dict:
    if len(payload) < 8 or payload[-8:-4] != "6304":
        raise ValueError("missing CRC16 trailer")
    body = payload[:-4]
    expected = _crc16_ccitt_false(body.encode("ascii"))
    actual = payload[-4:].upper()
    if expected != actual:
        raise ValueError(f"CRC mismatch (expected {expected}, got {actual})")

    fields = _walk_tlv(body[:-4])  # strip the literal '6304' we already validated
    out: dict = {"crc_ok": True}
    for tag, value in fields:
        if tag == "00":
            out["payload_format"] = value
        elif tag == "01":
            out["dynamic"] = (value == "12")
        elif tag == "38":
            sub = dict(_walk_tlv(value))
            out["napas_aid"] = sub.get("00")
            if "01" in sub:
                bb = dict(_walk_tlv(sub["01"]))
                out["bank_bin"] = bb.get("00")
                out["account"] = bb.get("01")
            out["service_code"] = sub.get("02")
        elif tag == "53":
            out["currency"] = value
        elif tag == "54":
            out["amount"] = int(value)
        elif tag == "58":
            out["country"] = value
        elif tag == "59":
            out["recipient"] = value
        elif tag == "60":
            out["city"] = value
        elif tag == "62":
            sub = dict(_walk_tlv(value))
            out["memo"] = sub.get("08")

    # Reverse-lookup the bank short code.
    bins_path = Path(__file__).resolve().parent.parent / "assets" / "bank-bins.json"
    bins = json.loads(bins_path.read_text(encoding="utf-8"))
    inverse = {v: k for k, v in bins.items()}
    out["bank"] = inverse.get(out.get("bank_bin", ""), None)
    return out


def main() -> int:
    payload = sys.stdin.read().strip()
    try:
        result = parse(payload)
    except ValueError as exc:
        print(json.dumps({"ok": False, "reason": str(exc)}))
        return 2
    print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
