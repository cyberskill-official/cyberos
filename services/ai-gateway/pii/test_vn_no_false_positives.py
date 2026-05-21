"""FR-AI-012 §5 — Negative fixture tests for VN PII recognizers.

Ensures recognizers do NOT false-positive on plain Vietnamese text,
dates, VND amounts, and bare names without NĐD labels.
"""

import pytest

from recognizers import VN_RECOGNIZERS
from recognizers.confidence import CONFIDENCE_HIGH, CONFIDENCE_MED


@pytest.fixture
def analyzer():
    from presidio_analyzer import AnalyzerEngine

    a = AnalyzerEngine()
    for rec in VN_RECOGNIZERS:
        a.registry.add_recognizer(rec)
    return a


# ── Dates ────────────────────────────────────────────────────────────────────


def test_dates_do_not_false_positive(analyzer):
    for text in [
        "Ngày 12/05/2026",
        "Sinh năm 1990",
        "Hợp đồng số 2024-001",
        "Thời gian: 14:30 ngày 21/05/2026",
    ]:
        results = analyzer.analyze(
            text=text,
            language="vi",
            entities=["VN_CCCD", "VN_MST", "VN_PHONE"],
        )
        high_conf = [r for r in results if r.score >= CONFIDENCE_HIGH]
        assert len(high_conf) == 0, (
            f"unexpected high-confidence match in: {text!r} → {high_conf}"
        )


# ── VND amounts ──────────────────────────────────────────────────────────────


def test_vnd_amounts_do_not_false_positive(analyzer):
    for text in [
        "100,000,000 VND",
        "Phí 50.000 VNĐ",
        "1,234,567,890 đồng",
        "Lương: 20,000,000 VND/tháng",
    ]:
        results = analyzer.analyze(
            text=text,
            language="vi",
            entities=["VN_PHONE", "VN_BANK_ACCOUNT"],
        )
        med_conf = [r for r in results if r.score >= CONFIDENCE_MED]
        assert len(med_conf) == 0, (
            f"unexpected match in VND amount: {text!r} → {med_conf}"
        )


# ── Bare names (no NĐD label) ───────────────────────────────────────────────


def test_plain_vietnamese_names_do_not_match_ndd(analyzer):
    """A bare name without the NĐD label should not trigger VN_NDD."""
    for text in [
        "Khách hàng Nguyễn Văn A đã thanh toán",
        "Nhân viên Trần Thị B phụ trách",
        "Giám đốc Lê Văn C ký tên",
    ]:
        results = analyzer.analyze(
            text=text, language="vi", entities=["VN_NDD"]
        )
        assert len(results) == 0, (
            f"unexpected NĐD match in bare-name text: {text!r}"
        )


# ── Plain numbers (no context) ───────────────────────────────────────────────


def test_random_numbers_do_not_false_positive_high(analyzer):
    """Random digit strings should not produce high-confidence matches."""
    for text in [
        "Số lượng: 1,000 sản phẩm",
        "Khoảng cách: 123.45 km",
        "Nhiệt độ: 35.2°C",
        "Diện tích: 1,500 m²",
    ]:
        results = analyzer.analyze(
            text=text,
            language="vi",
            entities=["VN_CCCD", "VN_MST", "VN_PHONE", "VN_BANK_ACCOUNT"],
        )
        high_conf = [r for r in results if r.score >= CONFIDENCE_HIGH]
        assert len(high_conf) == 0, (
            f"unexpected high-confidence match: {text!r} → {high_conf}"
        )
