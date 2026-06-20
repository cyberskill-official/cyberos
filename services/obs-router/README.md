# obs-router

The alert router for CyberOS (FR-OBS-007). It receives Alertmanager webhooks, triages each firing alert
through CUO, and routes the result to CHAT or PagerDuty. A sev-1 always pages; everything else is paged
or posted based on the triage confidence.

```
Alertmanager --POST /alert--> obs-router --> CUO triage --> { CHAT (>= 0.70) | PagerDuty (< 0.70) }
                                                  |
                                            sev-1 always pages
```

## The routing decision

For each firing alert the router asks the CUO `obs.triage-alert` skill for a verdict: a confidence in
`[0, 1]`, a one-line summary, a suspected cause, and a runbook when one matches. Then it routes:

- A sev-1 pages on-call (PagerDuty) regardless of confidence, and also posts to CHAT.
- Confidence at or above 0.70: the alert is well understood, so the summary goes to CHAT (`#oncall`).
- Confidence below 0.70, or any triage error or timeout: the alert pages on-call.

Paging on an uncertain alert is the safe default. The router never silences or downgrades an alert.

## Configuration (env)

Every dependency is optional, so the service starts and degrades safely when one is unset.

- `OBS_ROUTER_BIND` - listen address (default `0.0.0.0:7777`).
- `OBS_ROUTER_WEBHOOK_SECRET` - shared secret required in the `X-CyberOS-Webhook-Secret` header. Unset
  disables auth (dev only); a wrong secret is always 401 when this is set.
- `OBS_CUO_TRIAGE_URL` - the CUO triage endpoint (see below). Unset means triage fails, so every alert
  pages.
- `OBS_CHAT_WEBHOOK_URL` - the CHAT incoming-webhook URL. Unset means the CHAT leg fails over to paging.
- `OBS_PAGERDUTY_ROUTING_KEY` - the PagerDuty Events API v2 routing key.
- `OBS_PAGERDUTY_ENDPOINT` - override the Events API endpoint (default the public enqueue URL).

## The CUO triage endpoint

`obs.triage-alert` is a CUO skill, and CUO runs skills in-process rather than behind a server, so the
endpoint obs-router calls is a thin HTTP front door in the CUO module: `cuo/triage_server.py`. Run it
next to obs-router:

```sh
# In modules/cuo. Real triage needs an LLM invoker (ANTHROPIC_API_KEY) or the cyberos-skill binary.
export CYBEROS_ROOT="$(git rev-parse --show-toplevel)"
python -m cuo.triage_server --port 8731

# Then point obs-router at it.
export OBS_CUO_TRIAGE_URL="http://localhost:8731/"
```

The endpoint speaks the contract in `src/cuo_triage.rs`: it accepts
`{"skill":"obs.triage-alert@1","alert":{...}}` and returns
`{"confidence","summary","suspected_cause","suggested_runbook"}`.

If triage cannot reach its inputs - no invoker on the host, the skill invocation fails, or it raises -
the endpoint returns HTTP 200 with confidence 0.0 and a summary that says triage ran blind. obs-router
then pages, which is the correct outcome for an alert that could not be assessed. The endpoint does not
return a 5xx for this case, because an unsure triage is a low-confidence answer, not a server fault.

## What still needs a live environment

The in-repo router and the triage endpoint are unit- and property-tested. End-to-end validation needs the
real targets wired: a CUO host with an LLM invoker, a CHAT webhook, and a PagerDuty routing key. Writing
the `obs.alert_triaged` / `obs.alert_acked` rows to the memory audit chain (rather than the current log
sink) is the remaining follow-up.
