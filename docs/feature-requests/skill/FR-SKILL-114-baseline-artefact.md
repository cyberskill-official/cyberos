---
id: FR-SKILL-114
title: "BASELINE.md artefact at v0.x → v1.0 promotion — design-time performance baseline + with/without-skill comparison + token-budget transparency"
module: SKILL
priority: MAY
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-103, FR-SKILL-111, FR-SKILL-112, FR-SKILL-113]
depends_on: [FR-SKILL-103]
blocks: []

source_pages:
  - modules/skill/README.md#part-13--validate--debug
  - modules/skill/ANTHROPIC_GUIDE_DIGEST.md#64--medium-value-baselinemd-artefact-at-promotion
  - modules/skill/README.md#recipe-11
source_decisions:
  - DEC-055 (acceptance auto-pause at <40% over 7 days)
  - DEC-091 (host-portability — partner_connector flag gated on safety baseline)
  - DEC-092 (self-audit + auto-refinement — anomaly signals depend on baseline)

language: yaml + markdown + python (analyser) + rust (broker check)
service: modules/skill/_template/  (scaffold) + modules/skill/feature-request-audit/  (RUBRIC + audit rule) + modules/cuo/cuo/  (baseline analyser)
new_files:
  - modules/skill/_template/author/BASELINE.md
  - modules/cuo/cuo/baseline.py
  - modules/cuo/tests/test_baseline.py
  - modules/cuo/tests/fixtures/baseline-valid.md
  - modules/cuo/tests/fixtures/baseline-missing-section.md
  - modules/cuo/tests/fixtures/baseline-numeric-out-of-range.md
modified_files:
  - modules/skill/feature-request-audit/RUBRIC.md                      # add FM-114 (baseline-present-at-v1)
  - feature-request-audit skill        # §3.11 adds promotion-readiness rule
  - modules/skill/README.md                                            # Recipe 11 expanded; Part 13 validation pyramid mentions baseline
  - modules/skill/ANTHROPIC_GUIDE_DIGEST.md                            # §6.4 status update + path

