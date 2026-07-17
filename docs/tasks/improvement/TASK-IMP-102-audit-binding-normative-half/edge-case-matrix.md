# TASK-IMP-102 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | LIFECYCLE FLIP | body-bound audit, status flips reviewing -> done | R1 pass, no binding-gap note | t06 (first arm) |
| 2 | REAL DRIFT | body-bound audit, a §1 clause edited after | R1 red, "SPEC DRIFT" naming both hashes | t06 (drift arm) |
| 3 | LEGACY HONEST | no body field, audit hashed committed bytes | audit-commit path, R1 pass on lifecycle churn | t06 (legacy arm) |
| 4 | LEGACY DISHONEST | no body field, sha recorded pre-flip (the corpus's real shape) | gap named as legacy; still pass - a note is not a verdict | t06 (dishonest-legacy arm) |
| 5 | BOTH FIELDS MATCH | audit written post-flip carrying both | body decides; file field is provenance | t06 first arm's shape (file field deliberately bogus, body correct) |
| 6 | MISSING LIFECYCLE FIELD | spec without `shipped:` at all | normalizer drops what exists; absence is not drift | normalizer field-list semantics (t06 fixtures lack `routed_back_count`) |
| 7 | FUTURE FIELD | a new lifecycle field added to the template | one place to extend: LIFECYCLE_FIELDS, named in §12 | reviewed - documented, not asserted |
| 8 | SHALLOW CLONE | legacy audit whose commit is unreachable | "binding unverifiable" note, never a verdict | t04's existing arm |
| 9 | SECURITY | none - hashing + prose | n/a | reviewed |
