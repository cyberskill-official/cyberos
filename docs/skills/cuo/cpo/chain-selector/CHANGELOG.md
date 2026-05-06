# CHANGELOG — `cuo/cpo/chain-selector`

> Format: Keep a Changelog 1.1.0. SemVer at the skill level: MAJOR breaks the chain_plan output schema or removes a profile. MINOR adds a new profile or extends the selection-rule set additively. PATCH is editorial / clarification.

---

## v0.1.0 — 2026-05-06 (initial scaffold)

### Added

- `SKILL.md` — full v0.2.0 frontmatter. Reads `project_brief@1` frontmatter (contract dependency); emits chain_plan (no own contract — chain_plan is just a list of skill_ids).
- `INVARIANTS.md` — 4 invariants (deterministic selection from frontmatter; chain_plan size matches profile; no skipping the audit gate when `client_visible: true`; user-override is recorded with reasoning).
- `HUMAN_SUMMARY.md`.
- `envelopes/{input,output}.json`.
- `acceptance/README.md` — priority test scenarios.

### Selection rules

3-tier profile (`lean` / `standard` / `full`) with first-match-wins ordering. Rules detailed in SKILL.md §"Selection rules".

### Driver

User said "B: yes — chain-selector skill" in registry v0.2.7 design conversation. Closes the gap between "every project goes through the full chain" (overkill for small projects) and "no chain at all" (loses the audit gates). The 3-tier profile is the lean-vs-full negotiation.

### Backwards compatibility

First version. No predecessor.
