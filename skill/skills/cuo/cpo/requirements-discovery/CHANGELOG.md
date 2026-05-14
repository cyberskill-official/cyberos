# CHANGELOG — `cuo/cpo/requirements-discovery`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the skill level: MAJOR breaks the input/output envelope or the `project_brief@1` body shape; MINOR adds backwards-compatible fields, new optional behaviour, or new interview questions; PATCH is editorial.

---

## v0.1.0 — 2026-05-06 (initial scaffold)

### Added

- `SKILL.md` — entry. Full v0.2.0 frontmatter (33 fields). Owns the chain entry point: project-kind classification → triage → discovery interview → BRAIN reads → synthesis → amendment-batch → write.
- `CHANGELOG.md` — this file.
- `INVARIANTS.md` — 6 invariants. INV-001 (BRAIN-must-be-reachable; refuse if unreachable) is sev-0.
- `STANDALONE_INTERVIEW.md` — 20-question script (5 triage + 15 discovery). Project-kind-agnostic.
- `HUMAN_SUMMARY.md` — chat-rendered template covering brief written + triage verdict + amendments + open questions.
- `envelopes/requirements-discovery.input.json` — JSON Schema (1 required, 6 optional).
- `envelopes/requirements-discovery.output.json` — JSON Schema with `BRIEF_COMPLETE` / `HALTED_HITL` / `TRIAGE_REJECTED` outcomes.
- `acceptance/README.md` — priority scenarios pending v0.3.0 harness.

### Driver

User's request after registry v0.2.3 (verbatim): "the first inputs should be the BRAIN info itself, because i'll create new project and begin interact with it: so BRAIN + human inputs => PRD/SRS/other specs.... => cuo/cpo/fr-author". Identified the missing chain entry point. Q1-Q6 design-questions were answered in chat (recorded in registry CHANGELOG v0.2.4 driver section). v0.1.0 ships the scaffold; runtime in v0.3.0+.

### What this version DOESN'T do (intentionally)

- No executable runtime — gated on the harness build.
- No `AMENDMENT_PROTOCOL.md` reference doc — pattern described inline in SKILL.md; full doc lands at v0.2.0.
- No reference docs (HITL_PROTOCOL, UNTRUSTED_CONTENT, etc.) — land at v0.2.0; expect divergence from cpo siblings per REF-015.
- No PIPELINE.md worked example — pending one chained run against a real project idea.
- No `prd-author` chain (would default `chain_to: ['cuo/cpo/prd-author']`) — left unset because prd-author is itself a scaffold.

### Backwards compatibility

First version. No predecessor.

## How to add a future entry

Standard sub-sections:

- **Added** — new fields, new sections, new BOOT codes, new references/*.md docs, new interview questions.
- **Changed** — semantics changes that don't break the schema; interview-question phrasing updates.
- **Removed** — fields/questions deprecated.
- **Backwards compatibility** — what briefs from prior versions still validate.
- **Acceptance evidence** — pointer to the test artifact or run that validated the release.
