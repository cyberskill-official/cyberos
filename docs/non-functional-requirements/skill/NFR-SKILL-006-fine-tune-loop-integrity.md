---
id: NFR-SKILL-006
title: "SKILL fine-tune loop integrity — author+audit pair MUST stay in lockstep"
module: SKILL
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "0 author skills exist without a sibling audit skill; CI gates publish"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-SKILL-107]
---

## §1 — Statement (BCP-14 normative)

1. Every author skill at `skill/public/<name>/` **MUST** have a sibling audit skill at `skill/public/<name>-audit/` carrying matching version + same set of declared outputs.
2. Publishing an author skill without its audit sibling **MUST** be rejected at the publish gate; the rejection error references the missing audit-skill path.
3. The audit skill's `RUBRIC.md` **MUST** declare a score scale (`0–10`), a passing threshold (`≥ 8/10`), and at least 5 measurable criteria covering structure + content + actionability.
4. Author skills **MUST** call their audit sibling at least once per produced output and persist the audit row alongside the output.
5. Catalog-level invariant: `count(skill/public/*/) == 2 × count(skill/public/*-audit/)` — every non-audit folder has its audit pair.

## §2 — Why this constraint

The SKILL catalog runs on a strict author+audit pair convention (87+ pairs as of Session N) — it's the platform's quality-feedback loop. If an author skill could ship without its auditor, the whole feedback loop silently collapses: synthesised outputs land without a self-check, and the rubric drift compounds. The CI gate makes this structural: the two skills are released as a unit or not at all. The RUBRIC discipline ensures audit skills aren't trivially passing every output — there's a real, declared bar.

## §3 — Measurement

- CI metric `skill_pair_drift_count` — counts authors without audits; must be 0 to merge.
- Per-skill counter `skill_audit_invocation_total{author_skill}` — surfaces authors that don't actually call their auditor.
- Histogram `skill_audit_score_distribution{author_skill}` — surfaces auditors that always pass everything (rubber-stamp).

## §4 — Verification

- CI gate `skill-pair-integrity` (T) — walks `skill/public/`; fails on any unpaired author.
- Pre-publish check (T) — bundle's manifest declares its audit-pair; the broker rejects publish if pair missing from registry.
- Quarterly RUBRIC drift audit — sample 20 random auditors; assert pass rate is in `40-95%` window (rubber-stamp detection).

## §5 — Failure handling

- Unpaired author detected → CI block, contributor required to author the audit sibling.
- Auditor pass rate > 95% sustained → sev-3 review of the RUBRIC by skill-catalog owners; may indicate rubber-stamping.
- Auditor pass rate < 40% sustained → sev-3 review; may indicate the author skill is structurally broken.

---

*End of NFR-SKILL-006.*
