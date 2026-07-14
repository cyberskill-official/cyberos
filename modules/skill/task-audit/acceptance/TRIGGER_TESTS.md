---
skill_id: task-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for task-audit

> Verifies the supervisor classifier routes auditor requests correctly. Per TASK-SKILL-112.

## Positive triggers (MUST route here)

- "Audit this FR for completeness"
- "Has FR-007 changed since the last audit?"
- "Tell me which FRs would fail acceptance today"
- "Re-run the rubric against this FR collection"
- "Audit this mixed batch - some FRs are task@1, some engineering-spec"   (per-file detection is the audit's job - TASK-CUO-208)

## Negative triggers (MUST NOT route here)

- "Turn this PRD into a backlog of FRs" → task-author
- "Generate FRs from this spec" → task-author
- "Draft a tech spec from this FR" → fr-to-tech-spec
- "What's the team's holiday schedule?" → none
- "Convert this FR from task@1 to engineering-spec@1" → task-author   (conversion = re-authoring - TASK-CUO-208)

## Authoring notes

- Positive 1-3 anchor on "audit", "check", "re-audit" verbs from the
  description-format triggers (TASK-SKILL-111).
- Positive 4 is the "re-audit" repeat case — operators come back to an
  existing FR-collection to re-run the rubric.
- Negative 1-2 derived from common confusion in pilot (author/audit pair).
  These ARE the author's positive triggers — by design, the classifier MUST
  pick the right side based on verb cues ("draft" / "generate" vs "audit" /
  "check").
- Negative 3 from the planned downstream chain (fr → tech-spec).
- Negative 4 is canonical "no skill" sanity.
- Re-author when classifier_version MAJOR-bumps.
- TASK-CUO-208 cases (P5/N5, last bullets above): template-profile routing added with TEMPLATE_PROFILES.md.
