"""Command-line entry point — `cyberos-cuo` per cuo/docs/SPEC.md §3.

Subcommands (Phase 1):
    list-personas              enumerate cuo/<persona>/ — shipped vs planned vs extinct
    list-workflows <persona>   enumerate cuo/<persona>/workflows/ with status flags
    route "<query>"            persona-match + workflow-match (dry; no skill invocation)
    dry-run <persona>/<wf>     plan the skill chain (validate + step-plan; no invoke)

Subcommands deferred to Phase 2:
    run <persona>/<wf>         actually walk the chain via cyberos-skill run + memory audit

Resolves the cuo/ + skill/ roots either from --cuo-root/--skill-root flags or by
walking up from the current working directory looking for cuo/MODULE.md + skill/MODULE.md.
"""

from __future__ import annotations

import sys
from datetime import datetime, timezone
from pathlib import Path

import click

from cuo import __version__
from cuo.core.memory_bridge import memory_is_available, emit_chain_result
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

    Resolution order:
      1. CYBEROS_ROOT env var (highest priority after CLI flags)
      2. Walk up from `start`
    """
    import os

    # 1. Explicit env var
    env_root = os.environ.get("CYBEROS_ROOT")
    if env_root:
        root = Path(env_root).resolve()
        if (root / "modules" / "cuo" / "MODULE.md").is_file() and (
            root / "modules" / "skill" / "MODULE.md"
        ).is_file():
            return (root, "modules")
        if (root / "cuo" / "MODULE.md").is_file() and (root / "skill" / "MODULE.md").is_file():
            return (root, "flat")

    # 2. Walk up from start
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
            "       Set CYBEROS_ROOT env var or pass --cuo-root and --skill-root explicitly.",
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
    from cuo.api import list_personas
    list_personas(show_extinct=show_extinct, show_planned=show_planned, cuo_root=ctx.obj["cuo_root"])


@main.command("list-workflows")
@click.argument("persona_slug")
@click.pass_context
def cmd_list_workflows(ctx: click.Context, persona_slug: str) -> None:
    """Enumerate workflows for a persona with validation status."""
    from cuo.api import list_workflows
    list_workflows(persona_slug, cuo_root=ctx.obj["cuo_root"], skill_root=ctx.obj["skill_root"])


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
    from cuo.api import dry_run
    dry_run(persona_workflow, cuo_root=ctx.obj["cuo_root"], skill_root=ctx.obj["skill_root"])


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
    type=click.Choice(["auto", "subprocess", "llm", "brief"]),
    default="auto",
    help="Skill invoker: auto, subprocess (cyberos-skill binary), llm (Anthropic API / mock-llm), "
         "brief (generate execution brief for host LLM).",
)
@click.option(
    "--input",
    "raw_inputs",
    multiple=True,
    help="Initial workflow input as KEY=PATH (or KEY=VALUE). Repeat for multiple.",
)
@click.option("--continue-on-failure", is_flag=True, default=False)
@click.option(
    "--memory-emit/--no-memory-emit",
    default=False,
    help="Emit per-step + workflow_complete rows to the memory audit chain (Phase 3).",
)
@click.option(
    "--actor",
    default="cuo-supervisor",
    help="Actor name attached to memory audit rows (only used with --memory-emit).",
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
@click.option(
    "--fr-id",
    default=None,
    help="Force a specific FR (e.g. TASK-MEMORY-117). Shorthand for `--input task_id=<value>`. "
         "Used by ship-tasks to target one FR rather than picking from BACKLOG.",
)
@click.option(
    "--auto-claim/--no-auto-claim",
    default=True,
    help="Auto-claim phase transitions for zero-touch flows (default true). When false, the "
         "workflow halts between phases and waits for an explicit `cyberos-cuo claim` event "
         "(HITL pickup). Currently every phase auto-runs; flag is forward-compatible for "
         "when reviewer/tester claim semantics are wired in.",
)
@click.option(
    "--rework/--no-rework",
    default=False,
    help="Rework mode: allow bypass of current status checks and force restart.",
)
@click.option(
    "--brief-output",
    type=click.Path(dir_okay=False, path_type=Path),
    default=None,
    help="Write execution brief to FILE instead of stdout (brief mode only).",
)
@click.option(
    "--backlog",
    type=click.Path(exists=True, dir_okay=False, path_type=Path),
    default=None,
    help="Path to BACKLOG.md for FR status lookup. Defaults to cyberos docs/tasks/BACKLOG.md.",
)
@click.pass_context
def cmd_execute(
    ctx: click.Context,
    persona_workflow: str,
    output_dir: Path,
    invoker: str,
    raw_inputs: tuple,
    continue_on_failure: bool,
    memory_emit: bool,
    actor: str,
    explain: bool,
    no_handler_dispatch: bool,
    task_id: str | None,
    auto_claim: bool,
    rework: bool,
    brief_output: Path | None,
    backlog: Path | None,
) -> None:
    """Execute a workflow chain (Phase 2 — actual invocation, not dry-run).

    Walks the workflow's skill_chain step-by-step via the selected invoker.
    With --invoker=auto, uses the `cyberos-skill` binary if it's on PATH,
    otherwise tries LLMInvoker. Raises an error if neither is available.

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

    # --fr-id is a typed shorthand for --input task_id=<value>.
    if task_id is not None:
        parsed_inputs["task_id"] = task_id
    # --auto-claim flag flows into the workflow's input bundle so phase
    # transitions can read it from the hand-off map.
    parsed_inputs["auto_claim"] = auto_claim
    parsed_inputs["rework"] = rework

    # Auto-detect host LLM environment → brief mode (disabled: user wants real execution)
    # if invoker == "auto":
    #     from cuo.core.invoker import detect_host_environment
    #     _host_env = detect_host_environment()
    #     if _host_env:
    #         invoker = "brief"

    # Brief mode: generate an execution brief instead of executing.
    if invoker == "brief":
        from cuo.core.supervisor import brief_chain
        cuo_path = ctx.obj["cuo_root"]
        skill_path = ctx.obj["skill_root"]
        personas = discover_personas(cuo_path)
        persona = next((p for p in personas if p.slug == persona_slug), None)
        if persona is None:
            click.echo(f"error: persona {persona_slug!r} not found", err=True)
            sys.exit(2)
        brief = brief_chain(
            persona=persona,
            workflow_slug=workflow_slug,
            skill_root=skill_path,
            output_dir=output_dir,
            inputs=parsed_inputs,
            task_id=task_id,
            project_root=Path.cwd(),
            backlog_path=backlog,
        )
        if brief_output:
            brief_output.parent.mkdir(parents=True, exist_ok=True)
            brief_output.write_text(brief, encoding="utf-8")
            click.echo(f"# brief written to {brief_output}", err=True)
        else:
            click.echo(brief)
        sys.exit(0)

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
            backlog_path=backlog,
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

    # Phase 3: memory audit emission (opt-in).
    if memory_emit:
        click.echo("")
        click.echo("# memory emission")
        if not memory_is_available(ctx.obj["skill_root"]):
            click.echo("  SKIPPED — memory not reachable (memory module not importable OR .cyberos/memory/store missing)")
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


