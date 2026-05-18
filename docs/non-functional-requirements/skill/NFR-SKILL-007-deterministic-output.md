---
id: NFR-SKILL-007
title: "SKILL deterministic-output mode — fixed seed produces byte-identical output"
module: SKILL
category: reliability
priority: SHOULD
verification: T
phase: P1
slo: "deterministic-flag skills produce byte-identical outputs over 100 reruns with fixed seed"
owner: CTO
created: 2026-05-18
related_frs: [FR-SKILL-103, FR-SKILL-105]
---

## §1 — Statement (BCP-14 normative)

1. Skills declaring `deterministic: true` in their manifest frontmatter **MUST** produce **byte-identical outputs** when invoked twice with the same `{inputs, seed, capabilities}` tuple.
2. The runtime **MUST** propagate a deterministic seed (32-byte RNG seed) into every non-deterministic op (LLM sampling, UUID generation, timestamp where the skill needs one — that's now derived from input).
3. Non-deterministic skills (no flag, or `deterministic: false`) are not bound by this — they may produce different outputs on rerun.
4. Determinism applies to **outputs only**, not to wall-clock-bound side effects (audit-row `committed_at`, log timestamps).
5. The runtime **MUST** record the seed used in the audit row so a replay can reproduce the exact output.

## §2 — Why this constraint

Determinism unlocks: (a) regression testing — diff old vs new output, signal real changes vs noise; (b) audit-replay — re-run a 90-day-old skill invocation and verify the persisted output; (c) cheap caching — fingerprint = hash(inputs, seed), no expensive recomputation. The cost of determinism is real (seeded sampling, deterministic UUID derivation), so it's opt-in via `deterministic: true`. Skills that are inherently non-deterministic (LLM brainstorm) opt out cleanly.

## §3 — Measurement

- CI gate per `deterministic: true` skill: rerun 5 times with same seed; assert all outputs hash-match.
- Counter `skill_deterministic_violation_total{skill}` — fires when a flagged skill produces drift in production.
- Per-skill metric `skill_seed_propagation_coverage_pct` — measures how many internal ops respect the seed.

## §4 — Verification

- Per-skill test in CI (T) — fixture inputs + fixed seed; assert byte-identical output across 5 reruns.
- Replay test (T, quarterly) — pick 10 random `deterministic: true` invocations from 30+ days ago; replay; assert output matches archived.
- Property test (T) — random inputs + random seed → consistent output.

## §5 — Failure handling

- Determinism violation in CI → block release of the skill.
- Production violation → sev-3; the skill's `deterministic` flag is auto-flipped to `false` pending RCA.
- Replay test fail → sev-2; audit-replay is a regulatory promise; investigate immediately.

---

*End of NFR-SKILL-007.*
