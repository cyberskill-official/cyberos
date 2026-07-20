---
id: TASK-IMP-082
title: Status page provenance stamp becomes a corpus fingerprint (byte-stable)
template: task@1
type: improvement
module: improvement
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T11:45:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-074]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 hardening"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: 2026-07-16
memory_chain_hash: null
effort_hours: 3
service: tools/docs-site
new_files:
  - scripts/tests/test_render_stamp.sh
modified_files:
  - tools/docs-site/render-status-hub.mjs
source_pages:
  - "tools/docs-site/render-status-hub.mjs:307 (COMMIT = process.env.CYBEROS_COMMIT || gitCommit(ROOT) - the HEAD default)"
  - "tools/docs-site/render-status-hub.mjs:304-306 (the file's own comment already names the parent-sha chase the hook flow creates)"
  - "tools/docs-site/render-status-hub.mjs:451 + footer template (the two 'built from' surfaces) and the cs-data JSON commit field"
  - "tools/install/build.sh:171 (renderer vendors into payload docs-tools/ - consumer repos inherit the fix)"
  - "IMPROVEMENT_HANDOFF.md IMP-01 (reproduced twice on sachviet 2026-07-16: every re-install on an unchanged corpus produced a one-stamp diff; committing it re-arms the loop - sachviet commit df24cb3)"
source_decisions:
  - "2026-07-16 Stephen: feed IMPROVEMENT_HANDOFF.md through /create-tasks then /ship-tasks against cyberos itself; PLAN batch 1 approved with this item at p0."
---

# TASK-IMP-082: Status page provenance stamp becomes a corpus fingerprint (byte-stable)

## Summary

Change the status renderer's default provenance stamp from the current git HEAD to a deterministic fingerprint of the render inputs themselves. Today every bare re-render on an advanced HEAD differs by exactly the stamp, and committing the page advances HEAD again - a self-chasing loop the renderer's own comment describes but only mitigates behind an env pin nobody sets. A corpus fingerprint ends the chase in every flow: re-renders are byte-stable until the task corpus, CHANGELOG, or VERSION actually changes.

## Problem

The stamp default is repo position, not corpus content:

<untrusted_content source="tools/docs-site/render-status-hub.mjs:304-307">
// CYBEROS_COMMIT pins the provenance stamp. A page staged by the pre-commit hook necessarily // carries the PARENT commit's sha (the new one does not exist yet), so a later re-render would // differ from it by the stamp alone. Pinning lets a freshness check compare CONTENT. const COMMIT = process.env.CYBEROS_COMMIT || gitCommit(ROOT);
</untrusted_content>

No production caller (install, migrate, status-page.sh, run-gates, the pre-commit hooks) sets `CYBEROS_COMMIT`, so the mitigation is dead code in practice. Reproduced twice on the sachviet consumer repo on 2026-07-16: each re-install on an unchanged corpus dirtied the tree by one stamp; committing that (df24cb3) armed the next diff. A tracked, generated file that can never be clean is a standing false-positive in every `git status`.

## Proposed Solution

