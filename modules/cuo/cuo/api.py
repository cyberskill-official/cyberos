"""Pure-Python API for CUO workflow operations.

Importable functions that wrap the core CUO logic without Click dependency.
Used by both ``cyberos-cuo`` (via cli.py) and ``cyberos workflow`` (via __main__.py).
"""

from __future__ import annotations

import sys
from pathlib import Path

from cuo.core.catalog import discover_personas, discover_workflows
from cuo.core.invoker import select_invoker
from cuo.core.llm_invoker import LLMInvoker
from cuo.core.supervisor import dry_run_chain, execute_chain


def _resolve(cuo_root: Path | None, skill_root: Path | None) -> tuple[Path, Path]:
    """Resolve cuo/ + skill/ roots, falling back to CWD walk."""
    from cuo.cli import _resolve_roots
    return _resolve_roots(cuo_root, skill_root)


def list_personas(
    *,
    show_extinct: bool = False,
    show_planned: bool = True,
    cuo_root: Path | None = None,
) -> None:
    """Enumerate persona folders — shipped, planned, extinct."""
    cuo_path, _ = _resolve(cuo_root, None)
    personas = discover_personas(cuo_path)
    n_total = len(personas)
    n_shipped = sum(1 for p in personas if p.has_workflows)
    n_extinct = sum(1 for p in personas if p.is_extinct)
    print(f"# {n_shipped} shipped + {n_total - n_shipped - n_extinct} planned + {n_extinct} extinct = {n_total} total")

    for p in personas:
        if p.is_extinct and not show_extinct:
            continue
        if not p.has_workflows and not p.is_extinct and not show_planned:
            continue
        flag = "EXTINCT" if p.is_extinct else ("shipped" if p.has_workflows else "planned")
        n_wf = len(list(p.workflows_dir.glob("*.md"))) if p.workflows_dir.is_dir() else 0
        title_suffix = f" — {p.disambiguated_title}" if p.disambiguated_title else ""
        print(f"  {p.slug:35s} [{flag:8s}] {n_wf} workflows{title_suffix}")


def list_workflows(
    persona_slug: str,
    *,
    cuo_root: Path | None = None,
    skill_root: Path | None = None,
) -> None:
    """Enumerate workflows for a persona with validation status."""
    from cuo.core.validator import validate_chain

    cuo_path, skill_path = _resolve(cuo_root, skill_root)
    personas = discover_personas(cuo_path)
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        print(f"error: persona {persona_slug!r} not found", file=sys.stderr)
        sys.exit(2)

    workflows = discover_workflows(persona)
    if not workflows:
        print(f"# {persona_slug}: no workflows shipped")
        return

    print(f"# {persona_slug}: {len(workflows)} workflow(s)")
    for wf in workflows:
        v = validate_chain(wf, skill_path)
        valid_flag = "valid" if v.valid else f"BLOCKED ({len(v.missing_skills)} missing + {len(v.planned_skills)} planned)"
        print(
            f"  {wf.slug:50s} cadence={wf.cadence:10s} chain={v.chain_length:2d} [{valid_flag}]"
        )


def dry_run(
    persona_workflow: str,
    *,
    cuo_root: Path | None = None,
    skill_root: Path | None = None,
) -> None:
    """Plan (but do not execute) a workflow's skill chain."""
    if "/" not in persona_workflow:
        print("error: PERSONA_WORKFLOW must be <persona-slug>/<workflow-slug>", file=sys.stderr)
        sys.exit(2)

    persona_slug, workflow_slug = persona_workflow.split("/", 1)
    cuo_path, skill_path = _resolve(cuo_root, skill_root)
    personas = discover_personas(cuo_path)
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        print(f"error: persona {persona_slug!r} not found", file=sys.stderr)
        sys.exit(2)

    result = dry_run_chain(persona, workflow_slug, skill_path)
    flag = "RUNNABLE" if result.runnable else "BLOCKED"
    print(f"# dry-run {result.workflow_id} → {flag}")
    print(f"  chain length: {result.validation.chain_length}")
    print(f"  found:        {len(result.validation.found_skills)}")
    print(f"  missing:      {len(result.validation.missing_skills)}")
    print(f"  planned:      {len(result.validation.planned_skills)}")
    print("")
    print("# step plan")
    for line in result.step_plan:
        print(f"  {line}")
    if result.notes:
        print("")
        print("# notes")
        for note in result.notes:
            print(f"  * {note}")
    sys.exit(0 if result.runnable else 1)


