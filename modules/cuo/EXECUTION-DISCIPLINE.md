# CyberOS execution discipline - continuous run, halt-only conditions

Spec status: Normative. The key words MUST, MUST NOT, SHALL, SHALL NOT, SHOULD, MAY are BCP-14 (RFC 2119, RFC 8174) when capitalised.

Scope: every CUO workflow, and every agent (Claude, Codex, or any other) that executes a CyberOS build chain. A USER instruction in the active chat takes precedence (memory protocol AGENTS.md §0.1). Absent such an instruction, this doctrine governs when an agent may stop. It was added at the operator's explicit request (2026-06-20): CyberOS must not insert a pause unless the operator's decision or a manual run is genuinely required.

## §1  Continue by default

An agent executing a CyberOS workflow SHALL run continuously to completion. It MUST NOT stop to request confirmation, approval, permission, or acknowledgement for any work it can perform and verify by itself. Reaching the end of one unit of work (a slice, a phase, an FR) is NOT a reason to pause: the agent advances to the next eligible unit on its own.

## §2  The only permitted halts (exhaustive)

An agent MAY stop ONLY for one of these four conditions. Nothing else is a halt.

1. **Operator-decision fork.** A choice the spec, the codebase, and sensible defaults cannot resolve, AND that materially changes direction or is costly to reverse (ADR-class: a real architectural or product fork). Self-resolvable ambiguity is NOT a fork. Pick the obvious default, record it (an ADR or a one-line note), and continue.
2. **Manual or operator-only action.** An action the agent must not perform itself: `git push`, deploy or promote, a destructive operation (hard delete, purge, history rewrite, dropping data), entering or rotating a secret or credential, a financial action, or anything policy reserves for the human. The agent prepares and names the action; the operator runs it.
3. **Hard blocker past the budget.** A failure the agent cannot self-resolve within the workflow's circuit-breaker budget (for example, 5 consecutive test failures). The agent documents the blocker, routes the unit back cleanly (status to `ready_to_implement` with the reason), and moves to the next eligible unit. It does NOT spin, and it does NOT silently ship a partial result.
4. **Operator stop signal.** An explicit Ctrl-C or workflow-stop event.

## §3  Self-resolve and continue (never a halt)

The following are the agent's own responsibility. It fixes them and proceeds; it MUST NOT pause to ask about any of them:

- compile errors, type errors, lint, clippy, formatting;
- a test the agent's own change broke;
- a module gate (awh or caf) that goes RED on the agent's own change;
- a choice between equivalent implementations;
- the order of slices within an FR, or of FRs within the backlog;
- routine re-verification.

Verification, not confirmation, earns the right to proceed. When the build, the tests, and the module gates pass, that IS the permission.

## §4  Reporting is not pausing

An agent MAY emit a progress note at a milestone. Emitting a note MUST NOT block execution waiting for a reply unless a §2 condition also applies. "I finished X, continuing to Y" is a report. "I finished X, shall I do Y?" is a forbidden pause when Y is self-resolvable.

## §5  Relationship to the lifecycle

This doctrine sharpens, and is consistent with, `chief-technology-officer/ship-feature-requests.md` §12 (no partial-ship-and-pause within an FR) and its outer loop §11 (`while ! stop_signal`). §12's "pause between FRs" is NOT a mandatory stop: the outer loop advances to the next eligible FR on its own, and the agent halts between FRs only when a §2 condition applies. The same rule governs the `architect-new-system` workflow and any net-new project build (including projects CyberOS drives through an external executor such as Codex).
