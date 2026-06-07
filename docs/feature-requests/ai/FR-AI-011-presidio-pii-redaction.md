---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-011
title: "Presidio EN-base PII redaction in-flight (every prompt)"
module: AI
priority: MUST
status: ready_to_implement
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-21
memory_chain_hash: null
related_frs: [FR-AI-002, FR-AI-005, FR-AI-008, FR-AI-012, FR-AI-013, FR-AI-021]
depends_on: [FR-AI-008]
blocks: [FR-AI-012, FR-AI-013]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#pii-redaction
  - website/docs/legal/data-processing.html#minimisation
source_decisions:
  - PDPL Art. 7 (Vietnam personal data protection — minimisation principle)
  - GDPR Art. 5(1)(c) (data minimisation)
  - Singapore PDPA s. 18 (purpose limitation)
  - archive/2026-05-14/RESEARCH_REVIEW.md §4.1 (Presidio vs cloud-DLP trade-off study)

# ───── Build envelope ─────
language: rust 1.81 + python 3.11 (presidio sidecar)
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/redact.rs
  - services/ai-gateway/src/redact/presidio_client.rs
  - services/ai-gateway/src/redact/types.rs
  - services/ai-gateway/src/redact/restoration.rs
  - services/ai-gateway/pii/presidio_sidecar.py
  - services/ai-gateway/pii/Dockerfile.presidio
  - services/ai-gateway/tests/redact_test.rs
  - services/ai-gateway/tests/redact_no_log_test.rs
