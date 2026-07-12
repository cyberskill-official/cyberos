# `backlog-state-update-author` - failure modes

1. Concurrent insert race - expected_absent + post-image single-occurrence turns it into deterministic fail-and-retry (put_if semantics).
2. Header counts drift - BSU-INS-004 permits that section's header counts only; regenerator reconciles periodically.
3. Hand-mangled BACKLOG - exact `## <module>` grammar required; anything else -> needs_human.
4. Class/suffix mismatch - BSU-INS-002 exact-format rule.
5. Backlog format evolution (e.g. done rows moved to header counts) - the regenerator is authoritative; rules bind to its current grammar.
