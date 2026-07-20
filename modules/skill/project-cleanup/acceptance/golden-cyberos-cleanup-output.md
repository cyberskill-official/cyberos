# project-cleanup — golden expected output (cyberos flavor)

This is the expected structural shape of the cleanup report for a cyberos repo with 4 stale fragments in `docs/tasks/ai/`.

The skill is NOT byte-deterministic (timestamps + sha varies per run). Acceptance check is **structural**: the run MUST produce a report that matches the categories + counts below.

---

```
Project: /path/to/cyberos
Scope detected: cyberos (task-audit skill present)
Started: <ISO-8601>

Phase 1 — Inventory
  Files scanned: <N>
  Fragments detected: 4
    - docs/tasks/ai/SLICE_1_AUDIT_SUMMARY.md (132 lines)
    - docs/tasks/ai/SLICE_2_AUDIT_SUMMARY.md (74 lines)
    - docs/tasks/ai/SLICE_3_AUDIT_SUMMARY.md (17 lines)
    - docs/tasks/ai/AI_GATEWAY_COMPLETE_SUMMARY.md (45 lines)
  Suspicious leftovers: 4 (all _SUMMARY.md)
  Orphan audits: 0
  Broken links: 0

Phase 2 — Absorb proposals
  Proposal 1: SLICE_1_AUDIT_SUMMARY.md → ai/README.md (rationale: same_dir_readme)
  Proposal 2: SLICE_2_AUDIT_SUMMARY.md → ai/README.md
  Proposal 3: SLICE_3_AUDIT_SUMMARY.md → ai/README.md
  Proposal 4: AI_GATEWAY_COMPLETE_SUMMARY.md → ai/README.md
  Operator approves all 4 → merge into ai/README.md under "Historical: AI Gateway slice arc"

Phase 3 — Delete leftovers
  4 files queued: all 4 absorbed fragments
  Operator confirms → delete

Phase 4 — Verify state (cyberos flavor)
  Task DAG coherence: 0 errors ✓
  Missing audits: 0 ✓
  Specs not at 10/10: 0 ✓
  Per-module spec/audit balance: 24/24 modules balanced ✓
  BACKLOG.md sync header present: ✓
  CHANGELOG.md last entry: <N> days ago

Overall: PASS

Report written: /path/to/cyberos/CLEANUP_REPORT_<timestamp>.md
Report SHA-256: <hash>
```

## What "PASS" means

- 4 fragments detected → 4 absorbed → 4 deleted (no leftover)
- Task DAG coherent (reciprocity errors = 0)
- All audits present + at 10/10
- No structural drift

## What "FAIL" would look like

If any of:
- Reciprocity errors > 0 (broken DAG)
- Missing audits > 0
- Specs not at 10/10 > 0
- Operator declined critical absorbs leaving fragments behind

→ overall: FAIL + report flags each issue.
