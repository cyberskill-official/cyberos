"""Command-line entry point — `cyberos-cuo` per cuo/docs/SPEC.md §3.

Subcommands (Phase 1):
    list-personas              enumerate cuo/<persona>/ — shipped vs planned vs extinct
    list-workflows <persona>   enumerate cuo/<persona>/workflows/ with status flags
    route "<query>"            persona-match + workflow-match (dry; no skill invocation)
    dry-run <persona>/<wf>     plan the skill chain (validate + step-plan; no invoke)

Subcommands deferred to Phase 2:
    run <persona>/<wf>         actually walk the chain via cyberos-skill run + BRAIN audit

Resolves the cuo/ + skill/ roots either from --cuo-root/--skill-root flags or by
walking up from the current working directory looking for cuo/MODULE.md + skill/MODULE.md.
"""

from __future__ import annotations

import sys
from pathlib import Path

import click

from cuo import __version__
from cuo.core.brain_bridge import brain_is_available, emit_chain_result
from cuo.core.catalog import discover_personas, discover_workflows
from cuo.core.invoker import SubprocessInvoker, select_invoker
from cuo.core.llm_invoker import LLMInvoker
from cuo.core.router import route
from cuo.core.supervisor import dry_run_chain, execute_chain
from cuo.core.validator import validate_chain


def _find_cyberos_root(start: Path) -> tuple[Path, str] | None:
    """Walk up from `start` looking for the cyberos repo root.

    Returns (cyberos_root, layout) tuple where layout is either:
      - "modules"   — modules at <root>/modules/{cuo,skill}/  (current layout)
      - "flat"      — modules at <root>/{cuo,skill}/          (legacy layout)
    Or None if neither found within 8 ancestors.
    """
    cur = start.resolve()
    for _ in range(8):
        # Prefer the new modules/ layout
        if (cur / "modules" / "cuo" / "MODULE.md").is_file() and (
            cur / "modules" / "skill" / "MODULE.md"
        ).is_file():
            return (cur, "modules")
        # Fallback to legacy flat layout (pre-2026-05-18 refactor)
        if (cur / "cuo" / "MODULE.md").is_file() and (cur / "skill" / "MODULE.md").is_file():
            return (cur, "flat")
        if cur.parent == cur:
            break
        cur = cur.parent
    return None


def _resolve_roots(cuo_root: Path | None, skill_root: Path | None) -> tuple[Path, Path]:
    """Resolve cuo/ + skill/ roots from explicit flags OR cwd walk."""
    if cuo_root and skill_root:
        return cuo_root.resolve(), skill_root.resolve()

    discovered = _find_cyberos_root(Path.cwd())
    if discovered is None:
        click.echo(
            "error: could not locate cuo/MODULE.md + skill/MODULE.md by walking from cwd.\n"
            "       Pass --cuo-root and --skill-root explicitly.",
            err=True,
        )
        sys.exit(2)

    root, layout = discovered
    sub = root / "modules" if layout == "modules" else root
    return (
        cuo_root.resolve() if cuo_root else sub / "cuo",
        skill_root.resolve() if skill_root else sub / "skill",
    )


@click.group(help="CyberOS CUO supervisor — persona-aware orchestration.")
@click.version_option(version=__version__, prog_name="cyberos-cuo")
@click.option("--cuo-root", type=click.Path(exists=True, file_okay=False, path_type=Path), default=None)
@click.option("--skill-root", type=click.Path(exists=True, file_okay=False, path_type=Path), default=None)
@click.pass_context
def main(ctx: click.Context, cuo_root: Path | None, skill_root: Path | None) -> None:
    """CyberOS CUO supervisor — persona-aware orchestration."""
    ctx.ensure_object(dict)
    cuo_path, skill_path = _resolve_roots(cuo_root, skill_root)
    ctx.obj["cuo_root"] = cuo_path
    ctx.obj["skill_root"] = skill_path


