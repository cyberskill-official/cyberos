"""Closed 11-persona catalogue and persona JWT validation."""

from __future__ import annotations

import re
from dataclasses import dataclass, field


class PersonaError(ValueError):
    """Raised when a requested persona or persona JWT is invalid."""


@dataclass(frozen=True)
class SupervisorPersona:
    key: str
    display_name: str
    keyword_bank: tuple[str, ...]
    system_prompt: str
    defer_to_human_matrix: tuple[str, ...] = field(default_factory=tuple)
    workflow_slug: str | None = None


PERSONAS: dict[str, SupervisorPersona] = {
    "genie": SupervisorPersona(
        "genie",
        "Genie",
        ("route", "help", "workflow", "ask", "draft"),
        "Route the request to the best CyberOS workflow. Prefer asking when uncertain.",
    ),
    "ceo": SupervisorPersona(
        "ceo",
        "Chief Executive Officer",
        ("strategy", "board", "investor", "capital", "okr"),
        "Act as the CEO persona and protect strategic decisions from automation mistakes.",
        ("capital_commit", "board_commitment", "hire_c_level"),
        "chief-executive-officer",
    ),
    "coo": SupervisorPersona(
        "coo",
        "Chief Operating Officer",
        ("delivery", "capacity", "operating", "vendor", "quarterly review"),
        "Act as the COO persona for operating cadence and delivery decisions.",
        ("vendor_termination", "capacity_lock"),
        "chief-operating-officer",
    ),
    "cfo": SupervisorPersona(
        "cfo",
        "Chief Financial Officer",
        ("invoice", "vat", "budget", "forecast", "cash", "billing"),
        "Act as the CFO persona. Never emit money-moving actions without a human.",
        ("invoice_emit", "wire_transfer", "payment_release", "budget_approval"),
        "chief-financial-officer",
    ),
    "cmo": SupervisorPersona(
        "cmo",
        "Chief Marketing Officer",
        ("campaign", "brand", "press", "analyst", "marketing"),
        "Act as the CMO persona for marketing and brand workflows.",
        ("press_release_publish", "campaign_launch"),
        "chief-marketing-officer",
    ),
    "cto": SupervisorPersona(
        "cto",
        "Chief Technology Officer",
        ("deploy", "architecture", "incident", "rollback", "feature request", "ship"),
        "Act as the CTO persona for architecture, incidents, and engineering execution.",
        ("production_deploy", "data_delete", "incident_close"),
        "chief-technology-officer",
    ),
    "chro": SupervisorPersona(
        "chro",
        "Chief Human Resources Officer",
        ("people", "comp", "talent", "hire", "onboarding", "hr"),
        "Act as the CHRO persona and avoid autonomous employee-impacting actions.",
        ("offer_send", "termination", "comp_change"),
        "chief-human-resources-officer",
    ),
    "cso": SupervisorPersona(
        "cso",
        "Chief Security Officer",
        ("security", "soc2", "threat", "vulnerability", "physical security"),
        "Act as the security persona; default to human oversight for destructive controls.",
        ("destructive_security_change", "access_revoke", "incident_declare"),
        "chief-information-security-officer",
    ),
    "clo": SupervisorPersona(
        "clo",
        "Chief Legal Officer",
        ("legal", "contract", "msa", "nda", "regulatory", "ip"),
        "Act as the legal persona and never bind the company without a human.",
        ("contract_sign", "regulatory_filing_submit", "legal_position_commit"),
        "chief-legal-officer",
    ),
    "cdo": SupervisorPersona(
        "cdo",
        "Chief Data Officer",
        ("data", "governance", "quality", "customer 360", "memory"),
        "Act as the data persona for governance, quality, and data-product routing.",
        ("data_export", "retention_policy_change"),
        "chief-data-officer",
    ),
    "cpo": SupervisorPersona(
        "cpo",
        "Chief Product Officer",
        ("product", "roadmap", "prd", "metrics", "feature"),
        "Act as the product persona for roadmap and product artefact routing.",
        ("roadmap_publish", "pricing_change"),
        "chief-product-officer",
    ),
}


_CLAIM_RE = re.compile(r"^cuo-(?P<persona>[a-z]+)@(?P<version>\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$")


def get_persona(key: str) -> SupervisorPersona:
    try:
        return PERSONAS[key]
    except KeyError as exc:
        raise PersonaError(f"unknown persona: {key}") from exc


def validate_agent_persona_claim(claim: str | None, requested_persona: str) -> str:
    """Validate `agent_persona: cuo-<persona>@<semver>` and return semver."""

    if claim is None or not claim:
        raise PersonaError("missing agent_persona claim")
    match = _CLAIM_RE.match(claim)
    if match is None:
        raise PersonaError(f"malformed agent_persona claim: {claim}")
    claimed = match.group("persona")
    if claimed != requested_persona:
        raise PersonaError(f"persona_mismatch: claimed={claimed} requested={requested_persona}")
    return match.group("version")


def persona_workflow_slug(persona_key: str) -> str | None:
    return get_persona(persona_key).workflow_slug
