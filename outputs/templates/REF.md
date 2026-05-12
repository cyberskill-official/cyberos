---
memory_id: mem_${UUID7}
scope: memories/refinements
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
  consent_scope: ["refinement", "protocol-evolution"]
tags: []
relationships:
  - kind: implements
    target: ${IMPLEMENTS_DEC_ID}
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

# REF-${NEXT_NNN} ${SLUG_TITLE}

## Trigger
[What failure mode / observation triggered this refinement?]

## Tier
[TIER 1 minimum-viable / TIER 2 standard / TIER 3 expanded]

## AGENTS.md section
[Which section gets the amendment? § __ ]

## Exact prose to insert
[Verbatim text the §0.5 protocol upgrade adopts]

## Capability eval
- **What new behavior:** [describe]
- **Test fixture:** `runtime/tests/refinements/REF-${NEXT_NNN}/capability.test.py`
- **Pass criteria:** [memory rejected pre-REF now accepted, or vice versa]

## Regression eval
- **What to verify:** all existing memories still validate
- **Test fixture:** `runtime/tests/refinements/REF-${NEXT_NNN}/regression.test.py`
- **Pass criteria:** zero new validator failures

## Implements decision
DEC-${IMPLEMENTS_DEC_NNN}

## Related
- Bundle: [bundle letter]
- Protocol pin transition: [before SHA → after SHA]
- CHANGELOG entry: [link]
