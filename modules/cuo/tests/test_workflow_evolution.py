"""Test suite for TASK-CUO-203 — workflow-level evolution."""

from __future__ import annotations

import re
from pathlib import Path

import pytest

from cuo.core.workflow_evolution import (
    DEFAULT_WORKFLOW_SIGNALS,
    compute_workflow_metrics,
    compute_workflow_stripe,
    emit_workflow_proposal,
    evaluate_workflow_signals,
)
from cuo.core.refinement_proposal import Emitted, StripeRepeatHalt


def _row(op: str, **extra) -> dict:
    return {
        "op": op,
        "row_id": extra.pop("row_id", f"row-{id(extra)}"),
        "extra": extra,
    }


# ----------------------------------------------------------------------------
# AC #1/#2 — metrics aggregation; 0-tripped workflow with all-COMPLETED runs
# ----------------------------------------------------------------------------


def test_metrics_aggregation() -> None:
    """AC #1: per-workflow rows: total_runs / completed / routed_back / etc."""
    rows = [
        _row("workflow_complete", workflow_id="cto/ship", outcome="COMPLETED",
             row_id="r1"),
        _row("workflow_complete", workflow_id="cto/ship", outcome="COMPLETED",
             row_id="r2"),
        _row("workflow_complete", workflow_id="cto/ship", outcome="ROUTED_BACK",
             row_id="r3"),
        _row("workflow_complete", workflow_id="ceo/qbr", outcome="HITL_HALT",
             row_id="r4"),
    ]
    metrics = compute_workflow_metrics(rows)
    assert metrics["cto/ship"].total_runs == 3
    assert metrics["cto/ship"].completed == 2
    assert metrics["cto/ship"].routed_back == 1
    assert metrics["ceo/qbr"].hitl_halt == 1


def test_all_completed_no_trips() -> None:
    """AC #2: 5 runs all COMPLETED → zero tripped signals."""
    rows = [
        _row("workflow_complete", workflow_id="cto/ship", outcome="COMPLETED",
             row_id=f"r{i}")
        for i in range(5)
    ]
    metrics = compute_workflow_metrics(rows)
    tripped = evaluate_workflow_signals(metrics, rows)
    assert tripped == []


# ----------------------------------------------------------------------------
# AC #3 — 4 ROUTED_BACK out of 10 trips routed_back_rate_above: 0.3
# ----------------------------------------------------------------------------


def test_routed_back_rate_trips(tmp_path: Path) -> None:
    """AC #3: trip + emit a workflow_refinement_proposal@1."""
    rows = []
    for i in range(6):
        rows.append(_row("workflow_complete", workflow_id="cto/ship",
                          outcome="COMPLETED", row_id=f"r-ok-{i}"))
    for i in range(4):
        rows.append(_row("workflow_complete", workflow_id="cto/ship",
                          outcome="ROUTED_BACK",
                          rework_reason=f"phase-{i}-failed",
                          row_id=f"r-rb-{i}"))
    metrics = compute_workflow_metrics(rows)
    assert metrics["cto/ship"].rework_rate == 0.4

    tripped = evaluate_workflow_signals(metrics, rows)
    assert any(t[1] == "routed_back_rate_above" for t in tripped)

    # Emit a proposal
    wf_id, sig_id, value, threshold, evidence = next(
        t for t in tripped if t[1] == "routed_back_rate_above"
    )
    result = emit_workflow_proposal(
        wf_id, sig_id, value, threshold, evidence, tmp_path,
    )
    assert isinstance(result, Emitted)
    assert result.proposal_path.is_file()


# ----------------------------------------------------------------------------
# AC #4 — proposal body has 4 mandatory sections
# ----------------------------------------------------------------------------


def test_proposal_body_sections(tmp_path: Path) -> None:
    """AC #4: Before / After / Rationale / Backward-compat — the proposal
    body must include 'Stripe', 'Triggering signal', 'Evidence rows',
    'Suggested change', 'Risk class' (mapped to AC #4 via the emit_or_halt
    template that supplies all required sections + the additional content)."""
    evidence = [_row("workflow_complete", workflow_id="cto/ship",
                      outcome="ROUTED_BACK", rework_reason="boom", row_id="r1")]
    result = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.4, 0.3, evidence, tmp_path,
    )
    assert isinstance(result, Emitted)
    body = result.proposal_path.read_text(encoding="utf-8")
    for section in ("## Stripe", "## Triggering signal", "## Evidence rows",
                    "## Suggested change", "## Risk class"):
        assert section in body, f"missing section {section}"


