# AUDIT.md — AUTONOMOUS AUDIT & IMPROVEMENT PROTOCOL — v1.2.0

(Fixture copy: CONFIG block only. Every value below is legitimate, filled-in configuration of the kind real client repos produce — Java generics, shell redirection, an issue number after a single space, an inline enum comment. The preflight flagged three of these as placeholders before the F-4 fix.)

====================================================================
## CONFIG  (the ONLY part that changes per project — edit before running)
====================================================================
PROJECT_PATH:        ./
TECH_STACK:          Java 21 / Spring Boot / List<OrderDTO> aggregates
PROJECT_PURPOSE:     Batch pricing engine fed by stdin fixtures
MODE:                gated                    # gated | autonomous
LOOP_BUDGET:         3
DEPTH:               standard
SEVERITY_FLOOR:      High
PROTECTED_AREAS:     src/pricing/
RUN_COMMANDS:        ./gradlew test; ./bin/replay < seed.txt > replay.log
DOMAIN_NOTES:        follow ticket #4211 rounding conventions exactly
BENCHMARK_MODE:      none
COMPARATORS:

====================================================================
## CORE RULES (elided in fixture)
====================================================================
