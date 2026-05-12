---
memory_id: mem_${UUID7}
scope: memories/people
classification: personnel
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
  consent_event: ${CONSENT_EVENT_ID}
  consent_scope: ["personnel", "people-graph"]
tags: []
relationships: []
retention:
  rule: personnel-7-years-post-employment
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: ${SYNC_CLASS:publishable}
source_freshness_tier: ${FRESHNESS_TIER:20}
---

# PERSON-${NEXT_NNN} ${SLUG_TITLE}

## Identity
- **Subject ID:** subject:${SUBJECT_ID_TARGET}
- **Display name:** ${DISPLAY_NAME}
- **Role:** ${ROLE}
- **Email (work):** ${WORK_EMAIL}
- **Timezone:** ${TZ}
- **Language(s):** ${LANGUAGES}

## Working preferences
- **Working hours:** [range, timezone-anchored]
- **Communication style:** [async/sync preference, response time expectations]
- **Decision style:** [consult-first / decide-and-broadcast / etc.]

## Context
[How this person relates to the project / org / client]

## Consent
- **Consent given for:** [people-graph inclusion / decisions tracking / etc.]
- **Consent event:** [audit row ID referencing the consent moment]
- **Retention:** [per-classification — usually 7 years post-employment-end for personnel]

## Privacy
- This is a `personnel`-class memory; conflicts NEVER auto-resolve (always human review per §9.1)
- Compensation, gov-ID, home address, health PII are DENYLISTED per §9.3 — do not record here
- For sensitive ops, see `member/${SUBJECT_ID_TARGET}/private/`
