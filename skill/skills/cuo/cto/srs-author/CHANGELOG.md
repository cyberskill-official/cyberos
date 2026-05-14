# CHANGELOG — `cuo/cto/srs-author`

> Format: Keep a Changelog 1.1.0.

---

## v0.1.0 — 2026-05-06 (initial scaffold)

### Added

- `SKILL.md` (full v0.2.0 frontmatter; deps on `prd@1`, `srs@1`, `nats-subjects@v1`).
- `CHANGELOG.md`.
- `INVARIANTS.md` — 5 invariants. INV-001 (refuse non-pass PRDs) sev-0; INV-002 (no llm-implicit on Architecture claims) sev-0.
- `STANDALONE_INTERVIEW.md` — 5-7 architectural questions.
- `HUMAN_SUMMARY.md`.
- `envelopes/{input,output}.json`.
- `acceptance/README.md`.

### Driver

User said "do all stages" — Stage C. SRS authoring sits between audited PRD and tech-spec authoring, providing the architectural-review seam.

### Backwards compatibility

First version. No predecessor.