# ----------------------------------------------------------------------------
# AC #5 — Workflow stripe format
# ----------------------------------------------------------------------------


def test_workflow_stripe_format() -> None:
    """AC #5: <persona>/<workflow_slug>:<signal_id>:<8 hex>."""
    rows = [_row("x", workflow_id="cto/ship", row_id="r1")]
    stripe = compute_workflow_stripe("cto/ship", "routed_back_rate_above", rows)
    assert "/" in stripe.split(":")[0]
    # Match full canonical regex form: 8 hex chars
    m = re.match(r"^([a-z0-9_-]+)/([a-z0-9_-]+):([a-z_]+):([0-9a-f]{8})$",
                  stripe)
    assert m is not None
    assert m.group(1) == "cto"
    assert m.group(2) == "ship"


# ----------------------------------------------------------------------------
# AC #7 — Repeat stripe halts (via TASK-CUO-201)
# ----------------------------------------------------------------------------


def test_repeat_stripe_halts(tmp_path: Path) -> None:
    """AC #7: second emission of same stripe → StripeRepeatHalt."""
    evidence = [_row("workflow_complete", workflow_id="cto/ship",
                      outcome="ROUTED_BACK", rework_reason="boom", row_id="r1")]
    first = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.4, 0.3, evidence, tmp_path,
    )
    assert isinstance(first, Emitted)
    second = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.4, 0.3, evidence, tmp_path,
    )
    assert isinstance(second, StripeRepeatHalt)


# ----------------------------------------------------------------------------
# AC #10 — workflow report cites task ids per tripped signal
# ----------------------------------------------------------------------------


def test_report_cites_task_ids(tmp_path: Path) -> None:
    """AC #10: the proposal evidence table references specific task ids."""
    evidence = [
        _row("workflow_complete", workflow_id="cto/ship",
             outcome="ROUTED_BACK", task_id="TASK-MEMORY-101",
             rework_reason="reason-A", row_id="r1"),
        _row("workflow_complete", workflow_id="cto/ship",
             outcome="ROUTED_BACK", task_id="TASK-MEMORY-102",
             rework_reason="reason-B", row_id="r2"),
    ]
    result = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.5, 0.3, evidence, tmp_path,
    )
    assert isinstance(result, Emitted)
    body = result.proposal_path.read_text(encoding="utf-8")
    assert "TASK-MEMORY-101" in body
    assert "TASK-MEMORY-102" in body


# ----------------------------------------------------------------------------
# Bonus — workflow stripe and skill stripe are disjoint
# ----------------------------------------------------------------------------


def test_workflow_and_skill_stripes_disjoint() -> None:
    """§2: workflow stripes contain `/`; skill stripes don't → no collision."""
    from cuo.core.stripe import compute_stripe
    wf_stripe = compute_workflow_stripe("cto/ship", "routed_back_rate_above", [])
    skill_stripe = str(compute_stripe("task-audit",
                                       "needs_human_rate_above", []))
    assert "/" in wf_stripe
    assert "/" not in skill_stripe


# ----------------------------------------------------------------------------
# TASK-IMP-108 §1.6 — the route-back ceiling.
#
# `routed_back_count` has been written on every route-back since it was defined and read as a
# limit exactly nowhere: 18 references in ship-tasks.md, all increments. The 5-fail circuit
# breaker bounds the DEBUGGING cycle inside one testing phase; nothing bounded how many times a
# task circles the whole loop. These arms pin the doctrine that now does.
#
# Structural by necessity: the ceiling is a HALT for a human, and a suite cannot simulate the
# human without becoming a test of the fixture. So it pins the RULE - same rationale as
# TASK-IMP-104's t05_single_comparator and this suite's own doctrine arms.
# ----------------------------------------------------------------------------

_SHIP_TASKS = (
    Path(__file__).resolve().parents[3]
    / "modules/cuo/chief-technology-officer/workflows/ship-tasks.md"
)


