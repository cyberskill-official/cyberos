"""Two-stage router — query → persona match → workflow match.

Per cuo/MODULE.md §1.1: the router uses keyword matching, the funding-stage matrix,
the disambiguation matrix, and the request's domain language to:

    1. Identify the persona (which C-role best suited for this work)
    2. Select a workflow (within that persona's workflows/ folder)
    3. Return a RoutingDecision with confidence + alternatives

Phase 1 router is keyword-based + filename-fuzzy. Phase 2 will layer in LLM
re-ranking for ambiguous queries per legacy v0.1.0 architecture.

Confidence threshold: 0.5 on a 0.0-1.0 scale (per cuo/docs/SPEC.md).
Vietnamese-diacritic-aware scoring preserved from legacy v0.1.0.
"""

from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass, field

from cuo.core.catalog import PersonaEntry, WorkflowEntry, discover_workflows


# Persona keyword bank — high-leverage terms that score persona matches.
# Each persona has a primary slug + alias list. Aliases include common
# abbreviations, role variants, and Vietnamese transliterations where
# CyberSkill's HCMC operations require them.
_PERSONA_KEYWORDS: dict[str, list[str]] = {
    "ceo": ["chief-executive-officer", "chief executive", "executive officer", "tổng giám đốc"],
    "cto": ["chief-technology-officer", "chief technology", "technology officer", "vp engineering", "head of engineering"],
    "cfo": ["chief-financial-officer", "chief financial", "financial officer", "vp finance", "giám đốc tài chính"],
    "coo": ["chief-operating-officer", "chief operating", "operating officer", "vp operations", "head of delivery"],
    "chro": ["chief-human-resources-officer", "chief human resources", "vp people", "head of people", "vp hr"],
    "ciso": ["chief-information-security-officer", "chief information security", "head of security", "vp security"],
    "caio": ["chief-ai-officer", "chief ai", "head of ai", "vp ai", "ai officer"],
    "clo-legal": ["clo", "general counsel", "chief legal", "head of legal", "vp legal"],
    "chief-of-staff": ["chief of staff", "cos", "head of staff"],
    "cpo-product": ["cpo product", "chief product", "vp product", "head of product"],
    "cpo-privacy": ["cpo privacy", "chief privacy", "data protection officer", "dpo"],
    "cpo-people": ["cpo people", "chief people", "vp people"],
    "cpo-procurement": ["cpo procurement", "chief procurement", "vp procurement"],
    "cdo-data": ["cdo data", "chief data", "vp data", "head of data"],
    "cdo-digital": ["cdo digital", "chief digital", "head of digital"],
    "cdo-diversity": ["cdo diversity", "chief diversity", "head of dei"],
    "cso-strategy": ["cso strategy", "chief strategy", "vp strategy", "head of strategy"],
    "cso-security": ["cso security", "chief security", "physical security officer"],
    "cso-sustainability": ["cso sustainability", "chief sustainability", "head of sustainability"],
    "cso-sales": ["cso sales", "chief sales", "vp sales", "head of sales"],
    "cro-revenue": ["cro revenue", "chief revenue", "vp revenue"],
    "cro-risk": ["cro risk", "chief risk", "head of risk"],
    "cro-restructuring": ["cro restructuring", "chief restructuring", "turnaround officer"],
    "cco-customer": ["cco customer", "chief customer", "vp customer success", "head of cs"],
    "cco-communications": ["cco comms", "chief communications", "head of pr", "vp comms"],
    "cco-commercial": ["cco commercial", "chief commercial", "head of channels"],
    "cco-compliance": ["cco compliance", "chief compliance", "head of compliance"],
    "cao-admin": ["cao admin", "chief administrative", "head of admin"],
    "cao-accounting": ["cao accounting", "chief accounting", "controller"],
    "cio-information": ["cio information", "chief information", "vp it", "head of it"],
    "cio-investment": ["cio investment", "chief investment", "head of investments"],
    "clo-learning": ["clo learning", "chief learning", "head of l&d", "vp learning"],
    "cmo": ["chief-marketing-officer", "chief marketing", "vp marketing", "head of marketing"],
    "cgo": ["chief-growth-officer", "chief growth", "vp growth", "head of growth"],
    "cxo": ["chief-experience-officer", "chief experience", "vp customer experience", "head of cx"],
    "chief-architect": ["chief architect", "principal architect", "head of architecture"],
    "chief-brand-officer": ["chief brand", "head of brand", "vp brand"],
    "chief-trust-officer": ["chief trust", "head of trust", "vp trust"],
    "chief-ethics-officer": ["chief ethics", "head of ai ethics", "ethics officer"],
    "chief-esg-officer": ["chief esg", "head of esg"],
    "chief-transformation-officer": ["chief transformation", "head of transformation"],
    "chief-innovation-officer": ["chief innovation", "head of innovation"],
    "chief-knowledge-officer": ["chief knowledge", "head of km", "knowledge officer"],
    "chief-medical-officer": ["chief medical", "cmo medical", "head of medical"],
    "chief-automation-officer": ["chief automation", "head of automation", "rpa lead"],
    "chief-remote-officer": ["chief remote", "head of remote", "remote lead"],
    "chief-happiness-officer": ["chief happiness", "head of happiness", "wellbeing lead"],
}


