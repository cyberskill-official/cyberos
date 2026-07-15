---
id: TASK-SKILL-112
title: "`acceptance/TRIGGER_TESTS.md` convention — positive + negative trigger phrases verified against the supervisor classifier"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: SKILL
priority: p1
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-SKILL-111, TASK-CUO-101, TASK-CUO-103, TASK-SKILL-103]
depends_on: [TASK-SKILL-103, TASK-CUO-101]
blocks: []

source_pages:
  - modules/skill/README.md#part-13--validate--debug
  - modules/skill/ANTHROPIC_GUIDE_DIGEST.md#51--confirmed-gaps
  - modules/skill/_template/author/acceptance/README.md
  - modules/skill/_template/audit/acceptance/README.md
source_decisions:
  - DEC-055 (acceptance auto-pause at <40% over 7 days)
  - DEC-091 (host-portability — CCSM is source of truth)
  - DEC-180 (frontmatter declares memory scopes + tool requirements)

language: yaml + markdown + python (CUO supervisor) + rust (broker validator)
service: modules/cuo/cuo/ (router smoke test) + modules/skill/task-audit/ (RUBRIC entry) + modules/skill/_template/ (scaffold)
new_files:
  - modules/skill/_template/author/acceptance/TRIGGER_TESTS.md
  - modules/skill/_template/audit/acceptance/TRIGGER_TESTS.md
  - modules/skill/task-author/acceptance/TRIGGER_TESTS.md
  - modules/skill/task-audit/acceptance/TRIGGER_TESTS.md
  - modules/skill/product-requirements-document-author/acceptance/TRIGGER_TESTS.md
  - modules/cuo/cuo/trigger_tests.py
  - modules/cuo/tests/test_trigger_tests.py
modified_files:
  - modules/skill/_template/author/acceptance/README.md                # cross-link to TRIGGER_TESTS.md
  - modules/skill/_template/audit/acceptance/README.md                 # cross-link to TRIGGER_TESTS.md
  - modules/skill/task-audit/RUBRIC.md                      # add FM-113 (trigger-tests-present)
  - Task-audit skill        # §3.10 mentions trigger-tests rule
  - website docs (SKILL appendices)                                    # Part 13.2 validation pyramid grows a new tier (Layer 1.5: triggering tests); Part 24.1 self-test checklist adds row
  - website docs (SKILL Appendix J)                                    # §6.2 status badge updates when task ships
  - website docs (CUO appendices)                                      # documents the trigger-tests smoke test path
