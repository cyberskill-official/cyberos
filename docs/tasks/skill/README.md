# SKILL module — task index

_Generated 2026-05-17 — 11 tasks, 84 engineering-hours total._

## tasks

| Task | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-SKILL-101](TASK-SKILL-101-memory-integration/spec.md) | MUST | 1 | 6 | Skill memory integration — skill.invoked_started + skill.invoked_completed audit rows (skill.* namesp |
| [TASK-SKILL-102](TASK-SKILL-102-oci-registry/spec.md) | MUST | 1 | 10 | Self-hosted OCI registry for .skill bundles — cosign signing + tenant-scoped + immutable tags + 100M |
| [TASK-SKILL-103](TASK-SKILL-103-frontmatter-extension/spec.md) | MUST | 1 | 7 | SKILL.md frontmatter extension — allowed_memory_scopes + allowed_tools + version + signature enforced |
| [TASK-SKILL-104](TASK-SKILL-104-capability-broker/spec.md) | MUST | 1 | 12 | Capability broker — subprocess sandbox enforces allowed_tools + allowed_memory_scopes at invoke time; |
| [TASK-SKILL-105](TASK-SKILL-105-memory-capture-bundle/spec.md) | MUST | 2 | 9 | memory-capture@1 skill bundle — canonical SDK-style entry point for emitting memory capture rows from  |
| [TASK-SKILL-106](TASK-SKILL-106-memory-sync-bundle/spec.md) | SHOULD | 3 | 4 | memory-sync@1 skill bundle — operator-facing sync trigger that defers to Stage 4 orchestrator (slice- |
| [TASK-SKILL-107](TASK-SKILL-107-synthesis-author/spec.md) | COULD | 1 | 3 | synthesis-author@1 skill — nightly multi-memory auto-evolve composes derived memories from clustered  |
| [TASK-SKILL-108](TASK-SKILL-108-vietnam-mst-validate/spec.md) | MUST | 3 | 7 | vietnam-mst-validate@1 skill — Vietnamese Tax ID (MST) validation against General Department of Taxation  |
| [TASK-SKILL-109](TASK-SKILL-109-vietnam-bank-transfer/spec.md) | MUST | 3 | 7 | vietnam-bank-transfer@1 skill — VietQR + Napas247 transfer-code generator with bank-prefix validation, BR |
| [TASK-SKILL-110](TASK-SKILL-110-vietnam-vat-invoice/spec.md) | MUST | 3 | 11 | vietnam-vat-invoice@1 skill — Vietnamese e-invoice (hóa đơn) Decree 123 XML emitter with GDT submission,  |
| [TASK-SKILL-201](TASK-SKILL-201-oci-registry-deploy/spec.md) | MUST | 1 | 8 | SKILL OCI registry deploy for `.skill` bundles — R3 distribution stage with signed bundles + tag imm |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-SKILL-101→TASK-AI-003

**This module is depended on by:**

- **TEN**: TASK-TEN-005→TASK-SKILL-107

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._