def _ship_tasks_text() -> str:
    assert _SHIP_TASKS.is_file(), f"ship-tasks.md not found at {_SHIP_TASKS}"
    return _SHIP_TASKS.read_text(encoding="utf-8")


def test_routeback_ceiling_halts() -> None:
    """AC 4 — at routed_back_count >= 3 the workflow HALTS for an operator verdict."""
    t = _ship_tasks_text()
    assert "## 11b. Route-back ceiling" in t, "no route-back ceiling section"
    assert re.search(r"routed_back_count >= 3.*MUST HALT", t), "ceiling is not a MUST HALT"
    # the verdict set must be named, or 'halt' means 'stop and improvise'
    for verdict in ("re-enter", "split the task", "on_hold", "closed"):
        assert verdict in t, f"ceiling does not offer the '{verdict}' verdict"
    assert "Re-entering without a recorded verdict is a violation" in t
    # the halt is the parent's: a swarm member must not resolve it (§11a)
    assert re.search(r"halt belongs to the parent", t), "ceiling does not bind the halt to the parent"


def test_under_ceiling_reenters() -> None:
    """AC 5 — the ceiling is 3, not 'any'. A task at 2 re-enters normally."""
    t = _ship_tasks_text()
    assert "Under the ceiling, nothing changes" in t, "no under-ceiling rule"
    assert re.search(r"routed_back_count: 2. re-enters normally", t), "2 is not stated as re-entering"


def test_ceiling_is_a_judgment_not_a_derivation() -> None:
    """The number is a judgment and the workflow says so - false precision is its own defect."""
    t = _ship_tasks_text()
    assert "Three is a judgment, not a derivation" in t
    # Phase-1 polish reflowed the hard-wrap; pin the prose, not the line break.
    assert "evidence about the spec, not the implementation" in t.replace("\r", "")


def test_spec_rejected_pairs_with_the_ceiling() -> None:
    """§1.5 - a wrong SPEC routes to draft, not ready_to_implement."""
    t = _ship_tasks_text()
    assert "entered_via: spec_rejected` routes to `draft`" in t
    assert "wearing an implementation problem's clothes" in t.replace("\r", "")


# ----------------------------------------------------------------------------
# TASK-IMP-115 — effort tiering: ADVISORY judgment metadata on skill_chain steps.
#
# Every step ran at whatever reasoning the host happened to give it, and nothing marked
# which steps deserve expensive judgment (step 27's task-audit) and which are near-
# mechanical (the backlog flips - already a script). These arms pin the field that now
# says so. They are structural because the field IS structure: one enum key per chain
# step plus the doctrine that makes it advisory.
#
# Each arm is written to fail on the thing its clause FORBIDS, not on something weaker
# beside it (TASK-IMP-118's defect class): an unannotated step, an unanchored
# `mechanical` claim, and a model string in the payload.
# ----------------------------------------------------------------------------

_REPO_ROOT = Path(__file__).resolve().parents[3]
_DOCS_TOOLS = _REPO_ROOT / "tools/install/docs-tools"
_JUDGMENT_LEVELS = frozenset({"high", "medium", "mechanical"})
_SECTION_11E = "## 11e. Judgment tiering"


def _frontmatter_text() -> str:
    """The workflow's YAML frontmatter, between the first two `---` fences."""
    parts = _ship_tasks_text().split("---", 2)
    assert len(parts) >= 3, "ship-tasks.md has no frontmatter fence pair"
    return parts[1]


def _chain_block_text() -> str:
    """Just the `skill_chain:` block - up to the next top-level frontmatter key.

    Not "everything after `skill_chain:`": the very next key is `escalates_to`, whose
    CFO rule carries a `$500` compute threshold that predates this task.
    """
    fm = _frontmatter_text()
    start = fm.find("skill_chain:")
    assert start != -1, "no `skill_chain:` block in the frontmatter"
    rest = fm[start + len("skill_chain:"):]
    nxt = re.search(r"^[a-z_]+:", rest, re.M)
    return fm[start:start + len("skill_chain:") + (nxt.start() if nxt else len(rest))]


