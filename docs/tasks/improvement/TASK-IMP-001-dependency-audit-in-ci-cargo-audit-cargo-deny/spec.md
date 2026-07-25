---
id: TASK-IMP-001
title: "npm audit + license gate for install tooling"
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
blocks: [TASK-IMP-044]
related_tasks: [TASK-IMP-043, TASK-IMP-044]
routed_back_count: 0
awh: N/A
verify: T
phase: "Wave 1 - see and survive"
owner: Stephen Cheng (CTO)
created: 2026-07-08
effort_hours: 4
draft_reason: authoring
entered_via: audit
service: tools/install
new_files:
  - .github/workflows/npm-supply-chain.yml
  - tools/install/check-npm-supply-chain.sh
  - tools/install/check-npm-licenses.mjs
  - tools/install/npm-license-allowlist.txt
  - tools/install/docs-tools/package.json
  - tools/install/docs-tools/package-lock.json
  - tools/install/mcp/package-lock.json
  - tools/install/tests/test_npm_supply_chain.sh
  - tools/install/tests/fixtures/npm-supply-chain/critical-audit.json
  - tools/install/tests/fixtures/npm-supply-chain/gpl-package.json
modified_files:
  - docs/tasks/improvement/TASK-IMP-001-dependency-audit-in-ci-cargo-audit-cargo-deny/spec.md
source_pages:
  - "docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md R19"
  - "tools/install/mcp/package.json"
source_decisions:
  - "2026-07-25 session operator (batch/12b): REFRAME IMP-001 from cargo-audit to npm-audit + license gate over tools/install/mcp and docs-tools."
---

# TASK-IMP-001: npm audit + license gate for install tooling

## Summary

Reframe R19 from cargo-audit to Node surfaces that ship with the install payload. Add npm audit (high+) and a license/dep allowlist gate in CI for mcp and docs-tools.

## Problem

Neither install Node package root had a CI supply-chain gate. R19 cargo-audit is platform-scoped and out of 1.x payload scope.

## Proposed Solution

Private zero-dep package.json for docs-tools, check-npm-supply-chain.sh + license allowlist, npm-supply-chain.yml, planted critical/GPL fail fixtures.

## Alternatives Considered

- Keep cargo-audit as R19. Rejected for 1.x (operator reframe).
- Dependabot alone. Rejected without fail-closed audit/license gate.
- Full license-checker/Syft. Deferred for zero-dep trees.

## Success Metrics

- Primary: CI runs npm audit high+ and license allowlist; planted fixtures exit non-zero.
- Guardrail: zero-dep trees stay green.

## Scope

In scope: npm audit + license allowlist for mcp and docs-tools, workflow, fixtures, tests.

### Out of scope / Non-Goals

- cargo-audit for services/*.
- Scanning tools/caf.

## Dependencies

None blocking. TASK-IMP-044 depends on this gate.

## AI Authorship Disclosure

- **Tools used:** Composer (Cursor agent) under batch/12b operator mandate.
- **Scope:** Spec and gate for payload npm surfaces.
  **re-derived and CONFIRMED:** tools/install/mcp/package.json exists with zero runtime deps; R19 names cargo-audit.
  **re-derived and CORRECTED:** title and clauses target npm audit + license allowlist (operator 2026-07-25), not cargo-audit.
  **measured and ADDED:** docs-tools lacked package.json audit root — added with CI, allowlist, fixtures.
- **Human review:** Operator session HITL override for batch/12b; evidence under docs/batches/.

## 1. Description (normative)

- 1.1 A CI workflow MUST run npm audit --audit-level=high for tools/install/mcp and tools/install/docs-tools on every push and pull_request, and a high or critical advisory MUST fail the job.
- 1.2 tools/install/docs-tools MUST carry a package.json so it is an auditable npm package root (may be private with zero dependencies).
- 1.3 A license/dep gate MUST fail closed when any dependency is GPL-family, AGPL-family, unknown, or absent from tools/install/npm-license-allowlist.txt.
- 1.4 The audit checker MUST accept a planted audit-report JSON fixture and exit non-zero when that report contains a high or critical vulnerability.
- 1.5 The license checker MUST exit non-zero on a planted package.json that declares a GPL-only dependency not on the allowlist.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - workflow shape — test: `tools/install/tests/test_npm_supply_chain.sh::t_workflow_declared`
- [x] AC 2 (traces_to: #1.2) - docs-tools package.json private:true — test: `tools/install/tests/test_npm_supply_chain.sh::t_docs_tools_package_json`
- [x] AC 3 (traces_to: #1.3) - allowlist exists — test: `tools/install/tests/test_npm_supply_chain.sh::t_license_allowlist_exists`
- [x] AC 4 (traces_to: #1.4) - planted critical fails — test: `tools/install/tests/test_npm_supply_chain.sh::t_planted_critical_fails`
- [x] AC 5 (traces_to: #1.5) - planted GPL fails — test: `tools/install/tests/test_npm_supply_chain.sh::t_planted_gpl_fails`

## 3. Edge cases

- Zero-dependency package.json MUST pass both checks.
- Adding a real dependency requires an allowlist row before CI goes green.
