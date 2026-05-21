"""VN_ADDRESS — Vietnamese postal address recognizer.

Multi-segment address with street, ward, district, city components.
"""

from presidio_analyzer import Pattern, PatternRecognizer

from .confidence import CONFIDENCE_HIGH, CONFIDENCE_MED


class VnAddressRecognizer(PatternRecognizer):
    """VN_ADDRESS — multi-segment Vietnamese postal address."""

    VERSION = "1.0.0"

    PATTERNS = [
        Pattern(
            name="vn_address_full",
            regex=(
                r"\d+\s+[\w\sÀ-ỹ]+,\s*"
                r"(?:Ph(?:ư|u)(?:ơ|o)ng|X[aã]|Qu(?:ậ|a)n|Huy(?:ệ|e)n|Tp\.?|TP\.?|Th(?:à|a)nh ph(?:ố|o))"
                r"[^,]+,\s*"
                r"(?:Qu(?:ậ|a)n|Huy(?:ệ|e)n|Tp\.?|TP\.?)"
                r"[^,]+,\s*"
                r"(?:Tp\.?|TP\.?|Th(?:à|a)nh ph(?:ố|o))"
                r"[\w\sÀ-ỹ\.]+"
            ),
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_address_partial",
            regex=(
                r"(?:Qu(?:ậ|a)n|Huy(?:ệ|e)n|Tp\.?|TP\.?|Ph(?:ư|u)(?:ơ|o)ng|X[aã])"
                r"\s+[\w\sÀ-ỹ\d]+"
            ),
            score=CONFIDENCE_MED,
        ),
    ]

    def __init__(self):
        super().__init__(
            supported_entity="VN_ADDRESS",
            patterns=self.PATTERNS,
            context=["địa chỉ", "address"],
        )
