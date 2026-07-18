---
id: TASK-IMP-117
title: The template teaches the violation it is linted for
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:10:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
service: tools/install
new_files:
  - tools/install/docs-tools/fm001-migrate.mjs
  - tools/install/tests/test_fm001_migrate.sh
modified_files:
  - tools/install/templates/TASK-TEMPLATE.md
  - tools/install/build.sh
  - tools/install/install.sh
  - docs/tasks
routed_back_count: 1
awh: N/A
---

# TASK-IMP-117 - the template teaches the violation it is linted for

FM-001 fires on 501 of 544 specs. The cause is not 501 authoring mistakes: it is line 2 of
`tools/install/templates/TASK-TEMPLATE.md`, which ships `id: TASK-<MODULE>-<NNN>  # module-scoped,
e.g. TASK-AUTH-001.` Every spec is born violating a rule the machine floor enforces, because the
file that teaches the format contradicts the file that checks it.

The rule is right. `id:` parses as `"TASK-X-001  # module-scoped, ..."` for any consumer that does
not strip trailing comments - the exact defect that put `priority: MUST  # MUST | SHOULD` into
TASK-AI-001..005's parsed values and wrote FM-001 in the first place. What was missing was a
mechanism: the lint existed, the template disagreed, and nothing ever ran the two against each
other.

The template is VENDORED. Every repo that installed CyberOS carries this copy, so every consumer
corpus has the same disease. A migrator that lives only in this repo fixes one of N corpora.

## Summary

Fix `TASK-TEMPLATE.md` so specs are born FM-001 clean, ship a migrator in the payload so any
installed repo can clean its own corpus, and migrate this repo's 501 non-conformant specs.

## Problem

FM-001 fires on 501 of 544 specs - 92% of the corpus. 149 of them are `status: done`: they passed
two human gates while violating a machine-floor rule. The cause is line 2 of the shipped template.
Every spec inherits the violation by construction, and because the lint was never run corpus-wide,
nobody found out. The template is vendored, so every consumer repo has the same disease.

## Proposed Solution

Move every trailing frontmatter comment in the template to its own line above its field, keeping
all guidance text. Ship `docs-tools/fm001-migrate.mjs` (stdlib, guarded, idempotent, `--check`) in
the payload so consumers can run it themselves. Migrate this corpus. The body is never touched, so
the 40 audit bindings survive.

## Alternatives Considered

- Narrow FM-001 to only the fields consumers parse (id/status/priority/type). Rejected: the rule
  was written in response to a real defect, and a rule narrowed to fit a broken corpus stops being
  a rule.
- Fix the template only, leave the 501. Rejected by the operator: the machine floor stays
  un-runnable repo-wide, which is exactly how this hid for the life of the project.
- Delete the template's annotations instead of moving them. Rejected: they are the template's
  teaching value; an author reading `type: feature` with no hint is worse off.
- Run the migrator across consumer repos from here. Rejected: shipping a tool the operator runs is
  a migration; reaching into another repo and rewriting 500 files is an incident.

## Success Metrics

- FM-001 count across `docs/tasks/*/*/spec.md` goes 501 -> 0.
- A spec authored fresh from the template lints clean with no hand-editing.
- All 40 `audited_body_sha256_prefix` values are unchanged after the migration.
- `dist/cyberos/docs-tools/fm001-migrate.mjs` is present, so a consumer repo can do the same.

## AI Authorship Disclosure

- Tools used: Claude (Fable 5) during the 2026-07-17 hardening run, via the CyberOS create-tasks
  and ship-tasks workflows.
- Scope: spec drafted by the agent from a defect it surfaced while advancing TASK-IMP-116 past its
  review gate. The 501/544 and 149-done figures are measured, not estimated. The migration scope
  (all corpora, including done specs, not just live ones) is the operator's recorded decision - the
  agent had proposed the narrower live-specs-only option and was overruled.
- Human review: @stephencheng at the ready_to_implement gate, and again at both HITL gates.

## Dependencies

None. FM-001 already exists in task-lint (TASK-IMP-084) and the `relUnderRoot` guard already exists
in docs-tools (TASK-IMP-109). This task adds no rule and invents no guard; it makes the template
obey a rule that has been on the books since the machine floor shipped.

## 1. Clauses

1.1 `TASK-TEMPLATE.md` frontmatter carries no trailing comments and no aligned continuation
comments. Every comment is own-line, above the field it documents. No guidance text is deleted -
the annotations are the template's teaching value and they move, they do not go.
Test: `t01_template_is_clean`

1.2 A new payload helper `docs-tools/fm001-migrate.mjs` rewrites trailing frontmatter comments to
own-line comments. It ships in the payload (build.sh vendor list) so any installed repo can run it
against its own corpus. Node stdlib only, exit-code discipline (0 clean/migrated, 2 usage), `--json`,
`--check` (report without writing).
Test: `t02_migrator_moves_trailing_comments`

