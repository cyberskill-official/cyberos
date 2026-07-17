# TASK-IMP-101 implementation plan

1. **Reconcile entry §** (clause 1.1) - placed immediately before Resume semantics so a reader meets the two trust mechanisms together: trigger conditions, the invocation, the fork table, the no-silent-execution rule, the explicit deferral to resume semantics when a manifest verifies.
2. **depends_on evidence gate §** (1.2) - the MUST sentence, the rationale (one bad claim becomes a subtree of them), the override path with its memory row, and the three false-block guards (both homes, off-ramps, history).
3. **Chain step 0** (1.3) - conditional entry above step 1 with NO renumbering of 1-31 (renumbering would invalidate every manifest's recorded step indices); `reconcile_report` added to outputs.
4. **Version + pins** (1.4) - 2.6.4 -> 2.7.0; the t12 and t09_doctrine_wiring exact pins move together (the known pair, disclosed).
5. **t14** (1.5) - both §§, step 0, and the version asserted in source AND scratch payload.

Deliberate non-change: no renumbering, no new statuses, no retroactive corpus reconciliation.
