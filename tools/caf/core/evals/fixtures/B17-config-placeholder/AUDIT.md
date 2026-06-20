# AUDIT.md — AUTONOMOUS AUDIT & IMPROVEMENT PROTOCOL — v1.2.0

(Fixture copy: only the CONFIG block matters to the preflight; the rules and
state machine are elided. This simulates a target repo where the human pasted
the protocol but never edited CONFIG — the diverse-codebase first-run failure
mode, review gap G-D.)

====================================================================
## CONFIG  (the ONLY part that changes per project — edit before running)
====================================================================
PROJECT_PATH:        ./                       # working dir; you may only edit here
TECH_STACK:          <e.g. Python 3.12 / FastAPI / Postgres>
PROJECT_PURPOSE:     <one line: what this software does and for whom>
MODE:                gated                    # gated | autonomous
LOOP_BUDGET:         3                        # max macro-loops this run
DEPTH:               thorough                 # quick | standard | deep — "thorough" is NOT in the set
SEVERITY_FLOOR:      High                     # only act on issues >= this severity
PROTECTED_AREAS:     <paths/modules/biz-logic that must NOT change behavior>
RUN_COMMANDS:        <how to build / test / lint / start, e.g. `pytest -q`>
DOMAIN_NOTES:        <constraints, compliance, non-obvious gotchas>
BENCHMARK_MODE:      auto                     # auto | provided | none
COMPARATORS:         <optional: real products to compare against, or leave blank>

====================================================================
## CORE RULES (elided in fixture)
====================================================================
