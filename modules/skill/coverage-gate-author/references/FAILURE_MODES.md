# `coverage-gate-author` - failure modes

1. Coverage laundering via tiny helper files - touched-files basis from the git diff, not a curated list.
2. Truncated terminal hiding failures - COV-STRUCT-001 requires the full capture.
3. ECM rows silently dropped - COV-GATE-003 reconciles against the matrix artefact by id.
4. Threshold hardcode vs config drift - named constant + FR-CUO-207 override hook in the rubric header.
5. Flaky suite passed on retry without record - reruns are recorded; debugging-cycle owns the failure vector analysis.
