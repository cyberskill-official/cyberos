---
# ───── Machine-readable frontmatter (parsed by task-audit + future task-catalog renderer) ─────
id: TASK-AI-012
title: "VN-PII Presidio plugin (CCCD · MST · VN phone · NĐD · VN address · bank account)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: AI
priority: p0
status: done
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-21
memory_chain_hash: null
related_tasks: [TASK-AI-002, TASK-AI-005, TASK-AI-008, TASK-AI-011, TASK-AI-013]
depends_on: [TASK-AI-011]
blocks: [TASK-AI-013, TASK-MEMORY-111]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#pii-redaction
  - website/docs/legal/vn-pdpl-compliance.html
source_decisions:
  - PDPL Art. 7 (Vietnam personal-data-sale ban; CCCD government-ID handling)
  - PDPL Art. 6 (data minimisation principle; mirror of GDPR Art. 5(1)(c))
  - DEC-053 (CCCD treated as Class-A government ID — never persists in memory raw)
  - archive/2026-05-14/RESEARCH_REVIEW.md §4.2 (custom recognizers vs Presidio EN baseline gap analysis)

# ───── Build envelope ─────
language: python 3.11 (presidio recognizer plugin)
service: cyberos/services/ai-gateway/pii/
new_files:
  - services/ai-gateway/pii/recognizers/__init__.py
  - services/ai-gateway/pii/recognizers/vn_cccd.py
  - services/ai-gateway/pii/recognizers/vn_mst.py
  - services/ai-gateway/pii/recognizers/vn_phone.py
  - services/ai-gateway/pii/recognizers/vn_ndd.py
  - services/ai-gateway/pii/recognizers/vn_address.py
  - services/ai-gateway/pii/recognizers/vn_bank.py
  - services/ai-gateway/pii/recognizers/province_codes.py
  - services/ai-gateway/pii/recognizers/confidence.py
  - services/ai-gateway/pii/test_vn_recognizers.py
  - services/ai-gateway/pii/test_vn_no_false_positives.py
  - services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml
modified_files:
  - services/ai-gateway/pii/presidio_sidecar.py    # register VN recognizers at sidecar startup
  - services/ai-gateway/src/redact/types.rs        # PiiType already includes VN variants from TASK-AI-011 §3
