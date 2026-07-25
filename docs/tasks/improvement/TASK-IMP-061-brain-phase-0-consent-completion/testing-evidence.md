# TASK-IMP-061 testing evidence

```
cd modules/memory && PYTHONPATH=cyberos:runtime pytest tests/test_personnel_consent.py -q
........                                                                 [100%]
8 passed in 0.18s
```

Cases: empty store; operational skip; missing consent; null event; unresolved;
file resolve; audit_id resolve; README not an event.
