# `runtime/tests/` — Integration tests + skill fixtures

End-to-end tests that exercise the umbrella binary, the memory-mutation API, and individual skill runners. Unit-test scope is small (most logic is integration-shaped).

## Layout

```
runtime/tests/
├── skills/              ← per-skill fixture inputs + expected outputs
│   └── fr-with-tasks/   ← reference fixtures for the fr-with-tasks runner
├── integration/         ← chain-end-to-end tests
└── conftest.py          ← shared pytest fixtures
```

## Running

```shell
# All tests
cd runtime/tests && pytest

# A single skill's tests
pytest runtime/tests/skills/fr-with-tasks/

# With live LLM (Anthropic API key required; flips into Tier α.10 streaming mode)
ANTHROPIC_API_KEY=... pytest -m live_llm
```

## Adding a new test

1. Drop input fixtures in `runtime/tests/skills/<skill_id>/inputs/`.
2. Drop expected-output snapshots in `runtime/tests/skills/<skill_id>/expected/`.
3. Author the test function in `runtime/tests/skills/<skill_id>/test_<scenario>.py` using `pytest`.

## Test corpus

The cross-skill test corpus (Tier α.5) lives under `runtime/tests/skills/_corpus/`. Every chain skill MUST pass these scenarios before its CHANGELOG is bumped.

## Related

- BaseSkillRunner that tests target: [`../skill_runners/base.py`](../skill_runners/base.py)
- Cross-skill validator (Tier α.7): [`../tools/cyberos_cross_skill.py`](../tools/cyberos_cross_skill.py)