# Workflow keyword bank — common natural-language phrases mapped to workflow patterns.
# Used as a secondary scoring layer once the persona is matched.
_WORKFLOW_KEYWORDS: list[tuple[str, list[str]]] = [
    ("board-update", ["board deck", "board update", "quarterly board", "board chapter"]),
    ("roadmap", ["roadmap", "product roadmap", "transformation roadmap"]),
    ("strategy", ["annual strategy", "strategic plan", "vision document"]),
    ("review", ["quarterly review", "monthly review", "weekly review"]),
    ("forecast", ["forecast", "rolling forecast", "quarterly forecast"]),
    ("budget", ["annual budget", "budget cycle"]),
    ("close", ["monthly close", "book close", "financial close"]),
    ("incident", ["incident response", "post-incident", "postmortem"]),
    ("hire", ["hire decision", "interview loop", "offer letter"]),
    ("comp", ["compensation", "comp cycle", "annual comp"]),
    ("crisis", ["crisis response", "crisis comms"]),
    ("breach", ["breach response", "data breach", "security incident"]),
    ("pia", ["privacy impact", "dpia", "privacy assessment"]),
    ("dsr", ["data subject request", "gdpr access", "dsar"]),
    ("churn", ["churn analysis", "customer churn"]),
    ("cab", ["customer advisory board", "cab cycle"]),
    ("pipeline", ["pipeline review", "weekly pipeline", "sales pipeline"]),
    ("account-plan", ["account plan", "strategic account"]),
    ("nps", ["nps program", "customer nps", "net promoter"]),
    ("erm", ["enterprise risk framework", "risk framework"]),
    ("kri", ["kri dashboard", "risk indicator"]),
]


@dataclass
class RoutingDecision:
    """Result of routing a natural-language query through persona + workflow matching."""

    persona_slug: str
    workflow_slug: str
    confidence: float
    rationale: str
    alternative_personas: list[tuple[str, float]] = field(default_factory=list)
    alternative_workflows: list[tuple[str, float]] = field(default_factory=list)

    def __repr__(self) -> str:
        return (
            f"RoutingDecision(persona={self.persona_slug!r}, "
            f"workflow={self.workflow_slug!r}, conf={self.confidence:.2f})"
        )


def _normalize(text: str) -> str:
    """Lowercase + strip diacritics for diacritic-insensitive matching.

    Critical for Vietnamese queries where users may type "giám đốc" or "giam doc".
    Preserved pattern from legacy v0.1.0 router.
    """
    text = text.lower().strip()
    nfkd = unicodedata.normalize("NFKD", text)
    return "".join(c for c in nfkd if not unicodedata.combining(c))


def _score_persona(query_norm: str, persona_slug: str, keywords: list[str]) -> float:
    """Score a persona against a normalized query.

    Returns a confidence in [0.0, 1.0]. The scoring favors exact-keyword hits
    over slug-substring hits, and weights longer keywords more highly.
    """
    score = 0.0

    # Exact slug match is a strong signal.
    if persona_slug in query_norm:
        score += 0.6

    # Per-keyword scoring.
    for kw in keywords:
        kw_norm = _normalize(kw)
        if not kw_norm:
            continue
        if re.search(rf"\b{re.escape(kw_norm)}\b", query_norm):
            # Longer keywords get more weight (more specific signal).
            kw_weight = min(0.5, len(kw_norm) / 50.0)
            score += kw_weight
        elif kw_norm in query_norm:
            # Substring (no word-boundary) — half weight.
            kw_weight = min(0.25, len(kw_norm) / 100.0)
            score += kw_weight

    return min(1.0, score)


