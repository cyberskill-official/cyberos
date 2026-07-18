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
installed repo can clean its own corpus, and migrate this repo's non-conformant specs. FM-001 has
TWO structural classes: TRAILING COMMENTS (the template defect above) and NESTED MAPS (a top-level
key whose value is an indented child map). The trailing-comment class was migrated corpus-wide at
commit 4c02b556; this task's migrator carries a SECOND capability that clears the nested-map
residual (4004 findings across 140 specs) plus one apostrophe edge, so FM-001 reaches 0 for real.

## Problem

FM-001 fires on 501 of 544 specs - 92% of the corpus. 149 of them are `status: done`: they passed
two human gates while violating a machine-floor rule. The cause is line 2 of the shipped template.
Every spec inherits the violation by construction, and because the lint was never run corpus-wide,
nobody found out. The template is vendored, so every consumer repo has the same disease.

That framing is only the FIRST of FM-001's two structural classes. After the trailing-comment class
was migrated corpus-wide (commit 4c02b556), a re-lint showed the count did NOT reach 0: a residual
of 4004 findings across 140 specs remained, plus 1 residual trailing-comment. The 4004 are a SECOND
class - nested-map frontmatter under a single top-level key, `build_envelope:`, whose indented
children the strict task@1 reader flags one line at a time as "indented line outside a block list".
No tool reads `build_envelope` (it is inert data), task-lint rejects nested maps by design, and a
`done` sibling (`TASK-SKILL-104`) already carries the same envelope data FLAT at top level - the
shape `batch-select` reads. The 1 residual is a defect in the migrator's own quote detector (a
mid-value apostrophe in `broker's` read as an opening quote), not in the lint. The full evidence,
with a command behind every number, is `docs/tasks/_audits/2026-07-18-fm001-nested-map-fork.md`.

## Proposed Solution

Move every trailing frontmatter comment in the template to its own line above its field, keeping
all guidance text. Ship `docs-tools/fm001-migrate.mjs` (stdlib, guarded, idempotent, `--check`) in
the payload so consumers can run it themselves. The migrator carries TWO capabilities: it moves
trailing comments own-line (class 1), and it FLATTENS a nested-map key by hoisting its children to
top-level keys (class 2), reconciling any collision by order-preserving union and halting on a
genuine scalar conflict. Migrate this corpus with both. The body is never touched, so the audit
bindings survive - and the bound specs carry neither class, so the migrator is a no-op on every one
of them.

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

- FM-001 count across `docs/tasks/*/*/spec.md` reaches 0 corpus-wide, across BOTH structural classes
  (trailing comments AND nested maps). The trailing-comment class went first (commit 4c02b556); this
  task's second capability clears the nested-map residual of 4004 findings across 140 specs plus the
  one apostrophe edge.
- A spec authored fresh from the template lints clean with no hand-editing.
- All 40 `audited_body_sha256_prefix` values are unchanged after the migration. The mechanism is
  §1.4 (body untouched) reinforced by disjointness: every audit-bound spec carries NEITHER class, so
  the migrator is a byte-for-byte no-op on it.
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

1.8 The migrator ALSO flattens a nested-map frontmatter key - a top-level key whose value is an
indented child block (e.g. `build_envelope:` followed by `  language: rust 1.81`), which the strict
task@1 reader rejects one line at a time as "indented line outside a block list". It hoists the
children to top-level keys: the child block is dedented by its own base indent and the parent line
dropped, so `  new_files:` / `    - x` become top-level `new_files:` / `  - x`. This is GENERAL to
any nested-map key per FM-001's definition, not hard-coded to `build_envelope`. Each child value and
its order are preserved byte-for-byte (only leading indentation changes); a child that is itself a
block list (`new_files:` / `modified_files:` with `- item` lines) becomes a top-level block list. A
child block that is ALREADY a plain block list (all `- item`, e.g. `source_pages:`) is not a nested
map and is left untouched. When a hoisted key already exists at top level the migrator RECONCILES by
order-preserving union - two block lists merge, exact-duplicate item values deduped, nothing unique
dropped; two scalars of equal value dedupe to one - and NEVER silently drops or overwrites. A
genuine scalar conflict, or a list/scalar kind mismatch, HALTS that file: it is named, and nothing
in it is migrated.
Test: `t09_flattens_nested_map`

