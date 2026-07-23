---
batch: ship/batch-8f-entrypoint
members:
  - TASK-IMP-138
started: 2026-07-23T23:30:00+07:00
ended: 2026-07-24T00:30:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8F — entrypoint identity Branch A (TASK-IMP-138)

Base: `ship/batch-8e-benchmarks` @ `7062b4dd`.

## Delivered

- Root `AGENTS.md` + `CLAUDE.md` were symlinks to the protocol; replaced with real thin-spine / pointer files (protocol restored at `modules/memory/cyberos/data/AGENTS.md`).
- Pointer files name `.cyberos/AGENT-ENTRY.md` first; describe AGENTS.md as thin spine.
- `install.sh` platform AGENTS exception removed; pointer templates updated.
- `test_entrypoint_identity.sh` 6/6 green.
