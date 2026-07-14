---
id: NFR-KB-006
title: "KB translation parity — translated doc MUST reference + lag source by ≤ 24h"
module: KB
category: maintainability
priority: SHOULD
verification: T
phase: P1
slo: "100% of `translation_of:` docs reference an existing source; sync lag ≤ 24h after source change"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-KB-009]
---

## §1 — Statement (BCP-14 normative)

1. Documents with `translation_of: <doc_id>` frontmatter **MUST** point to a real source document in the same tenant.
2. When the source doc updates, the translated doc **MUST** be flagged "out of sync" within 24h; UI shows this on the translated doc.
3. Translated docs **MUST NOT** auto-update from the source — translation is a human-mediated activity.
4. The platform **MUST** show side-by-side diff between current translation and source's latest version.
5. Bulk translation drift (> 25% of corpus stale) **MUST** trigger sev-3 product review.

## §2 — Why this constraint

CyberOS is multilingual (VN + EN minimum). Translation-of relationships keep the language variants linked, but auto-translation would silently produce wrong nuance. The out-of-sync flag is the "this needs human attention" signal. The 24h lag is the freshness floor; the diff UI makes the human's job easier.

## §3 — Measurement

- Counter `kb_translation_dangling_total` — must be 0.
- Gauge `kb_translation_stale_count` — surfaces drift.
- Histogram `kb_translation_stale_age_days`.

## §4 — Verification

- Integration test (T) — create translation_of; update source; assert flag within 24h.
- CI gate (T) — all `translation_of:` refs resolve.
- Snapshot test (T) — diff UI renders correctly.

## §5 — Failure handling

- Dangling translation_of → CI block.
- Stale > 25% → product retrospective.
- Auto-update detected (shouldn't happen) → sev-2; investigate.

---

*End of NFR-KB-006.*
