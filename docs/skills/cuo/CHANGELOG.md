# CHANGELOG — `cuo/` namespace

> Persona-namespace history. Tracks which sub-personas exist, which are gated, and which workflows they own.

---

## v0.1.0 — 2026-05-05 (initial CUO namespace)

### Added

- `cuo/README.md` — namespace index. Lists all 14 sub-personas (DEC-052), routing rules, persona-card contract, `_shared/` policy, phase availability.
- `cuo/cpo/` — first persona-card folder (Chief Product Officer). P0 persona; owns FR backlog management workflows.
- `cuo/_shared/feature-request-template/` — first cross-persona shared skill; holds the `feature_request@1` schema body.

### Notes

Persona folders for `ceo`, `coo`, `cfo`, `cmo`, `cto`, `chro`, `cseco`, `clo`, `cdo`, `caio`, `cxo`, `cro`, `cso-sustainability` are intentionally absent. They will be created lazily as their first workflow lands. PRD §14 phase gating prevents premature routing.

## How to add a future entry

For each release, prepend `## vX.Y.Z — <ISO date> (<one-line summary>)`. Standard sub-sections:

- **Added** — new persona folders, new shared skills, new routing rules.
- **Changed** — voice/scope changes to existing persona cards.
- **Promoted** — persona moved from `gated_until_phase: P<n>` → operational.
- **Deprecated** — persona scheduled for retirement (rare).
