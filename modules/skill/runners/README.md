# `skill/runners/` — LLM-driven skill execution framework

> **Status: legacy.** These Python runners ship today. They are preserved for parity testing while the Rust + Wasmtime host is built out (see `../docs/SPEC.md` migration plan). They will be retired in Phase 7 once WASM execution proves equivalent behaviour on the full skill catalogue.

---


Concrete Python runners that execute the 11 chain skills under [`../skills/cuo/`](../skills/cuo/). Each runner wraps an LLM call (Anthropic SDK), applies that skill's `INVARIANTS.md` validations, and re-prompts on validation failure (multi-iteration self-audit, Tier α.3).

## Files

| File | Purpose |
| --- | --- |
| [`base.py`](base.py) | `BaseSkillRunner` framework: cache, telemetry, streaming output, iteration loop. All runners subclass this. |
| [`task_with_subtasks.py`](task_with_subtasks.py) | Reference implementation — runner for `cuo/cpo/task-with-subtasks` (the most-used skill). |

## How to add a new runner

1. Copy `task_with_subtasks.py` to `<skill_id>.py` (e.g. `fr_audit.py`).
2. Override:
   - `skill_id` — matches the SKILL.md location under `docs/skills/`.
   - `output_filename_pattern` — what the emitted artefact is called.
   - `interview_questions` — standalone-mode prompts (used only when run outside a chain).
   - `build_prompt(inputs, prior_artefacts)` — compose the prompt from SKILL.md + contract templates + inputs.
   - `validate_emit(body, inputs)` — run that skill's INVARIANTS against the emitted body. Return a list of findings.
3. `cyberos chain run --with-llm` discovers runners by `skill_id` and dispatches automatically (`base.load_runner(...)`).

## Invocation

Directly:
```shell
python3 runtime/skill_runners/task_with_subtasks.py <output_dir> --pitch "..." [--spec-file path] [--max-iterations 3]
```

Through the chain:
```shell
cyberos chain run --pitch "..." --profile solo --with-llm
```

## Related

- The deterministic-runner framework is documented in [`../docs/README.md`](../docs/README.md) Part 21 (Tier α).
- Validation rules referenced by `validate_emit` are codified in each skill's `INVARIANTS.md`.
- Cache + telemetry tooling: `../tools/benchmark.py`, `cyberos analytics`.