def _score_workflow(query_norm: str, workflow: WorkflowEntry) -> float:
    """Score a workflow against the (already persona-matched) query.

    Combines:
      - exact slug-substring match (strong signal, +0.6)
      - hyphen-tolerant slug-token overlap (the slug "architect-new-system" matches
        a query containing the tokens "architect" + "new" + "system" — the substring
        check fails on hyphen-vs-space but token overlap catches it)
      - purpose-text token overlap
      - workflow-keyword bank
    """
    score = 0.0

    # Slug match — exact substring is a strong signal (handles "monthly close" search
    # for an "monthly-close" slug only when the query literally contains the hyphen).
    wf_slug_norm = _normalize(workflow.slug)
    if wf_slug_norm and wf_slug_norm in query_norm:
        score += 0.6

    # Hyphen-tolerant slug-token overlap. Slugs like `architect-new-system` split into
    # ["architect", "new", "system"] and we score how many of those tokens appear in
    # the query as word-boundary matches.
    slug_tokens = [t for t in re.split(r"[-_]", wf_slug_norm) if len(t) >= 3]
    if slug_tokens:
        query_tokens_set = set(re.findall(r"\w{3,}", query_norm))
        slug_hits = sum(1 for t in slug_tokens if t in query_tokens_set)
        if slug_hits:
            # Full hit (every slug token present) gives 0.7; partial scales down.
            slug_token_score = 0.7 * (slug_hits / len(slug_tokens))
            score += slug_token_score

    # Purpose-text overlap.
    purpose_norm = _normalize(workflow.purpose)
    if purpose_norm:
        purpose_tokens = set(re.findall(r"\w{4,}", purpose_norm))
        query_tokens = set(re.findall(r"\w{4,}", query_norm))
        if purpose_tokens and query_tokens:
            overlap = len(purpose_tokens & query_tokens) / len(query_tokens)
            score += min(0.3, overlap * 0.5)

    # Workflow-keyword bank.
    for marker, phrases in _WORKFLOW_KEYWORDS:
        if marker in wf_slug_norm:
            for phrase in phrases:
                if _normalize(phrase) in query_norm:
                    score += 0.2
                    break

    return min(1.0, score)


