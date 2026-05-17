# `happiness_program_rubric@1.0` — machine-checkable Happiness Program rubric

> Per Session-C Tier-3 rebuild (2026-05-17). Sourced from C-Suite Reference §5.7 niche personas.

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `happiness-program@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1-160 chars after trimming | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `provenance.source_path` | required, path exists, readable | error | false |
| `FM-104` | `provenance.source_hash` | required, matches `^[0-9a-f]{64}$` | error | false |

## §3  Always-required sections

Per the `contracts/happiness-program/template.md` H2 list.

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SEC-001` | Every H2 declared in template present in authored artefact | error |
| `SEC-002` | Each required H2 is non-empty (≥1 non-blank line) | error |
| `SEC-003` | Section ordering matches the template | warning |
| `SEC-004` | Heading hierarchy well-formed | warning |

## §4  Conditionally-required sections

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `COND-001` | When trigger condition fires per template metadata, the conditional section MUST be present | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | error |
| `QA-NUM-001` | Unsourced numeric target | error → needs_human |
| `QA-VAGUE-001` | Vague placeholder (`TBD`, `TODO`, `<placeholder>`) | warning |
| `QA-OWNER-001` | Action item without owner | error |
| `QA-DUE-001` | Action item without due date | error |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` blocks | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3 matches) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source artefact hash differs from `provenance.source_hash` | Reset open + needs_human issues to open | warning → needs_human |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `cyberos/skill/contracts/happiness-program/CONTRACT.md` + `template.md`
- `cyberos/docs/The C-Suite Reference.md` §5.7 — niche persona role profile.
- Skill-specific FM-105+ + QA-NNN rules added in v1.1 per fine-tune discipline.
