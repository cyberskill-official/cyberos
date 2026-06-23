---
# Identity
name: obs.triage-alert
description: >-
  CUO runtime skill invoked by obs-router (FR-OBS-007) once per Alertmanager alert. It reads the alert,
  consults the affected service's RED metrics, recent deploys, and the runbook corpus, and returns a
  triage verdict: a calibrated confidence, a short summary, the suspected cause, and a suggested runbook
  when one clearly matches. obs-router routes on that confidence - at or above 0.70 it posts the summary
  to CHAT, below it pages on-call, and a sev-1 always pages regardless. Use this when triaging a fired
  alert into a human-readable cause and a next step. Do NOT use it to decide routing (that is
  obs-router's own pure decision) or to suppress or silence an alert.
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  invocation_handle: obs.triage-alert@1
  invoked_by: services/obs-router (cuo_triage.rs)
allowed_memory_scopes:
  read:
    - project:*
    - module:obs
  write:
    - project:obs/triage/{alert_fingerprint}
---

# obs.triage-alert

A single alert in, a triage verdict out. This skill does not page, post, or route - obs-router owns
those. Its only job is to turn one alert into an honest assessment that a tired on-call engineer can act
on, and a confidence that tells obs-router whether the assessment is trustworthy enough to send to CHAT
instead of paging.

## 1. Input

obs-router invokes the skill with one alert:

```json
{
  "skill": "obs.triage-alert@1",
  "alert": {
    "name": "HighErrorRate",
    "severity": "sev2",
    "fingerprint": "fp-...",
    "trace_id": "abc123",
    "summary": "5xx rate above 2% for api-gateway"
  }
}
```

## 2. Procedure

1. Read the alert: the name, the labels, and the summary annotation. Identify the affected service and
   what the alert actually measures.
2. Pull the recent signal for that service: its RED metrics (the `cyberos_requests_total`,
   `cyberos_errors_total`, `cyberos_duration_ms` series from FR-OBS-003), the error-rate and latency
   trend over the last 30 minutes, and any deploy in the last hour.
3. Search the runbook corpus (`runbooks-corpus/`) for a runbook whose triggers match this alert. A match
   needs the runbook's alert name or symptom to line up with the alert, not just a keyword overlap.
4. Form a single most-likely cause. Prefer a cause the signal supports: a deploy that lines up in time, a
   dependency that is also alerting, a saturation metric that crossed its limit. If the signal is thin,
   say so rather than guessing.
5. Set the confidence per section 4.

## 3. Output contract

Return exactly this shape (obs-router's `cuo_triage.rs` parses it; extra fields are ignored, a missing
`suggested_runbook` is allowed):

```json
{
  "confidence": 0.82,
  "summary": "api-gateway 5xx jumped from 0.3% to 2.4% at 14:02, two minutes after deploy v0.4.7.",
  "suspected_cause": "Regression in deploy v0.4.7 - the error onset tracks the rollout.",
  "suggested_runbook": { "title": "Roll back a bad gateway deploy", "url": "https://kb/.../rollback-gateway" }
}
```

`confidence` is a number in `[0, 1]`. `summary` is one or two sentences. `suspected_cause` names the most
likely cause in plain language. `suggested_runbook` is the matching runbook, or `null` when none clearly
matches - never invent a runbook URL.

## 4. Confidence calibration

Confidence is the probability that the suspected cause is correct, not how severe the alert is. It is the
single most important output: at 0.70 and above obs-router trusts the verdict enough to post to CHAT
instead of paging, so an overconfident wrong answer sends a real incident to a channel instead of a pager.

- 0.85 and up: the signal points one clear way (a deploy lines up in time, a named dependency is failing,
  a saturation metric is pinned) and a runbook matches.
- 0.70 to 0.85: a likely cause with supporting signal, but an alternative is not ruled out.
- below 0.70: thin or conflicting signal, a novel alert, or several plausible causes. obs-router pages on
  this - which is the safe outcome when triage is unsure.

When in doubt, return a lower confidence. Paging a human on an uncertain alert is cheap; sending a real
incident to a chat channel because triage was falsely confident is not.

## 5. Guardrails

- Stay within this alert and the CyberSkill systems. Do not speculate beyond the signal.
- Never invent a runbook, a URL, a metric value, or a deploy. If you did not see it, do not cite it.
- Never lower the severity or suggest suppressing the alert; routing and severity are not this skill's
  job.
- A sev-1 is paged by obs-router no matter what you return, so do not soften a sev-1 summary.
- If you cannot reach the metrics or the runbook corpus, return a low confidence with a summary that says
  the triage ran without its inputs - obs-router will page, which is correct.
