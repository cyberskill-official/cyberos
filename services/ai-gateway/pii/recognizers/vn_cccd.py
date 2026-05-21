"""VN_CCCD — 12-digit Vietnamese citizen identity number recognizer.

First 3 digits are a valid province code. Supports labeled (high confidence)
and bare (low confidence) patterns.
"""

from presidio_analyzer import Pattern, PatternRecognizer

from .province_codes import VALID_PROVINCE_CODES_3DIGIT
from .confidence import CONFIDENCE_HIGH, CONFIDENCE_LOW


class VnCccdRecognizer(PatternRecognizer):
    """VN_CCCD — 12-digit citizen identity number; first 3 digits are province code."""

    VERSION = "1.0.0"

    PATTERNS = [
        Pattern(
            name="vn_cccd_strict_with_context",
            regex=r"(?:CCCD|cccd|C[că]n c(?:ước|uoc) c[ôo]ng d[âa]n|c[că]n c(?:ước|uoc)|[đd][iị]nh danh)\s*[:\.]?\s*(\d{12})",
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_cccd_plain",
            regex=r"\b(\d{12})\b",
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
        digits = "".join(c for c in pattern_text if c.isdigit())
        if len(digits) < 3:
            return False
        return digits[:3] in VALID_PROVINCE_CODES_3DIGIT
