"""Trigger-test runner for TASK-SKILL-112.

Loads `<skill-folder>/acceptance/TRIGGER_TESTS.md` fixtures and asserts the
CUO supervisor's classifier routes positive triggers to the named skill and
negative triggers elsewhere.

Adapter note: CUO v3 routes at the workflow level (`route(query) → workflow_slug`),
not at the skill level. To bridge to SKILL.md-level trigger tests, we read the
routed workflow's `skill_chain` and check whether the expected skill is the
first invoked skill (the "entry skill" of that workflow).

Per TASK-SKILL-112 §1 #7. Used by CI gates + skill-bundle auditors.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

import yaml

# Lazy imports — route() loads the catalog which may not be needed for fixture-only tests.
# Importers can monkeypatch cuo.trigger_tests.classify for testing.


@dataclass(frozen=True)
class TriggerTestRow:
    phrase: str
    expected_skill: str | None  # None = MUST NOT route to fixture skill; str = MUST route to this skill (positive or named negative); "none" = MUST route nowhere
    is_positive: bool


@dataclass(frozen=True)
class SkillRoutingResult:
    """Adapter result from routing a phrase: the entry skill of the routed workflow."""
    skill_id: str | None       # None if no workflow matched OR matched workflow has no chain
    workflow_slug: str | None
    confidence: float

    def __repr__(self) -> str:
        return f"SkillRoutingResult(skill={self.skill_id!r}, workflow={self.workflow_slug!r}, conf={self.confidence:.2f})"


@dataclass(frozen=True)
class TriggerTestResult:
    skill_id: str
    classifier_version: str
    min_confidence: float
    rows: list[tuple[TriggerTestRow, SkillRoutingResult, bool]]  # (input, output, passed)

    @property
    def passed(self) -> bool:
        return all(passed for _, _, passed in self.rows)

    @property
    def failures(self) -> list[tuple[TriggerTestRow, SkillRoutingResult]]:
        return [(r, c) for r, c, p in self.rows if not p]

    def summary(self) -> str:
        if self.passed:
            return f"✓ {self.skill_id} — all {len(self.rows)} triggers route correctly"
        lines = [f"✗ {self.skill_id} — {len(self.failures)}/{len(self.rows)} trigger(s) failed:"]
        for row, result in self.failures:
            kind = "positive" if row.is_positive else "negative"
            expected = row.expected_skill or "none"
            actual = result.skill_id or "none"
            lines.append(f"    [{kind}] \"{row.phrase}\" → expected {expected}, got {actual} (conf {result.confidence:.2f})")
        return "\n".join(lines)


# ─── Fixture loader ─────────────────────────────────────────────────────────────

def load_fixture(path: Path) -> tuple[dict, list[TriggerTestRow]]:
    """Parse TRIGGER_TESTS.md → (frontmatter_dict, rows)."""
    if not path.exists():
        raise FileNotFoundError(f"TRIGGER_TESTS.md not found at {path}")
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        raise ValueError(f"{path}: missing leading '---' frontmatter delimiter")
    try:
        end = text.index("\n---\n", 4)
    except ValueError:
        raise ValueError(f"{path}: missing closing '---' frontmatter delimiter")
    fm = yaml.safe_load(text[4:end])
    body = text[end + 5:]

    if not isinstance(fm, dict):
        raise ValueError(f"{path}: frontmatter is not a YAML mapping")
    for required in ("skill_id", "min_confidence", "classifier_version"):
        if required not in fm:
            raise ValueError(f"{path}: missing required frontmatter key '{required}'")

    rows: list[TriggerTestRow] = []
    section = None
    for line in body.splitlines():
        stripped = line.strip()
        if stripped.startswith("## Positive"):
            section = "positive"
        elif stripped.startswith("## Negative"):
            section = "negative"
        elif stripped.startswith("## "):
            section = None
        elif stripped.startswith("- ") and section in ("positive", "negative"):
            content = stripped[2:].strip()
            if section == "positive":
                # Strip surrounding quotes
                phrase = _unquote(content)
                rows.append(TriggerTestRow(
                    phrase=phrase,
                    expected_skill=fm["skill_id"],
                    is_positive=True,
                ))
            else:  # negative
                if "→" in content:
                    phrase_part, target_part = content.rsplit("→", 1)
                    phrase = _unquote(phrase_part.strip())
                    target = target_part.strip()
                    expected = None if target == "none" else target
                else:
                    phrase = _unquote(content)
                    expected = None
                rows.append(TriggerTestRow(
                    phrase=phrase,
                    expected_skill=expected,
                    is_positive=False,
                ))
    return fm, rows


def _unquote(s: str) -> str:
    """Strip a single pair of surrounding double quotes."""
    s = s.strip()
    if s.startswith('"') and s.endswith('"') and len(s) >= 2:
        return s[1:-1]
    return s


# ─── Paraphrase-distinct check (Levenshtein distance) ───────────────────────────

def _levenshtein(a: str, b: str) -> int:
    if len(a) < len(b):
        return _levenshtein(b, a)
    if not b:
        return len(a)
    previous = list(range(len(b) + 1))
    for i, ca in enumerate(a):
        current = [i + 1]
        for j, cb in enumerate(b):
            cost = 0 if ca == cb else 1
            current.append(min(
                previous[j + 1] + 1,
                current[j] + 1,
                previous[j] + cost,
            ))
        previous = current
    return previous[-1]


def are_paraphrase_distinct(a: str, b: str, threshold: int = 3) -> bool:
    """Two phrases are paraphrase-distinct if their Levenshtein distance > threshold."""
    return _levenshtein(a.lower(), b.lower()) > threshold


def check_paraphrase_distinct(phrases: list[str]) -> list[tuple[str, str, int]]:
    """Return list of (phrase_a, phrase_b, edit_distance) for any pair that fails distinctness."""
    failures = []
    for i, a in enumerate(phrases):
        for b in phrases[i + 1:]:
            d = _levenshtein(a.lower(), b.lower())
            if d <= 3:
                failures.append((a, b, d))
    return failures


# ─── Confidence relationship validator ──────────────────────────────────────────

def validate_confidence_relationship(min_confidence: float, defer_below: float) -> bool:
    """TASK-SKILL-112 §1 #15: fixture min_confidence MUST be ≥ skill's defer_below."""
    return min_confidence >= defer_below


