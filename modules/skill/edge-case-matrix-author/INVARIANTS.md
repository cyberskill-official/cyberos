# `edge-case-matrix-author` - invariants

Lifted from SKILL.md's normative prose (FR-SKILL-118 AC 2 discipline: no invariant without a prose source).

1. One matrix row per category-and-trigger; a row without a covering-test pointer is incomplete, not optional.
2. SECURITY rows cite test paths that exist (or will exist in this FR's new_files) - never aspirational prose.
3. DEGRADATION rows always carry both detection and recovery.
4. The matrix precedes implementation; rows added after code lands are marked as such.

Enforced at audit time by `edge-case-matrix-audit` per RUBRIC.md (edge_case_matrix_rubric@1.0).
