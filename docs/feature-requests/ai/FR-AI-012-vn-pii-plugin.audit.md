---
fr_id: FR-AI-012
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (213 lines)
score_post_expansion: 9.0/10      # after expanding to FR-AI-001 depth (~840 lines)
score_post_revision: 10/10         # after 4 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes)
---

## §1 — Verdict summary

FR-AI-012 was expanded from 213 lines (compressed first-pass) to ~840 lines matching FR-AI-001 depth. The expansion added 11 §1 clauses (#5 province-code validation, #6 MST format, #7 PHONE-vs-MST registration order, #8 NDD label-bound, #9 partial-redact stub, #10 BANK distinguishing, #11 determinism, #12 no-network-calls, #13 sidecar-refuses-on-registration-failure, #14 version endpoint, #15 startup logging), 7 additional §2 paragraphs (custom-vs-Presidio-EN gap, context-sensitive matching, partial-redact rationale, single-source province codes, single-source confidence constants, registration-order rationale, no-network-calls rationale, label-bound NDD rationale), full Python class skeletons in §3 for all 6 recognizers + shared modules (`province_codes.py`, `confidence.py`), 5 additional §4 ACs (#13 recall ≥99% on 200-sample, #14 registration order, #15 sidecar refuses to start, #16 version endpoint, #17 determinism), full pytest bodies in §5 (12 positive tests + 3 negative tests + recall-floor test + sidecar-startup test), expanded §6 with fixture file structure + test runner output, code/concept/operational deps in §7, 5 example payloads in §8 including version endpoint and startup logs, 16 failure modes in §10, 8 implementation notes in §11.

Four residual issues prevent 10/10.

## §2 — Findings

### ISS-001 — AC #16 (version endpoint) lacks a test body in §5
- **severity:** error
- **rule_id:** test-coverage
- **location:** §4 AC #16, §5 (verification)
- **status:** open

#### Description
AC #16 says: *"`GET /recognizers/version` returns JSON with keys `VN_CCCD`, `VN_MST`, `VN_PHONE`, `VN_NDD`, `VN_ADDRESS`, `VN_BANK_ACCOUNT`; values are semver strings."*

§5 has 16 test bodies but none of them call the version endpoint or assert the response shape. A code-gen agent reading the FR has no template for the test.

This is the same pattern as FR-AI-007 ISS-001, FR-AI-008 ISS-002, FR-AI-009 ISS-001, FR-AI-010 ISS-001, FR-AI-011 ISS-001 — ACs reference behaviors without matching test bodies.

#### Suggested fix
Add the test to §5:

```python
def test_version_endpoint_returns_six_entries():
    """AC #16: every registered recognizer's version is exposed."""
    from fastapi.testclient import TestClient
    from presidio_sidecar import app
    client = TestClient(app)
    resp = client.get("/recognizers/version")
    assert resp.status_code == 200
    body = resp.json()
    expected_keys = {"VN_CCCD", "VN_MST", "VN_PHONE", "VN_NDD", "VN_ADDRESS", "VN_BANK_ACCOUNT"}
    assert set(body.keys()) == expected_keys
    import re
    semver = re.compile(r"^\d+\.\d+\.\d+$")
    for entity, version in body.items():
        assert semver.match(version), f"{entity}: {version!r} is not semver"
```

### ISS-002 — §1 #12 "no network calls" not verified by any test or lint
- **severity:** error
- **rule_id:** correctness / test-coverage
- **location:** §1 #12, §5 (verification)
- **status:** open

#### Description
§1 #12 mandates: *"MUST NOT make any network call. All recognizers are pure regex + lookup tables."*

This is a load-bearing claim — a network call from a recognizer would (a) blow the 30ms p95 latency budget (FR-AI-011 §1 #6), (b) introduce a failure mode where network-down → recognizer-skip → PII passthrough, (c) potentially leak prompt fragments to whatever endpoint was called. Yet there is no test or CI lint that enforces "no `import requests`, no `import urllib`, no `import http.client`, no `import socket` in `recognizers/*.py`".

A future PR could accidentally add a network call (e.g., "let me fetch the latest province code list from gov.vn at startup") and slip past review.

#### Suggested fix
Add a CI lint test:

```python
# services/ai-gateway/pii/test_no_network_imports.py
import ast
from pathlib import Path

FORBIDDEN_IMPORTS = {
    "requests", "urllib", "urllib2", "urllib3", "httpx", "aiohttp",
    "http.client", "socket", "ssl", "ftplib", "smtplib", "asyncio.open_connection",
}

def test_no_network_imports_in_recognizers():
    """§1 #12: recognizer modules MUST be pure regex + lookup tables."""
    recognizers_dir = Path(__file__).parent / "recognizers"
    for py_file in recognizers_dir.glob("*.py"):
        tree = ast.parse(py_file.read_text())
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    assert alias.name.split(".")[0] not in FORBIDDEN_IMPORTS, \
                        f"{py_file.name}: forbidden network import: {alias.name}"
            elif isinstance(node, ast.ImportFrom):
                assert node.module is None or node.module.split(".")[0] not in FORBIDDEN_IMPORTS, \
                    f"{py_file.name}: forbidden network import-from: {node.module}"
```

Add §10 row: *"Future PR adds network call in a recognizer → `test_no_network_imports.py` fails on PR."*

### ISS-003 — `register_vn_recognizers` is not idempotent; double-call silently adds duplicates
- **severity:** warning
- **rule_id:** robustness
- **location:** §3 (`register_vn_recognizers` function), §6 (sidecar startup)
- **status:** open

#### Description
The function uses `analyzer.registry.add_recognizer(rec)` directly. Presidio's `add_recognizer` doesn't dedupe — calling it twice with the same recognizer instance adds two copies to the registry. Result: every text gets analyzed by the same recognizer twice; the analyzer returns duplicate `RecognizerResult` entries; the anonymizer picks one but the OBS metric counts double; latency increases.

The §6 skeleton calls `register_vn_recognizers` exactly once at sidecar startup, so the bug doesn't trigger in production. But in tests (especially the `test_sidecar_refuses_to_start_on_registration_failure` test from §5), repeated test runs in the same Python process accumulate duplicates.

This is the same pattern as FR-AI-009 ISS-004 — `init`-style functions need either guard-against-double-call OR an explicit reset.

#### Suggested fix
Add an idempotency guard:

```python
_REGISTERED = False

def register_vn_recognizers(analyzer: AnalyzerEngine):
    """§1 #13: refuse to start if any registration fails. Idempotent: subsequent calls no-op."""
    global _REGISTERED
    if _REGISTERED:
        logger.warning("register_vn_recognizers called twice; ignoring second invocation")
        return
    for rec in VN_RECOGNIZERS:
        try:
            analyzer.registry.add_recognizer(rec)
            logger.info(f"registered {rec.__class__.__name__} v{rec.VERSION} "
                        f"({len(getattr(rec, 'PATTERNS', []))} patterns)")
        except Exception as e:
            raise RuntimeError(f"recognizer_registration_failed: {rec.__class__.__name__}: {e}")
    _REGISTERED = True

def reset_for_tests():
    """Test-only: reset the registration guard. NOT for production use."""
    global _REGISTERED
    _REGISTERED = False
```

Add §10 row: *"`register_vn_recognizers` called twice (test loop, sidecar reload) → second call is a no-op with WARN log; tests use `reset_for_tests()` for clean state."*

### ISS-004 — Recall test only checks aggregate ≥99%; per-type recall floor missing
- **severity:** warning
- **rule_id:** correctness / test-coverage
- **location:** §1 #1 ("recall ≥ 99%"), §4 AC #13, §5 (`test_recall_at_least_99_percent_on_200_sample_fixture`)
- **status:** open

#### Description
The aggregate recall test computes `correct / total` across the entire 200-sample fixture. If the fixture has 50 CCCD samples and 50 PHONE samples (and 100 of other types), and the CCCD recognizer regresses to 0% recall while PHONE stays at 100%, the aggregate becomes:
- `(0 + 50 + ~99) / 200 = ~0.745` — yes, this would catch a 0% regression.

But if the regression is more subtle — say CCCD drops from 100% to 90% (5 misses out of 50), and PHONE stays at 100%:
- `(45 + 50 + 99) / 200 = 0.97` — FAILS the gate, but operator can't tell from the assertion message which recognizer regressed.

And in a more pernicious case: CCCD has 50 samples, drops 1, but the fixture mostly biases toward easier types (50% of samples are obvious labeled patterns):
- Aggregate stays at ≥99% even with one type's recall at 96%.

§1 #1 says "recall ≥ 99%" without specifying per-type. In practice, the regulatory concern is per-type — a 96% recall on `VN_CCCD` means 4% of CCCDs leak, regardless of how good the other recognizers are.

#### Suggested fix
Tighten the test to compute and assert per-type recall:

```python
from collections import defaultdict

def test_recall_at_least_99_percent_per_type(analyzer):
    """AC #13 + per-type guard: each entity type MUST individually hit ≥99% recall."""
    fixture_path = Path(__file__).parent / "fixtures" / "vn_pii_200_samples.yaml"
    samples = yaml.safe_load(fixture_path.read_text())

    correct_by_type = defaultdict(int)
    total_by_type = defaultdict(int)

    for sample in samples["samples"]:
        text = sample["text"]
        for expected in sample["expected_entities"]:
            total_by_type[expected] += 1
            results = analyzer.analyze(text=text, language="vi", entities=[expected])
            if any(r.entity_type == expected for r in results):
                correct_by_type[expected] += 1

    failures = []
    for entity_type, total in total_by_type.items():
        recall = correct_by_type[entity_type] / total
        if recall < 0.99:
            failures.append(f"{entity_type}: recall={recall:.4f} ({correct_by_type[entity_type]}/{total})")

    assert not failures, "Per-type recall floor violated:\n" + "\n".join(failures)
```

Update §1 #1 to say: *"each recognizer individually MUST hit recall ≥ 99% on its samples in the 200-sample fixture; the aggregate across all types MUST also be ≥ 99%."*

### ISS-005 — AUTHORING.md §3.6 rule 20 (redacted forms in audit rows) — VN_MST/VN_CCCD audit-row representation not specified
- **severity:** warning
- **rule_id:** authoring-md-§3.6 (rule 20)
- **location:** §1 (no clause about audit-row redacted form), §3 (recognizers), §8 (example payloads)
- **status:** open

#### Description
AUTHORING.md §3.6 rule 20 says "Audit rows MUST carry redacted forms when the field is PII (e.g. `mst_redacted: \"03******78\"`); never the full value." When the redaction pipeline runs and a `VN_MST` is found, FR-AI-003 emits a BRAIN audit row. That row's `extra` map should carry `mst_redacted: "03******78"` (first 2 + last 2 of the MST), NOT the full MST string. The §3 recognizers produce placeholders like `<VN_MST_1>` but the audit row needs the redacted-but-recognizable form so operators can pattern-match without seeing the full PII. The spec doesn't currently say which redacted form. A spec violation risk: a future PR could write `extra.mst: "0312345678"` to the BRAIN row.

#### Suggested fix
Add §1 #17: "**MUST** produce per-type redacted display forms for audit-row emission per AUTHORING.md §3.6 rule 20: `VN_MST` → `<first-2>******<last-2>` (e.g. `03******78` for 0312345678); `VN_CCCD` → `<first-3>******<last-3>` (e.g. `031******678`); `VN_PHONE` → `<first-2>***<last-4>` (e.g. `09***1234`); `VN_BANK_ACCOUNT` → `***<last-4>` (e.g. `***6789`). The helper module `cyberos_pii::vn::redact_for_audit::<T>(value: &str) -> String` provides these per-type formatters; the FR-AI-003 emit path MUST call them when serialising `extra.{mst,cccd,phone,bank_account}_redacted` fields. AC #17 verifies via a round-trip test that no audit row written during a VN-PII detection contains any digit-sequence longer than 4 consecutive digits of the original."

### ISS-006 — AUTHORING.md §3.6 rule 21 (tenant-scoped PII allowlist) — no clause about per-tenant allowlist
- **severity:** warning
- **rule_id:** authoring-md-§3.6 (rule 21)
- **location:** §1 (no clause about allowlist), §3 (recognizer pipeline), §11 (notes)
- **status:** open

#### Description
AUTHORING.md §3.6 rule 21 says "Tenant-scoped PII allowlists exist (`pii_allowlist: [\"regex\", ...]` in `manifest.tenants[].pii_allowlist`); use them for legitimate-exception fields like KYC vendor MSTs." Some tenants (e.g., a KYC vendor) legitimately need to pass MST strings through to their LLM — for them, the MST is the subject matter, not PII. The spec has no clause about reading `policy.pii_allowlist` and skipping redaction for matching strings. Without this, KYC use cases are blocked — every MST gets `<VN_MST_N>` placeholdered and the LLM can't reason about MST validation flow. The recognizers should consult the per-tenant allowlist BEFORE emitting `RecognizerResult`s.

#### Suggested fix
Add §1 #18: "**MUST** consult `policy.ai_policy.pii_allowlist: Vec<String>` (compiled to `Vec<Regex>` at policy-load time per FR-AI-005) per AUTHORING.md §3.6 rule 21. Before emitting any `RecognizerResult` for an entity type ∈ `VN_MST | VN_CCCD | VN_PHONE | VN_BANK_ACCOUNT`, the recognizer pipeline MUST check whether the matched text matches ANY allowlist regex for the active tenant. If yes, suppress the recognizer result (no redaction; PII flows through to the LLM). The audit-row `extra` field `pii_allowlist_hit_count: u32` records how many suppressions happened per call so operators can audit allowlist usage. AC #18 verifies via a test: with `pii_allowlist: [\"^03\\\\d{8}$\"]`, an MST starting with `03` is NOT redacted; an MST starting with `04` IS redacted (per-regex match)."

## §3 — Strengths preserved through expansion

- §3 introduces `province_codes.py` and `confidence.py` as shared single-source-of-truth modules; recognizers import from them — preventing the drift class of bugs called out in FR-AI-006 ISS-001.
- §3's `VnNddRecognizer` uses `EntityRecognizer` (not `PatternRecognizer`) because it needs label-bound capture-group extraction. Different base class is well-documented in the rationale.
- §1 #7 explicitly specifies registration order (`VN_PHONE` first, `VN_BANK_ACCOUNT` last) AND §5 has a test asserting it — preventing the "ordered list reordered by accident" regression.
- §1 #14 version endpoint + §5 (after ISS-001 fix) version test gives FR-AI-013's recall gate a way to detect drift between deployed sidecar and fixture file.
- §10 inventory covers 16 distinct paths including the `pii_redaction_extra` policy-field-missing row (a real customer-onboarding failure mode).
- §11 explicitly documents the quarterly fixture-regen process and the `confidence.py` calibration cadence — operationally precise.

## §4 — Resolution

All 6 mechanical revisions applied:
- ISS-001 RESOLVED (2026-05-16): §5 `test_version_endpoint_returns_six_entries` added asserting response shape + semver; AC #16 covered.
- ISS-002 RESOLVED (2026-05-16): §5 `test_no_network_imports.py` CI lint AST-parses every recognizer file rejecting forbidden network imports; §10 row added.
- ISS-003 RESOLVED (2026-05-16): §3 `register_vn_recognizers` uses `_REGISTERED` global guard with WARN-on-double-call; `reset_for_tests()` provides test isolation; §10 row added.
- ISS-004 RESOLVED (2026-05-16): §5 aggregate recall test replaced with `test_recall_at_least_99_percent_per_type`; §1 #1 tightened to per-type AND aggregate floors.
- ISS-005 RESOLVED (2026-05-16, AUTHORING.md compliance pass): §1 #17 added with per-type redacted-display formatters (mst_redacted, cccd_redacted, phone_redacted, bank_account_redacted); helper `cyberos_pii::vn::redact_for_audit::<T>`; AC #17 added.
- ISS-006 RESOLVED (2026-05-16, AUTHORING.md compliance pass): §1 #18 added consulting `policy.ai_policy.pii_allowlist`; suppression before recognizer-result emit; `pii_allowlist_hit_count` audit field; AC #18 added.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of FR-AI-012 audit (final). Status: PASS at 10/10.*
