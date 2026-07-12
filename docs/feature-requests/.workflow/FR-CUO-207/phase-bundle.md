# FR-CUO-207 phase bundle

## repo-context-map (step 1)
Touched surface: init.sh detector block (was elif-chain = single-stack; §1 #1 demands union) +
gates.env write; run-gates.sh gate() (2-arg -> 4-arg with config + provenance); README; coverage-gate
pair contract hook. Patterns: scaffold-once discipline copied from BACKLOG/AGENTS handling; test suite
copies test_check_latest's payload+fixture-repo pattern. Blast radius: 2 scripts, 1 README, 2 skill
files, 1 new test suite.

## edge-case matrix (step 5) -> covering test
NULL: empty repo -> floor + config pointer (t07); config absent -> pure autodetect (t01-t03).
BOUNDS: threshold unset -> 90 (t06 second half). MALFORMED: tab-indented yaml -> exit 2 + line number +
zero gates run (t08, proves no half-apply via ran-anyway sentinel). RACE: none (read-only detection).
SECURITY: config commands eval'd by run-gates - SAME trust level as gates.env (operator-owned file in
their own repo; no new surface). Nested markers (node_modules/package.json) can't false-fire: root-only
checks ($root/package.json literal paths). DEGRADATION: unknown config keys warn, never fail (grep loop);
detector tool absent (golangci-lint) -> documented fallback (go vet), never invented commands (t01/t03).

## implementation (steps 6-14)
init.sh: claim() union detector (documented order rust,node,python,go,maven,gradle,dotnet,php,ruby,make;
first claim per gate wins), per-gate SRC_* provenance into gates.env, config.yaml scaffolded once with
detected commands as comments. JVM coverage deliberately undetected (audit ISS-004). run-gates.sh:
dependency-free yaml-subset reader (awk, top-level + one nesting), per-key resolution
config > autodetect > absent, provenance line per gate, CYBEROS_COVERAGE_THRESHOLD export, loud
malformed-fail before any gate, floor message naming config.yaml. README stack table; coverage-gate
SKILL.md + RUBRIC.md name the env hook (closes the FR-SKILL-118 constant's promise).

## observability (step 15)
Provenance line per gate is the feature (audit ISS-006); malformed-config error carries file + line;
floor message names the fix file. All greppable stable prefixes: "gate <name>:", "MALFORMED", "floor only".

## code review vs §1 (steps 16-18)
#1 six new stacks + union + never-invent PASS (t01/t02/t03); #2 per-key config + unknown-warn PASS
(t04 + warn loop); #3 scaffold-once + detected-as-comments PASS (t05); #4 resolution order + provenance
PASS (t04); #5 threshold flow PASS (t06); #6 floor + pointer PASS (t07); #7 dependency-free parse +
loud fail PASS (t08). Secret scan: none. Injection: config values eval'd - documented same-trust as
gates.env. Backcompat: gates.env alone (no config.yaml) behaves exactly as before + provenance lines.

## coverage gate (steps 21-29)
test_gate_autodetect.sh 8/8 (one per AC). Full regression: 7/7 cyberos-init suites, ship_manifest 8/8
upstream. ECM rows covered per matrix above. tests_failed=0; bash-only FR (no python coverage basis).

## HITL record
Gate 1: pending below. Gate 2: per verdict.
