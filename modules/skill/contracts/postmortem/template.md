---
template: postmortem@1
title: <incident-id> — Post-mortem
incident_id: <Linear/Jira/PagerDuty ref>
severity: sev2    # sev1 | sev2 | sev3 | sev4 | sev5
started_at:  2026-MM-DDTHH:MM:SS+07:00
detected_at: 2026-MM-DDTHH:MM:SS+07:00
resolved_at: 2026-MM-DDTHH:MM:SS+07:00
duration_minutes: <integer>
mttd_minutes: <integer>
mttr_minutes: <integer>
services_affected: [<service-a>, <service-b>]
customer_impact: "<# customers / requests / dollars>"
provenance: { source_path: ./incident-timeline.md, source_hash: sha256:<hash> }
facilitator: @<facilitator>
participants: [@<a>, @<b>, @<c>]
---

# <incident-id> — Post-mortem

## 1. Incident Summary
1-2 paragraphs.

## 2. Timeline
| minute | event | source |
|---|---|---|
| T+0 | <start> | <log ref> |
| T+5 | <detected> | <pager> |
| T+M | <resolved> | <log ref> |

## 3. Customer Impact
Numbers — affected users / failed requests / revenue loss.

## 4. Detection
How we found out; MTTD analysis; could we have detected earlier.

## 5. Response
What we did; what worked; what didn't.

## 6. Contributing Factors (Five-Whys)
- Why 1: ...
- Why 2: ...
- Why 3: ...

## 7. What Went Well
- ...

## 8. What Went Wrong
- ...

## 9. Where We Got Lucky
- ...

## 10. Action Items
| title | owner | due_date | linked_ticket | severity |
|---|---|---|---|---|

## 11. Lessons Learned
- ...

## 12. SLO/SLA Impact
Error-budget burn calculation per SLO.

<!-- ## 13. Public Communication Log   — when sev1/sev2 -->
<!-- ## 14. Data Breach Assessment     — when personal-data exposure -->
<!-- ## 15. Security Disclosure        — when security exploit -->
<!-- ## 16. Executive Brief            — when sev1 -->
<!-- ## 17. Model Behaviour Analysis   — when AI/ML system involved -->
