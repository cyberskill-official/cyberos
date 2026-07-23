---
id: TASK-CUO-207
title: "Portability hardening - install.sh gate autodetect for Go/JVM/.NET/PHP/Ruby + per-repo .cyberos/config.yaml overrides"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: cuo
priority: p1
status: done
verify: T
phase: Wave C - strengthen the workflows
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-CUO-208, TASK-SKILL-118, TASK-IMP-070]
depends_on: []
blocks: [TASK-CUO-208]
source_pages:
  - tools/install/install.sh
  - tools/install/gates/run-gates.sh
  - tools/install/README.md
source_decisions:
  - "2026-07-12 operator goal: the two workflows will be used heavily across other projects after /install; today's gate autodetect covers Rust/Node/Python only, and there is no per-repo way to override gate commands, coverage threshold, or defaults without editing vendored files."
  - "Unknown stacks keep degrading to the reduced-profile floor - portability means detecting more, never guessing."
language: bash + yaml
service: tools/install/
new_files:
  - tools/install/tests/test_gate_autodetect.sh
modified_files:
  - tools/install/install.sh
  - tools/install/gates/run-gates.sh
  - tools/install/README.md
---

# TASK-CUO-207: Gate autodetect portability + per-repo config

## §1 - Description

Make /install produce working gates on the stacks CyberSkill's client projects actually use, and give every repo one sanctioned place to override what detection gets wrong - so vendored files stay pristine and updates never clobber local decisions.

Normative clauses:

