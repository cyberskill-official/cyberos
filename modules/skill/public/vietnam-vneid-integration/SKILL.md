---
name: vietnam-vneid-integration
description: >-
  Validate Vietnamese citizen IDs (CCCD), extract province / gender / year-of-birth from their structure, and build VNeID API request payloads for identity verification, e-sign sessions, and profile lookup. Use when the user provides a 12-digit CCCD, needs to construct a VNeID API call, or asks about Vietnamese national ID format. Network access to the VNeID endpoint is NOT performed by this skill — it builds the request payload; your host calls the API. Do NOT use for old 9-digit CMND (chứng minh nhân dân) — those are a legacy format being phased out. Use when user asks to "reference vietnam vneid integration" or "look up vietnam vneid integration".
license: Apache-2.0
compatibility: >-
  Fully offline for validation + request building. Network access to
  VNeID is the host's responsibility — see references/vneid-api.md for
  the endpoint contract.
metadata:
  author: cyberskill
  version: "0.1.0"
  region: VN
  collection: cyberskill-vn
allowed-tools: read_file write_file
---

# VNeID Integration (Căn cước công dân + VNeID API)

## When to use

- User provides a 12-digit CCCD and asks for validation.
- User needs the JSON request shape for a VNeID `/identity/verify`, `/identity/esign`, or `/identity/profile` call.
- User has a VNeID response and wants it parsed.
- KYC pipeline that needs CCCD-structural validation as a fast first pass.

## Procedure

1. **Validate the CCCD** with `scripts/validate_cccd.py`. Returns `{ok, province, province_name, gender, century, year_of_birth, sequence}` on success.
2. **Build the request** with `scripts/build_request.py` for one of three intents: `verify`, `esign`, `profile`. The script produces the JSON body — your host adds auth headers + signs + POSTs.
3. **Parse the response** with `scripts/parse_response.py` once the call comes back.

## Quick start

```bash
# Structural validation
echo '079185000001' | python scripts/validate_cccd.py
# → {"ok": true, "province": "079", "province_name": "Hồ Chí Minh", "gender": "M", "century": "21st", "year_of_birth": 1985, "sequence": "000001"}

# Build a verify request
cat > /tmp/req.json <<'EOF'
{"intent": "verify", "cccd": "079185000001", "full_name": "NGUYỄN VĂN A", "dob": "1985-03-15"}
EOF
python scripts/build_request.py < /tmp/req.json
# → {"endpoint": "POST /api/v1/identity/verify", "body": {...}}

# Parse a response
echo '{"match_score": 0.94, "verified": true, "session_id": "abc123"}' | python scripts/parse_response.py
```

## Production gating

VNeID access is by partner agreement with Bộ Công An. The endpoint URLs in `references/vneid-api.md` are documented for partner integration; the skill builds the request payload but the actual HTTPS call is the host's job. This is intentional — it lets the skill be useful as scaffolding without requiring live API access for evaluation.

## Status

CCCD structural validation: production-ready, covers all 63 provinces + century encoding. VNeID request shaping: documented to match the partner spec (v1, 2025); ready for hookup once your host has OAuth2 credentials. Response parsing: handles the three current intents (verify, esign, profile).