@main.command("list-personas")
@click.option("--show-extinct/--no-show-extinct", default=False)
@click.option("--show-planned/--no-show-planned", default=True)
@click.pass_context
def cmd_list_personas(ctx: click.Context, show_extinct: bool, show_planned: bool) -> None:
    """Enumerate persona folders under cuo/ — shipped, planned, extinct."""
    personas = discover_personas(ctx.obj["cuo_root"])
    n_total = len(personas)
    n_shipped = sum(1 for p in personas if p.has_workflows)
    n_extinct = sum(1 for p in personas if p.is_extinct)
    click.echo(f"# {n_shipped} shipped + {n_total - n_shipped - n_extinct} planned + {n_extinct} extinct = {n_total} total")

    for p in personas:
        if p.is_extinct and not show_extinct:
            continue
        if not p.has_workflows and not p.is_extinct and not show_planned:
            continue
        flag = "EXTINCT" if p.is_extinct else ("shipped" if p.has_workflows else "planned")
        n_wf = len(list(p.workflows_dir.glob("*.md"))) if p.workflows_dir.is_dir() else 0
        title_suffix = f" — {p.disambiguated_title}" if p.disambiguated_title else ""
        click.echo(f"  {p.slug:35s} [{flag:8s}] {n_wf} workflows{title_suffix}")


@main.command("list-workflows")
@click.argument("persona_slug")
@click.pass_context
def cmd_list_workflows(ctx: click.Context, persona_slug: str) -> None:
    """Enumerate workflows for a persona with validation status."""
    personas = discover_personas(ctx.obj["cuo_root"])
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        click.echo(f"error: persona {persona_slug!r} not found", err=True)
        sys.exit(2)

    workflows = discover_workflows(persona)
    if not workflows:
        click.echo(f"# {persona_slug}: no workflows shipped")
        return

    click.echo(f"# {persona_slug}: {len(workflows)} workflow(s)")
    for wf in workflows:
        v = validate_chain(wf, ctx.obj["skill_root"])
        valid_flag = "valid" if v.valid else f"BLOCKED ({len(v.missing_skills)} missing + {len(v.planned_skills)} planned)"
        click.echo(
            f"  {wf.slug:50s} cadence={wf.cadence:10s} chain={v.chain_length:2d} [{valid_flag}]"
        )


@main.command("route")
@click.argument("query")
@click.option("--show-alternatives/--no-show-alternatives", default=True)
@click.pass_context
def cmd_route(ctx: click.Context, query: str, show_alternatives: bool) -> None:
    """Route a natural-language query to a (persona, workflow) pair."""
    personas = discover_personas(ctx.obj["cuo_root"])
    decision = route(query, personas)
    if decision is None:
        click.echo(f"# route({query!r}) → NO_MATCH (no persona/workflow scored above threshold)")
        sys.exit(1)

    click.echo(f"# route({query!r}) → {decision.persona_slug}/{decision.workflow_slug}")
    click.echo(f"  confidence: {decision.confidence:.2f}")
    click.echo(f"  rationale:  {decision.rationale}")
    if show_alternatives and decision.alternative_personas:
        click.echo("  alternative personas:")
        for slug, sc in decision.alternative_personas:
            click.echo(f"    {slug:35s} {sc:.2f}")
    if show_alternatives and decision.alternative_workflows:
        click.echo("  alternative workflows:")
        for slug, sc in decision.alternative_workflows:
            click.echo(f"    {slug:35s} {sc:.2f}")


@main.command("dry-run")
@click.argument("persona_workflow")
@click.pass_context
def cmd_dry_run(ctx: click.Context, persona_workflow: str) -> None:
    """Plan (but do not execute) a workflow's skill chain.

    PERSONA_WORKFLOW format: <persona-slug>/<workflow-slug>, e.g. chief-technology-officer/architect-new-system.
    """
    if "/" not in persona_workflow:
        click.echo("error: PERSONA_WORKFLOW must be <persona-slug>/<workflow-slug>", err=True)
        sys.exit(2)

    persona_slug, workflow_slug = persona_workflow.split("/", 1)
    personas = discover_personas(ctx.obj["cuo_root"])
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        click.echo(f"error: persona {persona_slug!r} not found", err=True)
        sys.exit(2)

    result = dry_run_chain(persona, workflow_slug, ctx.obj["skill_root"])
    flag = "RUNNABLE" if result.runnable else "BLOCKED"
    click.echo(f"# dry-run {result.workflow_id} → {flag}")
    click.echo(f"  chain length: {result.validation.chain_length}")
    click.echo(f"  found:        {len(result.validation.found_skills)}")
    click.echo(f"  missing:      {len(result.validation.missing_skills)}")
    click.echo(f"  planned:      {len(result.validation.planned_skills)}")
    click.echo("")
    click.echo("# step plan")
    for line in result.step_plan:
        click.echo(f"  {line}")
    if result.notes:
        click.echo("")
        click.echo("# notes")
        for note in result.notes:
            click.echo(f"  * {note}")
    sys.exit(0 if result.runnable else 1)