Make the default stamp a content fingerprint: the first 12 hex chars of sha256 over the ordered render inputs the script already reads (every task spec's bytes in sorted path order, CHANGELOG.md, VERSION). Properties: no git invocation on the default path, no wall clock, identical corpus in means identical page out - including the hook flow, because the page itself is not a render input. `CYBEROS_COMMIT` keeps winning when set, so any caller that wants a git sha can still pin one. The visible label shape ("built from `<stamp>`", footer, cs-data `commit` field) stays as is; only the value's derivation changes.

## Alternatives Considered

- Stamp from the last commit touching the inputs (`git log -1 --format=%h -- docs/tasks CHANGELOG.md VERSION`). Rejected: still chases in the pre-commit flow (the staged corpus's commit does not exist yet, so the landed page stamps the parent and the next render differs), and it needs git, which the renderer otherwise avoids.
- Drop the stamp entirely. Rejected: the stamp is the freshness check the comment at :304 wants - "compare CONTENT" is exactly what a corpus fingerprint is; removing it removes the check.
- Have every caller set CYBEROS_COMMIT. Rejected: five call sites across two hook variants plus consumer repos; one changed default beats N remembered call sites.

## Success Metrics

- Primary: re-render churn goes to zero - two consecutive renders on an unchanged corpus are byte-identical, and render -> commit page -> render is byte-identical. Baseline (today, sachviet evidence): every such cycle produces a 3-line diff. Deadline: this task's final acceptance.
- Guardrail: a real corpus change still changes the stamp exactly once (the freshness check keeps working), asserted by the same suite on every run.

## Scope

In scope: the stamp derivation in `render-status-hub.mjs`, its three output surfaces (header meta, footer, cs-data JSON), and a regression suite.

### Out of scope / Non-Goals

- Any other renderer behavior (lenses, drawer, spec chunks, KPI math).
- The hook scripts and their staging flow (unchanged; they simply stop producing churn).
- A freshness CLI that compares stamp vs corpus (possible follow-up, not here).

## Dependencies

- None upstream. Downstream: consumer repos inherit via the payload docs-tools copy (build.sh:171); TASK-IMP-083 and TASK-IMP-084 are cone-disjoint batch siblings.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted by the model from IMPROVEMENT_HANDOFF.md IMP-01 plus direct source investigation; implementation follows under ship-tasks supervision.
- **Human review:** PLAN approved by the operator on 2026-07-16; spec audit and both HITL acceptance gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The renderer's default stamp MUST be `fp-` plus the first 12 lowercase hex chars of sha256 over the ordered render inputs: every task spec file's raw bytes in sorted repo-relative path order, then CHANGELOG.md bytes when present, then VERSION bytes when present. The `fp-` prefix makes fingerprints distinguishable from git shas at a glance.
- 1.2 `CYBEROS_COMMIT`, when set and non-empty, MUST override the default exactly as today (explicit pin wins).
- 1.3 Two consecutive renders over an unchanged corpus MUST produce byte-identical index.html (whole file, not just the stamp).
- 1.4 Rendering, committing the rendered page, and rendering again MUST produce byte-identical output - the page's own bytes MUST NOT be a render input (the chase case).
- 1.5 Any change to a render input MUST change the stamp on the next render, and the render after that MUST be stable again.
- 1.6 The default path MUST NOT invoke git; rendering in a directory that is not a git checkout MUST produce the same fingerprint semantics.
- 1.7 All three stamp surfaces (header meta line, footer, cs-data `commit` field) MUST carry the same value; no other page content changes.
- 1.8 A regression suite MUST land at `scripts/tests/test_render_stamp.sh` covering 1.3, 1.4, 1.5, 1.2, and 1.6, discovered by the existing run_all glob.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: §1 #1.1, #1.7) - stamp is the fp- fingerprint on all three surfaces - test: `scripts/tests/test_render_stamp.sh::t01_fingerprint_on_all_surfaces`
- [ ] AC 2 (traces_to: §1 #1.3) - double render is byte-identical, on a populated AND an empty corpus - test: `scripts/tests/test_render_stamp.sh::t02_double_render_stable`
- [ ] AC 3 (traces_to: §1 #1.4) - render, commit page, render is byte-identical - test: `scripts/tests/test_render_stamp.sh::t03_commit_chase_ended`
- [ ] AC 4 (traces_to: §1 #1.5) - corpus edit changes the stamp exactly once - test: `scripts/tests/test_render_stamp.sh::t04_corpus_edit_changes_once`
- [ ] AC 5 (traces_to: §1 #1.2) - CYBEROS_COMMIT override wins - test: `scripts/tests/test_render_stamp.sh::t05_env_pin_wins`
- [ ] AC 6 (traces_to: §1 #1.6) - non-git directory renders with fingerprint, no git spawned - test: `scripts/tests/test_render_stamp.sh::t06_no_git_needed`
- [ ] AC 7 (traces_to: §1 #1.8) - suite is discovered by run_all - verify: `bash scripts/tests/run_all.sh` lists test_render_stamp.sh among suites (ops check recorded in the gate log; the glob discovery is the runner's own contract).

## 3. Edge cases

- Empty corpus (0 tasks, no CHANGELOG): fingerprint over VERSION alone (or the empty input set) MUST still be deterministic - t02 runs this shape.
- Task file with CRLF or trailing-whitespace edits: bytes changed means stamp changes - correct by definition (content is the contract); documented so nobody "fixes" it.
- Sorted path order MUST be bytewise (locale-independent sort) or the fingerprint drifts between machines - implementation sorts with a fixed comparator, t02 re-runs under LC_ALL=C and a UTF-8 locale.
- Very large corpora: hashing is streaming (per-file update), no full concat buffer - inspection note in review, no perf gate.
- CYBEROS_COMMIT set to empty string MUST fall through to the default (matches current `||` semantics) - covered inside t05.
- Security-class: none - no new input surface; the renderer already reads exactly these files. Nearest concern is hash choice: sha256 (node crypto), not a homegrown hash; asserted by code review.

## 4. Out of scope / non-goals

Duplicated intentionally with `## Scope` for template conformance: hooks, freshness CLI, and all other renderer features are untouched.

## 5. Protected invariants this task must not weaken

- Determinism doctrine: no wall clock, no randomness, no environment-dependent output beyond the documented pin.
- The status page stays a pure function of the corpus; markdown remains the record of truth.
- Payload sync doctrine: rebuild dist, version-sync, full suite before commit.
- HITL: both human-acceptance gates are recorded verdicts; the agent never sets done.

*End of TASK-IMP-082.*
