# CHANGELOG — `cuo/cto/spec-to-impl-plan`

> Format: Keep a Changelog 1.1.0.

---

## v0.1.0 — 2026-05-06 (initial scaffold)

### Added

- `SKILL.md` — full v0.2.0 frontmatter. Consumes either `tech_spec@1` (standard/full) OR audited `feature_request@1` (lean profile). Emits `impl_plan@1` markdown + optionally creates tickets via PROJ MCP.
- `CHANGELOG.md`.
- `INVARIANTS.md` — 4 invariants. INV-001 (refuse non-pass input) sev-0; INV-002 (no auto-create-without-approval) sev-0.
- `STANDALONE_INTERVIEW.md` — 2-3 sprint-planning questions.
- `HUMAN_SUMMARY.md`.
- `envelopes/{input,output}.json` — supports both standard/full input shape (tech_spec_path) and lean shape (fr_path + audit_path).
- `acceptance/README.md`.

### Driver

User said "implement spec-to-impl-plan" — Stage closing for the chain. Last skill before tickets land in PROJ MCP. The chain now goes end-to-end: human chat → BRAIN → brief → PRD → (SRS) → FRs → tech-specs → impl-plan + tickets.

### Backwards compatibility

First version.
