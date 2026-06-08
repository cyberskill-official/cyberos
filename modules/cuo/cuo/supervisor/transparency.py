"""EU AI Act Art. 13 disclosure helpers."""

from __future__ import annotations

from .state import Candidate, PathTaken, Transparency


def build_transparency(
    *,
    chosen: Candidate | None,
    confidence: float,
    alternatives: list[Candidate],
    path_taken: PathTaken,
    llm_used: bool,
) -> Transparency:
    return Transparency(
        skill_chosen=chosen.skill_name if chosen else None,
        confidence=confidence,
        alternatives=[
            {"skill": c.skill_name, "confidence": c.confidence}
            for c in alternatives[:3]
        ],
        path_taken=path_taken,
        llm_used=llm_used,
    )
