---
template: closure@1
title: <project> — closure package
project: <project name>
client: <client name>
closure_date: 2026-MM-DD
closure_version: 1.0.0
linked_sow: ./sow.md
provenance: { source_path: ./sow.md, source_hash: sha256:<hash> }
client_nps: 9    # 0-10
signers:
  - { handle: "@<sponsor>", role: "Client_Sponsor", signed_at: "2026-MM-DDTHH:MM:SS+07:00" }
  - { handle: "@<em>",      role: "EM",             signed_at: "2026-MM-DDTHH:MM:SS+07:00" }
  - { handle: "@<tl>",      role: "TL",             signed_at: "2026-MM-DDTHH:MM:SS+07:00" }
---

# <project> — closure package

## 1. Sign-Off Certificate
Formal client + CyberSkill acceptance text + signature table.

## 2. Deliverables Accepted
| # | Deliverable (per SOW §3) | Accepted at | Accepted by |
|---|---|---|---|

## 3. Lessons Learned
Compiled from per-iteration retros.

## 4. Knowledge Transfer
What was handed over; to whom; in what form.

## 5. Source-Code Handover
| repo | branch | tag | access transferred to |
|---|---|---|---|

## 6. Runbook and Operations Handover
| runbook artefact | on-call transition plan |
|---|---|

## 7. Credentials Rotation
| credential | rotated_at | new holder |
|---|---|---|

## 8. Asset Handover
Designs, contracts, third-party licenses, vendor accounts.

## 9. Closure Metrics
| metric | value |
|---|---|
| on-time delivery % | ... |
| on-budget %        | ... |
| defect leakage     | ... |
| DORA at closure    | freq=..., lead=..., CFR=..., MTTR=... |

## 10. Client NPS and Verbatim Feedback
NPS: <0-10>. Verbatim: "...".

## 11. Surviving Obligations
Warranty, support, NDA, IP, audit-rights.

## 12. Next-Steps Proposal
Renewal / phase-2 / referenceability discussion.

<!-- ## 13. People Offboarding              — when dedicated_team / staff_aug -->
<!-- ## 14. Data Disposition                — when personal-data processing -->
<!-- ## 15. Warranty Expiry Notice          — when warranty about to expire -->
<!-- ## 16. Service Disengagement Plan      — when managed_services -->