def route(
    query: str,
    personas: list[PersonaEntry],
    *,
    persona_threshold: float = 0.5,
    workflow_threshold: float = 0.5,
) -> RoutingDecision | None:
    """Route a natural-language query to a (persona, workflow) pair.

    Two-stage with a domain-language fallback:

      Stage 1 (persona-first): score every persona against the query's keyword
      bank. If a persona scores ≥ `persona_threshold`, restrict workflow
      matching to that persona's workflows.

      Stage 2 fallback (workflow-first / domain-language path per MODULE.md §1.1):
      if no persona clears the threshold, score every workflow across all
      personas. The persona is implied by the workflow with the strongest
      domain-match. This handles queries like "Architect a new system" where
      the persona signal ("CTO") is absent but the workflow signal
      ("architect-new-system") is overwhelming.

    Args:
        query: free-form English (or Vietnamese-diacritic-aware) request.
        personas: list of discovered personas from `discover_personas()`.
        persona_threshold: min confidence to take the persona-first path.
        workflow_threshold: min confidence to accept a workflow match.

    Returns:
        Best `RoutingDecision` with workflow confidence ≥ workflow_threshold,
        OR None if no candidate clears the threshold. The alternatives list is
        always populated with the top-3 next-best candidates.
    """
    query_norm = _normalize(query)
    if not query_norm:
        return None

    active_personas = [p for p in personas if not p.is_extinct and p.has_workflows]
    if not active_personas:
        return None

    # Stage 1 — score personas against the keyword bank.
    persona_scores: list[tuple[str, float]] = []
    for persona in active_personas:
        keywords = _PERSONA_KEYWORDS.get(persona.slug, [persona.slug])
        sc = _score_persona(query_norm, persona.slug, keywords)
        if sc > 0:
            persona_scores.append((persona.slug, sc))
    persona_scores.sort(key=lambda x: x[1], reverse=True)

    chosen_personas: list[PersonaEntry]
    persona_path_used: str

    if persona_scores and persona_scores[0][1] >= persona_threshold:
        # Persona-first path — restrict to top-3 personas (handles ambiguous cases
        # where the strongest persona doesn't have the best workflow).
        top_persona_slugs = [s for s, _ in persona_scores[:3]]
        chosen_personas = [p for p in active_personas if p.slug in top_persona_slugs]
        persona_path_used = "persona-first"
    else:
        # Domain-language fallback — score workflows across every persona.
        chosen_personas = active_personas
        persona_path_used = "domain-language fallback"

    # Stage 2 — workflow matching within chosen personas.
    workflow_scores: list[tuple[PersonaEntry, WorkflowEntry, float]] = []
    for persona in chosen_personas:
        for wf in discover_workflows(persona):
            sc = _score_workflow(query_norm, wf)
            if sc > 0:
                workflow_scores.append((persona, wf, sc))
    workflow_scores.sort(key=lambda x: x[2], reverse=True)

    if not workflow_scores:
        return None

    top_persona, top_wf, top_wf_conf = workflow_scores[0]
    if top_wf_conf < workflow_threshold:
        return None

    # Combined confidence — use the persona score if it cleared the threshold,
    # otherwise fall back to the workflow score alone (domain-language path).
    persona_conf = next((sc for s, sc in persona_scores if s == top_persona.slug), 0.0)
    if persona_conf >= persona_threshold:
        combined_conf = (persona_conf * top_wf_conf) ** 0.5
    else:
        # Domain-language match — weight slightly down since persona is implied.
        combined_conf = top_wf_conf * 0.85

    rationale = (
        f"[{persona_path_used}] persona {top_persona.slug!r} scored "
        f"{persona_conf:.2f}; workflow {top_wf.slug!r} scored {top_wf_conf:.2f}"
    )

    return RoutingDecision(
        persona_slug=top_persona.slug,
        workflow_slug=top_wf.slug,
        confidence=combined_conf,
        rationale=rationale,
        alternative_personas=persona_scores[1:4],
        alternative_workflows=[(wf.slug, sc) for _, wf, sc in workflow_scores[1:4]],
    )


def score_one_off(
    query: str,
    personas: list[PersonaEntry],
    *,
    persona_threshold: float = 0.0,
    workflow_threshold: float = 0.0,
) -> list[RoutingDecision]:
    """Return sorted rule-score candidates for a single query.

    FR-CUO-101's supervisor needs the Phase 1 scorer unchanged but as a ranked
    list rather than a single terminal decision. This function shares the same
    primitive scoring helpers as `route()` and keeps deterministic ordering for
    replay-equivalence tests.
    """
    query_norm = _normalize(query)
    if not query_norm:
        return []

    active_personas = [p for p in personas if not p.is_extinct and p.has_workflows]
    if not active_personas:
        return []

    persona_scores: list[tuple[str, float]] = []
    for persona in active_personas:
        keywords = _PERSONA_KEYWORDS.get(persona.slug, [persona.slug])
        sc = _score_persona(query_norm, persona.slug, keywords)
        if sc > 0:
            persona_scores.append((persona.slug, sc))
    persona_scores.sort(key=lambda x: (-x[1], x[0]))

    candidates: list[RoutingDecision] = []
    for persona in active_personas:
        persona_conf = next((sc for s, sc in persona_scores if s == persona.slug), 0.0)
        for wf in discover_workflows(persona):
            wf_conf = _score_workflow(query_norm, wf)
            if wf_conf <= 0:
                continue
            if persona_conf > 0 and persona_conf >= persona_threshold:
                combined_conf = (persona_conf * wf_conf) ** 0.5
                path = "persona-first"
            else:
                combined_conf = wf_conf * 0.85
                path = "domain-language fallback"
            if combined_conf < workflow_threshold:
                continue
            candidates.append(
                RoutingDecision(
                    persona_slug=persona.slug,
                    workflow_slug=wf.slug,
                    confidence=combined_conf,
                    rationale=(
                        f"[{path}] persona {persona.slug!r} scored "
                        f"{persona_conf:.2f}; workflow {wf.slug!r} scored {wf_conf:.2f}"
                    ),
                    alternative_personas=persona_scores[1:4],
                    alternative_workflows=[],
                )
            )

    candidates.sort(key=lambda d: (-d.confidence, d.persona_slug, d.workflow_slug))
    return candidates
