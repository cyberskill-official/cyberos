---
id: NFR-CUO-002
title: "CUO BRAIN audit-chain emission — every chain execution MUST emit ≥ 2 rows"
module: CUO
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of executed chains produce 1 row per step + 1 summary row; reconciliation drift < 0.001%"
owner: CTO
created: 2026-05-18
related_frs: [FR-CUO-101, FR-CUO-103]
---

## §1 — Statement (BCP-14 normative)

1. Every CUO chain execution with `--brain-emit` flag set **MUST** emit one `kind=cuo.step` row per executed step **plus** one `kind=cuo.chain.end` summary row carrying `{persona, workflow, step_count, started_at, ended_at, outcome}`.
2. Emission **MUST** advance the BRAIN HEAD sequence visibly — the operator can verify execution by inspecting HEAD delta.
3. Audit emit failures **MUST NOT** silently swallow — if `Writer.commit_row()` fails, the chain execution exits non-zero and the partial row state is preserved for forensics.
4. The `cuo.chain.end` summary **MUST** include the chain's complete `skill_chain` (list of skill ids) so a post-hoc reader can reconstruct what ran without parsing the per-step rows.
5. Rows **MUST** be emitted in committed order — `cuo.chain.end` strictly after the last `cuo.step`.

## §2 — Why this constraint

The CUO supervisor is the platform's orchestration brain. Without a complete audit chain, post-hoc debugging "what did the supervisor do at 02:14?" is impossible. The 2-rows-per-chain minimum (per-step batch + summary) is the contract Phase 3 already implements (`project_cyberos_v3_phase3` memory: HEAD `01→03` per chain). The order requirement matters because BRAIN replay walks rows in HEAD order — a summary appearing before its steps would break replay semantics.

## §3 — Measurement

- Counter `cuo_brain_emit_attempt_total{stage=step|summary, result=success|fail}`.
- Daily reconciliation: count of `kind=cuo.chain.end` rows vs count of `cuo_execute_chain_total` counter — drift < 0.001%.
- Gauge `cuo_brain_emit_latency_ms` — emission must not slow chain execution.

## §4 — Verification

- Integration test `modules/cuo/tests/test_brain_emit_smoke.py` (T) — executes a chain with `--brain-emit`; asserts HEAD advanced by step_count + 1.
- Daily production reconciliation job — fail loud on drift.
- Replay test (T) — emit a chain, drop the writer, replay from Layer-1; assert same row count + order.

## §5 — Failure handling

- Single emit failure → chain exits non-zero; manual investigation.
- Reconciliation drift > 0.001% → sev-2; BRAIN writer or commit path broken.
- Out-of-order rows detected → sev-1 (replay semantics broken); halt CUO writes until root-caused.

---

*End of NFR-CUO-002.*
