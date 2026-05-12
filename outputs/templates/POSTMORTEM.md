---
memory_id: mem_${UUID7}
scope: memories/refinements
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
  source: manual
  source_ref: ${INCIDENT_TS}
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["postmortem", "blameless"]
tags: [postmortem, blameless, ${TAGS}]
relationships: []
retention:
  rule: indefinite
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: publishable
source_freshness_tier: 5
---

# POSTMORTEM-${NEXT_NNN} ${SLUG_TITLE}

> Blameless postmortem. Traces the original signal, the missed call, and the rule that would have caught it. Per `engineering/incident-response` pattern.

## What happened
[1-paragraph factual summary. No blame.]

## Timeline
- **${TS_SIGNAL}** — original signal first observed
- **${TS_MISS}** — decision made that turned out to be wrong
- **${TS_DETECT}** — failure became visible
- **${TS_FIX}** — corrective action taken

## Original signal
[What was the early warning that, in hindsight, should have triggered different action?]

## Missed call
[What decision was made? What was the reasoning at the time? Why did it look reasonable?]

## What would have caught it
[The rule, hook, validator, eval, or process that would have prevented this — even retrospectively.]

## Root cause
[Single sentence. The deepest "why" that, if fixed, prevents this class of failure.]

## Actions
- [ ] Action 1 — owner: subject:X — by: YYYY-MM-DD
- [ ] Action 2 — owner: subject:Y — by: YYYY-MM-DD

## Refinement candidate
Does this warrant a §0.4 refinement proposal? [yes/no/maybe]
- If yes: link `memories/refinements/REF-NNN-<slug>.md`

## Related
- Original decision: [DEC-NNN]
- Rejected refinement that should have been adopted: [REJECTED-NNN]
- Similar past incidents: [POSTMORTEM-NNN]
