# TASK-IMP-089 gate-log draft (seed for audit.md §gate-log)

Recorded 2026-07-17 against the working tree on `batch/3-decided-items` (template edited, suite extended; dist/ untouched pending the batch parent's rebuild). All commands run from the repo root.

## Gating suite - scripts/tests/test_template_schema.sh (AC 1-3 + t01-t07 regression)

    bash scripts/tests/test_template_schema.sh

Output (verbatim, ANSI stripped; ~1.2 s wall including the scratch payload build):

      ok   t01
      ok   t02
      ok   t03
      ok   t04
      ok   t05
      ok   t06
      ok   t07
      ok   t08_single_out_of_scope_home
      ok   t08_duplicate_reintroduction_fails
      ok   t08_payload_carries_shape
    ----
    pass=10 fail=0

Exit code 0. t01-t07 all green (no existing scenario weakened); the three t08 arms are AC 1 / AC 2 / AC 3 in order.

## Shape greps (the facts t08a asserts, shown raw)

    grep -niE '^## +([0-9]+\. *)?out of scope' tools/install/templates/TASK-TEMPLATE.md
    # -> no output, exit 1 (zero out-of-scope H2s - clause 1.1)

    grep -nE '^## [0-9]+\.' tools/install/templates/TASK-TEMPLATE.md
    # -> 36:## 1. Description (normative)
    # -> 46:## 2. Acceptance criteria
    # -> 52:## 3. Edge cases
    # -> 58:## 4. Protected invariants this task must not weaken
    #    (numbered H2s run 1-4; nothing at 5 - clause 1.2 renumber complete)

    grep -c 'must never be made green by weakening' tools/install/templates/TASK-TEMPLATE.md
    # -> 1 (invariants body intact - "content unchanged" half of clause 1.2)

## Reference sweep (spec §3: update template-adjacent references only)

    grep -rn "Protected invariants" tools modules scripts README.md
    # -> tools/install/templates/TASK-TEMPLATE.md:58 only (the renumbered heading itself)

    grep -rni "4\. Out of scope\|5\. Protected" tools modules scripts
    # -> no hits (pre-change this returned TASK-TEMPLATE.md:58 and :62)

Adjudicated non-hits (left untouched by design): tools/install/tests/test_*.sh:2 headers citing "TASK-XXX-NNN §5 suite" reference those tasks' own historical specs (docs/tasks corpus, excluded by spec §3); tools/install/README.md:122/135 "### 4./### 5." number distribution channels, not template sections. Template-adjacent references updated: none needed.

## Diff surface

    git diff --stat
    # -> scripts/tests/test_template_schema.sh    | 61 ++++++++++++++++++++++++++++++++
    # -> tools/install/templates/TASK-TEMPLATE.md |  6 +---
    # -> 2 files changed, 62 insertions(+), 5 deletions(-)

    git status --porcelain
    # ->  M scripts/tests/test_template_schema.sh
    # ->  M tools/install/templates/TASK-TEMPLATE.md
    #    (plus this task folder's artefacts; no other repo file touched)

## Summary

AC 1 PASS (t08_single_out_of_scope_home) - AC 2 PASS (t08_duplicate_reintroduction_fails) - AC 3 PASS (t08_payload_carries_shape). Suite 10/10, exit 0; zero template-adjacent references needed updating; scratch payload byte-identical to source (cmp inside t08c).
