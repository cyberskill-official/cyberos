"""FR-AI-012 §5 — Sidecar startup test.

AC #15: sidecar refuses to start if any recognizer registration fails.
"""

import pytest
from unittest.mock import patch

from presidio_analyzer import AnalyzerEngine, Pattern

from recognizers import VnAddressRecognizer, VN_RECOGNIZERS
from presidio_sidecar import register_vn_recognizers, reset_vn_for_tests


def test_sidecar_refuses_to_start_on_registration_failure():
    """AC #15: any recognizer registration error → sidecar startup raises."""
    reset_vn_for_tests()
    bad_pattern = Pattern("bad", "(unbalanced", 0.5)
    with patch.object(VnAddressRecognizer, "PATTERNS", [bad_pattern]):
        analyzer = AnalyzerEngine()
        with pytest.raises(RuntimeError, match="recognizer_registration_failed"):
            register_vn_recognizers(analyzer)
    reset_vn_for_tests()
