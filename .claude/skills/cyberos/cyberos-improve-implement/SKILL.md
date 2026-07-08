---
name: cyberos-improve-implement
description: "[RETIRED] The separate improvement-implement loop is retired. Improvement work is now a feature-request (class: improvement) driven by the ship-feature-requests workflow. Do not use this skill; see the body for where to go."
---
# cyberos-improve-implement (RETIRED)

Retired on 2026-07-08. CyberOS runs a single implementation workflow now: `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md`.

Improvement and hardening tasks are feature-requests carrying `class: improvement`; they run the same lifecycle as any FR, with HITL required at the two human-acceptance gates (`reviewing -> ready_to_test` and `testing -> done`). There is no separate implement loop.

- Run work via `ship-feature-requests` (section 1a covers improvement FRs and their gate profile).
- Improvement-class home and backlog migration: `docs/feature-requests/improvement/README.md`.
- Halt and HITL doctrine: `modules/cuo/EXECUTION-DISCIPLINE.md` §2a.
