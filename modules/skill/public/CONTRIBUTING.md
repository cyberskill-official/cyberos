# Contributing

1. **Open an issue first.** Use the [skill-proposal template](.github/ISSUE_TEMPLATE/skill-proposal.md) for new skills or the [bug template](.github/ISSUE_TEMPLATE/bug.md) for defects.
2. **Conform to the open Agent Skills spec.** `SKILL.md` at the skill root with YAML frontmatter (`name`, `description`, `license`, `metadata`) and a Markdown body.
3. **Ship fixtures.** A `tests/fixtures.json` corpus covering happy path, failure modes, and Vietnam-specific edge cases is required for every skill.
4. **Cite specific Vietnamese instruments.** Reference decrees and circulars by number (e.g. "Nghị định 13/2023/NĐ-CP"), not by topic.
5. **Apache 2.0 only.** By opening a PR you agree your contribution is licensed under Apache-2.0.
6. **CI must pass.** `.github/workflows/validate.yml` validates frontmatter and fixtures.
