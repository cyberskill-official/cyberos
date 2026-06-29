"""FR-CHAT-004 — Vietnamese message search helpers."""

from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass, field


_TOKEN_RE = re.compile(r"[\w]+", re.UNICODE)


def normalize_vietnamese(text: str) -> str:
    """Normalize user text for dual-index search.

    Keeps Vietnamese diacritics for exact PGroonga-style matching while folding
    case, width, and composed/decomposed Unicode differences.
    """
    return unicodedata.normalize("NFC", text).casefold().strip()


def _strip_marks(text: str) -> str:
    decomposed = unicodedata.normalize("NFD", text)
    return "".join(ch for ch in decomposed if unicodedata.category(ch) != "Mn")


def vietnamese_bigrams(text: str) -> list[str]:
    """Return word tokens plus overlapping bigrams for Vietnamese recall."""
    normalized = normalize_vietnamese(text)
    tokens = _TOKEN_RE.findall(normalized)
    out: list[str] = []
    for token in tokens:
        out.append(token)
        folded = _strip_marks(token)
        if folded != token:
            out.append(folded)
        compact = token.replace("_", "")
        if len(compact) > 1:
            out.extend(compact[i : i + 2] for i in range(len(compact) - 1))
    return sorted(set(out))


@dataclass
class SearchIndex:
    postings: dict[str, set[str]] = field(default_factory=dict)

    def add(self, message_id: str, body: str) -> None:
        for token in vietnamese_bigrams(body):
            self.postings.setdefault(token, set()).add(message_id)

    def search(self, query: str) -> list[str]:
        tokens = vietnamese_bigrams(query)
        scores: dict[str, int] = {}
        for token in tokens:
            for message_id in self.postings.get(token, set()):
                scores[message_id] = scores.get(message_id, 0) + 1
        return [
            mid
            for mid, _score in sorted(scores.items(), key=lambda item: (-item[1], item[0]))
        ]


def recall_at(found: list[str], expected: set[str], k: int = 20) -> float:
    if not expected:
        return 1.0
    return len(set(found[:k]) & expected) / len(expected)
