---
id: TASK-OBS-008
title: "OBS compliance views: scoped JSON API + Ed25519 proof"
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
related_tasks: [TASK-OBS-002, TASK-OBS-009, TASK-AUTH-004]
depends_on: [TASK-OBS-002]
blocks: [TASK-OBS-009]

source_pages:
  - website/docs/modules/obs.html#compliance-views

source_decisions:
  - DEC-175 2026-05-15 — Four slice-3 views: EU AI Act, PDPL, SOC 2, ISO 27001; PCI DSS deferred slice 5+
  - DEC-176 2026-05-15 — Ed25519 chain proof over canonical view payload; auditor verifies independently
  - DEC-177 2026-05-15 — Read-only; compliance view never mutates the audit chain
  - DEC-178 2026-05-15 — Auditor JWT carries external_auditor role; provisioned per engagement

language: rust 1.81
service: cyberos/services/obs-compliance-view/
new_files:
  - services/obs-compliance-view/Cargo.toml
  - services/obs-compliance-view/src/lib.rs
  - services/obs-compliance-view/src/main.rs
  - services/obs-compliance-view/src/views.rs
  - services/obs-compliance-view/src/auth.rs
  - services/obs-compliance-view/src/proof.rs
  - services/obs-compliance-view/src/query.rs
  - services/obs-compliance-view/src/summary.rs
  - services/obs-compliance-view/src/window.rs
  - services/obs-compliance-view/src/pii_scan.rs

modified_files: []