def run(
    persona_workflow: str,
    *,
    output_dir: Path | None = None,
    module: str | None = None,
    backlog_path: Path | None = None,
    max_frs: int = 0,
    invoker: str = "auto",
    memory_emit: bool = True,
    actor: str = "cuo-drain",
    halt_on_repeat_rework: int = 2,
    rework: bool = False,
    brief_output: Path | None = None,
    cuo_root: Path | None = None,
    skill_root: Path | None = None,
) -> None:
    """Drain eligible FRs through a persona workflow.

    This is the high-level zero-touch entry point. Reads BACKLOG.md,
    finds eligible FRs, and runs the workflow on each one.
    """
    from cuo.core.backlog_reader import (
        parse_backlog, next_eligible, routed_back_count,
    )

    if "/" not in persona_workflow:
        print("error: PERSONA_WORKFLOW must be <persona-slug>/<workflow-slug>", file=sys.stderr)
        sys.exit(2)
    persona_slug, workflow_slug = persona_workflow.split("/", 1)

    cuo_path, skill_path = _resolve(cuo_root, skill_root)
    personas = discover_personas(cuo_path)
    persona = next((p for p in personas if p.slug == persona_slug), None)
    if persona is None:
        print(f"error: persona {persona_slug!r} not found", file=sys.stderr)
        sys.exit(2)

    # Resolve BACKLOG path
    if backlog_path is None:
        cyberos_root = cuo_path.parent.parent  # modules/cuo → cyberos
        backlog_path = cyberos_root / "docs" / "feature-requests" / "BACKLOG.md"
    if not backlog_path.is_file():
        print(f"error: BACKLOG.md not found at {backlog_path}", file=sys.stderr)
        sys.exit(2)

    audit_dir = cuo_path.parent.parent / ".cyberos-memory" / "audit"

    if output_dir is None:
        # Backlog lives at docs/feature-requests/BACKLOG.md — walk up 3 levels
        # to reach the project root for .cyberos-memory/ placement.
        project_root = backlog_path.parent.parent.parent
        output_dir = project_root / ".cyberos-memory" / "cuo-steps"
    output_dir.mkdir(parents=True, exist_ok=True)
    halt_reason_path = output_dir / "DRAIN_HALT.md"

    # Auto-detect host LLM environment → brief mode (disabled: user wants real execution)
    # if invoker == "auto":
    #     from cuo.core.invoker import detect_host_environment
    #     _host_env = detect_host_environment()
    #     if _host_env:
    #         invoker = "brief"

    if invoker == "llm":
        inv = LLMInvoker()
        inv_kind = f"LLMInvoker(mode={inv.mode})"
    elif invoker == "brief":
        inv = None
        inv_kind = "BriefGenerator"
    else:
        inv = select_invoker(invoker)
        inv_kind = type(inv).__name__

    print(f"# drain {persona_workflow}")
    print(f"  module filter:  {module or '(none — all modules)'}")
    print(f"  invoker:        {inv_kind}")
    print(f"  memory emit:    {memory_emit}")
    print(f"  halt on repeat rework count: {halt_on_repeat_rework}")
    print("")

    # Brief mode: generate execution brief(s) instead of executing.
    if invoker == "brief":
        from cuo.core.supervisor import brief_chain
        frs_run = 0
        processed_fr_ids: set[str] = set()
        while True:
            if max_frs and frs_run >= max_frs:
                break
            rows = parse_backlog(backlog_path)
            eligible = next_eligible(rows, module=module, rework=rework,
                                      skip_fr_ids=processed_fr_ids)
            if eligible is None:
                break
            frs_run += 1
            processed_fr_ids.add(eligible.fr_id)
            brief = brief_chain(
                persona=persona,
                workflow_slug=workflow_slug,
                skill_root=skill_path,
                output_dir=output_dir,
                fr_id=eligible.fr_id,
                inputs={"fr_id": eligible.fr_id, "module": module or eligible.module,
                        "rework": rework},
                project_root=Path.cwd(),
            )
            if brief_output:
                brief_output.parent.mkdir(parents=True, exist_ok=True)
                if max_frs != 1 and frs_run > 1:
                    # Multiple FRs: append with separator
                    with open(brief_output, "a", encoding="utf-8") as f:
                        f.write(f"\n\n---\n\n## [{frs_run}] {eligible.fr_id}\n\n")
                        f.write(brief)
                else:
                    brief_output.write_text(brief, encoding="utf-8")
            elif max_frs == 1:
                print(brief)
            else:
                print(f"## [{frs_run}] {eligible.fr_id}")
                print(brief)
                print("")
        if brief_output:
            print(f"# brief written to {brief_output}")
        print(f"# brief summary: {frs_run} brief(s) generated")
        sys.exit(0)

    frs_run = 0
    completed = 0
    routed_back = 0
    processed_fr_ids: set[str] = set()
    while True:
        if max_frs and frs_run >= max_frs:
            print(f"# drain halted: --max-frs={max_frs} reached")
            break
        rows = parse_backlog(backlog_path)
        eligible = next_eligible(rows, module=module, rework=rework,
                                  skip_fr_ids=processed_fr_ids)
        if eligible is None:
            print(f"# drain complete: no more eligible FRs"
                  f"{' in module=' + module if module else ''}")
            break

        fr_output_dir = output_dir
        print(f"## [{frs_run + 1}] {eligible.fr_id} — {eligible.title[:60]}")

        result = execute_chain(
            persona=persona,
            workflow_slug=workflow_slug,
            skill_root=skill_path,
            output_dir=fr_output_dir,
            fr_id=eligible.fr_id,
            inputs={"fr_id": eligible.fr_id, "auto_claim": True,
                    "module": module or eligible.module,
                    "rework": rework},
            invoker=inv,
            stop_on_failure=True,
        )
        frs_run += 1
        processed_fr_ids.add(eligible.fr_id)
        print(f"   outcome: {result.outcome}  ({len(result.step_results)} steps, "
              f"{result.total_duration_ms} ms)")

        if memory_emit:
            from cuo.core.memory_bridge import memory_is_available, emit_chain_result
            if memory_is_available(skill_path):
                emit_chain_result(result, skill_path, actor=actor)

        if result.outcome == "COMPLETED":
            completed += 1
        elif result.outcome == "ROUTED_BACK":
            routed_back += 1
            rbc = routed_back_count(eligible.fr_id, audit_dir)
            print(f"   routed_back_count for {eligible.fr_id}: {rbc}")
            if halt_on_repeat_rework and rbc >= halt_on_repeat_rework:
                print(f"# drain HALTED: {eligible.fr_id} routed back "
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
            print(f"# drain HALTED: {eligible.fr_id} hit {result.outcome}")
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

    print("")
    print(f"# drain summary: {frs_run} FRs run, {completed} completed, "
          f"{routed_back} routed back")
    sys.exit(0 if frs_run > 0 else 0)
