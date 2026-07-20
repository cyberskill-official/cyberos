---
artefact: observability-injection@1
task_id: TASK-IMP-087
branch_coverage_estimate: n/a (no executable branches - docs-only)
created: 2026-07-16
verdict: pass (observability-injection-audit: vacuity justified honestly - a tracked markdown gate document; its greppable shape IS the observable surface, and every drift mode has a recorded re-runnable detector)
---
# Observability injection - TASK-IMP-087

Honest vacuity statement: this task adds no code, no service, no CLI, no build step, and no runtime path of any kind. There is nothing to instrument, log, or trace - and adding any of that would be scope invention. What the task CAN honestly claim as observability is the document's machine-readable shape and the recorded detector set that makes drift visible:

- The row grammar is the telemetry. Stable row ids (`A1`-`E4`), a fixed 5-cell shape, a closed state set and verbatim IMP-15.N tags mean the document's health is greppable at any moment: G1 (shape), G2 (closed set), G3 (waiver reasons), G4-G6 (required content present). The exact commands with their 2026-07-16 outputs are on the record in gate-log-draft.md; every one is re-runnable against the living file and asserts shape rather than specific states, so the designed drift (rows flipping open -> checked as the release is worked) never trips them, while the undesigned drift (a sixth state, a four-cell row, an empty waiver) trips them immediately.
- State transitions announce themselves in git. The document is tracked; every line flip is a diff on a row id with its Evidence cell filled - the review surface for "who checked what, with which command or artefact" is the commit history, which is the same observability story the rest of docs/ governance uses (BACKLOG flips, status page regens).
- The release gate's failure mode is silence (a line nobody worked), and the document is built so silence is enumerable: `awk -F'|' '/^\| [A-E][0-9]+ \|/ && $5 ~ /open/' docs/release/RELEASE-CHECKLIST.md` lists exactly what still blocks the tag. Zero output = ready (15 rows today).
- The credential scan (G8) is the standing security detector for the two evidence cells that will tempt pasting (B3 session records, B4 run links); it runs in seconds and its zero-hit baseline is recorded.
- Nothing here wires CI. That is a spec Non-Goal ("CI can lift lines from it later" - the Alternatives note); the detectors are deliberately shell-greps a human or agent replays, because automating the gate before the first release proves the list's shape was explicitly rejected in the spec.

branch_coverage_estimate is n/a because there are no branches; the analogous honest number is detector coverage of the AC surface: 4/4 ACs carry at least one recorded, re-runnable check (G1-G3 -> AC 1, G4-G6 -> AC 2, G7 -> AC 3, G8-G9 -> AC 4), and no drift mode named in the edge-case matrix lacks a Covered-by detector.
