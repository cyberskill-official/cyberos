---
id: TASK-OBS-009
title: "OBS chain-of-custody manifest: Ed25519 sign + verify CLI"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: obs
priority: p0
status: done
entered_via: rework
routed_back_count: 1
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-OBS-008, TASK-AUTH-006]
depends_on: [TASK-OBS-008]
blocks: []

source_pages:
  - website/docs/modules/obs.html#chain-of-custody

source_decisions:
  - DEC-180 2026-05-15 — Ed25519 over manifest signable bytes + SHA-256 of canonical rows; auditor verifies offline
  - DEC-181 2026-05-15 — Manifest accompanies exports; PDF cover deferred; JSON sidecar is the slice-3 path
  - DEC-182 2026-05-15 — Interrupted export MUST carry ExportState::Incomplete and fail verification
  - DEC-183 2026-05-15 — Standalone verify_manifest binary for offline auditor verification

language: rust 1.81
service: cyberos/services/obs-compliance-view/
new_files:
  - services/obs-compliance-view/src/manifest.rs
  - services/obs-compliance-view/src/manifest_signing.rs
  - services/obs-compliance-view/src/bin/verify_manifest.rs
  - services/obs-compliance-view/docs/manifest-format.md

modified_files:
  - services/obs-compliance-view/src/lib.rs
  - services/obs-compliance-view/Cargo.toml

