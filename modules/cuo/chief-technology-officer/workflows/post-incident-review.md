---
workflow_id: chief-technology-officer/post-incident-review
workflow_version: 1.0.0
purpose: Drive a blameless post-mortem after a sev1/sev2 incident — from raw timeline + pager logs to a 10/10 postmortem + action items in the project tracker.
persona: cuo/chief-technology-officer
cadence: per-event
status: shipped
pattern: persona_pair
peer_persona: chief-risk-officer
peer_workflow: per-incident-postmortem
shared_artefact: incident-report
handoff_step: 3

inputs:
  - { name: incident_timeline, source: PagerDuty / incident.io / Linear incident, format: markdown export }
  - { name: pager_logs,        source: on-call alerting system,                    format: log export }
  - { name: slack_threads,     source: incident channel + war-room dms,            format: text exports (in <untrusted_content> wrappers) }
  - { name: customer_impact,   source: support tickets + status-page entries,      format: markdown summary }

outputs:
  - { name: postmortem, format: postmortem@1, recipient: cuo/cto + cuo/cto's directs + cuo/ceo (if sev1) }
  - { name: action_items_in_tracker, format: linear/jira/github-projects tickets, recipient: impl team }

skill_chain:
  - { step: 1, skill: postmortem-author, inputs_from: { incident_timeline: incident_timeline, pager_logs: pager_logs, slack_threads: slack_threads, customer_impact: customer_impact }, outputs_to: postmortem_draft }
  - { step: 2, skill: postmortem-audit,  inputs_from: postmortem_draft, outputs_to: postmortem }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "severity == sev1 — sev1 always involves CEO via §4 COND-004 exec brief" }
  - { persona: cuo/chief-legal-officer,   when: "incident involved personal-data exposure — GDPR Art. 33 72h notification window applies (COND-002)" }
  - { persona: cuo/chief-communications-officer, when: "sev1/sev2 — public-comms log required (COND-001)" }
  - { persona: cuo/chief-information-security-officer,        when: "incident involved a security exploit (COND-003)" }
  - { persona: cuo/chief-ai-officer,        when: "incident touched an AI/ML system (COND-005 — model behaviour analysis)" }

consults:
  - { persona: cuo/chief-operating-officer,         when: "incident involved process-side failure modes (escalation paths, runbook gaps)" }

audit_hooks:
  - postmortem-author emits one artefact_write row
  - postmortem-audit emits one artefact_write row per iteration
  - workflow emits a workflow_complete row with the verdict (pass / needs_human / fail) + action-item count
  - QA-BLAME-001 fires trigger an immediate HITL halt for blameless-rewrite
---

# Blameless post-mortem — `chief-technology-officer/post-incident-review`

The CTO's discipline for post-incident analysis. Two-step skill chain (`postmortem-author` → `postmortem-audit`) but heavy on operator-side decisions: the audit RUBRIC's `QA-BLAME-001` rule will halt the chain immediately if the draft contains blameful language. Per Google SRE, blameless culture is the difference between an organisation that learns from incidents and one that buries them.

## When to invoke

CUO routes here when the user says things like:

- "We had a sev1 last night — write the post-mortem"
- "Drive the post-mortem for incident INC-1234"
- "Compile the blameless review for the auth-service outage"
- "Run post-mortem on yesterday's deploy that broke billing"

## How to invoke

```bash
cyberos-cuo run cuo/chief-technology-officer/post-incident-review \
  --input incident_timeline=./incidents/INC-1234/timeline.md \
  --input pager_logs=./incidents/INC-1234/pagerduty-export.log \
  --input slack_threads=./incidents/INC-1234/slack-export.txt \
  --input customer_impact=./incidents/INC-1234/impact-summary.md \
  --output-dir ./incidents/INC-1234/postmortem/
```

## Expected duration

- **Happy path (sev3/sev4):** 20–40 minutes.
- **Sev1/sev2 with all conditional sections firing** (public comms / GDPR / security disclosure / executive brief): 1–3 hours of skill-chain + escalation round-trips with CEO / CLO / CISO / CCO-Communications.
- **Blameful-rewrite cycle (QA-BLAME-001):** add 1-2 hours; the chain halts on each blameful-language match and the operator must re-draft.

## Skill chain — step by step

### Step 1: `postmortem-author`
- **What it does:** Authors the 12-section blameless post-mortem (per `postmortem-audit/RUBRIC.md` SEC-001..012) from the 4 input streams.
- **Inputs:** the 4 input artefacts (timeline, pager, slack, impact).
- **Outputs:** `postmortem_draft` — a `postmortem@1` markdown.
- **Pause point:** PLAN approval on which conditional sections fire (sev1/sev2 → public comms log; personal data → GDPR §14; security exploit → §15; sev1 → §16 exec brief; AI/ML → §17 model behaviour).

### Step 2: `postmortem-audit`
- **What it does:** Validates against `postmortem_rubric@1.0`. Most-likely-to-fire rules: QA-BLAME-001 (blameful language → error → needs_human), QA-TIMELINE-001 (timeline gaps >15 min), QA-ACTION-001/002 (action items without owner/due-date), QA-IMPACT-001 (vague customer impact).
- **Inputs:** `postmortem_draft`.
- **Outputs:** `postmortem` at 10/10.
- **Pause point:** any QA-BLAME-001 hit triggers HITL with the specific lines that read blamefully + a suggested rewrite tone.

## Failure modes — per step

| Step | Code | What happens | Recovery |
|---|---|---|---|
| 1 | BOOT-001 | One of the 4 input streams missing | Operator supplies what's available; document gaps in §6 contributing factors |
| 1 | HITL | Conditional-section trigger ambiguity (e.g. "did this expose personal data?") | Operator confirms; chain resumes |
| 2 | needs_human (QA-BLAME-001) | Blameful language found | Operator rewrites the offending lines blamelessly; re-run audit |
| 2 | needs_human (QA-ACTION-001/002) | Action items lack owner or due_date | Operator fills in; re-run audit |
| 2 | needs_human (QA-CVE-001) | Fabricated CVE | If COND-003 fires and the draft cites a CVE, validate the CVE ID against NVD/MITRE; if fabricated, remove |

## Operator-side decisions

The CTO is pulled in at:

1. **Severity classification at step 1 PLAN** — sev1/sev2/sev3/sev4/sev5 drives which conditional sections fire.
2. **Conditional-section trigger sign-off** — did personal data leak (COND-002)? Did a security exploit fire (COND-003)? Was AI/ML involved (COND-005)?
3. **Blameful-rewrite cycles** — the QA-BLAME-001 rewrite is the most operator-intensive part of any post-mortem. Don't rush it; blameful language ships forever.
4. **Action-item ownership** — every action item needs an owner + due date; the audit's QA-ACTION-001/002 enforce this.
5. **Public-comms approval** — for sev1/sev2, the status-page entry + customer email need CCO-Communications + CLO-Legal sign-off (the workflow `escalates_to:` these personas).

## Cross-references

- `../README.md` — CTO 9-block spec.
- `../../../docs/Software Development Process.md` §2(j) — Operations: incidents.
- `../../../docs/The C-Suite Reference.md` §5.3 — CTO output: post-mortems are an operational output.
- `../../../skill/postmortem-author/SKILL.md`, `../../../skill/postmortem-audit/RUBRIC.md` — per-skill specs.
- Google SRE Book — blameless post-mortem culture (the discipline this workflow encodes).
