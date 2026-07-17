# TASK-IMP-102 gate-log evidence (implementing -> ready_to_review)

E1 - suite (AC 1-3), full run: `test_task_reconcile: pass=6 fail=0`
  ok t06_body_binding_preferred - four arms:
    (a) body-bound audit + lifecycle flip reviewing->done  -> R1 pass, ZERO binding-gap notes
    (b) body-bound audit + clause edit after the audit     -> R1 red, "SPEC DRIFT" naming both hashes
    (c) legacy audit (honest, hashed committed bytes)      -> resolves "via the audit commit", R1 pass
    (d) legacy audit bound to bytes no commit carries      -> "legacy audit: no audited_body_sha256_prefix"
                                                              named, R1 still pass (a note is not a verdict)

E2 - AC 4, the contract (modules/skill/task-audit/SKILL.md):
  $ grep -c "audited_body_sha256" modules/skill/task-audit/SKILL.md            -> 4
  :86   payload_hash_field: audited_body_sha256   # the VERIFIABLE binding (TASK-IMP-102)
  :146  fixity_notes: ... "Two runs against the same audited_body_sha256 produce identical reports ..."
  :191  re_entrancy: idempotent_on_audited_body_sha256
  §12   "Byte-binding: what an audit records about what it judged" - both fields defined, the
        normative-half field list named, the legacy read rule stated.

E3 - the convention's first witness: TASK-IMP-102's own audit.md carries
  audited_body_sha256_prefix: 5c530084993c87d5, computed over its own normative half. It
  survived this batch's own status flips (draft -> ready_to_implement -> implementing -> ...)
  with R1 green - the property the old field could never have.

E4 - whole-tree gates: 25/25 suites (A 8/8, B 7/7, C 9/9); build ok (skills=53);
  chain OK 25 referenced/53 vendored; sync OK 1.0.0 across 7 artifacts;
  anchors OK 450 references resolved.
