# CyberOS execution discipline - continuous run, halt-only conditions

Spec status: Normative. The key words MUST, MUST NOT, SHALL, SHALL NOT, SHOULD, MAY are BCP-14 (RFC 2119, RFC 8174) when capitalised.

Scope: every CUO workflow, and every agent (Claude, Codex, or any other) that executes a CyberOS build chain. A USER instruction in the active chat takes precedence (memory protocol AGENTS.md §0.1). Absent such an instruction, this doctrine governs when an agent may stop. It was added at the operator's explicit request (2026-06-20): CyberOS must not insert a pause unless the operator's decision or a manual run is genuinely required.

## §1  Continue by default

An agent executing a CyberOS workflow SHALL run continuously to completion. It MUST NOT stop to request confirmation, approval, permission, or acknowledgement for any work it can perform and verify by itself. Reaching the end of one unit of work (a slice, a phase, a task) is NOT a reason to pause: the agent advances to the next eligible unit on its own.

## §2  The only permitted halts (exhaustive)

An agent MAY stop ONLY for one of these four conditions. Nothing else is a halt.

1. **Operator-decision fork.** A choice the spec, the codebase, and sensible defaults cannot resolve, AND that materially changes direction or is costly to reverse (ADR-class: a real architectural or product fork). Self-resolvable ambiguity is NOT a fork. Pick the obvious default, record it (an ADR or a one-line note), and continue.
2. **Operator-only action — by POLICY, never by capability.** An action policy reserves for the human: `git push`, deploy or promote, a destructive operation (hard delete, purge, history rewrite, dropping data), entering or rotating a secret or credential, a financial action, **accepting a task across a human-acceptance gate (see §2a)**. The agent prepares and names the action; the operator runs it.

   **This list is closed, and it is about permission — not about reach.** "The agent cannot do it from a shell" was never a reason to be on it. Where the agent has OS or browser control (see §2b), an action being *outside the terminal* is NOT a halt: it drives the GUI itself and continues. Halting for a task it is merely inconvenient to reach is a §4 forbidden pause wearing a §2 costume.
3. **Hard blocker past the budget.** A failure the agent cannot self-resolve within the workflow's circuit-breaker budget (for example, 5 consecutive test failures). The agent documents the blocker, routes the unit back cleanly (status to `ready_to_implement` with the reason), and moves to the next eligible unit. It does NOT spin, and it does NOT silently ship a partial result.
4. **Operator stop signal.** An explicit Ctrl-C or workflow-stop event.

## §2b  OS and browser control — reach is not permission (operator request, 2026-07-15)

Where the executing agent has OS or browser control, it SHALL use it rather than halt. Added
at the operator's explicit request: *"agents can control OS/browser if necessary, instead of
stop and tell user do."*

**Use it for** anything the agent could do in a shell if only the thing had a shell: clicking
through a GUI installer or a settings pane, reading a dashboard the CLI does not expose,
driving a local admin UI, opening a rendered page to check it actually rendered, filling a
local form, restarting a local app. Reach for a dedicated API or CLI first when one exists —
GUI driving is the fallback, not the default, because it is slower and more brittle. But an
action being GUI-only is NOT a reason to stop.

**Do NOT use it to route around §2.** The halt list is policy, and a policy halt is not a
capability problem to be solved with a mouse:

- `git push` stays operator-only. Clicking "Sync" in a git GUI is the same push. So is
  merging a PR in a browser.
- Deploys, promotes, destructive ops, secrets, financial actions: same. A browser makes them
  *easier to do*, not *permitted*.
- Both HITL gates (§2a) stay human. Clicking "approve" in a UI on the human's behalf is
  forging the verdict the gate exists to collect — the single worst thing this section could
  be misread to license.

The test: **would this still be operator-only if the agent had hands?** If yes, it is §2 and
you halt. If the only obstacle was reach, it is §3 and you continue.

Report what you drove (§4). Screenshots or a step list of what was clicked belong in the
run's evidence, so a human can audit an action they did not watch.

## §2a  Human acceptance is required (HITL)

HITL is required platform-wide, not optional. This section supersedes any earlier text (in this file, in `STATUS-REFERENCE.md`, or in a workflow `.md`) that described human-in-the-loop as optional or the lifecycle as fully auto-flipping.

