# TASK-IMP-099 code review

Reviewer: batch-4 ship-tasks sub-agent (serial owner of both ship-tasks.md tasks this round). Diff: modules/cuo/chief-technology-officer/workflows/ship-tasks.md (queue-selection line reworded + workflow_version 2.6.3 -> 2.6.4), tools/install/tests/test_workflow_helpers.sh (t12 AND t09 pins moved to 2.6.4, header comments corrected, t13_queue_rule_p0_p3 added).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | queue-selection prose ranks `p0` before `p1` before `p2` before `p3` with the FM-105 legacy-mapping parenthetical; no other ordering wording survives | ship-tasks.md:312 carries the reworded rule on one physical line; sentence tail (`created` ascending, id ascending, echo format) byte-intact; `grep -ci moscow` over the file = 1 (the parenthetical is the only mention, gate-log-draft.md E2) |
| 1.2 | workflow_version bumps to 2.6.4 and t12's exact pin moves in the same change, disclosed | frontmatter line 3 `workflow_version: 2.6.4`; t12 pin now `^workflow_version: 2\.6\.4$`; DISCLOSURE below covers the pin moves and the bump |
| 1.3 | suite asserts the scratch payload's distributed workflow carries the p0-p3 rule and no bare MoSCoW ordering rule (parenthetical exempted) | t13_queue_rule_p0_p3 ok over source AND $TMP/payload/cuo/ship-tasks.md via the existing ensure_payload; negative grep targets the rule shape (MoSCoW value on BOTH sides of "before", case-insensitive); probes recorded: retired wording CAUGHT, parenthetical allowed (E3) |
| AC 1 | source and scratch payload carry the p0-p3 rule; no bare MoSCoW ordering | t13 ok (suite tail 13/13, E1); payload cuo/ship-tasks.md line 312 carries the rule (E4) |
| AC 2 | version 2.6.4 pinned in source and payload | t12 ok (source + payload cuo/), t09 ok (source + payload cuo/ + plugin copy), t13 payload pin ok; direct greps in E4 |
| AC 3 | t01-t11 green | suite tail 13/13 (E1) - every pre-existing scenario passed; behavioral scenarios t01-t08, t10, t11 have zero diff |

## DISCLOSURE (spec §1.2 requires this to be explicit)

1. **t12 exact pin moved**: `^workflow_version: 2\.6\.3$` -> `^workflow_version: 2\.6\.4$` in tools/install/tests/test_workflow_helpers.sh, in the same change as the frontmatter bump - the exact-pin discipline is preserved (a version-agnostic regex was considered and spec-rejected; every future normative edit must again move the pin deliberately).
2. **t09 pin moved too (beyond the spec's letter)**: t09_doctrine_wiring carried the IDENTICAL exact pin (line 456 pre-change, undeclared in the spec's source_pages, discovered during context mapping). Moving only t12's pin ships a red t09 and violates AC 3 (t01-t11 green). Both pins moved together as one deliberate bump. This is the minimal deviation consistent with the spec's own guardrail; no behavioral assertion in t09 changed - only the version string and its failure message.
3. **Suite header comments corrected** for t09 ("2.6.4 since TASK-IMP-099") and t12, plus the new t13 entry - a comment asserting 2.6.3 above a test asserting 2.6.4 would be the lying- header shape this very suite polices.
4. **workflow_version 2.6.4** is this round's single bump, carried by this task by batch-4 plan; TASK-IMP-097 (same file, same round, same writer) deliberately shipped no bump.

## Judgment

- **Correctness vs authority**: the prose now teaches the scale FM-105 enforces (modules/skill/task-audit/RUBRIC.md:26) and modules/cuo/cuo/ship_manifest.py already ranks (`_PRIORITY_RANK` p0..p3 with the MUST/SHOULD/COULD/WONT legacy mapping) - doc, linter, and rank code agree for the first time; tool-driven selection behavior is unchanged.
- **Blast radius**: two files; the reword replaced one physical line; the sentence tail and echo format are byte-identical; t01-t11's behavioral fixtures and assertions untouched.
- **Failure mode if wrong**: a doc-driven agent could mis-rank legacy-priority tasks - guarded by the parenthetical pointing at FM-105's mapping and by rank code accepting both scales; reintroduction of MoSCoW ordering prose is now a named suite failure (t13 negative grep).
- **Pattern honesty**: the negative grep was probed both ways before shipping (retired wording caught; parenthetical and single-sided prose like step 27's "MUST be passed ... BEFORE this workflow" not caught) - evidence in gate-log-draft.md E3, not just asserted here.
- **Security**: queue ordering is an integrity surface for unattended runs; this change makes silent reordering harder (exact phrase pinned by t13, exact version pinned by three scenarios across all vendored copies). No execution surface, no secrets.
- **AI-specific**: the bump is deliberate, two-sided, and disclosed (rows 1-4 above); nothing in the diff is a silent behavioral change.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
