"""Rule-score node wrapping the existing Phase 1 router."""

from __future__ import annotations

from pathlib import Path

from cuo.core.catalog import discover_personas
from cuo.core.router import score_one_off

from ..persona import persona_workflow_slug
from ..state import Candidate


def rule_score(query: str, *, cuo_root: Path, persona_key: str = "genie") -> list[Candidate]:
    personas = discover_personas(cuo_root)
    decisions = score_one_off(query, personas, persona_threshold=0.5, workflow_threshold=0.0)
    wanted_persona = persona_workflow_slug(persona_key)
    out: list[Candidate] = []
    for d in decisions:
        if wanted_persona is not None and d.persona_slug != wanted_persona:
            continue
        skill = f"{d.persona_slug}/{d.workflow_slug}"
        out.append(
            Candidate(
                skill_name=skill,
                confidence=d.confidence,
                arguments={"persona_workflow": skill},
                score_components={"rationale": d.rationale},
                persona_slug=d.persona_slug,
                workflow_slug=d.workflow_slug,
                operation=_operation_from_skill(skill),
            )
        )
    return out


def _operation_from_skill(skill_name: str) -> str | None:
    lower = skill_name.lower()
    if "invoice" in lower:
        return "invoice_emit"
    if "deploy" in lower:
        return "production_deploy"
    if "contract" in lower or "msa" in lower or "nda" in lower:
        return "contract_sign"
    if "roadmap" in lower:
        return "roadmap_publish"
    return None
