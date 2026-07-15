# `type: chore` — use the feature skeleton

**Load `templates/feature.md` and render it verbatim.** This file is a pointer, not a
skeleton. There is no template-include engine here — templates are read as literal
markdown — so `{{> feature.md }}` would render as that exact string. Follow the
pointer instead.

## Why this file exists at all

`task-author` HALTS when `templates/{type}.md` is missing (task-author/SKILL.md §4,
step W2), deliberately: a missing template must be loud, never resolved by a silent
fallback to `feature`. That rule is what stops a bug from being authored as a feature
with no reproduction and no regression test.

The cost of that rule is that the FM-108 enum and this directory must agree. No task
carries `type: chore` yet — this file exists so the first one does not halt the author
on a type the contract already admits.

## Why not its own skeleton

A chore is mechanical toil — regenerate, rotate, migrate, bump. It is feature-shaped
because even toil needs a scope with explicit non-goals and a definition of done; a
chore whose boundary nobody wrote down is how a dependency bump becomes a refactor.
The distinction lives in the `type` field, which is exactly what a discriminator is
for.

`rubrics/common.md` §2 is the whole gate for this type — no extra rule family. If
improvements ever need one, add `rubrics/improvement.md`; the dispatch already reads
it if present. Inventing rules for a shape nobody has complained about is how you get
a taxonomy nobody fills in correctly.

## If this ever stops being true

The moment a chore genuinely needs a section a feature does not, replace this pointer
with a real skeleton. Make that a deliberate act, not drift.
