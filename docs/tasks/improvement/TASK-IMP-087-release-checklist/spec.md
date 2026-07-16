---
id: TASK-IMP-087
title: Release-readiness checklist for 1.0.0 at docs/release/
template: task@1
type: chore
module: improvement
status: testing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T15:12:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-082, TASK-IMP-083, TASK-IMP-084, TASK-IMP-085]
routed_back_count: 0
awh: N/A
verify: I
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 2
service: docs/release
new_files:
  - docs/release/RELEASE-CHECKLIST.md
modified_files: []
source_pages:
  - "IMPROVEMENT_HANDOFF.md IMP-15 (the seven pre-tag lines: S1 items landed-or-waived, npm pack + npx smoke, plugin-zip session test, channel matrix re-verify, release.yml dry-run, CHANGELOG section, fresh-clone consumer test)"
  - "IMPROVEMENT_HANDOFF.md section 5 evidence index + the 2026-07-16 channel research (.devin/rules preferred, .agents/skills shared dir)"
  - "operator decisions 2026-07-16: IMP-06 scaffold task@1 in consumer config.yaml; IMP-07 drop template section 4; IMP-11 manifests untracked session state - all three become pre-release implementation lines"
source_decisions:
  - "2026-07-16 Stephen: PLAN batch 2 approved with this item at p1; IMP-06/07/11 decisions recorded at the same gate."
---

# TASK-IMP-087: Release-readiness checklist for 1.0.0 at docs/release/

## Summary

VERSION says 1.0.0 but nothing defines what "ready to tag" means. Write the release checklist as a living tracked document: every line carries an owner, a state (open, checked, or waived with reason), and the verification command or evidence link where one exists. Seed it with the seven IMP-15 lines, the three operator decisions recorded today (IMP-06/07/11, now pre-release implementation items), and the channel-freshness matrix from the 2026-07-16 research.

## Problem

The pre-1.0.0 hardening run closed real payload gaps found only by running the workflows against a live consumer repo. The remaining risk is release mechanics that have never been exercised: the npm package has never been packed, the plugin zip never loaded into a live session end-to-end from a release asset, the tag flow never dry-run. A checklist nobody wrote is a release gate nobody holds.

## Proposed Solution

`docs/release/RELEASE-CHECKLIST.md`: a table (line, owner, state, evidence) grouped into (a) code readiness - S1 handoff items landed or waived with reason; (b) artifact readiness - `npm pack` dry-run of `@cyberskill/cyberos` plus an `npx` smoke on a scratch repo, plugin zip loaded into a live Claude Code or Cowork session with the three commands triggered once, `release.yml` tag-flow dry-run attaching payload assets; (c) channel readiness - the agent-surface matrix re-verified against current tool conventions (including `.devin/rules/` preference and the shared `.agents/skills/` dir); (d) docs readiness - CHANGELOG release section, GUIDE pass, fresh-clone consumer test (`git clone sachviet && npm ci && npm run coverage` green); (e) the three decided items IMP-06/07/11 implemented or explicitly deferred past 1.0.0. English, no secrets, cross-linked to IMPROVEMENT_HANDOFF.md and the batch evidence.

## Alternatives Considered

- Extend GUIDE.md instead of a new file. Rejected: the GUIDE is shipped inside the payload to consumers; the release gate is a platform-repo operator document - different audience, different lifecycle.
- A GitHub issue template or PR checklist. Rejected for now: the repo's governance runs on tracked markdown under docs/ (tasks, status, decisions); the release gate belongs in the same corpus, and CI can lift lines from it later.
- Automate every line now. Rejected: half the lines are one-time human acts (loading a plugin into a session, judging a waiver); automate after the first release proves the list's shape.

## Success Metrics

- Primary: the checklist exists with every line carrying owner + state + evidence column, and zero lines in an undefined state. Baseline: no release definition exists. Deadline: this task's final acceptance (working the lines to checked/waived is the release itself, owned by the operator afterward).
- Guardrail: the recorded grep set in the gate log proves the seven IMP-15 lines, the three decision lines, and the channel matrix are all present.