@main.command("execute")
@click.argument("persona_workflow")
@click.option(
    "--output-dir",
    type=click.Path(file_okay=False, path_type=Path),
    required=True,
    help="Directory where per-step output JSON files are written.",
)
@click.option(
    "--invoker",
    type=click.Choice(["auto", "mock", "subprocess", "llm"]),
    default="auto",
    help="Skill invoker: auto, mock (deterministic), subprocess (cyberos-skill binary), llm (Anthropic API / mock-llm).",
)
@click.option(
    "--input",
    "raw_inputs",
    multiple=True,
    help="Initial workflow input as KEY=PATH (or KEY=VALUE). Repeat for multiple.",
)
@click.option("--continue-on-failure", is_flag=True, default=False)
@click.option(
    "--brain-emit/--no-brain-emit",
    default=False,
    help="Emit per-step + workflow_complete rows to the BRAIN audit chain (Phase 3).",
)
@click.option(
    "--actor",
    default="cuo-supervisor",
    help="Actor name attached to BRAIN audit rows (only used with --brain-emit).",
)
@click.option(
    "--explain",
    is_flag=True,
    default=False,
    help="Show which Handler was picked + why (Phase 4 dispatch rationale).",
)
@click.option(
    "--no-handler-dispatch",
    is_flag=True,
    default=False,
    help="Bypass Phase 4 handler dispatch; always use linear execute_chain (debug).",
)
@click.pass_context
def cmd_execute(
    ctx: click.Context,
    persona_workflow: str,
    output_dir: Path,
    invoker: str,
    raw_inputs: tuple,
    continue_on_failure: bool,
    brain_emit: bool,
    actor: str,
    explain: bool,
    no_handler_dispatch: bool,
) -> None:
    """Execute a workflow chain (Phase 2 — actual invocation, not dry-run).

    Walks the workflow's skill_chain step-by-step via the selected invoker.
    With --invoker=auto, uses the `cyberos-skill` binary if it's on PATH,
    otherwise falls back to MockInvoker (deterministic placeholder output).

    PERSONA_WORKFLOW format: <persona-slug>/<workflow-slug>.
    """
    if "/" not in persona_workflow:
        click.echo("error: PERSONA_WORKFLOW must be <persona-slug>/<workflow-slug>", err=True)
        sys.exit(2)
    persona_slug, workflow_slug = persona_workflow.split("/", 1)

    personas = discover_personas(ctx.obj["cuo_root"])
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        click.echo(f"error: persona {persona_slug!r} not found", err=True)
        sys.exit(2)

    # Parse --input KEY=VALUE pairs.
    parsed_inputs: dict = {}
    for raw in raw_inputs:
        if "=" not in raw:
            click.echo(f"error: --input {raw!r} must be KEY=VALUE", err=True)
            sys.exit(2)
        k, v = raw.split("=", 1)
        parsed_inputs[k.strip()] = v.strip()

    if invoker == "llm":
        inv = LLMInvoker()
        inv_kind = f"LLMInvoker(mode={inv.mode})"
    else:
        inv = select_invoker(invoker)
        inv_kind = type(inv).__name__
    if invoker == "subprocess" and not SubprocessInvoker.is_available():
        click.echo(
            f"warning: --invoker=subprocess requested but `cyberos-skill` binary not on PATH. "
            "All steps will FAIL with 'binary not available'.",
            err=True,
        )

    # Phase 4: handler dispatch by workflow `pattern:` frontmatter.
    # Locate the WorkflowEntry so we can read frontmatter.pattern.
    from cuo.core.catalog import discover_workflows
    from cuo.core.handlers import HandlerDispatchError, pattern_of, pick_handler

    persona_workflows = discover_workflows(persona)
    wf_entry = next((w for w in persona_workflows if w.slug == workflow_slug), None)

    handler_obj = None
    pattern = "linear"
    if wf_entry is not None and not no_handler_dispatch:
        pattern = pattern_of(wf_entry)
        try:
            handler_obj = pick_handler(wf_entry)
        except HandlerDispatchError as e:
            click.echo(f"error: handler dispatch failed — {e}", err=True)
            sys.exit(2)

    if explain:
        click.echo(f"# dispatch")
        click.echo(f"  pattern:        {pattern}")
        if handler_obj is not None:
            click.echo(f"  handler:        {handler_obj.__class__.__name__}")
        else:
            click.echo(f"  handler:        (bypassed via --no-handler-dispatch)")
        click.echo(f"  invoker:        {inv_kind}")
        click.echo(f"  workflow_file:  {wf_entry.workflow_file if wf_entry else '<unresolved>'}")
        click.echo("")

    # Execute via Handler dispatch path OR direct execute_chain fallback.
    extra_audit: list = []
    if handler_obj is not None and pattern != "linear":
        # Non-linear pattern — use Handler.execute() and unwrap to ChainResult
        click.echo(f"# dispatched to {handler_obj.__class__.__name__}")
        hr = handler_obj.execute(
            persona=persona,
            workflow=wf_entry,
            skill_root=ctx.obj["skill_root"],
            output_dir=output_dir,
            inputs=parsed_inputs,
            invoker=inv,
        )
        result = hr.chain_result
        extra_audit = list(hr.extra_audit_kinds)
        for n in hr.notes:
            # Surface handler notes onto the chain result for unified display
            if n not in result.notes:
                result.notes.append(n)
    else:
        # Linear pattern (or --no-handler-dispatch) — straight execute_chain
        result = execute_chain(
            persona=persona,
            workflow_slug=workflow_slug,
            skill_root=ctx.obj["skill_root"],
            output_dir=output_dir,
            inputs=parsed_inputs,
            invoker=inv,
            stop_on_failure=not continue_on_failure,
        )

    click.echo(f"# execute {result.workflow_id} → {result.outcome}")
    click.echo(f"  invoker:      {inv_kind}")
    click.echo(f"  chain length: {result.validation.chain_length if result.validation else '?'}")
    click.echo(f"  steps run:    {len(result.step_results)}")
    click.echo(f"  total time:   {result.total_duration_ms} ms")
    click.echo(f"  output dir:   {result.output_dir}")
    click.echo("")
    click.echo("# step results")
    for s in result.step_results:
        marker = {"OK": "✓", "MOCKED": "○", "FAILED": "✗", "SKIPPED": "−"}.get(s.status, "?")
        click.echo(f"  {marker} step {s.step:2d} [{s.status:7s}] {s.skill:35s} {s.duration_ms:5d}ms")
        for note in s.notes:
            click.echo(f"      note: {note}")
    if result.notes:
        click.echo("")
        click.echo("# chain notes")
        for n in result.notes:
            click.echo(f"  * {n}")

    if extra_audit:
        click.echo("")
        click.echo(f"# handler audit kinds ({len(extra_audit)})")
        for a in extra_audit:
            click.echo(f"  - {a.get('kind', '<unknown>')}: {dict((k, v) for k, v in a.items() if k != 'kind')}")

    # Phase 3: BRAIN audit emission (opt-in).
    if brain_emit:
        click.echo("")
        click.echo("# BRAIN emission")
        if not brain_is_available(ctx.obj["skill_root"]):
            click.echo("  SKIPPED — BRAIN not reachable (memory module not importable OR .cyberos-memory missing)")
        else:
            br = emit_chain_result(result, ctx.obj["skill_root"], actor=actor)
            if br.emitted:
                click.echo(f"  emitted: {br.rows_written} row(s)")
                if br.chain_head_after:
                    click.echo(f"  chain head: {br.chain_head_after[:16]}...")
            else:
                click.echo(f"  SKIPPED — {br.reason_skipped}")
            for n in br.notes:
                click.echo(f"  note: {n}")

    sys.exit(0 if result.outcome in ("COMPLETED", "COMPLETED_BATCH") else 1)


if __name__ == "__main__":
    main()
