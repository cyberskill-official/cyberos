"""Parse a VNeID API response into a normalised shape.

Reads JSON from stdin (the raw response body). Writes a normalised
result envelope to stdout. Handles all three current intents — verify,
esign, profile — by sniffing the response shape.
"""

from __future__ import annotations

import json
import sys


def parse(resp: dict) -> dict:
    # Verify shape: match_score + verified flag.
    if "match_score" in resp:
        return {
            "ok": True,
            "intent": "verify",
            "verified": bool(resp.get("verified", False)),
            "match_score": float(resp["match_score"]),
            "session_id": resp.get("session_id"),
        }
    # Esign shape: session_url + expires_at.
    if "session_url" in resp:
        return {
            "ok": True,
            "intent": "esign",
            "session_url": resp["session_url"],
            "expires_at": resp.get("expires_at"),
            "session_id": resp.get("session_id"),
        }
    # Profile shape: full_name + dob (mandatory by spec).
    if "full_name" in resp or "dob" in resp:
        return {
            "ok": True,
            "intent": "profile",
            "full_name": resp.get("full_name"),
            "dob": resp.get("dob"),
            "address": resp.get("address"),
            "mst": resp.get("mst"),  # optional — tax code if linked
        }
    # Error shape.
    if "error" in resp or "code" in resp:
        return {
            "ok": False,
            "code": resp.get("code"),
            "error": resp.get("error") or resp.get("message"),
        }
    return {"ok": False, "reason": "unrecognised VNeID response shape"}


def main() -> int:
    try:
        resp = json.loads(sys.stdin.read())
    except json.JSONDecodeError as exc:
        print(json.dumps({"ok": False, "reason": f"invalid JSON: {exc}"}))
        return 2
    result = parse(resp)
    print(json.dumps(result, ensure_ascii=False))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    sys.exit(main())
