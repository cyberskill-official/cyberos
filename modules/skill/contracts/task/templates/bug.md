---
id: TASK-<MODULE>-<NNN>
title: "<one line: the observable wrong behaviour, not the suspected cause>"
template: task@1
type: bug
module: <module>
author: "@<handle>"
department: engineering
status: draft
priority: p0 | p1 | p2 | p3
severity: sev1 | sev2 | sev3 | sev4      # BUG ONLY — impact if left unfixed. Distinct from priority.
created_at: <ISO 8601 with timezone>
ai_authorship: none | assisted | co_authored | generated_then_reviewed
eu_ai_act_risk_class: not_ai | minimal | limited | high
client_visible: true | false
depends_on: []
blocks: []
first_bad_commit: <sha> | null           # BUG ONLY — from `git bisect`, when known
regression_test: <path::testname>        # BUG ONLY — REQUIRED. Must be red at HEAD~ and green at HEAD.
incident: <link> | null                  # BUG ONLY — required when severity is sev1
---

# {title}

> **Bug template (`type: bug`).** A bug is not a small feature. It is a claim that
> the system does something other than what it promised, and the burden of proof is
> different: you must be able to *make it happen on demand* before you are allowed
> to claim you fixed it.
>
> Rule family `BUG-*` in `modules/skill/task-audit/RUBRIC.md` §10 gates this template.

## Reproduction

Deterministic steps. If a reader cannot make the bug happen by following these, the
task is not ready to leave `draft` (BUG-001).

```
1.
2.
3.
```

**Environment**: <os / runtime / service version / config that matters>
**Frequency**: always | intermittent (<n> in <m> attempts)

## Expected vs observed

| | |
|---|---|
| **Expected** | <what the system promised — cite the clause, spec or contract that promises it> |
| **Observed** | <what it actually did — paste the error, the wrong value, the trace> |

Stating these separately is not ceremony. It is how you find the bugs where the
*expectation* was wrong (BUG-002).

## Blast radius

- **Who is affected**: <tenants / users / services>
- **Since when**: <version or commit — see `first_bad_commit`>
- **Workaround**: <exists / none>
- **Data integrity**: <is anything already persisted wrong? this decides whether a
  fix is enough, or whether a backfill is also owed>

## Root cause

The mechanism, not the symptom. "The request 500s" is a symptom. "The connection
pool is exhausted because `close()` is skipped on the error path in `foo.rs:214`"
is a cause.

BUG-003 rejects a root cause that merely restates the observed behaviour.

## Fix

<what changes, and why that change makes the cause impossible rather than unlikely>

## Regression test

The bug analogue of the edge-case matrix, and the reason this template exists.

```
<path::testname>
```

This test MUST:

1. **fail** when checked out at `first_bad_commit` (or `HEAD~` if unknown), and
2. **pass** at `HEAD`.

Both halves are machine-checkable, and `coverage-gate-audit`'s `REGRESSION-*` family
runs them (see RUBRIC §10). A test that passes before the fix proves nothing — it
does not test the bug. This is the single rule that stops "fixed" from meaning
"the symptom went away while I was looking at it".

## Edge cases

Scope the matrix to the *cause's neighbourhood*, not the whole feature. If the cause
is a missing `close()` on an error path, enumerate the other error paths — not the
happy path.

| category | trigger | covered by |
|---|---|---|
| | | |

## Prevention

Why did this reach production? What class of bug is it? What would have caught it?
Answer honestly, or `postmortem-author` will have to (required for `sev1`).
