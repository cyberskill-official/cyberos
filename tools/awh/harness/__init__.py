"""auto-work-harness — hardening toolkit for autonomous coding agents.

Implements the prioritized roadmap from
"Pushing Autonomous Coding Agents Further (2025-2026)":

    Stage 0  harness.stage0_verification  — deterministic gates, read-only tests, egress sandbox
    Stage 1  harness.stage1_measurement   — pass@k statistics, golden set, multi-seed eval runner
    Stage 2  harness.stage2_structural    — LLM-judge verifier, property-based + mutation testing
    Stage 3  harness.stage3_parallel      — git worktree manager, planner -> worker -> verifier

Design principle (Anthropic, "How we contain Claude across products", 2026):
    "Design for containment at the environment layer first, then steer behavior
     at the model layer."
"""

__version__ = "0.1.0"
