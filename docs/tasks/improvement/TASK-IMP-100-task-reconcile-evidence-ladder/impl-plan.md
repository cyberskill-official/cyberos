# TASK-IMP-100 implementation plan

1. **The ladder** (clauses 1.1-1.4) - five rungs, read-only; R1 composes task-lint + the audit binding, R2 the artefact set across both homes (bundle-aware), R3 ship-manifest verify, R4 `git ls-tree HEAD` per claimed path, R5 the cited suites under `--run-tests` only.
2. **The recommendation map** (1.3) - load-bearing reds -> route_back; artefacts-only gap -> adopt_candidate; else resume_at_phase(N) via the PHASE_OF table; draft/ready_to_implement -> not_applicable.
3. **The report** - reconcile-report@1 with per-rung verdicts, drift_score, `hitl: required`, one recommendation; `--json` for machines; `--out` writes (parent created, inside root).
4. **The skill** (1.5) - machine-floor-first loop, the judgment half, and the hard no-silent-execution rule.
5. **Vendoring** (1.6) - build.sh guarded copy + VENDORED_SKILLS entry (chain coverage demands the skill in both payload trees).
6. **Suite** - five scenarios over scratch git repos shaped like real corpus entries, each bending exactly one thing.

Order note: R1's design changed twice under dogfooding (see the gate log) - the ladder was written, run against the live corpus, and corrected before the suite was frozen. That sequence is deliberate: a measuring tool that has never measured the real thing measures nothing.
