---
fr_id: FR-OBS-005
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-OBS-005 expanded from 144 lines to ~810. Added 6 §1 clauses (#4 outgoing HTTP injection, #8 tokio::spawn preservation, #9 subprocess preservation, #10 W3C-compliant generation, #11 strict parser + WARN with hash, #12 self-metrics). 7 §2 rationale paragraphs. Full Rust types + tracecontext + logging layer + exemplar + HTTP wrapper + CI workflow in §3. 16 ACs. 5 full Rust test bodies. 17 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Outgoing HTTP propagation not specified
First-pass §1 #4 said "forward traceparent to downstream HTTP calls" without mechanism. Resolved: §3 `InstrumentedClient` wrapper auto-injects via `inject_traceparent` + baggage; AC #8 + #9.

### ISS-002 — tokio::spawn loses trace_id
Async background work (audit-row emit, etc.) loses trace_id without `Instrument`. Resolved: §1 #8 + §11 note about `tracing::Instrument`; AC #12.

### ISS-003 — Subprocess loses trace_id
brain_writer is a subprocess; without env-var propagation, chain breaks. Resolved: §1 #9 OTEL_TRACE_ID env var; AC #13.

### ISS-004 — Malformed traceparent handling unspecified
First-pass §10 said "Generate new trace_id" but didn't specify WARN log or attacker-poisoning concern. Resolved: §1 #11 strict parser + WARN with hash16; §2 rationale paragraph on attacker poisoning; AC #15 + #16.

### ISS-005 — Field name standardisation missing
Loki query `{trace_id="..."}` only works if every log uses field name `trace_id`. Resolved: `logging.rs` ObsContextLayer enforces canonical field names; §10 row.

### ISS-006 — End-to-end CI test not specified
First-pass §4 AC #5 said "CI test" without methodology. Resolved: §1 #7 + `end_to_end_correlation_test.rs` + obs-correlation-gate.yml workflow; queries Loki + Tempo + LangSmith + Prometheus and asserts agreement.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-OBS-005 audit.*
