"""
Shared confidence-score constants for VN PII recognizers.
Tuned against the 200-sample VN PII test set.
Update one file when calibrating recall/precision trade-offs.
"""

CONFIDENCE_HIGH = 0.99   # explicit label match (CCCD:, MST:, NĐD:, STK:)
CONFIDENCE_MED  = 0.85   # strong contextual match (e.g., bank name nearby)
CONFIDENCE_LOW  = 0.50   # bare pattern match without context
