---
name: cyberos-improve-review
description: "[RETIRED] The separate improvement-review pass is retired. Review and acceptance now happen inside the ship-feature-requests workflow at the mandatory human-acceptance gates. Do not use this skill; see the body for where to go."
---
# cyberos-improve-review (RETIRED)

Retired on 2026-07-08. Review and acceptance are part of the single `ship-feature-requests` workflow now, at the two mandatory human-acceptance gates: review acceptance (`reviewing -> ready_to_test`) and final acceptance (`testing -> done`). A human records each verdict; the agent never self-sets `done`.

- The workflow: `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md` (§8 review, §10 acceptance, and the HITL section).
- HITL doctrine: `modules/cuo/EXECUTION-DISCIPLINE.md` §2a and `modules/skill/contracts/feature-request/STATUS-REFERENCE.md` §1.4.
