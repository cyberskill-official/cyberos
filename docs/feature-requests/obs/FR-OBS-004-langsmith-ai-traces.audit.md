---
fr_id: FR-OBS-004
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
---

## §1 — Verdict summary

FR-OBS-004 expanded from 131 lines to ~840. Added 7 §1 clauses (#1 W3C trace_id correlation, #5 RedactedPrompt newtype enforcement, #8 retry logic, #9 token auth + rotation, #11 Idempotency-Key, #12 100KB truncation, expanded #4 with per-region routing). 8 §2 rationale paragraphs. Full Rust types + client + payload builder + langsmith docker-compose in §3. 17 ACs. 7 full Rust test bodies. 18 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — RedactedPrompt enforcement at API boundary not specified
First-pass §6 took `redacted_prompt: &str` — caller could pass raw. Resolved: §1 #5 + §3 `RedactedPrompt(String)` newtype; raw `String` won't compile; test asserts compile-safety.

### ISS-002 — W3C trace_id correlation mechanism unspecified
First-pass §1 #1 said "matches the OTel trace_id" without format spec. Different formats (raw bytes vs hex) break correlation. Resolved: §1 #1 explicit hex format `format!("{trace_id:032x}")`; AC #12 + §5 test.

### ISS-003 — No retry logic; transient LangSmith errors lose data
First-pass §10 said "drop export silently" on first failure. Transient blips lose data unnecessarily. Resolved: §1 #8 3 retries with exponential backoff (100/250/500ms); auth-failed drops immediately; AC #4 + #5 + §5 tests.

### ISS-004 — Per-region routing missing (residency violation for non-default regions)
First-pass §1 #4 used a single URL `https://langsmith.cyberos.world`. Sg1 + Eu1 + Vn1 tenants need per-region. Resolved: §1 #4 per-region URLs; deploy/obs/langsmith-config.yaml; AC #10.

### ISS-005 — Per-tenant opt-in audit row not specified
First-pass mentioned `langsmith_export: bool` policy field but no audit row when toggled. Privacy decision should be auditable. Resolved: §1 #3 enabled-via-FR-AI-021 CLI emits `obs.langsmith_export_enabled` memory row; AC #16.

### ISS-006 — Payload size limit unspecified
First-pass had no truncation. Huge payloads (RAG context) crash LangSmith UI. Resolved: §1 #12 100KB cap with marker; AC #11 + §5 test.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-OBS-004 audit.*
