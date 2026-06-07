"""VN_BANK_ACCOUNT — Vietnamese bank account number recognizer.

10-14 digit account number with bank-name proximity or context keywords.
Distinguishes from CCCD (12 digits with province prefix) and MST (10 digits).
"""

from presidio_analyzer import Pattern, PatternRecognizer

from .province_codes import VALID_PROVINCE_CODES_3DIGIT, VALID_PROVINCE_CODES_2DIGIT, VN_BANK_NAMES
from .confidence import CONFIDENCE_HIGH, CONFIDENCE_MED, CONFIDENCE_LOW


class VnBankAccountRecognizer(PatternRecognizer):
    """VN_BANK_ACCOUNT — 10-14 digit account; bank-name proximity boost."""

    VERSION = "1.0.0"

    _BANK_NAMES_RE = r"(?:" + "|".join(VN_BANK_NAMES) + r")"

    PATTERNS = [
        Pattern(
            name="vn_bank_with_context",
            regex=rf"(?:STK|t[aà]i kho(?:ả|a)n|account number|s(?:ố|o) t[aà]i kho(?:ả|a)n)\s*(?:{_BANK_NAMES_RE})?\s*[:\.]?\s*(\d{{10,14}})",
            score=CONFIDENCE_HIGH,
        ),
        Pattern(
            name="vn_bank_with_bank_name",
            regex=rf"{_BANK_NAMES_RE}[\w\s:,]*?(\d{{10,14}})",
            score=CONFIDENCE_MED,
        ),
        Pattern(
            name="vn_bank_plain",
            regex=r"\b(\d{10,14})\b",
            score=CONFIDENCE_LOW,
        ),
    ]

    def __init__(self):
        super().__init__(
            supported_entity="VN_BANK_ACCOUNT",
            patterns=self.PATTERNS,
            context=["STK", "tài khoản", "account", "số tài khoản"],
            supported_language="vi",
        )

    def validate_result(self, pattern_text: str) -> bool:
        """Distinguish from VN_CCCD (12 digits) and VN_MST (10 digits)."""
        digits = "".join(c for c in pattern_text if c.isdigit())
        lowered = pattern_text.lower()
        if any(
            keyword in lowered
            for keyword in ("stk", "tài khoản", "tai khoan", "account", "vietcombank")
        ):
            return 10 <= len(digits) <= 14
        if len(digits) == 12 and digits[:3] in VALID_PROVINCE_CODES_3DIGIT:
            return False  # likely a CCCD
        if len(digits) == 10 and digits[:2] in VALID_PROVINCE_CODES_2DIGIT:
            return False  # likely an MST
        return True
