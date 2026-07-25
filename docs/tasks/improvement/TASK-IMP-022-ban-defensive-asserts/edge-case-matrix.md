# TASK-IMP-022 edge-case matrix

Mirrors spec §3. "Covered by" names the arm that would go red, not the suite that happens to touch the code.

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | NULL/EMPTY | test file with no `assert` | scanned, no finding, counted in the file total | t02 (both fixture functions are assert-bearing; the file total is asserted via exit 0) |
| 2 | NULL/EMPTY | a declared corpus root missing on disk | `WARN shell root missing: <root>` on stderr; scan continues; exit unaffected | t01 (scratch has `modules/m/tests` only for the Python arm; the shell roots exist but are empty) |
| 3 | NULL/EMPTY | empty repo — zero test files | `files=0 findings=0`, exit 0. Vacuous by construction and not claimed as coverage | reviewed (spec edge) |
| 4 | BOUNDS | `assert False` | not flagged — an unreachable marker, not a defensive assertion | t04 (second scratch: `assert False` file must exit 0) |
| 5 | BOUNDS | waiver reason exactly 12 chars | suppresses; 11 chars is `DA-005` | t07 (`meh` = 3 chars → DA-005; the reasoned arm is 44 chars) |
| 6 | BOUNDS | disjunction nested in a comprehension (`any(x or y for …)`) | flagged — `ast.walk` reaches the `BoolOp` inside the assert's test | t01 by construction; live proof at `test_baseline.py:149` (pre-fix) |
| 7 | BOUNDS | `assert a and b` | not flagged — a conjunction is strictly stronger, never a finding | t02 (no `BoolOp(Or)` present → exit 0) |
| 8 | MALFORMED | Python test file that does not parse | `DA-000` at the syntax-error line — reported, never skipped silently | reviewed (`scan_python` `except SyntaxError`); no fixture, see note |
| 9 | MALFORMED | `or` inside an assert MESSAGE, not its test | not flagged — the AST separates `Assert.test` from `Assert.msg` | t02 (second function: `assert types, "… not found or admits no types"`) |
| 10 | MALFORMED | disjunction continued with `\` across lines | flagged, reported at the `assert`'s own line | t03 |
| 11 | MALFORMED | `n="$(grep -c x f)" \|\| true` | not flagged — an output capture, not an assertion; grep exits 1 on zero matches | t05 (second scratch) |
| 12 | SECURITY | the scanner is pointed at hostile test content | never executed: `ast.parse` does not evaluate, and no scanned shell line is invoked | reviewed — the scanner has no `eval`/`exec`/`subprocess` path |
| 13 | SECURITY | a waiver used to silence a genuine defect | **bounded, not prevented** — the reason is mandatory, every waiver prints on every run, and `t08` pins the corpus at zero waivers, so the first one has to be argued in review | t07, t08 |
| 14 | SECURITY | `\|\| true` inside a heredoc fixture | not flagged — heredoc bodies are data. Stated as a bound so it is not mistaken for coverage | t05 (the suite itself is the live proof: it contains the shape and the corpus is clean) |
| 15 | CONCURRENT | two agents adding test files between a scan and a commit | no shared state; the lint re-walks the tree per run, and pre-commit runs it on the staged tree | t08 |
| 16 | DEGRADATION | `run_all.sh` gains a fourth globbed root | `t09` red until the lint's root list is extended — drift caught, not inherited | t09 |
| 17 | DEGRADATION | `python3` absent | the wrapper exits non-zero from the heredoc; `run_all.sh` reports the suite red. No silent skip | reviewed — the repo already hard-requires `python3` (`check_doc_anchors.sh`, `test_corpus_hygiene.sh`) |
| 18 | DEGRADATION | Rust/Go assertions under `services/**` | out of scope by declaration; a green run makes no claim about them | — (stated bound; no test by design) |

**Note on row 8.** No fixture: a deliberately unparsable file in `scripts/tests/` would be collected by
`run_all.sh`'s glob only if named `test_*.sh`, and a `.py` one under a scratch root is reachable — but the
arm proves an error path whose only observable is a `DA-000` line, and it shares its code path with nothing
else. Reviewed by inspection rather than asserted, and named here rather than left off the matrix.
