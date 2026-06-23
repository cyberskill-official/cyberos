"""Stage 3 — selective parallelism (only after Stages 0-2).

The survey's safe coding wins are isolation-based, not chatty swarms:

  worktree    git worktree-per-task isolation (incident.io's production default),
              with a collision guard, per-worktree env/port isolation, and
              serialized merge THROUGH the integration layer (never agent-to-agent).
  pipeline    planner -> parallel workers (isolated worktrees) -> verifier gate
              (Augment Intent / Anthropic orchestrator-worker shape). Writes stay
              serialized; cross-agent handoff is via filesystem artifacts.

Rule of thumb (LangChain): read actions parallelize; write actions don't.
"""
