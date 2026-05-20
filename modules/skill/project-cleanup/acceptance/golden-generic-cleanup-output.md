# project-cleanup — golden expected output (generic flavor)

For a generic repo without feature-request-audit skill (or the legacy feature-request-audit skill), the skill
runs the generic verification pipeline.

---

```
Project: /path/to/generic-repo
Scope detected: generic (no cyberos signature)
Started: <ISO-8601>

Phase 1 — Inventory
  Files scanned: <N>
  Fragments detected: 1
    - notes/draft_idea.md (23 lines, matches draft_*.md pattern)
  Suspicious leftovers: 1
  Orphan audits: N/A (generic flavor)
  Broken links: 1
    - README.md → docs/old-spec.md (file does not exist)

Phase 2 — Absorb proposals
  Proposal 1: notes/draft_idea.md → notes/README.md (rationale: same_dir_readme if exists; else largest_sibling_md)
  Operator approves → merge

Phase 3 — Delete leftovers
  1 file queued
  Operator confirms → delete

Phase 4 — Verify state (generic flavor)
  Top-level README.md: ✓
  CHANGELOG.md: ⚠️  missing
  LICENSE: ✓
  Broken markdown links: 1 ⚠️
  Orphan markdowns: 0
  
Overall: WARN (CHANGELOG missing, 1 broken link)

Report written: /path/to/generic-repo/CLEANUP_REPORT_<timestamp>.md
```

## Acceptance rule

Generic-flavor cleanup MUST:
- Detect the 1 seeded fragment
- Propose absorb to nearest parent
- Detect the 1 seeded broken link
- Detect the missing CHANGELOG.md
- Report overall: WARN (not FAIL — broken link + missing changelog are
  warnings, not blockers)