allowed_tools:
  - file_read: modules/**, docs/tasks/skill/**
  - file_write: modules/skill/{_template,task-audit,task-author/acceptance,task-audit/acceptance,product-requirements-document-author/acceptance}, modules/cuo/{cuo,tests}, docs/tasks/skill/**
  - bash: cd modules/cuo && python -m pytest tests/test_trigger_tests.py
disallowed_tools:
  - require backfill of TRIGGER_TESTS.md across all 104 pairs in one commit batch (lazy backfill per §1 #11)
  - block CUO supervisor from booting if a skill lacks TRIGGER_TESTS.md (graceful degradation per §1 #9)
  - auto-generate trigger phrases by inferring from the description (TASK-SKILL-111 covers description; this task validates separately)

effort_hours: 10
subtasks:
  - "1.0h: _template/author/acceptance/TRIGGER_TESTS.md scaffold (with stub fixtures + format guide)"
  - "1.0h: _template/audit/acceptance/TRIGGER_TESTS.md scaffold (analogous)"
  - "0.5h: _template/{author,audit}/acceptance/README.md cross-link updates"
  - "1.5h: 3 exemplar TRIGGER_TESTS.md (task-author, task-audit, prd-author) — positive + negative phrases authored from real session logs"
  - "1.0h: task-audit/RUBRIC.md — add FM-113 trigger-tests-present rule + severity scheme (warning on draft, error on accepted+)"
  - "0.5h: task-audit skill §3.10 rule update"
  - "1.0h: modules/cuo/cuo/trigger_tests.py — Python module that loads a skill's TRIGGER_TESTS.md and runs each phrase against the router (uses CUO v3 supervisor's classify_act); returns pass/fail summary"
  - "1.5h: modules/cuo/tests/test_trigger_tests.py — happy path + parametric tests; integrates into existing 49/50 test suite"
  - "1.0h: README.md Part 13.2 + Part 24.1 updates"
  - "0.5h: ANTHROPIC_GUIDE_DIGEST.md §6.2 status update when CI passes"
  - "0.5h: documentation pass — cross-link TRIGGER_TESTS.md from TASK-SKILL-111 (description field carries the triggers; this task validates they actually route)"
risk_if_skipped: "Without triggering tests, the supervisor's classifier behaviour is silently coupled to skill descriptions. When TASK-SKILL-111 ships and an author widens a description to catch a new user phrasing, the wider phrasing may overlap with a sibling skill's triggers — the classifier picks the wrong skill 30% of the time. The regression surfaces only in production OBS after a week of <40% acceptance, which auto-pauses the skill per DEC-055. By then, customers have already seen wrong-skill responses. Inventory the failure: task-audit's description is enriched to mention 'audit existing tasks'; supervisor classifier now matches 'audit my task collection' to task-audit when the user actually meant chain-selector → task-author (re-author from refined source). The misrouting is silent — both skills' bodies look plausible to a non-expert user; the wrong-skill output ships, the user's intent isn't met, OBS records 'no correction' (user doesn't push back) and the acceptance metric is computed against the wrong baseline. TRIGGER_TESTS.md catches this at edit time: the moment the description widens, the fixture's negative-trigger row ('audit my task collection' MUST NOT match task-audit) fails the CI gate. Cost of the task ≈ 10 hours; cost of NOT shipping ≈ silent classifier regressions every fine-tune cycle, each surfacing 5-7 days late in OBS, each requiring root-cause analysis on production telemetry rather than design-time test failure."
---

## §1 — Description (BCP-14 normative)

This task establishes the `acceptance/TRIGGER_TESTS.md` convention — a per-skill fixture listing positive and negative trigger phrases that the supervisor classifier MUST match correctly. It closes the third Anthropic-guide test layer (triggering tests) that CyberOS currently lacks (per `modules/skill/ANTHROPIC_GUIDE_DIGEST.md` §5.1 Gap 2).

1. Every production skill (`status: accepted` or higher) **MUST** carry an `acceptance/TRIGGER_TESTS.md` file at `<skill-folder>/acceptance/TRIGGER_TESTS.md`. The file declares positive trigger phrases the supervisor classifier MUST route to this skill, and negative phrases the classifier MUST NOT route here.
2. The file's frontmatter **MUST** declare `skill_id` (kebab-case folder name) + `min_confidence: <float in [0.0, 1.0]>` (lowest classifier confidence accepted as a positive match — default `0.7`) + `classifier_version: <semver>` (the CUO router version the fixtures were authored against — protects against silent classifier regressions).
3. The file's body **MUST** contain two sections — `## Positive triggers (MUST route here)` and `## Negative triggers (MUST NOT route here)` — each containing a bulleted list of natural-language phrases (one phrase per bullet, ≤120 chars per phrase, in human-natural form).
4. Every production skill **MUST** carry **≥ 3 positive triggers** AND **≥ 3 negative triggers**. The 3-floor balances coverage against authoring cost. Skills with broader surface SHOULD scale up — `chain-selector` may carry 8-12 positive; `task-audit` may carry 5-8.
5. Positive triggers **MUST** be distinct **paraphrases** — not lexical variants. `"draft a task"` and `"author a task"` are duplicates (same verb meaning, same noun); `"draft a task"` and `"turn this PRD into a backlog"` are paraphrases (different surface, same intent). The validator rejects positive-trigger lists where any two phrases have edit distance ≤3 (single-character variants).
6. Negative triggers **MUST** be drawn from one of three pools: (a) phrases that route to a sibling skill in the same persona (e.g. `task-author` → `task-audit`), (b) phrases that route to a different persona (cross-persona disambiguation), (c) phrases that should NOT route to any skill (the supervisor returns "I'm not sure which workflow"). Each negative trigger SHOULD carry an inline `→` annotation pointing to the *expected* target skill or `→ none` for the "no workflow" case.
7. The CUO supervisor's classifier **MUST** be invocable via a Python entry point `cyberos.cuo.trigger_tests.run_for_skill(skill_path: Path) -> TriggerTestResult`. The function reads `acceptance/TRIGGER_TESTS.md`, invokes `cyberos.cuo.router.classify` per the existing v3 supervisor (TASK-CUO-101 / TASK-CUO-103) for each phrase, asserts the routing matches the fixture, and returns a `TriggerTestResult` with per-phrase verdicts.
8. The CI gate **MUST** run `cyberos.cuo.trigger_tests.run_all()` as part of every `cuo` test invocation. Failures block merge for any skill at `status: accepted` or higher. Skills at `status: draft` produce CI warnings only.
9. The CUO supervisor's boot path **MUST NOT** fail if a skill lacks `TRIGGER_TESTS.md`; the file is a build-time + CI-time artefact, not runtime. Missing file → supervisor logs `WARNING: skill <id> has no TRIGGER_TESTS.md (rule FM-113 fires only at audit time)` and continues. Graceful degradation is required so a partially-backfilled catalog doesn't break runtime.
10. The auditor rule **MUST** be `FM-113 trigger-tests-present` with severity `error` for production skills (`status: accepted` or higher); severity `warning` for `status: draft`. Issue verdict `needs_human` — the auditor never auto-generates trigger phrases (they require human knowledge of how users actually phrase requests).
11. Existing skills **MUST** be backfilled lazily — the rule fires only on `status: accepted` or higher; scaffold/draft skills are exempt. The next fine-tune cycle for each production skill brings it into compliance as a normal artefact-add (per `human_fine_tune.required_artifacts` extension).
12. The fixture file **SHOULD** carry a section `## Authoring notes` explaining where the trigger phrases came from. Three acceptable sources: (a) OBS-mined real user phrasings (when available), (b) Anthropic guide examples paraphrased for the skill, (c) author's a-priori intuition documented as such. Phrases sourced from (a) carry higher trust than (c).
13. When TASK-SKILL-111's description-format check passes (skill carries ≥2 trigger phrases in description) AND TASK-SKILL-112's TRIGGER_TESTS.md passes (classifier actually routes those phrases), the skill is **routing-stable**. The README Part 13.1 validation pyramid grows a new explicit layer between Layer 1 (mechanical) and Layer 2 (functional) — call it **Layer 1.5: triggering**. CI gates run Layer 1 → Layer 1.5 → Layer 2 in order.
14. The fixture **MUST** be byte-stable across runs against the same classifier_version. Re-running the same TRIGGER_TESTS.md against the same router version MUST produce identical pass/fail verdicts. (Non-determinism in the classifier is a separate concern surfaced by `deterministic_drift` anomaly signal per `_template/author/SKILL.md` line 105.)
15. The fixture's `min_confidence` floor **MUST** be ≥ the skill's `confidence_band.defer_below` (per `_template/author/SKILL.md` line 92). A skill that defers below 0.5 confidence cannot meaningfully be tested with `min_confidence: 0.3` — the test would accept a result the skill itself would reject. The validator enforces the relationship.

## §2 — Why this design (rationale for humans)

**Why a separate file rather than extending TASK-SKILL-111's description-format rule (§1 #1)?** Two different concerns. TASK-SKILL-111 enforces *what the description says*; TASK-SKILL-112 enforces *what the classifier actually does*. The description can carry the right phrases and the classifier can still route wrong (e.g. a recently-updated sibling skill's description now overlaps; the classifier's softmax tips the scales the wrong way). Separating them gives orthogonal regression-catch: 111 catches description regressions; 112 catches classifier regressions.

**Why ≥3 positive + ≥3 negative as the floor (§1 #4)?** Below 3 doesn't catch the paraphrase variance space (Anthropic's guide p. 15 example test suites have 3-5 each). Above 5 inflates authoring cost. The 3-floor is the smallest number that catches typical user phrasing variations without padding the fixture with near-duplicates.

**Why mandate paraphrase distinctness (§1 #5)?** A fixture with `"draft a task"` + `"author a task"` + `"write a task"` tests one phrasing three times — the classifier either matches all three or none. A fixture with `"draft a task"` + `"turn this PRD into a backlog"` + `"expand the spec into tasks"` tests three different paraphrases — the classifier might match all three (good — wide trigger surface) or only one (signals the description is too narrow). Edit-distance ≤3 catches single-character drift (`"audit"` vs `"audits"`); larger differences are accepted.

**Why three pools for negative triggers (§1 #6)?** Each pool catches a different failure class:
- Pool (a) — sibling routing: catches when a description widens and now overlaps a sister skill.
- Pool (b) — cross-persona: catches when a description's domain language overlaps another persona's surface (e.g. `chro` and `chief-people-officer` overlap).
- Pool (c) — no-skill: catches over-trigger on completely unrelated phrases ("What's the weather?").

The inline `→` annotation makes the expected target explicit and human-readable; without it, "negative trigger" is ambiguous.

**Why an inline `→ <skill>` annotation (§1 #6)?** Documentation as test. A reader sees `- "audit my task collection" → task-audit` and immediately understands "this skill must NOT match this phrase; the right skill is task-audit". The annotation is also machine-readable: the validator can assert the classifier *did* route to the named target, not just *didn't* route here. This catches false-positives where the classifier happened to route nowhere for an unrelated reason.

**Why a Python entry point in the CUO module rather than the broker (§1 #7)?** The classifier lives in CUO v3 (TASK-CUO-101). The classifier's stage-1 (filesystem-catalog domain-language fallback) and stage-2 (LLM router) both run in Python. Adding a sibling module `cyberos.cuo.trigger_tests` keeps the test fixture close to the thing under test. The broker (Rust) is the wrong layer for classifier testing — the broker validates frontmatter shape, not classifier behaviour.

**Why CI integration via `cuo` test suite (§1 #8)?** The CUO v3 test suite already exists (per memory `cuo Phase 4` — 49/50 tests pass). Adding `test_trigger_tests.py` to that suite keeps the gate close to the code being tested. The alternative — a standalone CI gate — would split the test infrastructure.

**Why graceful degradation on missing file (§1 #9)?** Lazy backfill means production skills accumulate TRIGGER_TESTS.md over weeks. If the supervisor refused to boot without 100% coverage, the rollout couldn't begin. Boot-time logging + audit-time enforcement is the same pattern AGENTS.md uses for memory files (warn + continue, audit later).

**Why severity scheme matches TASK-SKILL-111 (§1 #10)?** Both rules are about portability + classifier robustness; both protect production skills; both allow draft skills to iterate without friction. Consistent severity makes the auditor's behaviour predictable.

**Why no auto-generation of trigger phrases (§1 #10)?** Trigger phrases require *human* knowledge: what does this user demographic actually type? An LLM could synthesise plausible phrases but would systematically miss domain-specific jargon, regional phrasing (VN-locale users), and inside-baseball terminology. The auditor verdict is always `needs_human` for this reason.

**Why an `## Authoring notes` source-attribution section (§1 #12)?** Trust calibration. Phrases mined from real OBS logs are higher-trust than author intuition. When a fixture fails CI months later, the source-attribution tells the operator whether to update the fixture (intuition was wrong) or the description (real users have moved on). Without source-attribution, every CI failure looks the same.

**Why the new "Layer 1.5: triggering" tier in the validation pyramid (§1 #13)?** Anthropic's guide Chapter 3 lists three test areas (triggering / functional / performance). CyberOS's current pyramid has 1 (mechanical) / 2 (functional) / 3 (operational). Inserting 1.5 between 1 and 2 honours the Anthropic structure while keeping CyberOS's existing layers. The numbering 1.5 signals "between" — Layer 1 is structure, Layer 1.5 is routing, Layer 2 is behaviour.

**Why byte-stable verdicts across runs (§1 #14)?** Reproducibility is a CyberOS invariant (per `_template/audit/SKILL.md` line 136 `determinism.reproducible: true` for auditor skills). The trigger-test fixture inherits the same invariant: re-running against the same classifier_version produces identical results. Drift across runs surfaces `deterministic_drift` per the existing anomaly-signal framework.

**Why `min_confidence ≥ defer_below` (§1 #15)?** The skill's own confidence band declares what it considers "I'm sure enough to act"; testing against a lower bar would let phrases pass that the skill itself would refuse to act on. The validator enforces the relationship so authors can't accidentally weaken the test.

## §3 — API contract

### Fixture format — `acceptance/TRIGGER_TESTS.md`

```markdown
---
skill_id: task-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for task-author

## Positive triggers (MUST route here)

- "Turn this PRD into a backlog of tasks"
- "Draft a task for the new email-bounce handling"
- "Expand the spec into task markdowns"
- "Generate the task backlog from these source docs"

## Negative triggers (MUST NOT route here)

- "Audit this existing task for completeness" → task-audit
- "Has TASK-007 changed since the last audit?" → task-audit
- "Draft a tech spec from this task" → task-to-tech-spec
- "What's our company holiday schedule?" → none

## Authoring notes

- Positive triggers 1-3 derived from real OBS user phrasings observed during the v0.2.0 pilot (week 2026-W18).
- Positive trigger 4 is author intuition (canonical CyberOS phrasing).
- Negative triggers 1-2 derived from common confusion observed in pilot (users confused the author/audit pair).
- Negative trigger 3 derived from the planned task-to-tech-spec routing (TASK-SKILL-111 description-format makes this trigger unambiguous).
- Negative trigger 4 is a canonical "no skill" sanity case.
```

### Python entry point — `modules/cuo/cuo/trigger_tests.py`

```python
"""Trigger-test runner for TASK-SKILL-112.

Loads TRIGGER_TESTS.md fixtures and runs each phrase against the CUO router.
Asserts the routing matches the fixture's expectations.
"""

from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

import yaml

from cyberos.cuo.router import classify, ClassificationResult


@dataclass(frozen=True)
class TriggerTestRow:
    phrase: str
    expected_skill: str | None  # None = MUST NOT match this skill; "<skill_id>" = MUST route to this; "none" = MUST route nowhere
    is_positive: bool


@dataclass(frozen=True)
class TriggerTestResult:
    skill_id: str
    classifier_version: str
    rows: list[tuple[TriggerTestRow, ClassificationResult, bool]]  # (input, output, passed)

    @property
    def passed(self) -> bool:
        return all(passed for _, _, passed in self.rows)

    @property
    def failures(self) -> list[tuple[TriggerTestRow, ClassificationResult]]:
        return [(r, c) for r, c, p in self.rows if not p]


def load_fixture(path: Path) -> tuple[dict, list[TriggerTestRow]]:
    """Parse TRIGGER_TESTS.md into (frontmatter, rows)."""
    text = path.read_text(encoding="utf-8")
    # Split frontmatter
    if not text.startswith("---\n"):
        raise ValueError(f"{path}: missing frontmatter")
    end = text.index("\n---\n", 4)
    fm = yaml.safe_load(text[4:end])
    body = text[end + 5:]

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
            # Parse: `"<phrase>"` or `"<phrase>" → <target>`
            if section == "positive":
                phrase = content.strip('"').rstrip()
                # Strip trailing quote variants
                if phrase.endswith('"'):
                    phrase = phrase[:-1]
                rows.append(TriggerTestRow(
                    phrase=phrase,
                    expected_skill=fm["skill_id"],
                    is_positive=True,
                ))
            else:  # negative
                if "→" in content:
                    phrase_part, target_part = content.rsplit("→", 1)
                    phrase = phrase_part.strip().strip('"').strip()
                    target = target_part.strip()
                    expected = None if target == "none" else target
                else:
                    phrase = content.strip('"').strip()
                    expected = None
                rows.append(TriggerTestRow(
                    phrase=phrase,
                    expected_skill=expected,
                    is_positive=False,
                ))
    return fm, rows


def run_for_skill(skill_path: Path) -> TriggerTestResult:
    """Run TRIGGER_TESTS.md fixture for one skill against the classifier."""
    fixture_path = skill_path / "acceptance" / "TRIGGER_TESTS.md"
    if not fixture_path.exists():
        raise FileNotFoundError(f"No TRIGGER_TESTS.md at {fixture_path}")
    fm, rows = load_fixture(fixture_path)
    skill_id = fm["skill_id"]
    min_confidence = float(fm.get("min_confidence", 0.7))
    classifier_version = fm.get("classifier_version", "unknown")

    out_rows: list[tuple[TriggerTestRow, ClassificationResult, bool]] = []
    for row in rows:
        result = classify(row.phrase)
        if row.is_positive:
            # MUST route to this skill with confidence ≥ min_confidence
            passed = (result.skill_id == skill_id and result.confidence >= min_confidence)
        else:
            # MUST NOT route to this skill; SHOULD route to expected_skill (if specified)
            if row.expected_skill is None:
                passed = (result.skill_id != skill_id)
            else:
                passed = (result.skill_id == row.expected_skill and result.confidence >= min_confidence)
        out_rows.append((row, result, passed))

    return TriggerTestResult(
        skill_id=skill_id,
        classifier_version=classifier_version,
        rows=out_rows,
    )


def run_all(catalog_root: Path) -> dict[str, TriggerTestResult]:
    """Run every TRIGGER_TESTS.md in the catalog and return per-skill results."""
    results: dict[str, TriggerTestResult] = {}
    for fixture in catalog_root.glob("**/acceptance/TRIGGER_TESTS.md"):
        skill_path = fixture.parent.parent
        try:
            results[skill_path.name] = run_for_skill(skill_path)
        except FileNotFoundError:
            continue
    return results
