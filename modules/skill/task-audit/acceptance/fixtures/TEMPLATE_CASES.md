# Task-audit - template-detection cases (TASK-CUO-208 AC 3/4/5/7)

Executable case table; fixtures inline below (each case: detection -> families -> expected verdict).

| case | fixture | detected | expected |
|---|---|---|---|
| TC-01 | fr1-missing-alternatives (below) | task@1 (template key) | fail SEC-004 |
| TC-02 | engspec-missing-s10 (below) | engineering-spec@1 (§ grammar) | fail §12 structural (missing §10) |
| TC-03 | ambiguous-both-markers (below) | BOTH | needs_human `template_ambiguous` naming the conflict |
| TC-04 | mixed batch = TC-01 file + any repo engineering-spec exemplar | per file | each judged by own families; one audit report per file |
| TC-05 | same interview authored twice (author AC 3) | n/a | engineering-spec output carries §1..§11 + end marker; task output carries SEC-001..007 sections + FM fields |

## fixture: fr1-missing-alternatives (frontmatter fragment + sections)

```markdown
---
template: task@1
title: Example
---
## Summary
x
## Problem
x
## Proposed Solution
x
## Success Metrics
x
## Scope
x
## Dependencies
x
```
(No `## Alternatives Considered` -> SEC-004.)

## fixture: engspec-missing-s10

```markdown
---
id: TASK-X-001
---
## §1 - Description
1. MUST x.
## §2 - Why this design
...sections §3-§9, §11 present, §10 absent...
*End of TASK-X-001.*
```

## fixture: ambiguous-both-markers

```markdown
---
template: task@1
---
## §1 - Description
## §11 - Implementation notes
```
(Template key AND § grammar -> needs_human, never a guessed profile.)
