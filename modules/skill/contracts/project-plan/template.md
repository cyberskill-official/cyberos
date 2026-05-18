---
template: project-plan@1
title: <Project> — Project Plan
author: @<engagement-manager>
project: <Project name>
linked_srs: ./srs.md
linked_sow: ./sow.md
engagement_model: fixed_price
plan_version: 1.0.0
effective_date: 2026-MM-DD
target_close_date: 2026-MM-DD
provenance: { source_path: ./srs.md, source_hash: sha256:<hash> }
governance_framework: hybrid_lite    # pmbok_8 | prince2_7 | hybrid_lite | none
---

# <Project> — Project Plan

## 1. Technical Feasibility Memo
<Tech spikes summary, build-vs-buy, key risks.>

## 2. Cost/Benefit Analysis
<Cited numbers.>

## 3. Schedule and Milestones
| # | Milestone | Target date | Acceptance gate | Depends on |
|---|---|---|---|---|

## 4. RAID Log
| id | type (R/A/I/D) | description | owner | due | likelihood | impact | mitigation |
|---|---|---|---|---|---|---|---|

## 5. Communication Plan
Daily standup (internal); weekly client status; fortnightly demo; monthly steering; QBR.

## 6. Resourcing Plan
| Person | Role | FTE % | Start | End | PTO assumptions |
|---|---|---|---|---|---|

## 7. Quality Plan
Test approach summary + acceptance-gate strategy. References `test-strategy@1` once created.

## 8. Definition of Ready / Done
References `definition-of-ready-and-done@1` at `./dor-dod.md` (or inlined here for small engagements).

## 9. Change-Control Process
<How scope changes are proposed, approved, priced.>

## 10. Approval and Sign-off
| Role | Approver | Approved at |
|---|---|---|

<!-- ## 11. PMBOK Performance Domains Mapping       — required when governance_framework: pmbok_8 -->
<!-- ## 11. PRINCE2 Elements Mapping                — required when governance_framework: prince2_7 -->
<!-- ## 12. Stage-Gate Plan                         — required when engagement_model: fixed_price -->
<!-- ## 13. Regulatory Compliance Plan              — required when project is regulated -->
