---
batch: batch/12b-supply-chain
members:
  - TASK-IMP-001
  - TASK-IMP-003
  - TASK-IMP-043
  - TASK-IMP-044
started: 2026-07-25
ended: 2026-07-25
---

# Batch 12b — PR-B supply-chain

Operator-scoped 1.x payload supply-chain batch (not full `services/*` platform hardening).

| Task | Outcome |
|------|---------|
| TASK-IMP-001 | Reframes R19 cargo-audit → npm audit + license allowlist on `tools/install/mcp` + `docs-tools` |
| TASK-IMP-003 | gitleaks CI (SHA-pinned) + pre-push soft-skip + `.gitleaks.toml` allowlists |
| TASK-IMP-043 | Action SHA pins + CycloneDX payload SBOM on release + SHA256SUMS posture doc (cosign deferred) |
| TASK-IMP-044 | Dependabot weekly npm + github-actions (+ cargo `/services` secondary) |

Session HITL gates: `batch-12b-gate1-acceptance.md`, `batch-12b-gate2-acceptance.md`.