modified_files:
  - services/ai-gateway/src/handlers/chat.rs    # call redact between precheck and router
  - services/ai-gateway/src/lib.rs              # export redact module
  - services/ai-gateway/Cargo.toml              # reqwest, serde, tracing
  - deploy/compose/ai-gateway.yml               # add presidio sidecar service
  - deploy/k8s/ai-gateway/presidio-sidecar.yaml # k8s sidecar manifest
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,pii}/**
  - bash: cargo test -p cyberos-ai-gateway redact
  - bash: docker compose up -d presidio
disallowed_tools:
  - run a non-localhost presidio (sidecar MUST be loopback only — no network egress)
  - skip redaction on any chat path (every chat call MUST go through redact())
  - log raw prompt text after redaction (logs MUST contain only the redacted form)
  - persist RestorationMap to disk, Postgres, or memory (memory-only by §1 #4)
  - format!("{:?}", PiiType) for OBS labels (use as_metric_label() per FR-AI-007 ISS-003 pattern)
  - include prompt fragments in RedactError variants (error messages MUST NOT leak input)

# ───── Estimated work ─────
effort_hours: 6
sub_tasks:
  - "1.0h: Presidio sidecar FastAPI (Python; localhost only; cycles AnalyzerEngine.analyze + AnonymizerEngine.anonymize)"
  - "0.5h: Sidecar Dockerfile + compose/k8s wiring with localhost-only bind (127.0.0.1:5050)"
  - "1.0h: Rust client (reqwest to localhost:5050) with timeout + structured error mapping (no prompt leak in error msgs)"
  - "1.0h: PiiType enum + PII registry (which types redact at gateway-level vs preserve)"
  - "1.0h: Restoration module (UUID-tagged placeholders, idempotent placeholder generation, Drop-clears-map invariant)"
  - "0.5h: Integration with chat handler (call redact between precheck and router::call_provider)"
  - "1.0h: Tests (12 cases — credit card, SSN, email, phone, name, date, address, IP, no-PII, mixed, idempotency, no-leak-in-logs)"
risk_if_skipped: "Sensitive PII (CCCD, credit card, SSN, etc.) leaks to every LLM provider on every call. GDPR Art. 5(1)(c) + PDPL Art. 7 + Singapore PDPA s.18 data-minimisation violated by every call. Audit trail (memory ai.invocation row) contains raw PII forever — not GDPR-erasable in practice. Catastrophic compliance failure on first auditor review; vacate all enterprise contracts that require DPA compliance; brand damage. The other PII work (FR-AI-012 VN layer, FR-AI-013 recall-floor CI) layers ON TOP of this baseline."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** redact English-base PII from every prompt before dispatching to any LLM provider. The redaction:

1. **MUST** run a Presidio sidecar (localhost-only, no network egress) — Python 3.11 + presidio-analyzer 2.2.x + presidio-anonymizer 2.2.x in a separate container/process. The sidecar's HTTP server MUST bind to `127.0.0.1:5050` exclusively; binding to `0.0.0.0` is a deploy-time validation failure.
2. **MUST** redact the following PII types by default: `CREDIT_CARD`, `US_SSN`, `EMAIL_ADDRESS`, `PHONE_NUMBER`, `PERSON`, `LOCATION`, `IP_ADDRESS`, `IBAN_CODE`, `US_BANK_NUMBER`, `MEDICAL_LICENSE`. The default set is a closed enum (`PiiType`) and CANNOT be reduced by tenant policy in slice 3 — only EXTENDED via `pii_redaction_extra` (FR-AI-012's VN types). Each replacement preserves a typed placeholder (`<EMAIL_1>`, `<PERSON_2>`, etc.) where the integer suffix is monotonic per (PiiType, request).
3. **MUST** produce a restoration map (`{ "<EMAIL_1>": "user@example.com" }`) returned alongside the redacted prompt. The handler retains the map for the duration of the request; the LLM never sees the raw values.
4. **MUST NOT** persist the restoration map to disk, Postgres, memory, OBS metrics, error messages, or any structured log field. The map exists for the request lifetime only and is dropped via `RestorationMap`'s `Drop` impl (which `zeroize`s the underlying memory).
5. **MUST** restore typed placeholders in the LLM response BEFORE returning to the caller IF AND ONLY IF the placeholders appear in tool-call argument fields (e.g., a function call with `to: "<EMAIL_1>"` becomes `to: "user@example.com"`). For text-only response fields, placeholders MUST stay as-is in the response — the LLM was instructed (via system prompt) to not output unredacted PII; if a placeholder appears in plain text, returning it to the caller is the safe default.
6. **MUST** complete redaction within **30ms p95** for prompts ≤ 8KB. Larger prompts scale linearly with content size; the latency budget is `30ms + 4ms/KB` for prompts ≤ 64KB. Prompts >64KB are rejected at the request-validation layer (FR-API-002) and never reach `redact()`.
7. **MUST** fail closed: if the Presidio sidecar is unreachable, returns a non-2xx, or times out (>2s), the gateway MUST refuse the call with `503 SERVICE_UNAVAILABLE` (`pii_redaction_unavailable`). Calls do NOT proceed unredacted under any circumstance — there is no `bypass_redaction` policy flag.
8. **MUST** carry the PII-type counts to memory audit row (FR-AI-002's `ai.invocation`): `extra.pii_redactions = { "credit_card": 1, "email_address": 3, ... }`. Counts only, NEVER values. The keys MUST come from `PiiType::as_metric_label()` for stability across refactors.
9. **MUST** emit OTel metrics with stable label values: `ai_redact_calls_total{outcome}` (counter; outcome ∈ `ok`/`sidecar_unreachable`/`sidecar_timeout`/`sidecar_error`/`invalid_prompt`), `ai_redact_latency_ms{outcome}` (histogram), `ai_redact_pii_types_total{type}` (counter; type from `PiiType::as_metric_label()`), `ai_redact_prompt_size_bytes` (histogram). All labels MUST be sourced from `as_metric_label()` methods — never `format!("{:?}", ...)`.
10. **SHOULD** support tenant-level PII-type extensions via `policy.ai_policy.pii_redaction_extra: ["VN_CCCD", "VN_MST"]` — but English-base 10 types are always on (no opt-out at slice 3). FR-AI-012 wires the VN types into the sidecar's recognizer registry.
11. **MUST** be idempotent — calling `redact(prompt, policy)` twice with the same `(prompt, policy_snapshot)` MUST produce restoration maps with the SAME placeholder names mapped to the SAME values. Within a single process, the placeholder counter resets per request; across processes, identical `(prompt, policy)` produces identical redacted output. This is enforced by the deterministic ordering of Presidio analyzer results (sorted by `start` offset).
12. **MUST NOT** include any prompt fragment in `RedactError` variant payloads. The `SidecarError { status, message }` variant's `message` is the SIDECAR's textual error response — which MUST NOT echo the request body. The sidecar MUST return generic errors (`"validation failed"`, `"recognizer error"`); leaking the prompt would defeat redaction.
13. **MUST NOT** log the raw prompt text at any tracing level (including DEBUG and TRACE) inside the redact module or downstream. The `tracing::info!`/`debug!` calls MUST log only the redacted form, the latency, and the counts. A dedicated test (`redact_no_log_test.rs`) inspects emitted log records and asserts no raw PII appears.
14. **MUST** emit a sev-1 alarm when `ai_redact_calls_total{outcome="sidecar_unreachable"}` exceeds 5 events in 60 seconds. A persistent sidecar-down state stops every chat call; operators MUST be paged immediately.
15. **MUST** validate the sidecar binds to `127.0.0.1` only at deploy time. The compose/k8s manifests use `network_mode: "host"` with explicit `extra_hosts` (or k8s `hostNetwork: false` with localhost-only service); a startup integration test confirms the sidecar refuses connections from non-loopback addresses.
16. **MUST** ensure the original (un-redacted) text NEVER reaches the memory audit-row emit path per feature-request-audit skill §3.6 rule 18. The call sequence is `redact::redact(text) → RedactionResult { redacted_text, map } → cost_ledger::precheck(.., redacted_text) → memory row.extra.prompt_snippet = first 256 chars of redacted_text`. The `RestorationMap` is held in a separate `Zeroizing<HashMap<...>>` that is NEVER serialised into any chain or log row. AC #17 verifies via a `tracing-test` capture that no chain row written during a redaction-round-trip test contains any of the original PII values.
17. **MUST** use the `cyberos_pii::redact_for_log(text, &policy)` helper in every log statement that takes a text field per feature-request-audit skill §3.6 rule 19. Direct `tracing::info!(?text)` / `tracing::debug!(prompt = %prompt)` / etc. with raw text are spec violations. The codebase enforces this via `#[deny(clippy::disallowed_methods)]` registered in `clippy.toml`. The `redact_for_log` helper applies a fast-path regex redaction (email/phone/MST) without a sidecar round-trip — it's the right balance for log latency.

This is the EN baseline. FR-AI-012 adds the Vietnamese-specific layer (CCCD, MST, VN phone formats); FR-AI-013 adds the recall-floor CI gate (≥ 99% precision on a curated 200-sample test set).

---

## §2 — Why this design (rationale for humans)

**Why Presidio?** Microsoft's open-source Presidio is the leading non-cloud PII redaction stack. It runs locally (no third-party API), has good EN coverage out of the box (the 10 default types), and is extensible via custom recognizers (FR-AI-012 adds VN-specific patterns). Cloud DLP alternatives (AWS Comprehend Medical, GCP DLP, Azure PII Detection) all require sending the prompt to a third-party API — defeating the point of redaction. We considered AWS DLP and rejected it: even though our compute is on AWS Bedrock, the DLP product is region-locked and the tenant residency rules (FR-AI-005) require some prompts stay in `ap-southeast-1`, where DLP isn't fully available.

**Why a sidecar (Python) instead of an in-process Rust library?** Presidio is Python and slow to load — the spaCy NER models alone take ~3 seconds to initialise. Running it as a sidecar with a warm process pool gives us 5-15ms redaction latency per request vs 3+ seconds cold-start. The sidecar is localhost-only — bound to `127.0.0.1:5050` — so there's no network egress, no compliance exposure, no DDoS surface. PyO3 (Rust calling Python) was considered and rejected: Presidio's transitive deps (spaCy + transformers + NLTK) make in-process embedding hostile to Rust's static-linking ergonomics, and the Python GIL limits concurrency to one redaction at a time per process.

**Why fail closed on sidecar unreachable?** If we proceeded unredacted, every prompt with PII would leak to the LLM AND get persisted in the memory audit row — a compliance breach for every affected request. The cost of refusing the call (operator sees `503`, retries) is tiny compared to the cost of a single PII leak (which could trigger a regulator notification, a public disclosure, brand damage, contract loss). The "open the door if the metal detector breaks" alternative is unconscionable for a compliance gate.

**Why typed placeholders (`<EMAIL_1>`) instead of generic `[REDACTED]`?** Two reasons. (1) The LLM can still reason about the structure of the prompt — "send an email to <EMAIL_1>" is parseable as an instruction; "send an email to [REDACTED]" is ambiguous (was that originally a name? an email?). (2) Tool-call restoration becomes mechanical — when the LLM responds with a tool call `{"to": "<EMAIL_1>"}`, the gateway re-injects the real email by literal-string lookup. Generic placeholders would require fuzzy matching, which is fragile.

**Why don't we persist the restoration map?** It's the secret key that re-PIIs the response. Persisting it would create a centralised PII store that defeats redaction's purpose — auditors would correctly flag "your gateway maintains a database that maps every placeholder to the original PII value, indexed by request ID; that's exactly the kind of repository GDPR Article 5 says you shouldn't have." Memory-only with explicit `Drop` zeroizing is the right contract.

**Why restore in tool-call args but NOT in text response fields?** Asymmetric trust. Tool-call args are structured: the LLM has produced a literal `<EMAIL_1>` token because it wants the system to send to that email. Restoration is mechanical and the result feeds an automated action (sending email, calling API). Text response fields are free-form: if the LLM produces "I sent the email to <EMAIL_1>", restoring would leak the email back to a possibly-untrusted UI. The conservative default is "restore only what the gateway will use to act; never leak in human-visible text". Tenants who want full restoration can explicitly request it via the `restore_in_text: true` policy field (slice 5).

**Why does idempotency matter?** Two angles. (1) Tenant DX: a developer testing the same prompt twice should see the same placeholders ("did my code change the redaction?"). (2) Audit: the memory row's `pii_redactions` count must match between dry-run and real calls; idempotency ensures comparing two runs is deterministic. Idempotency is achieved by sorting Presidio's analyzer results by `start` offset and assigning placeholder indices in order — the same input prompt produces the same numeric suffixes.

**Why do `RedactError` variants forbid prompt fragments?** A common bug pattern: the error message includes the input as context for debugging ("failed to redact prompt: '<first 100 chars>'"). This leaks the very PII we just refused to send. The `RedactError::SidecarError` variant's `message` MUST be the sidecar's GENERIC textual error code (e.g., `"recognizer_init_failed"`, `"analyzer_timeout"`), NEVER an echo of the request. A dedicated CI lint (`grep`-based) flags any `format!` in the redact module that interpolates a prompt-derived value into an error.

**Why the `127.0.0.1` bind requirement?** Defense in depth. If the sidecar bound to `0.0.0.0`, an in-pod attacker (or a k8s NetworkPolicy misconfiguration) could call the redact endpoint and exfiltrate prompts. Localhost-bind makes that attack surface zero — the sidecar is reachable only via the loopback interface, which an in-pod process can use but no cross-pod or cross-node traffic can. The deploy-time integration test (`assert_sidecar_loopback_only`) verifies this by attempting to connect from a different pod's IP and asserting refusal.

**Why no DEBUG/TRACE logging of raw prompts?** A common debugging temptation: "let me log the prompt at DEBUG level so I can see what the user sent". This is the single highest-risk pattern for PII leak — DEBUG logs ship to log aggregators (Datadog, Splunk) and persist for 30+ days; operators across geos can view them. The redact module's `tracing` calls MUST always log the REDACTED form and counts only. The dedicated test `redact_no_log_test.rs` captures emitted records and asserts no raw email/phone/etc. appears.

**Why the sev-1 alarm on `sidecar_unreachable > 5/60s`?** A persistent sidecar-down state means every chat call is failing with `503` — that's a full chat-module outage. Sev-1 (page immediately) is the right severity. The threshold of 5 events in 60s avoids paging on a single transient blip (which the next request will retry past) while catching real outages within seconds.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signatures

```rust
// services/ai-gateway/src/redact.rs

/// Redact PII from a prompt using the Presidio sidecar.
/// Idempotent: same (prompt, policy_snapshot) → same redacted_text + same placeholders.
/// Fails closed: any sidecar error → Err (caller returns 503).
pub async fn redact(prompt: &str, policy: &TenantPolicy) -> Result<RedactionResult, RedactError>;

/// Restore typed placeholders in the LLM response.
/// MUST be called only on tool-call argument fields, NEVER on free-form text response fields
/// (per §1 #5).
pub fn restore(text: &str, map: &RestorationMap) -> String;
```

### Types

```rust
// services/ai-gateway/src/redact/types.rs

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionResult {
    pub redacted_text: String,
    pub map: RestorationMap,                   // ephemeral; not persisted; zeroed on Drop
    pub counts: HashMap<PiiType, u32>,         // for audit row
    pub latency_ms: u32,
}

/// Restoration map. Drop impl zeroizes the underlying string memory to avoid PII
/// lingering in process heap.
#[derive(Debug, Default)]
pub struct RestorationMap {
    inner: HashMap<String, zeroize::Zeroizing<String>>,
}

impl RestorationMap {
    pub fn get(&self, placeholder: &str) -> Option<&str> {
        self.inner.get(placeholder).map(|z| z.as_str())
    }
    pub fn insert(&mut self, placeholder: String, value: String) {
        self.inner.insert(placeholder, zeroize::Zeroizing::new(value));
    }
}

// Drop is auto-derived because Zeroizing<String> zeros on its own Drop.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PiiType {
    CreditCard, UsSsn, EmailAddress, PhoneNumber, Person, Location,
    IpAddress, IbanCode, UsBankNumber, MedicalLicense,
    /* Slice-3 extensions for FR-AI-012 (declared here for ABI stability) */
    VnCccd, VnMst, VnPhone, VnAddress,
}

