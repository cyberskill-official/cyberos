---
template: stage-gate@1
title: <Project> — Stage <letter> Gate
project: <Project name>
stage_name: a    # one of: a, b, c, d, e, f, g, h, i, j, k, l, m  (per SDP §2)
# stage_custom: true   # set when stage_name is a free string outside SDP §2
gate_date: 2026-MM-DD
gate_version: 1.0.0
provenance:
  source_path: ./project-plan.md
  source_hash: sha256:<hash>
decision: go    # go | go_with_conditions | no_go | deferred
decision_recorded_at: 2026-MM-DDTHH:MM:SS+07:00
signers:
  - { handle: "@<em>",       role: "EM",             signed_at: "2026-MM-DDTHH:MM:SS+07:00" }
  - { handle: "@<tl>",       role: "TL",             signed_at: "2026-MM-DDTHH:MM:SS+07:00" }
  - { handle: "@<sponsor>",  role: "Client_Sponsor", signed_at: "2026-MM-DDTHH:MM:SS+07:00" }
linked_project_plan: ./project-plan.md
---

# <Project> — Stage <letter> Gate

## 1. Stage

<SDP §2(<letter>) stage name — e.g. "(b) Requirements gathering and analysis">.

## 2. Entry Criteria — Met?

| # | Criterion | Met? | Evidence |
|---|---|---|---|
| 1 | <Criterion> | Y/N | <link> |

## 3. Exit Criteria — Met?

| # | Criterion | Met? | Evidence |
|---|---|---|---|
| 1 | <Criterion> | Y/N | <link> |

## 4. Open Risks and Issues

| RAID id | Description | Likelihood | Impact | Mitigation | Owner |
|---|---|---|---|---|---|

## 5. Decision

**Decision:** <go | go_with_conditions | no_go | deferred>

**Rationale:** <required paragraph; cite gate-meeting minutes>.

## 6. Conditions

<!-- Required only when decision = go_with_conditions; otherwise delete this section. -->

| # | Condition | Owner | Due |
|---|---|---|---|

## 7. Signatures

| Role | Signer | Signed at |
|---|---|---|
| EM | @<em> | <ts> |
| TL | @<tl> | <ts> |
| Client Sponsor | @<sponsor> | <ts> |

<!-- ── Conditionally-required sections (uncomment per COND-002..004) ── -->
<!-- ## 8. Remediation Plan      — required when decision = no_go -->
<!-- ## 8. Deferral Reason       — required when decision = deferred -->