@main.command("resume")
@click.argument("persona_workflow")
@click.option(
    "--output-dir",
    type=click.Path(file_okay=False, path_type=Path),
    required=True,
    help="Directory containing existing step output JSON files from a previous run.",
)
@click.option(
    "--fr-id",
    required=True,
    help="FR to resume (e.g. TASK-MEMORY-117).",
)
@click.option(
    "--invoker",
    type=click.Choice(["auto", "subprocess", "llm"]),
    default="auto",
    help="Skill invoker for remaining steps.",
)
@click.option(
    "--backlog",
    type=click.Path(exists=True, dir_okay=False, path_type=Path),
    default=None,
    help="Path to BACKLOG.md for FR status lookup.",
)
@click.pass_context
def cmd_resume(
    ctx: click.Context,
    persona_workflow: str,
    output_dir: Path,
    task_id: str,
    invoker: str,
    backlog: Path | None,
) -> None:
    """Resume a workflow chain from existing step output files.

    Scans OUTPUT_DIR for previously completed step JSON files (stepNN_*.json),
    rebuilds the hand-off map, and continues execution from the next uncompleted
    step in the chain.
    """
    if "/" not in persona_workflow:
        click.echo("error: PERSONA_WORKFLOW must be <persona-slug>/<workflow-slug>", err=True)
        sys.exit(2)
    persona_slug, workflow_slug = persona_workflow.split("/", 1)

    cuo_path = ctx.obj["cuo_root"]
    skill_path = ctx.obj["skill_root"]
    personas = discover_personas(cuo_path)
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        click.echo(f"error: persona {persona_slug!r} not found", err=True)
        sys.exit(2)

    # Show what we found
    existing = sorted(output_dir.glob(f"step*_*.json"))
    click.echo(f"# resume {persona_workflow}")
    click.echo(f"  output dir: {output_dir}")
    click.echo(f"  task_id:      {task_id}")
    click.echo(f"  existing:   {len(existing)} step file(s)")
    for f in existing:
        click.echo(f"    {f.name}")

    if invoker == "llm":
        inv = LLMInvoker()
    elif invoker == "auto":
        from cuo.core.invoker import detect_host_environment
        _host_env = detect_host_environment()
        if _host_env:
            click.echo(
                f"# auto-detected host environment: {_host_env} — "
                "use `--invoker=llm` or `--invoker=subprocess` to continue execution",
                err=True,
            )
            click.echo("# cannot resume with brief invoker — host LLM must complete remaining steps manually", err=True)
            sys.exit(1)
        inv = select_invoker(invoker)
    else:
        inv = select_invoker(invoker)

    result = execute_chain(
        persona=persona,
        workflow_slug=workflow_slug,
        skill_root=skill_path,
        output_dir=output_dir,
        task_id=task_id,
        inputs={"task_id": task_id, "auto_claim": True},
        invoker=inv,
        stop_on_failure=True,
        backlog_path=backlog,
    )
    click.echo(f"\n# outcome: {result.outcome} ({len(result.step_results)} steps, {result.total_duration_ms} ms)")
    for sr in result.step_results:
        click.echo(f"  step {sr.step:2d} [{sr.status:7s}] {sr.skill}")
    sys.exit(0 if result.outcome == "COMPLETED" else 1)