```

### Auditor rule (added to `modules/skill/task-audit/RUBRIC.md`)

```markdown
### FM-113 — trigger-tests-present

**Statement:** Every production skill (`status: accepted` or higher) MUST carry `acceptance/TRIGGER_TESTS.md` with ≥3 positive + ≥3 negative triggers conforming to TASK-SKILL-112 §1. Phrases MUST be paraphrase-distinct (edit-distance > 3).

**Severity:** error on `status: accepted | building | shipped`; warning on `status: draft`.

**Auto-fix:** never (trigger phrases require human authorship — verdict `needs_human`).

**Check (deterministic):** invoke `python -m cyberos.cuo.trigger_tests <skill_path>`; if exit code non-zero OR the result's `.passed` is False, the rule fails. Specific sub-codes via process stderr: `fixture_missing` | `insufficient_positive` | `insufficient_negative` | `paraphrase_duplicate` | `classifier_routing_mismatch`.
```

## §4 — Acceptance criteria

1. **Fixture parses** — a well-formed `TRIGGER_TESTS.md` with 4 positive + 4 negative triggers → `load_fixture` returns `(frontmatter_dict, list[TriggerTestRow])` with 8 rows.
2. **Fixture rejected if missing frontmatter** — file without leading `---` → `ValueError`.
3. **Fixture rejected if < 3 positive triggers** — fixture with 2 positive → auditor rule FM-113 fires `insufficient_positive`.
4. **Fixture rejected if < 3 negative triggers** — fixture with 2 negative → auditor rule FM-113 fires `insufficient_negative`.
5. **Paraphrase-distinct check** — fixture with `"draft a task"` + `"draft a task"` (edit-distance 2) → FM-113 fires `paraphrase_duplicate`.
6. **Positive trigger routing PASS** — classifier returns this skill with confidence ≥ min_confidence → row pass.
7. **Positive trigger routing FAIL** — classifier returns different skill OR confidence < min_confidence → row fail.
8. **Negative trigger routing PASS — no expected target** — classifier returns NOT this skill (anything else) → row pass.
9. **Negative trigger routing PASS — expected target named** — classifier returns the named expected skill with confidence ≥ min_confidence → row pass; if classifier routes nowhere or to a third skill → row fail.
10. **Negative trigger routing FAIL** — classifier returns this skill (the negative trigger leaked) → row fail.
11. **`run_for_skill` integration** — given a valid skill folder, the function returns a `TriggerTestResult` whose `.passed` is True if all rows pass, False otherwise.
12. **`run_all` catalog walk** — given the catalog root, returns one `TriggerTestResult` per skill that has a `TRIGGER_TESTS.md`; skills without the file are skipped (not failures).
13. **Missing fixture on production skill triggers FM-113 audit issue** — `status: accepted` skill without `TRIGGER_TESTS.md` → auditor reports one FM-113 issue with severity `error`.
14. **Missing fixture on draft skill triggers FM-113 warning** — `status: draft` skill without `TRIGGER_TESTS.md` → auditor reports one FM-113 issue with severity `warning`.
15. **Backfill exemplar — task-author** — `modules/skill/task-author/acceptance/TRIGGER_TESTS.md` exists, 4 positive + 4 negative; `run_for_skill` returns `.passed = True` against current classifier (3.0.0-a4).
16. **Backfill exemplar — task-audit** — analogous; positive triggers ARE the negative triggers of the author exemplar (cross-reference verified).
17. **Backfill exemplar — prd-author** — analogous; positive triggers anchored on PRD-specific phrasings.
18. **`min_confidence ≥ defer_below` validator enforced** — fixture with `min_confidence: 0.3` for a skill with `confidence_band.defer_below: 0.5` → auditor reports FM-113 with `confidence_relationship_invalid`.
19. **Graceful degradation on supervisor boot** — CUO supervisor starts with one skill missing TRIGGER_TESTS.md → boots successfully with one WARNING log line; no crash.
20. **CI gate exit code** — `python -m cyberos.cuo.trigger_tests --catalog modules/skill/` → exit 0 if all production skills pass, exit 1 if any fail.
21. **Reproducibility** — running `run_all` twice against the same classifier_version → byte-identical results (allowing for the test execution's own ordering nondeterminism, which the test sorts).
22. **README Part 13.2 validation pyramid updated** — the diagram + table reflect the new Layer 1.5 (triggering).
23. **task-audit skill §3.10 entry added** — rule 41 references TASK-SKILL-112 and the TRIGGER_TESTS.md convention.

## §5 — Verification

```python
# modules/cuo/tests/test_trigger_tests.py
import pytest
from pathlib import Path
from cyberos.cuo.trigger_tests import (
    load_fixture, run_for_skill, run_all, TriggerTestRow, TriggerTestResult,
)

