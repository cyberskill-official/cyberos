---
# Template for memories/decisions/DEC-NNN-<slug>.md
# Variables (filled by `cyberos add DEC`):
#   ${UUID7}, ${TS_NOW}, ${SUBJECT_ID}, ${NEXT_NNN}, ${SLUG}, ${SLUG_TITLE},
#   ${CLASSIFICATION:operational}, ${AUTHORITY:human-edited}, ${TAGS},
#   ${PROV_SOURCE}, ${PROV_SOURCE_REF}, ${SYNC_CLASS:publishable},
#   ${FRESHNESS_TIER:5}
memory_id: mem_${UUID7}
scope: memories/decisions
classification: ${CLASSIFICATION:operational}
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
  source: ${PROV_SOURCE:chat}
  source_ref: ${PROV_SOURCE_REF}
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["decision"]
tags: []
relationships: []
retention:
  rule: indefinite
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: ${SYNC_CLASS:publishable}
source_freshness_tier: ${FRESHNESS_TIER:5}
---

# DEC-${NEXT_NNN} ${SLUG_TITLE}

## Context
[2-5 sentences: what's the issue motivating this decision? What constraints and forces are at play?]

## Decision
[1-3 sentences: clearly state the choice being made.]

## Alternatives Considered

### Alternative 1: [Name]
- **Pros:** [benefits]
- **Cons:** [drawbacks]
- **Why not:** [specific rejection reason]

### Alternative 2: [Name]
- **Pros:** [benefits]
- **Cons:** [drawbacks]
- **Why not:** [specific rejection reason]

## Consequences

### Positive
- [benefit 1]
- [benefit 2]

### Negative
- [trade-off 1]
- [trade-off 2]

### Risks
- [risk and mitigation]

## Related
- Supersedes: [DEC-NNN or none]
- Implements: [REF-NNN or none]
- See also: [DEC-NNN, FACT-NNN]
