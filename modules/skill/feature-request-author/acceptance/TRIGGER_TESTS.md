---
skill_id: feature-request-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for feature-request-author

> Verifies the supervisor classifier routes these user phrasings correctly. Per FR-SKILL-112.

## Positive triggers (MUST route here)

- "Turn this PRD into a backlog of FRs"
- "Draft a feature request for the new email-bounce handling"
- "Expand the spec into FR markdowns"
- "Generate the FR backlog from these source docs"

## Negative triggers (MUST NOT route here)

- "Audit this existing FR for completeness" → feature-request-audit
- "Has FR-007 changed since the last audit?" → feature-request-audit
- "Draft a tech spec from this FR" → fr-to-tech-spec
- "What's our company holiday schedule?" → none

## Authoring notes

- Positive triggers 1-3 derived from author intuition matching the description's
  trigger phrases (FR-SKILL-111 enrichment); will be cross-checked against real
  OBS user phrasings once the runtime ships and OBS dashboards have volume.
- Positive trigger 4 is the canonical CyberOS phrasing observed during pilot.
- Negative triggers 1-2 derived from common author/audit confusion observed
  during the v0.2.0 pilot (users confused the author/audit pair).
- Negative trigger 3 derived from the planned fr-to-tech-spec routing (when
  that skill goes from scaffold → accepted in P1).
- Negative trigger 4 is a canonical "no skill" sanity case.
- Re-author when classifier_version MAJOR-bumps (today v3.0.0-a4).
