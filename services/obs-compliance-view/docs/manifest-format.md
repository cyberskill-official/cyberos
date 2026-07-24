# Chain-of-custody manifest format (TASK-OBS-009)

Auditors receive a **manifest JSON** alongside the canonical **rows JSON** bytes exported from a compliance view. The manifest pins what was exported, the memory chain head at export time, and cryptographic proofs an auditor can verify offline.

## Manifest fields

| Field | Type | Description |
|-------|------|-------------|
| `export_id` | string | Unique export identifier (ULID-style string) |
| `tenant_id` | string | Tenant UUID |
| `regulation` | string | e.g. `EU AI Act`, `PDPL`, `SOC 2`, `ISO 27001` |
| `time_range_start` | string | RFC3339 UTC start of query window |
| `time_range_end` | string | RFC3339 UTC end of query window |
| `row_count` | u64 | Number of rows in the export |
| `chain_head_at_export` | string | Hex-encoded memory chain head at export time |
| `exporter_subject_id` | string | Auditor JWT `sub` |
| `exporter_email` | string | Auditor email (placeholder discipline applies) |
| `exported_at` | string | RFC3339 UTC timestamp of export |
| `sha256_of_rows` | string | Hex SHA-256 of the canonical rows bytes |
| `public_key_id` | string | Signing key version identifier |
| `state` | `Complete` \| `Incomplete` | `Incomplete` exports MUST NOT be trusted |
| `ed25519_signature` | string (base64) | Ed25519 signature over `signable_bytes` (optional until signed) |

## Signable bytes

The signature covers a JSON object with **sorted keys** containing every field above **except** `ed25519_signature`. The same manifest input always produces the same bytes (RFC 8785-style determinism via `BTreeMap` serialization in `manifest.rs`).

## Row hash

`sha256_of_rows` is the hex SHA-256 of the exact canonical rows file bytes passed to verification. Any change to row content changes the hash.

## Offline verification

```bash
verify_manifest \
  --manifest export_manifest.json \
  --rows export_rows.json \
  --pubkey <64-hex-char-ed25519-public-key>
```

Exit codes: `0` = PASS, `1` = FAIL (reason printed), `2` = usage error.

Verification checks, in order:

1. `state` is `Complete`
2. Recomputed row hash equals `sha256_of_rows`
3. `ed25519_signature` is present and valid base64 (64 bytes)
4. Ed25519 signature verifies over `signable_bytes` with the supplied public key

## Trust model (DEC-180, DEC-183)

The auditor does not need CyberOS infrastructure access. They need the manifest file, the rows file, and the published public key (out-of-band or operator-provided). Tampering with either file after signing fails verification.