@main.command("drain")
@click.argument("persona_workflow")
@click.option(
    "--output-dir",
    type=click.Path(file_okay=False, path_type=Path),
    required=True,
    help="Directory where per-FR run artefacts are written. One subdir per FR.",
)
@click.option(
    "--module",
    "module_filter",
    default=None,
    help="Filter FRs by module (e.g. 'memory'). Only FRs whose ID matches "
         "FR-<MODULE>-<NNN> with matching module slug are picked.",
)
@click.option(
    "--backlog",
    "backlog_path",
    type=click.Path(exists=True, dir_okay=False, path_type=Path),
    default=None,
    help="Path to BACKLOG.md. Defaults to <cyberos_root>/docs/tasks/BACKLOG.md.",
)
@click.option(
    "--max-frs",
    type=int,
    default=0,
    help="Max FRs to drain in this run. 0 = unbounded (until empty or HITL halt).",
)
@click.option(
    "--invoker",
    type=click.Choice(["auto", "subprocess", "llm", "brief"]),
    default="auto",
)
@click.option("--memory-emit/--no-memory-emit", default=True)
@click.option("--actor", default="cuo-drain")
@click.option(
    "--halt-on-repeat-rework",
    type=int,
    default=2,
    help="Halt the drain loop when an FR routes back this many times "
         "(default 2). Set 0 to disable.",
)
@click.option(
    "--rework/--no-rework",
    default=False,
    help="Rework mode: allow selecting done FRs and re-running them from implementing to done.",
)
@click.option(
    "--brief-output",
    type=click.Path(dir_okay=False, path_type=Path),
    default=None,
    help="Write execution brief(s) to FILE instead of stdout (brief mode only).",
)
@click.pass_context
def cmd_drain(
    ctx: click.Context,
    persona_workflow: str,
    output_dir: Path,
    module_filter: str | None,
    backlog_path: Path | None,
    max_frs: int,
    invoker: str,
    memory_emit: bool,
    actor: str,
    halt_on_repeat_rework: int,
    rework: bool,
    brief_output: Path | None,
) -> None:
    """Drain a module's BACKLOG by running PERSONA_WORKFLOW on each eligible FR."""
    from cuo.api import run
    run(
        persona_workflow,
        output_dir=output_dir,
        module=module_filter,
        backlog_path=backlog_path,
        max_frs=max_frs,
        invoker=invoker,
        memory_emit=memory_emit,
        actor=actor,
        halt_on_repeat_rework=halt_on_repeat_rework,
        rework=rework,
        brief_output=brief_output,
        cuo_root=ctx.obj["cuo_root"],
        skill_root=ctx.obj["skill_root"],
    )


