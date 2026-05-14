"""Rule-based routing — Phase 1 of CUO.

Algorithm:
    1. Normalise the query (strip Vietnamese diacritics, lowercase, drop punct).
    2. For each catalog skill, compute a relevance score:
         +5 for a verbatim skill-name match (with dashes → spaces)
         +3 per keyword from the per-skill keyword bank that appears
         +2 if the query has Vietnamese diacritics and the skill is region=VN
    3. Pick the top scorer above a minimum-confidence threshold (3.0);
       below threshold, return None ("ask the operator to clarify").
    4. Run a per-skill argument extractor against the original query to
       hydrate the decision with structured args (MST, amount, bank, …).

The keyword bank is intentionally small and skill-specific — production
CUO will hand this work to an LLM, but Phase 1 demonstrates the
architecture without a model dependency.
"""

from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass, field
from typing import Callable

from .catalog import SkillEntry

CONFIDENCE_THRESHOLD = 3.0
CONFIDENCE_SATURATION = 10.0

# Per-skill argument extractor. Maps skill name → function (query → dict).
ARG_EXTRACTORS: dict[str, Callable[[str], dict]] = {}


@dataclass
class RoutingDecision:
    skill_name: str
    confidence: float            # 0.0-1.0
    arguments: dict
    rationale: str
    alternative_skills: list[str] = field(default_factory=list)


def register_extractor(skill_name: str):
    def decorator(fn):
        ARG_EXTRACTORS[skill_name] = fn
        return fn
    return decorator


# ---------------------------------------------------------------------------
# Argument extractors (regex-only; production would use an LLM extractor).
# ---------------------------------------------------------------------------


@register_extractor("vn-mst-validate")
def _extract_mst(query: str) -> dict:
    m = re.search(r"\b(\d{10}(?:-\d{3})?)\b", query)
    return {"input": m.group(1)} if m else {}


@register_extractor("vn-vat-invoice")
def _extract_invoice(query: str) -> dict:
    # Best-effort — production CUO routes this through an LLM extractor.
    amount = re.search(
        r"(\d+(?:[.,]\d+)?)\s*(?:k|nghin|nghìn|m|trieu|triệu|tr|vnd|đ)?",
        query.lower(),
    )
    return {"_extracted": "see body — too rich for a regex"} if amount else {}


_BANK_CODES = (
    "VCB BIDV CTG ACB TCB MB VPB HDB TPB SCB OCB SHB MSB EIB LPB SEAB "
    "VIB NAB AGRIBANK STB"
).split()
_BANK_RE = re.compile(r"\b(" + "|".join(_BANK_CODES) + r")\b", re.IGNORECASE)


@register_extractor("vn-bank-transfer")
def _extract_qr(query: str) -> dict:
    bank_re = _BANK_RE.search(query)
    # Amount: a digit run that is immediately followed (with optional space) by
    # a currency suffix (VND/đ/k/m/tr). This anchors the amount to its unit
    # and lets us distinguish it from an account number.
    amount_match = re.search(
        r"(\d[\d,.]*)\s*(vnd|đ|k|m|tr)\b",
        query,
        re.IGNORECASE,
    )
    amount_span: tuple[int, int] | None = amount_match.span(1) if amount_match else None
    amount_value = amount_match.group(1).replace(",", "") if amount_match else None

    # Account: the first 6–19-digit run that is NOT the amount span and that
    # appears after the bank code (if any). The bank code typically precedes
    # the account in natural language.
    bank_end = bank_re.end() if bank_re else 0
    account_value: str | None = None
    for m in re.finditer(r"\b(\d{6,19})\b", query):
        if amount_span and m.span(1) == amount_span:
            continue
        if m.start() < bank_end:
            continue
        account_value = m.group(1)
        break

    return {
        "bank": bank_re.group(1).upper() if bank_re else None,
        "account": account_value,
        "amount": amount_value,
    }


@register_extractor("vneid-integration")
def _extract_cccd(query: str) -> dict:
    m = re.search(r"\b(\d{12})\b", query)
    return {"input": m.group(1)} if m else {}


# ---------------------------------------------------------------------------
# Scoring helpers.
# ---------------------------------------------------------------------------


def _normalize(s: str) -> str:
    """Strip Vietnamese diacritics, lowercase, drop non-alnum."""
    nfkd = unicodedata.normalize("NFKD", s)
    no_diacritics = "".join(c for c in nfkd if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9\s]", " ", no_diacritics.lower()).strip()


def _has_vietnamese_diacritics(s: str) -> bool:
    return any(unicodedata.combining(c) for c in unicodedata.normalize("NFKD", s))


# Per-skill keyword bank. Tokens are matched in normalised form (diacritics
# stripped, lowercased). Keep this list short — it's the rule-based stand-in
# for the LLM Phase 2 will use.
_KEYWORD_BANK = {
    "vn-mst-validate":     ["mst", "tax code", "ma so thue", "validate tax", "kiem tra mst"],
    "vn-vat-invoice":      ["invoice", "hoa don", "vat", "gtgt", "e-invoice", "xuat hoa don"],
    "vn-bank-transfer":    ["transfer", "qr", "chuyen khoan", "vietqr", "napas", "ma qr"],
    "vneid-integration":   ["cccd", "citizen id", "can cuoc", "vneid", "id card", "danh tinh"],
    "vn-tax-filing":       ["filing", "return", "to khai", "ke khai thue", "monthly vat",
                            "quarterly vat", "tax return", "vat return"],
    "vn-legal-compliance": ["compliance", "law", "decree", "nghi dinh", "thong tu", "pdpd",
                            "cybersecurity", "personal data"],
}


def route(query: str, catalog: list[SkillEntry]) -> RoutingDecision | None:
    """Return the top-scoring skill above threshold, or None."""
    query_norm = _normalize(query)
    has_vn = _has_vietnamese_diacritics(query)

    scored: list[tuple[float, SkillEntry, str]] = []
    for skill in catalog:
        score = 0.0
        rationale_parts: list[str] = []

        # +5 if skill name appears verbatim (with or without dashes)
        name_norm = skill.name.replace("-", " ")
        if name_norm in query_norm:
            score += 5.0
            rationale_parts.append(f"name match `{skill.name}`")

        # +3 per keyword hit
        for kw in _KEYWORD_BANK.get(skill.name, []):
            kw_norm = _normalize(kw)
            if kw_norm and kw_norm in query_norm:
                score += 3.0
                rationale_parts.append(f"keyword `{kw}`")

        # +2 for VN region match if query has Vietnamese diacritics
        if has_vn and skill.region == "VN":
            score += 2.0
            rationale_parts.append("VN region match")

        if score > 0:
            scored.append((score, skill, "; ".join(rationale_parts)))

    if not scored:
        return None
    scored.sort(key=lambda t: -t[0])
    top_score, top_skill, top_rationale = scored[0]
    if top_score < CONFIDENCE_THRESHOLD:
        return None

    extractor = ARG_EXTRACTORS.get(top_skill.name)
    args = extractor(query) if extractor else {}

    # Normalise confidence to 0-1 (saturate at score=CONFIDENCE_SATURATION)
    confidence = min(top_score / CONFIDENCE_SATURATION, 1.0)

    return RoutingDecision(
        skill_name=top_skill.name,
        confidence=confidence,
        arguments=args,
        rationale=top_rationale,
        alternative_skills=[s.name for _s, s, _r in scored[1:4]],
    )
