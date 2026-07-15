# `type: improvement` — use the feature skeleton

**Load `templates/feature.md` and render it verbatim.** This file is a pointer, not a
skeleton. There is no template-include engine here — templates are read as literal
markdown — so `{{> feature.md }}` would render as that exact string. Follow the
pointer instead.

## Why this file exists at all

`task-author` HALTS when `templates/{type}.md` is missing (task-author/SKILL.md §4,
step W2), deliberately: a missing template must be loud, never resolved by a silent
fallback to `feature`. That rule is what stops a bug from being authored as a feature
with no reproduction and no regression test.

The cost of that rule is that the FM-108 enum and this directory must agree. 215 tasks
carry `type: improvement` today; without this file every one of them would halt the
author on a type the contract explicitly admits.

## Why not its own skeleton

An improvement is a feature-shaped record: hardening, refactors, audit remediation and
dependency bumps still need a summary, a problem statement with evidence, success
metrics, a scope with explicit non-goals, and an edge-case matrix. The distinction
lives in the `type` field, which is exactly what a discriminator is for. Duplicating
eleven sections to change a label is how the two drift apart.

`rubrics/common.md` §2 is the whole gate for this type — no extra rule family. If
improvements ever need one, add `rubrics/improvement.md`; the dispatch already reads
it if present. Inventing rules for a shape nobody has complained about is how you get
a taxonomy nobody fills in correctly.

## If this ever stops being true

The moment an improvement genuinely needs a section a feature does not, replace this
pointer with a real skeleton. Make that a deliberate act, not drift.