FIXTURES = Path(__file__).parent / "fixtures" / "trigger_tests"


def test_load_well_formed_fixture(tmp_path: Path):
    fixture = tmp_path / "TRIGGER_TESTS.md"
    fixture.write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for foo-author

## Positive triggers (MUST route here)

- "draft a foo"
- "turn this bar into a foo"
- "generate the foo backlog"

## Negative triggers (MUST NOT route here)

- "audit my foo" → foo-audit
- "draft a quux" → quux-author
- "what is the weather?" → none
""", encoding="utf-8")
    fm, rows = load_fixture(fixture)
    assert fm["skill_id"] == "foo-author"
    assert fm["min_confidence"] == 0.7
    assert len(rows) == 6
    positives = [r for r in rows if r.is_positive]
    negatives = [r for r in rows if not r.is_positive]
    assert len(positives) == 3
    assert len(negatives) == 3
    # Negative annotations parsed
    assert negatives[0].expected_skill == "foo-audit"
    assert negatives[2].expected_skill is None  # "→ none"


def test_load_missing_frontmatter(tmp_path: Path):
    fixture = tmp_path / "TRIGGER_TESTS.md"
    fixture.write_text("# No frontmatter here\n", encoding="utf-8")
    with pytest.raises(ValueError, match="missing frontmatter"):
        load_fixture(fixture)


def test_run_for_skill_all_pass(monkeypatch, tmp_path: Path):
    # Stub classify to return foo-author for positive phrases, foo-audit for negatives.
    from cyberos.cuo import router

    def fake_classify(phrase: str):
        if "audit" in phrase:
            return router.ClassificationResult(skill_id="foo-audit", confidence=0.9)
        if "quux" in phrase:
            return router.ClassificationResult(skill_id="quux-author", confidence=0.85)
        if "weather" in phrase:
            return router.ClassificationResult(skill_id=None, confidence=0.0)
        return router.ClassificationResult(skill_id="foo-author", confidence=0.85)

    monkeypatch.setattr(router, "classify", fake_classify)
    monkeypatch.setattr("cyberos.cuo.trigger_tests.classify", fake_classify)

    skill_dir = tmp_path / "foo-author"
    accept_dir = skill_dir / "acceptance"
    accept_dir.mkdir(parents=True)
    (accept_dir / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers (MUST route here)

- "draft a foo"
- "turn this bar into a foo"
- "generate the foo backlog"

## Negative triggers (MUST NOT route here)

- "audit my foo" → foo-audit
- "draft a quux" → quux-author
- "what is the weather?" → none
""", encoding="utf-8")

    result = run_for_skill(skill_dir)
    assert result.passed is True
    assert len(result.failures) == 0


