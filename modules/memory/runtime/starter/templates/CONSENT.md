---
memory_id: mem_${UUID7}
scope: meta/consent
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
  consent_scope: ["consent-record"]
tags: [consent, phase-0]
relationships: []
retention:
  rule: consent-record-retain-with-personnel
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: ${SYNC_CLASS:local-only}
source_freshness_tier: ${FRESHNESS_TIER:20}
---

# CONSENT-${CONSENT_EVENT_ID}

## Subject
- **Subject ID:** subject:${SUBJECT_ID_TARGET}
- **Display name:** ${DISPLAY_NAME}

## Consent moment
- **Method:** ${ACK_METHOD:signed_contract|hr_recorded|console_click|sso_consent}
- **Recorded at:** ${TS_NOW}
- **Recorded by:** ${SUBJECT_ID}
- **Scope granted:** ${CONSENT_SCOPE:personnel, people-graph}

## Notice (optional product link)
- **Notice version / hash:** ${NOTICE_REF:n/a — Layer-1 record only}
- **Note:** This file is a Layer-1 BRAIN consent record (AGENTS.md §19). It is
  NOT the product EVAL `subject_acknowledgment` row. Workplace monitoring
  capture still requires the EVAL ledger + counsel-cleared notice per
  `docs/deploy/brain-capture-activation.md`.

## Reference
Personnel memories MUST set `consent.consent_event: ${CONSENT_EVENT_ID}` to
match this file's basename (`meta/consent/${CONSENT_EVENT_ID}.md`).
