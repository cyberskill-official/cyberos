---
id: obs.triage-alert
version: 1.0.0
description: Triage Alertmanager alerts using KB runbook RAG; emit confidence, summary, suspected cause, and suggested runbook.
tools: [kb_search]
output_schema:
  type: object
  required: [confidence, summary, suspected_cause]
  properties:
    confidence: { type: number, minimum: 0, maximum: 1 }
    summary: { type: string }
    suspected_cause: { type: string }
    suggested_runbook:
      type: object
      required: [kb_article_id, title, url]
      properties:
        kb_article_id: { type: string }
        title: { type: string }
        url: { type: string }
---

# obs.triage-alert@1

Given an Alertmanager alert, search the KB runbook corpus for similar past
incidents, synthesize a triage summary, and assess confidence.

## Procedure

1. Extract `alertname`, `severity`, `tenant_id`, `trace_id`, labels, and annotations.
2. Search KB runbooks for the alert name, summary, service, and suspected resource.
3. Read the top three matching runbooks.
4. Summarize what happened, why it is likely happening, and what the operator should try first.
5. Return confidence based on runbook match quality, alert clarity, and historical incident similarity.

## Output

Return JSON matching `output_schema`. Keep confidence conservative: below `0.70`
when the alert lacks a clear runbook match or suspected cause.
