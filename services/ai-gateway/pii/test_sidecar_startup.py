"""FR-AI-012 §5 — Sidecar startup test.

AC #15: sidecar refuses to start if any recognizer registration fails.
"""

import os
from unittest.mock import patch

os.environ["CYBEROS_PII_PATTERN_ONLY_NLP"] = "1"

import pytest

from recognizers import VnAddressRecognizer, VN_RECOGNIZERS
from pattern_nlp import create_pattern_analyzer
from presidio_sidecar import register_vn_recognizers, reset_vn_for_tests


def test_sidecar_refuses_to_start_on_registration_failure():
    """AC #15: any recognizer registration error → sidecar startup raises."""
    reset_vn_for_tests()
    analyzer = create_pattern_analyzer()
    original_add = analyzer.registry.add_recognizer

    def fail_on_address(recognizer):
        if isinstance(recognizer, VnAddressRecognizer):
            raise RuntimeError("synthetic registration failure")
        return original_add(recognizer)

    with patch.object(analyzer.registry, "add_recognizer", side_effect=fail_on_address):
        with pytest.raises(RuntimeError, match="recognizer_registration_failed"):
            register_vn_recognizers(analyzer)
    reset_vn_for_tests()