def test_run_for_skill_positive_misroute(monkeypatch, tmp_path: Path):
    from cyberos.cuo import router

    def fake_classify(phrase: str):
        # Bug: positive phrase routes to wrong skill
        return router.ClassificationResult(skill_id="bar-author", confidence=0.95)

    monkeypatch.setattr("cyberos.cuo.trigger_tests.classify", fake_classify)

    skill_dir = tmp_path / "foo-author"
    accept_dir = skill_dir / "acceptance"
    accept_dir.mkdir(parents=True)
    (accept_dir / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers (MUST route here)

- "draft a foo"
- "turn this bar into a foo"
- "generate the foo backlog"

## Negative triggers (MUST NOT route here)

- "irrelevant" → none
""", encoding="utf-8")

    result = run_for_skill(skill_dir)
    assert result.passed is False
    assert len(result.failures) >= 3   # all positives fail


def test_run_for_skill_negative_leak(monkeypatch, tmp_path: Path):
    from cyberos.cuo import router

    def fake_classify(phrase: str):
        # Bug: negative phrase leaks into this skill
        return router.ClassificationResult(skill_id="foo-author", confidence=0.95)

    monkeypatch.setattr("cyberos.cuo.trigger_tests.classify", fake_classify)

    skill_dir = tmp_path / "foo-author"
    accept_dir = skill_dir / "acceptance"
    accept_dir.mkdir(parents=True)
    (accept_dir / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers (MUST route here)

- "draft a foo"
- "another"
- "third"

## Negative triggers (MUST NOT route here)

- "audit my foo" → foo-audit
- "draft a quux" → quux-author
- "weather?" → none
""", encoding="utf-8")

    result = run_for_skill(skill_dir)
    assert result.passed is False
    # Negative rows leak into this skill — all 3 negatives fail
    failures = [r for r, _, _ in result.failures]
    assert all(not r.is_positive for r in failures)


