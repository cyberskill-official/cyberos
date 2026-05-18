# SKILL module — feature request index

_Generated 2026-05-17 — 11 FRs, 84 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-SKILL-101](FR-SKILL-101-brain-integration.md) | MUST | 1 | 6 | Skill BRAIN integration — skill.invoked_started + skill.invoked_completed audit rows (skill.* namesp |
| [FR-SKILL-102](FR-SKILL-102-oci-registry.md) | MUST | 1 | 10 | Self-hosted OCI registry for .skill bundles — cosign signing + tenant-scoped + immutable tags + 100M |
| [FR-SKILL-103](FR-SKILL-103-frontmatter-extension.md) | MUST | 1 | 7 | SKILL.md frontmatter extension — allowed_brain_scopes + allowed_tools + version + signature enforced |
| [FR-SKILL-104](FR-SKILL-104-capability-broker.md) | MUST | 1 | 12 | Capability broker — subprocess sandbox enforces allowed_tools + allowed_brain_scopes at invoke time; |
| [FR-SKILL-105](FR-SKILL-105-brain-capture-bundle.md) | MUST | 2 | 9 | brain-capture@1 skill bundle — canonical SDK-style entry point for emitting BRAIN capture rows from  |
| [FR-SKILL-106](FR-SKILL-106-brain-sync-bundle.md) | SHOULD | 3 | 4 | brain-sync@1 skill bundle — operator-facing sync trigger that defers to Stage 4 orchestrator (slice- |
| [FR-SKILL-107](FR-SKILL-107-synthesis-author.md) | COULD | 1 | 3 | synthesis-author@1 skill — nightly multi-brain auto-evolve composes derived memories from clustered  |
| [FR-SKILL-108](FR-SKILL-108-vietnam-mst-validate.md) | MUST | 3 | 7 | vietnam-mst-validate@1 skill — Vietnamese Tax ID (MST) validation against General Department of Taxation  |
| [FR-SKILL-109](FR-SKILL-109-vietnam-bank-transfer.md) | MUST | 3 | 7 | vietnam-bank-transfer@1 skill — VietQR + Napas247 transfer-code generator with bank-prefix validation, BR |
| [FR-SKILL-110](FR-SKILL-110-vietnam-vat-invoice.md) | MUST | 3 | 11 | vietnam-vat-invoice@1 skill — Vietnamese e-invoice (hóa đơn) Decree 123 XML emitter with GDT submission,  |
| [FR-SKILL-201](FR-SKILL-201-oci-registry-deploy.md) | MUST | 1 | 8 | SKILL OCI registry deploy for `.skill` bundles — R3 distribution stage with signed bundles + tag imm |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-SKILL-101→FR-AI-003

**This module is depended on by:**

- **TEN**: FR-TEN-005→FR-SKILL-107

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._