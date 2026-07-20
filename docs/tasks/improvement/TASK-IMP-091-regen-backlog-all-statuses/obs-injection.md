# TASK-IMP-091 observability injection

The script is a one-shot operator tool: read corpus, compute, write one file, print a summary. It has no service lifetime, so spans and counters would have nothing to attach to. The honest observability surface is its stdout/stderr contract, and this task strengthens exactly that:

- **State transition** (the only one): `regenerated BACKLOG.md: N tasks across M modules` - pre-existing, still the success line, now truthful about N because every folder emits a row.
- **Error branch** (new): unparseable frontmatter prints one stderr line per offending file plus a halt message naming the count, and exits non-zero before any write. Previously this branch printed and continued - the failure was observable only as a silently short backlog.
- **No PII / secrets**: output is task ids, statuses, titles.

Branch coverage: 2 of 2 outcome branches (success summary, halt) asserted in the suite; 100 % of the change's reachable branches.
