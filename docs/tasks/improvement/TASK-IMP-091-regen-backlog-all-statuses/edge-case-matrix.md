# TASK-IMP-091 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | TERMINAL STATUS | corpus contains done/closed/on_hold tasks | one row each, status carried verbatim | t03_every_status_emitted |
| 2 | OFF-RAMP | status `cannot_reproduce` / `duplicate` (legal, absent from STATUS_ORDER) | emitted, and counted after the ordered statuses | t03_every_status_emitted; `status_line()` appends unlisted statuses sorted |
| 3 | BYTE PARITY | today's real corpus | regenerated improvement section identical to the committed object | t01_live_corpus_parity (compares against `git show HEAD:...`) |
| 4 | SEPARATOR IN TITLE | TASK-IMP-081's title quotes `TASK-IMP-080`; titles containing ` - ` | emitted verbatim; parity is byte-level so any mangling fails | t01_live_corpus_parity |
| 5 | TOTALS | any status distribution | Totals equals an independently computed frontmatter tally | t02_totals_true (tally computed without importing the script) |
| 6 | MALFORMED | a spec.md with unparseable frontmatter | halt naming every offending file; BACKLOG.md not written | t03_every_status_emitted (halt half: rc != 0, file named, sha256 unchanged) |
| 7 | EMPTY MODULE | module folder with zero task folders | no section emitted (module never enters `mods`) | pre-existing behavior, unchanged by this task |
| 8 | LIVE FILE SAFETY | suite runs in the repo | live `docs/tasks/BACKLOG.md` byte-untouched | every scenario runs on a scratch copy; `git status --short docs/tasks/BACKLOG.md` empty (gate log) |
| 9 | DEGRADATION | pyyaml absent | suite exits 2 with a clear message rather than a false pass | suite preamble `python3 -c "import yaml"` guard |
| 10 | SECURITY | none - read-compute-write over tracked markdown | no execution surface, no untrusted input | reviewed |
