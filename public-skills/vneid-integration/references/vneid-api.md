# VNeID API — partner integration reference

VNeID is the national digital-identity platform operated by **Bộ Công An (Ministry of Public Security)**. Access is by partner agreement; this document specifies the request/response shapes that `build_request.py` and `parse_response.py` work with.

## Base

- **Base URL** — `https://api.vneid.gov.vn` (production); `https://sandbox.vneid.gov.vn` (sandbox).
- **Auth** — OAuth2 client-credentials grant; the bearer token rotates every 60 minutes.
- **Signing** — request bodies for write endpoints (`verify`, `esign`) must be signed with the partner's private key; signature goes in `X-Signature: <base64(RSA-SHA256)>`. The reference scripts emit the body — your host signs it before POSTing.

## Common headers

| Header | Value | Notes |
|---|---|---|
| `Content-Type` | `application/json` | mandatory for write endpoints |
| `Authorization` | `Bearer <token>` | OAuth2 bearer |
| `X-Partner-Id` | `<partner-id>` | issued at onboarding |
| `X-Trace-Id` | UUIDv4 | client-generated; surfaced in MoPS audit logs |
| `X-Signature` | base64 RSA-SHA256 | required on `verify` and `esign` |

## Endpoints

### POST `/api/v1/identity/verify`

Verify that a `(cccd, full_name, dob)` triple matches the registry record.

**Request body:**

```json
{
  "cccd": "079185000001",
  "full_name": "NGUYỄN VĂN A",
  "dob": "1985-03-15",
  "request_time": "2026-05-14T03:45:12.123456+00:00"
}
```

**Response (200):**

```json
{
  "session_id": "ses_abc123",
  "match_score": 0.94,
  "verified": true,
  "matched_fields": ["full_name", "dob", "province"]
}
```

**Response (rejection):**

```json
{ "code": "VNEID_404", "error": "CCCD not found in registry" }
```

### POST `/api/v1/identity/esign`

Request an e-signature session for a document. The user opens `session_url` on a device with the VNeID app installed and authorises the signature with biometric.

**Request body:**

```json
{
  "cccd": "079185000001",
  "document_url": "https://your-app.example/contracts/abc.pdf",
  "callback_url": "https://your-app.example/vneid/cb",
  "request_time": "2026-05-14T03:45:12.123456+00:00"
}
```

**Response (200):**

```json
{
  "session_id": "esign_xyz789",
  "session_url": "https://vneid.gov.vn/esign/xyz789",
  "expires_at": "2026-05-14T04:15:12.000+00:00"
}
```

Once the user signs, MoPS POSTs the signature artefacts to `callback_url`. The callback shape is not implemented by this skill — handle it in your host.

### GET `/api/v1/identity/profile/{cccd}`

Retrieve a verified subset of the registry profile for a CCCD. The caller MUST have the data subject's prior consent recorded on file per Decree 13/2023.

**Query:** `?fields=full_name,dob,address`

**Response (200):**

```json
{
  "cccd": "079185000001",
  "full_name": "NGUYỄN VĂN A",
  "dob": "1985-03-15",
  "address": "123 Lê Lợi, P. Bến Nghé, Q.1, TP.HCM",
  "mst": "0312345678"
}
```

`mst` is optional — included only if the citizen has linked a tax code to their VNeID profile.

## Rate limits + quotas

Sandbox: 30 req/min per partner. Production: negotiated, typically 600 req/min plus burst. Exceeding the quota returns HTTP 429 with `Retry-After`.

## Sandbox tips

The sandbox accepts a fixed set of CCCDs from the `001`, `079`, and `001` provinces (Hà Nội + Hồ Chí Minh) seeded with fictional citizens. Real CCCDs in sandbox return 404; conversely, sandbox CCCDs in production return 404.

## Compliance hooks

- Every `verify`/`esign` is logged to MoPS audit storage with the `X-Trace-Id` as the correlation key. Retain the trace-id on the partner side for at least 12 months.
- The `X-Partner-Id` is the legal entity responsible for the request — abuse routes back via the partner agreement.
- Decree 13/2023 (PDPD) requires a CBDTIA filing if any of the data crosses borders. See `vn-legal-compliance` skill.
