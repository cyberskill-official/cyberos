"""LLM-as-judge verifier: one call, a rubric, a 0-1 score + pass/fail.

The one multi-agent pattern with strong *coding* evidence is a separate verifier
agent reviewing a diff against the spec — "never validate your own code in the
same context window" (Augment Intent; Anthropic orchestrator-worker). Anthropic's
research post found a *single* judge call scoring 0.0-1.0 against an explicit
rubric "most consistent and aligned with human judgements"; adding judges did not
help. This module is that single call, with:

  * a pluggable backend (so the judge runs in a FRESH context / different model),
  * robust JSON parsing (code-fence tolerant),
  * fail-closed behavior — an unparseable judgment never passes.

It is deliberately NOT used alone; see scorermix.py for the 60/30/10 blend.
"""

from __future__ import annotations

import json
import math
import re
from collections.abc import Callable
from dataclasses import dataclass, field

__all__ = [
    "Criterion",
    "Rubric",
    "VerifierResult",
    "Verifier",
    "VerifierError",
    "StubBackend",
    "code_review_rubric",
    "build_prompt",
    "parse_judgment",
]

# A backend is any callable mapping a prompt to the model's raw text response.
JudgeBackend = Callable[[str], str]


class VerifierError(ValueError):
    """Raised when a judgment cannot be parsed (-> fail-closed)."""


@dataclass(frozen=True)
class Criterion:
    name: str
    description: str = ""
    weight: float = 1.0
    required: bool = False  # if True, a failure fails the whole verification


@dataclass(frozen=True)
class Rubric:
    criteria: list[Criterion]
    pass_threshold: float = 0.7  # weighted-score bar for an overall pass

    def names(self) -> list[str]:
        return [c.name for c in self.criteria]

    def total_weight(self) -> float:
        return sum(c.weight for c in self.criteria) or 1.0


def code_review_rubric() -> Rubric:
    """Default rubric for reviewing a code diff against a spec."""
    return Rubric(
        criteria=[
            Criterion(
                "spec_adherence",
                "Implements exactly what the task/spec requires.",
                2.0,
                required=True,
            ),
            Criterion("correctness", "Logic is correct; edge cases handled.", 2.0, required=True),
            Criterion("no_scope_creep", "No unrelated or speculative changes.", 1.0),
            Criterion(
                "test_quality", "Tests are meaningful, not tautological; cover the change.", 1.5
            ),
            Criterion(
                "security",
                "No secret leakage, injection, or unsafe ops introduced.",
                1.5,
                required=True,
            ),
        ],
        pass_threshold=0.75,
    )


def build_prompt(artifact: str, rubric: Rubric, spec: str = "") -> str:
    """Construct the judge prompt. Asks for strict JSON only."""
    crit_lines = "\n".join(
        f"- {c.name} (weight {c.weight}{', REQUIRED' if c.required else ''}): {c.description}"
        for c in rubric.criteria
    )
    schema = (
        '{"criteria": {"<name>": {"score": <0.0-1.0>, "pass": <bool>, "note": "<short>"}}, '
        '"summary": "<one line>"}'
    )
    return (
        "You are a strict, independent code reviewer. Score the ARTIFACT against the "
        "RUBRIC. Judge only what is present; do not assume unseen context.\n\n"
        f"SPEC:\n{spec or '(see artifact)'}\n\n"
        f"RUBRIC:\n{crit_lines}\n\n"
        f"ARTIFACT:\n{artifact}\n\n"
        f"Respond with ONLY this JSON (no prose, no code fence):\n{schema}\n"
    )


def _extract_json(text: str) -> dict:
    """Pull the first JSON object out of a model response (fence/prose tolerant)."""
    fenced = re.search(r"```(?:json)?\s*(\{.*?\})\s*```", text, re.DOTALL)
    candidate = fenced.group(1) if fenced else None
    if candidate is None:
        start = text.find("{")
        end = text.rfind("}")
        if start == -1 or end == -1 or end <= start:
            raise VerifierError("no JSON object found in judge response")
        candidate = text[start : end + 1]
    try:
        data = json.loads(candidate)
    except json.JSONDecodeError as e:
        raise VerifierError(f"judge response was not valid JSON: {e}") from e
    if not isinstance(data, dict):
        raise VerifierError("judge JSON root is not an object")
    return data


@dataclass
class VerifierResult:
    passed: bool
    overall_score: float
    per_criterion: dict[str, dict]
    summary: str = ""
    raw: str = ""
    error: str = ""

    def to_dict(self) -> dict:
        return {
            "passed": self.passed,
            "overall_score": round(self.overall_score, 6),
            "per_criterion": self.per_criterion,
            "summary": self.summary,
            "error": self.error,
        }


def parse_judgment(text: str, rubric: Rubric) -> VerifierResult:
    """Parse + score a judge response against the rubric. Fail-closed on bad input."""
    data = _extract_json(text)  # raises VerifierError -> caller fails closed
    crit = data.get("criteria", {})
    if not isinstance(crit, dict):
        raise VerifierError("'criteria' must be an object")

    per: dict[str, dict] = {}
    weighted = 0.0
    required_failed = False
    for c in rubric.criteria:
        entry = crit.get(c.name, {}) if isinstance(crit.get(c.name), dict) else {}
        try:
            score = float(entry.get("score", 0.0))
        except (TypeError, ValueError):
            score = 0.0
        # A non-finite judge score (json.loads accepts NaN/Infinity) must fail to 0.0,
        # not inflate to 1.0 via max/min — a malformed/adversarial judge fails closed.
        if not math.isfinite(score):
            score = 0.0
        score = max(0.0, min(1.0, score))
        # Only honor an EXPLICIT boolean `pass`; a truthy string like "no" must not pass.
        raw_pass = entry.get("pass")
        passed = raw_pass if isinstance(raw_pass, bool) else score >= 0.5
        per[c.name] = {"score": score, "pass": passed, "note": str(entry.get("note", ""))}
        weighted += c.weight * score
        if c.required and not passed:
            required_failed = True

    overall = weighted / rubric.total_weight()
    passed = (overall >= rubric.pass_threshold) and not required_failed
    return VerifierResult(
        passed=passed,
        overall_score=overall,
        per_criterion=per,
        summary=str(data.get("summary", "")),
        raw=text,
    )


@dataclass
class Verifier:
    """Runs one judge call in a fresh backend context and scores it.

    ``backend`` must be a callable ``str -> str``. Supply an adapter that targets a
    DIFFERENT context/model than the one that wrote the code.
    """

    backend: JudgeBackend
    rubric: Rubric = field(default_factory=code_review_rubric)

    def verify(self, artifact: str, spec: str = "") -> VerifierResult:
        prompt = build_prompt(artifact, self.rubric, spec)
        try:
            raw = self.backend(prompt)
        except Exception as e:  # backend/network failure -> fail closed
            return VerifierResult(False, 0.0, {}, summary="", raw="", error=f"backend error: {e}")
        try:
            return parse_judgment(raw, self.rubric)
        except VerifierError as e:
            return VerifierResult(False, 0.0, {}, summary="", raw=raw, error=str(e))


@dataclass
class StubBackend:
    """Deterministic backend for tests/demos: returns a fixed response or JSON-encodes a dict."""

    response: str | dict

    def __call__(self, prompt: str) -> str:
        return self.response if isinstance(self.response, str) else json.dumps(self.response)
