# TASK-IMP-100 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | NULL/EMPTY | status draft / ready_to_implement | not_applicable, exit 0, no recommendation | t04 (not_applicable arm) |
| 2 | LIFECYCLE CHURN | status/shipped flipped after the audit (every shipped task) | R1 pass - the normative half is what the audit judged | t04 (lifecycle-churn arm) |
| 3 | REAL DRIFT | a §1 clause edited after the audit commit | R1 red, route_back, "SPEC DRIFT" named | t04 |
| 4 | BINDING GAP | audit sha matches no committed version (pre-flip hashing) | note, not a verdict; substantive check still runs | t04 + the dogfood finding in the gate log |
| 5 | UNCOMMITTED CLAIM | deliverable on disk, no commit carries it (the 086 class) | R4 red, route_back, "UNCOMMITTED CLAIM" named | t02 (dirty arm) |
| 6 | ADOPT SHAPE | deliverables green at HEAD, no artefacts in either home | adopt_candidate | t03 |
| 7 | ARTEFACT HOME | historical bundle under docs/tasks/.workflow/<id>/ | accepted - no false adopt for the existing corpus | t03 (bundle arm) |
| 8 | CITATION ROT | cited suite renamed/absent | R5 red (TRACE-003 at run time), route_back | t02 (missing arm via fixture) |
| 9 | NO MANIFEST | out-of-band work | R3 absent - a finding, never a failure | t01/t03 (absent in every fixture) |
| 10 | SECURITY | tool must not mutate the tree | whole-tree fingerprint identical across runs | t04 (read-only arm) |
| 11 | DEGRADATION | task-lint or ship-manifest absent from the repo | rung notes "skipped"/"cannot verify", no false red | rung1/rung3 guards (probed manually; recorded) |
