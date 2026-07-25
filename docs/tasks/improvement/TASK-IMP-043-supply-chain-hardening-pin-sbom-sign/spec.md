---
id: TASK-IMP-043
title: "Payload supply-chain: pin Actions, emit SBOM, checksum posture"
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-08T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-001, TASK-IMP-069]
routed_back_count: 0
awh: N/A
verify: T
phase: "Wave 4 - hardening"
owner: Stephen Cheng (CTO)
created: 2026-07-08
effort_hours: 4
draft_reason: authoring
entered_via: audit
service: tools/install + .github/workflows
new_files:
  - tools/install/emit-payload-sbom.sh
  - tools/install/tests/test_payload_sbom.sh
  - docs/deploy/PAYLOAD-SUPPLY-CHAIN.md
modified_files:
  - .github/workflows/suite-gate.yml
  - .github/workflows/payload-gate.yml
  - .github/workflows/release.yml
source_pages:
  - "docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md R25"
  - "tools/install/release-assets.sh"
source_decisions:
  - "2026-07-25 session operator (batch/12b): scope R25 to payload — pin Actions; emit SBOM; prefer SHA256SUMS + SBOM over full cosign/GHCR."
---

# TASK-IMP-043: Payload supply-chain (pin, SBOM, checksum posture)

## Summary

Pin official actions/* in suite-gate, payload-gate, and release.yml to full SHAs; emit CycloneDX SBOM for payload release; document SHA256SUMS integrity and deferred cosign.

## Problem

Payload workflows pin Actions by mutable major tags. Releases publish SHA256SUMS but no SBOM. Full cosign-for-GHCR is platform-heavy for 1.x payload consumers.

## Proposed Solution

SHA-pin official actions; emit-payload-sbom.sh (hermetic CycloneDX); upload SBOM from release.yml; document posture.

## Alternatives Considered

- Full cosign of GHCR now. Deferred for 1.x payload scope.
- Require syft only. Rejected; hermetic emitter keeps tests offline.
- Pin every third-party Action. Deferred.

## Success Metrics

- Primary: official actions SHA-pinned; SBOM emitter produces CycloneDX; tests green.
- Guardrail: existing SHA256SUMS behaviour intact.

## Scope

In scope: Action SHA pins, SBOM script, release upload, docs, tests.

### Out of scope / Non-Goals

- cosign/GHCR attestation.
- Pinning tauri-action, rust-toolchain, Play upload.

## Dependencies

Adjacent to TASK-IMP-069.

## AI Authorship Disclosure

- **Tools used:** Composer (Cursor agent) under batch/12b operator mandate.
- **Scope:** payload pin + SBOM + checksum posture.
  **re-derived and CONFIRMED:** release-assets emits SHA256SUMS; workflows used floating actions/*@vN.
  **re-derived and CORRECTED:** R25 cosign/GHCR deferred; scope is payload SBOM + Action pins.
  **measured and ADDED:** no SBOM emitter or posture doc existed.
- **Human review:** Operator session HITL override for batch/12b; evidence under docs/batches/.

## 1. Description (normative)

- 1.1 suite-gate, payload-gate, and release.yml MUST pin each actions/checkout, setup-node, setup-java, and upload-artifact use to a full 40-character commit SHA.
- 1.2 tools/install/emit-payload-sbom.sh MUST write a CycloneDX JSON document listing payload files with sha256 hashes.
- 1.3 The release.yml payload job MUST invoke the SBOM emitter and upload the SBOM artifact to the GitHub Release.
- 1.4 docs/deploy/PAYLOAD-SUPPLY-CHAIN.md MUST document SHA256SUMS as the consumer integrity check and that cosign/GHCR signing is deferred.
- 1.5 A hermetic test MUST dry-run the SBOM emitter against a minimal fixture and assert CycloneDX shape.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - SHA pins — test: `tools/install/tests/test_payload_sbom.sh::t_actions_sha_pinned`
- [x] AC 2 (traces_to: #1.2) - emit script — test: `tools/install/tests/test_payload_sbom.sh::t_emit_script_present`
- [x] AC 3 (traces_to: #1.3) - release SBOM — test: `tools/install/tests/test_payload_sbom.sh::t_release_uploads_sbom`
- [x] AC 4 (traces_to: #1.4) - posture doc — test: `tools/install/tests/test_payload_sbom.sh::t_posture_doc`
- [x] AC 5 (traces_to: #1.5) - dry-run — test: `tools/install/tests/test_payload_sbom.sh::t_sbom_dry_run`

## 3. Edge cases

- Empty payload directory MUST fail the emitter loudly.