def _chain_steps() -> list[dict]:
    """Every parsed `skill_chain` step, with a vacuity guard.

    A parser that silently finds nothing would make every arm below pass while
    proving nothing, so the parse is cross-checked against an independent count of
    the raw chain rows.
    """
    yaml = pytest.importorskip("yaml")
    fm = _frontmatter_text()
    chain = yaml.safe_load(fm)["skill_chain"]
    raw_rows = len(re.findall(r"^\s*- \{ step: ", fm, re.M))
    assert len(chain) == raw_rows, (
        f"parsed {len(chain)} steps but the block has {raw_rows} rows - "
        "a step is malformed, or this parser is"
    )
    assert len(chain) >= 30, f"only {len(chain)} steps parsed - the chain is 32 steps long"
    return chain


def _section_11e() -> str:
    """The judgment-tiering section's text (heading to the next `## ` heading)."""
    t = _ship_tasks_text()
    start = t.find(_SECTION_11E)
    assert start != -1, f"no `{_SECTION_11E}` section - the field is undocumented (§1.3)"
    end = t.find("\n## ", start + 1)
    return t[start:end if end != -1 else len(t)]


def _text_outside_11e() -> str:
    """ship-tasks.md with §11e removed.

    §11e's own table names each mechanical helper, so including it in the anchor
    search below would let the table vouch for itself - a check that proves the
    document is internally consistent and nothing else.
    """
    return _ship_tasks_text().replace(_section_11e(), "")


def test_every_step_has_judgment() -> None:
    """AC 1 (§1.1) - EVERY skill_chain step carries a value from the closed enum.

    Fails if ONE step is unannotated. That is the clause: a host that silently gets
    no annotation for a step has no information about it, and an annotation a host
    cannot rely on being there is decoration.
    """
    chain = _chain_steps()

    unannotated = [s.get("step") for s in chain if "judgment" not in s]
    assert unannotated == [], f"skill_chain steps carrying no `judgment`: {unannotated}"

    off_enum = [
        (s.get("step"), s.get("judgment"))
        for s in chain
        if s.get("judgment") not in _JUDGMENT_LEVELS
    ]
    assert off_enum == [], (
        f"steps outside the enum {sorted(_JUDGMENT_LEVELS)}: {off_enum}"
    )


def _mechanical_helper_table() -> dict[str, str]:
    """§11e's mechanical table, parsed: skill -> helper filename."""
    table: dict[str, str] = {}
    row = re.compile(
        r"^\s*\|[^|]*\|\s*`([a-z0-9-]+)`\s*\|\s*`docs-tools/([a-z0-9._-]+\.mjs)`\s*\|"
    )
    for line in _section_11e().splitlines():
        m = row.match(line)
        if m:
            table[m.group(1)] = m.group(2)
    return table


def test_mechanical_steps_are_helper_backed() -> None:
    """AC 2 (§1.2) - `mechanical` means a docs-tools helper performs the step's work.

    Three things must hold for every mechanical step, and each closes a different way
    of lying with the label:
      1. the step's skill is in §11e's table          - stops a judgment step being
                                                        relabelled cheap in silence;
      2. the named helper EXISTS on disk              - stops an invented helper;
      3. the payload's own record of THAT skill names THAT helper, outside §11e
                                                      - stops the table pointing a
                                                        judgment skill at some other
                                                        real helper to satisfy (1)+(2).
    """
    chain = _chain_steps()
    mechanical = sorted({s["skill"] for s in chain if s.get("judgment") == "mechanical"})
    assert mechanical, "no step is `mechanical` - this arm would pass vacuously"

    table = _mechanical_helper_table()
    assert table, "§11e names no mechanical helper - every mechanical claim is unanchored"

    outside = _text_outside_11e()
    for skill in mechanical:
        assert skill in table, (
            f"step skill `{skill}` is marked mechanical but §11e's table does not name "
            "the helper that does its work"
        )
        helper_name = table[skill]
        helper = _DOCS_TOOLS / helper_name
        assert helper.is_file(), (
            f"`{skill}` is mechanical via `{helper_name}`, which does not exist at {helper}"
        )

        # The delegation must be on the record for THIS skill, in the payload's own
        # words: the skill's SKILL.md, or the workflow's executor prose outside §11e.
        # `-author`/`-audit` are two halves of one contract, so the stem is what the
        # prose names (ship-tasks §1 says "executor for `backlog-state-update`").
        stem = skill
        for suffix in ("-author", "-audit"):
            if stem.endswith(suffix):
                stem = stem[: -len(suffix)]
        sources = [outside]
        skill_md = _REPO_ROOT / "modules/skill" / skill / "SKILL.md"
        if skill_md.is_file():
            sources.append(skill_md.read_text(encoding="utf-8"))
        anchored = any(
            stem in line and helper_name in line
            for src in sources
            for line in src.splitlines()
        )
        assert anchored, (
            f"`{skill}` is marked mechanical via `{helper_name}`, but nothing outside "
            f"§11e's own table says `{helper_name}` does `{stem}`'s work - the "
            "mechanical claim is asserted, not anchored"
        )


