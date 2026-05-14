"""Build the JSON request body for a VNeID API call.

Reads JSON from stdin:
    {
      "intent": "verify" | "esign" | "profile",
      "cccd": "079185000001",
      "full_name": "NGUYỄN VĂN A",     # required for verify
      "dob": "1985-03-15",              # required for verify
      "document_url": "https://...",    # required for esign
      "callback_url": "https://...",    # required for esign
      "fields": ["full_name", "dob"]    # optional for profile (default: all)
    }

Writes the partner-integration envelope to stdout:
    {
      "endpoint": "POST /api/v1/identity/verify",
      "headers": { ... },
      "body":    { ... }
    }

The actual HTTPS POST is the host's responsibility. This script only shapes
the payload + documents which headers / signing artefacts the host must add.
"""

from __future__ import annotations

import json
import re
import sys
import uuid
from datetime import datetime, timezone

CCCD = re.compile(r"^\d{12}$")
DOB = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def _common_headers() -> dict:
    return {
        "Content-Type": "application/json",
        "Authorization": "Bearer <OAUTH2_TOKEN>",
        "X-Partner-Id": "<PARTNER_ID>",
        "X-Trace-Id": str(uuid.uuid4()),
    }


def build(req: dict) -> dict:
    intent = req.get("intent")
    cccd = req.get("cccd", "")
    if not CCCD.fullmatch(cccd):
        raise ValueError("cccd must be 12 digits")

    if intent == "verify":
        full_name = req.get("full_name")
        dob = req.get("dob")
        if not full_name:
            raise ValueError("full_name is required for verify")
        if not dob or not DOB.fullmatch(dob):
            raise ValueError("dob (YYYY-MM-DD) is required for verify")
        return {
            "endpoint": "POST /api/v1/identity/verify",
            "headers": _common_headers(),
            "body": {
                "cccd": cccd,
                "full_name": full_name,
                "dob": dob,
                "request_time": datetime.now(timezone.utc).isoformat(),
            },
        }

    if intent == "esign":
        doc = req.get("document_url")
        cb = req.get("callback_url")
        if not doc:
            raise ValueError("document_url is required for esign")
        if not cb:
            raise ValueError("callback_url is required for esign")
        return {
            "endpoint": "POST /api/v1/identity/esign",
            "headers": _common_headers(),
            "body": {
                "cccd": cccd,
                "document_url": doc,
                "callback_url": cb,
                "request_time": datetime.now(timezone.utc).isoformat(),
            },
        }

    if intent == "profile":
        fields = req.get("fields") or ["full_name", "dob", "address"]
        return {
            "endpoint": f"GET /api/v1/identity/profile/{cccd}",
            "headers": _common_headers(),
            "query": {"fields": ",".join(fields)},
        }

    raise ValueError(f"unknown intent: {intent!r}; expected verify | esign | profile")


def main() -> int:
    try:
        req = json.loads(sys.stdin.read())
    except json.JSONDecodeError as exc:
        print(json.dumps({"ok": False, "reason": f"invalid JSON: {exc}"}))
        return 2
    try:
        out = build(req)
    except ValueError as exc:
        print(json.dumps({"ok": False, "reason": str(exc)}))
        return 2
    print(json.dumps(out, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
