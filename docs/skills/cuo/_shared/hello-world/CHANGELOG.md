# CHANGELOG — `cuo/_shared/hello-world`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).

---

## v1.0.0 — 2026-05-05 (initial)

### Added

- `SKILL.md` — the simplest possible CyberOS skill. Demonstrates the full 27-field frontmatter contract on a trivially small body.
- `envelopes/input.json` — `{name, output_path}` schema.
- `envelopes/output.json` — `{skill_id, skill_version, output_path, greeting_sha256}` schema.
- `acceptance/golden-input.json` + `acceptance/golden-output-stephen.md`
  + `acceptance/golden-envelope.json` — regression-test fixtures.

### Why v1.0.0 not v0.1.0

This skill's behaviour is locked by its purpose (a teaching example). Future v2.0.0 would mean a different skill entirely — at which point we'd rename it. So we're starting at v1.0.0 to signal stability.

### Used as

The canonical first example in `cyberos/docs/skills/GETTING_STARTED.md` (Beginner section). When the guide says "your first skill", it means this folder.