1.3 The migrator NEVER splits on a `#` inside a quoted value. `title: "Fix the # parsing bug"` is
left byte-identical. A `#` that is not preceded by whitespace is not a comment.
Test: `t03_hash_inside_value_is_not_a_comment`

1.4 The migrator NEVER touches the body. Only the frontmatter block between the first two `---`
lines is in scope. This is what preserves `audited_body_sha256_prefix` on the 40 bound specs -
proven live twice this session: a status edit moves the file hash and leaves the body hash fixed.
Test: `t04_body_is_untouched_and_body_hash_holds`

1.5 Idempotent. Running the migrator twice produces a byte-identical file the second time.
Test: `t05_idempotent`

1.6 Migration of this repo's corpus: all 501 specs, including the 149 at `status: done`. Per the
operator's scope decision, done specs are migrated - the frontmatter is metadata about the task,
not the shipped record, and the body hash that binds the audit is untouched by 1.4.
Test: `t06_corpus_is_fm001_clean`

1.7 The migrator refuses to run outside a repo root it can confirm, using the same `relUnderRoot`
guard as task-reconcile / coverage-scope / verify-goals: confine, exist, `git ls-tree HEAD` tracked.
An untracked path is REFUSED, not migrated.
Test: `t07_guard_refuses_untracked_and_escaping_paths`

## Scope

In scope: `TASK-TEMPLATE.md`, the new `docs-tools/fm001-migrate.mjs` + its suite, the build.sh
vendor list, this repo's 544-spec corpus, and a note in the install summary telling a consumer repo
the migrator exists.

### Out of scope / Non-Goals

- Running the migrator against consumer repos. This ships the tool; the operator runs it per repo.
  An agent that reaches into another repo and rewrites 500 files is not a migration, it is an
  incident.
- FM-112's `# UNREVIEWED` markers. Moving a marker to its own line keeps it firing, correctly - a
  human still has not confirmed `ai_authorship`. Clearing FM-112 is a human attestation and this
  task does not forge it.
- The other 2 rules TASK-EVAL-001 trips (FM-004, FM-112). Out of cone.
- Any change to FM-001 itself. The rule is right; the template was wrong.

## 3. Edge case matrix

| # | Category | Trigger | Expected | Test |
|---|---|---|---|---|
| 1 | NULL/EMPTY | frontmatter with no comments | byte-identical no-op | t05 |
| 2 | NULL/EMPTY | file with no frontmatter block | refuse, exit 2, name the file | t07 |
| 3 | BOUNDS | comment is the whole line already | left alone, not doubled | t05 |
| 4 | BOUNDS | `#` at column 0 inside frontmatter | already own-line, untouched | t05 |
| 5 | MALFORMED | `#` inside a double-quoted value | NOT a comment, untouched | t03 |
| 6 | MALFORMED | `#` inside a single-quoted value | NOT a comment, untouched | t03 |
| 7 | MALFORMED | `#` with no leading whitespace (`a#b`) | NOT a comment, untouched | t03 |
| 8 | MALFORMED | list item `- path  # note` | comment moves above the item, indent preserved | t02 |
| 9 | MALFORMED | CRLF line endings | round-trip preserved, never normalized | t02 |
| 10 | MALFORMED | aligned continuation comment (template lines 6-9) | folded into the own-line block | t01 |
| 11 | CONCURRENT | two migrators, same file | two-phase atomic write, last wins, never truncated | t05 |
| 12 | SECURITY | path escapes the repo root | REFUSED, not executed | t07 |
| 13 | SECURITY | path exists but is untracked at HEAD | REFUSED - an untracked spec is not corpus | t07 |
| 14 | DEGRADATION | git absent / not a repo | refuse and say so; never migrate unguarded | t07 |
| 15 | DEGRADATION | file unreadable mid-run | report the file, exit non-zero, migrate nothing | t07 |

## 4. Out of scope / non-goals

See "## Scope -> ### Out of scope / Non-Goals" above - this section is the engineering half's pointer to it.

## Acceptance criteria

- AC1: `TASK-TEMPLATE.md` passes task-lint with zero FM-001. Cited: `t01_template_is_clean`.
- AC2: A spec authored fresh from the migrated template passes task-lint clean. Cited:
  `t01_template_is_clean`.
- AC3: The migrator satisfies 1.2-1.5 and 1.7. Cited: `t02`..`t05`, `t07`.
- AC4: `docs/tasks/*/*/spec.md` reports zero FM-001 corpus-wide. Cited: `t06_corpus_is_fm001_clean`.
- AC5: The 40 audit-bound specs keep their `audited_body_sha256_prefix` byte-for-byte across the
  migration. Cited: `t04_body_is_untouched_and_body_hash_holds`.
- AC6: `dist/cyberos/docs-tools/fm001-migrate.mjs` exists and matches source. Cited:
  `t08_payload_carries_it`.
