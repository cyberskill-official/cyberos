---
skill_id: feature-request-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for feature-request-audit

> Verifies the supervisor classifier routes auditor requests correctly. Per FR-SKILL-112.

## Positive triggers (MUST route here)

- "Audit this FR for completeness"
- "Has FR-007 changed since the last audit?"
- "Tell me which FRs would fail acceptance today"
- "Re-run the rubric against this FR collection"

## Negative triggers (MUST NOT route here)

- "Turn this PRD into a backlog of FRs" → feature-request-author
- "Generate FRs from this spec" → feature-request-author
- "Draft a tech spec from this FR" → fr-to-tech-spec
- "What's the team's holiday schedule?" → none

## Authoring notes

- Positive 1-3 anchor on "audit", "check", "re-audit" verbs from the
  description-format triggers (FR-SKILL-111).
- Positive 4 is the "re-audit" repeat case — operators come back to an
  existing FR-collection to re-run the rubric.
- Negative 1-2 derived from common confusion in pilot (author/audit pair).
  These ARE the author's positive triggers — by design, the classifier MUST
  pick the right side based on verb cues ("draft" / "generate" vs "audit" /
  "check").
- Negative 3 from the planned downstream chain (fr → tech-spec).
- Negative 4 is canonical "no skill" sanity.
- Re-author when classifier_version MAJOR-bumps.
