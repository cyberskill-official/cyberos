---
id: TASK-IMP-003
title: "Secret scanning in CI and pre-push"
template: task@1
type: improvement
module: improvement
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-08T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-001, TASK-IMP-043]
routed_back_count: 0
awh: N/A
verify: T
phase: "Wave 1 - see and survive"
owner: Stephen Cheng (CTO)
created: 2026-07-08
effort_hours: 3
draft_reason: authoring
entered_via: audit
service: .github/workflows + .githooks
new_files:
  - .github/workflows/secret-scan.yml
  - .gitleaks.toml
  - tools/install/check-secrets.sh
  - tools/install/tests/test_secret_scan.sh
modified_files:
  - .githooks/pre-push
source_pages:
  - "docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md R27"
source_decisions:
  - "2026-07-25 session operator (batch/12b): gitleaks in CI hard-fail + pre-push soft-skip; allowlist false positives."
---

# TASK-IMP-003: Secret scanning in CI and pre-push

## Summary

Add SHA-pinned gitleaks CI, .gitleaks.toml allowlists, check-secrets.sh, and pre-push soft-skip when gitleaks is missing locally.

## Problem

Secrets can land in git with no mechanical stop. Pre-push only gates Rust services/. Docs contain example tokens that look like secrets.

## Proposed Solution

Pin gitleaks-action by SHA, allowlist known false-positive paths, wrap gitleaks with --probe-fixture fail proof, soft-skip in pre-push when binary absent.

## Alternatives Considered

- GitHub secret scanning only. Rejected as sole control.
- Hard-fail pre-push when gitleaks missing. Rejected; soft-skip + CI hard-fail.
- TruffleHog. Deferred; gitleaks is R27.

## Success Metrics

- Primary: CI secret-scan SHA-pinned; probe fixture fails; clean tree with allowlists passes.
- Guardrail: pre-push does not block when gitleaks absent.

## Scope

In scope: workflow, config, check script, pre-push, tests.

### Out of scope / Non-Goals

- Rewriting historical example-token commits.
- Enterprise GHAS configuration.

## Dependencies

None.

## AI Authorship Disclosure

- **Tools used:** Composer (Cursor agent) under batch/12b operator mandate.
- **Scope:** gitleaks CI + pre-push soft-skip.
  **re-derived and CONFIRMED:** pre-push gates Rust services/ only; R27 names gitleaks.
  **re-derived and CORRECTED:** baseline scan reported 42 findings (docs/status + FR examples) — allowlists required.
  **measured and ADDED:** probe-fixture fail path and SHA-pinned Action were absent.
- **Human review:** Operator session HITL override for batch/12b; evidence under docs/batches/.

## 1. Description (normative)

- 1.1 A CI workflow MUST run gitleaks on push and pull_request and MUST fail when leaks are reported (no continue-on-error).
- 1.2 The gitleaks Action invocation MUST be pinned to a full commit SHA.
- 1.3 .gitleaks.toml MUST allowlist documented false-positive paths so the gate is enforceable.
- 1.4 .githooks/pre-push MUST invoke the secret checker when gitleaks is installed, and MUST soft-skip with a clear message when it is not.
- 1.5 tools/install/check-secrets.sh --probe-fixture MUST plant a fake AWS-shaped key in a temporary directory and exit non-zero when gitleaks is available.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - hard-fail workflow — test: `tools/install/tests/test_secret_scan.sh::t_workflow_hard_fail`
- [x] AC 2 (traces_to: #1.2) - SHA pin — test: `tools/install/tests/test_secret_scan.sh::t_action_sha_pinned`
- [x] AC 3 (traces_to: #1.3) - allowlist — test: `tools/install/tests/test_secret_scan.sh::t_allowlist_present`
- [x] AC 4 (traces_to: #1.4) - pre-push soft-skip — test: `tools/install/tests/test_secret_scan.sh::t_pre_push_soft_skip`
- [x] AC 5 (traces_to: #1.5) - probe fixture — test: `tools/install/tests/test_secret_scan.sh::t_probe_fixture_fails`

## 3. Edge cases

- Allowlists MUST be path-scoped; real secrets outside them MUST still fail CI.