def test_missing_fixture_raises(tmp_path: Path):
    skill_dir = tmp_path / "foo-author"
    skill_dir.mkdir()
    with pytest.raises(FileNotFoundError):
        run_for_skill(skill_dir)


def test_run_all_walks_catalog(monkeypatch, tmp_path: Path):
    from cyberos.cuo import router

    def fake_classify(phrase: str):
        return router.ClassificationResult(skill_id="foo-author", confidence=0.85)

    monkeypatch.setattr("cyberos.cuo.trigger_tests.classify", fake_classify)

    # Two skills, one with TRIGGER_TESTS.md and one without
    (tmp_path / "foo-author" / "acceptance").mkdir(parents=True)
    (tmp_path / "foo-author" / "acceptance" / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers (MUST route here)
- "a"
- "b"
- "c"

## Negative triggers (MUST NOT route here)
- "x" → none
- "y" → none
- "z" → none
""", encoding="utf-8")
    (tmp_path / "bar-author").mkdir()

    results = run_all(tmp_path)
    assert "foo-author" in results
    assert "bar-author" not in results  # gracefully skipped


def test_paraphrase_distinct_check():
    # Edit-distance ≤3 between "draft a task" and "draft a task" → duplicate
    from cyberos.cuo.trigger_tests import are_paraphrase_distinct
    assert not are_paraphrase_distinct("draft a task", "draft a task")
    assert are_paraphrase_distinct("draft a task", "turn this PRD into a backlog")


def test_confidence_relationship_validator():
    # min_confidence ≥ defer_below required
    from cyberos.cuo.trigger_tests import validate_confidence_relationship
    # Skill with defer_below 0.5 — min_confidence must be ≥ 0.5
    assert validate_confidence_relationship(min_confidence=0.5, defer_below=0.5) is True
    assert validate_confidence_relationship(min_confidence=0.7, defer_below=0.5) is True
    assert validate_confidence_relationship(min_confidence=0.3, defer_below=0.5) is False
```

## §6 — Implementation skeleton

§3 covers the new files. Wiring:

1. `modules/cuo/cuo/trigger_tests.py` is added as a new module; re-exported from `modules/cuo/cuo/__init__.py`.
2. `modules/cuo/cli.py` gains a `trigger-tests <skill_path>` subcommand that wraps `run_for_skill`.
3. `modules/cuo/cli.py` gains a `trigger-tests-all` subcommand that wraps `run_all` and exits 0/1 based on catalog-wide pass.
4. `modules/skill/task-audit/RUBRIC.md` gains FM-113 entry; the auditor's 8-step loop (per `_template/audit/AUDIT_LOOP.md`) loads the rubric at start and runs FM-113 along with every other FM-NNN rule.
5. CI integration: `cd modules/cuo && python -m pytest tests/test_trigger_tests.py` is added to the existing pytest invocation (the test suite already runs on every PR — adding 7-9 new tests is additive).
6. Lazy backfill: as each production skill's next fine-tune cycle fires (via `human_fine_tune.signals_to_initiate`), the operator authors `acceptance/TRIGGER_TESTS.md` as part of the cycle's `required_artifacts` extension.

## §7 — Dependencies

**Depends on:**
- **TASK-SKILL-103** (frontmatter-extension) — provides the broker, `cyberos skill validate` CLI, and the structured-error machinery.
- **TASK-CUO-101** (CUO catalog scanner + 2-stage router) — provides `cyberos.cuo.router.classify` which the trigger-tests runner calls.

**Blocks:** none.

**Related:**
- **TASK-SKILL-111** (description-format enrichment) — complementary; 111 puts triggers in the description, 112 validates the classifier routes those triggers correctly.
- **TASK-CUO-103** (LLM router / Anthropic Messages API integration) — when the real LLM router is wired, the trigger-tests fixtures gate every classifier change.

**Cross-module:**
- **OBS module** (TASK-OBS-001..009) — when OBS ships, the per-phrase routing telemetry feeds back as candidate positive-trigger additions to TRIGGER_TESTS.md (future v0.3.0 feedback loop, sketched in TASK-SKILL-116).

## §8 — Example payloads

### Example 1 — fixture (task-audit) with full source attribution

```markdown
---
skill_id: task-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for task-audit

## Positive triggers (MUST route here)

- "Audit this task for completeness"
- "Has TASK-007 changed since the last audit?"
- "Tell me which tasks would fail acceptance today"
- "Re-run the rubric against this task collection"

## Negative triggers (MUST NOT route here)

- "Turn this PRD into a backlog of tasks" → task-author
- "Generate tasks from this spec" → task-author
- "Draft a tech spec from this task" → task-to-tech-spec
- "What's the team's holiday schedule?" → none

## Authoring notes

- Positive 1-3 derived from OBS week 2026-W18 (real user phrasings).
- Positive 4 is author intuition — covers the "re-audit" repeat case.
- Negative 1-2 from common author/audit confusion in pilot.
- Negative 3 from the planned task-to-tech-spec routing.
- Negative 4 is the canonical "no skill" sanity case.
```

### Example 2 — TriggerTestResult JSON (CI output)

```json
{
  "skill_id": "task-audit",
  "classifier_version": "3.0.0-a4",
  "passed": true,
  "rows": [
    {
      "phrase": "Audit this task for completeness",
      "expected_skill": "task-audit",
      "is_positive": true,
      "classifier_result": {"skill_id": "task-audit", "confidence": 0.91},
      "passed": true
    },
    {
      "phrase": "Turn this PRD into a backlog of tasks",
      "expected_skill": "task-author",
      "is_positive": false,
      "classifier_result": {"skill_id": "task-author", "confidence": 0.88},
      "passed": true
    }
  ]
}
```

### Example 3 — audit issue block (FM-113 firing — fixture missing)

```
ISSUE
id:              ISS-009
rule_id:         FM-113
severity:        error
category:        trigger_tests_fixture
location:        acceptance/TRIGGER_TESTS.md
evidence:        "file does not exist"
description:     "Production skill (status: accepted) MUST carry acceptance/TRIGGER_TESTS.md with ≥3 positive + ≥3 negative triggers per TASK-SKILL-112 §1. The auditor cannot proceed without it."
suggestion:      "Author acceptance/TRIGGER_TESTS.md. See modules/skill/_template/author/acceptance/TRIGGER_TESTS.md for the scaffold. Mine OBS logs (week-window) or use author intuition (document as such in ## Authoring notes)."
auto_fix_applied: false
resolution:      null
opened_at:       "2026-05-19T14:00:00Z"
updated_at:      "2026-05-19T14:00:00Z"
```

### Example 4 — audit issue block (FM-113 firing — paraphrase duplicate)

```
ISSUE
id:              ISS-010
rule_id:         FM-113
severity:        error
category:        paraphrase_duplicate
location:        acceptance/TRIGGER_TESTS.md
evidence:        "Positive triggers: \"draft a task\" and \"draft a task\" (edit-distance 2)"
description:     "Positive triggers must be paraphrase-distinct (edit-distance > 3 per TASK-SKILL-112 §1 #5). \"draft a task\" and \"draft a task\" differ by 1 character — same surface, same intent. Replace one with a distinct paraphrase."
suggestion:      "Replace \"draft a task\" with a paraphrase like \"turn this PRD into a backlog\" or \"author the task from this spec\". Aim for verb-or-noun substitution, not lexical variant."
auto_fix_applied: false
resolution:      null
opened_at:       "2026-05-19T14:00:00Z"
updated_at:      "2026-05-19T14:00:00Z"
```

## §9 — Open questions

**All resolved during authoring.**

Deferred to follow-up tasks:
- **TASK-SKILL-116** (placeholder — not yet specified): OBS-driven candidate trigger-phrase suggestions — mine real user phrasings; auto-propose additions to TRIGGER_TESTS.md as `refinement_proposal` envelopes for human review. Phase P2+, requires OBS to be live.
- **TASK-SKILL-117** (placeholder — not yet specified): trigger-test localisation — VN-locale users' phrasings drive a parallel `TRIGGER_TESTS.vi.md`. Phase P2+.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Production skill ships without TRIGGER_TESTS.md | Auditor rule FM-113 fires `fixture_missing` at next audit cycle | Skill cannot transition to `building` or `shipped` | Author the fixture; re-run audit |
| Fixture has < 3 positive triggers | FM-113 `insufficient_positive` | Audit issue; skill stays in current status | Author additional triggers |
| Fixture has < 3 negative triggers | FM-113 `insufficient_negative` | Audit issue | Author additional negatives across the 3 pools |
| Paraphrase duplicate in positive list | FM-113 `paraphrase_duplicate` | Audit issue | Replace one with a paraphrase-distinct alternative |
| Positive trigger misroutes (classifier picks another skill) | `run_for_skill` returns `passed=False`; CI gate fails | Block merge | Investigate: was the description widened (TASK-SKILL-111)? Is the sibling skill's description overlapping? Adjust descriptions or trigger phrases |
| Negative trigger leaks (classifier picks this skill) | `run_for_skill` returns `passed=False`; CI gate fails | Block merge | Same root cause as misroute; classifier overlap between siblings |
| Negative trigger expected target wrong | `run_for_skill` reports row failure; CI gate fails | Block merge | Either update fixture expectation OR fix the target skill's description (depends on which is wrong) |
| Classifier version mismatch (fixture authored against v3.0.0-a4; CI runs v3.0.0-a5) | Mismatch logged as warning; tests still run | Fixture stays valid until classifier behaviour changes | When classifier MAJOR-bumps, sweep all TRIGGER_TESTS.md; update `classifier_version`; re-verify |
| min_confidence too low (< defer_below) | Auditor rule FM-113 fires `confidence_relationship_invalid` | Audit issue | Raise min_confidence to ≥ defer_below |
| Supervisor boots without TRIGGER_TESTS.md for some skills | Supervisor logs WARNING and continues per §1 #9 | No runtime crash; audit catches it later | Lazy backfill via fine-tune cycle |
| Test execution non-determinism (parallel tests racing) | `run_all` produces different orderings → flaky CI | Sort results by skill_id before assertion | Test code uses `sorted(...)` on iteration |
| Author copy-pastes positive triggers from a sibling skill | Both skills' fixtures pass individually but classify wrong in production (the same phrase MUST positively route to one skill — the other's "MUST NOT" fails) | CI gate catches: the sibling's negative trigger row fails because the classifier picks this skill | Author differentiates trigger phrasing across siblings |
| Real OBS phrasing doesn't appear in TRIGGER_TESTS.md (gap coverage) | OBS tracking metric `untested_trigger_rate` shows phrasings the classifier sees but no fixture covers | Coverage gap; not a failure per se | Lazy backfill: add the phrase to TRIGGER_TESTS.md in next fine-tune cycle |
| Test execution timeout (classify is slow for some phrase) | pytest timeout marker fires; test fails as timeout | CI flag | Increase pytest timeout for the trigger-tests test class; investigate router slowness |

## §11 — Implementation notes

- **Why not auto-generate trigger phrases from descriptions?** Two reasons. (1) Trigger phrases reflect real user phrasings, not the author's idealised description prose; auto-generation from the description would systematically miss the gap that this task is designed to surface. (2) Auto-generated fixtures would silently pass even when the description is wrong — the test would only verify "the classifier matches what the description says", not "the classifier matches what real users type". Human authorship of trigger phrases is the load-bearing design choice.
- **Why edit-distance 3 for paraphrase-distinctness?** Empirical. Edit-distance 1-2 catches single-character drift; edit-distance 3 catches one short word change (`a` → `the`); edit-distance >3 catches genuine paraphrasing (different verbs, different nouns). The threshold can be tuned in a future PATCH — currently 3 is conservative.
- **Why ≥3 floor for both positive and negative?** Below 3 doesn't test the paraphrase-variance space. The fixture's value comes from covering distinct phrasings; 3 is the smallest set that demonstrates coverage. Above 5 inflates authoring cost; the rule never complains about >5.
- **Why three negative-trigger pools (sibling / cross-persona / none)?** Each pool catches a different failure class. A fixture with all three pools represented makes the test resilient against multiple regression types — a description widening (sibling pool), a persona surface expansion (cross-persona pool), or an overall classifier softening (none pool).
- **Why the `→ <target>` annotation on negative triggers?** Documentation + machine-readable assertion. The annotation lets the test verify the classifier routes to the *expected* target, not just *doesn't* route here. This catches false-positives where the classifier happens to return nowhere for unrelated reasons.
- **Why a Python module rather than a Rust crate?** The classifier lives in CUO (Python per TASK-CUO-101). Keeping the test infrastructure in the same language as the thing under test keeps the type contracts honest. The broker (Rust per TASK-SKILL-103) handles frontmatter validation; CUO handles classifier behaviour. Both are correct layers for their respective concerns.
- **Why graceful degradation on boot (§1 #9)?** Lazy backfill needs a long ramp. If the supervisor refused to boot without 100% TRIGGER_TESTS.md coverage, the first deploy would fail at the first un-backfilled skill. WARN + continue lets the catalog evolve organically; the auditor catches gaps at the right cadence (per-skill fine-tune).
- **Why CI integration via the existing CUO test suite rather than a new gate?** Two reasons. (1) The CUO test suite already runs on every PR (49/50 tests pass per the v3.0.0-a4 memory entry); adding 7-9 trigger-tests tests is additive. (2) A separate gate would mean two CI infrastructures to maintain; consolidation is cheaper.
- **The "Layer 1.5: triggering" naming is deliberate.** Inserting between existing Layer 1 (mechanical) and Layer 2 (functional) signals "this is between structure and behaviour — it's about *routing*". The half-step naming makes it clear this is an addition, not a renumbering of existing layers. README.md Part 13.1's validation pyramid diagram absorbs the new layer at the right height.
- **TRIGGER_TESTS.md vs trigger_tests.json — why Markdown?** Three reasons. (1) Authors write in Markdown for everything else (SKILL.md, INVARIANTS.md, RUBRIC.md); a JSON fixture would be the only non-Markdown artefact in a skill folder. (2) The `## Authoring notes` section is prose — JSON would force comments-as-key-values or YAML embedding. (3) The fixture is human-read more often than machine-read; readability matters. The parser strips Markdown lightly (just `## Section` headers + `- ` bullets), so the format is trivially machine-parseable too.
- **Why fix-attribution per phrase in `## Authoring notes`?** Trust calibration. A failing CI gate on a phrase sourced from "real OBS log week 2026-W18" is high-signal — real users typed this; if it's failing, the description widened past the safe envelope. A failing gate on a phrase sourced from "author intuition" is lower-signal — the author may have been wrong about how users phrase requests. The source-attribution lets the fine-tune cycle prioritise.
- **Cross-task coupling with TASK-SKILL-111**: TASK-SKILL-111 enforces description format (≥2 trigger phrases in frontmatter); TASK-SKILL-112 enforces classifier behaviour (those phrases actually route). Either can ship first. Once both are live, the contract is: description carries triggers, fixture asserts triggers route. If only 111 ships, classifier regressions still slip through. If only 112 ships, the description can ship with no triggers and the fixture still passes (because the body's `## When to invoke` triggers the supervisor). Together they close the loop.

---

*End of TASK-SKILL-112.*