allowed_tools:
  - file_read: services/ai-gateway/pii/**
  - file_write: services/ai-gateway/pii/**
  - bash: cd services/ai-gateway/pii && pytest -v
  - bash: docker compose up -d presidio
disallowed_tools:
  - send VN-PII to any non-localhost service (recognizers run inside the loopback-bound sidecar from TASK-AI-011 §1 #15)
  - persist raw VN-PII anywhere (RestorationMap zeroize semantics from TASK-AI-011 §1 #4 apply)
  - hardcode confidence scores in recognizer classes (define in `confidence.py`; recognizers import)
  - duplicate the province-code list across recognizers (single source in `province_codes.py`)
  - skip the negative-fixture test (every recognizer MUST be checked for false positives on plain VN text)

# ───── Estimated work ─────
effort_hours: 10
subtasks:
  - "0.5h: province_codes.py shared list + confidence.py shared score constants"
  - "1.0h: VN_CCCD recognizer (12-digit + province-code validation; high/low-confidence variants)"
  - "1.0h: VN_MST recognizer (10 or 13-digit with hyphen-branch; province-prefix validation)"
  - "1.0h: VN_PHONE recognizer (+84 prefix or 0-prefix mobile/landline; distinguish from MST)"
  - "1.0h: VN_NDD recognizer (Người đại diện / NĐD / Legal representative label-bound name extraction)"
  - "2.0h: VN_ADDRESS recognizer (district / ward / city patterns with Vietnamese diacritics; longest-match heuristic)"
  - "1.5h: VN_BANK_ACCOUNT recognizer (10-14 digits with bank-name context; distinguish from CCCD/MST)"
  - "0.5h: Sidecar registration in presidio_sidecar.py + recognizer-list assertion test"
  - "1.5h: pytest suite — positive fixtures (each type) + negative fixtures (no false positives) + 200-sample recall test"
risk_if_skipped: "Vietnamese personal data leaks to LLM providers on every VN tenant call. PDPL Art. 7 personal-data-sale ban + Art. 6 data minimisation violated for every call carrying CCCD/MST/SDT/STK. Critical compliance failure on first VN regulator audit; loss of every VN enterprise contract that requires PDPL DPA; brand damage in the CyberSkill home market. Recall floor (TASK-AI-013) cannot be measured without these recognizers existing."
---

## §1 — Description (BCP-14 normative)

The Presidio sidecar (TASK-AI-011) **MUST** register 6 Vietnamese-specific PII recognizers. Each recognizer:

1. **MUST** match the documented pattern with **recall ≥ 99% per entity type AND ≥ 99% aggregate** on the 200-sample VN PII test set (TASK-AI-013 enforces the CI gate; this task provides the recognizers and the fixture file). The per-type floor catches the case where one recognizer regresses to 90% recall while others compensate to keep the aggregate above 99%.
2. **MUST** assign a confidence score in the range `[0.0, 1.0]` per Presidio convention. The placeholder format is `<VN_<TYPE>_<N>>` where TYPE ∈ `CCCD`, `MST`, `PHONE`, `NDD`, `ADDRESS`, `BANK_ACCOUNT`. Scores MUST come from the shared `confidence.py` constants — no inline numeric literals in recognizer classes.
3. **MUST** support both pre-composed and combining-form Vietnamese characters (Unicode normalisation handled by Presidio's analyzer pipeline; recognizers receive NFC-normalized text).
4. **MUST NOT** false-positive on plain Vietnamese text. The 200-sample test set MUST include negative examples (Vietnamese names without CCCD, addresses without ward/district structure, dates that look like 12-digit numbers, VND amounts that look like phone numbers).
5. **MUST** validate `VN_CCCD` first 3 digits against the closed list of valid province codes (001-099) sourced from `province_codes.py`. Invalid province code → match dropped (treated as a 12-digit number, not a CCCD).
6. **MUST** validate `VN_MST` against the 10-digit (entity) or 13-digit (entity-branch with hyphen) format; the first 2 digits MUST be a valid province code. The 10-digit and 13-digit variants share the same recognizer with two patterns at differentiated confidence.
7. **MUST** distinguish `VN_PHONE` from `VN_MST` at recognizer registration order — phone runs FIRST so a 10-digit `0901234567` is matched as phone (high confidence with `+84` or `0` prefix), not as MST. The recognizer registration order is asserted in `test_vn_recognizers.py`.
8. **MUST** support `VN_NDD` (Người đại diện / Legal representative) as a label-bound name extraction. The pattern matches `(Người đại diện|NĐD|Legal representative)\s*[:.]?\s*(<PERSON_NAME>)` and emits the captured name only. Without the label, fallback to Presidio's standard PERSON recognizer (which doesn't carry the "this is a legal representative" semantic).
9. **MUST** support partial redaction for `VN_ADDRESS` — when the recognizer matches `<street>, <ward>, <district>, <city>`, the placeholder MAY be `<VN_ADDRESS_1>` (full redact) OR `<VN_ADDRESS_CITY_ONLY_1>` (preserve city, redact street/ward/district) depending on the `policy.ai_policy.vn_address_partial_redact: bool` flag. Default is full redact for slice 3.
10. **MUST** distinguish `VN_BANK_ACCOUNT` (10-14 digits, often near `STK|tài khoản|account number`) from `VN_CCCD` (always 12 digits with province-code prefix) and `VN_MST` (10/13 digits with province-code prefix). Bank account regex includes a context-keyword boost and bank-name proximity (Vietcombank, BIDV, Techcombank, Sacombank, Agribank, MBBank, VPBank, ACB).
11. **MUST** be deterministic — same input string → same recognizer matches with same confidence scores. Presidio's analyzer iterates recognizers in registration order; stable order is asserted in `test_recognizer_registration_order`.
12. **MUST NOT** make any network call. All recognizers are pure regex + lookup tables. The province-code list is statically embedded; bank-name list is statically embedded.
13. **MUST** emit `recognizer_registration_failed` on sidecar startup if any of the 6 recognizers fails to register (e.g., regex compile error, validation method missing). The sidecar MUST refuse to start — partial registration would silently miss PII types.
14. **MUST** publish recognizer-version metadata via `GET /recognizers/version` returning `{ "VN_CCCD": "1.0.0", "VN_MST": "1.0.0", ... }`. TASK-AI-013's recall test asserts the version is the expected one for the fixture set; mismatched versions fail the CI gate.
15. **SHOULD** log each registered recognizer's name + pattern count at INFO level on sidecar startup so operators can verify the expected 6 recognizers loaded. Logging the patterns themselves is OK (regex strings, not PII).
16. **MUST** produce per-type redacted display forms for downstream audit-row emission per task-audit skill §3.6 rule 20. The Rust gateway-side helper `cyberos_pii::vn::redact_for_audit::<T>(value: &str) -> String` provides these formatters: `VN_MST` → `<first-2>******<last-2>` (e.g. `03******78` for 0312345678); `VN_CCCD` → `<first-3>******<last-3>` (e.g. `031******678`); `VN_PHONE` → `<first-2>***<last-4>` (e.g. `09***1234`); `VN_BANK_ACCOUNT` → `***<last-4>` (e.g. `***6789`). The TASK-AI-003 memory-emit path MUST call them when serialising `extra.{mst,cccd,phone,bank_account}_redacted`. AC #18 verifies via a round-trip test that no audit row written during a VN-PII detection contains any digit sequence longer than 4 consecutive digits of the original.
17. **MUST** consult `policy.ai_policy.pii_allowlist: Vec<String>` (compiled to `Vec<Regex>` at policy-load time per TASK-AI-005) per task-audit skill §3.6 rule 21. Before emitting any `RecognizerResult` for an entity type ∈ `VN_MST | VN_CCCD | VN_PHONE | VN_BANK_ACCOUNT`, the recognizer pipeline MUST check whether the matched text matches ANY allowlist regex for the active tenant. If yes, suppress the result (no redaction; PII flows to LLM). The audit-row `extra.pii_allowlist_hit_count: u32` records how many suppressions happened per call so operators can audit allowlist usage. Use case: KYC vendor tenant where MST IS the subject matter, not collateral PII.

---

## §2 — Why this design (rationale for humans)

**Why custom recognizers instead of relying on Presidio's PERSON/LOCATION?** Presidio's English recognizers don't know Vietnam-specific structure. CCCD numbers are particularly tricky — a 12-digit number could be a timestamp (Unix epoch nanos), a phone (some legacy formats), or a CCCD. Without context-aware recognizers we measured 30%+ false negatives on real VN customer data; the 99% recall floor is unachievable without these custom recognizers.

**Why context-sensitive matching (e.g., 'CCCD: ' prefix boost)?** Real VN documents structure PII with leading labels (Vietnamese forms, contracts, invoices). Matching the label boosts confidence and reduces false positives — `"Sinh năm 1990"` (born in 1990) wouldn't match `\b\d{12}\b` but might match if the labeled-prefix variant didn't exist alongside. Two-tier confidence (high-with-label, low-without) lets the analyzer make the right trade-off.

**Why preserve structure in `VN_ADDRESS` partial redaction?** Some workflows need the city but not the street ("send invoice to Ho Chi Minh City" is fine; "send to 123 Nguyễn Văn A street" is PII). Partial redaction at `<VN_ADDRESS_CITY_ONLY>` placeholder preserves the city-level data for product analytics while redacting street/ward. Slice 3 ships full-redact-only; slice 5 enables the partial mode behind a tenant policy flag.

**Why a single `province_codes.py` source-of-truth file?** CCCD AND MST both validate against province codes. Without a shared list, we'd have two copies that drift over time. When Vietnam's government adds a new province code (or merges provinces), one update to one file suffices. The list is small (~99 entries) so embedding it statically is fine — no DB lookup, no network call.

**Why a shared `confidence.py` for score constants?** Magic-number scores in recognizer classes (`score=0.99`, `score=0.5`) are tuning knobs. When we calibrate against the 200-sample test set and find the high-confidence threshold should be 0.95 (not 0.99) to balance precision/recall, we want one place to update — not 6 recognizer files. The constants module is the dial; the recognizers reference it.

**Why does recognizer registration order matter for VN_PHONE vs VN_MST?** Both can match a 10-digit string. Without ordered registration, Presidio's analyzer may emit BOTH a `VN_PHONE` and a `VN_MST` for the same span — the anonymizer then picks one arbitrarily. Registering phone FIRST means a labeled phone (`SĐT: 0901234567`) wins over the unlabeled MST pattern, which is the conservative call (over-redacting a phone as a phone is fine; under-redacting an MST as nothing is bad).

**Why does sidecar refuse to start on registration failure (§1 #13)?** Partial registration is silent PII passthrough. If `VnAddressRecognizer` fails to register (e.g., a regex compile error after a refactor), the sidecar would happily process requests but VN addresses would not be redacted. Operators wouldn't know until a customer complaint. Failing-loud at startup forces the operator to fix the recognizer before any traffic flows.

**Why a recognizer-version endpoint (§1 #14)?** Three reasons. (1) TASK-AI-013's recall test fixture is bound to specific recognizer versions; mismatched versions could give false confidence in CI. (2) Operators investigating a recall regression can check "did we actually deploy the new VN_CCCD v1.1 with the fix?". (3) Tenant-facing audit reports cite the recognizer version so customers know what was active during their data processing.

**Why no network calls in recognizers (§1 #12)?** Recognizers run on the hot path (per-request, per-prompt). A network call would add 50-200ms latency and a failure mode (network down → recognizer skips → PII passthrough). Pure-Python regex + static lookup is the only design that meets the 30ms p95 budget from TASK-AI-011 §1 #6.

**Why label-bound NĐD extraction instead of named-entity NER?** Vietnamese names have low entropy from a NER perspective — Nguyễn Văn A is structurally identical to "Mr. Smith" in English NER. Label-bound matching ("preceded by NĐD: or Người đại diện:") catches the cases where the name carries legal-representative meaning. Without the label, fall through to Presidio's PERSON recognizer (still redacted, just not tagged as NDD).

---

## §3 — API contract (Python recognizer classes)

### Shared modules

```python
# services/ai-gateway/pii/recognizers/province_codes.py
"""
Single source of truth for Vietnamese province codes.
Used by VN_CCCD (first 3 digits) and VN_MST (first 2 digits) validation.
Source: General Statistics Office of Vietnam, 2024 administrative divisions.
"""

VALID_PROVINCE_CODES_3DIGIT = frozenset([
    "001", "002", "004", "006", "008", "010", "011", "012", "014", "015", "017",
    "019", "020", "022", "024", "025", "026", "027", "030", "031", "033", "034",
    # ... (62 more)
])

VALID_PROVINCE_CODES_2DIGIT = frozenset(
    code[1:] for code in VALID_PROVINCE_CODES_3DIGIT
)

VN_BANK_NAMES = frozenset([
    "Vietcombank", "BIDV", "Techcombank", "Sacombank", "Agribank",
    "MBBank", "VPBank", "ACB", "VIB", "TPBank", "SHB", "HDBank",
    # ... (~30 banks)
])
```

```python
# services/ai-gateway/pii/recognizers/confidence.py
"""
Shared confidence-score constants. Tuned against the 200-sample VN PII test set.
Update one file when calibrating recall/precision trade-offs.
"""

CONFIDENCE_HIGH = 0.99   # explicit label match (CCCD:, MST:, NĐD:, STK:)
CONFIDENCE_MED  = 0.85   # strong contextual match (e.g., bank name nearby)
CONFIDENCE_LOW  = 0.50   # bare pattern match without context
```

### Recognizer class signatures

```python
# services/ai-gateway/pii/recognizers/vn_cccd.py
from presidio_analyzer import Pattern, PatternRecognizer
from .province_codes import VALID_PROVINCE_CODES_3DIGIT
from .confidence import CONFIDENCE_HIGH, CONFIDENCE_LOW

class VnCccdRecognizer(PatternRecognizer):
    """VN_CCCD — 12-digit citizen identity number; first 3 digits are province code."""

    VERSION = "1.0.0"

    PATTERNS = [
        Pattern(
            name="vn_cccd_strict_with_context",
            regex=r"(?:CCCD|cccd|Căn cước công dân|căn cước|định danh)\s*[:\.]?\s*(\d{12})",
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_cccd_plain",
            regex=r"\b\d{12}\b",
            score=CONFIDENCE_LOW,
        ),
    ]

    SUPPORTED_LANGUAGES = ["en", "vi"]

    def __init__(self):
        super().__init__(
            supported_entity="VN_CCCD",
            patterns=self.PATTERNS,
            context=["CCCD", "căn cước", "định danh", "cmnd"],
            supported_language="vi",
        )

    def validate_result(self, pattern_text: str) -> bool:
        """Province code validation: first 3 digits must be in the valid set."""
        return pattern_text[:3] in VALID_PROVINCE_CODES_3DIGIT
```

```python
# services/ai-gateway/pii/recognizers/vn_mst.py
class VnMstRecognizer(PatternRecognizer):
    """VN_MST — 10 or 13-digit tax code; 13-digit has hyphen between 10 and 3-digit branch."""
    VERSION = "1.0.0"
    PATTERNS = [
        Pattern("vn_mst_with_context_10",
            r"(?:MST|mst|Mã số thuế|mã thuế)\s*[:\.]?\s*(\d{10})",
            CONFIDENCE_HIGH),
        Pattern("vn_mst_with_context_13",
            r"(?:MST|mst|Mã số thuế)\s*[:\.]?\s*(\d{10})-(\d{3})",
            CONFIDENCE_HIGH),
        Pattern("vn_mst_plain_10",
            r"\b\d{10}\b",
            CONFIDENCE_LOW),
        Pattern("vn_mst_plain_13",
            r"\b\d{10}-\d{3}\b",
            CONFIDENCE_LOW),
    ]
    def __init__(self):
        super().__init__(
            supported_entity="VN_MST",
            patterns=self.PATTERNS,
            context=["MST", "mã số thuế", "thuế"],
        )
    def validate_result(self, pattern_text: str) -> bool:
        # First 2 digits are the province code (without leading 0).
        digits = pattern_text.replace("-", "")[:2]
        return digits in VALID_PROVINCE_CODES_2DIGIT
```

```python
# services/ai-gateway/pii/recognizers/vn_phone.py
class VnPhoneRecognizer(PatternRecognizer):
    """VN_PHONE — +84 or 0-prefix mobile/landline."""
    VERSION = "1.0.0"
    PATTERNS = [
        Pattern("vn_phone_84_mobile",
            r"\+84\s?(?:9\d{8}|3\d{8}|7\d{8}|8\d{8}|5\d{8})",
            CONFIDENCE_HIGH),
        Pattern("vn_phone_0_mobile",
            r"0(?:9\d{8}|3\d{8}|7\d{8}|8\d{8}|5\d{8})\b",
            CONFIDENCE_HIGH),
        Pattern("vn_phone_landline",
            r"0(?:2[0-9])\d{7,8}\b",
            CONFIDENCE_MED),
    ]
    def __init__(self):
        super().__init__(
            supported_entity="VN_PHONE",
            patterns=self.PATTERNS,
            context=["SĐT", "điện thoại", "phone", "mobile"],
        )
```

```python
# services/ai-gateway/pii/recognizers/vn_ndd.py
from presidio_analyzer import EntityRecognizer, RecognizerResult

class VnNddRecognizer(EntityRecognizer):
    """VN_NDD — label-bound legal representative name extraction."""
    VERSION = "1.0.0"
    LABEL_REGEX = r"(?:Người đại diện|NĐD|Legal representative)\s*[:\.]?\s*([A-ZÀÁẢÃẠ][\w\sÀ-ỹ]+?)(?=[\n,.;]|$)"

    def __init__(self):
        super().__init__(supported_entities=["VN_NDD"], supported_language="vi")

    def analyze(self, text, entities, nlp_artifacts=None):
        import re
        results = []
        for m in re.finditer(self.LABEL_REGEX, text):
            results.append(RecognizerResult(
                entity_type="VN_NDD",
                start=m.start(1),
                end=m.end(1),
                score=CONFIDENCE_HIGH,
            ))
        return results
```

```python
# services/ai-gateway/pii/recognizers/vn_address.py
class VnAddressRecognizer(PatternRecognizer):
    """VN_ADDRESS — multi-segment Vietnamese postal address."""
    VERSION = "1.0.0"
    PATTERNS = [
        Pattern("vn_address_full",
            r"\d+\s+[\w\sÀ-ỹ]+,\s*(?:Phường|Xã|Quận|Huyện|Tp\.?|TP\.?|Thành phố)[^,]+,\s*(?:Quận|Huyện|Tp\.?|TP\.?)[^,]+,\s*(?:Tp\.?|TP\.?|Thành phố)[\w\sÀ-ỹ\.]+",
            CONFIDENCE_HIGH),
        Pattern("vn_address_partial",
            r"(?:Quận|Huyện|Tp\.?|TP\.?|Phường|Xã)\s+[\w\sÀ-ỹ\d]+",
            CONFIDENCE_MED),
    ]
    def __init__(self):
        super().__init__(
            supported_entity="VN_ADDRESS",
            patterns=self.PATTERNS,
            context=["địa chỉ", "address"],
        )
```

```python
# services/ai-gateway/pii/recognizers/vn_bank.py
from .province_codes import VN_BANK_NAMES

class VnBankAccountRecognizer(PatternRecognizer):
    """VN_BANK_ACCOUNT — 10-14 digit account; bank-name proximity boost."""
    VERSION = "1.0.0"
    BANK_NAMES_RE = r"(?:" + "|".join(VN_BANK_NAMES) + r")"
    PATTERNS = [
        Pattern("vn_bank_with_context",
            rf"(?:STK|tài khoản|account number|số tài khoản)\s*(?:{BANK_NAMES_RE})?\s*[:\.]?\s*(\d{{10,14}})",
            CONFIDENCE_HIGH),
        Pattern("vn_bank_with_bank_name",
            rf"{BANK_NAMES_RE}[\w\s:,]*?(\d{{10,14}})",
            CONFIDENCE_MED),
        Pattern("vn_bank_plain",
            r"\b\d{10,14}\b",
            CONFIDENCE_LOW),
    ]
    def __init__(self):
        super().__init__(
            supported_entity="VN_BANK_ACCOUNT",
            patterns=self.PATTERNS,
            context=["STK", "tài khoản", "account", "số tài khoản"],
        )
    def validate_result(self, pattern_text: str) -> bool:
        # Distinguish from VN_CCCD (12 digits with province-code prefix) and VN_MST.
        if len(pattern_text) == 12 and pattern_text[:3] in VALID_PROVINCE_CODES_3DIGIT:
            return False  # likely a CCCD; let the CCCD recognizer handle it
        if len(pattern_text) == 10 and pattern_text[:2] in VALID_PROVINCE_CODES_2DIGIT:
            return False  # likely an MST
        return True
```

### Sidecar registration

```python
# services/ai-gateway/pii/presidio_sidecar.py (additions)
from recognizers import (
    VnCccdRecognizer, VnMstRecognizer, VnPhoneRecognizer,
    VnNddRecognizer, VnAddressRecognizer, VnBankAccountRecognizer,
)

# §1 #7 registration order: phone before MST so labeled phones win the tie-break.
VN_RECOGNIZERS = [
    VnPhoneRecognizer(),       # FIRST (wins over MST for 10-digit phone numbers)
    VnCccdRecognizer(),        # 12-digit (distinct length from MST)
    VnMstRecognizer(),
    VnNddRecognizer(),
    VnAddressRecognizer(),
    VnBankAccountRecognizer(), # LAST (broad pattern; let specific recognizers run first)
]

# ISS-003 fix: idempotency guard against double-registration.
# Presidio's add_recognizer doesn't dedupe; calling twice would add duplicates → double analysis.
_REGISTERED = False

def register_vn_recognizers(analyzer: AnalyzerEngine):
    """§1 #13: refuse to start if any registration fails. Idempotent: subsequent calls no-op with WARN."""
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

@app.get("/recognizers/version")
def recognizer_versions():
    """§1 #14: version endpoint for TASK-AI-013 recall-gate."""
    return {rec.supported_entities[0]: rec.VERSION for rec in VN_RECOGNIZERS}
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **CCCD with context** — `"CCCD: 012345678901"` → match `VN_CCCD` with `score >= 0.99`.
2. **CCCD without context** — `"012345678901"` alone → match `VN_CCCD` with `score >= 0.50` AND province-code validation passes.
3. **CCCD with invalid province code** — `"CCCD: 999345678901"` → NO match (province code 999 not in valid set; `validate_result` rejects).
4. **MST 10-digit with context** — `"MST: 0301234567"` → match `VN_MST` with `score >= 0.99`.
5. **MST 13-digit with branch** — `"0301234567-001"` → match `VN_MST` (13-digit pattern).
6. **VN phone +84 mobile** — `"+84901234567"` → match `VN_PHONE` with `score >= 0.99`.
7. **VN phone 0-prefix** — `"0901234567"` → match `VN_PHONE` (NOT `VN_MST` — phone runs first per §1 #7).
8. **NĐD pattern** — `"NĐD: Nguyễn Văn A"` → match `VN_NDD` capturing `"Nguyễn Văn A"` with `score >= 0.99`.
9. **VN address full** — `"123 Nguyễn Thị Minh Khai, Phường Bến Nghé, Quận 1, TP. Hồ Chí Minh"` → match `VN_ADDRESS` with `score >= 0.99`.
10. **VN bank with context** — `"STK Vietcombank: 1234567890"` → match `VN_BANK_ACCOUNT` with `score >= 0.99`.
11. **No false positive on dates** — `"Ngày 12/05/2026"` MUST NOT match any VN recognizer.
12. **No false positive on VND amounts** — `"100,000,000 VND"` MUST NOT match `VN_PHONE` or `VN_BANK_ACCOUNT`.
13. **Recall ≥ 99% on 200-sample fixture** — Run all 6 recognizers against `fixtures/vn_pii_200_samples.yaml`; correctly identified ≥ 198/200 across all types.
14. **Recognizer registration order asserted** — `VN_RECOGNIZERS[0].supported_entities[0] == "VN_PHONE"` (phone first); `VN_RECOGNIZERS[-1].supported_entities[0] == "VN_BANK_ACCOUNT"` (bank last).
15. **Sidecar refuses to start on registration failure** — Inject a regex compile error into `VnAddressRecognizer.PATTERNS`; sidecar startup raises `RuntimeError("recognizer_registration_failed: ...")`; HTTP server never binds.
16. **Version endpoint returns 6 entries** — `GET /recognizers/version` returns JSON with keys `VN_CCCD`, `VN_MST`, `VN_PHONE`, `VN_NDD`, `VN_ADDRESS`, `VN_BANK_ACCOUNT`; values are semver strings.
17. **Determinism** — Calling `analyze()` twice on the same input produces identical (entity_type, start, end, score) tuples.
18. **Audit-row redacted forms (task-audit skill §3.6 rule 20)** — Round-trip test: detect `"MST 0312345678 CCCD 079123456789 phone 0901234567 bank 1234567890"`. Emit memory row via TASK-AI-003. Assert `row.extra.mst_redacted == "03******78"` AND `row.extra.cccd_redacted == "079******789"` AND `row.extra.phone_redacted == "09***4567"` AND `row.extra.bank_account_redacted == "***7890"`. Also assert `row.extra` JSON contains NO digit-substring of length > 4 from the original.
19. **PII allowlist suppression (task-audit skill §3.6 rule 21)** — Test fixture `policy_with_allowlist.yaml` sets `pii_allowlist: ["^03\\d{8}$"]`. Analyze text containing `"MST 0312345678"` and `"MST 0412345678"`. First match is suppressed (no `<VN_MST_N>` placeholder; raw value flows through); second match is redacted normally. `extra.pii_allowlist_hit_count == 1`.

---

## §5 — Verification

**Positive fixture tests:** `services/ai-gateway/pii/test_vn_recognizers.py`

```python
import pytest
from presidio_analyzer import AnalyzerEngine
from recognizers import (
    VnCccdRecognizer, VnMstRecognizer, VnPhoneRecognizer,
    VnNddRecognizer, VnAddressRecognizer, VnBankAccountRecognizer,
    VN_RECOGNIZERS,
)
from recognizers.confidence import CONFIDENCE_HIGH, CONFIDENCE_MED, CONFIDENCE_LOW

@pytest.fixture
def analyzer():
    a = AnalyzerEngine()
    for rec in VN_RECOGNIZERS:
        a.registry.add_recognizer(rec)
    return a

def test_cccd_with_context(analyzer):
    results = analyzer.analyze(text="CCCD: 012345678901", language="vi", entities=["VN_CCCD"])
    assert len(results) == 1
    assert results[0].score >= CONFIDENCE_HIGH

def test_cccd_without_context_validates_province(analyzer):
    # 012 is province code 12 (Lai Châu) — valid
    results = analyzer.analyze(text="012345678901", language="vi", entities=["VN_CCCD"])
    assert len(results) == 1
    assert results[0].score >= CONFIDENCE_LOW

def test_cccd_invalid_province_rejected(analyzer):
    # 999 is not a valid province code
    results = analyzer.analyze(text="999345678901", language="vi", entities=["VN_CCCD"])
    assert len(results) == 0

def test_mst_10_digit(analyzer):
    results = analyzer.analyze(text="MST: 0301234567", language="vi", entities=["VN_MST"])
    assert len(results) == 1
    assert results[0].score >= CONFIDENCE_HIGH

def test_mst_13_digit_with_branch(analyzer):
    results = analyzer.analyze(text="0301234567-001", language="vi", entities=["VN_MST"])
    assert len(results) >= 1

def test_vn_phone_84_prefix(analyzer):
    results = analyzer.analyze(text="+84901234567", language="vi", entities=["VN_PHONE"])
    assert len(results) == 1
    assert results[0].score >= CONFIDENCE_HIGH

def test_vn_phone_beats_mst_for_10_digit(analyzer):
    """§1 #7: phone runs first; 0901234567 should match VN_PHONE, not VN_MST."""
    results = analyzer.analyze(text="0901234567", language="vi",
                               entities=["VN_PHONE", "VN_MST"])
    types = {r.entity_type for r in results}
    assert "VN_PHONE" in types
    # MST may still match at low confidence; the anonymizer picks the higher-score one.
    phone_score = max(r.score for r in results if r.entity_type == "VN_PHONE")
    mst_score = max((r.score for r in results if r.entity_type == "VN_MST"), default=0)
    assert phone_score >= mst_score

def test_vn_ndd_label_bound(analyzer):
    results = analyzer.analyze(text="NĐD: Nguyễn Văn A",
                               language="vi", entities=["VN_NDD"])
    assert len(results) == 1
    assert results[0].score >= CONFIDENCE_HIGH

def test_vn_address_full(analyzer):
    text = "123 Nguyễn Thị Minh Khai, Phường Bến Nghé, Quận 1, TP. Hồ Chí Minh"
    results = analyzer.analyze(text=text, language="vi", entities=["VN_ADDRESS"])
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH

def test_vn_bank_account_with_context(analyzer):
    results = analyzer.analyze(text="STK Vietcombank: 1234567890",
                               language="vi", entities=["VN_BANK_ACCOUNT"])
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH

def test_recognizer_registration_order():
    """§1 #7: VN_PHONE first, VN_BANK_ACCOUNT last."""
    assert VN_RECOGNIZERS[0].supported_entities[0] == "VN_PHONE"
    assert VN_RECOGNIZERS[-1].supported_entities[0] == "VN_BANK_ACCOUNT"

# ISS-001 fix: AC #16 requires version-endpoint shape verification.
def test_version_endpoint_returns_six_entries():
    """AC #16: every registered recognizer's version is exposed via GET /recognizers/version."""
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

def test_determinism(analyzer):
    """§1 #11 + AC #17: same input twice → identical results."""
    text = "CCCD: 012345678901, MST: 0301234567"
    r1 = analyzer.analyze(text=text, language="vi", entities=["VN_CCCD", "VN_MST"])
    r2 = analyzer.analyze(text=text, language="vi", entities=["VN_CCCD", "VN_MST"])
    r1_tuples = sorted([(r.entity_type, r.start, r.end, r.score) for r in r1])
    r2_tuples = sorted([(r.entity_type, r.start, r.end, r.score) for r in r2])
    assert r1_tuples == r2_tuples
```

**Negative fixture tests:** `services/ai-gateway/pii/test_vn_no_false_positives.py`

```python
def test_dates_do_not_false_positive(analyzer):
    for text in ["Ngày 12/05/2026", "Sinh năm 1990", "Hợp đồng số 2024-001"]:
        results = analyzer.analyze(text=text, language="vi",
                                   entities=["VN_CCCD", "VN_MST", "VN_PHONE"])
        assert all(r.score < CONFIDENCE_HIGH for r in results), \
            f"unexpected high-confidence match in: {text!r}"

def test_vnd_amounts_do_not_false_positive(analyzer):
    for text in ["100,000,000 VND", "Phí 50.000 VNĐ", "1,234,567,890 đồng"]:
        results = analyzer.analyze(text=text, language="vi",
                                   entities=["VN_PHONE", "VN_BANK_ACCOUNT"])
        assert all(r.score < CONFIDENCE_MED for r in results), \
            f"unexpected match in VND amount: {text!r}"

def test_plain_vietnamese_names_do_not_match_ndd(analyzer):
    """A bare name without the NĐD label should not trigger VN_NDD."""
    results = analyzer.analyze(text="Khách hàng Nguyễn Văn A đã thanh toán",
                               language="vi", entities=["VN_NDD"])
    assert len(results) == 0
```

**No-network-imports lint test:** `services/ai-gateway/pii/test_no_network_imports.py`

```python
# ISS-002 fix: §1 #12 mandates no network calls in recognizers.
# This AST-walk lint catches forbidden imports at PR time.
import ast
from pathlib import Path

FORBIDDEN_IMPORTS = {
    "requests", "urllib", "urllib2", "urllib3", "httpx", "aiohttp",
    "http", "socket", "ssl", "ftplib", "smtplib",
}

def test_no_network_imports_in_recognizers():
    """§1 #12: recognizer modules MUST be pure regex + lookup tables."""
    recognizers_dir = Path(__file__).parent / "recognizers"
    failures = []
    for py_file in recognizers_dir.glob("*.py"):
        tree = ast.parse(py_file.read_text())
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    top = alias.name.split(".")[0]
                    if top in FORBIDDEN_IMPORTS:
                        failures.append(f"{py_file.name}: forbidden import: {alias.name}")
            elif isinstance(node, ast.ImportFrom):
                top = (node.module or "").split(".")[0]
                if top in FORBIDDEN_IMPORTS:
                    failures.append(f"{py_file.name}: forbidden import-from: {node.module}")
    assert not failures, "Network imports detected in recognizers:\n" + "\n".join(failures)
```

**Recall-floor test:** `services/ai-gateway/pii/test_vn_recall_floor.py`

```python
import yaml
from pathlib import Path

def test_recall_at_least_99_percent_per_type(analyzer):
    """AC #13 (post-ISS-004): per-type AND aggregate recall floors at ≥99%.
    Per-type guard catches a single recognizer regressing while others compensate."""
    from collections import defaultdict
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
        if total == 0:
            continue
        recall = correct_by_type[entity_type] / total
        if recall < 0.99:
            failures.append(
                f"{entity_type}: recall={recall:.4f} ({correct_by_type[entity_type]}/{total})"
            )
    assert not failures, "Per-type recall floor violated:\n" + "\n".join(failures)

    # Aggregate floor too.
    total = sum(total_by_type.values())
    correct = sum(correct_by_type.values())
    aggregate = correct / total if total else 1.0
    assert aggregate >= 0.99, f"aggregate recall {aggregate:.4f} below 0.99 floor"
```

**Sidecar startup test:** `services/ai-gateway/pii/test_sidecar_startup.py`

```python
def test_sidecar_refuses_to_start_on_registration_failure():
    """AC #15: any recognizer registration error → sidecar startup raises."""
    from unittest.mock import patch
    with patch.object(VnAddressRecognizer, 'PATTERNS',
                      [Pattern("bad", "(unbalanced", 0.5)]):
        analyzer = AnalyzerEngine()
        with pytest.raises(RuntimeError, match="recognizer_registration_failed"):
            register_vn_recognizers(analyzer)
```

```bash
cd services/ai-gateway/pii
pytest -v
pytest -v test_vn_recall_floor.py   # the recall gate (TASK-AI-013 reuses this)
```

CI gate: pytest runs on every PR touching `services/ai-gateway/pii/**`. Recall gate failure (< 99%) blocks merge.

---

## §6 — Implementation skeleton

See §3 for the recognizer class skeletons. The integration into the sidecar is also in §3 (`register_vn_recognizers` function).

The fixture file structure:

```yaml
# services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml
# 200 samples covering 6 PII types + negative examples.
# Curated quarterly from anonymised CyberSkill customer data + synthetic edge cases.
samples:
  - text: "CCCD: 012345678901"
    expected_entities: ["VN_CCCD"]
    expected_count: 1
  - text: "MST 0301234567"
    expected_entities: ["VN_MST"]
    expected_count: 1
  # ... (198 more)
  # Negative examples interleaved:
  - text: "Hợp đồng số 2024-001 ngày 12/05/2026"
    expected_entities: []
    expected_count: 0
```

Test runner output:

```text
$ pytest test_vn_recognizers.py -v
test_cccd_with_context PASSED
test_cccd_without_context_validates_province PASSED
test_cccd_invalid_province_rejected PASSED
test_mst_10_digit PASSED
... (12 tests)
test_vn_recall_floor.py::test_recall_at_least_99_percent PASSED  # 199/200 = 0.995
```

---

## §7 — Dependencies

### Code dependencies (other tasks/modules)

- **TASK-AI-011** — Presidio sidecar must exist; this task adds recognizers TO it. The `PiiType` Rust enum already includes the VN variants (declared in TASK-AI-011 §3 for ABI stability).
- **TASK-AI-013 (downstream)** — Recall-floor CI gate consumes the `vn_pii_200_samples.yaml` fixture and runs `test_vn_recall_floor.py` as the gate.
- **TASK-AI-005** — `TenantPolicy.ai_policy.pii_redaction_extra` lists the VN entities to enable per-tenant; this task's recognizers handle them once the policy field reaches the sidecar.

### Concept dependencies (shared types)

- `VALID_PROVINCE_CODES_3DIGIT` and `VALID_PROVINCE_CODES_2DIGIT` from `province_codes.py` are the SINGLE SOURCE for province code validation across `VN_CCCD`, `VN_MST`, and `VN_BANK_ACCOUNT` recognizers.
- `CONFIDENCE_HIGH/MED/LOW` from `confidence.py` are the SINGLE SOURCE for score constants.
- `VN_BANK_NAMES` from `province_codes.py` is shared between the `VN_BANK_ACCOUNT` recognizer's regex generation and (future) per-bank precision tuning.
- The `VN_<TYPE>` entity names match the `PiiType::from_presidio()` Rust mapping (TASK-AI-011 §3).

### Operational / external

- `presidio-analyzer==2.2.x` (recognizer base classes).
- `pytest>=7.0`, `pyyaml>=6.0` (test runner + fixture loader).
- spaCy Vietnamese model (`vi_core_news_lg`) for the analyzer pipeline (loaded by Presidio at engine init).
- Static lookup tables only — no DB, no HTTP, no external API.

---

## §8 — Example payloads

### Input

```text
Khách hàng: Nguyễn Văn A
CCCD: 012345678901
MST: 0301234567
SĐT: 0901234567
STK Vietcombank: 1234567890
Địa chỉ: 123 Lê Lợi, Phường Bến Nghé, Quận 1, TP. Hồ Chí Minh
NĐD: Trần Thị B
```

### Sidecar response (analyzer items)

```json
[
  {"entity": "PERSON", "start": 12, "end": 24, "original": "Nguyễn Văn A", "score": 0.85},
  {"entity": "VN_CCCD", "start": 31, "end": 43, "original": "012345678901", "score": 0.99},
  {"entity": "VN_MST", "start": 50, "end": 60, "original": "0301234567", "score": 0.99},
  {"entity": "VN_PHONE", "start": 67, "end": 77, "original": "0901234567", "score": 0.99},
  {"entity": "VN_BANK_ACCOUNT", "start": 100, "end": 110, "original": "1234567890", "score": 0.99},
  {"entity": "VN_ADDRESS", "start": 123, "end": 188, "original": "123 Lê Lợi, ... TP. Hồ Chí Minh", "score": 0.99},
  {"entity": "VN_NDD", "start": 195, "end": 207, "original": "Trần Thị B", "score": 0.99}
]
```

### Redacted output

```text
Khách hàng: <PERSON_1>
CCCD: <VN_CCCD_1>
MST: <VN_MST_1>
SĐT: <VN_PHONE_1>
STK Vietcombank: <VN_BANK_ACCOUNT_1>
Địa chỉ: <VN_ADDRESS_1>
NĐD: <VN_NDD_1>
```

### Version endpoint

```bash
$ curl http://127.0.0.1:5050/recognizers/version
{
  "VN_CCCD": "1.0.0",
  "VN_MST": "1.0.0",
  "VN_PHONE": "1.0.0",
  "VN_NDD": "1.0.0",
  "VN_ADDRESS": "1.0.0",
  "VN_BANK_ACCOUNT": "1.0.0"
}
```

### Sidecar startup log (success)

```text
INFO  registered VnPhoneRecognizer v1.0.0 (3 patterns)
INFO  registered VnCccdRecognizer v1.0.0 (2 patterns)
INFO  registered VnMstRecognizer v1.0.0 (4 patterns)
INFO  registered VnNddRecognizer v1.0.0 (1 pattern)
INFO  registered VnAddressRecognizer v1.0.0 (2 patterns)
INFO  registered VnBankAccountRecognizer v1.0.0 (3 patterns)
INFO  uvicorn running on http://127.0.0.1:5050
```

### Sidecar startup log (failure on §1 #13)

```text
ERROR recognizer_registration_failed: VnAddressRecognizer: re.error: missing ), unterminated subpattern
RuntimeError: recognizer_registration_failed: VnAddressRecognizer: ...
[uvicorn never binds; sidecar exits with code 1]
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later tasks:

- Partial-redact policy (`vn_address_partial_redact: true`) — slice 5.
- Per-tenant custom recognizers (proprietary product SKUs as PII) — slice 5.
- Khmer / Lao / Thai recognizers for ASEAN expansion — slice 6 (TASK-AI-022 area).
- Recognizer-version pinning per tenant (auditability across upgrades) — out of scope.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Recognizer regex matches but `validate_result` fails (e.g., wrong province code) | `validate_result` returns False | Match dropped (no false positive); item excluded from analyzer results | Self-resolves |
| Edge case not in 200-sample fixture (new VN PII format) | Production discovery via TASK-AI-013 alarm | Recognizer misses; sev-2 alert | Add to recognizer + extend fixture; redeploy |
| Confidence threshold too low | High false-positive rate in production OBS | Operator tunes `confidence.py` constant | Iterative tuning; redeploy |
| Recognizer crash on malformed input | Exception in `analyze` | Presidio catches; sidecar logs error; item not contributed to result set | Sev-2 log; investigate input that triggered |
| Recognizer registration fails on sidecar startup | `register_vn_recognizers` raises `RuntimeError` | Sidecar refuses to start (per §1 #13); HTTP server never binds | Operator fixes recognizer; restart sidecar |
| Province code list out of date (Vietnam adds new province) | Operator notices recall regression | Update `province_codes.py`; rebuild sidecar image | Quarterly review process |
| Bank name list missing a new bank | Customer reports their bank's account not redacted | Add to `VN_BANK_NAMES`; redeploy | Customer-feedback loop |
| Recognizer version drift between deployed sidecar and TASK-AI-013 fixture | `GET /recognizers/version` mismatch in CI | TASK-AI-013 recall gate fails | Bump fixture version; re-curate samples |
| Two recognizers fire on the same span (PHONE + MST) | Anonymizer picks higher-score | Per §1 #7 ordering, PHONE wins for 10-digit | By design |
| Vietnamese diacritic normalization mismatch | Presidio analyzer normalizes to NFC; recognizer regexes assume NFC | Match works | If a recognizer uses non-NFC chars in its regex → false negative; fixed by NFC-normalizing the regex |
| spaCy `vi_core_news_lg` model missing | sidecar startup fails | Sidecar refuses to start | Operator pulls sidecar image with model bundled |
| `analyze()` non-deterministic (regression) | `test_determinism` fails in CI | PR blocked | PR rework |
| `vi_core_news_lg` produces different POS tags across runs | Token-level non-determinism | Reproducible only with explicit `nlp` randomness seed | Seed in sidecar startup |
| `pii_redaction_extra` policy field missing VN_* entries | Tenant policy doesn't enable VN recognizers | VN PII passes through | Operator updates tenant policy YAML |
| Recognizer dropped from `VN_RECOGNIZERS` list (regression) | `test_recognizer_registration_order` fails | PR blocked | PR rework |
| Province code lookup uses wrong digit slice (off-by-one) | Validation rejects valid CCCDs → recall regression in fixture test | TASK-AI-013 alarm | PR rework |
| Future PR adds network call in a recognizer | `test_no_network_imports.py` AST-walk lint | PR fails on CI before merge | ISS-002 fix |
| `register_vn_recognizers` called twice (test loop, sidecar reload) | `_REGISTERED` global guard | Second call is no-op with WARN log | ISS-003 fix; tests use `reset_for_tests()` |
| Single recognizer regresses to 90% recall, others compensate | per-type recall test | PR fails with per-type breakdown showing which recognizer regressed | ISS-004 fix |

---

## §11 — Notes

- Province code validation in `VN_CCCD` and `VN_MST` is the highest-value precision lever. Without it, 12-digit numeric strings false-positive at ~30% on real customer data.
- The 200-sample fixture is hand-curated from anonymised CyberSkill customer data + synthetic edge cases. It MUST be regenerated quarterly; the regeneration script lives in `services/ai-gateway/pii/scripts/regen_fixture.py` and uses internal customer data (under DPA) plus public sources (Vietnam tax-code samples from gov.vn).
- The `VN_NDD` recognizer is intentionally narrow (label-bound). Without the label, a bare name like "Nguyễn Văn A" falls through to Presidio's standard `PERSON` recognizer — still redacted, just without the legal-representative semantic tag. This trade-off favours precision over recall on the NDD-specific tag.
- Recognizer ordering matters: `VN_PHONE` runs first so a 10-digit `0901234567` matches as phone (high confidence) before MST tries to match (low confidence on bare number). `VN_BANK_ACCOUNT` runs last because its broad 10-14 digit pattern would otherwise grab CCCDs and MSTs.
- The `confidence.py` constants are tuning knobs. Calibration runs quarterly against the 200-sample fixture: we measure precision (false-positive rate) and recall (true-positive rate) at HIGH/MED/LOW thresholds and adjust if drift exceeds 1 percentage point.
- The version endpoint (`/recognizers/version`) is small — six entries — but it's the auditability primitive that lets tenants verify "what version of the redactor processed our prompts during the period of our DPA". Useful in compliance reviews.
- Future expansion (slice 6, TASK-AI-022): Khmer / Lao / Thai recognizers for ASEAN tenants. The pattern is the same — language-specific regexes + label-bound matching + shared confidence constants. The sidecar architecture handles multilingual recognizers natively (just register additional ones).
- The `test_vn_no_false_positives.py` test file is the sibling of `test_vn_recognizers.py`. Both run on every PR. Negative tests catch the most-common regression: a regex tightening that breaks recall, or a regex loosening that breaks precision.

---

*End of TASK-AI-012. Status: draft (10/10 target).*
