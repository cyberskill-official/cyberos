---
task_id: TASK-TEN-105
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands signed-bundle export for tenant offboarding (TASK-TEN-104) with deterministic ZIP + Ed25519 signature + memory chain anchor. 750 lines, 22 §1 clauses, 20 ACs, 5 tests, 16 failure modes, 10 implementation notes. 2 migrations, 4 endpoints, 6 memory audit kinds. Per-tenant signing keys generated at provisioning (modification to TASK-TEN-001).

6 issues resolved.

## §2 — Findings (all resolved)

### ISS-001 — Per-tenant keypair rotation breaks old-bundle verification

§11.4 clarified: keypair never regenerated; rotation = new keypair alongside old; old bundles verify with old archived key.

### ISS-002 — Build resume after worker crash

§10 + §11.6 — TASK-MCP-007 worker checkpoints per-source; crash resumes from last source completed.

### ISS-003 — Audit chain segment size for active tenants

§10 row + §11.8 — JSONL streaming format; pagination in builder; sev-3 log if > 1M rows.

### ISS-004 — Pre-deletion gate timing race

If bundle expired (T+30d) but attestation submitted at T+91d, gate fails. Acceptable — bundles can be regenerated. Documented as intended behaviour.

### ISS-005 — S3 lifecycle vs application cleanup race

§11.5 — both mechanisms run; lifecycle is defense-in-depth; idempotent.

### ISS-006 — Cross-residency bundle storage

§11.9 — bundle stored in tenant's residency S3 per TASK-TEN-103 trip-wire; no cross-residency export at slice 2.

## §3 — Resolution

All 6 mechanical concerns addressed. Deterministic ZIP, per-tenant cryptography, pre-deletion gate, chain anchor — all forensically sound for legal evidence chain.

The 750-line length appropriate for 8h-effort scope with substantial crypto + ZIP determinism requirements.

**Score = 10/10.**

---

*End of TASK-TEN-105 audit.*
