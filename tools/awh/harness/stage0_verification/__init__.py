"""Stage 0 — deterministic verification gates and containment.

The strongest empirical result in the survey is that *infrastructure-level*
controls cut reward hacking and silent failure far more reliably than prompts
(METR 2025; ImpossibleBench arXiv:2510.20270; Anthropic containment 2026).

This package provides the deterministic layer:

  policy            pure decision engine for PreToolUse deny rules
  hooks/            Claude Code hook entry points (PreToolUse, Stop, PostToolUse)
  readonly          make test/scoring files read-only (or hidden) at the OS level
  settings.template.json  wiring for ~/.claude/settings.json or .claude/settings.json
"""
