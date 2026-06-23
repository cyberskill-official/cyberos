"""Stage 1 — the measurement pillar (build this before any multi-agent work).

A custom golden set from your own repos, run multi-seed, scored against a
*baseline* (block-on-regression, never block-on-absolute-threshold), is the
discipline that turns an agent protocol into versioned, tested software.

  goldenset   task schema + loader (YAML)
  runner      multi-seed execution, pass@k/pass^k report, regression gate

Backed by harness.common.stats (Codex pass@k, tau-bench pass^k, power analysis).
"""
