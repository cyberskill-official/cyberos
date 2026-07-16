# TASK-IMP-091 implementation plan

1. **Delete the ACTIVE constant and its emission filter** (clause 1.1) - rows come from `sorted(rows)` over every folder; the `- (nothing remaining)` placeholder goes with it (a section with folders always has rows now).
2. **`status_line(tally)` helper** (clause 1.2) - one formatter for both the per-module header and the repo-wide Totals: STATUS_ORDER first, then any legal-but-unlisted status sorted, zero counts omitted. One function means header and Totals cannot drift from each other.
3. **Promote the unparseable skip to a halt** (clause 1.3 / §3) - collect offenders, print each to stderr, `sys.exit(...)` BEFORE the write. This is the guard the old code inverted: it warned, then wrote a backlog missing those tasks.
4. **Header prose** - the file's own preamble said it "lists ONLY remaining work"; that sentence was the bug's charter and is rewritten to state one row per task in every status.
5. **New suite** (clause 1.5) - `scripts/tests/test_regen_backlog.sh`, three scenarios, all on scratch copies.
6. **Gates** - the suite, then the parent's full pair.

Deliberate non-change: migrate/adopt phases, other sections' content, and the queue's eligibility logic (which reads frontmatter, not the index).