allowed_tools:
  - file_read: modules/**, docs/feature-requests/skill/**
  - file_write: modules/skill/{_template,feature-request-audit,README.md,ANTHROPIC_GUIDE_DIGEST.md}, modules/cuo/{cuo,tests}, docs/feature-requests/skill/**
  - bash: cd modules/cuo && python -m pytest tests/test_baseline.py

disallowed_tools:
  - block v0.x → v1.0 promotion at v0.x stage (BASELINE.md is required AT promotion time, not before — drafts and v0.x skills are exempt)
  - auto-generate baseline numbers (numbers must come from real measurements per §1 #5)
  - bypass the operator-attestation requirement (the `attested_by` field requires a human signoff per §1 #8)

effort_hours: 8
sub_tasks:
  - "1.0h: _template/author/BASELINE.md scaffold — full template with all required fields + worked-example skeleton"
  - "0.5h: feature-request-audit/RUBRIC.md — add FM-114 baseline-present-at-v1 rule (severity: error on v1.0.0+; warning on v0.x; auto-fix: never — numbers must be measured)"
  - "0.5h: feature-request-audit skill §3.11 documentation-discipline expansion — add promotion-readiness sub-rule"
  - "1.0h: README.md Recipe 11 (Plan a skill promotion v0.x → v1.0) expanded to 8 paragraphs documenting the BASELINE.md artefact + the 4-check promotion gate (acceptance ≥80% × 4w + zero open refinement proposals + ≥3 acceptance fixtures + clear CHANGELOG driver)"
  - "1.0h: modules/cuo/cuo/baseline.py — Python parser + validator for BASELINE.md frontmatter + numeric range checks"
  - "1.0h: modules/cuo/tests/test_baseline.py — happy + 3 negative fixtures + numeric-bound + missing-section tests"
  - "0.5h: 3 fixture BASELINE.md files (valid / missing-section / out-of-range)"
  - "1.0h: ANTHROPIC_GUIDE_DIGEST.md §6.4 update — status badge + cross-reference to FR-SKILL-114"
  - "1.0h: integration with FR-SKILL-103 broker — broker reads `gated_until_phase: P3` (partner_connector skills require BASELINE.md before flag flips); broker rejects partner_connector: true on any skill lacking BASELINE.md if skill_version >= 1.0.0"
  - "0.5h: cross-referencing — add BASELINE.md path into v1.0-promotion checklist in feature-request-audit skill + into the v1.0-promotion section of every skill's CHANGELOG template"
risk_if_skipped: "Without BASELINE.md, v1.0 promotion is a 'vibes-based' decision per Anthropic guide Chapter 2 p. 9 — operators promote skills without quantitative justification. Two consequences: (1) partner-connector skills (per FR-SKILL-103 exposability) ship without trust calibration evidence; the `exposable_as.partner_connector: true` flag flips on skills whose with-vs-without performance was never measured. When a partner complains that the skill doesn't add value, there's no design-time baseline to defend the trust↔exposability link. (2) Operators have no record of why a skill was promoted; six months later, when the skill's acceptance rate drifts, there's no anchor to compare against ('was the 80% acceptance ever real, or was it always borderline?'). The artefact is small (one markdown file ~80-120 lines per promoted skill) but high-leverage at promotion time. Cost of the FR ≈ 8 hours including the validator and recipe expansion; cost of NOT shipping ≈ when the first partner connector ships, the trust calibration gap surfaces as a customer-visible blame ('why does this skill exist?') with no documented answer."
---

## §1 — Description (BCP-14 normative)

This FR introduces the `BASELINE.md` artefact — a per-skill design-time performance comparison written at v0.x → v1.0 promotion. It closes the Anthropic guide Chapter 2 "performance comparison" measurement layer (per `modules/skill/ANTHROPIC_GUIDE_DIGEST.md` §6.4) that CyberOS currently lacks at design time (it tracks the metrics in production via OBS but doesn't fossilise the promotion-time justification).

1. Every skill at `skill_version: 1.0.0` or higher **MUST** carry a `BASELINE.md` file at `<skill-folder>/BASELINE.md` (sibling of SKILL.md). The file documents the design-time performance comparison + the trust justification that earned the promotion.
2. The file's frontmatter **MUST** declare: `skill_id` (kebab-case folder name) + `baseline_version: <semver>` (the baseline doc's own version, independent of skill_version) + `baseline_measured_at: <ISO 8601 with timezone>` + `attested_by: <persona-id | human:<id>>` (who signed off) + `next_review_due: <ISO 8601>` (when this baseline should be re-measured — default +12 months).
3. The body **MUST** contain six sections in order: `## Workflow under test`, `## Without-skill baseline`, `## With-skill measurements`, `## Token-budget transparency`, `## Trust calibration`, `## Authoring notes`.
4. `## Without-skill baseline` **MUST** contain three numbered measurements: (a) **tool-call count** (how many tool invocations the workflow takes when a human runs it without this skill), (b) **token count** (total prompt + completion tokens consumed), (c) **failure rate** (% of attempts that ended in user redirection or abandonment). Each MUST cite the measurement methodology + sample size (n=N) + measurement window (date range).
5. `## With-skill measurements` **MUST** mirror the three measurements with the skill enabled, computed against the same sample set + window. Numbers MUST come from real measurements (not estimates); the `attested_by` field's signoff certifies this. Optionally a fourth measurement: **iteration count** (per-artefact auto-refinement iterations to convergence — relevant for audit-loop skills).
6. The skill is **eligible for v1.0 promotion** if all of: tool-call ratio (with / without) ≤ 0.7 (skill reduces tool calls by ≥30%); token ratio ≤ 0.7; failure-rate ratio ≤ 0.5 (skill cuts failure rate by ≥50%). These thresholds are calibrated to the Anthropic guide Chapter 2 p. 9 "Quantitative metrics" guidance (90% trigger rate, X tool calls, 0 failed API calls). Skills not meeting all three thresholds **MAY** still be promoted with explicit operator override + reason captured in `## Authoring notes`.
7. `## Token-budget transparency` **MUST** declare the skill's expected token budget per invocation (prompt + completion bounded; mean + 95th percentile). Operators reading the budget can decide whether to enable the skill in token-constrained sessions.
8. `## Trust calibration` **MUST** declare the skill's `confidence_band.default` value + the rationale (why 0.7 vs 0.9, etc.) + the `defer_below` trigger threshold + the empirical acceptance rate observed during the v0.x measurement window. This section is the audit anchor — six months later, when acceptance rate drifts, this section is the reference point.
9. `## Authoring notes` **MAY** contain: measurement caveats, edge cases not covered, known failure modes deferred to a v1.1 follow-up, attestation chain (who reviewed the baseline before the operator attestation).
10. The auditor rule **MUST** be `FM-114 baseline-present-at-v1` with severity `error` for `skill_version >= 1.0.0`; severity `info` (advisory) for v0.x. The rule fires at promotion-time CI gate, not at every fine-tune cycle.
11. The Rust broker (FR-SKILL-103) **MUST** check `BASELINE.md` presence when `skill_version >= 1.0.0` AND `exposable_as.partner_connector: true`. Missing baseline on a partner-flagged skill rejects load with `FrontmatterError::PartnerWithoutBaseline`.
12. The `attested_by` field's identity **MUST** be either a CyberOS persona id (e.g. `cuo-cpo` for CPO-attested baselines) or a `human:<id>` form (e.g. `human:stephen-cheng`). The attestation chain is part of the audit log (per AGENTS.md §7) — the `op:"baseline.attested"` row records the signoff.
13. `next_review_due` **MUST** be set; default +12 months. When the date passes, the auditor rule severity escalates from `info` (compliant) to `warning` (review overdue). After 24 months without re-measurement, the rule escalates to `error` (stale baseline).
14. Existing skills at v1.0+ (only `cuo/_shared/hello-world` at v1.0.0 today per `MODULE.md`) **MUST** be backfilled with a retrospective BASELINE.md as part of this FR's implementation. Future v1.0 promotions ship with BASELINE.md at the promotion-commit boundary; lazy backfill is not allowed (the artefact's whole point is to justify *this* promotion).
15. The validation pyramid in `modules/skill/README.md` Part 13.1 **MUST** be updated to reference BASELINE.md at the promotion gate (Layer 3 — operational telemetry now anchors against the design-time baseline). The pyramid diagram's third layer caption updates from "production telemetry" to "production telemetry vs design-time baseline".

## §2 — Why this design (rationale for humans)

**Why one artefact per skill rather than a catalog-wide spreadsheet (§1 #1)?** Per-skill BASELINE.md travels with the skill folder — when a skill is exported (per the AGENTS.md §11 plug-in contract), the baseline travels with it. Catalog-wide spreadsheets diverge from per-skill state over time. The single source of truth lives next to SKILL.md.

**Why six body sections (§1 #3)?** Each section answers a question a future operator asks: (a) what does this skill do? (Workflow under test), (b) what's the cost without it? (Without-skill), (c) what's the cost with it? (With-skill), (d) what's the runtime cost? (Token-budget), (e) why do we trust it? (Trust calibration), (f) what's not measured? (Authoring notes). Fewer sections lose context; more sections inflate authoring cost.

**Why the 30% / 30% / 50% threshold split (§1 #6)?** Empirical, derived from the Anthropic guide Chapter 3 baseline-comparison example (p. 16): "15 back-and-forth messages → 2 clarifying questions; 12,000 tokens → 6,000 tokens; 3 failed API calls → 0 failed API calls". The example shows ~87% reduction in tool calls, 50% in tokens, 100% in failures. CyberOS's thresholds (30% / 30% / 50%) are conservative — easier to meet, but still demonstrate real value. Stricter thresholds would reject legitimate-but-marginal skills; looser would let weak skills slip through. The thresholds are configurable per skill in §1 #6's override clause.

**Why operator override is permitted (§1 #6)?** Some skills earn promotion on dimensions the baseline doesn't quantify — for example, audit skills whose value is "catches violations a human would miss" rather than "reduces tool calls". The override captures the reason; the auditor records the override + reason in the audit chain.

**Why token-budget transparency (§1 #7)?** Operators running in token-constrained sessions (free tier, mobile contexts) need to know which skills are budget-aware. A skill claiming "saves 50% tokens vs baseline" might still consume 50,000 tokens — important to surface.

**Why explicit trust calibration section (§1 #8)?** This section answers "why is `confidence_band.default: 0.7` the right value for this skill?". Without it, six months later when acceptance drifts, the operator can't tell whether the original calibration was wrong or whether the underlying user behaviour changed. The section anchors the audit chain.

**Why FM-114 fires at promotion-time CI gate, not every fine-tune (§1 #10)?** Fine-tune is iterative; the baseline is a one-shot promotion artefact. Firing the rule on every fine-tune cycle would create noise (the baseline exists, the rule passes, every commit) without informational value. The rule fires when `skill_version` crosses from 0.x to 1.x — exactly the promotion event.

**Why partner_connector skills carry a stricter requirement (§1 #11)?** Per FR-SKILL-103's trust-exposability link, partner-connector skills face external scrutiny (partner SLAs, billing, tenancy). The baseline is the operator's defence: "this skill earns its position because of X measurements; here's the attestation chain". Without baseline, the trust-exposability gate is operator-vibes-based.

**Why attestation by persona OR human (§1 #12)?** Some baselines are attested by the responsible C-suite persona (`cuo-cpo` for product-owned skills); others by named humans (when persona attribution isn't clean). The two forms cover both governance models. The audit row records either form.

**Why a 12-month review cadence (§1 #13)?** Skill behaviour drifts over LLM model updates, training data shifts, and user-base evolution. Re-measurement every 12 months catches the drift. The warning/error escalation at 12/24 months gives operators a graceful runway: 12 months = "schedule a re-measurement"; 24 months = "this baseline is stale enough to block promotion-derivative actions".

**Why backfill existing v1.0+ skills (§1 #14)?** The artefact's value is *justifying this promotion*. A v1.0 skill without a baseline is a v1.0 skill that was never justified. Lazy backfill would defeat the purpose. Today only `cuo/_shared/hello-world` is at v1.0 (per MODULE.md) — the backfill is one skill, trivial cost.

**Why update the validation pyramid (§1 #15)?** Layer 3 (operational telemetry) now has a design-time anchor (BASELINE.md). The pyramid's caption update makes the dependency explicit — production metrics get their meaning by comparison to the baseline, not in isolation. Future operators reading README Part 13 understand the dependency without grepping.

## §3 — API contract

### BASELINE.md frontmatter contract

```yaml
---
skill_id: feature-request-author
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: cuo-cpo
next_review_due: 2027-05-19T00:00:00+07:00
---
```

### Body section contract (worked example)

```markdown
## Workflow under test

The workflow is "turn a PRD into a feature-request backlog". Human-baseline:
the operator reads the PRD, identifies candidate FRs by hand, writes each
FR markdown, runs the audit by hand. Skill-augmented: invoke
`feature-request-author` with the PRD path; it generates a manifest +
backlog, halts at PLAN approval, the operator approves, it writes the FRs.

## Without-skill baseline

**Measurement window:** 2026-04-15 → 2026-05-15 (4 weeks). **Sample size:** n=12
real PRD-to-backlog flows across cpo/cto/cmo personas.

| Measurement | Without skill (mean ± stddev) | Methodology |
|---|---|---|
| Tool-call count | 47 ± 12 | OBS `tool_call_count` per session; sessions filtered by `task_taxonomy: prd-to-backlog` |
| Token count | 84,000 ± 18,000 | OBS `tokens_total` per session |
| Failure rate | 33% | Sessions that ended with user "abort" or "reformulate" / total sessions (4 of 12) |

## With-skill measurements

Same window, same sample (n=12 the operators chose to re-run with the skill).

| Measurement | With skill (mean ± stddev) | Ratio vs baseline | Pass threshold |
|---|---|---|---|
| Tool-call count | 11 ± 3 | 0.23 | ✓ (≤0.7) |
| Token count | 22,000 ± 4,500 | 0.26 | ✓ (≤0.7) |
| Failure rate | 8% | 0.24 | ✓ (≤0.5) |
| Iteration count | 1.8 ± 0.6 | — | (audit-loop skill — info-only) |

**Verdict:** all three thresholds passed. Skill is v1.0-eligible per FR-SKILL-114 §1 #6.

## Token-budget transparency

Per-invocation token budget for `feature-request-author`:

- **Prompt tokens (mean):** 14,000
- **Prompt tokens (95th percentile):** 19,500
- **Completion tokens (mean):** 8,000
- **Completion tokens (95th percentile):** 13,000
- **Total ceiling guidance:** budget 40,000 tokens for a worst-case invocation.

Operators in token-constrained sessions (free tier) should enable only when
a backlog is genuinely needed.

## Trust calibration

`confidence_band.default: 0.7` chosen because:

- FR authoring is judgement work (`determinism.reproducible: false`).
- The 0.7 default balances "model is confident enough to act" (0.7) against
  "model defers on ambiguity" (≤0.5 triggers HITL).
- During the v0.x measurement window, the empirical acceptance rate was **84%**
  (10 of 12 batches accepted as-shipped). 84% > the DEC-055 auto-pause threshold
  of 40% by a wide margin; the skill is operationally stable.

`defer_below: 0.5` chosen because:

- Below 0.5, the model is genuinely uncertain about an FR claim.
- HITL surfaces the question to the operator — the alternative is silent
  fabrication (anti-pattern per `references/ANTI_FABRICATION.md`).

## Authoring notes

- Sample size n=12 is small. Caveats: the cpo persona contributed 8 of 12;
  the cto and cmo personas contributed 4. Generalisation to less-represented
  personas needs more data; recommend re-measuring in v1.1.
- Baseline measurement excluded one PRD that was malformed (no `## Summary`
  section); the skill correctly refused with `INPUTS_CHANGED`. Reported as
  an FR-AI-XXX follow-up for the input-validation pipeline.
- Attestation chain: measurement gathered by claude-opus-4-7 (this session)
  during sweep; reviewed by cuo-cpo; signed off by stephen-cheng (human).
```

### Auditor rule — addition to `modules/skill/feature-request-audit/RUBRIC.md`

```markdown
### FM-114 — baseline-present-at-v1

**Statement:** Skills at `skill_version >= 1.0.0` MUST carry `BASELINE.md` per FR-SKILL-114 §1. The artefact justifies the promotion + anchors future drift detection.

**Severity:**
- error on `skill_version >= 1.0.0`
- info on `skill_version < 1.0.0` (advisory — pre-promotion drafts may add BASELINE.md early)
- escalates to warning if `next_review_due` is overdue >0 days
- escalates to error if `next_review_due` is overdue >365 days (stale baseline)

**Auto-fix:** never (numbers must come from real measurements — verdict `needs_human`).

**Check (deterministic):** invoke `python -m cyberos.cuo.baseline <skill_path>`; exits 0 if valid, exits 1 with structured error otherwise. Sub-codes: `file_missing` | `frontmatter_invalid` | `section_missing` | `numeric_out_of_range` | `attested_by_invalid` | `review_overdue`.
```

### Python validator — `modules/cuo/cuo/baseline.py`

```python
"""Validate BASELINE.md artefacts per FR-SKILL-114."""

from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
import re
import yaml


REQUIRED_SECTIONS = [
    "## Workflow under test",
    "## Without-skill baseline",
    "## With-skill measurements",
    "## Token-budget transparency",
    "## Trust calibration",
    "## Authoring notes",
]
REQUIRED_FM_KEYS = {"skill_id", "baseline_version", "baseline_measured_at", "attested_by", "next_review_due"}
ATTESTOR_RE = re.compile(r"^(cuo-[a-z]+|human:[a-z][a-z0-9_-]*)$")


@dataclass(frozen=True)
class BaselineValidationResult:
    skill_id: str
    valid: bool
    issues: list[str]


def validate(path: Path) -> BaselineValidationResult:
    if not path.exists():
        return BaselineValidationResult("?", False, ["file_missing"])
    text = path.read_text(encoding="utf-8")
    issues: list[str] = []
    skill_id = "?"

    # 1. Parse frontmatter
    if not text.startswith("---\n"):
        return BaselineValidationResult("?", False, ["frontmatter_invalid: missing leading ---"])
    try:
        end = text.index("\n---\n", 4)
    except ValueError:
        return BaselineValidationResult("?", False, ["frontmatter_invalid: missing closing ---"])
    fm = yaml.safe_load(text[4:end])
    body = text[end + 5:]
    if not isinstance(fm, dict):
        return BaselineValidationResult("?", False, ["frontmatter_invalid: not a dict"])

    skill_id = fm.get("skill_id", "?")

    # 2. Required keys present
    missing = REQUIRED_FM_KEYS - set(fm.keys())
    if missing:
        issues.append(f"frontmatter_invalid: missing keys {sorted(missing)}")

    # 3. attested_by format
    attested_by = fm.get("attested_by", "")
    if attested_by and not ATTESTOR_RE.match(str(attested_by)):
        issues.append(f"attested_by_invalid: '{attested_by}'")

    # 4. next_review_due in future (warn) or past (escalate)
    try:
        due = datetime.fromisoformat(str(fm.get("next_review_due", "")))
        now = datetime.now(timezone.utc)
        if due.tzinfo is None:
            due = due.replace(tzinfo=timezone.utc)
        days_overdue = (now - due).days
        if days_overdue > 365:
            issues.append("review_overdue: stale baseline (>365 days)")
        elif days_overdue > 0:
            issues.append(f"review_overdue: {days_overdue} days")
    except (ValueError, TypeError):
        issues.append("frontmatter_invalid: next_review_due is not ISO 8601")

    # 5. Required body sections
    for section in REQUIRED_SECTIONS:
        if section not in body:
            issues.append(f"section_missing: '{section}'")

    return BaselineValidationResult(skill_id, len(issues) == 0, issues)
```

## §4 — Acceptance criteria

1. **Valid BASELINE.md passes** — fixture `baseline-valid.md` with all required frontmatter keys + 6 body sections + numeric measurements within thresholds → `validate` returns `valid=True`.
2. **Missing file rejected** — non-existent path → `valid=False`, issues includes `"file_missing"`.
3. **Missing frontmatter rejected** — file without `---` fences → `valid=False`, issues includes `frontmatter_invalid`.
4. **Missing required frontmatter key rejected** — fixture without `attested_by:` → `valid=False`, issues mentions the missing key.
5. **Invalid attested_by form rejected** — fixture with `attested_by: random-person` → `valid=False`, issues includes `attested_by_invalid`.
6. **Missing body section rejected** — fixture without `## Token-budget transparency` → `valid=False`, issues includes `section_missing`.
7. **Review overdue (>0 days) flagged as warning** — fixture with `next_review_due: 2024-01-01` → `valid=False`, issues includes `review_overdue: N days`.
8. **Review stale (>365 days) flagged as error** — fixture with `next_review_due: 2023-01-01` → `valid=False`, issues includes `review_overdue: stale baseline`.
9. **Auditor rule FM-114 fires on v1.0+ skill without BASELINE.md** — running auditor on a v1.0 skill missing BASELINE.md → audit reports one FM-114 issue (severity error, status needs_human).
10. **Auditor rule FM-114 fires as info on v0.x skill** — same input but `skill_version: 0.2.0` → audit reports one FM-114 issue (severity info — advisory).
11. **Broker rejects partner_connector v1.0+ without baseline** — load attempt → `FrontmatterError::PartnerWithoutBaseline`.
12. **Numeric threshold pass check** — fixture with tool-call ratio 0.5 (≤0.7) + token ratio 0.5 + failure ratio 0.4 → meets thresholds; promotion eligible.
13. **Numeric threshold fail check** — fixture with tool-call ratio 0.8 → does NOT meet threshold; promotion would require operator override + reason captured in Authoring notes.
14. **Backfill hello-world v1.0** — `cuo/_shared/hello-world/BASELINE.md` exists post-FR-ship; passes `validate`.
15. **README Recipe 11 expanded** — Recipe 11 in README.md mentions BASELINE.md + the 3-threshold check + the operator-override clause.
16. **feature-request-audit skill §3.11 entry added** — new sub-rule references FR-SKILL-114.
17. **Validation pyramid updated** — README.md Part 13 mentions BASELINE.md as the design-time anchor.
18. **Template scaffold present** — `_template/author/BASELINE.md` is a complete scaffold an author can copy.
19. **CI integration** — `python -m pytest modules/cuo/tests/test_baseline.py` runs as part of the existing CUO test suite.
20. **OTel span emitted** — `skill.baseline.validate` with attributes `skill_id`, `outcome`, `issue_count`.

## §5 — Verification

```python
# modules/cuo/tests/test_baseline.py
import pytest
from datetime import datetime, timezone, timedelta
from pathlib import Path
from cyberos.cuo.baseline import validate, BaselineValidationResult


FIXTURES = Path(__file__).parent / "fixtures"


def test_valid_baseline_passes():
    result = validate(FIXTURES / "baseline-valid.md")
    assert result.valid is True
    assert result.issues == []


def test_missing_file_rejected(tmp_path):
    result = validate(tmp_path / "nonexistent.md")
    assert result.valid is False
    assert "file_missing" in result.issues


def test_missing_frontmatter_rejected(tmp_path):
    p = tmp_path / "BASELINE.md"
    p.write_text("# No frontmatter here\n", encoding="utf-8")
    result = validate(p)
    assert result.valid is False
    assert any("frontmatter_invalid" in i for i in result.issues)


def test_missing_required_key_rejected(tmp_path):
    p = tmp_path / "BASELINE.md"
    p.write_text("""---
skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
next_review_due: 2027-05-19T00:00:00+07:00
---

## Workflow under test
## Without-skill baseline
## With-skill measurements
## Token-budget transparency
## Trust calibration
## Authoring notes
""", encoding="utf-8")
    # Missing: attested_by
    result = validate(p)
    assert result.valid is False
    assert any("attested_by" in i for i in result.issues)


def test_invalid_attested_by(tmp_path):
    p = tmp_path / "BASELINE.md"
    p.write_text("""---
skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: random-person
next_review_due: 2027-05-19T00:00:00+07:00
---

## Workflow under test
## Without-skill baseline
## With-skill measurements
## Token-budget transparency
## Trust calibration
## Authoring notes
""", encoding="utf-8")
    result = validate(p)
    assert result.valid is False
    assert any("attested_by_invalid" in i for i in result.issues)


def test_section_missing_rejected(tmp_path):
    p = tmp_path / "BASELINE.md"
    p.write_text("""---
skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: cuo-cpo
next_review_due: 2027-05-19T00:00:00+07:00
---

## Workflow under test
## Without-skill baseline
## With-skill measurements
## Trust calibration
## Authoring notes
""", encoding="utf-8")
    # Missing: ## Token-budget transparency
    result = validate(p)
    assert result.valid is False
    assert any("Token-budget transparency" in i for i in result.issues)


def test_review_overdue_flagged(tmp_path):
    p = tmp_path / "BASELINE.md"
    p.write_text("""---
skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2024-01-01T00:00:00+07:00
attested_by: cuo-cpo
next_review_due: 2024-06-01T00:00:00+07:00
---

## Workflow under test
## Without-skill baseline
## With-skill measurements
## Token-budget transparency
## Trust calibration
## Authoring notes
""", encoding="utf-8")
    result = validate(p)
    assert result.valid is False
    assert any("review_overdue" in i for i in result.issues)


def test_human_attestation_accepted(tmp_path):
    p = tmp_path / "BASELINE.md"
    p.write_text("""---
skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: human:stephen-cheng
next_review_due: 2027-05-19T00:00:00+07:00
---

## Workflow under test
## Without-skill baseline
## With-skill measurements
## Token-budget transparency
## Trust calibration
## Authoring notes
""", encoding="utf-8")
    result = validate(p)
    assert result.valid is True
```

## §6 — Implementation skeleton

§3 covers the surface. Wiring:

1. `modules/skill/_template/author/BASELINE.md` is a fully-populated scaffold an author copies + edits.
2. `modules/cuo/cuo/baseline.py` is added as a new module; re-exported from `modules/cuo/cuo/__init__.py`.
3. `modules/cuo/cli.py` gains a `validate-baseline <skill_path>` subcommand.
4. README Recipe 11 expanded from 3 paragraphs to 8 paragraphs covering the FR-SKILL-114 contract.
5. feature-request-audit skill §3.11 (documentation-discipline) adds the rule.
6. RUBRIC.md FM-114 added.

## §7 — Dependencies

**Depends on:**
- **FR-SKILL-103** (frontmatter-extension) — provides the broker that gates partner_connector skills on BASELINE.md presence.

**Blocks:** none.

**Related:**
- **FR-SKILL-111** (description trigger enrichment) — orthogonal; description is loaded by host classifier; baseline is read by promotion auditor.
- **FR-SKILL-112** (TRIGGER_TESTS.md) — orthogonal; routing test vs promotion artefact.
- **FR-SKILL-113** (XML-free frontmatter) — orthogonal; frontmatter shape vs new artefact.

**Cross-module:**
- **OBS module** — when OBS ships, the production telemetry feeds back into the baseline at next review (12-month cadence).

## §8 — Example payloads

### Example 1 — valid BASELINE.md (worked example)

(See §3 body section contract — that's the worked example.)

### Example 2 — audit issue (FM-114 firing on v1.0+ skill without baseline)

```
ISSUE
id:              ISS-013
rule_id:         FM-114
status:          needs_human
severity:        error
category:        baseline_missing
location:        BASELINE.md (file does not exist)
evidence:        "Skill at skill_version: 1.0.0 has no BASELINE.md sibling file."
description:     "Skills at v1.0+ MUST carry BASELINE.md per FR-SKILL-114 §1. The artefact justifies the v0.x → v1.0 promotion."
suggestion:      "Copy modules/skill/_template/author/BASELINE.md to <skill_path>/BASELINE.md; measure tool-call count / token count / failure rate without and with the skill enabled; populate the 6 body sections; attest by persona-id or human:<id>."
auto_fix_applied: false
resolution:      null
opened_at:       "2026-05-19T16:00:00Z"
updated_at:      "2026-05-19T16:00:00Z"
```

### Example 3 — CLI output

```bash
$ python -m cyberos.cuo.baseline modules/skill/feature-request-author/
✓ feature-request-author — baseline valid (all 3 thresholds met)

$ python -m cyberos.cuo.baseline modules/skill/closure-author/
✗ closure-author — baseline invalid:
  - section_missing: '## Token-budget transparency'
  - review_overdue: 47 days
```

## §9 — Open questions

**All resolved during authoring.**

Deferred to follow-up FRs:
- **FR-SKILL-118** (placeholder — not yet specified): re-measurement automation — at the 12-month review-due date, automatically schedule a re-measurement run using OBS data + propose updated numbers via a `refinement_proposal` envelope. Phase P2+.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Author at v1.0 promotion skips BASELINE.md | FM-114 fires; CI gate fails | Promotion blocked | Measure baseline + author the artefact |
| Author writes BASELINE.md with estimated (not measured) numbers | Audit step has no detection at file-write time; trust gap surfaces in production when acceptance drifts | Skill might pass initial promotion but drift fast | Periodic re-measurement (12-month cadence) surfaces drift |
| Author writes BASELINE.md with cherry-picked sample | Same as above — undetectable at write time | Same as above | Audit attestation chain documents the methodology; reviewer should reject cherry-picking |
| `next_review_due` set far in future (e.g. 2099) | Validator accepts (no upper bound) | Stale-by-default | feature-request-audit skill §3.11 prescribes default +12 months |
| `attested_by` is a placeholder like `human:tbd` | ATTESTOR_RE accepts (matches `human:` prefix pattern) | Defeats the attestation chain | Reviewer rejects on PR review (manual safeguard) |
| BASELINE.md present but numbers fail thresholds | Validator passes (numbers themselves aren't gated by validator); operator override reasoning in `## Authoring notes` should explain | Override is recorded; promotion proceeds with documented exception | Periodic review surfaces the exception's continued validity |
| Skill drops `skill_version` from 1.0.0 → 0.x (rare) | FM-114 severity drops from error to info | Baseline becomes advisory | Operator removes BASELINE.md if downgrade is intentional |
| `next_review_due` in past at PR-merge time | Validator flags `review_overdue` — could block merge | Operator updates baseline + advances `next_review_due` | Re-measurement cycle |
| Numbers fail 1-of-3 thresholds (e.g. tool-call good, token good, failure bad) | Override clause applies | Promotion proceeds with documented exception | Authoring notes explain why the failure-rate threshold doesn't apply (e.g. audit skill's "failure" semantically means "found violations") |
| Validator misclassifies an ISO 8601 timestamp | Test fixture catches | Type-system robust | Add test case |
| Token-budget section's numbers are stale | Authoring notes flag staleness | Operator decides | Re-measurement cycle |
| Backfill of hello-world is rejected because hello-world is "teaching example" | Operator-override clause applies | hello-world's BASELINE.md notes "teaching example — thresholds not applicable" | Document in Authoring notes |
| Migration mistake — BASELINE.md committed but skill_version not bumped to 1.0 | FM-114 fires as info (≤v1) but is benign | Audit noise | Bump skill_version OR remove BASELINE.md until ready |

## §11 — Implementation notes

- **Why per-skill rather than module-wide BASELINE?** Skills export individually (per AGENTS.md §11 plug-in contract). A module-wide CSV would be lost on export. Per-skill BASELINE.md travels with the skill folder.
- **Why human-attestation as `human:<id>` form?** Distinguishes from persona-attestation. A persona attestation can be auto-generated by a workflow (e.g. cuo-cpo signs off on its own promotion via a chained skill); human attestation requires a real signature in the audit row (per AGENTS.md §7).
- **Why ASCII-only sections rather than Vietnamese-localised?** English is the engineering lingua franca (per README Part 17.2). VN-locale doesn't need a parallel BASELINE.md — the artefact is for internal governance, not end-user UX.
- **Why 12-month review cadence rather than 6 or 24?** Empirical. LLM model updates ship roughly every 6 months; user-behaviour shifts every 12-18 months. 12 months catches both; 6 months would be too frequent for stable-baseline skills; 24 months too lax.
- **Operator override is the escape valve.** Some skills add value in dimensions the baseline doesn't measure (audit skills, trust-calibration skills, governance skills). The override preserves the rigor while allowing legitimate exceptions. The audit row records the override + reason.
- **Why FM-114 doesn't gate `status: accepted` (only v1.0+)?** Most CyberOS skills will live at `skill_version: 0.x` for many cycles. Forcing a baseline at every accept-status promotion would inflate authoring cost. The artefact is for v1.0+ where partner-exposure + drift-anchor questions become real.
- **The baseline numbers are operator-attested, not validator-checked.** The validator confirms the file exists + has the right shape + has a valid attestation. It does NOT verify the numbers themselves (that would require running the actual workload). The operator-attestation chain (persona-id or human-id signoff) is the trust anchor; periodic re-measurement is the drift detector.
- **Foundation-stage value:** today only hello-world is v1.0; backfilling its baseline costs ~15 minutes. The infrastructure (template + validator + auditor rule + recipe) is what earns its place — when the first production skill promotes from v0.x to v1.0 (likely feature-request-author or feature-request-audit at a Phase 2+ milestone), the artefact is ready.

---

*End of FR-SKILL-114.*
