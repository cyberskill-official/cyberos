# CHANGELOG — `cuo/cpo/prd-author`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the skill level.

---

## v0.1.0 — 2026-05-06 (initial scaffold)

### Added

- `SKILL.md` — full v0.2.0 frontmatter (33 fields). Input contract `project_brief@1`; output contract `prd@1`; wire-protocol contract `nats_subjects@1`.
- `CHANGELOG.md` — this file.
- `INVARIANTS.md` — 7 invariants. INV-001 (refuse rejected briefs) is sev-0; INV-002 (no llm-implicit on Goals) is sev-0.
- `STANDALONE_INTERVIEW.md` — 3-5 follow-up question script for PRD-specific decisions.
- `HUMAN_SUMMARY.md` — chat-rendered batch-completion template.
- `envelopes/prd-author.input.json` — JSON Schema (2 required, 5 optional + `proceed_despite_revise` flag).
- `envelopes/prd-author.output.json` — JSON Schema with `PRD_COMPLETE` / `HALTED_HITL` / `REFUSED_REJECTED_BRIEF` / `REFUSED_REVISE_NEEDS_OVERRIDE` / `EXHAUSTED` / `USER_ABORTED` outcomes.
- `acceptance/README.md` — priority test scenarios pending v0.3.0 harness.

### Driver

User's request after registry v0.2.3: a chain entry point for new projects (BRAIN + human → PRD → fr-author). v0.2.4 ships the chain entry point (`requirements-discovery`) + this skill (`prd-author`) which consumes the brief and produces the PRD.

### What this version DOESN'T do (intentionally)

- No executable runtime — gated on the harness build.
- No `AMENDMENT_PROTOCOL.md` reference doc — at v0.2.0.
- No reference docs — at v0.2.0; expect divergence from cpo siblings per REF-015.
- No PIPELINE.md worked example — pending a chained run.
- No `next_skill_recommendation: cuo/cpo/prd-audit` (because prd-audit doesn't exist until v0.2.5).

### Backwards compatibility

First version. No predecessor.

## How to add a future entry

Standard sub-sections.
