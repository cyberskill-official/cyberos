"""VN_NDD — Vietnamese legal representative (Người đại diện) recognizer.

Label-bound name extraction: matches a name preceded by NĐD, Người đại diện,
or Legal representative labels.
"""

import re

from presidio_analyzer import EntityRecognizer, RecognizerResult

from .confidence import CONFIDENCE_HIGH


class VnNddRecognizer(EntityRecognizer):
    """VN_NDD — label-bound legal representative name extraction."""

    VERSION = "1.0.0"

    # Match name after label; capture only the name portion.
    LABEL_REGEX = re.compile(
        r"(?:Ng(?:ười|uoi) [đd][aạ]i di(?:ệ|e)n|N[ĐD]D|Legal representative)"
        r"(?:\s+ph[aá]p lu(?:ậ|a)t)?"
        r"\s*[:\.]?\s*([A-ZÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬĐÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊ"
        r"ÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴ]"
        r"[\w\sÀ-ỹ]+?)(?=[\n,.;]|$)",
        re.UNICODE,
    )

    def __init__(self):
        super().__init__(
            supported_entities=["VN_NDD"],
            supported_language="vi",
        )

    def analyze(self, text, entities, nlp_artifacts=None):
        results = []
        for m in self.LABEL_REGEX.finditer(text):
            results.append(
                RecognizerResult(
                    entity_type="VN_NDD",
                    start=m.start(1),
                    end=m.end(1),
                    score=CONFIDENCE_HIGH,
                )
            )
        return results
