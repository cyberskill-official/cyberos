# Refinement Evals (Aspect 3.2)

Each adopted REF-NNN should have a folder here with:

```
runtime/tests/refinements/REF-NNN/
├── capability.md       # what new behavior does this REF enable?
├── capability.test.py  # pass = memory rejected pre-REF now accepted (or vice versa)
├── regression.md       # what existing memories does this REF affect?
└── regression.test.py  # pass = all existing memories still validate
```

## Adoption gate

`cyberos eval REF-NNN` must pass BOTH:
1. capability eval — confirms the new behavior works
2. regression eval — confirms no existing memory breaks

Failing either blocks the §0.5 protocol upgrade per `eval-harness` skill.

## Template

Use `runtime/tests/refinements/_template/` as starting point. Copy to `REF-NNN/` and fill in.

## Naming

Test files use pytest-style `test_*` functions but standalone-runnable too:
```python
def test_capability():
    """REF-NNN capability eval: <what>"""
    assert ...

def test_regression():
    """REF-NNN regression eval: <existing memory class still validates>"""
    assert ...

if __name__ == "__main__":
    test_capability()
    test_regression()
```
