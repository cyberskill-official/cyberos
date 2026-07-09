---
description: Scaffold the FR workflow into the current repo (gate autodetect + docs/feature-requests + config).
---
Set up the FR workflow in the current repo:

1. If `.cyberos/init.sh` exists, run `bash .cyberos/init.sh` and report what it detected.
2. Otherwise replicate it directly:
   - Detect this repo's build / lint / test / coverage commands (Cargo, package.json, pyproject, go.mod, or Makefile).
   - Create `docs/feature-requests/_audits/` and `.cyberos/`.
   - Write `.cyberos/gates.env` with `BUILD_CMD` / `LINT_CMD` / `TEST_CMD` / `COVERAGE_CMD`, `COVERAGE_MIN=90`, `CAF_ENABLED=false`, `AWH_ENABLED=false`, `HITL_REQUIRED=true`.
   - Create `docs/feature-requests/BACKLOG.md` from `skills/ship-feature-requests/machine` guidance (or the pack template).

Then tell the user to write their first FR (`FR-001-<slug>.md`, `status: ready_to_implement`), add its row to `BACKLOG.md`, and run `/ship-fr`.
