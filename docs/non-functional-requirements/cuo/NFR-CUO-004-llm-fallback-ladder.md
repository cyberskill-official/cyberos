---
id: NFR-CUO-004
title: "CUO LLM invoker fallback ladder — MockInvoker → SubprocessInvoker → LLMInvoker"
module: CUO
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of invoker selection follows the documented ladder; explicit selection always honoured"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-CUO-101, TASK-CUO-103]
---

## §1 — Statement (BCP-14 normative)

1. The CUO supervisor's `select_invoker("auto")` function **MUST** prefer in priority order: (a) `SubprocessInvoker` if `cyberos-skill` binary is on `$PATH`, else (b) `MockInvoker`.
2. Explicit selection (`--invoker llm`, `--invoker subprocess`, `--invoker mock`) **MUST** always override the ladder; no auto-fallback when explicit.
3. The selected invoker **MUST** be recorded in the per-chain audit row so post-hoc readers know what ran.
4. `LLMInvoker` **MUST NOT** be auto-selected — it has cost and side-effect implications and must be opt-in.
5. When `LLMInvoker` is selected without `ANTHROPIC_API_KEY` set, it **MUST** fall back to mock-llm behaviour (deterministic stub responses), logging the fallback to the audit row.

## §2 — Why this constraint

Three invokers exist for distinct purposes: Mock for fast tests, Subprocess for production execution against shipped SKILL CLI, LLM for unimplemented skills that should be simulated by direct LLM call. Each has different cost + side-effect surface. The auto ladder favours the cheapest-safe option. Explicit selection lets operators override (e.g., force mock for chaos testing). The audit record makes the choice replayable. The "LLMInvoker not auto-selected" rule prevents an accidental config drift from rerouting production chains to live LLM calls — that has both cost and correctness consequences.

## §3 — Measurement

- Counter `cuo_invoker_selected_total{invoker, mode=auto|explicit}`.
- Gauge `cuo_llm_invoker_no_key_fallback_count` — surfaces missing config.

## §4 — Verification

- Unit test `modules/cuo/tests/test_invoker_select.py` (T) — tests every ladder branch + explicit selection.
- Integration test (T) — execute chains with each invoker; assert audit row records correct invoker.
- CI guard (T) — assert `select_invoker("auto")` never returns `LLMInvoker`.

## §5 — Failure handling

- Auto-select returns LLMInvoker → sev-2; ladder broken; revert immediately.
- LLMInvoker selected without API key in production → sev-3; investigate config; mock fallback is logged but not silent.
- Subprocess binary missing on production node → sev-2; node has lost the CLI; reprovision.

---

*End of NFR-CUO-004.*