allowed_tools:
  - file_read: services/obs-compliance-view/**
  - file_write: services/obs-compliance-view/{src,docs}/**
  - bash: cd services && cargo test -p cyberos-obs-compliance-view manifest_signing

disallowed_tools:
  - export compliance data without a signed manifest when manifest path is enabled
  - silently truncate exports without ExportState::Incomplete
  - sign with anything other than Ed25519
  - claim manifest_pdf.rs or per-view export wiring as shipped this batch

effort_hours: 8
subtasks:
  - "0.5h: manifest.rs — Manifest struct + signable_bytes + row hash"
  - "1.0h: manifest_signing.rs — Ed25519 sign + offline verify verdict"
  - "0.5h: bin/verify_manifest.rs — auditor CLI (--manifest --rows --pubkey)"
  - "0.5h: docs/manifest-format.md — field reference for auditors"
  - "0.5h: batch/9b-obs adopt re-spec + audit"

risk_if_skipped: "Without a signed chain-of-custody manifest, compliance exports are unverifiable JSON — auditors cannot independently prove integrity or detect tampering mid-flight (DEC-180, DEC-183)."
---

# TASK-OBS-009: Chain-of-custody manifest — Ed25519 sign + offline verifier

## Summary

Every compliance export MUST carry a chain-of-custody `Manifest` signed with Ed25519 over deterministic `signable_bytes`, plus a SHA-256 of the canonical row bytes. As-built code lives in `manifest.rs`, `manifest_signing.rs`, and the standalone `bin/verify_manifest.rs` CLI. This batch adds `services/obs-compliance-view/docs/manifest-format.md` as the auditor-facing field reference.

## Problem

The prior engineering-spec claimed `manifest_pdf.rs`, per-view export hooks under `views/{eu_ai_act,...}.rs`, CDN auto-fetch in the verifier, zip sidecars, and standalone `tests/manifest_*` files. The live crate ships pure manifest types + signing + a hex-pubkey CLI with inline tests in `manifest_signing.rs`. `docs/manifest-format.md` was never authored. FM-004 blocked re-entry (`task@1` + `## §N`).

## Proposed Solution

Adopt the as-built manifest stack:

- `manifest.rs` — `Manifest`, `ExportState`, `sha256_of_rows`, `signable_bytes` (sorted-key JSON, signature excluded)
- `manifest_signing.rs` — `sign` (base64 Ed25519) and `verify` → `Verdict::Pass | Fail(reason)`
- `bin/verify_manifest.rs` — offline CLI: `--manifest`, `--rows`, `--pubkey` (64 hex chars); exit 0 PASS / 1 FAIL
- `docs/manifest-format.md` — documents every manifest field and verification steps for auditors

## Alternatives Considered

- **PDF cover page (`manifest_pdf.rs`) in slice 3.** Rejected: JSON manifest + CLI verifier ship first; PDF deferred.
- **Wire manifest into per-view axum handlers this batch.** Rejected: phantom `views/eu_ai_act.rs` paths never existed; export integration is a follow-on slice.
- **CDN public-key fetch in verify_manifest.** Rejected: slice ships `--pubkey` hex for fully offline verification; CDN fetch deferred.

## Success Metrics

- Primary: sign → verify PASS on canonical rows; tampered rows or manifest fields FAIL closed; `Incomplete` and unsigned manifests FAIL.
- Guardrail: `cargo test -p cyberos-obs-compliance-view manifest_signing::` green.

## Scope

In scope (as-built + this batch doc):

- `services/obs-compliance-view/src/manifest.rs`
- `services/obs-compliance-view/src/manifest_signing.rs`
- `services/obs-compliance-view/src/bin/verify_manifest.rs`
- `services/obs-compliance-view/docs/manifest-format.md` (authored this batch)

### Out of scope / Non-Goals

- `manifest_pdf.rs` / PDF cover page + QR code
- Phantom per-view export paths (`views/{eu_ai_act,pdpl,soc2,iso27001}.rs`, `export/{pdf,json}.rs`)
- Standalone `tests/manifest_*` integration files (inline `#[cfg(test)]` in `manifest.rs` / `manifest_signing.rs` instead)
- CDN auto-fetch of public keys in `verify_manifest` (requires `--pubkey` hex offline)
- `obs.export_compliance` memory-row emit from HTTP export handlers (deferred with export wiring)

## Dependencies

`depends_on: [TASK-OBS-008]` (compliance view crate and row shape). Soft: TASK-AUTH-006 infra signing-key rotation; manifest-format doc references DEC-180 field list.

## 1. Description (normative)

- 1.1 `manifest.rs` MUST define `Manifest` with export metadata fields (`export_id`, `tenant_id`, `regulation`, time range, `row_count`, `chain_head_at_export`, exporter ids, `sha256_of_rows`, `public_key_id`, `state`, optional `ed25519_signature`).
- 1.2 `Manifest::signable_bytes` MUST serialize every field except the signature as sorted-key JSON so signatures are reproducible (DEC-180).
- 1.3 `Manifest::sha256_of_rows` MUST hash the canonical row bytes deterministically; verifiers MUST compare recomputed hash to `sha256_of_rows`.
- 1.4 `manifest_signing.rs` MUST Ed25519-sign `signable_bytes` and store base64 `ed25519_signature` on the manifest.
- 1.5 `manifest_signing::verify` MUST return `Verdict::Pass` only when `state == Complete`, rows hash matches, signature is present, and Ed25519 verifies (DEC-183).
- 1.6 `ExportState::Incomplete` MUST fail verification even if the signature bytes are well-formed (DEC-182).
- 1.7 `bin/verify_manifest.rs` MUST verify offline given `--manifest`, `--rows`, and `--pubkey` (64 hex chars); it MUST exit 0 on PASS and 1 on FAIL.
- 1.8 `docs/manifest-format.md` MUST document the manifest field list and offline verification procedure for auditors.
- 1.9 This adopt MUST NOT claim `manifest_pdf.rs` or per-view export handler wiring as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.4,#1.5) - sign then offline verify passes - test: `services/obs-compliance-view/src/manifest_signing.rs::sign_then_verify_passes`
- [ ] AC 2 (traces_to: #1.3,#1.5) - tampered rows fail on hash mismatch - test: `services/obs-compliance-view/src/manifest_signing.rs::tampered_rows_fail_on_hash`
- [ ] AC 3 (traces_to: #1.5) - tampered manifest field fails on signature - test: `services/obs-compliance-view/src/manifest_signing.rs::a_tampered_field_fails_on_signature`
- [ ] AC 4 (traces_to: #1.6) - incomplete export fails closed - test: `services/obs-compliance-view/src/manifest_signing.rs::incomplete_export_fails_closed`
- [ ] AC 5 (traces_to: #1.5) - unsigned manifest fails - test: `services/obs-compliance-view/src/manifest_signing.rs::an_unsigned_manifest_fails`
- [ ] AC 6 (traces_to: #1.1,#1.2) - Manifest fields + signable bytes exclude signature - test: `services/obs-compliance-view/src/manifest.rs::signable_bytes_are_deterministic_and_exclude_the_signature`
- [ ] AC 7 (traces_to: #1.3) - row hash is deterministic - test: `services/obs-compliance-view/src/manifest.rs::sha256_of_rows_is_deterministic`
- [ ] AC 8 (traces_to: #1.7) - verify_manifest binary ships in crate manifest - verify: `services/obs-compliance-view/Cargo.toml` `[[bin]] name = "verify_manifest"`
- [ ] AC 9 (traces_to: #1.8) - auditor field reference present - verify: `services/obs-compliance-view/docs/manifest-format.md`
- [ ] AC 10 (traces_to: #1.9) - Out of scope lists manifest_pdf and phantom per-view export paths - verify: this spec Scope / new_files

## Verification

```bash
cd services && cargo test -p cyberos-obs-compliance-view manifest_signing::
cd services && cargo test -p cyberos-obs-compliance-view manifest::
cargo build -p cyberos-obs-compliance-view --bin verify_manifest
```

| Path | Covers |
|------|--------|
| `src/manifest.rs` tests | Signable bytes + row hash determinism |
| `src/manifest_signing.rs` tests | DEC-180 sign/verify + DEC-182 incomplete |
| `src/bin/verify_manifest.rs` | DEC-183 offline CLI |
| `docs/manifest-format.md` | Auditor-facing field reference |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9b-obs`.
- **Scope:** Re-spec/adopt against as-built manifest modules + verifier CLI; adds `manifest-format.md`; defers PDF cover and per-view export wiring.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9b-obs adopt — TASK-OBS-009 re-spec against as-built manifest stack.*
