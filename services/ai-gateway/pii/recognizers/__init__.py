"""TASK-AI-012 — Vietnamese PII recognizers for Presidio."""

from .vn_cccd import VnCccdRecognizer
from .vn_mst import VnMstRecognizer
from .vn_phone import VnPhoneRecognizer
from .vn_ndd import VnNddRecognizer
from .vn_address import VnAddressRecognizer
from .vn_bank import VnBankAccountRecognizer

# §1 #7 registration order: phone FIRST (wins over MST for 10-digit numbers),
# bank LAST (broad pattern; let specific recognizers run first).
VN_RECOGNIZERS = [
    VnPhoneRecognizer(),
    VnCccdRecognizer(),
    VnMstRecognizer(),
    VnNddRecognizer(),
    VnAddressRecognizer(),
    VnBankAccountRecognizer(),
]

__all__ = [
    "VnCccdRecognizer",
    "VnMstRecognizer",
    "VnPhoneRecognizer",
    "VnNddRecognizer",
    "VnAddressRecognizer",
    "VnBankAccountRecognizer",
    "VN_RECOGNIZERS",
]