# ─── Classifier adapter ─────────────────────────────────────────────────────────

def classify(phrase: str) -> SkillRoutingResult:
    """Adapter: route a phrase via CUO's `route()` and return the entry skill of the resolved workflow.

    This is the function tests monkeypatch when isolating from the live classifier.
    """
    try:
        from cuo.core.router import route as _route
    except ImportError:
        return SkillRoutingResult(skill_id=None, workflow_slug=None, confidence=0.0)

    decision = _route(phrase)
    if decision is None or decision.confidence < 0.1:
        return SkillRoutingResult(skill_id=None, workflow_slug=None, confidence=0.0)

    # Resolve the entry skill from the routed workflow's skill_chain.
    try:
        from cuo.core.catalog import discover_workflows
        workflows = discover_workflows()
        for wf in workflows:
            if wf.persona == decision.persona_slug and wf.slug == decision.workflow_slug:
                if wf.skill_chain:
                    first = wf.skill_chain[0]
                    skill_id = first.get("skill") or first.get("id")
                    if skill_id:
                        return SkillRoutingResult(
                            skill_id=str(skill_id),
                            workflow_slug=decision.workflow_slug,
                            confidence=decision.confidence,
                        )
    except Exception:
        pass

    return SkillRoutingResult(
        skill_id=None,
        workflow_slug=decision.workflow_slug,
        confidence=decision.confidence,
    )


# ─── Runners ────────────────────────────────────────────────────────────────────

def run_for_skill(skill_path: Path) -> TriggerTestResult:
    """Load and run TRIGGER_TESTS.md for one skill bundle against the classifier."""
    fixture_path = skill_path / "acceptance" / "TRIGGER_TESTS.md"
    fm, rows = load_fixture(fixture_path)
    skill_id = fm["skill_id"]
    min_confidence = float(fm.get("min_confidence", 0.7))
    classifier_version = str(fm.get("classifier_version", "unknown"))

    out_rows: list[tuple[TriggerTestRow, SkillRoutingResult, bool]] = []
    for row in rows:
        result = classify(row.phrase)
        if row.is_positive:
            passed = (result.skill_id == skill_id and result.confidence >= min_confidence)
        else:
            if row.expected_skill is None:
                # Must NOT route to fixture skill (anywhere else, including nowhere, is fine)
                passed = (result.skill_id != skill_id)
            else:
                # Must route to the named expected target
                passed = (result.skill_id == row.expected_skill and result.confidence >= min_confidence)
        out_rows.append((row, result, passed))

    return TriggerTestResult(
        skill_id=skill_id,
        classifier_version=classifier_version,
        min_confidence=min_confidence,
        rows=out_rows,
    )


def run_all(catalog_root: Path) -> dict[str, TriggerTestResult]:
    """Walk catalog_root and run TRIGGER_TESTS.md for every skill that has one."""
    results: dict[str, TriggerTestResult] = {}
    for fixture in sorted(catalog_root.glob("**/acceptance/TRIGGER_TESTS.md")):
        skill_path = fixture.parent.parent
        try:
            results[skill_path.name] = run_for_skill(skill_path)
        except (FileNotFoundError, ValueError):
            continue
    return results


# ─── CLI entry point ────────────────────────────────────────────────────────────

def main(argv: list[str] | None = None) -> int:
    argv = argv if argv is not None else sys.argv[1:]
    if not argv or argv[0] in ("-h", "--help"):
        print("Usage: python -m cuo.trigger_tests <skill_path>")
        print("       python -m cuo.trigger_tests --catalog <catalog_root>")
        return 2

    if argv[0] == "--catalog":
        if len(argv) < 2:
            print("Error: --catalog requires a path argument")
            return 2
        results = run_all(Path(argv[1]))
        fail_count = 0
        for skill_id, result in results.items():
            print(result.summary())
            if not result.passed:
                fail_count += 1
        print()
        print(f"Total: {len(results)} skills tested, {fail_count} failed")
        return 0 if fail_count == 0 else 1

    skill_path = Path(argv[0])
    try:
        result = run_for_skill(skill_path)
        print(result.summary())
        return 0 if result.passed else 1
    except FileNotFoundError as e:
        print(f"Error: {e}")
        return 1
    except ValueError as e:
        print(f"Error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
