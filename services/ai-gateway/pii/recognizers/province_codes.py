"""
Single source of truth for Vietnamese province codes.
Used by VN_CCCD (first 3 digits) and VN_MST (first 2 digits) validation.
Source: General Statistics Office of Vietnam, 2024 administrative divisions.
"""

# 3-digit codes used by CCCD (first 3 digits of the 12-digit number).
VALID_PROVINCE_CODES_3DIGIT = frozenset([
    # Northern
    "001", "002", "004", "006", "008", "010", "011", "012", "014", "015",
    "017", "019", "020", "022", "024", "025", "026", "027", "030", "031",
    "033", "034", "035", "036", "037", "038", "040", "042", "044", "045",
    "046", "048", "049", "051", "052", "053", "054", "055", "056", "058",
    # Central
    "060", "062", "064", "066", "068", "070", "072", "074", "075", "077",
    "079", "080", "082", "084", "086", "087", "088", "089", "091", "092",
    "093", "094", "095", "096",
    # Southern
    "097", "098", "099",
])

# 2-digit codes derived from 3-digit (strip leading zero) — used by MST.
VALID_PROVINCE_CODES_2DIGIT = frozenset(
    code[1:] for code in VALID_PROVINCE_CODES_3DIGIT
)

# Major Vietnamese banks for bank-account context matching.
VN_BANK_NAMES = frozenset([
    "Vietcombank", "BIDV", "Techcombank", "Sacombank", "Agribank",
    "MBBank", "VPBank", "ACB", "VIB", "TPBank", "SHB", "HDBank",
    "OCB", "SCB", "LienVietPostBank", "VietABank", "NamABank",
    "PGBank", "BacABank", "PVcomBank", "SeABank", "MSB",
    "VietinBank", "Eximbank", "DongABank", "KienLongBank",
])
