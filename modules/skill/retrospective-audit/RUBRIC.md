# `retro_rubric@1.0` — machine-checkable Retrospective rubric

> Sourced from `../../../modules/cuo/docs/module.md` Template §4.8 (Start/Stop/Continue + DORA Review). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `retrospective@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `iteration_id` | required, string (sprint name / project phase / quarter) | error | false |
| `FM-103` | `iteration_start`, `iteration_end` | required, ISO 8601 dates | error | false |
| `FM-104` | `facilitator` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-105` | `participants` | required, array of `^@[A-Za-z0-9_.-]{1,38}$` (>=2) | error | false |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required (typically the retro notes export) | error | false |
| `FM-107` | `team_mood` | required, integer 1-5 (group average) | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Team Mood` (the 1-5 number + brief narrative) | error |
| `SEC-002` | `## 2. DORA Metric Trends` (deployment frequency / lead time / change failure rate / failed-deployment recovery time — value, delta vs prior iteration, trend arrow) | error |
| `SEC-003` | `## 3. Continue` (≥3 things that worked — keep doing them) | error |
| `SEC-004` | `## 4. Stop` (≥3 things that didn't work — stop doing them) | error |
| `SEC-005` | `## 5. Start` (≥1 thing to begin trying) | error |
| `SEC-006` | `## 6. Action Items` (≤2 top actions with owner + due_date + linked_ticket) | error |
| `SEC-007` | `## 7. Wins to Celebrate` | warning |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Iteration included a sev1/sev2 incident | `## 8. Incident Reflection` linking the post-mortem(s) and any process changes adopted | error |
| `COND-002` | Iteration was a quarterly business review | `## 9. QBR-Specific Sections` (revenue / NPS / roadmap delta) | error |
| `COND-003` | Team adopted AI tools this iteration | `## 10. AI-Tooling Impact` (DORA delta with vs without AI — per SDP §5.6) | warning |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-BLAME-001` | Blameful language | Body contains personal-attribution complaints ("X dropped the ball", "Y didn't deliver") | warning → needs_human (`scope_decomposition`) — reframe systemically |
| `QA-ACTION-001` | Action item without owner | A row in §6 lacks `owner:` | error |
| `QA-ACTION-002` | Action item without due_date | A row in §6 lacks `due_date:` | error |
| `QA-ACTION-003` | More than 2 top action items | §6 has >2 actions (per Template §4.8's "top 1-2 actions") — escalate to needs_human or move surplus to §11 | warning |
| `QA-DORA-001` | DORA metric without value | A metric in §2 lacks a numeric value | error |
| `QA-DORA-002` | DORA metric without trend arrow | A metric in §2 lacks `trend: ↑/→/↓` | warning |
| `QA-MOOD-001` | Team mood out of range | `team_mood` not in 1-5 | error |
| `QA-CARRY-001` | No reflection on prior-iteration actions | Last iteration's retro had open action items; this retro doesn't mention them | warning |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |
| `XCHAIN-003` | Action items in §6 land in the linked project tracker (verified via tool link if available) | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source notes hash differs from `provenance.source_hash` | Reset open + needs_human | warning → needs_human |
| `STALE-002` | Action items past due_date not closed | warning (track separately) |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `../../../modules/cuo/docs/module.md` Template §4.8 + §5.6 — Retrospective + DORA review sources