@main.group("harness")
@click.pass_context
def cmd_harness(ctx: click.Context) -> None:
    """Continuous-improvement harness (TASK-CUO-200..203)."""
    pass


@cmd_harness.command("report")
@click.option("--since", default="7d", help="Time window: 24h, 7d, 30d, 4w.")
@click.option("--skill", "skill_filter", default=None,
              help="Limit to one skill (slug).")
@click.option("--workflow", "workflow_filter", default=None,
              help="Limit workflow section to one id.")
@click.option(
    "--out",
    "out_path",
    type=click.Path(dir_okay=False, path_type=Path),
    default=None,
    help="Output markdown path. Defaults to docs/harness/harness-report-<date>.md.",
)
@click.option(
    "--memory-root",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
    help="Path to .cyberos/memory/store/ for audit-row emission. "
         "If unset, the report is written but no audit row is emitted.",
)
@click.option("--watch", is_flag=True, default=False,
              help="Re-emit the report every N seconds (default 300).")
@click.option("--watch-interval", type=int, default=300,
              help="Watch interval in seconds (default 300).")
@click.pass_context
def cmd_harness_report(
    ctx: click.Context,
    since: str,
    skill_filter: str | None,
    workflow_filter: str | None,
    out_path: Path | None,
    memory_root: Path | None,
    watch: bool,
    watch_interval: int,
) -> None:
    """Build the TASK-CUO-200 daily report and write it to disk."""
    from cuo.core.harness import compute_report, emit_report, parse_window
    import time as _time

    window = parse_window(since)
    cyberos_root = ctx.obj["cuo_root"].parent.parent  # modules/cuo → cyberos
    if out_path is None:
        out_path = (cyberos_root / "docs" / "harness"
                    / f"harness-report-{__import__('datetime').date.today().isoformat()}.md")
    if memory_root is None:
        candidate = cyberos_root / ".cyberos/memory/store"
        if (candidate / "manifest.json").is_file():
            memory_root = candidate

    audit_dir = memory_root / "audit" if memory_root else None
    skill_root = ctx.obj["skill_root"]

    def _emit_once() -> None:
        report = compute_report(
            audit_dir=audit_dir, skill_root=skill_root, window=window,
            skill_filter=skill_filter, workflow_filter=workflow_filter,
        )
        written = emit_report(report, out_path, memory_root=memory_root)
        click.echo(f"# harness report → {written}")
        click.echo(f"  window:           {since}")
        click.echo(f"  rows walked:      {report.total_rows_walked}")
        click.echo(f"  skills inspected: {report.skills_inspected}")
        click.echo(f"  breaches:         {len(report.breaches)}")
        click.echo(f"  workflows:        {len(report.workflow_metrics)}")
        for b in report.breaches[:5]:
            click.echo(f"    ⚠ {b.skill_name}::{b.signal_id}  "
                       f"value={b.value:.3f} thr={b.threshold:.3f}")

    _emit_once()
    if watch:
        click.echo(f"\n# watching (interval={watch_interval}s) — Ctrl-C to stop")
        while True:
            try:
                _time.sleep(watch_interval)
            except KeyboardInterrupt:
                click.echo("\n# watch interrupted")
                break
            _emit_once()