1.9 A quote is a string delimiter ONLY when it BEGINS the scalar value. In a PLAIN (unquoted) scalar
a mid-token apostrophe is a literal character, so a block item such as `- allow ... broker's ... (per
§1 #4 - seal stdin/stdout/stderr only)` correctly detects the ` #4` as a trailing comment and moves
it own-line - matching task-lint, which reads `broker's` as a literal apostrophe and ` #` as a
plain-scalar comment. A `#` inside a value that BEGINS with a quote (`label: 'issue # 42 stays'`,
`title: "Fix the # bug"`) stays data. This corrects a defect in the 1.3 detector, which entered
single-quote state on the mid-value apostrophe and so missed the comment
(`docs/tasks/_audits/2026-07-18-fm001-nested-map-fork.md` §5: task-lint's quote model is right, the
migrator's was wrong). No existing clause is weakened: 1.3's quoted-value protection (edges #5/#6) is
preserved exactly - a value that BEGINS with a quote is still honoured.
Test: `t10_apostrophe_then_hash_is_a_comment`

## Scope

In scope: `TASK-TEMPLATE.md`, the new `docs-tools/fm001-migrate.mjs` (BOTH capabilities - the
trailing-comment move and the nested-map flatten, plus the §1.9 quote fix) + its suite, the build.sh
vendor list, this repo's corpus (trailing-comment class already migrated at 4c02b556; this task
clears the nested-map residual across 140 specs and the one apostrophe edge), and a note in the
install summary telling a consumer repo the migrator exists.

### Out of scope / Non-Goals

- Running the migrator against consumer repos. This ships the tool; the operator runs it per repo.
  An agent that reaches into another repo and rewrites 500 files is not a migration, it is an
  incident.
- FM-112's `# UNREVIEWED` markers. Moving a marker to its own line keeps it firing, correctly - a
  human still has not confirmed `ai_authorship`. Clearing FM-112 is a human attestation and this
  task does not forge it.
- The other 2 rules TASK-EVAL-001 trips (FM-004, FM-112). Out of cone.
- Any change to FM-001 itself. The rule is right for BOTH classes; only the template (class 1) and
  the specs (class 2) were wrong. Specifically, RELAXING FM-001 to accept nested maps (the
  investigation's route b) is REJECTED: nested maps are outside the documented task@1 subset, no tool
  reads `build_envelope`, and task-lint rejects them by design - accepting them would be a `task@2`
  schema bump, not a lint fix. This task takes route (a): migrate the nested maps to the flat shape a
  `done` sibling already uses. The lint is unchanged.

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
| 16 | NESTED MAP | `build_envelope:` with 6 map children | 6 top-level keys; values + order preserved; FM-001 cleared | t09 |
| 17 | NESTED MAP | top-level `new_files` + `build_envelope.new_files`, one item shared | order-preserving union; exact dup deduped; nothing dropped; no FM-003 | t09 |
| 18 | NESTED MAP | a `done` spec carrying `build_envelope` | flattened; body untouched, so `audited_body_sha256_prefix` holds | t04, t09 |
| 19 | MALFORMED | hoisted key collides as a scalar with a different value | HALT; name the file; migrate nothing in it | t09 |
| 20 | MALFORMED | plain scalar `broker's ... #4` (apostrophe then ` #`) | ` #4` detected as a comment and moved own-line | t03, t10 |
| 21 | NESTED MAP | flatten output re-run through the migrator | byte-identical no-op (idempotent across both passes) | t05, t09 |

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
- AC7: The 140 nested-map (`build_envelope`) specs are flattened - their children hoisted to
  top-level keys, values and order preserved, collisions unioned - and the corpus FM-001 count is 0
  INCLUDING them. Cited: `t09_flattens_nested_map` (capability, scratch corpus) plus a real-corpus
  check: `node tools/install/docs-tools/task-lint.mjs --json docs/tasks`, parsing the JSON array and
  filtering `rule_id === "FM-001"`, reports length 0 (before: 4005 = 4004 nested-map + 1 apostrophe).
  AC4's "FM-001 = 0 corpus-wide" is now genuinely reachable because this second capability clears the
  class AC1-AC6's trailing-comment migrator never modelled.
