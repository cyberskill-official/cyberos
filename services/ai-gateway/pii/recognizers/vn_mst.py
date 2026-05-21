"""VN_MST — Vietnamese tax code (Mã số thuế) recognizer.

10-digit entity code or 13-digit entity-branch (with hyphen).
First 2 digits are a valid province code.
"""

from presidio_analyzer import Pattern, PatternRecognizer

from .province_codes import VALID_PROVINCE_CODES_2DIGIT
from .confidence import CONFIDENCE_HIGH, CONFIDENCE_LOW


class VnMstRecognizer(PatternRecognizer):
    """VN_MST — 10 or 13-digit tax code; 13-digit has hyphen between 10 and 3-digit branch."""

    VERSION = "1.0.0"

    PATTERNS = [
        Pattern(
            name="vn_mst_with_context_10",
            regex=r"(?:MST|mst|M[aã] s(?:ố|o) thu(?:ế|e)|m[aã] thu(?:ế|e))\s*[:\.]?\s*(\d{10})",
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_mst_with_context_13",
            regex=r"(?:MST|mst|M[aã] s(?:ố|o) thu(?:ế|e))\s*[:\.]?\s*(\d{10}-\d{3})",
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_mst_plain_10",
            regex=r"\b(\d{10})\b",
            score=CONFIDENCE_LOW,
        ),
        Pattern(
            name="vn_mst_plain_13",
            regex=r"\b(\d{10}-\d{3})\b",
            score=CONFIDENCE_LOW,
        ),
    ]

    def __init__(self):
        super().__init__(
            supported_entity="VN_MST",
            patterns=self.PATTERNS,
            context=["MST", "mã số thuế", "thuế"],
        )

    def validate_result(self, pattern_text: str) -> bool:
        """First 2 digits are the province code (without leading 0)."""
        digits = pattern_text.replace("-", "")[:2]
        return digits in VALID_PROVINCE_CODES_2DIGIT