Two lifecycle transitions are human-acceptance gates and therefore fall under §2 condition 2 (operator-only action): review acceptance (`reviewing -> ready_to_test`) and final acceptance (`testing -> done`). At each of these, the agent MUST drive the unit up to the gate with every machine gate green (coverage, TRACE-004, awh, caf), record the evidence, and then HALT for a recorded human verdict. The agent MUST NOT self-certify either transition, and MUST NOT set a task to `done` itself.

This does not loosen §1 or §3. Between the gates the agent still runs continuously and self-resolves everything it can verify (compile, lint, tests it broke, a red module gate on its own change); it does not pause for self-resolvable work. The only added stops are the two human-acceptance verdicts. A human recording acceptance is the permission to cross the gate, exactly as a green build is the permission to proceed within a phase.

## §2c  Explain before you ask (operator request, 2026-07-18)

Every halt that asks a human to decide — a §2 condition 1 fork, and both §2a acceptance gates —
MUST deliver its explanation BEFORE its options. Added at the operator's explicit request: *"when
need decision you need to explain in easy to understand way, include the context, then give me
decision questions."*

The explanation is part of the gate, not a courtesy. A human who cannot understand what they are
approving cannot judge it, and a gate that collects an uninformed verdict has recorded a signature,
not a decision — it launders the agent's own choice through a human and makes it look accountable.
That is worse than no gate.

In order, the agent MUST:

1. **State the decision in plain language** — what is being decided, and why now. No jargon, no
   bare identifiers, no assumed context: the decider was not in the loop that produced the question.
2. **Give the context needed to judge it** — what the thing IS and what it is for, BEFORE asking
   whether to ship it. If answering would require the human to go read the code, the explanation is
   not finished.
3. **Then present 2-4 options**, each with its consequence and whether it is reversible.

**An option's framing MUST NOT smuggle the agent's conclusion in as fact.** A claim inside an option
is a claim, and §3's standard applies to it before it is stated: check it on the real target, or
mark it unverified. "Option C is infeasible — nothing records X" is a verdict the human is being
asked to reach, dressed as a constraint they must accept; absent a command that proves it, the agent
MUST say it has not checked rather than assert it. An unverified claim that steers a human verdict
is the worst instance of the defect TASK-IMP-124 names.

"What do you mean?" is a failed gate, not a failed human. Re-explain; do not re-ask.

## §3  Self-resolve and continue (never a halt)

The following are the agent's own responsibility. It fixes them and proceeds; it MUST NOT pause to ask about any of them:

- compile errors, type errors, lint, clippy, formatting;
- a test the agent's own change broke;
- a module gate (awh or caf) that goes RED on the agent's own change;
- a choice between equivalent implementations;
- the order of slices within a task, or of tasks within the backlog;
- routine re-verification;
- **an action that is GUI-only rather than operator-only** (§2b) — drive it and continue;
- **a check the agent can run on the real target** rather than infer. Verifying on a proxy
  and reporting the proxy's answer is not verification. If the claim is about the operator's
  machine, the installed payload, or the rendered page, go look at that thing.

Verification, not confirmation, earns the right to proceed. When the build, the tests, and the module gates pass, that IS the permission.

## §4  Reporting is not pausing

An agent MAY emit a progress note at a milestone. Emitting a note MUST NOT block execution waiting for a reply unless a §2 condition also applies. "I finished X, continuing to Y" is a report. "I finished X, shall I do Y?" is a forbidden pause when Y is self-resolvable.

## §5  Relationship to the lifecycle

This doctrine sharpens, and is consistent with, `chief-technology-officer/ship-tasks.md` §12 (no partial-ship-and-pause within a task) and its outer loop §11 (`while ! stop_signal`). §12's "pause between tasks" is NOT a mandatory stop: the outer loop advances to the next eligible task on its own, and the agent halts between tasks only when a §2 condition applies. The same rule governs the `architect-new-system` workflow and any net-new project build (including projects CyberOS drives through an external executor such as Codex).

## Run-state manifests (TASK-CUO-206)

Interrupted ship runs resume from `docs/tasks/.workflow/<task-ID>.ship.json` (ship-manifest@1) instead of restarting the 31-step chain - see 'Resume semantics' in `chief-technology-officer/workflows/ship-tasks.md`. The manifest is a cache: artefacts are re-hashed on resume and HITL gates always re-ask. A session ending mid-task is therefore recoverable state, not lost work - but it is never a licence to skip a gate.

