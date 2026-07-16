---
audit_template_version: "audit_rubric@2.0"
audited_file: "docs/tasks/improvement/TASK-IMP-083-hookspath-aware-status-hook/spec.md"
audited_file_sha256_prefix: "ae26442c5686b994"
rubric_version: "audit_rubric@2.0"
skill_id: "task-audit"
skill_version: "1.0.0"
last_audit_at: "2026-07-16T14:28:40Z"
overall_status: "pass"
iterations: 2
issue_counts: { total: 2, open: 0, needs_human: 0, fixed: 1, wontfix: 1 }
trace_id: "cowork-cyberos-improvement-batch-2026-07-16"
caller_persona: "operator:stephen-cheng"
---

# TASK-IMP-083-hookspath-aware-status-hook spec audit - audit_rubric@2.0

Families walked: FM (all pass - task@1, title <=72, closed enums, ISO created_at, no
UNREVIEWED markers, corpus extras additive), SEC (seven H2s present and non-empty, one H1),
COND (COND-004 three labeled bullets present; others not triggered), QA (metrics carry
baseline/target/deadline grounded in run evidence; >=3 distinct alternatives; Out-of-scope
subsection; no unsourced numeric targets; no cross-team claims), SAFE (one sourced block,
closed, unnested, clean scan), TRACE (every numbered clause cited by >=1 AC; every AC names
a test or a justified ops verification; every test path is in new_files or exists on disk;
draft status exempts TRACE-004; no deferred slices).

ISSUE ISS-001 (QA-006/TRACE-001, fixed): authoring investigation of uninstall.sh revealed
its ownership test is still the head-5 heuristic install.sh condemns (uninstall.sh:24-28) -
a short foreign hook carrying our appended block would be DELETED whole. Clause 1.5
extended, AC 8 + edge-case row added. This is the audit loop catching a live defect.
ISSUE ISS-002 (QA-008, wontfix-info): "husky adapters" named in Non-Goals reference a tool
family, not a team dependency; generic hooksPath resolution covers their directories.

SUMMARY
verdict:         pass
issues_open:     0
issues_human:    0
iterations:      2
next_action:     ship

## §gate-log

Populated during implementation (ship-tasks testing phase).
