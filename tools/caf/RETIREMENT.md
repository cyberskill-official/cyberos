# Retiring standalone code-audit-framework (CAF) after the CyberOS absorption

CAF is now vendored into CyberOS at `tools/caf/` and wired into the ship gate as step 29 (see `docs/verification/caf-absorption-design.md` and `modules/cuo/chief-technology-officer/workflows/ship-tasks.md`). This mirrors the awh retirement (`tools/awh/RETIREMENT.md`): once the vendored copy is self-contained and gating, the standalone repos `CyberSkill/code-audit-framework` and `CyberSkill/code-audit-field-data` can be archived so there is one source of truth.

## Preconditions (all must hold before deleting anything)

1. Vendored and self-contained. `tools/caf/` runs without the standalone repo. Proof: `cd <cyberos> && PYTHONPATH=tools/caf/core/evals python3 -m code_audit_validator --all` returns `40/40 fixtures OK - ALL GREEN`, exit 0. (Captured during the absorption, 2026-06-20.)
2. Gate wired. Every gated module (those with `modules/<m>/.awh/goldenset.yaml`) also has `modules/<m>/audit-profile.yaml`; `bash scripts/caf_precommit_check.sh` is GREEN; step 29 and the step-29 dual condition are present in `ship-tasks.md`.
3. Field-data preserved. `tools/caf/field-data/` holds the `code-audit-field-data` records, reports, schemas, and pilot. New CyberOS audit runs emit `--emit-feedback` records here, so the calibration loop continues from inside CyberOS.
4. Pinned provenance recorded. The source sha of `code-audit-framework` at vendor time is written below, so the vendored copy is traceable.

## Retirement procedure (owner-run, on the host)

```bash
# 1. Tag the standalone repos at the absorbed sha (provenance), then bundle them for cold storage.
cd ~/Projects/CyberSkill/code-audit-framework
git tag -a absorbed-into-cyberos -m "Vendored into CyberOS tools/caf on 2026-06-20" && git push --tags
git bundle create ~/Projects/_archive/code-audit-framework-absorbed.bundle --all

cd ~/Projects/CyberSkill/code-audit-field-data
git tag -a absorbed-into-cyberos -m "Vendored into CyberOS tools/caf/field-data on 2026-06-20" && git push --tags
git bundle create ~/Projects/_archive/code-audit-field-data-absorbed.bundle --all

# 2. (optional) Once the bundles are stored and the CyberOS gate has run green on a real task,
#    archive or remove the standalone working copies.
#    git remote archive / repo settings -> Archive, or:
#    rm -rf ~/Projects/CyberSkill/code-audit-framework ~/Projects/CyberSkill/code-audit-field-data
```

Do not delete until preconditions 1-4 hold and at least one real task has passed the caf-gate end to end on a build machine. The vendored `tools/caf/` plus `field-data/` is then the single source of truth.

## Vendor provenance

- Vendored: 2026-06-20, via `tar` (excluding `.git/`, `.venv/`, `node_modules/`, `site/`).
- Source: `CyberSkill/code-audit-framework` (record the exact sha here on the host: `git -C ~/Projects/CyberSkill/code-audit-framework rev-parse HEAD`).
- Self-test at vendor time: `code_audit_validator --all` = 40/40 GREEN, exit 0.
