"""VN_PHONE — Vietnamese phone number recognizer.

Matches +84 prefix or 0-prefix mobile/landline numbers.
Registered FIRST in the recognizer chain so 10-digit phone numbers
win over MST (§1 #7).
"""

from presidio_analyzer import Pattern, PatternRecognizer

from .confidence import CONFIDENCE_HIGH, CONFIDENCE_MED


class VnPhoneRecognizer(PatternRecognizer):
    """VN_PHONE — +84 or 0-prefix mobile/landline."""

    VERSION = "1.0.0"

    PATTERNS = [
        Pattern(
            name="vn_phone_84_mobile",
            regex=r"\+84\s?(?:9\d{8}|3\d{8}|7\d{8}|8\d{8}|5\d{8})",
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_phone_0_mobile",
            regex=r"0(?:9\d{8}|3\d{8}|7\d{8}|8\d{8}|5\d{8})\b",
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_phone_landline",
            regex=r"0(?:2[0-9])\d{7,8}\b",
            score=CONFIDENCE_MED,
        ),
    ]

    def __init__(self):
        super().__init__(
            supported_entity="VN_PHONE",
            patterns=self.PATTERNS,
            context=["SĐT", "điện thoại", "phone", "mobile"],
        )
