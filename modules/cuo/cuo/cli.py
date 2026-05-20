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
    help="Force a specific FR (e.g. FR-MEMORY-117). Shorthand for `--input fr_id=<value>`. "
         "Used by ship-feature-requests to target one FR rather than picking from BACKLOG.",
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
    fr_id: str | None,
    auto_claim: bool,
    rework: bool,
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

    # --fr-id is a typed shorthand for --input fr_id=<value>.
    if fr_id is not None:
        parsed_inputs["fr_id"] = fr_id
    # --auto-claim flag flows into the workflow's input bundle so phase
    # transitions can read it from the hand-off map.
    parsed_inputs["auto_claim"] = auto_claim
    parsed_inputs["rework"] = rework

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

    # Phase 3: memory audit emission (opt-in).
    if memory_emit:
        click.echo("")
        click.echo("# memory emission")
        if not memory_is_available(ctx.obj["skill_root"]):
            click.echo("  SKIPPED — memory not reachable (memory module not importable OR .cyberos-memory missing)")
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
    help="Path to BACKLOG.md. Defaults to <cyberos_root>/docs/feature-requests/BACKLOG.md.",
)
@click.option(
    "--max-frs",
    type=int,
    default=0,
    help="Max FRs to drain in this run. 0 = unbounded (until empty or HITL halt).",
)
@click.option(
    "--invoker",
    type=click.Choice(["auto", "mock", "subprocess", "llm"]),
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
) -> None:
    """Drain a module's BACKLOG by running PERSONA_WORKFLOW on each eligible FR.

    Designed for the "ship next eligible FR until empty or HITL needed" prompt
    pattern. The loop:

      1. Reads BACKLOG.md
      2. Finds the next FR in ready_to_implement with all deps done
        (optionally filtered to a single module via --module).
      3. Runs PERSONA_WORKFLOW with --fr-id=<id> and auto-claim on.
      4. On COMPLETED → next iteration.
      5. On ROUTED_BACK → check routed_back_count; if >= --halt-on-repeat-rework,
        write a halt-reason file and stop. Otherwise, continue.
      6. On HITL_HALT (escalation match, circuit-breaker trip): write halt
        reason, stop the loop.
      7. On BACKLOG drained (no more eligible FRs): clean exit.

    This is the high-level zero-touch entry point. Use plain `execute` for
    single-FR runs.
    """
    from cuo.core.backlog_reader import (
        parse_backlog, next_eligible, routed_back_count,
    )

    if "/" not in persona_workflow:
        click.echo("error: PERSONA_WORKFLOW must be <persona-slug>/<workflow-slug>", err=True)
        sys.exit(2)
    persona_slug, workflow_slug = persona_workflow.split("/", 1)

    personas = discover_personas(ctx.obj["cuo_root"])
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        click.echo(f"error: persona {persona_slug!r} not found", err=True)
        sys.exit(2)

    # Resolve BACKLOG path
    if backlog_path is None:
        cyberos_root = ctx.obj["cuo_root"].parent.parent  # modules/cuo → cyberos
        backlog_path = cyberos_root / "docs" / "feature-requests" / "BACKLOG.md"
    if not backlog_path.is_file():
        click.echo(f"error: BACKLOG.md not found at {backlog_path}", err=True)
        sys.exit(2)

    audit_dir = ctx.obj["cuo_root"].parent.parent / ".cyberos-memory" / "audit"

    output_dir.mkdir(parents=True, exist_ok=True)
    halt_reason_path = output_dir / "DRAIN_HALT.md"

    if invoker == "llm":
        inv = LLMInvoker()
        inv_kind = f"LLMInvoker(mode={inv.mode})"
    else:
        inv = select_invoker(invoker)
        inv_kind = type(inv).__name__

    click.echo(f"# drain {persona_workflow}")
    click.echo(f"  module filter:  {module_filter or '(none — all modules)'}")
    click.echo(f"  invoker:        {inv_kind}")
    click.echo(f"  memory emit:    {memory_emit}")
    click.echo(f"  halt on repeat rework count: {halt_on_repeat_rework}")
    click.echo("")

    frs_run = 0
    completed = 0
    routed_back = 0
    while True:
        if max_frs and frs_run >= max_frs:
            click.echo(f"# drain halted: --max-frs={max_frs} reached")
            break
        rows = parse_backlog(backlog_path)
        eligible = next_eligible(rows, module=module_filter, rework=rework)
        if eligible is None:
            click.echo(f"# drain complete: no more eligible FRs"
                       f"{' in module=' + module_filter if module_filter else ''}")
            break

        fr_output_dir = output_dir / eligible.fr_id
        fr_output_dir.mkdir(parents=True, exist_ok=True)
        click.echo(f"## [{frs_run + 1}] {eligible.fr_id} — {eligible.title[:60]}")

        from cuo.core.supervisor import execute_chain
        result = execute_chain(
            persona=persona,
            workflow_slug=workflow_slug,
            skill_root=ctx.obj["skill_root"],
            output_dir=fr_output_dir,
            inputs={"fr_id": eligible.fr_id, "auto_claim": True,
                    "module": module_filter or eligible.module,
                    "rework": rework},
            invoker=inv,
            stop_on_failure=True,
        )
        frs_run += 1
        click.echo(f"   outcome: {result.outcome}  ({len(result.step_results)} steps, "
                   f"{result.total_duration_ms} ms)")

        if memory_emit:
            from cuo.core.memory_bridge import memory_is_available, emit_chain_result
            if memory_is_available(ctx.obj["skill_root"]):
                emit_chain_result(result, ctx.obj["skill_root"], actor=actor)

        if result.outcome == "COMPLETED":
            completed += 1
        elif result.outcome == "ROUTED_BACK":
            routed_back += 1
            rbc = routed_back_count(eligible.fr_id, audit_dir)
            click.echo(f"   routed_back_count for {eligible.fr_id}: {rbc}")
            if halt_on_repeat_rework and rbc >= halt_on_repeat_rework:
                click.echo(f"# drain HALTED: {eligible.fr_id} routed back "
                           f"{rbc} times (>= --halt-on-repeat-rework={halt_on_repeat_rework})")
                halt_reason_path.write_text(
                    f"# DRAIN HALT\n\n"
                    f"FR `{eligible.fr_id}` has been rework-routed {rbc} times — "
                    f"likely a real spec issue. HITL inspection required.\n\n"
                    f"## Last attempt\n\n"
                    f"- outcome: {result.outcome}\n"
                    f"- notes: {result.notes}\n"
                    f"- output dir: {fr_output_dir}\n\n"
                    f"## Next action\n\n"
                    f"Review the FR spec + last debug trace. Either patch the spec, "
                    f"flip status to on_hold/closed, or override routed_back_count.\n",
                    encoding="utf-8",
                )
                sys.exit(2)
        elif result.outcome in ("FAILED", "HITL_HALT"):
            click.echo(f"# drain HALTED: {eligible.fr_id} hit {result.outcome}")
            halt_reason_path.write_text(
                f"# DRAIN HALT\n\n"
                f"FR `{eligible.fr_id}` outcome: **{result.outcome}**.\n\n"
                f"- notes: {result.notes}\n"
                f"- step results:\n"
                + "\n".join(f"  - step {s.step} [{s.status}] {s.skill}: "
                            f"{', '.join(s.notes[:2])}"
                            for s in result.step_results)
                + f"\n\n## Next action\n\nReview the failure, decide whether to "
                  f"patch + retry or mark on_hold.\n",
                encoding="utf-8",
            )
            sys.exit(2)

    click.echo("")
    click.echo(f"# drain summary: {frs_run} FRs run, {completed} completed, "
               f"{routed_back} routed back")
    sys.exit(0 if frs_run > 0 else 0)


@main.group("harness")
@click.pass_context
def cmd_harness(ctx: click.Context) -> None:
    """Continuous-improvement harness (FR-CUO-200..203)."""
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
    help="Path to .cyberos-memory/ for audit-row emission. "
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
    """Build the FR-CUO-200 daily report and write it to disk."""
    from cuo.core.harness import compute_report, emit_report, parse_window
    import time as _time

    window = parse_window(since)
    cyberos_root = ctx.obj["cuo_root"].parent.parent  # modules/cuo → cyberos
    if out_path is None:
        out_path = (cyberos_root / "docs" / "harness"
                    / f"harness-report-{__import__('datetime').date.today().isoformat()}.md")
    if memory_root is None:
        candidate = cyberos_root / ".cyberos-memory"
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
    """Refinement-proposal operator workflow (FR-CUO-201)."""
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
    """List refinement proposals by status (FR-CUO-201 AC #6)."""
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
    """Move an open proposal to applied/ (lifecycle only; FR-CUO-202 wires diff)."""
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
