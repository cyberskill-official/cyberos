---
fr_id: FR-CHAT-008
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 16
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per feature-request-audit skill §0; ISS-007..016 added)
---

## §1 — Verdict summary

FR-CHAT-008 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 25 §1 clauses (plugin intercept, webhook, CUO routing, threaded reply, memory audit, PII redact, dedup, sync_class, 2s budget, metrics, skip-imported, skip-Lumi-own, rate-limit-30/min, tenant enable/disable, placeholder/edit pattern, trace_id propagation, channel context to CUO, builtin help command, builtin cancel command, multiple-mentions-single-invocation, user_display+role in audit, response-redaction, chat.lumi_error audit, two-key-launch enable+feature-flag, budget tokens+dollars in audit). 15 §2 rationale paragraphs. §3 contains: full Rust handle_webhook with skip-checks + rate-limit + placeholder + edit + response-redaction + budget capture, parser.rs with mention regex + count_mentions + parse_builtin_cmd + help template, tenant_settings.rs + feature_flag.rs + rate_limiter.rs (Redis sliding window) + reply.rs (post + edit), DDL for cyberos_chat_tenant_settings + cyberos_tenant_features. 30 ACs. §5 contains 19 named test bodies covering regex precision + outcome routing + imported-skip + Lumi-self-skip + rate-limit + tenant-disabled + placeholder-edit + trace-propagation + builtin-help-no-CUO + cancel + multi-mention + response-redaction + lumi_error-audit + budget-capture. §6 deepens with 12 wiring subsections (plugin/service split rationale, transport, placeholder edit semantics, budget capture, Redis storage choice, tenant settings caching, feature-flag wiring, Lumi system user provisioning, trace chain, error severity routing, test fixtures, failure routing matrix). §8 lists 7 example payloads. §10 lists 45 failure rows. §11 lists 25 implementation notes covering hook timing choice, dedup edge cases, context-window calibration, regex precision rationale, placeholder edit-window dependency, rate-limit calibration, fail-open vs fail-closed semantics, footer UX, sentence-boundary heuristic for VN text, builtin command anchoring.

## §2 — Findings (all resolved)

### ISS-001 — Mention regex precision
Substring match risk. Resolved: word-boundary regex + AC #3.

### ISS-002 — Dedup
Edits trigger re-route. Resolved: §1 #7 + AC #11.

### ISS-003 — Reply threading
Inline lost in busy. Resolved: §1 #4 root_id; AC #6.

### ISS-004 — Latency budget
> 2s = broken-feel. Resolved: §1 #9 + AC #12.

### ISS-005 — PII redaction
Sensitive @lumi prompts. Resolved: §1 #6 + AC #10.

### ISS-006 — CUO routing path
Direct LLM call = no policy. Resolved: §1 #3 via FR-CUO-101 + persona=lumi.

### ISS-007 — Imported posts would re-trigger Lumi (strict-redo pass)
Original spec didn't address @lumi mentions in historical messages from FR-CHAT-006/007 imports. Processing them would trigger thousands of LLM calls and produce anachronistic replies. Resolved: §1 #11 + `is_imported` propagation from plugin to service + early-return check; AC #16 + test body verify.

### ISS-008 — Lumi could trigger itself (strict-redo pass)
Original spec didn't filter Lumi-authored replies. A reply containing `@lumi` in quoted text would re-trigger and produce an infinite reply loop. Resolved: §1 #12 + user_id check against `lumi_system_user_id()`; AC #17 + test body verify; §11 documents the recursion-depth safety net.

### ISS-009 — No per-user rate limit (strict-redo pass)
Without rate limiting, one user (or bot script) could monopolise CUO budget. Resolved: §1 #13 + Redis sliding-window limiter at 30/min/user; AC #18 + test body verify; §6.5 explains Redis-vs-Postgres choice; §11 explains fail-OPEN rationale.

### ISS-010 — No tenant-level Lumi enable/disable (strict-redo pass)
Some tenants opt out of LLM features for compliance or cost. Original spec processed every tenant. Resolved: §1 #14 + #24 (two-key launch: tenant_settings.lumi_enabled AND tenant_features.lumi) + `chat.lumi_skipped` audit row; AC #19 + AC #20 + test bodies verify.

### ISS-011 — No placeholder for long responses (strict-redo pass)
The original 2s budget would fail-silent for legitimate long LLM calls. Users would see no reply for 30s+ and assume broken. Resolved: §1 #15 + 2s placeholder + MM PATCH edit when CUO resolves; AC #21 + test body verify.

### ISS-012 — Trace_id not propagated across hops (strict-redo pass)
Original spec emitted trace_id in audit but didn't specify the propagation chain. Distributed debugging would be impossible. Resolved: §1 #16 + W3C traceparent on every hop (plugin → service → CUO → MM reply); AC #22 + test body verifies same trace_id appears in all 4 locations; §6.9 documents the full chain.

### ISS-013 — Builtin help/cancel commands missing (strict-redo pass)
`@lumi help` and `@lumi cancel` are predictable user actions that don't benefit from LLM calls (help) or would race the CUO call (cancel). Resolved: §1 #18 (help template) + §1 #19 (cancel with cuo::cancel_latest_for); AC #24 + AC #25 + test bodies verify; §11 explains anchored regex prevents mid-sentence matches.

### ISS-014 — Multiple mentions in same post unclear (strict-redo pass)
A user typing `@lumi do A and @lumi do B` could either trigger two CUO calls (wasteful) or one (sensible). Original spec didn't say. Resolved: §1 #20 + count_mentions helper + single CUO invocation; AC #26 + test body verify; memory audit records mention_count for visibility.

### ISS-015 — Response not redacted before audit (strict-redo pass)
LLM responses can echo PII from prompt or hallucinate new PII from training data. Original spec only redacted request body. Resolved: §1 #22 + separate redaction pass on response before audit emit; AC #28 + test body verify; §11 explains training-data leak rationale.

### ISS-016 — Error visibility insufficient (strict-redo pass)
Original spec metric `chat_lumi_mentions_total{outcome=error}` increments but no memory audit row, so operator post-mortem requires log digging. Resolved: §1 #23 + `chat.lumi_error` memory row with error_class + redacted error_message + SEV-2 escalation on sustained rate; AC #29 + test body verify; §6.10 documents severity routing.

## §3 — Resolution

All 16 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (LLM persona flow + tenant gating + rate limiting + placeholder UX + multi-hop tracing + builtin commands + budget capture + error observability), not by line targets.

---

*End of FR-CHAT-008 audit.*
