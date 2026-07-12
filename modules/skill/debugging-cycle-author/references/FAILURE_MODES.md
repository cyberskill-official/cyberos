# `debugging-cycle-author` - failure modes

1. Hypothesis theater (vague guesses) - DBG-STRUCT-001 rejects vacuous rows.
2. Counter gamed by interleaving no-op green runs - resets require a genuinely green suite, recorded in the row.
3. Fix touches files outside the FR's blast radius - review step catches; trace rows carry the diff scope.
4. Endless vector reclassification of the same failure - attempt indices are monotonic; the budget counts attempts, not vectors.
5. Circuit break without revert - the breaker's on_trip steps are part of the workflow contract (ship doc), audited via the trace.
