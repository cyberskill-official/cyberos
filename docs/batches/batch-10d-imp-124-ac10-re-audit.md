---
batch: batch/10d-imp-124
artefact: ac10-re-audit
recorded: 2026-07-25
auditor: Stephen Cheng (session operator) — distinct from the IMP-124 implementer pen for this wave
rule_under_test: TRACE-007 + amended COND-004 (TASK-IMP-124)
---

# AC 10 — recorded re-audit of the three motivating documents

Per TASK-IMP-124 §1.10 / AC 10: TRACE-007 is judgment-family; this artefact records a model/human audit verdict over the pinned revisions, not a shell assertion that an audit file *says* FAIL.

## 1. TASK-IMP-122 rewrite 4 AC 10 — `63705483:280` → FAIL (TRACE-007, NUMERIC)

- **Claim (quoted):** `102dc507` paired with "1525 files per side".
- **Derivation present in that revision?** No command/composition that produces 1525 for the mandated cone.
- **Re-run:** mandated cone (cuo plugin mcp lib docs-tools + memory−store + 6 root scripts) → **1534** / `102dc507`; rejected cone → **1525** / `86cafee8`.
- **Verdict:** TRACE-007 FAIL — derivation ABSENT for the originated 1525; the number supports the rejected cone's scope, narrower than / mismatched to the claim that pairs it with `102dc507`.

## 2. TASK-IMP-121 — `15894b1e:spec.md:168` → FAIL (TRACE-007, UNIVERSAL NEGATIVE)

- **Claim (quoted):** byte-exact restore of a no-trailing-newline operator hook is "IMPOSSIBLE" / "no uninstall-side rule can invert it".
- **Derivation present?** Measurement of the author's own line-oriented candidate strip (16→17), generalised to all rules.
- **Counter-example attempted:** byte-oriented awk strip → 6B→6B BYTE-EXACT; line-oriented → 6B→7B DIFFERS.
- **Verdict:** TRACE-007 FAIL — author's-own-candidate measurement does NOT discharge a universal negative; scope of derivation narrower than claim.

## 3. TASK-IMP-122 rewrite 5 disclosure — `15894b1e:spec.md:279-304` → PASS (amended COND-004)

- **Scope partition:** carries `re-derived and CONFIRMED`, `re-derived and CORRECTED`, `measured and ADDED`; names corrections (`1525 → 1534`); treats prior audit figures as claims to verify.
- **Verdict:** amended COND-004 PASS — worked prototype of the partition shape.

## Summary

| Document | Rule | Verdict |
|---|---|---|
| `63705483` AC 10 | TRACE-007 NUMERIC | FAIL |
| `15894b1e` IMP-121 `:168` | TRACE-007 UNIVERSAL NEGATIVE | FAIL |
| `15894b1e` IMP-122 disclosure | COND-004 partition | PASS |
