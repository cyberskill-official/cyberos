---
memory_id: mem_${UUID7}
scope: memories/preferences
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
  consent_scope: ["preference"]
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
source_freshness_tier: ${FRESHNESS_TIER:25}
---

# PREF-${NEXT_NNN} ${SLUG_TITLE}

## Preference
[State the preference in one sentence, action-oriented.]

## Scope
- **Applies to:** [agent / project / subject / context]
- **Override-rule:** [what conditions allow ignoring this preference]

## Rationale
[Why this preference exists. Reference the friction / failure / experience that drove it.]

## Examples
- ✓ Following: [example of behavior matching the preference]
- ✗ Violating: [example of behavior NOT matching]

## Related
- See also: [PREF-NNN, DEC-NNN]
