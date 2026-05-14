# Căn cước công dân (CCCD) — format reference

## What it is

The CCCD ("Căn cước công dân") is Vietnam's national citizen identity number, in use since 2016 and made mandatory under **Luật Căn cước công dân (2014)** and the implementing **Decree 137/2015/NĐ-CP**. The 2023 **Luật Căn cước** (Citizen ID Law) retained the same 12-digit numeric structure while renaming the physical document to "Thẻ căn cước".

It supersedes the older 9-digit **Chứng minh nhân dân (CMND)** which is being phased out (deadline extended several times; current target: end of 2025 for full retirement).

## Structure — 12 digits

```
PPP G YY NNNNNN
└┬┘ │ └┬┘ └──┬──┘
 │  │  │    └──── 6 digits — sequence number (random)
 │  │  └─────── 2 digits — last 2 of birth year
 │  └────────── 1 digit — gender + century encoding
 └──────────── 3 digits — province / city of registration
```

### Province code (digits 1–3)

3-digit code matching the General Statistics Office (GSO) province-code table. Range `001–096` with gaps for retired codes (provinces that were merged or renamed historically). See `province-codes.md` for the full list of 63 currently-active provinces / cities.

### Gender + century code (digit 4)

Single digit encoding both sex and century of birth:

| Digit | Gender | Century | Year range |
|-------|--------|---------|------------|
| 0     | Male   | 20th    | 1900–1999  |
| 1     | Female | 20th    | 1900–1999  |
| 2     | Male   | 21st    | 2000–2099  |
| 3     | Female | 21st    | 2000–2099  |
| 4     | Male   | 22nd    | 2100–2199  |
| 5     | Female | 22nd    | 2100–2199  |
| 6     | Male   | 23rd    | 2200–2299  |
| 7     | Female | 23rd    | 2200–2299  |
| 8     | Male   | 24th    | 2300–2399  |
| 9     | Female | 24th    | 2300–2399  |

Even digit = male, odd digit = female. The pair (0,1) covers the 1900s; each subsequent pair advances a century.

### Birth year (digits 5–6)

Last 2 digits of the year of birth. Combine with the century base from digit 4. Example: digit-4 = 1, digits 5–6 = "85" → female, born 1985.

### Sequence number (digits 7–12)

6 digits, assigned by Bộ Công An at registration. Not strictly random in any cryptographic sense, but functionally unguessable for an external party. Not checksummed.

## Worked example

`079185000001`

| Position | Digits | Meaning |
|----------|--------|---------|
| 1–3      | `079`  | Province → Hồ Chí Minh |
| 4        | `1`    | Female, 20th century |
| 5–6      | `85`   | Born 1985 |
| 7–12     | `000001` | Sequence |

## Validation rules in this skill

1. `^\d{12}$` — exactly 12 digits, no separators.
2. Province code MUST be in the active table (`province-codes.json`).
3. Gender+century digit 0–9 is always accepted structurally (all 10 values are valid encodings).

We do NOT validate the year-of-birth as "plausible" (e.g. not in the future) because the skill is structural-only — same posture as `vn-mst-validate`. A live registry lookup against the Cơ sở Dữ liệu Quốc gia về Dân cư (CSDLQGDC) via VNeID is the way to confirm an issued CCCD; that requires partner credentials.

## Live verification

Structural validation is the cheap first pass. For KYC pipelines, the next step is a `POST /api/v1/identity/verify` call against VNeID with `{cccd, full_name, dob}` — see `vneid-api.md`. The returned `match_score` (0.0–1.0) reflects similarity to the registry record; partners typically set a threshold ≥ 0.85 for auto-approval and 0.6–0.85 for human review.

## Legacy CMND notes

The 9-digit CMND has no embedded structure and no checksum — it's a flat sequence number. Do not run CMND values through this validator; use the legacy `vn-cmnd-validate` skill if/when it ships. The 12-digit CCCD strictly replaces it.
