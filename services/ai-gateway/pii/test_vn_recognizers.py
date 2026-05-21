"""FR-AI-012 §5 — Positive fixture tests for VN PII recognizers."""

import pytest
import re

from recognizers import (
    VnCccdRecognizer,
    VnMstRecognizer,
    VnPhoneRecognizer,
    VnNddRecognizer,
    VnAddressRecognizer,
    VnBankAccountRecognizer,
    VN_RECOGNIZERS,
)
from recognizers.confidence import CONFIDENCE_HIGH, CONFIDENCE_MED, CONFIDENCE_LOW


@pytest.fixture
def analyzer():
    from presidio_analyzer import AnalyzerEngine

    a = AnalyzerEngine()
    for rec in VN_RECOGNIZERS:
        a.registry.add_recognizer(rec)
    return a


# ── VN_CCCD ──────────────────────────────────────────────────────────────────


def test_cccd_with_context(analyzer):
    results = analyzer.analyze(
        text="CCCD: 012345678901", language="vi", entities=["VN_CCCD"]
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


def test_cccd_without_context_validates_province(analyzer):
    # 012 is province code 12 (Lai Châu) — valid
    results = analyzer.analyze(
        text="012345678901", language="vi", entities=["VN_CCCD"]
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_LOW


def test_cccd_invalid_province_rejected(analyzer):
    # 999 is not a valid province code
    results = analyzer.analyze(
        text="999345678901", language="vi", entities=["VN_CCCD"]
    )
    assert len(results) == 0


# ── VN_MST ───────────────────────────────────────────────────────────────────


def test_mst_10_digit(analyzer):
    results = analyzer.analyze(
        text="MST: 0301234567", language="vi", entities=["VN_MST"]
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


def test_mst_13_digit_with_branch(analyzer):
    results = analyzer.analyze(
        text="0301234567-001", language="vi", entities=["VN_MST"]
    )
    assert len(results) >= 1


# ── VN_PHONE ─────────────────────────────────────────────────────────────────


def test_vn_phone_84_prefix(analyzer):
    results = analyzer.analyze(
        text="+84901234567", language="vi", entities=["VN_PHONE"]
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


def test_vn_phone_0_prefix(analyzer):
    results = analyzer.analyze(
        text="0901234567", language="vi", entities=["VN_PHONE"]
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


def test_vn_phone_beats_mst_for_10_digit(analyzer):
    """§1 #7: phone runs first; 0901234567 should match VN_PHONE, not VN_MST."""
    results = analyzer.analyze(
        text="0901234567", language="vi", entities=["VN_PHONE", "VN_MST"]
    )
    types = {r.entity_type for r in results}
    assert "VN_PHONE" in types
    phone_score = max(r.score for r in results if r.entity_type == "VN_PHONE")
    mst_score = max(
        (r.score for r in results if r.entity_type == "VN_MST"), default=0
    )
    assert phone_score >= mst_score


# ── VN_NDD ───────────────────────────────────────────────────────────────────


def test_vn_ndd_label_bound(analyzer):
    results = analyzer.analyze(
        text="NĐD: Nguyễn Văn A", language="vi", entities=["VN_NDD"]
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


def test_vn_ndd_english_label(analyzer):
    results = analyzer.analyze(
        text="Legal representative: Lê Văn C", language="vi", entities=["VN_NDD"]
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


# ── VN_ADDRESS ───────────────────────────────────────────────────────────────


def test_vn_address_full(analyzer):
    text = "123 Nguyễn Thị Minh Khai, Phường Bến Nghé, Quận 1, TP. Hồ Chí Minh"
    results = analyzer.analyze(text=text, language="vi", entities=["VN_ADDRESS"])
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


def test_vn_address_partial(analyzer):
    text = "Quận 1, TP. Hồ Chí Minh"
    results = analyzer.analyze(text=text, language="vi", entities=["VN_ADDRESS"])
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_MED


# ── VN_BANK_ACCOUNT ──────────────────────────────────────────────────────────


def test_vn_bank_account_with_context(analyzer):
    results = analyzer.analyze(
        text="STK Vietcombank: 1234567890", language="vi",
        entities=["VN_BANK_ACCOUNT"],
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_HIGH


def test_vn_bank_account_with_bank_name(analyzer):
    results = analyzer.analyze(
        text="Techcombank 34567890123", language="vi",
        entities=["VN_BANK_ACCOUNT"],
    )
    assert len(results) >= 1
    assert results[0].score >= CONFIDENCE_MED


# ── Registration order ───────────────────────────────────────────────────────


def test_recognizer_registration_order():
    """§1 #7: VN_PHONE first, VN_BANK_ACCOUNT last."""
    assert VN_RECOGNIZERS[0].supported_entities[0] == "VN_PHONE"
    assert VN_RECOGNIZERS[-1].supported_entities[0] == "VN_BANK_ACCOUNT"


# ── Version endpoint ─────────────────────────────────────────────────────────


def test_version_endpoint_returns_six_entries():
    """AC #16: every registered recognizer's version is exposed."""
    from presidio_sidecar import app
    from fastapi.testclient import TestClient

    client = TestClient(app)
    resp = client.get("/recognizers/version")
    assert resp.status_code == 200
    body = resp.json()
    expected_keys = {
        "VN_CCCD", "VN_MST", "VN_PHONE", "VN_NDD", "VN_ADDRESS", "VN_BANK_ACCOUNT",
    }
    assert set(body.keys()) == expected_keys
    semver = re.compile(r"^\d+\.\d+\.\d+$")
    for entity, version in body.items():
        assert semver.match(version), f"{entity}: {version!r} is not semver"


# ── Determinism ──────────────────────────────────────────────────────────────


def test_determinism(analyzer):
    """§1 #11 + AC #17: same input twice → identical results."""
    text = "CCCD: 012345678901, MST: 0301234567"
    r1 = analyzer.analyze(text=text, language="vi", entities=["VN_CCCD", "VN_MST"])
    r2 = analyzer.analyze(text=text, language="vi", entities=["VN_CCCD", "VN_MST"])
    r1_tuples = sorted([(r.entity_type, r.start, r.end, r.score) for r in r1])
    r2_tuples = sorted([(r.entity_type, r.start, r.end, r.score) for r in r2])
    assert r1_tuples == r2_tuples