@main.group("proposal")
@click.pass_context
def cmd_proposal(ctx: click.Context) -> None:
    """Refinement-proposal operator workflow (TASK-CUO-201)."""
    pass


def _proposals_root(ctx: click.Context) -> Path:
    """Locate docs/proposals/ under the cyberos root."""
    cyberos_root = ctx.obj["cuo_root"].parent.parent
    return cyberos_root / "docs" / "proposals"


@cmd_proposal.command("list")
@click.option("--status",
              type=click.Choice(["all", "open", "applied", "rejected", "pending_approval"]),
              default="all")
@click.pass_context
def cmd_proposal_list(ctx: click.Context, status: str) -> None:
    """List refinement proposals by status (TASK-CUO-201 AC #6)."""
    from cuo.core.refinement_proposal import list_proposals
    listing = list_proposals(_proposals_root(ctx))
    statuses = (["open", "pending_approval", "applied", "rejected"]
                if status == "all" else [status])
    total = 0
    for st in statuses:
        files = listing.get(st, [])
        click.echo(f"# {st}: {len(files)}")
        for f in files:
            stripe = f.stem.rsplit("-", 1)[0]
            mtime = datetime.fromtimestamp(f.stat().st_mtime, tz=timezone.utc)
            click.echo(f"  {stripe}    {mtime.isoformat()}    {f.name}")
        total += len(files)
    click.echo(f"\n# total: {total}")


@cmd_proposal.command("show")
@click.argument("stripe_id")
@click.pass_context
def cmd_proposal_show(ctx: click.Context, stripe_id: str) -> None:
    """Print a proposal's markdown to stdout."""
    root = _proposals_root(ctx)
    for sub in ("open", "pending_approval", "applied", "rejected"):
        d = root / sub
        if not d.is_dir():
            continue
        matches = sorted(d.glob(f"{stripe_id}*.md"))
        if matches:
            click.echo(matches[0].read_text(encoding="utf-8"))
            return
    click.echo(f"error: no proposal matching {stripe_id!r}", err=True)
    sys.exit(2)


@cmd_proposal.command("apply")
@click.argument("stripe_id")
@click.pass_context
def cmd_proposal_apply(ctx: click.Context, stripe_id: str) -> None:
    """Move an open proposal to applied/ (lifecycle only; TASK-CUO-202 wires diff)."""
    from cuo.core.refinement_proposal import apply_proposal_lifecycle
    applied = apply_proposal_lifecycle(_proposals_root(ctx), stripe_id)
    if applied is None:
        click.echo(f"error: no open proposal matching {stripe_id!r}", err=True)
        sys.exit(2)
    click.echo(f"# applied {stripe_id} → {applied}")


@cmd_proposal.command("reject")
@click.argument("stripe_id")
@click.option("--reason", required=True, help="Why this proposal is rejected.")
@click.pass_context
def cmd_proposal_reject(ctx: click.Context, stripe_id: str, reason: str) -> None:
    """Reject an open proposal — append rationale + move to rejected/."""
    from cuo.core.refinement_proposal import reject_proposal
    rejected = reject_proposal(_proposals_root(ctx), stripe_id, reason)
    if rejected is None:
        click.echo(f"error: no open proposal matching {stripe_id!r}", err=True)
        sys.exit(2)
    click.echo(f"# rejected {stripe_id} → {rejected}")


@cmd_proposal.command("approve")
@click.argument("stripe_id")
@click.pass_context
def cmd_proposal_approve(ctx: click.Context, stripe_id: str) -> None:
    """Move a pending_approval proposal to applied/ (gated HITL step)."""
    from cuo.core.refinement_proposal import approve_proposal
    applied = approve_proposal(_proposals_root(ctx), stripe_id)
    if applied is None:
        click.echo(f"error: no pending_approval proposal matching {stripe_id!r}", err=True)
        sys.exit(2)
    click.echo(f"# approved {stripe_id} → {applied}")


if __name__ == "__main__":
    main()
