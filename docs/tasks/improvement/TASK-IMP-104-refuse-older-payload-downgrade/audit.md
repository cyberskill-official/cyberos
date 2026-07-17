---
task_id: TASK-IMP-104
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: c86b48ec807e7fad
audited_body_sha256_prefix: b258a2cedb4f0347
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 84 lines, 6 §1 clauses, 5 ACs, 5 edge cases. Gap verified on main: install.sh:21 reads avail_ver, :41 prints it, nothing compares; version.sh already carries a comparator. Passes after 6 findings.

## §2 - Findings (all resolved)

### ISS-001 - depends_on was empty despite a real ordering constraint with 103
Both guard the same vendor step; a refused downgrade must not take a lock it will not use, and shipping this first forces 103 to re-open the same lines. Resolved: `depends_on: [TASK-IMP-103]` with reciprocal `blocks` on 103, §1 #1.1 makes the order normative, and Dependencies names TASK-IMP-101's evidence gate.

### ISS-002 - A second comparator would drift from the first
Two implementations of one comparison eventually disagree. Resolved: §1 #1.2 forbids it; AC 5 verifies the negative structurally.

### ISS-003 - `unknown` payload version could be read as older
install.sh already sets `avail_ver=unknown` when VERSION is absent, and ordering against unknown is undefined. Resolved: §3 edge case makes unknown non-comparable - it proceeds rather than refusing.

### ISS-004 - A gate with no key gets bypassed destructively
Refusing with no override invites `rm -rf .cyberos`, which loses the operator's config - worse than the downgrade. Resolved: §1 #1.4 provides the override and requires both versions recorded.

### ISS-005 - Silent equal-version re-install could become noisy
Adding output to the documented idempotent path makes every re-install look eventful. Resolved: §1 #1.5 requires no new output; AC 3 asserts it.

### ISS-006 - Pre-release ordering is undefined by this spec
`1.0.0-rc1` vs `1.0.0` has no obvious answer, and inventing one contradicts the shared comparator. Resolved: §3 defers to the existing comparator and pins its behavior in the suite.

## §3 - Resolution

All 6 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-104 audit.*