allowed_tools:
  - file_read: services/obs-compliance-view/**
  - file_write: services/obs-compliance-view/{src,tests}/**
  - bash: cd services && cargo test -p cyberos-obs-compliance-view

disallowed_tools:
  - expose compliance view across tenant boundaries
  - mutate audit chain from compliance view
  - export raw PII from compliance view
  - claim per-regime views/ file tree or PDF export as shipped this batch

effort_hours: 14
subtasks:
  - "1.0h: views.rs — View enum + kind table for four regulations"
  - "1.0h: auth.rs — external_auditor JWT + tenant scope"
  - "1.0h: query.rs — read-only l1_audit_log fetch"
  - "0.5h: window.rs — 365-day cap"
  - "0.5h: summary.rs — by_kind BTreeMap summary"
  - "0.5h: proof.rs — Ed25519 sign/verify over canonical payload"
  - "0.5h: pii_scan.rs — defence-in-depth regex scan"
  - "1.5h: main.rs — axum JSON HTTP shell + access audit emit"
  - "1.0h: batch/9b-obs adopt re-spec + audit"

risk_if_skipped: "External auditors need pre-built views over the immutable audit chain. Without tenant scoping and Ed25519 proof, every engagement becomes a bespoke SQL exercise and auditors cannot independently verify response integrity."
---

# TASK-OBS-008: OBS compliance views — tenant-scoped JSON API + Ed25519 proof

## Summary

Ship a read-only `obs-compliance-view` service over the memory audit chain. As-built surface is a flat Rust crate: `views.rs` (four regulation kind-filters), `auth.rs` (auditor JWT + tenant scope), `query.rs` / `summary.rs` / `window.rs` / `pii_scan.rs`, `proof.rs` (Ed25519 over canonical JSON — not `chain_proof.rs`), and `main.rs` (axum JSON HTTP API on `GET /:view` + `GET /healthz`). Responses are `{ payload, proof }` JSON envelopes.

## Problem

The prior engineering-spec claimed a per-regime `views/{eu_ai_act,pdpl,soc2,iso27001}.rs` tree, `chain_proof.rs`, PDF/JSON exporters, Grafana `compliance.json`, and standalone integration tests. The live crate consolidates views into `views.rs`, names the signer `proof.rs`, serves JSON only from `main.rs`, and keeps unit tests inline in each module. FM-004 blocked re-entry (`task@1` frontmatter + `## §N` body).

## Proposed Solution

Adopt the as-built layout:

- `views.rs` — `View` enum, slug parse, fixed `kinds()` per regulation (DEC-175)
- `auth.rs` — RS256 JWKS or HS256 dev verifier; `external_auditor` role; `enforce_tenant_scope`
- `query.rs` — tenant + kind + window read on `l1_audit_log` with RLS GUC (read-only, DEC-177)
- `window.rs` — reject inverted or >365-day windows
- `summary.rs` — `total_rows`, `by_kind` (`BTreeMap`), seq span
- `proof.rs` — Ed25519 `sign` / `verify` over canonical payload bytes (DEC-176)
- `pii_scan.rs` — email / VN CCCD / VN phone regex defence
- `main.rs` — authenticate → scope tenant → validate window → fetch → summarize → PII scan → sign → best-effort `obs.compliance_view_accessed` emit

## Alternatives Considered

- **Resume the engineering-spec paths as-is.** Rejected: phantom modules (`chain_proof.rs`, `views/eu_ai_act.rs`) do not exist; FM-004 blocks.
- **PDF export in slice 3.** Rejected: JSON API + proof ship first; PDF deferred.
- **Per-view export integration tests against live Postgres.** Rejected: slice ships inline unit tests; dedicated integration suite deferred.

## Success Metrics

- Primary: four view slugs parse and filter kinds; cross-tenant `?tenant_id=` refused; Ed25519 proof round-trips; 365-day window enforced.
- Guardrail: `cargo test -p cyberos-obs-compliance-view` green on the inline module tests cited below.

## Scope

In scope (as-built):

- `services/obs-compliance-view/src/{views,auth,proof,query,summary,window,pii_scan,main,lib}.rs`
- JSON HTTP API (`GET /eu-ai-act|pdpl|soc2|iso27001`, `GET /healthz`)
- Inline `#[cfg(test)]` modules in the files above

### Out of scope / Non-Goals

- Per-regime file tree (`views/eu_ai_act.rs`, `views/mod.rs`, etc.)
- PDF export (`export/pdf.rs`, wkhtmltopdf)
- Grafana dashboard (`deploy/obs/grafana/dashboards/compliance.json`)
- Postgres integration tests (`tests/eu_ai_act_test.rs`, `tests/cross_tenant_test.rs`, etc.)
- Chain-of-custody manifest signing (TASK-OBS-009)

## Dependencies

`depends_on: [TASK-OBS-002]` (tenant-aware query proxy pattern). Soft: TASK-AUTH-004 JWT/JWKS; `cyberos-audit-chain` for access-audit emit; TASK-OBS-009 manifest builds on this crate.

## 1. Description (normative)

- 1.1 `views.rs` MUST define exactly four `View` variants (EU AI Act, PDPL, SOC 2, ISO 27001) with stable URL slugs and non-empty `kinds()` filters per DEC-175.
- 1.2 `main.rs` MUST expose a JSON HTTP API: `GET /:view` for the four slugs and `GET /healthz`; successful responses MUST be a JSON envelope `{ payload, proof }` (not PDF).
- 1.3 `auth.rs` MUST verify auditor JWTs and MUST require the `external_auditor` role before serving any view (DEC-178).
- 1.4 `enforce_tenant_scope` MUST refuse a cross-tenant `?tenant_id=` query parameter when it differs from the JWT tenant (403).
- 1.5 `query.rs` MUST fetch rows read-only from `l1_audit_log` filtered by tenant, view kinds, and time window; it MUST NOT mutate the chain (DEC-177).
- 1.6 `window.rs` MUST reject inverted windows and windows wider than 365 days.
- 1.7 `proof.rs` MUST Ed25519-sign the canonical serialized payload bytes and expose offline `verify`; the module MUST NOT be named `chain_proof.rs` in the as-built tree.
- 1.8 `pii_scan.rs` MUST scan the serialized response for raw PII patterns before serve; matches MUST fail closed (HTTP 500 path in `main.rs`).
- 1.9 `summary.rs` MUST produce a summary block with stable `by_kind` ordering (`BTreeMap`) included in the signed payload.
- 1.10 `main.rs` SHOULD best-effort emit `obs.compliance_view_accessed` via `cyberos_audit_chain::emit_genesis` without failing the auditor read on emit error.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - view slug parse round-trips all four regulations - test: `services/obs-compliance-view/src/views.rs::parse_roundtrips_slug_for_every_view`
- [ ] AC 2 (traces_to: #1.1) - every view selects a non-empty kind set - test: `services/obs-compliance-view/src/views.rs::every_view_selects_a_non_empty_kind_set`
- [ ] AC 3 (traces_to: #1.1) - headline kinds present per regulation - test: `services/obs-compliance-view/src/views.rs::views_select_their_headline_kinds`
- [ ] AC 4 (traces_to: #1.3) - external_auditor role authorized - test: `services/obs-compliance-view/src/auth.rs::auditor_role_is_authorized`
- [ ] AC 5 (traces_to: #1.3) - missing auditor role refused - test: `services/obs-compliance-view/src/auth.rs::missing_auditor_role_is_refused`
- [ ] AC 6 (traces_to: #1.4) - cross-tenant tenant_id param refused - test: `services/obs-compliance-view/src/auth.rs::cross_tenant_param_is_refused`
- [ ] AC 7 (traces_to: #1.6) - window over 365 days rejected - test: `services/obs-compliance-view/src/window.rs::a_window_over_the_limit_is_rejected`
- [ ] AC 8 (traces_to: #1.6) - window at 365-day limit allowed - test: `services/obs-compliance-view/src/window.rs::a_window_at_the_limit_is_allowed`
- [ ] AC 9 (traces_to: #1.7) - Ed25519 sign then verify round-trips - test: `services/obs-compliance-view/src/proof.rs::sign_then_verify_roundtrips`
- [ ] AC 10 (traces_to: #1.7) - tampered canonical bytes fail verify - test: `services/obs-compliance-view/src/proof.rs::a_tampered_response_fails_verification`
- [ ] AC 11 (traces_to: #1.8) - raw email caught by PII scan - test: `services/obs-compliance-view/src/pii_scan.rs::raw_email_is_caught`
- [ ] AC 12 (traces_to: #1.9) - summary counts group by kind - test: `services/obs-compliance-view/src/summary.rs::counts_group_by_kind_and_track_seq_span`
- [ ] AC 13 (traces_to: #1.5) - read-only tenant-scoped l1_audit_log query - verify: `services/obs-compliance-view/src/query.rs` `fetch_rows`
- [ ] AC 14 (traces_to: #1.10) - best-effort obs.compliance_view_accessed emit - verify: `services/obs-compliance-view/src/main.rs` `emit_genesis` block
- [ ] AC 15 (traces_to: #1.2) - JSON HTTP routes live in main.rs (not per-view modules) - verify: `services/obs-compliance-view/src/main.rs` router + `View::parse`
- [ ] AC 16 (traces_to: #1.2,#1.7) - Out of scope lists phantom paths; new_files cite flat as-built tree only - verify: this spec Scope / new_files

## Verification

```bash
cd services && cargo test -p cyberos-obs-compliance-view proof::
cd services && cargo test -p cyberos-obs-compliance-view views:: auth:: window:: pii_scan:: summary::
```

| Path | Covers |
|------|--------|
| `src/views.rs` tests | DEC-175 kind table + slug routing |
| `src/auth.rs` tests | DEC-178 auditor role + tenant scope |
| `src/window.rs` tests | 365-day cap |
| `src/proof.rs` tests | DEC-176 Ed25519 proof |
| `src/pii_scan.rs` tests | Response PII defence |
| `src/summary.rs` tests | Summary block shape |
| `src/main.rs` | JSON HTTP shell + handler pipeline |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9b-obs`.
- **Scope:** Re-spec/adopt against as-built `cyberos-obs-compliance-view`; removed phantom per-regime tree, `chain_proof.rs`, PDF, Grafana dashboard, and Postgres integration tests from claimed surface.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9b-obs adopt — TASK-OBS-008 re-spec against as-built obs-compliance-view.*