1. Gate autodetection MUST extend to, in documented order after the existing Rust/Node/Python detectors: Go (`go.mod` -> build `go build ./...`, lint `go vet ./...`, test `go test ./...`, coverage `go test -coverprofile`), Maven (`pom.xml` -> `mvn -q -DskipTests package` / `mvn -q verify`), Gradle (`build.gradle` or `build.gradle.kts`, preferring `./gradlew` when present -> `build` / `test`), .NET (`*.sln` or `*.csproj` -> `dotnet build` / `dotnet test`), PHP (`composer.json` -> `composer validate --strict` plus `vendor/bin/phpunit` when present), Ruby (`Gemfile` -> `bundle exec rspec` when spec/ exists, else `bundle exec rake test` when a Rakefile exists). Multi-stack repos MUST union the detected gates; detection MUST never invent a command whose tool marker file is absent.
2. A per-repo config file `.cyberos/config.yaml` MUST be honored by `run-gates.sh` when present, with keys: `gates.build`, `gates.lint`, `gates.test`, `gates.coverage` (string commands; each overrides ONLY its own gate), `coverage_threshold` (integer, default 90), `task_template` (`engineering-spec@1` | `task@1`, consumed by TASK-CUO-208), `profile` (`full` | `reduced`). Unknown keys MUST warn, not fail.
3. `install.sh` MUST scaffold `.cyberos/config.yaml` exactly once (never clobber an existing one, same discipline as BACKLOG/AGENTS), pre-filled with every value commented out and the DETECTED commands written as comments beside each key, so the file documents what will run by default.
4. `run-gates.sh` MUST resolve each gate as: config value if set, else autodetected, else absent - and MUST print one provenance line per gate before running it: `gate <name>: <command> (source: config|autodetect:<stack>|absent)`.
5. `coverage_threshold` MUST flow to the coverage gate: `run-gates.sh` exposes it (env `CYBEROS_COVERAGE_THRESHOLD`) and the coverage-gate skill contract reads it, defaulting to 90 when unset (hook already named by TASK-SKILL-118's rubric constants).
6. Repos where nothing is detected and no config exists MUST keep today's reduced-floor behavior with an explicit message naming the config file as the fix.
7. Config parsing MUST be dependency-free (grep/sed-level YAML subset: top-level and one nesting level, scalar values); a malformed config MUST fail gates loudly with the offending line, never half-apply.

## §2 - Why this design

Config-over-autodetect (per key, not all-or-nothing) matches how real repos deviate: usually one gate is special, the rest are standard. Scaffolding the config WITH detection results as comments makes /install self-documenting on day one and keeps the file inert until the operator uncomments a line - update-safe by construction. The provenance line kills the classic debugging question ("which command even ran?") across a fleet of differently-shaped repos.

## §3 - Contract

```yaml
# .cyberos/config.yaml (scaffolded form, everything commented)
# gates:
#   build: "go build ./..."        # autodetected: go
#   lint: "go vet ./..."           # autodetected: go
#   test: "go test ./..."          # autodetected: go
#   coverage: "go test -coverprofile=coverage.out ./..."   # autodetected: go
# coverage_threshold: 90
# task_template: engineering-spec@1
# profile: full
```

Provenance output: `gate test: go test ./... (source: autodetect:go)` | `gate lint: make lint (source: config)` | `gate coverage: (source: absent)`.

## §4 - Acceptance criteria

1. **Each new stack detects** (§1 #1) - fixture repos for Go, Maven, Gradle (with and without wrapper), .NET, PHP (with and without phpunit), Ruby (rspec and rake variants) each yield the specified commands and nothing else.
2. **Multi-stack unions** (§1 #1) - a fixture with `go.mod` + `package.json` yields both stacks' gates, deduplicated by gate name with both provenance lines.
3. **No marker, no command** (§1 #1) - a PHP fixture without `vendor/bin/phpunit` gets `composer validate --strict` only.
4. **Config overrides per key** (§1 #2, #4) - config setting only `gates.lint` leaves build/test/coverage autodetected; provenance lines show `config` for lint and `autodetect` for the rest.
5. **Scaffold once, never clobber** (§1 #3) - first init writes the commented config with detected values; a hand-edited config survives a re-install byte-identical.
6. **Threshold flows** (§1 #5) - `coverage_threshold: 85` surfaces as `CYBEROS_COVERAGE_THRESHOLD=85` in the gate environment; unset -> 90.
7. **Reduced floor preserved with pointer** (§1 #6) - an empty fixture repo reports the floor message naming `.cyberos/config.yaml`.
8. **Malformed config fails loudly** (§1 #7) - a config with a tab-indented or unparseable line fails `run-gates.sh` citing the line number; no gate runs.

## §5 - Verification

```bash
# tools/install/tests/test_gate_autodetect.sh
t01_stack_matrix()               # AC 1  (8 fixture repos, expected command sets)
t02_multistack_union()           # AC 2
t03_marker_gating()              # AC 3
t04_config_per_key_override()    # AC 4
t05_scaffold_once()              # AC 5
t06_threshold_env()              # AC 6
t07_reduced_floor_message()      # AC 7
t08_malformed_config_loud()      # AC 8
```

## §6 - Implementation skeleton

`install.sh`: extend the detector case-block; emit the commented config via heredoc when absent. `run-gates.sh`: minimal reader (`cfg_get key` via awk over the two-level subset), resolution order per gate, provenance echo, threshold export. README: stack table + config reference.

## §7 - Dependencies

Blocks TASK-CUO-208 (it reads `task_template` from this config). TASK-SKILL-118's coverage rubric names the threshold hook this task turns on. No dependency on Wave A.

## §8 - Example payloads

```
$ bash .cyberos/cuo/gates/run-gates.sh
gate build: ./gradlew build (source: autodetect:gradle)
gate lint: (source: absent)
gate test: ./gradlew test (source: autodetect:gradle)
gate coverage: mvn -q verify (source: config)
```

## §9 - Open questions

None blocking. Container-based stacks (Docker-only repos) stay reduced-floor by design; a `gates.*` config line is their sanctioned path - documenting a docker-compose example in README is part of #1's doc work, not a new detector. JVM coverage is deliberately undetected (jacoco/kover wiring is repo-specific); the scaffolded config's commented `gates.coverage` line is the sanctioned path there too.

## §10 - Failure modes inventory

1. Wrong tool version on the machine (gradle vs gradlew drift) - wrapper-preferred rule (#1) plus provenance line make the executed command visible; config overrides when the default is wrong.
2. Autodetect finds a marker in a vendored subdir (`node_modules/package.json`) - detectors MUST scan the repo root only (as today); fixture t03 includes a nested marker that must not fire.
3. Config injection (config runs arbitrary commands) - accepted by design: the config is repo-committed and operator-owned, same trust level as a Makefile; the provenance line keeps it visible.
4. YAML subset surprises (quotes, colons in commands) - the reader supports quoted scalars; t08 covers an unsupported construct failing loudly rather than mis-parsing.
5. Threshold set above 100 or non-integer - reader validates 1..100 integer; else loud fail (t08 family).

## §11 - Implementation notes

Keep detector order stable and documented in README (first-party stacks first); the union rule means order only affects provenance labels, never which gates exist. The config reader lives in run-gates.sh (payload-vendored) so target repos need no extra file.

*End of TASK-CUO-207.*
