# CUO contract summary

This file is a quick reference. The normative spec is `AGENTS.md`.

## Public surface

### CLI

```
cyberos-cuo catalog
cyberos-cuo route <query> [--invoke] [--record]
```

* `catalog` — enumerate the skills the router knows about.
* `route` — score the query against the catalog, return a `RoutingDecision`. With `--invoke`, also execute the chosen skill. With `--invoke --record`, also append the decision to the BRAIN audit chain.

### Python API

```python
from cuo.core.catalog import discover, SkillEntry
from cuo.core.router import route, RoutingDecision
from cuo.core.invoker import invoke, InvocationResult
from cuo.core.memory_bridge import record_decision

catalog: list[SkillEntry] = discover(skill_root_path)
decision: RoutingDecision | None = route(query, catalog)
if decision:
    result: InvocationResult = invoke(decision.skill_name, decision.arguments.get("input"), skill_root_path)
    record_decision(asdict(decision), asdict(result), memory_module_root)
```

## Data shapes

### `SkillEntry`

```python
@dataclass
class SkillEntry:
    name: str
    description: str
    capabilities: list[str]      # allowed-tools, space-split
    region: str | None           # e.g. "VN"
    collection: str | None       # e.g. "cyberskill-vn"
    skill_dir: Path
```

### `RoutingDecision`

```python
@dataclass
class RoutingDecision:
    skill_name: str
    confidence: float            # 0.0-1.0
    arguments: dict              # skill-specific structured args
    rationale: str               # human-readable explanation
    alternative_skills: list[str]
```

### `InvocationResult`

```python
@dataclass
class InvocationResult:
    skill_name: str
    exit_code: int
    output: str
    stderr: str
```

## Confidence threshold

* Score `>= 3.0` (on the 0–10 rule-based scale) → claim a match.
* Score `< 3.0` → return `None`. CUO escalates to the operator.

## Phase status

| Phase | Status |
|---|---|
| 1 — Rule-based router | shipped |
| 2 — LLM-driven router | pending |
| 3 — Multi-skill chains | pending |
| 4 — Persona switching (sub-CUOs: CPO, CTO, …) | pending |

See `AGENTS.md` for protocol normativity, `ROUTING.md` for the scoring rationale, `CHANGELOG.md` for release history.
