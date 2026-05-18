---
id: NFR-EMAIL-006
title: "EMAIL dual-LLM separation — quarantined LLM MUST NOT see untrusted content + tool surface"
module: EMAIL
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of CaMeL pipeline runs maintain LLM separation; 0 untrusted-content-with-tools events"
owner: CISO
created: 2026-05-18
related_frs: [FR-EMAIL-005]
---

## §1 — Statement (BCP-14 normative)

1. The CaMeL dual-LLM architecture **MUST** maintain strict separation: the "privileged" LLM has tool access but never sees raw email body; the "quarantined" LLM sees raw body but has zero tool access.
2. Bridging happens via a strongly-typed contract (structured fields, not free text); a violation of the contract surface **MUST** kill the pipeline.
3. Tool-call attempts from the quarantined LLM **MUST** be auto-rejected by the orchestration layer; rejected attempts are audited.
4. The privileged LLM's input **MUST NOT** contain any string derived directly from the raw email body — only fielded data from the quarantined LLM's structured output.
5. A static analysis pass **MUST** verify the data-flow graph in `modules/email/camel/` at CI time; violations block merge.

## §2 — Why this constraint

Dual-LLM is the entire mechanism that makes CaMeL safe. If the separation is breached — even subtly, like a "summary" passed verbatim into the privileged LLM that contains hidden instructions — the entire prompt-injection defence is bypassed. The contract-only bridge + tool-block + static analysis triple-check enforces the invariant at design, runtime, and code-review time.

## §3 — Measurement

- Counter `email_camel_quarantined_tool_attempt_total` — must be 0.
- Counter `email_camel_separation_violation_total` — must be 0.
- Static analysis gate exit code in CI.

## §4 — Verification

- Unit test (T) — fixture quarantined LLM attempts tool; assert blocked.
- Static-flow analyser (T) — verifies data graph in CI.
- Red-team test (T, quarterly) — adversarial probes against separation.

## §5 — Failure handling

- Tool attempt from quarantined → block + audit + sev-3 (expected adversarial pattern).
- Separation violation → sev-1; halt email pipeline; CISO postmortem.
- Static analysis failure → CI block.

---

*End of NFR-EMAIL-006.*
