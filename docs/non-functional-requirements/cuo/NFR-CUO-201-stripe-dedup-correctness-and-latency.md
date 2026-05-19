---
id: NFR-CUO-201
title: "stripe-dedup MUST be deterministic + collision-bounded + check < 10ms p95"
module: cuo
category: reliability
priority: MUST
verification: T
phase: P0
slo: "stripe collision rate ≤ 2⁻³² (8-hex SHA-256 truncation birthday bound); open/ glob check p95 < 10ms over ≤ 1000 open proposals"
owner: CTO
created: 2026-05-19
related_frs: [FR-CUO-201]
---

## §1 — Statement (BCP-14 normative)

1. `cuo.core.stripe.compute_stripe(skill_name, signal_id, evidence_rows)` **MUST** be deterministic — same inputs across processes, sessions, and OS versions produce the same stripe id (verified by repeating the call with identical args and asserting string equality).
2. The pattern_hash component **MUST** be exactly 8 lowercase hex characters (32 bits of entropy), giving birthday-bound collision probability ≤ 2⁻³² between two genuinely-different evidence projections.
3. The dedup lookup (`open_dir.glob(f"{stripe_id}-*.md")`) **MUST** complete in **p95 < 10ms** when `docs/proposals/open/` contains up to 1000 unresolved proposals. Linear filesystem scan is acceptable at this scale; index files are NOT required at < 10⁴ proposals.
4. The stripe id **MUST** survive a round-trip through `StripeId.parse()` byte-for-byte — `StripeId.parse(str(s)) == s`.
5. Workflow stripes (containing `/`) **MUST** be disjoint from skill stripes by the `/` separator presence — the regex `[a-z0-9_-]+(?:/[a-z0-9_-]+)?:` enforces this at parse time; the filesystem-safe form replaces `/` with `--` ONLY at the filename layer, preserving stripe-id identity in memory.

## §2 — Why this constraint

Stripe-dedup is the load-bearing rule of the FR-CUO-201 architecture: first-occurrence emits, second occurrence halts. If `compute_stripe` were non-deterministic (e.g. hashed a timestamp or used Python `hash()`), the dedup would silently fail and Stephen would get N proposals for the same root cause. The 32-bit collision space is small but adequate: at 1000 simultaneously-open proposals, the birthday-bound collision probability is C(1000, 2) × 2⁻³² ≈ 1.16 × 10⁻⁴. If real-world usage approaches 10⁴ open proposals, this NFR's threshold needs revisiting; for the foreseeable use case (Stephen + small team), 8 hex chars is generous.

The < 10ms p95 budget keeps the emitter latency negligible relative to the LLM proposal-authoring step (which dominates at 500ms-5s per proposal).

## §3 — Measurement

Determinism: `test_stripe_determinism` in `modules/cuo/tests/test_refinement_proposal.py` already verifies — same evidence → same stripe across two calls in the same process. Cross-process determinism is verified by importing `compute_stripe` in a fresh subprocess and asserting the same output.

Width: `test_stripe_hash_width` asserts `len(pattern_hash) == 8` and all chars in `0-9a-f`.

Lookup latency: NEW benchmark `modules/cuo/tests/bench_stripe_lookup.py` seeds `open/` with 1000 distinct proposal files + runs `emit_or_halt` against a fresh stripe 1000 times. Reports median + p95 + p99.

Collision probability: out of scope for runtime verification — confirmed by the SHA-256 birthday-bound math + the deterministic projection (sorted set of strings → canonical-JSON → SHA-256 truncated to 32 bits).

## §4 — Verification

Tests already passing (FR-CUO-201): `test_stripe_determinism`, `test_stripe_hash_width`, `test_repeat_stripe_halts_no_new_file`, `test_applied_proposal_reopens_stripe`, `test_workflow_and_skill_stripes_disjoint` (in FR-CUO-203's test file).

Inspection: `cuo.core.stripe._project()` uses `sorted({...})` projections — order-independent, hashable, JSON-serialisable. `json.dumps(..., sort_keys=True)` makes the canon string deterministic. `hashlib.sha256(...).hexdigest()[:8]` truncates to 8 chars deterministically.

## §5 — Failure handling

**Detection:** if the same root cause produces TWO open proposals at the same time, the dedup invariant has broken. The `cyberos-cuo proposal list` output should never show two `open` proposals with the same stripe id. A monitoring script `tools/check-proposal-dedup.sh` runs nightly and alerts on duplicate stripe ids.

**Alert:** sev-2 — duplicate stripes in `open/` directly violates Stephen's "don't waste time on rework" rule.

**On-call action:** (a) diff the two proposals to find the divergence; (b) consolidate manually (move one to `applied/`, leave the other in `open/`); (c) file a follow-up FR if the divergence reveals a `_project()` bug.

**Escalation:** if collision rate is observed > 10⁻⁴ in practice (vs the theoretical 10⁻⁴ at 1000 proposals), widen `pattern_hash` to 12 chars (96 bits of entropy → collision bound 2⁻⁹⁶). This is a minor bump under FR-CUO-202's classifier (cosmetic field width change).

## §6 — Notes

Filesystem-safety: workflow stripes contain `/`, which would create subdirectories if used directly as filenames. The `emit_or_halt` function escapes `/` to `--` ONLY in the filename layer — the in-memory `stripe_id` keeps the slash, preserving cross-component identity. This split is essential for the dedup glob to find existing entries.
