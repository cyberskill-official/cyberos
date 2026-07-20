---
artefact: observability-injection@1
task_id: TASK-IMP-098
branch_coverage_estimate: 100
created: 2026-07-17
verdict: pass (observability-injection-audit: vacuity justified honestly - run-to-completion CLI whose emitted skeleton IS the observability; every refusal announces with a distinct exit class)
---
# Observability injection - TASK-IMP-098

Honest vacuity statement: this task adds no service, no daemon, no network path, and no background loop. The tool is a run-to-completion CLI whose entire observable surface is the emitted skeleton + exit code - and that surface is the deliverable:

- The skeleton IS the observability. Every mechanical decision the tool made is visible IN the output it emits: the resolved base and HEAD shas with the resolution PROVENANCE ("base via --base '...'" vs the quoted entry-flip subject), the ambiguity note when the subject scan matched more than once (the operator sees the choice the tool made and how to override it), the report path + shape that fed the table, every touched file with its percentage or an explicit `no-coverage-data` marker, the deletions the table excluded, and literal TODO markers for every judgment field the tool refused to fake. Nothing is resolved silently; the gate reviewer can re-derive the whole table from the two facts the skeleton names (range + report).
- Exit codes are the gate surface, one class per failure mode: 2 usage/content, 3 base-unresolvable (distinct so a wrapper can prompt for --base specifically), 4 unsupported-report-by-name. All documented in --help (docs-tools convention); a caller gets a diagnosable class without parsing prose.
- Every refusal announces on stderr and refuses to half-emit: an unsupported report writes NO skeleton and NO --out file (t03 asserts both); a base the tool cannot prove is a refusal, never a guessed range (t01 asserts no skeleton leaks).
- Determinism is the monitoring contract: no clock, no randomness, bytewise-sorted table and deletions - identical repo + report + args emit byte-identical output, so drift is detectable by cmp. t02's expected-bytes compare (both report shapes, plus a rerun arm) gates exactly that on every run.
- No logging added, deliberately: a progress line would put non-deterministic chatter on the channel whose byte-stability is the contract; the one stderr line outside refusals is the --out confirmation ("coverage-scope: wrote <path>"), kept OFF stdout so the skeleton bytes stay clean for piping.
- The standing detectors are the suite (t01-t04 under scripts/tests/run_all.sh:43's glob on every gate) and t04's live run of the VENDORED copy against a scratch fixture repo - a payload regression fails the same suite that gates behavior. The consumer-scale detector is AC 5's parent-run sachviet reproduction, recorded in the gate log (consumer-repo evidence the fixture suite cannot carry).

branch_coverage_estimate 100 refers to announced-outcome branches: every terminating path ends in the skeleton on stdout, a written --out with a stderr confirmation, or a named stderr refusal with its documented exit class; there is no catch-and-continue that discards a defect unannounced (non-number pct values degrade to the VISIBLE no-coverage-data row, and report keys outside the repo root become visible the same way - both are presentation of the input's own gaps, not swallowed errors).
