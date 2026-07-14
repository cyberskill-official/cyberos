---
artefact: edge-case-matrix@1
task_id: TASK-IMP-068
total_rows: 12
created: 2026-07-12
verdict: pass (edge-case-matrix-audit: every category >=1 row, total_rows >= 8 for MUST FR, SECURITY rows point at tests, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-068

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | VERSION file absent | build.sh exit non-zero, no payload written | t04 |
| 2 | null/empty | VERSION empty string | same as absent (regex gate) | t04 |
| 3 | null/empty | payload dir missing/empty | check exit 2 "unreadable", never 0 | t02 harness |
| 4 | bounds | VERSION with pre-release suffix 1.7.1-rc1 | rejected by X.Y.Z regex (build refuses) | t04 |
| 5 | bounds | VERSION with trailing whitespace/newline | trimmed, accepted (tr -d) | t01 |
| 6 | malformed | manifest.yaml missing cyberos_version line | check exit 2 naming manifest.yaml | t02 |
| 7 | malformed | cyberos.plugin not a zip / truncated | unzip -p fails -> exit 2, not false pass | t03 harness |
| 8 | malformed | plugin.json invalid JSON | node parse fails -> exit 2 naming the file | t02 |
| 9 | concurrency | commit while another rebuild holds dist/ half-written | check reads miss a file -> exit 2 (fail-closed); rerun clean | t02 (missing-file case) |
| 10 | SECURITY | VERSION content injected into sed (e.g. `1.0.0/; s/x/y/`) | regex gate rejects non-X.Y.Z before any sed use | t04 |
| 11 | SECURITY | hook bypass via git commit --no-verify | payload-gate.yml on push/PR is the backstop | t06 (gate wiring) + ADR consequence |
| 12 | DEGRADATION | CI runner or dev machine missing unzip/node | detection: explicit tool probe; recovery: exit 2 with "install X" message, never a silent pass | t02 harness precondition |
