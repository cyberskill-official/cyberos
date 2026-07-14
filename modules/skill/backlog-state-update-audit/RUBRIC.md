# backlog_state_update_rubric@2.0

constants: TOTAL_ROWS_MIN=8 (MUST FRs) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable, TASK-CUO-207)
families: BSU | BSU-INS
verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human

## Rules (prose -> rule mapping, TASK-SKILL-118 AC 2)

Every rule cites the prose gate it encodes. A rule without a prose source is itself a review finding (TASK-SKILL-118 §10 #1).

| rule_id | gate | prose source |
|---|---|---|
| `BSU-001` | new_status is in the closed 10-value lifecycle enum (STATUS-REFERENCE §1) | SKILL.md status clause (v2 fixed enum, TASK-CUO-205 era) |
| `BSU-002` | line_number resolves to a real FR row in the pre-image | SKILL.md line_number clause |
| `BSU-003` | old_line matches byte-for-byte (optimistic concurrency) | SKILL.md old_line clause |
| `BSU-004` | evidence_artefact_ids cross-reference real memory rows | SKILL.md evidence clause |
| `BSU-005` | mutation_kind is exactly one of {status-cell-only, insert-row} | SKILL.md @2 mutation_kind enum (TASK-CUO-205 §1 #1) |
| `BSU-INS-001` | insert-row: row absent in pre-image, present exactly once in post-image | TASK-CUO-205 §1 #4 / SKILL.md insert clause |
| `BSU-INS-002` | insert-row: format + (improvement) suffix byte-exact per regenerator grammar | TASK-CUO-205 §1 #4 |
| `BSU-INS-003` | insert-row: placed in the correct module section, FR-STEM sort order kept | TASK-CUO-205 §1 #4 + INSERT_ROW_CASES CASE-08 |
| `BSU-INS-004` | insert-row: no other line changed except that section's header counts | TASK-CUO-205 §1 #4 |
| `BSU-INS-005` | insert-row: status valid enum AND equals FR frontmatter status at write time | TASK-CUO-205 §1 #4 |

## Scoring

/10 overall. Start at 10; each open finding subtracts per severity (blocker -2, major -1, minor -0.5, rounded toward fail).
Only 10/10 passes. `needs_human` on structural ambiguity (unparseable artefact, contradictory sources) - never a guessed verdict.

## Changelog

- backlog_state_update_rubric@2.0: initial file-form of the gates already normative in SKILL.md prose (TASK-SKILL-118; no bar raised, no bar lowered).
