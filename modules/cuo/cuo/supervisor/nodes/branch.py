"""Confidence-band branch node."""

from __future__ import annotations

from ..state import ASK_THRESHOLD, CASCADE_THRESHOLD_HIGH, CASCADE_THRESHOLD_LOW, Candidate, LlmRoutingPick, PathTaken


def choose_path(
    rule_scores: list[Candidate],
    *,
    cascade_taken: bool,
    llm_pick: LlmRoutingPick | None = None,
) -> tuple[PathTaken, Candidate | None]:
    candidates = list(rule_scores)
    if llm_pick is not None:
        candidates.insert(
            0,
            Candidate(
                skill_name=llm_pick.skill_name,
                confidence=llm_pick.confidence,
                arguments=llm_pick.arguments,
                score_components={"source": "llm_cascade", "rationale": llm_pick.rationale},
            ),
        )
    if not candidates:
        return "defer", None
    best = sorted(candidates, key=lambda c: (-c.confidence, c.skill_name))[0]
    if best.confidence >= ASK_THRESHOLD:
        return ("cascade_then_auto" if cascade_taken else "auto"), best
    if best.confidence >= CASCADE_THRESHOLD_HIGH:
        return ("cascade_then_ask" if cascade_taken else "ask"), best
    if best.confidence >= CASCADE_THRESHOLD_LOW and not cascade_taken:
        return "cascade", best
    if cascade_taken:
        return "cascade_then_ask", best
    return "defer", best