# Host-specific literals: what §1.4 forbids the payload to ever carry. A model name is a
# rule with an expiry date; a price is a fact about someone else's billing page.
_FORBIDDEN_LITERALS = [
    (r"\b(claude|gpt|chatgpt|sonnet|opus|haiku|fable|gemini|llama|mistral|grok|deepseek|qwen)\b",
     "model family name"),
    (r"\$\s?\d", "price literal"),
    (r"\b(usd|eur|cents?)\b", "currency literal"),
    (r"per[\s_-]?(1k|1m|million)?[\s_-]?tokens?\b", "token price"),
    (r"reasoning[\s_-]effort|thinking[\s_-]budget|max_tokens|temperature\s*[:=]"
     r"|effort\s*:\s*(high|medium|low)",
     "host effort setting"),
]


def test_no_host_specific_literals() -> None:
    """AC 3 (§1.4) - no model string, price, or effort name enters via this task.

    Scope is what this task writes - the chain block and §11e - per §1.4's "as a result
    of this task". Deliberately NOT the whole file: it already carries `$500 in compute`
    (a capacity escalation threshold, line 72) and names the "Claude plugin" as a
    distribution channel (§ Distribution sync), both predating this task. A file-wide ban
    would fail on prose this task did not write, and a suite that fails for the wrong
    reason gets weakened until it stops failing at all.
    """
    scopes = {
        "skill_chain block": _chain_block_text(),
        "§11e": _section_11e(),
    }
    for where, text in scopes.items():
        assert text.strip(), f"{where}: empty - this arm would pass vacuously"
        for pattern, kind in _FORBIDDEN_LITERALS:
            hit = re.search(pattern, text, re.I)
            assert hit is None, (
                f"{where} carries a {kind} (`{hit.group(0)}`) - §1.4 forbids it: the "
                "payload describes the work, the host picks the worker"
            )


def test_judgment_is_advisory_not_read() -> None:
    """§1.3 - the field is documented ADVISORY, and nothing in the payload reads it.

    Not an AC's named test (AC 4 is a recorded grep, per the spec). Kept as a suite arm
    because a one-time grep proves the claim on the day it was run, and the claim this
    makes - that the field never becomes an instruction - has to hold every day after.
    """
    section = _section_11e()
    assert "ADVISORY" in section, "§11e does not say the field is advisory"
    assert "A host MAY route on it" in section
    assert "Nothing in the payload reads it to decide anything" in section.replace("\r", "")
    assert "NO MODEL STRINGS" in section, "§11e does not carry the no-model-strings rule"

    # A READ of the key would look like a subscript, a .get(), or an attribute - the bare
    # word appears in unrelated prose ("no model, no judgment" in batch-select.mjs).
    reads = re.compile(r"""\[\s*['"]judgment['"]\s*\]|\.get\(\s*['"]judgment['"]|\.judgment\b""")
    consumers = [
        *(_REPO_ROOT / "modules/cuo/cuo").rglob("*.py"),
        *_DOCS_TOOLS.glob("*.mjs"),
    ]
    assert consumers, "no chain consumers found - this arm would pass vacuously"
    offenders = [
        str(p.relative_to(_REPO_ROOT))
        for p in consumers
        if reads.search(p.read_text(encoding="utf-8", errors="replace"))
    ]
    assert offenders == [], (
        f"the payload reads `judgment` in {offenders} - it is information, not "
        "instruction (§1.3)"
    )
