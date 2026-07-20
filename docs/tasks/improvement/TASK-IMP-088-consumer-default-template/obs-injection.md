---
artefact: observability-injection@1
task_id: TASK-IMP-088
branch_coverage_estimate: 100
created: 2026-07-16
verdict: pass (observability-injection-audit: vacuity justified honestly - install-time scaffold with no runtime path; the scaffolded file and the hygiene suite ARE the observable surface)
---
# Observability injection - TASK-IMP-088

Honest vacuity statement: this task adds no service, no loop, no runtime path - it changes one line of a file that install writes AT MOST ONCE per repo. Nothing to trace or meter:

- The artifact is the observability. The whole point of the recorded IMP-06 decision was that a LIVE config line is inspectable and overridable in place, where a conditional chain default is invisible: `cat .cyberos/config.yaml` now answers "which template profile will authoring resolve, and why" on every consumer repo. The scaffold header still says the file documents what runs today.
- Branches that exist, and how each announces: (a) consumer vs platform - two fixed literals selected by the pre-existing `-f` marker test; the outcome is readable in the scaffolded file itself (live line vs commented line). (b) create-once - an existing file short-circuits the whole block; the file's unchanged bytes are the evidence (t06_existing_config_untouched cmp's them). No new failure branch is introduced: the assignment cannot fail, the function is a pure `-f` test, and heredoc write failures abort the install loudly under `set -euo pipefail` exactly as before.
- The silent-wrong-default hazard is now gated, not just fixed. The one genuinely silent failure mode here was ordering (calling the detector before its definition: exit 127 forgiven inside `&&`, consumer line scaffolded on the platform repo). The hoist removes it and t06_platform_keeps_comment fails the suite the day anyone reintroduces it - the standing detector is the hygiene suite under scripts/tests/run_all.sh's glob on every gates run.
- No logging added, deliberately: install's summary block is a curated operator surface; a per-line scaffold announcement would be chatter. Silence plus an inspectable artifact is the contract.

branch_coverage_estimate 100 refers to the two real branches (consumer/platform selection, create-once skip): each is exercised by a dedicated t06 scenario on every suite run.