impl PiiType {
    /// Stable string for OBS metric labels and audit-row keys.
    /// MUST match the Presidio entity-type names so the sidecar response parses cleanly.
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::CreditCard => "credit_card",
            Self::UsSsn => "us_ssn",
            Self::EmailAddress => "email_address",
            Self::PhoneNumber => "phone_number",
            Self::Person => "person",
            Self::Location => "location",
            Self::IpAddress => "ip_address",
            Self::IbanCode => "iban_code",
            Self::UsBankNumber => "us_bank_number",
            Self::MedicalLicense => "medical_license",
            Self::VnCccd => "vn_cccd",
            Self::VnMst => "vn_mst",
            Self::VnPhone => "vn_phone",
            Self::VnAddress => "vn_address",
        }
    }

    /// Maps Presidio's UPPER_SNAKE entity type to our enum.
    pub fn from_presidio(s: &str) -> Option<Self> {
        match s {
            "CREDIT_CARD" => Some(Self::CreditCard),
            "US_SSN" => Some(Self::UsSsn),
            "EMAIL_ADDRESS" => Some(Self::EmailAddress),
            "PHONE_NUMBER" => Some(Self::PhoneNumber),
            "PERSON" => Some(Self::Person),
            "LOCATION" => Some(Self::Location),
            "IP_ADDRESS" => Some(Self::IpAddress),
            "IBAN_CODE" => Some(Self::IbanCode),
            "US_BANK_NUMBER" => Some(Self::UsBankNumber),
            "MEDICAL_LICENSE" => Some(Self::MedicalLicense),
            "VN_CCCD" => Some(Self::VnCccd),
            "VN_MST" => Some(Self::VnMst),
            "VN_PHONE" => Some(Self::VnPhone),
            "VN_ADDRESS" => Some(Self::VnAddress),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum RedactError {
    /// Sidecar process unreachable (connection refused, DNS fail, etc.).
    /// `reason` is the underlying transport error class — NEVER includes the prompt.
    SidecarUnreachable { reason: String },
    /// Sidecar didn't respond within SIDECAR_TIMEOUT (2s).
    SidecarTimeout { waited_ms: u32 },
    /// Sidecar returned non-2xx. `message` is the sidecar's GENERIC error code,
    /// NEVER an echo of the prompt (per §1 #12).
    SidecarError { status: u16, message: String },
    /// Prompt failed pre-validation (e.g., > 64KB).
    InvalidPrompt { reason: String },
}
```

### Presidio sidecar (Python)

```python
# services/ai-gateway/pii/presidio_sidecar.py
from fastapi import FastAPI, HTTPException
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from presidio_analyzer import AnalyzerEngine
from presidio_anonymizer import AnonymizerEngine
from presidio_anonymizer.entities import OperatorConfig

app = FastAPI()

# ISS-003 fix: FastAPI's default 422 handler echoes the request body in the response,
# which can leak prompt fragments (FR-AI-011 §1 #12 forbids this). Replace with a
# generic-message handler.
@app.exception_handler(RequestValidationError)
async def custom_validation_handler(request, exc):
    return JSONResponse(status_code=422, content={"detail": "validation_error"})
analyzer = AnalyzerEngine()
anonymizer = AnonymizerEngine()

DEFAULT_ENTITIES = [
    "CREDIT_CARD", "US_SSN", "EMAIL_ADDRESS", "PHONE_NUMBER", "PERSON",
    "LOCATION", "IP_ADDRESS", "IBAN_CODE", "US_BANK_NUMBER", "MEDICAL_LICENSE",
]

def placeholder_operator(entity_type: str):
    """Replace each occurrence with <ENTITY_TYPE_N> where N is the per-type index."""
    counter = {"n": 0}
    def op(original, _params):
        counter["n"] += 1
        return f"<{entity_type}_{counter['n']}>"
    return OperatorConfig("custom", {"lambda": op})

@app.post("/redact")
async def redact(req: RedactRequest) -> RedactResponse:
    try:
        results = analyzer.analyze(
            text=req.text,
            language="en",
            entities=DEFAULT_ENTITIES + req.extra_entities,
        )
        # Sort by start offset for deterministic placeholder assignment (§1 #11 idempotency).
        results.sort(key=lambda r: r.start)
        anonymized = anonymizer.anonymize(
            text=req.text,
            analyzer_results=results,
            operators={e: placeholder_operator(e) for e in DEFAULT_ENTITIES + req.extra_entities},
        )
        return RedactResponse(
            redacted_text=anonymized.text,
            items=[{"entity": r.entity_type, "start": r.start, "end": r.end,
                    "original": req.text[r.start:r.end]} for r in results],
        )
    except Exception as e:
        # GENERIC error message; do NOT echo the prompt (§1 #12).
        raise HTTPException(status_code=500, detail="redaction_internal_error")

# Bind to 127.0.0.1 ONLY — never 0.0.0.0 (§1 #15).
if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=5050)
```

### Latency budget

```
Wire latency (loopback HTTP):    ~1ms
Presidio analyzer:               ~5-15ms (depends on prompt length)
Presidio anonymizer:             ~2-5ms
Rust client + JSON parse:        ~2ms
─────────────────────────────────────
Total p95:                       ~30ms (8KB prompt budget)
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Credit card redacted** — `"My card is 4111-1111-1111-1111"` → `"My card is <CREDIT_CARD_1>"` with map `{ "<CREDIT_CARD_1>": "4111-1111-1111-1111" }`. Counts: `{ CreditCard: 1 }`.
2. **Multiple PII types** — `"Email john@example.com phone +84-90-123-4567"` → `"Email <EMAIL_ADDRESS_1> phone <PHONE_NUMBER_1>"`. Counts: `{ EmailAddress: 1, PhoneNumber: 1 }`.
3. **No PII passthrough** — `"What is the weather in Singapore?"` → text unchanged (Singapore is a LOCATION, so actually `"What is the weather in <LOCATION_1>?"`); test uses a no-PII string like `"What is 2+2?"` → unchanged; counts all zero.
4. **Counts accurate** — Input with 2 emails + 1 SSN → counts `{ EmailAddress: 2, UsSsn: 1 }`.
5. **Sidecar unreachable** — Sidecar process killed. Caller sees `RedactError::SidecarUnreachable`; gateway returns `503 SERVICE_UNAVAILABLE` (`pii_redaction_unavailable`); no call goes to LLM. `ai_redact_calls_total{outcome="sidecar_unreachable"}` increments.
6. **30ms p95 latency** — 1000 random prompts (1-8KB each); p95 ≤ 30ms, measured by Criterion benchmark.
7. **Restoration round-trip** — `restore(LLM_response_with_placeholders, map)` correctly substitutes typed placeholders back to original values.
8. **Restoration only for tool-call args** — A plain-text response that says "I sent the email to <EMAIL_1>" is RETURNED AS-IS to the caller (no restoration in text); only tool-call argument fields restore. AC test asserts text-field placeholders survive verbatim.
9. **Audit row carries counts only** — memory `ai.invocation` row's `extra.pii_redactions = {"email_address": 2, ...}`; no original PII values appear anywhere in the chain. Test inspects the audit row JSON and asserts no email/phone/SSN regex matches.
10. **Concurrent redactions isolated** — 100 concurrent redactions; each gets its own restoration map; no cross-request bleeding. Verified by giving each request a unique sentinel email and asserting each result map contains only its own.
11. **Idempotency** — `redact(P, policy)` called twice with the same `(P, policy)` produces identical `redacted_text` and identical `map` keys/values. The placeholder counter is per-request (resets each call); the deterministic sort guarantees same-input → same-output.
12. **No PII in error variants** — `RedactError::SidecarError { message }` MUST NOT contain any character sequence matching common PII regex (email, SSN, credit-card). Test inspects the error from a contrived sidecar that returns "your prompt was: foo@bar.com" (which the sidecar SHOULD NOT do per §1 #12) and asserts the Rust error variant has filtered/replaced the message.
13. **No raw prompt in logs** — Test attaches a tracing subscriber, runs `redact("user@example.com")`, and asserts no captured log record contains `"user@example.com"` at any level (DEBUG/TRACE/INFO/WARN/ERROR).
14. **Sev-1 alarm on persistent sidecar-down** — Configure 6 sidecar-down events in 60s; assert the OBS alarm fires (verifiable via metrics-test scaffold).
15. **Sidecar localhost-only bind** — Deploy-time integration test attempts to connect to the sidecar from a non-loopback IP; assertion: connection refused. Verifies §1 #15.
16. **Sidecar timeout (>2s)** — Sidecar returns response after 3s; `RedactError::SidecarTimeout { waited_ms: 2000 }`; gateway returns 503.
17. **Original PII never in memory chain (feature-request-audit skill §3.6 rule 18)** — Round-trip test: redact `"email a@b.com mst 0312345678"`, then run `cost_ledger::precheck` which emits a memory row. Read back the row via `MemoryStore::read_last_row()` and assert `row.extra.prompt_snippet.contains("a@b.com") == false` AND `row.extra.prompt_snippet.contains("0312345678") == false`. The redacted form `<EMAIL_ADDRESS_1>` / `<VN_MST_1>` MUST be the only form present.
18. **`redact_for_log` lint enforced (feature-request-audit skill §3.6 rule 19)** — A `cargo clippy --all-targets --all-features -- -D clippy::disallowed_methods` run on a synthetic file containing `tracing::info!(?prompt)` MUST exit non-zero. The lint config in `clippy.toml` lists the disallowed direct log methods.

---

## §5 — Verification

**Integration test:** `services/ai-gateway/tests/redact_test.rs`

```rust
use cyberos_ai_gateway::redact::{self, RedactionResult, RedactError, types::PiiType};
use cyberos_ai_gateway::policy::TenantPolicy;
use std::collections::HashMap;

mod mocks;
use mocks::{test_policy, mock_sidecar_with_response, kill_sidecar};

#[tokio::test]
async fn redacts_credit_card() {
    let res = redact::redact("My card is 4111-1111-1111-1111", &test_policy()).await.unwrap();
    assert!(res.redacted_text.contains("<CREDIT_CARD_1>"));
    assert!(!res.redacted_text.contains("4111-1111-1111-1111"));
    assert_eq!(res.counts[&PiiType::CreditCard], 1);
    assert_eq!(res.map.get("<CREDIT_CARD_1>"), Some("4111-1111-1111-1111"));
}

#[tokio::test]
async fn redacts_multiple_types() {
    let res = redact::redact("Email john@example.com phone +84-90-123-4567", &test_policy()).await.unwrap();
    assert!(res.redacted_text.contains("<EMAIL_ADDRESS_1>"));
    assert!(res.redacted_text.contains("<PHONE_NUMBER_1>"));
    assert_eq!(res.counts[&PiiType::EmailAddress], 1);
    assert_eq!(res.counts[&PiiType::PhoneNumber], 1);
}

#[tokio::test]
async fn no_pii_passthrough() {
    let res = redact::redact("What is 2 plus 2?", &test_policy()).await.unwrap();
    assert_eq!(res.redacted_text, "What is 2 plus 2?");
    assert!(res.counts.is_empty());
}

#[tokio::test]
async fn counts_accurate_for_multiple_emails_and_ssn() {
    let prompt = "Emails: a@x.com b@y.com SSN: 123-45-6789";
    let res = redact::redact(prompt, &test_policy()).await.unwrap();
    assert_eq!(res.counts[&PiiType::EmailAddress], 2);
    assert_eq!(res.counts[&PiiType::UsSsn], 1);
}

#[tokio::test]
async fn sidecar_unreachable_returns_err() {
    kill_sidecar().await;
    let res = redact::redact("hello", &test_policy()).await;
    assert!(matches!(res, Err(RedactError::SidecarUnreachable { .. })));
}

#[tokio::test]
async fn sidecar_timeout_returns_err() {
    let _g = mock_sidecar_with_response(std::time::Duration::from_secs(3), 200, "{}");
    let res = redact::redact("hello", &test_policy()).await;
    assert!(matches!(res, Err(RedactError::SidecarTimeout { waited_ms })
        if waited_ms == 2000));
}

#[tokio::test]
async fn restoration_round_trip_for_tool_args() {
    let res = redact::redact("Email john@example.com", &test_policy()).await.unwrap();
    let llm_tool_call_arg = res.redacted_text.replace("Email ", "");   // "<EMAIL_ADDRESS_1>"
    let restored = redact::restore(&llm_tool_call_arg, &res.map);
    assert_eq!(restored, "john@example.com");
}

#[tokio::test]
async fn restoration_does_not_apply_to_text_response_fields() {
    // The restore() function is the same; the AC is about CALLER discipline.
    // Test asserts: when the chat handler receives a text-only response with
    // <EMAIL_ADDRESS_1>, it returns AS-IS without calling restore().
    use mocks::run_chat_handler_with_text_response;
    let response = run_chat_handler_with_text_response("Email john@example.com",
        "I sent the email to <EMAIL_ADDRESS_1>").await;
    assert!(response.text.contains("<EMAIL_ADDRESS_1>"));   // placeholder visible to caller
    assert!(!response.text.contains("john@example.com"));   // raw value NOT in text
}

#[tokio::test]
async fn audit_row_carries_counts_only() {
    let prompt = "Send to john@example.com from 123-45-6789";
    let res = redact::redact(prompt, &test_policy()).await.unwrap();
    let audit_extra = mocks::build_invocation_extra(&res);
    let audit_json = serde_json::to_string(&audit_extra).unwrap();

    // Counts present.
    assert!(audit_json.contains("\"email_address\":1"));
    assert!(audit_json.contains("\"us_ssn\":1"));

    // Raw values absent (regex-checked).
    let email_re = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
    let ssn_re = regex::Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap();
    assert!(!email_re.is_match(&audit_json), "audit row leaked email");
    assert!(!ssn_re.is_match(&audit_json), "audit row leaked SSN");
}

#[tokio::test]
async fn concurrent_100_redactions_isolated() {
    let handles: Vec<_> = (0..100).map(|i| {
        tokio::spawn(async move {
            let prompt = format!("Email user{i}@cyberos.world");
            let res = redact::redact(&prompt, &test_policy()).await.unwrap();
            (i, res)
        })
    }).collect();
    let results: Vec<_> = futures::future::join_all(handles).await
        .into_iter().map(|r| r.unwrap()).collect();

    for (i, res) in &results {
        assert_eq!(res.map.get("<EMAIL_ADDRESS_1>").unwrap(), &format!("user{i}@cyberos.world"));
    }
}

#[tokio::test]
async fn idempotency_same_input_same_output() {
    let prompt = "Send to alice@example.com and bob@example.com";
    let r1 = redact::redact(prompt, &test_policy()).await.unwrap();
    let r2 = redact::redact(prompt, &test_policy()).await.unwrap();
    assert_eq!(r1.redacted_text, r2.redacted_text);
    assert_eq!(r1.map.get("<EMAIL_ADDRESS_1>"), r2.map.get("<EMAIL_ADDRESS_1>"));
    assert_eq!(r1.map.get("<EMAIL_ADDRESS_2>"), r2.map.get("<EMAIL_ADDRESS_2>"));
}

// ISS-002 fix: regression test — even with sidecar returning items in unsorted order,
// idempotency holds because Rust re-sorts defensively.
#[tokio::test]
async fn idempotency_holds_when_sidecar_returns_unsorted() {
    let _g = mocks::mock_sidecar_with_unsorted_response();
    let r = redact::redact("Email a@x.com first b@y.com second", &test_policy()).await.unwrap();
    assert_eq!(r.map.get("<EMAIL_ADDRESS_1>"), Some("a@x.com"),
        "first-by-position MUST be <EMAIL_ADDRESS_1> regardless of sidecar order");
    assert_eq!(r.map.get("<EMAIL_ADDRESS_2>"), Some("b@y.com"));
}

// ISS-004 fix: CI test — every Presidio recognizer entity has a PiiType variant.
// Lives in tests/redact_pii_type_coverage_test.rs to avoid pulling Presidio deps into the main test bin.
#[test]
fn every_presidio_entity_has_pii_type_variant() {
    let entities = mocks::sidecar_client::list_entities();
    let unmapped: Vec<_> = entities.iter()
        .filter(|e| PiiType::from_presidio(e).is_none())
        .collect();
    assert!(unmapped.is_empty(),
        "Presidio entities without PiiType variants: {unmapped:?}\n\
         Add variants to PiiType enum AND update from_presidio() match arm.");
}

// ISS-001 fix: AC #14 — sev-1 alarm on persistent sidecar-down.
#[tokio::test]
async fn sev1_alarm_fires_on_persistent_sidecar_down() {
    use mocks::AlarmHarness;
    let alarm = AlarmHarness::watch(
        "ai_redact_calls_total",
        mocks::prometheus_alarm_rule("sidecar_unreachable", 5, std::time::Duration::from_secs(60)),
    );
    kill_sidecar().await;
    for _ in 0..6 {
        let _ = redact::redact("hello", &test_policy()).await;
    }
    alarm.assert_fired_within(std::time::Duration::from_secs(2)).await;
}

// ISS-001 fix: AC #15 — sidecar localhost-only bind. Deploy-smoke test, ignored in unit-test mode.
#[tokio::test]
#[ignore = "requires deploy harness; run via `make smoke-test`"]
async fn assert_sidecar_loopback_only() {
    use mocks::deploy_harness::{spawn_sidecar_pod, spawn_intruder_pod};
    let sidecar_ip = spawn_sidecar_pod().await;
    let intruder = spawn_intruder_pod().await;
    let result = intruder.curl(&format!("http://{sidecar_ip}:5050/redact"), "{}").await;
    assert!(result.is_err(),
        "sidecar accepted non-loopback connection from {}; MUST bind 127.0.0.1 only", intruder.ip());
}

#[tokio::test]
async fn no_prompt_fragment_in_error_variants() {
    let _g = mock_sidecar_with_response(std::time::Duration::from_millis(10), 500,
        r#"{"detail": "your input was: leak@example.com"}"#);
    let err = redact::redact("hello leak@example.com", &test_policy()).await.unwrap_err();
    let err_str = format!("{err:?}");
    assert!(!err_str.contains("leak@example.com"),
        "RedactError leaked prompt fragment: {err_str}");
}
```

**No-log test:** `services/ai-gateway/tests/redact_no_log_test.rs`

```rust
use cyberos_ai_gateway::redact;
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn no_raw_prompt_in_logs() {
    let _ = redact::redact("Send to leak@example.com from 123-45-6789",
        &mocks::test_policy()).await;
    let logs = traced_test::logs_contain("");   // capture all
    assert!(!logs.iter().any(|l| l.contains("leak@example.com")),
        "raw email in logs: {logs:?}");
    assert!(!logs.iter().any(|l| l.contains("123-45-6789")),
        "raw SSN in logs: {logs:?}");
}
```

**Bench:** `services/ai-gateway/benches/redact_latency_bench.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};
fn bench_redact_p95(c: &mut Criterion) {
    c.bench_function("redact 8KB prompt p95", |b| {
        b.iter(|| { /* generate random 8KB prompt; call redact; record latency */ });
    });
}
criterion_group!(benches, bench_redact_p95);
criterion_main!(benches);
```

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
docker compose up -d presidio
cargo test -p cyberos-ai-gateway redact
cargo bench -p cyberos-ai-gateway redact_latency_bench
```

CI gate: bench p95 regression > 10% fails the PR. The `redact_no_log_test.rs` runs on every PR touching `src/redact/**`.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/redact.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tracing::{error, warn};

use crate::policy::TenantPolicy;

pub mod presidio_client;
pub mod restoration;
pub mod types;

pub use types::{RedactionResult, RestorationMap, PiiType, RedactError};

const SIDECAR_URL: &str = "http://127.0.0.1:5050/redact";
const SIDECAR_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_PROMPT_BYTES: usize = 64 * 1024;

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{
        register_counter_vec, register_histogram, register_histogram_vec,
        CounterVec, Histogram, HistogramVec,
    };

    pub static CALLS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_redact_calls_total",
        "Redact calls by outcome",
        &["outcome"]
    ).unwrap());

    pub static LATENCY_MS: Lazy<HistogramVec> = Lazy::new(|| register_histogram_vec!(
        "ai_redact_latency_ms",
        "Redaction latency in ms (loopback HTTP + Presidio analyze + anonymize)",
        &["outcome"],
        vec![5.0, 10.0, 20.0, 30.0, 50.0, 100.0, 250.0]
    ).unwrap());

    pub static PII_TYPES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_redact_pii_types_total",
        "Per-type PII redactions (cardinality bounded by PiiType variants)",
        &["type"]
    ).unwrap());

    pub static PROMPT_SIZE: Lazy<Histogram> = Lazy::new(|| {
        prometheus::register_histogram!(
            "ai_redact_prompt_size_bytes",
            "Prompt size at redact entry",
            vec![512.0, 1024.0, 4096.0, 8192.0, 16384.0, 65536.0]
        ).unwrap()
    });

    // ISS-004 fix: counter for Presidio entities that don't map to a PiiType variant.
    // Operator alarm: rate > 0 means there's silent PII passthrough; add the variant + redeploy.
    pub static UNKNOWN_ENTITIES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_redact_unknown_entity_dropped_total",
        "Presidio reported an entity type the Rust enum doesn't know about",
        &["entity"]
    ).unwrap());
}

pub async fn redact(prompt: &str, policy: &TenantPolicy) -> Result<RedactionResult, RedactError> {
    let started = Instant::now();
    metrics::PROMPT_SIZE.observe(prompt.len() as f64);

    if prompt.len() > MAX_PROMPT_BYTES {
        metrics::CALLS.with_label_values(&["invalid_prompt"]).inc();
        return Err(RedactError::InvalidPrompt {
            reason: format!("prompt size {} exceeds max {} bytes", prompt.len(), MAX_PROMPT_BYTES),
        });
    }

    let extra_entities: Vec<&str> = policy.ai_policy.pii_redaction_extra
        .as_deref().unwrap_or(&[]).iter().map(|s| s.as_str()).collect();

    let req_body = serde_json::json!({
        "text": prompt,
        "extra_entities": extra_entities,
    });

    let resp_result = tokio::time::timeout(
        SIDECAR_TIMEOUT,
        reqwest::Client::new()
            .post(SIDECAR_URL)
            .json(&req_body)
            .send(),
    ).await;

    let resp = match resp_result {
        Err(_) => {
            metrics::CALLS.with_label_values(&["sidecar_timeout"]).inc();
            return Err(RedactError::SidecarTimeout {
                waited_ms: SIDECAR_TIMEOUT.as_millis() as u32,
            });
        }
        Ok(Err(e)) => {
            metrics::CALLS.with_label_values(&["sidecar_unreachable"]).inc();
            // §1 #12: error reason is the connection-error class, NEVER the prompt.
            return Err(RedactError::SidecarUnreachable {
                reason: e.without_url().to_string(),
            });
        }
        Ok(Ok(r)) => r,
    };

    if !resp.status().is_success() {
        metrics::CALLS.with_label_values(&["sidecar_error"]).inc();
        let status = resp.status().as_u16();
        // §1 #12: do NOT echo the response body if it might contain a prompt fragment.
        // Only the GENERIC sidecar error code is allowed; otherwise replace with a placeholder.
        let body = resp.text().await.unwrap_or_default();
        let safe_message = sanitize_sidecar_error_message(&body);
        return Err(RedactError::SidecarError { status, message: safe_message });
    }

    let body: PresidioResponse = resp.json().await
        .map_err(|e| RedactError::SidecarError {
            status: 200,
            message: format!("response_parse_error: {}", e.without_url()),
        })?;

    let (redacted_text, map, counts) = build_placeholder_map_and_counts(prompt, &body);

    // OBS metrics for per-type counts.
    for (ty, n) in &counts {
        metrics::PII_TYPES.with_label_values(&[ty.as_metric_label()]).inc_by(*n as f64);
    }

    let elapsed_ms = started.elapsed().as_millis() as u32;
    metrics::LATENCY_MS.with_label_values(&["ok"]).observe(elapsed_ms as f64);
    metrics::CALLS.with_label_values(&["ok"]).inc();

    // §1 #13: log only the redacted form + counts; NEVER the raw prompt.
    tracing::debug!(
        latency_ms = elapsed_ms,
        counts = ?counts,
        redacted_size_bytes = redacted_text.len(),
        "redact_success"
    );

    Ok(RedactionResult {
        redacted_text,
        map,
        counts,
        latency_ms: elapsed_ms,
    })
}

/// §1 #12: sanitize sidecar error messages to forbid prompt-fragment leaks.
/// ISS-003 fix: switched from denylist (block long messages or those with @/digit-runs)
/// to allowlist (only known generic error codes pass through). Denylist was insufficient
/// because FastAPI's default 422 validation handler can echo request bodies in formats
/// that pass the heuristic (short JSON arrays without @ or digit runs).
fn sanitize_sidecar_error_message(body: &str) -> String {
    const KNOWN_ERROR_CODES: &[&str] = &[
        "redaction_internal_error",
        "validation_error",
        "recognizer_init_failed",
        "analyzer_timeout",
        "anonymizer_failed",
        "response_parse_error",
    ];
    let trimmed = body.trim();
    if KNOWN_ERROR_CODES.iter().any(|code| trimmed.contains(code)) {
        // Even when a known code is present, cap at 128 chars to bound surprise.
        trimmed.chars().take(128).collect()
    } else {
        "sidecar_returned_unrecognized_message_redacted".to_string()
    }
}

#[derive(Debug, Deserialize)]
struct PresidioResponse {
    redacted_text: String,
    items: Vec<PresidioItem>,
}

#[derive(Debug, Deserialize)]
struct PresidioItem {
    entity: String,
    start: usize,
    end: usize,
    original: String,
}

fn build_placeholder_map_and_counts(
    prompt: &str,
    body: &PresidioResponse,
) -> (String, RestorationMap, HashMap<PiiType, u32>) {
    let mut map = RestorationMap::default();
    let mut counts: HashMap<PiiType, u32> = HashMap::new();
    let mut per_type_counter: HashMap<&str, u32> = HashMap::new();

    // ISS-002 fix: defensive re-sort by start offset to guarantee §1 #11 idempotency
    // regardless of sidecar's response order. The sidecar's Python sort is the primary
    // contract; this Rust sort is belt-and-suspenders against sidecar regressions.
    let mut sorted_items: Vec<&PresidioItem> = body.items.iter().collect();
    sorted_items.sort_by_key(|item| item.start);

    for item in sorted_items {
        // ISS-004 fix: warn + counter on unknown entities so operators see PII passthrough
        // before the CI test catches it on next PR.
        let Some(ty) = PiiType::from_presidio(&item.entity) else {
            tracing::warn!(entity = %item.entity,
                "presidio_unknown_entity_dropped; PII not redacted; add variant to PiiType enum");
            metrics::UNKNOWN_ENTITIES.with_label_values(&[&item.entity]).inc();
            continue;
        };
        let n = per_type_counter.entry(ty.as_metric_label()).and_modify(|c| *c += 1).or_insert(1);
        let placeholder = format!("<{}_{}>", item.entity, n);
        map.insert(placeholder.clone(), item.original.clone());
        *counts.entry(ty).or_insert(0) += 1;
    }

    (body.redacted_text.clone(), map, counts)
}

pub fn restore(text: &str, map: &RestorationMap) -> String {
    let mut out = text.to_string();
    for (placeholder, value) in map.inner.iter() {
        out = out.replace(placeholder, value.as_str());
    }
    out
}
```

```rust
// services/ai-gateway/src/redact/restoration.rs

// RestorationMap type with Drop-via-Zeroizing is in src/redact/types.rs.
// Restore logic lives at the redact module root.
// This module is reserved for future expansion (FR-AI-022 partial restore policies).
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-001 / FR-AI-002** — `cost_ledger::precheck` runs BEFORE `redact()`; `cost_ledger::reconcile` runs AFTER `router::call_provider` returns; the memory `ai.invocation` row's `extra.pii_redactions` field is populated from `RedactionResult.counts`.
- **FR-AI-005** — `TenantPolicy.ai_policy.pii_redaction_extra` lives in the policy schema.
- **FR-AI-008** — `router::call_provider` is invoked AFTER redact returns; the redacted prompt is what reaches the LLM.
- **FR-AI-012 (downstream)** — Adds VN-specific recognizers (CCCD, MST, VN_PHONE, VN_ADDRESS) to the sidecar's analyzer registry. The Rust enum already includes the variants.
- **FR-AI-013 (downstream)** — Recall-floor CI gate runs the test fixture set against this redact() implementation.

### Concept dependencies (shared types)

- `PiiType::as_metric_label()` and `PiiType::from_presidio()` — the bidirectional mapping between Presidio's UPPER_SNAKE entity names and our snake_case metric labels.
- `RestorationMap` Drop semantics rely on `zeroize::Zeroizing<String>` — must be in Cargo.toml.
- The `tracing` calls in redact MUST NOT log prompt fragments (per §1 #13). The `redact_no_log_test.rs` enforces this.

### Operational / external

- `presidio-analyzer==2.2.x`, `presidio-anonymizer==2.2.x`, `spacy>=3.7`, `fastapi>=0.110` (Python sidecar).
- `reqwest` v0.12 with `rustls` features (Rust client).
- `serde` v1, `serde_json` v1 (JSON serde).
- `zeroize` v1.7+ (memory zeroing for RestorationMap).
- `tracing` v0.1 (structured logging).
- `prometheus` v0.13 (OBS).
- `tracing-test` v0.2 (test-only; for `no_raw_prompt_in_logs` test).
- Deploy: `docker-compose` or k8s sidecar pattern; presidio container on `127.0.0.1:5050`.

---

## §8 — Example payloads

### Request to redact

```json
{
  "text": "Send a thank-you to john@cyberos.world for closing the deal on +84-90-123-4567"
}
```

### Sidecar response

```json
{
  "redacted_text": "Send a thank-you to <EMAIL_ADDRESS_1> for closing the deal on <PHONE_NUMBER_1>",
  "items": [
    {"entity": "EMAIL_ADDRESS", "start": 19, "end": 38, "original": "john@cyberos.world"},
    {"entity": "PHONE_NUMBER", "start": 65, "end": 79, "original": "+84-90-123-4567"}
  ]
}
```

### Rust RedactionResult

```rust
RedactionResult {
    redacted_text: "Send a thank-you to <EMAIL_ADDRESS_1> for closing the deal on <PHONE_NUMBER_1>",
    map: { "<EMAIL_ADDRESS_1>" → "john@cyberos.world",
           "<PHONE_NUMBER_1>"  → "+84-90-123-4567" },
    counts: { EmailAddress: 1, PhoneNumber: 1 },
    latency_ms: 18,
}
```

### LLM tool call (placeholder)

```json
{ "tool": "send_email", "args": { "to": "<EMAIL_ADDRESS_1>", "body": "Thank you for closing!" } }
```

### After restore (executed)

```json
{ "tool": "send_email", "args": { "to": "john@cyberos.world", "body": "Thank you for closing!" } }
```

### memory audit row excerpt

```yaml
extra:
  pii_redactions:
    email_address: 1
    phone_number: 1
  # Note: NO raw values anywhere in this row.
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- VN-specific recognizers (FR-AI-012).
- Recall-floor CI gate (FR-AI-013).
- Tenant-level partial-restore policies (`restore_in_text: true`) — slice 5.
- LLM-provider-side PII redaction (e.g., Bedrock Guardrails) — slice 5; redundant for now.
- Per-tenant custom recognizers (e.g., proprietary product SKUs treated as PII) — out of scope for slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Sidecar process down | reqwest connection error | `Err(SidecarUnreachable)` → `503` from gateway | Operator restarts sidecar; pod restart in k8s |
| Sidecar timeout (>2s) | `tokio::time::timeout` fires | `Err(SidecarTimeout { waited_ms: 2000 })` → `503` | Investigate sidecar load; scale presidio replicas |
| Sidecar non-2xx | HTTP status check | `Err(SidecarError)` → `503`; sanitize message | Sev-1 log; investigate sidecar version mismatch |
| Sidecar response unparseable | serde error | `Err(SidecarError { status: 200 })` → `503` | Sev-1 log; investigate sidecar version mismatch |
| PII type unrecognised | `PiiType::from_presidio` returns None | Item silently skipped (will not be in counts or map) | Operator adds the type to PiiType enum; redeploy |
| Restoration map collision (same placeholder, two different values) | per-type counter increments → `<EMAIL_1>`, `<EMAIL_2>` | No collision possible — counter is per-(type, request) | By design |
| Prompt too large (>64KB) | size check at redact entry | `Err(InvalidPrompt)` → `413 PAYLOAD_TOO_LARGE` | Caller reduces prompt size |
| Concurrent restoration race | Map is per-request; no shared state | No issue; isolation guaranteed | No-op |
| Sidecar bound to 0.0.0.0 instead of 127.0.0.1 | Deploy-time integration test (`assert_sidecar_loopback_only`) | CI/CD pipeline fails | Operator fixes deploy manifest |
| Sidecar leaks prompt in error message | `sanitize_sidecar_error_message` allowlist filter | Replaced with `"sidecar_returned_unrecognized_message_redacted"` unless message contains a known error code | Sev-2 log; file ticket against sidecar |
| FastAPI default 422 body echo on malformed request | Custom `RequestValidationError` handler in sidecar | Returns `{"detail": "validation_error"}`; allowlist sanitizer in Rust catches anything missed | ISS-003 fix |
| Presidio reports an entity type with no `PiiType` variant | `from_presidio` returns None | Item dropped + `tracing::warn!` + `ai_redact_unknown_entity_dropped_total{entity}` increments | CI test `every_presidio_entity_has_pii_type_variant` fails on next PR — ISS-004 fix |
| `tracing::debug!` called with raw prompt | `redact_no_log_test.rs` CI gate | Test fails on every PR that introduces this | PR rework |
| RestorationMap accidentally serialized to memory | grep-based CI lint on src/cost_ledger/ | Lint fails on PR | PR rework |
| Persistent sidecar-down (>5 events / 60s) | OBS alarm rule | Sev-1 page to operator | Operator action |
| spaCy model corrupted on disk | sidecar startup error | Sidecar refuses to bind; gateway sees connection-refused; cascades to sev-1 | Operator re-pulls sidecar image |
| PiiType enum out of sync with sidecar registry | unknown `entity` in sidecar response → `from_presidio` None | Item dropped; potential PII passthrough | CI test asserts every Presidio entity has a PiiType variant |
| `Zeroize` impl missing for RestorationMap (regression) | compile-time (Zeroizing<String> field) | Build fails | Compiler error caught in CI |

---

## §11 — Notes

- Default 10 EN types is conservative — covers GDPR Art. 5(1)(c) data-minimisation, Singapore PDPA s.18 purpose limitation, and U.S. HIPAA medical-license redaction. FR-AI-012 adds the VN-specific layer for PDPL Art. 7.
- The sidecar pattern adds operational complexity (one more process to monitor). PyO3 alternative was considered but rejected: Presidio's transitive dependency surface (spaCy, NLTK, transformers) makes in-process embedding hostile to Rust's static-linking ergonomics; the GIL also limits in-process concurrency to one redaction at a time.
- Recall floor is FR-AI-013's job. This FR ensures the redaction RUNS; FR-AI-013 ensures it CATCHES ≥ 99% of real-world cases on a curated 200-sample test set. Slice 3 is incomplete without both FRs landed.
- The `RestorationMap` uses `zeroize::Zeroizing<String>` so the underlying allocation is wiped on Drop. This defends against memory-dump-based PII recovery (e.g., from a heap snapshot triggered by a panic). Without zeroization, the original PII could linger in process heap until reallocated.
- The `sanitize_sidecar_error_message` function is conservative — any error message longer than 64 bytes OR containing `@` OR containing 5+ consecutive digits is replaced with a placeholder. This may hide genuine sidecar errors (loss of debuggability) but the trade-off favours prompt-leak prevention. Operator workflow: if sidecar errors are mysterious, attach a sidecar-side debugger; never lower this filter.
- The deploy-time test `assert_sidecar_loopback_only` runs as part of the `make smoke-test` target. It spawns a temporary pod alongside the AI gateway and tries to `curl http://<gateway-ip>:5050/redact` — assertion: connection refused. Anything other than refusal (a redirect, a 404, a timeout) fails the smoke test.
- Per-tenant custom recognizers (e.g., proprietary SKU patterns) are not in slice 3. The `pii_redaction_extra` policy field accepts entity-type strings; FR-AI-012 wires the VN types; future FRs may add per-tenant custom recognizers via a `pii_recognizers_yaml` policy field that Presidio loads at sidecar startup.
- The `tracing-test` crate is dev-only — it captures emitted log records into an in-memory buffer for assertion. Production builds don't include it. The `redact_no_log_test.rs` test runs on every PR but is excluded from release-mode binaries.
- Future work (FR-AI-022): semantic-similarity redaction (e.g., a paraphrased SSN like "my social is twelve thirty-four fifty-six seven eight nine") that current Presidio doesn't catch. Requires LLM-based detection or richer regex; out of scope for slice 3 baseline.

---

*End of FR-AI-011. Status: draft (10/10 target).*
