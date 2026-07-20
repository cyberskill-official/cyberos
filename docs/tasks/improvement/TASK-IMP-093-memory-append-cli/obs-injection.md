---
artefact: observability-injection@1
task_id: TASK-IMP-093
branch_coverage_estimate: 100
created: 2026-07-17
verdict: pass (observability-injection-audit: run-to-completion CLI whose chain IS the observability; every failure and every self-heal announces)
---
# Observability injection - TASK-IMP-093

Honest vacuity statement: this task adds no service, no daemon, no network path, and no background loop. The appender is a run-to-completion CLI - but unlike a pure reporter, its OUTPUT is itself an observability substrate: every append leaves a tamper-evident, independently re-verifiable row on the BRAIN chain. Observability here is layered:

- The chain IS the audit trail. Each row carries actor, ts_ns, kind, path, prev_chain, chain - the §13 end-of-response accounting and the workflow's run reconstruction ("the per-phase workflow_phase_complete + final workflow_complete rows are enough to reconstruct the run", ship-tasks.md) now have a doc-driven writer. `verify` is the standing detector: any flipped bit in any row is named BY ORDINAL on any later run, and t01 additionally re-walks the chain with an independent (non-tool) reader so the suite would catch the tool lying about its own linkage.
- Every failure branch announces; none can go silent by construction. Bad kind / non-JSON / non-object payload / unsafe task token -> exit 2 with "refused before any write" phrasing; held or unparseable lease -> exit 3 "failing fast, nothing written"; chain/CRC/seq/HEAD damage -> exit 4 naming the first bad ordinal; compacted segments -> exit 4 naming the segment and pointing at the canonical walker. Exit codes are documented in --help (docs-tools convention) so a wrapper gets a clean boolean plus a diagnosable class without parsing prose.
- Every SELF-HEAL announces on stderr rather than happening silently: fresh-store bootstrap ("bootstrapped fresh store ... HEAD=0"), stale-tmp cleanup (names every file removed), stale-lease reap (names pid/host), and the benign HEAD re-publish after an interrupted publish ("HEAD was one behind the intact rows"). A silent self-heal is how inconsistencies become folklore; a loud one is a log line.
- Determinism is the monitoring contract: under --now/CYBEROS_NOW and a fixed --actor, identical inputs produce byte-identical frames, HEAD, and --json envelopes - drift is detectable by cmp, the same property the sibling helpers (ship-manifest, backlog-mutate) gate. The stdout success line carries seq, kind, truncated chain tip, record path, and new HEAD - the four facts an operator needs to eyeball a run.
- The standing detectors are the suite (t01-t04 under scripts/tests/run_all.sh:43's glob on every gate) and t04's live run of the VENDORED copy against a scratch store - a payload regression that drops or breaks the tool fails the same suite that gates its behavior.

branch_coverage_estimate 100 refers to announced-outcome branches: every terminating path ends in a stdout result line, a named stderr refusal, or a named stderr note; the only swallowed exceptions are the parent-dir-fsync fallback (platforms that cannot fsync a directory - rename atomicity still holds, documented in-code) and best-effort lease release on exit (TTL expiry heals it, per §4.2's own design).
