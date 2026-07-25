# TASK-IMP-146 testing evidence

Final machine-gate pass on `batch/11-wave2-residuals`:

```text
suites: pass=55 fail=0 skip=1
PASS  test
PASS  doctor (16/16; MMR 234 leaves)
GATES: GREEN (machine gates only).
```

The single skip is the repository's documented macOS release-assets skip (GNU tar runs in
Ubuntu CI). Targeted suites:

- `test_memory_append.sh`: 9 passed, 0 failed.
- `test_hitl_lock.sh`: 7 passed, 0 failed.
- Disk-exhaustion artifacts made the first full gate attempt red; after removing stale
  completed-test temp directories, both affected suites passed standalone and the full
  gate rerun was green. No code change was made to mask the environmental failure.

