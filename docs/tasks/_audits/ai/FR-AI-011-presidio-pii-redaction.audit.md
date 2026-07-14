---
task_id: TASK-AI-011
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (271 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-001 depth (~810 lines)
score_post_revision: 10/10         # after 4 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes)
---

## §1 — Verdict summary

TASK-AI-011 was expanded from 271 lines (compressed first-pass) to ~810 lines matching TASK-AI-001 depth. The expansion added 5 §1 clauses (#11 idempotency, #12 no PII in error variants, #13 no raw prompt in logs, #14 sev-1 alarm on persistent sidecar-down, #15 sidecar-localhost-only bind), 6 additional §2 paragraphs (in-process-via-PyO3 rejected rationale, fail-closed cost-benefit analysis, typed-placeholder reasoning, restoration-asymmetry rationale, idempotency justification, error-message safety), full schema types in §3 (`RestorationMap` with `Zeroizing<String>` Drop, `PiiType` with `as_metric_label`/`from_presidio` bidirectional helpers, `RedactError` with no-prompt-leak documentation), 6 additional §4 ACs (#11 idempotency, #12 no PII in errors, #13 no raw prompt in logs, #14 sev-1 alarm, #15 localhost bind, #16 sidecar timeout), full Rust integration test bodies in §5 (12 named tokio tests + a separate `redact_no_log_test.rs` + Criterion bench), expanded ~250-line §6 skeleton with metrics module + `sanitize_sidecar_error_message` filter + `build_placeholder_map_and_counts` helper, code/concept/operational deps in §7, 5 example payloads in §8 including memory audit row excerpt, 16 failure modes in §10, 8 implementation notes in §11.

Four residual issues prevent 10/10.

## §2 — Findings

### ISS-001 — AC #14 (sev-1 alarm) and AC #15 (sidecar localhost bind) lack test bodies in §5
- **severity:** error
- **rule_id:** test-coverage
- **location:** §4 ACs #14, #15; §5 (verification)
- **status:** open

#### Description
Two ACs reference verification mechanisms but no test body appears in §5:

- AC #14: *"Configure 6 sidecar-down events in 60s; assert the OBS alarm fires (verifiable via metrics-test scaffold)."* The phrase "verifiable via metrics-test scaffold" is hand-waving — no test code shows how to drive the alarm condition or assert the alarm firing.
- AC #15: *"Deploy-time integration test attempts to connect to the sidecar from a non-loopback IP; assertion: connection refused. Verifies §1 #15."* §11 references a `make smoke-test` target but §5 has no test body.

A code-gen agent reading the FR can implement the redact module but has no way to write the alarm test or the deploy smoke test from these ACs.

#### Suggested fix
Add the missing test bodies to §5:

```rust
// AC #14: assert sev-1 alarm fires after 6 sidecar-down events in 60s.
#[tokio::test]
async fn sev1_alarm_fires_on_persistent_sidecar_down() {
    use mocks::AlarmHarness;
    let alarm = AlarmHarness::watch("ai_redact_calls_total",
        prometheus_alarm_rule("sidecar_unreachable", 5, std::time::Duration::from_secs(60)));
    kill_sidecar().await;
    for _ in 0..6 {
        let _ = redact::redact("hello", &test_policy()).await;
    }
    alarm.assert_fired_within(std::time::Duration::from_secs(2)).await;
}

// AC #15: deploy-time smoke test verifies the sidecar refuses non-loopback connections.
// Lives in tests/deploy/sidecar_loopback_test.rs and runs as `make smoke-test`.
#[tokio::test]
#[ignore = "requires deploy harness; run via `make smoke-test`"]
async fn assert_sidecar_loopback_only() {
    use mocks::deploy_harness::{spawn_sidecar_pod, spawn_intruder_pod};
    let sidecar_ip = spawn_sidecar_pod().await;   // e.g., 10.0.0.5
    let intruder = spawn_intruder_pod().await;
    let result = intruder.curl(&format!("http://{sidecar_ip}:5050/redact"), "{}").await;
    assert!(result.is_err(),
        "sidecar accepted non-loopback connection from {}; MUST bind 127.0.0.1 only", intruder.ip());
}
```

Mark AC #15's test with `#[ignore]` so it doesn't run in unit-test mode but IS picked up by the deploy smoke harness.

### ISS-002 — §6 trusts sidecar's sort order; missing defensive re-sort breaks idempotency on regression
- **severity:** error
- **rule_id:** correctness / single-source-of-truth
- **location:** §1 #11 (idempotency), §3 (PresidioResponse), §6 skeleton (`build_placeholder_map_and_counts`)
- **status:** open

#### Description
§1 #11 mandates idempotency: same `(prompt, policy)` → same `redacted_text` and same placeholder names. The mechanism documented is "deterministic ordering of Presidio analyzer results (sorted by `start` offset)".

The sidecar (§3 Python) explicitly sorts: `results.sort(key=lambda r: r.start)`. The Rust side then iterates `body.items` in the order received and assigns `<TYPE_N>` indices.

If the sidecar's sort is removed in a future regression (e.g., a refactor changes the order), the Rust side silently produces non-idempotent results. The `placeholder counter` in `build_placeholder_map_and_counts` would assign `<EMAIL_ADDRESS_1>` to whichever email came first in the unsorted response — and that order can vary across requests for the same input.

This is the same single-source-of-truth pattern called out in TASK-AI-006 ISS-001 and TASK-AI-008 ISS-003: when correctness depends on a property that lives in a different module, the consumer should re-assert the property defensively, not trust the producer.

#### Suggested fix
Re-sort items in Rust before assigning placeholder indices:

```rust
fn build_placeholder_map_and_counts(
    prompt: &str,
    body: &PresidioResponse,
) -> (String, RestorationMap, HashMap<PiiType, u32>) {
    let mut map = RestorationMap::default();
    let mut counts: HashMap<PiiType, u32> = HashMap::new();
    let mut per_type_counter: HashMap<&str, u32> = HashMap::new();

    // ISS-002 fix: defensive re-sort by start offset to guarantee idempotency
    // regardless of sidecar's response order. The sidecar's Python sort is the
    // primary; this is belt-and-suspenders.
    let mut sorted_items: Vec<&PresidioItem> = body.items.iter().collect();
    sorted_items.sort_by_key(|item| item.start);

    for item in sorted_items {
        let Some(ty) = PiiType::from_presidio(&item.entity) else { continue; };
        let n = per_type_counter.entry(ty.as_metric_label()).and_modify(|c| *c += 1).or_insert(1);
        let placeholder = format!("<{}_{}>", item.entity, n);
        map.insert(placeholder.clone(), item.original.clone());
        *counts.entry(ty).or_insert(0) += 1;
    }

    (body.redacted_text.clone(), map, counts)
}
```

Add a regression test:

```rust
#[tokio::test]
async fn idempotency_holds_when_sidecar_returns_unsorted() {
    let _g = mock_sidecar_with_unsorted_response();   // returns items in reverse-start order
    let r = redact::redact("Email a@x.com first b@y.com second", &test_policy()).await.unwrap();
    assert_eq!(r.map.get("<EMAIL_ADDRESS_1>"), Some("a@x.com"),
        "first-by-position MUST be <EMAIL_ADDRESS_1> regardless of sidecar order");
    assert_eq!(r.map.get("<EMAIL_ADDRESS_2>"), Some("b@y.com"));
}
```

### ISS-003 — `sanitize_sidecar_error_message` doesn't catch FastAPI 422 validation errors that echo request body
- **severity:** error
- **rule_id:** correctness / PII safety
- **location:** §1 #12, §6 (`sanitize_sidecar_error_message` function)
- **status:** open

#### Description
The Python sidecar's `redact()` handler returns `detail: "redaction_internal_error"` (generic). But FastAPI's default 422 Unprocessable Entity response — emitted BEFORE our handler runs, on input validation failures — includes the offending request body in the response. Example: a request with a missing `text` field returns:

```json
{
  "detail": [
    {
      "loc": ["body", "text"],
      "msg": "field required",
      "type": "value_error.missing",
      "input": {"some_other_field": "leak@example.com"}
    }
  ]
}
```

The `input` field MAY echo the request body, which can contain prompt fragments if the request was malformed in an unusual way (e.g., the prompt was placed in the wrong field).

`sanitize_sidecar_error_message` catches long messages (>64 bytes) and ones with `@` or 5+ consecutive digits — but a 422 body may pass: `[{"loc":...,"msg":"field required","type":...}]` is < 64 bytes if the loc path is short and contains no `@`/digits.

#### Suggested fix
Two-part:

1. **Disable FastAPI's default 422 body echo** in the sidecar:

```python
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse

@app.exception_handler(RequestValidationError)
async def custom_validation_handler(request, exc):
    # Generic message; never echo the body (would leak PII per TASK-AI-011 §1 #12).
    return JSONResponse(status_code=422, content={"detail": "validation_error"})
```

2. **Tighten the Rust sanitizer** to be allowlist-based instead of denylist-based:

```rust
fn sanitize_sidecar_error_message(body: &str) -> String {
    // ISS-003 fix: allowlist exact known error codes; reject everything else.
    const KNOWN_ERROR_CODES: &[&str] = &[
        "redaction_internal_error",
        "validation_error",
        "recognizer_init_failed",
        "analyzer_timeout",
        "anonymizer_failed",
    ];
    let trimmed = body.trim();
    if KNOWN_ERROR_CODES.iter().any(|code| trimmed.contains(code)) {
        trimmed.to_string()
    } else {
        "sidecar_returned_unrecognized_message_redacted".to_string()
    }
}
```

Add §10 row: *"FastAPI default 422 body echo on malformed request → could leak prompt; mitigated by custom validation handler in sidecar AND allowlist-based Rust sanitizer."*

### ISS-004 — `PiiType::from_presidio` silently drops unknown entities; potential PII passthrough
- **severity:** warning
- **rule_id:** correctness / observability
- **location:** §3 (`PiiType::from_presidio`), §6 (`build_placeholder_map_and_counts`)
- **status:** open

#### Description
The `from_presidio` function returns `None` for any entity name not in the closed enum. The skeleton uses `let Some(ty) = PiiType::from_presidio(&item.entity) else { continue; };` — silently skipping unknown items.

If TASK-AI-012 adds `VN_CCCD` to the sidecar's recognizer registry but a maintainer forgets to add the variant to `PiiType` (or a typo: `VnCccd` vs `VnCcd`), the sidecar reports the entity but the Rust side drops it. The PII reaches the LLM unredacted — silent compliance failure.

§10 mentions a CI test for this ("CI test asserts every Presidio entity has a PiiType variant") but no test body is shown, and there's no runtime guard.

#### Suggested fix
Two-part:

1. **Emit a warn-level log + counter on unknown entities at runtime** so operators see the gap before a CI test catches it:

```rust
for item in sorted_items {
    let Some(ty) = PiiType::from_presidio(&item.entity) else {
        tracing::warn!(entity = %item.entity,
            "presidio_unknown_entity_dropped; PII not redacted; add variant to PiiType");
        metrics::UNKNOWN_ENTITIES.with_label_values(&[&item.entity]).inc();
        continue;
    };
    // ...
}
```

2. **Add the CI test body** to §5:

```rust
// services/ai-gateway/tests/redact_pii_type_coverage_test.rs
#[test]
fn every_presidio_entity_has_pii_type_variant() {
    // Hits the sidecar's /entities endpoint to enumerate registered recognizers.
    let entities = sidecar_client::list_entities();
    let unmapped: Vec<_> = entities.iter()
        .filter(|e| PiiType::from_presidio(e).is_none())
        .collect();
    assert!(unmapped.is_empty(),
        "Presidio entities without PiiType variants: {unmapped:?}\n\
         Add variants to PiiType enum AND update from_presidio() match arm.");
}
```

Register `metrics::UNKNOWN_ENTITIES` in the metrics module:

```rust
pub static UNKNOWN_ENTITIES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
    "ai_redact_unknown_entity_dropped_total",
    "Presidio reported an entity type the Rust enum doesn't know about",
    &["entity"]
).unwrap());
```

Add §10 row: *"Presidio reports an entity type with no `PiiType` variant → item dropped + warn log + counter increments + CI test fails on next PR."*

### ISS-005 — task-audit skill §3.6 rule 18 (scrub before chain commit) — order-of-operations not explicit in §1
- **severity:** warning
- **rule_id:** authoring-md-§3.6 (rule 18)
- **location:** §1 (no clause naming the memory-commit ordering), §6 (call-site in cost_ledger::precheck)
- **status:** open

#### Description
task-audit skill §3.6 rule 18 says "PII MUST be scrubbed via the `cyberos-memory-pii` ruleset BEFORE chain commit. Never depend on downstream redaction." This FR's `redact::redact` is the scrubber; TASK-AI-003 emits the memory audit row downstream. But §1 doesn't explicitly say "the redacted body MUST be the only form that reaches the memory emit path" — a future refactor could log the original-text-with-restoration-map to the chain, defeating the rule. The order-of-operations needs to be normative: `redact() → cost_ledger::precheck → memory emit (with redacted_text)`, and the memory row's `extra.prompt_snippet` field (if any) MUST be from the redacted text, NEVER the original.

#### Suggested fix
Add §1 #16: "**MUST** ensure the original (un-redacted) text NEVER reaches the memory audit-row emit path per task-audit skill §3.6 rule 18. The call sequence is: `redact::redact(text) → RedactionResult { redacted_text, map } → cost_ledger::precheck(.., redacted_text) → memory row.extra.prompt_snippet = first 256 chars of redacted_text`. The `RestorationMap` is held in a separate `Zeroizing<HashMap<...>>` that is NEVER serialised into any chain or log row. AC #16 verifies via a `tracing-test` capture that no chain row written during a redaction-round-trip test contains any of the original PII values."

### ISS-006 — task-audit skill §3.6 rule 19 (redact() helper in logs) — `tracing::info!(?text)` lint gate not specified
- **severity:** warning
- **rule_id:** authoring-md-§3.6 (rule 19)
- **location:** §1 #13 (no PII in logs), §5 (verification), §11 (notes)
- **status:** open

#### Description
task-audit skill §3.6 rule 19 says "Logs MUST use the `redact()` helper for sensitive fields. Never `tracing::info!(?email)` with raw PII; always `tracing::info!(email = %redact_email(email))`." §1 #13 says "no raw PII in logs" but doesn't mandate the `redact_*` helper pattern. A reviewer can't tell whether `tracing::info!(?prompt)` is forbidden (it is) or merely discouraged. The `redact_no_log_test.rs` test catches runtime leaks but a clippy-style lint would catch the pattern at compile time. task-audit skill rule 19 wants both.

#### Suggested fix
Add §1 #17: "**MUST** use the `cyberos_pii::redact_for_log(text, &policy)` helper in every log statement that takes a text field per task-audit skill §3.6 rule 19. Direct `tracing::info!(?text)` / `tracing::debug!(prompt = %prompt)` / etc. with raw text are spec violations. The codebase enforces this via `#[deny(clippy::disallowed_methods)]` on a custom lint registered in `clippy.toml`: `disallowed-methods = [\"tracing::info_span!\", \"tracing::debug_span!\"]` when called with un-wrapped text args. The `redact_for_log` helper applies a fast-path regex redaction (email/phone/MST) without a sidecar round-trip — it's the right balance for log latency."

## §3 — Strengths preserved through expansion

- §1 grew from 10 to 15 numbered MUST clauses; the no-PII-in-logs (§1 #13) and no-PII-in-errors (§1 #12) clauses anticipate the highest-risk leak vectors and have dedicated tests.
- §3's `RestorationMap` uses `zeroize::Zeroizing<String>` for Drop-time memory wiping — defends against heap-snapshot PII recovery.
- §3's `PiiType::from_presidio` provides a bidirectional mapping; the closed-set enum prevents the "unknown entity silently passes through" failure (when combined with ISS-004's runtime-warn fix).
- §5 includes a dedicated `redact_no_log_test.rs` test file that uses `tracing-test` to capture log records and assert no raw PII appears — this catches debug-logging regressions at PR time.
- §6 skeleton's `sanitize_sidecar_error_message` is conservative: any error message that LOOKS LIKE it might contain PII gets replaced with a generic placeholder. The trade-off (loss of debuggability) is documented in §11 and the operator workflow is to attach a sidecar-side debugger rather than lower the filter.
- §10 inventory covers 16 distinct paths including the deploy-time loopback-bind violation and the spaCy-model-corruption row.
- §11 explicitly calls out the PyO3 rejection rationale and the GIL constraint — saves future contributors from re-litigating the architecture decision.

## §4 — Resolution

All 6 mechanical revisions applied:
- ISS-001 RESOLVED (2026-05-16): §5 now includes `sev1_alarm_fires_on_persistent_sidecar_down` and `assert_sidecar_loopback_only` test bodies; ACs #14/15 covered.
- ISS-002 RESOLVED (2026-05-16): §6 `build_placeholder_map_and_counts` defensively re-sorts response items by `start` offset before placeholder indexing; `idempotency_holds_when_sidecar_returns_unsorted` test added.
- ISS-003 RESOLVED (2026-05-16): sidecar Python defines `RequestValidationError` handler returning generic `validation_error`; `sanitize_sidecar_error_message` switched to allowlist-based with explicit `KNOWN_ERROR_CODES`; §10 row added.
- ISS-004 RESOLVED (2026-05-16): §6 emits `tracing::warn!` + `ai_redact_unknown_entity_dropped_total{entity}` on unknown entities; CI test `every_presidio_entity_has_pii_type_variant` added; §10 row added.
- ISS-005 RESOLVED (2026-05-16, task-audit skill compliance pass): §1 #16 added asserting memory-emit ordering (redacted_text only, never original); RestorationMap held in `Zeroizing<HashMap>`, never serialised; AC #16 added (tracing-test PII-leak capture).
- ISS-006 RESOLVED (2026-05-16, task-audit skill compliance pass): §1 #17 added mandating `cyberos_pii::redact_for_log` helper for every log statement; clippy lint registered in `clippy.toml`; tracing::info!(?text) spec-violation.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-011 audit (final). Status: PASS at 10/10.*