## Scope

In scope: the checklist document, its cross-links, the recorded presence checks.

### Out of scope / Non-Goals

- Executing the checklist lines (that is the release run itself, operator-owned).
- Implementing IMP-06/07/11 (batch-3 tasks; the checklist references them as gate lines).
- CI automation of checklist lines.

## Dependencies

- Reads decisions and evidence from IMPROVEMENT_HANDOFF.md; no build coupling. Cone-disjoint from TASK-IMP-085 (docs-tools) and TASK-IMP-086 (BACKLOG.md).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted by the model from IMPROVEMENT_HANDOFF.md IMP-15 and the recorded operator decisions; implementation follows under ship-tasks supervision.
- **Human review:** PLAN approved by the operator on 2026-07-16; spec audit and both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The checklist MUST exist at `docs/release/RELEASE-CHECKLIST.md` as a table whose every line carries: the line text, an owner, a state from the closed set {open, checked, waived}, and an evidence cell (command, link, or reason - mandatory when state is waived).
- 1.2 It MUST contain the seven IMP-15 lines: S1 handoff items landed-or-waived; npm pack dry-run + npx smoke; plugin zip live-session test of the three commands; channel matrix re-verification; release.yml tag-flow dry-run; CHANGELOG 1.0.0 section; fresh-clone consumer test.
- 1.3 It MUST contain one line per operator decision recorded 2026-07-16 (IMP-06 config.yaml task@1 scaffold, IMP-07 template section-4 drop, IMP-11 untracked manifests) with state open and a pointer to the decision record.
- 1.4 It MUST carry the channel-freshness matrix (agent surface file per tool, including .devin/rules/ and .agents/skills/ candidates) with a re-verify-before-tag instruction and the research date.
- 1.5 Machine-checkable lines MUST name their command verbatim (npm pack, npx invocation, clone-and-coverage sequence, build/sync/suite trio); human-only lines MUST say what evidence satisfies them.
- 1.6 The document MUST be English, contain no secrets or tokens, and cross-link IMPROVEMENT_HANDOFF.md and the batch-1/2 evidence commits.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: §1 #1.1) - file exists; every table line has owner, closed-set state, evidence cell - verify: recorded structure greps in audit.md §gate-log (ops verification: a tracked operator document; a test suite for one markdown file is out of scope by design, see Non-Goals).
- [ ] AC 2 (traces_to: §1 #1.2, #1.3, #1.4) - seven IMP-15 lines + three decision lines + channel matrix present - verify: recorded presence greps in audit.md §gate-log (same rationale).
- [ ] AC 3 (traces_to: §1 #1.5) - every machine-checkable line names its verbatim command - verify: recorded command-cell extraction in audit.md §gate-log (same rationale).
- [ ] AC 4 (traces_to: §1 #1.6) - no secrets; cross-links resolve - verify: recorded secret-pattern scan and link check in audit.md §gate-log (same rationale).

## 3. Edge cases

- Waived lines: the state set forces a reason in the evidence cell - an empty waiver is a structure violation AC 1 catches.
- Checklist drift after batch 3 lands IMP-06/07/11: those lines flip to checked with commit evidence - the document is living; the ACs verify shape, not final states.
- Channel conventions moving again before the tag: the matrix carries its research date and the re-verify instruction rather than pretending permanence.
- Secrets: the plugin-session and tag-flow lines tempt token pasting - the no-secrets clause plus AC 4's scan guard it.
- Security-class: none beyond the secret scan; the document executes nothing.

## 4. Out of scope / non-goals

Duplicated intentionally with `## Scope` for template conformance: executing lines, implementing the decided items, CI automation.

## 5. Protected invariants this task must not weaken

- The release gate is operator-held: nothing here authorizes tagging, publishing, or pushing.
- No secrets in the repo.
- HITL: both human-acceptance gates are recorded verdicts; the agent never sets done.

*End of TASK-IMP-087.*
