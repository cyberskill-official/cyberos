---
task_id: TASK-AI-022
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (217 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-014 / TASK-AI-019 depth (~1080 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

TASK-AI-022 was expanded from 217 lines to ~1080 lines matching TASK-AI-014 / TASK-AI-019 depth.

The expansion added 7 §1 normative clauses (#6 typed attribute keys + AST lint as PII defence, #7 explicit traceparent injection on outgoing calls, #8 span events for retries vs. spans for failovers, #9 baggage propagation for downstream attribution, #12 explicit span-status conventions per OTel semantics, #13 OTLP buffer config + timeout, #14 graceful degradation on collector unreachability with metric tracking, #15 compile-time PII lint enforcement, #16 OTel metrics in parallel), 7 substantive §2 rationale paragraphs (OTel-vendor-neutral argument, W3C-vs-B3 IETF-standard frame, PII-prevention-vs-detection structural argument, typed-key-PR-review surface, 100%-sampling-on-errors high-value-debugging frame, baggage-vs-rederive efficiency, retry-events-vs-spans operational distinction, OTLP-gRPC-vs-HTTP efficiency, graceful-degradation separation-of-concerns, <1ms-overhead achievability proof, span-names-as-API stability), full Rust SDK init in §3 (OTLP gRPC + tonic + buffer config + Resource attrs), full typed attribute-key constants module with 20+ approved keys + comments showing why each is PII-safe + explicit FORBIDDEN-at-compile-time list, full W3C propagation module (HeaderExtractor + HeaderInjector + extract/inject helpers), full AST lint module using syn crate, full handler integration showing instrument macro + span status setting, expanded §4 from 8 to 17 acceptance criteria, full Rust test bodies in §5 (root span + child tree + W3C extract + malformed traceparent + outgoing carries traceparent + provider span attempt metadata + retry events + span status per outcome + 100% error sampling + collector unreachable + cache-hit emits no provider_call + PII lint + overhead benchmark), expanded §6 with boot order + tracing instrument examples + provider_call attempt skeleton, expanded §7 with code/concept/operational dep split, 5 example payloads in §8 (success trace + failover trace + cache-hit trace + W3C header chain + PII lint failure + collector-unreachable WARN), 21 failure modes in §10 (vs. 5 in first pass), 10 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — PII-in-attributes only spotted by collector-side scrub; no caller-side prevention

- **severity:** error
- **rule_id:** security / structural defence
- **location:** §1 #6 (claim "MUST NOT include PII"), §10 first-pass row "Detected by FR-OBS PII-scrub on collector"
- **status:** resolved

#### Description

The first-pass §10 had:

> *"PII accidentally added to span | Detected by FR-OBS PII-scrub on collector | Spans flagged; sev-1 | Engineer fixes call site."*

This is REACTIVE detection. By the time the collector flags a span, the PII has already left the gateway, traversed the OTel exporter, sat in the queue, been transported via gRPC, and stored in the collector's intake buffer — at minimum five locations where the PII existed transiently. Each location is a potential leak surface (logs, debug dumps, pcap captures).

The right defence is at the call site: prevent the PII from EVER becoming a span attribute. Reactive scrubbing is a fallback; structural prevention is the primary control.

This is the same prevention-vs-detection principle applied in TASK-AI-011 (PII redaction at request boundary, not at provider response) and TASK-AI-018 (cache cross-tenant isolation enforced at key derivation, not at lookup-time scan).

#### Suggested fix

1. Add §1 #6 normative requirement: typed attribute-key constants in `attributes.rs`; call sites use constants only; string-literal keys forbidden.
2. Add §1 #15 normative requirement: AST lint (`pii_lint.rs`) AST-walks all `*.rs` files in CI; rejects `set_attribute("string-literal", ...)` calls where the literal isn't in the allow-list.
3. Add `attributes.rs` skeleton in §3 with 20+ approved keys + per-key `// PII-safe because: ...` comment + explicit `// FORBIDDEN at compile time: user_email, prompt_text, ...` block.
4. Add `pii_lint.rs` skeleton in §3 using the `syn@2` crate to walk method calls.
5. Add ACs #7 + §5 test `lint_rejects_planted_user_email_attribute`.
6. Add §2 rationale paragraph on prevention-vs-detection.
7. Update §10 row: "PII accidentally added to span (string-literal key) → CI lint blocks PR" — making detection happen BEFORE merge, not after deploy.

### ISS-002 — Span name conventions inconsistent (`router.{provider}` vs `provider_call`)

- **severity:** error
- **rule_id:** consistency / API stability
- **location:** §1 #4 + §1 #5 (different naming patterns)
- **status:** resolved

#### Description

The first-pass had:
- §1 #4: child spans `ai_gateway.precheck`, `ai_gateway.alias_resolve`, ..., `ai_gateway.router.{provider}` — interpolated provider name.
- §1 #5: `ai_gateway.provider_call` — fixed name.

These are TWO different naming conventions for what should be the same span. An investigator searching for "all bedrock calls" would write either `service.name=ai-gateway AND name=ai_gateway.router.bedrock` (per #4) or `service.name=ai-gateway AND name=ai_gateway.provider_call AND attributes.provider=bedrock` (per #5). Both work but require different queries; mixed usage in spans makes both queries miss spans.

The OTel best practice is fixed span name + variable attribute (#5 pattern). Interpolated names produce attribute-cardinality explosion and break query stability.

#### Suggested fix

1. Standardise on §1 #5's pattern: span name `ai_gateway.provider_call`; provider as ATTRIBUTE.
2. Drop the `ai_gateway.router.{provider}` pattern entirely.
3. Document the convention in `docs/span-names.md` (new file in `new_files`).
4. Add AC #17: `span_names_match_doc` test asserts every emitted span name is documented; rejects ad-hoc naming.
5. Update §3 + §6 skeletons to use the canonical name.

### ISS-003 — Trace context propagation to provider HTTP calls claimed but no code shown

- **severity:** error
- **rule_id:** spec-completeness / promise-vs-implementation
- **location:** §1 #7 (claim), §3/§6 (no code)
- **status:** resolved

#### Description

The first-pass §1 #7 said: *"MUST propagate trace context to provider HTTP calls so downstream (LangSmith via TASK-OBS-004) can correlate."*

But no implementation: the §3 API contract was just `init_otel`, `root_span`, `child_span`, and a sample `instrument` macro. No `inject` function. No example of how a provider adapter (Bedrock SDK, OpenAI HTTP client) would carry the trace context.

Without the implementation, TASK-OBS-004's correlation breaks silently — the provider sees a request with no traceparent, can't link its own observability to ours.

#### Suggested fix

1. Add `propagation.rs` module in `new_files` and §3 with `HeaderExtractor`, `HeaderInjector`, `extract_context_from_headers`, `inject_context_into_headers`.
2. Show `inject_context_into_headers(...)` invocation in §6's `call_provider_attempt` skeleton.
3. Add AC #5 asserting outgoing HTTP carries `traceparent`.
4. Add §5 test `outgoing_provider_call_carries_traceparent` with mock provider asserting header presence.
5. Add §10 row: "Outgoing HTTP missing traceparent → integration test asserts → PR blocked."

### ISS-004 — `<1ms overhead` claim has no benchmark methodology

- **severity:** error
- **rule_id:** test-coverage / measurable claim
- **location:** §4 AC #7 (claim), §5 (no benchmark code)
- **status:** resolved

#### Description

First-pass AC #7 said: *"Overhead < 1ms — Benchmark: 1000 calls with OTel disabled vs enabled. Diff in p95 < 1ms."* But no benchmark test code was shown; "diff in p95" wasn't formalised.

A code-gen agent reading the FR can't write the benchmark. Worse, "OTel disabled" wasn't defined — is it `OTEL_SDK_DISABLED=true`? Conditional compilation? Removing the SDK entirely?

#### Suggested fix

1. Add `otel_overhead_benchmark_test.rs` to `new_files` and show the full implementation in §5.
2. Use `OTEL_SDK_DISABLED` env var (a documented OTel SDK flag) to toggle.
3. Show the methodology: 1000 calls each path → sort → compute p95 → assert `(p95_on - p95_off) < 1000µs`.
4. Mark with `#[ignore]` so it runs only with `--ignored` (long-running).
5. Add §5 invocation in the bash one-liners.
6. Add §10 row: "Span overhead exceeds 1ms p95 → benchmark test fails → CI fails."

### ISS-005 — OTel collector-unreachable behaviour underspecified

- **severity:** warning
- **rule_id:** robustness / availability under dependency failure
- **location:** §10 first-pass row "buffers internally; drops oldest after 10K"
- **status:** resolved

#### Description

The first-pass §10 had:

> *"Collector unreachable | OTLP connect error | OTel SDK buffers internally; drops oldest after 10K spans | OBS sev-2 alert; investigate collector."*

But: (a) the 10K buffer size wasn't in §1 (so the size is undocumented and can change); (b) "oldest dropped" wasn't observable (no metric); (c) there's no alarm threshold spec ("sev-2 at what drop rate?"); (d) `init_otel` boot-time behaviour wasn't covered (does the gateway refuse to bind if collector is unreachable at boot?).

A code-gen agent has no template for the buffer config OR the alarm. A future engineer hitting collector outages can't tell what's expected vs. what's broken.

#### Suggested fix

1. Add §1 #13 normative OTLP exporter config: `max_queue_size: 10240`, `max_export_batch_size: 512`, `export_timeout: 30s`, schedule delay 5s.
2. Add §1 #14 normative graceful-degrade: dropped spans increment `ai_gateway_otel_spans_dropped_total{reason}` with reason ∈ `queue_full | export_timeout | collector_unreachable`; sustained drop > 1% over 5min triggers OBS sev-2.
3. Add §1 #1 explicit boot-time behaviour: collector-unreachable at init → gateway refuses to bind (init_otel returns Err; main exits 1).
4. Add AC #14 + §5 test `collector_unreachable_increments_drop_counter`.
5. Add AC #15 asserting the buffer config matches §1 #13.
6. Add §10 rows for boot-time vs. mid-run unreachability (separate paths).
7. Add §2 rationale paragraph on separation of concerns (OTel outage doesn't cascade to gateway outage).

### ISS-006 — Span status codes (Ok/Error) not specified per OTel semantic conventions

- **severity:** warning
- **rule_id:** OTel semantic compliance
- **location:** §1 (no clause), §3 (no status setting)
- **status:** resolved

#### Description

The first-pass had no requirement to set span status (Ok/Error/Unset). OTel observability tools (Tempo, Honeycomb, Datadog) rely on span status for filtering ("show me all error spans"). Without explicit status, every span defaults to `Unset` — making error filtering impossible.

Worse, span status conveys outcome semantics that attributes don't: a `provider_call` span with `status_code: 503` should have status=Error; the attribute alone might not be queried by the alerting pipeline that watches for status=Error.

#### Suggested fix

1. Add §1 #12 normative requirement: standardised status per OTel conventions:
   - `Ok` for successful operations.
   - `Error` for any operation that returned an error (refuses count as errors).
   - `Unset` is not used; every span has explicit status.
2. Update §3 handler skeleton to show `span.record_status(Status::Ok)` and `Status::error(msg)` calls.
3. Add AC #13 asserting status set per outcome (success → Ok; ZdrViolation → Error).
4. Add §5 test `span_status_set_per_outcome` covering both paths.
5. Add §10 row: "Span status not set → integration test fails."

## §3 — Strengths preserved through expansion

- §3 introduces TYPED attribute-key constants (a Rust-idiomatic pattern) — call sites can't accidentally use the wrong key, and the AST lint (`pii_lint.rs`) catches any string-literal escape attempts at CI time.
- §1 #2 W3C TraceContext extraction with malformed-handling produces correct behaviour AND visibility (WARN log with hash16) without leaking the raw bad value.
- §1 #8 retry-as-event vs. failover-as-span is the operational-distinction primitive. Investigators see "3 retries on bedrock then fallback to anthropic" as TWO spans with retry events, NOT five flat spans.
- §1 #9 baggage propagation gives downstream services efficient access to tenant_id / persona / request_id without re-deriving — small cost, real benefit for cross-pillar tracing.
- §1 #14 graceful degrade with structured drop-reason metrics turns "OTel outage" into "OTel outage with visible blast radius and clear alarm threshold."
- §10 inventory grew from 5 rows to 21 — including the boot-time-unreachable path, the orphan-span path, the malformed-traceparent path, the failover-attributes-missing path, the cache-hit-regression path, and the BGE-sidecar-trace-propagation path. Each row has an unambiguous detection mechanism.
- §11 documents the `tracing::instrument` macro as the Rust-idiomatic pattern (rather than manual span management) — keeps call sites clean and maintainable.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the FR itself:

- **ISS-001 RESOLVED**: §1 #6 + §1 #15 add typed-attribute-key + AST-lint requirements; `attributes.rs` shown in §3 with 20+ approved keys + FORBIDDEN block; `pii_lint.rs` AST walker shown; AC #7 + §5 `lint_rejects_planted_user_email_attribute` test; §10 row updated to compile-time prevention; §2 prevention-vs-detection paragraph.

- **ISS-002 RESOLVED**: Span naming standardised on `ai_gateway.provider_call` with provider as ATTRIBUTE; the `ai_gateway.router.{provider}` pattern dropped; `docs/span-names.md` added to `new_files`; AC #17 asserts span names match doc.

- **ISS-003 RESOLVED**: `propagation.rs` module added to `new_files` and shown in §3 with extractor/injector + extract/inject helpers; §6 `call_provider_attempt` skeleton invokes `inject_context_into_headers`; AC #5 + §5 test `outgoing_provider_call_carries_traceparent`; §10 row.

- **ISS-004 RESOLVED**: `otel_overhead_benchmark_test.rs` added to `new_files` and shown in full in §5; uses `OTEL_SDK_DISABLED` env var; methodology explicit (1000 calls each → sort → p95 → assert <1000µs); marked `#[ignore]`; §10 row.

- **ISS-005 RESOLVED**: §1 #13 specifies OTLP exporter config (max_queue_size 10240, max_export_batch_size 512, export_timeout 30s); §1 #14 specifies graceful degrade with `ai_gateway_otel_spans_dropped_total{reason}` metric and sev-2 at >1% sustained 5min; §1 #1 explicit boot-time fail-closed; AC #14 + AC #15 + §5 test `collector_unreachable_increments_drop_counter`; §10 has separate rows for boot vs. mid-run.

- **ISS-006 RESOLVED**: §1 #12 normative status conventions (Ok/Error/Unset); §3 handler skeleton shows `record_status(Status::Ok)` + `Status::error(msg)`; AC #13 + §5 test `span_status_set_per_outcome` covers success + ZDR refuse paths; §10 row.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-022 audit (final). Status: PASS at 10/10.*
