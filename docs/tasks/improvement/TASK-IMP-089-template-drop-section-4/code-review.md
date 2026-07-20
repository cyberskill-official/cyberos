# TASK-IMP-089 — code review packet

Files under review: modified `tools/install/templates/TASK-TEMPLATE.md` (section-4 removal + renumber, +1/−5) and `scripts/tests/test_template_schema.sh` (t08 block: shared shape oracle, three arms, TMP harness line, runner line — +61/−0). Exactly the spec's `modified_files`, no new files. Suite state at review: test_template_schema 10/10 (t01–t07 untouched and regression-green, three t08 arms green), ~1.2 s including the scratch payload build. Working tree carries no other dirt (git status: only these two files); dist/ deliberately untouched — rebuild, version-sync and full suite before commit are the batch parent's step per payload-sync doctrine.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | TASK-TEMPLATE.md MUST NOT contain a `## 4. Out of scope / non-goals` section; `## Scope > ### Out of scope / Non-Goals` remains the single home | diff: the 4-line block (heading, blank, `- ...`, blank) is gone; template tail now runs `## 3. Edge cases` → `## 4. Protected invariants…`. Gated forever by `t08_single_out_of_scope_home` via `shape_why`, whose regex `^## +([0-9]+\. *)?out of scope` (case-insensitive) also catches a duplicate re-added at another number or unnumbered; the PRD H3 home is exempt by construction (`^## ` cannot match `###`) and lives untouched in the per-type templates / authored specs |
| 1.2 | Protected-invariants section renumbered to `## 4.` with content unchanged | diff shows exactly one changed line (`## 5.` → `## 4.`); the two body lines are byte-identical (git diff context). Gated: exact-literal heading count == 1 (`invariants-not-at-##4` token), stray-`## 5.` probe, and the body probe `must never be made green by weakening` (`invariants-body-missing` token) in t08a |
| 1.3 | test_template_schema.sh MUST assert the new shape: no duplicate out-of-scope heading, invariants present at section 4 | t08a asserts both on every run; `t08_duplicate_reintroduction_fails` proves the assert has teeth — an awk fixture re-adds the retired section 4 above the invariants and the arm demands the `duplicate-out-of-scope-H2` token SPECIFICALLY, so a decayed oracle failing for an unrelated reason cannot impersonate detection |
| 1.4 | rebuilt payload MUST carry the updated template | `t08_payload_carries_shape` — scratch `bash tools/install/build.sh "$TMP/payload"`, then `[ -f payload/cuo/templates/TASK-TEMPLATE.md ]` (named path, t07's lesson), shape oracle on the vendored copy, and `cmp -s` byte-parity against source (build.sh:36 vendors `templates/.` verbatim, so parity is the exact contract). Existing vendor gates keep covering dist/ after the batch parent's rebuild |

## Acceptance criteria

AC 1 `t08_single_out_of_scope_home` ok · AC 2 `t08_duplicate_reintroduction_fails` ok · AC 3 `t08_payload_carries_shape` ok. Full suite 10/10, 0 failed.

## Reference sweep (spec §3 "downstream prose citing section 5 invariants")

grep over tools/, modules/, scripts/, README.md for "Protected invariants", numbered section-4/5 headings and out-of-scope H2s: the ONLY hits were TASK-TEMPLATE.md's own two headings (now fixed). Adjudicated non-hits: `tools/install/tests/test_*.sh:2` headers cite "TASK-XXX-NNN §5 suite" — those tasks' own historical spec sections (docs/tasks corpus, excluded by spec §3); `tools/install/README.md:122/135` "### 4./### 5." number distribution channels. Zero template-adjacent references required updating.

## Diff size

2 modified files, +62/−5 total: TASK-TEMPLATE.md +1/−5 (block removal + renumber, nothing else touched — no pointer line added, per scope), test_template_schema.sh +61/−0 (WHY block, `shape_why`, three arms, one mktemp/trap line, one runner line). No new files, no dependency added, per-type templates and RUBRIC.md untouched (the rubric never required section 4).

## Verdict

| Check | State |
|---|---|
| §1 clauses 1.1–1.4 | each proven above by a named test or diff fact |
| Primary metric (spec from new template carries one home; schema test asserts shape every run) | pass (t08a runs in every suite sweep via run_all.sh:43's scripts/tests glob) |
| Guardrail metric (payload template matches source byte-for-byte after rebuild) | pass on scratch build (t08c `cmp -s`); dist/ parity lands with the batch parent's rebuild |
| Edge-case matrix rows 1–10 | every row's covered-by names a live function; t08b is the oracle's own canary |
| Invariants (existing specs untouched; corpus never linted by t08; both shapes stay rubric-valid; HITL gates intact) | intact — git status shows only the two spec-declared files |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
