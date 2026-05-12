---
memory_id: mem_${UUID7}
scope: memories/projects
classification: operational
authority: ${AUTHORITY:human-edited}
version: 1
created_at: ${TS_NOW}
created_by: ${SUBJECT_ID}
last_updated_at: ${TS_NOW}
updated_by: ${SUBJECT_ID}
supersedes: null
superseded_by: null
expires_at: null
provenance:
  source: ${PROV_SOURCE:manual}
  source_ref: ${PROV_SOURCE_REF}
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["project-anchor"]
tags: []
relationships: []
retention:
  rule: indefinite
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: ${SYNC_CLASS:shared}
source_freshness_tier: ${FRESHNESS_TIER:15}
---

# PROJECT-${NEXT_NNN} ${SLUG_TITLE}

## Identity
- **Project ID:** ${PROJECT_ID}
- **Started:** ${START_DATE}
- **Status:** [active / paused / completed / cancelled]

## Stakeholders
- **Owner:** subject:${OWNER_SUBJECT_ID}
- **Client (if any):** client:${CLIENT_ID}
- **Team:** [list subject IDs]

## Timeline
- **Start:** ${START_DATE}
- **Target completion:** ${TARGET_DATE}
- **Phases:** [P0 / P1 / etc. if known]

## Scope
[1-3 sentence summary of what this project delivers]

## Decisions
- DEC-NNN — [link to key decisions]
- DEC-NNN — [...]

## Status snapshot
- **Last updated:** ${TS_NOW}
- **Current focus:** [what's active right now]
- **Blockers:** [list or "none"]

## Related
- See also: [PROJECT-NNN, DEC-NNN, PERSON-NNN]
