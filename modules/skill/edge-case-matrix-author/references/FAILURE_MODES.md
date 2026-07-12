# `edge-case-matrix-author` - failure modes

1. Vacuous rows to hit the MUST minimum - audit rejects rows whose trigger duplicates another's semantics.
2. Security rows citing directories instead of test files - TRACE resolution requires a file.
3. Degradation rows with detection but no recovery - ECM-GATE-002 fails them.
4. Matrix drifts from implementation - coverage-gate's ecm_rows_uncovered closes the loop at testing.
5. Category force-fit (a bounds case dressed as race) - audit may reclassify with a finding.
