# TASK-IMP-102 implementation plan

1. **Define the normative half** (clause 1.1) - body + frontmatter minus status/shipped/routed_back_count/memory_chain_hash; the definition lives in task-audit §12 (the contract) and in task-reconcile's `normativeHalf()` (the reader), named identically in both.
2. **Record both fields** (1.1, 1.2) - `audited_body_sha256_prefix` as the binding; `audited_file_sha256_prefix` retained as provenance with the caveat stated in the contract.
3. **Re-state the skill's own claims** (1.3) - payload_hash_field, fixity_notes, re_entrancy move to the body hash; the file hash was never a stable key and the document now says so.
4. **Reader preference** (1.4) - R1 prefers the body field (direct, commit-independent); legacy audits fall back to the audit-commit reconstruction and are named legacy.
5. **Coverage** (1.5) - t06's four arms: flip-proof, drift-caught, legacy-honest, legacy-dishonest.

Deliberate non-changes: no corpus backfill, no task-lint enforcement of the new field (a rubric change, separate decision), no change to what an audit judges.
