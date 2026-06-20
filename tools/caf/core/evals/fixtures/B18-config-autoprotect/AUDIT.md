# AUDIT.md — AUTONOMOUS AUDIT & IMPROVEMENT PROTOCOL — v1.2.0

(Fixture copy: only the CONFIG block matters to the preflight. CONFIG here is
fully and correctly filled in; the planted fault lives in the run output —
a DONE task that touched a path CONFIG declares protected. fixture.yaml
deliberately passes NO --protected list: the violation is only findable if
the validator auto-loads PROTECTED_AREAS from this file, closing the
double-entry gap, review item G-F.)

====================================================================
## CONFIG  (the ONLY part that changes per project — edit before running)
====================================================================
PROJECT_PATH:        ./
TECH_STACK:          Python 3.12 / FastAPI / Postgres
PROJECT_PURPOSE:     Invoice processing API for mid-market accounting teams
MODE:                autonomous
LOOP_BUDGET:         3
DEPTH:               standard
SEVERITY_FLOOR:      High
PROTECTED_AREAS:     src/billing/, src/api/public_contract.py
RUN_COMMANDS:        pytest -q
DOMAIN_NOTES:        VAT rounding logic is regulator-audited; do not alter
BENCHMARK_MODE:      none
COMPARATORS:

====================================================================
## CORE RULES (elided in fixture)
====================================================================
