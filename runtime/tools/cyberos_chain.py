#!/usr/bin/env python3
"""
cyberos_chain.py — operator umbrella for the requirements → tasks chain.

S7.1 + S1.4 of skills-Stage-1 improvements (Batch 16).

Wraps the CPO/CTO skill chain in a single command. Default profile is
`solo` (skip PRD when the spec is concrete, collapse FR+spec into
fr-with-tasks, optionally create proj-tracker tickets).

Subcommands:
    cyberos chain run --pitch "<text>" [--profile solo|lean|standard|full]
                      [--output <dir>] [--skip-prd auto|force|never]
                      [--with-llm] [--dry-run]
    cyberos chain status [<output-dir>]
    cyberos chain resume <output-dir>
    cyberos chain estimate --pitch "<text>" --profile <p>
    cyberos chain graph <output-dir>

For solo profile, the chain is:
    spec (NL / PRD / SRS) → [optional prd-author] → fr-with-tasks → [optional fr-audit]
                                                                  → [optional proj sync]

This tool DRIVES the chain. It does NOT itself author FRs (that's the
skill's job once a real runtime exists). Today it produces a
chain-plan manifest + skeleton FR drafts that an LLM-fronted operator
fills in. With `--with-llm` + anthropic SDK + API key it actually calls
Claude for the bulk of the authoring.
"""
from __future__ import annotations
import argparse
import json
import os
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def slugify(s: str, n: int = 40) -> str:
    s = re.sub(r"[^a-z0-9\s-]", "", s.lower())
    s = re.sub(r"[\s_]+", "-", s).strip("-")
    return s[:n] or "untitled"


def output_dir_for(brain_root: Path, slug: str) -> Path:
    today = datetime.now(ICT).strftime("%Y-%m-%d")
    return brain_root / "planning" / f"{today}-{slug}"


# ----- Triage: should we skip PRD authoring? -----

def triage_skip_prd(spec_text: str) -> tuple[bool, list[str]]:
    """Return (can_skip, reasons). Skips when NL spec is already concrete."""
    reasons = []
    # ≥5 concrete acceptance criteria
    crit_pattern = re.compile(r"^(?:#{1,6}\s*)?(acceptance|criteria|success\s*criteria|done\s*when|measurable)",
                              re.IGNORECASE | re.MULTILINE)
    crits = len(crit_pattern.findall(spec_text))
    reasons.append(f"acceptance-headers: {crits}/5")
    crit_ok = crits >= 5

    # ≥1 measurable success metric (numeric + unit)
    metric_pattern = re.compile(r"\d+\s*(%|ms|sec|s|min|h|days?|weeks?|users?|requests?|qps|MB|KB|GB)\b", re.IGNORECASE)
    metrics = len(metric_pattern.findall(spec_text))
    reasons.append(f"measurable-metrics: {metrics}/1")
    metric_ok = metrics >= 1

    # Primary user / persona explicit
    persona_pattern = re.compile(r"(primary\s*(user|persona)|target\s*audience|user\s*persona|as\s+a[n]?\s+\w+\s*,\s*i\s+(want|need))",
                                 re.IGNORECASE)
    persona_found = bool(persona_pattern.search(spec_text))
    reasons.append(f"primary-persona-mentioned: {persona_found}")

    can_skip = crit_ok and metric_ok and persona_found
    return can_skip, reasons


# ----- Profile → chain plan -----

PROFILE_CHAINS = {
    "solo": ["fr-with-tasks", "fr-audit"],
    "lean": ["prd-author", "fr-author", "fr-audit", "spec-to-impl-plan"],
    "standard": ["prd-author", "prd-audit", "fr-author", "fr-audit", "fr-to-tech-spec", "spec-to-impl-plan"],
    "full":     ["prd-author", "prd-audit", "srs-author", "srs-audit", "fr-author", "fr-audit", "fr-to-tech-spec", "spec-to-impl-plan"],
}


def build_chain_plan(profile: str, skip_prd: bool, with_llm: bool) -> list[dict]:
    skills = list(PROFILE_CHAINS[profile])
    plan = []
    for i, s in enumerate(skills):
        skip = (s in ("prd-author", "prd-audit") and skip_prd and profile == "solo")
        plan.append({
            "step": i + 1,
            "skill_id": f"cuo/cpo/{s}" if s.startswith(("prd-", "fr-")) else f"cuo/cto/{s}",
            "status": "skipped" if skip else "pending",
            "skipped_reason": "skip_prd triage passed" if skip else None,
        })
    return plan


# ----- Estimate tokens / hours -----

PROFILE_TOKEN_ESTIMATES = {
    "solo":     {"min": 8000,  "max": 25000},
    "lean":     {"min": 15000, "max": 45000},
    "standard": {"min": 30000, "max": 90000},
    "full":     {"min": 60000, "max": 180000},
}


def cmd_run(args):
    brain_root = find_brain()
    pitch = args.pitch.strip()
    if not pitch:
        print("  ✗ --pitch is required", file=sys.stderr); return 2

    slug = slugify(pitch[:60])
    out_dir = Path(args.output) if args.output else output_dir_for(brain_root, slug)
    out_dir.mkdir(parents=True, exist_ok=True)

    # Triage skip-PRD?
    spec_text = pitch
    if args.spec_file:
        spec_text = Path(args.spec_file).read_text(encoding="utf-8")
    skip_prd, triage_reasons = (False, ["forced off"])
    if args.skip_prd == "auto":
        skip_prd, triage_reasons = triage_skip_prd(spec_text)
    elif args.skip_prd == "force":
        skip_prd, triage_reasons = True, ["forced on"]

    plan = build_chain_plan(args.profile, skip_prd, args.with_llm)

    # Write the chain manifest
    manifest = {
        "schema_version": 1,
        "created_at": datetime.now(ICT).isoformat(timespec="seconds"),
        "profile": args.profile,
        "skip_prd": skip_prd,
        "triage_reasons": triage_reasons,
        "slug": slug,
        "output_dir": str(out_dir),
        "pitch_first_120": pitch[:120],
        "spec_file": args.spec_file,
        "with_llm": args.with_llm,
        "plan": plan,
        "status": "PLANNED",
    }
    manifest_path = out_dir / "chain-manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

    try:
        out_display = out_dir.relative_to(brain_root)
    except ValueError:
        out_display = out_dir
    print(f"\n  cyberos chain — profile: {args.profile}")
    print(f"  output: {out_display}")
    print(f"  skip_prd: {skip_prd}  (triage: {', '.join(triage_reasons)})")
    print(f"\n  Plan:")
    for s in plan:
        marker = "·" if s["status"] == "pending" else "—"
        print(f"    {marker} step {s['step']}  {s['skill_id']}  [{s['status']}]")

    if args.dry_run:
        print(f"\n  (dry-run) manifest: {manifest_path.relative_to(brain_root)}")
        return 0

    # Run the chain.
    manifest["budget"] = {
        "max_tokens": args.max_tokens, "max_cost_usd": args.max_cost,
        "tokens_used_total": 0, "cost_usd_total": 0.0,
    }
    # Tier α — try to load deterministic runner per step
    import sys as _sys
    _sys.path.insert(0, str(brain_root / "runtime" / "skill_runners"))
    try:
        from base import load_runner, SkillCache  # type: ignore
        _runner_cache = SkillCache() if not args.no_cache else None
    except ImportError:
        load_runner = None
        _runner_cache = None

    print(f"\n  Executing chain… ({'LIVE LLM' if args.with_llm else 'placeholders'})\n")
    for step in plan:
        if step["status"] == "skipped":
            print(f"  [{step['step']}] {step['skill_id']}  SKIPPED ({step['skipped_reason']})")
            continue

        skill_name = step["skill_id"].split("/")[-1]
        # Tier α.1 — prefer deterministic runner when available
        runner = load_runner(step["skill_id"], brain_root) if (load_runner and args.with_llm) else None
        if runner is not None and args.with_llm:
            print(f"  [{step['step']}] {step['skill_id']}  via runner …", flush=True)
            step["status"] = "in_progress"
            step["started_at"] = datetime.now(ICT).isoformat(timespec="seconds")
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
            runner.model = args.model
            runner.step_max_tokens = args.step_max_tokens
            try:
                result = runner.run(
                    inputs={"pitch": pitch, "spec_text": spec_text if args.spec_file else "",
                            "spec_file": args.spec_file},
                    output_dir=out_dir,
                    max_iterations=args.max_iterations,
                    cache=_runner_cache,
                )
            except Exception as e:
                step["status"] = "failed"
                step["error"] = str(e)[:200]
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
                print(f"     ✗ runner failed: {e}")
                return 2
            step["status"] = result.status.lower() if result.status == "PASS" else ("hitl_paused" if result.status == "HITL_PAUSE" else "exhausted")
            if result.status == "PASS":
                step["status"] = "done"
            step["completed_at"] = datetime.now(ICT).isoformat(timespec="seconds")
            step["iterations"] = result.iterations
            step["tokens_used"] = result.tokens_used
            step["cost_usd"] = round(result.cost_usd, 6)
            step["findings"] = result.findings[:10]
            if result.artefact_path:
                step["output_paths"] = [str(result.artefact_path.relative_to(brain_root))]
            manifest["budget"]["tokens_used_total"] += result.tokens_used
            manifest["budget"]["cost_usd_total"] = round(manifest["budget"]["cost_usd_total"] + result.cost_usd, 6)
            print(f"     {result.status} after {result.iterations} iter(s); {result.tokens_used} tok; ${result.cost_usd:.4f}")
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
            if result.status != "PASS":
                manifest["status"] = "HITL_PAUSE" if result.status == "HITL_PAUSE" else "EXHAUSTED"
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
                return 1
            continue

        if args.with_llm:
            # Budget check
            if manifest["budget"]["tokens_used_total"] >= manifest["budget"]["max_tokens"]:
                print(f"  [{step['step']}] {step['skill_id']}  BUDGET EXCEEDED — pausing")
                step["status"] = "hitl_paused"
                step["hitl_question"] = "Budget exceeded; increase or abort?"
                manifest["status"] = "HITL_PAUSE"
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
                return 1

            # Load skill's SKILL.md as context
            skill_dir = brain_root / "docs" / "skills" / step["skill_id"]
            skill_md = skill_dir / "SKILL.md"
            if not skill_md.exists():
                # Try alternative paths
                for cand in (brain_root / "docs" / "skills").rglob("SKILL.md"):
                    if cand.parent.name == skill_name:
                        skill_md = cand; break

            skill_md_text = skill_md.read_text(encoding="utf-8") if skill_md.exists() else "(SKILL.md not found)"
            prev_artefacts = ""
            for prev in plan[:plan.index(step)]:
                if prev["status"] == "done":
                    pname = prev["skill_id"].split("/")[-1]
                    for f in out_dir.glob(f"{pname}*.md"):
                        prev_artefacts += f"\n\n--- prior artefact: {f.name} ---\n\n" + f.read_text(encoding="utf-8")[:4000]

            prompt = f"""You are executing the cyberos skill `{step['skill_id']}` to produce its declared artefact.

# SKILL.md
{skill_md_text[:8000]}

# Input pitch
{pitch}

{f"# Spec file content{chr(10)}{spec_text[:6000]}" if args.spec_file else ""}

{prev_artefacts if prev_artefacts else ""}

# Your task

Produce the artefact this skill is responsible for emitting. Follow the SKILL.md body shape exactly. Use YAML frontmatter where the skill prescribes it. No em dashes. No AI vocabulary (leverage, robust, ensure, comprehensive, seamless, delve, navigate, tapestry, etc.). Cite source_ref + authority markers per AGENTS.md §5.1. Wrap any quoted user input in <untrusted_content> blocks per §4.2.

Output ONLY the artefact body. No commentary, no markdown code fences around the whole thing."""

            print(f"  [{step['step']}] {step['skill_id']}  CALLING {args.model}…", flush=True)
            step["status"] = "in_progress"; step["started_at"] = datetime.now(ICT).isoformat(timespec="seconds")
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

            try:
                import anthropic  # type: ignore
                if not os.environ.get("ANTHROPIC_API_KEY"):
                    raise RuntimeError("ANTHROPIC_API_KEY not set")
                client = anthropic.Anthropic()
                msg = client.messages.create(
                    model=args.model, max_tokens=args.step_max_tokens,
                    messages=[{"role": "user", "content": prompt}],
                )
                body = "\n".join(b.text for b in msg.content if hasattr(b, "text"))
                in_tok = msg.usage.input_tokens
                out_tok = msg.usage.output_tokens
                # Sonnet pricing approx (operator can override via env)
                in_rate = float(os.environ.get("CYBEROS_INPUT_PER_MTOK", "3.0"))
                out_rate = float(os.environ.get("CYBEROS_OUTPUT_PER_MTOK", "15.0"))
                cost = in_tok / 1_000_000 * in_rate + out_tok / 1_000_000 * out_rate

                artefact_path = out_dir / f"{skill_name}.md"
                artefact_path.write_text(body, encoding="utf-8")

                step["status"] = "done"
                step["completed_at"] = datetime.now(ICT).isoformat(timespec="seconds")
                step["tokens_used"] = in_tok + out_tok
                step["cost_usd"] = round(cost, 6)
                step["output_paths"] = [str(artefact_path.relative_to(brain_root))]
                manifest["budget"]["tokens_used_total"] += step["tokens_used"]
                manifest["budget"]["cost_usd_total"] = round(manifest["budget"]["cost_usd_total"] + cost, 6)
                print(f"     ✓ wrote {artefact_path.name}  ({in_tok}+{out_tok} tok, ${cost:.4f})")
            except Exception as e:
                step["status"] = "failed"
                step["error"] = str(e)[:200]
                print(f"     ✗ failed: {e}")
                manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
                return 2
        else:
            # Placeholder mode (original behaviour)
            artefact = out_dir / f"{skill_name}.placeholder.md"
            artefact.write_text(
                f"# {skill_name} placeholder\n\n"
                f"Rerun `cyberos chain run --with-llm` to have Claude draft this artefact.\n\n"
                f"## Inputs\n\n- profile: {args.profile}\n- pitch: {pitch[:200]}…\n- spec_file: {args.spec_file}\n",
                encoding="utf-8",
            )
            step["status"] = "placeholder"
            step["output_paths"] = [str(artefact.relative_to(brain_root))]
            print(f"  [{step['step']}] {step['skill_id']}  placeholder written")

        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

    manifest["plan"] = plan
    if all(s["status"] in ("done", "skipped") for s in plan):
        manifest["status"] = "DONE"
    elif any(s["status"] == "placeholder" for s in plan):
        manifest["status"] = "PLACEHOLDERS_WRITTEN"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"\n  ✓ chain complete. Manifest: {manifest_path.relative_to(brain_root)}")
    if manifest["budget"]["tokens_used_total"] > 0:
        print(f"  total: {manifest['budget']['tokens_used_total']} tokens, ${manifest['budget']['cost_usd_total']:.4f}")
    return 0


def cmd_resume(args):
    """S4.2 + Tier α.2 — pick up a paused / partial chain run from its manifest.

    With --with-llm, calls the same runner pipeline as `cyberos chain run`.
    Without, flips placeholders to done (legacy behaviour).
    """
    target = Path(args.output_dir)
    mf = target / "chain-manifest.json"
    if not mf.exists():
        print(f"  no chain-manifest.json in {target}", file=sys.stderr); return 2
    m = json.loads(mf.read_text())
    brain_root = find_brain()

    # Find first resumable step
    resumable = [s for s in m["plan"] if s["status"] in ("pending", "hitl_paused", "in_progress", "exhausted", "placeholder", "failed")]
    if not resumable:
        print(f"  ✓ all steps done; nothing to resume")
        return 0

    print(f"\n  Resuming {target.name}  profile={m['profile']}")
    print(f"  {len(resumable)} step(s) to run  ({'LIVE LLM' if args.with_llm else 'placeholder flip'})\n")

    # Tier α.2 — runner-backed resume
    if args.with_llm:
        sys.path.insert(0, str(brain_root / "runtime" / "skill_runners"))
        try:
            from base import load_runner, SkillCache  # type: ignore
        except ImportError:
            load_runner = None
            SkillCache = None
        cache = SkillCache() if (SkillCache and not args.no_cache) else None

    pitch = m.get("pitch_first_120", "")
    spec_file = m.get("spec_file")
    spec_text = ""
    if spec_file and Path(spec_file).exists():
        spec_text = Path(spec_file).read_text(encoding="utf-8")

    for step in resumable:
        # Budget check
        budget = m.get("budget", {})
        if budget.get("max_tokens", 0) and budget.get("tokens_used_total", 0) >= budget["max_tokens"]:
            step["status"] = "hitl_paused"
            step["hitl_question"] = "Budget exceeded (tokens). Increase budget or abort?"
            m["status"] = "HITL_PAUSE"
            mf.write_text(json.dumps(m, indent=2) + "\n")
            print(f"  ⚠ budget exceeded; paused at step {step['step']}")
            return 1
        # Mark in progress
        step["status"] = "in_progress"
        step["started_at"] = datetime.now(ICT).isoformat(timespec="seconds")
        mf.write_text(json.dumps(m, indent=2) + "\n")

        runner = load_runner(step["skill_id"], brain_root) if (args.with_llm and load_runner) else None
        if runner is not None:
            runner.model = args.model
            runner.step_max_tokens = args.step_max_tokens
            try:
                result = runner.run(
                    inputs={"pitch": pitch, "spec_text": spec_text, "spec_file": spec_file},
                    output_dir=target,
                    max_iterations=args.max_iterations,
                    cache=cache,
                )
            except Exception as e:
                step["status"] = "failed"; step["error"] = str(e)[:200]
                print(f"  [{step['step']}] {step['skill_id']}  FAILED: {e}")
                mf.write_text(json.dumps(m, indent=2) + "\n")
                return 2
            step["status"] = ({"PASS": "done", "HITL_PAUSE": "hitl_paused",
                               "EXHAUSTED": "exhausted", "FAILED": "failed"}[result.status])
            step["completed_at"] = datetime.now(ICT).isoformat(timespec="seconds")
            step["iterations"] = (step.get("iterations") or 0) + result.iterations
            step["tokens_used"] = (step.get("tokens_used") or 0) + result.tokens_used
            step["cost_usd"] = round((step.get("cost_usd") or 0.0) + result.cost_usd, 6)
            if result.artefact_path:
                step["output_paths"] = [str(result.artefact_path.relative_to(brain_root))]
            m["budget"]["tokens_used_total"] += result.tokens_used
            m["budget"]["cost_usd_total"] = round(m["budget"]["cost_usd_total"] + result.cost_usd, 6)
            print(f"  [{step['step']}] {step['skill_id']}  {result.status} ({result.iterations} iter, {result.tokens_used} tok)")
            mf.write_text(json.dumps(m, indent=2) + "\n")
            if result.status != "PASS":
                m["status"] = "HITL_PAUSE" if result.status == "HITL_PAUSE" else "EXHAUSTED"
                mf.write_text(json.dumps(m, indent=2) + "\n")
                return 1
        else:
            # Legacy behaviour: flip placeholder→done
            print(f"  [{step['step']}] {step['skill_id']}  resumed; marked done (no runner)")
            step["status"] = "done"
            step["completed_at"] = datetime.now(ICT).isoformat(timespec="seconds")
            step["iterations"] = step.get("iterations", 0) + 1
            mf.write_text(json.dumps(m, indent=2) + "\n")

    # Final pass: chain done?
    if all(s["status"] in ("done", "skipped") for s in m["plan"]):
        m["status"] = "DONE"
        mf.write_text(json.dumps(m, indent=2) + "\n")
        print(f"\n  ✓ chain complete")
    return 0


def cmd_status(args):
    target = Path(args.output_dir) if args.output_dir else None
    if not target:
        # find latest planning/ dir
        brain_root = find_brain()
        plan_dir = brain_root / "planning"
        if not plan_dir.exists():
            print("  no planning/ dir; no chains run yet")
            return 0
        runs = sorted(plan_dir.iterdir())
        if not runs:
            print("  no planning runs found")
            return 0
        target = runs[-1]
    mf = target / "chain-manifest.json"
    if not mf.exists():
        print(f"  no chain-manifest.json in {target}", file=sys.stderr); return 2
    m = json.loads(mf.read_text())
    print(f"\n  chain @ {target}")
    print(f"  profile:  {m['profile']}    skip_prd: {m['skip_prd']}")
    print(f"  status:   {m['status']}")
    print(f"  created:  {m['created_at']}")
    print(f"\n  Plan:")
    for s in m["plan"]:
        marker = {"pending":"·","placeholder":"○","done":"✓","skipped":"—"}.get(s["status"], "?")
        print(f"    {marker} step {s['step']}  {s['skill_id']}  [{s['status']}]")
    return 0


def cmd_estimate(args):
    spec_text = args.pitch
    if args.spec_file:
        spec_text = Path(args.spec_file).read_text(encoding="utf-8")
    skip_prd, reasons = triage_skip_prd(spec_text) if args.profile == "solo" else (False, ["not solo"])
    est = PROFILE_TOKEN_ESTIMATES[args.profile]
    saved = 12000 if skip_prd else 0
    token_min, token_max = est["min"] - saved, est["max"] - saved
    cost_min = token_min / 1_000_000 * 3.0 + token_min / 4 / 1_000_000 * 15.0  # rough Sonnet $
    cost_max = token_max / 1_000_000 * 3.0 + token_max / 4 / 1_000_000 * 15.0
    print(f"\n  Estimate for profile {args.profile!r}:")
    print(f"  Tokens: {token_min:,} — {token_max:,}")
    print(f"  Cost (Claude Sonnet rates): ${cost_min:.3f} — ${cost_max:.3f} USD")
    print(f"  Skip-PRD triage: {skip_prd}  ({', '.join(reasons)})")
    if skip_prd:
        print(f"  (saving ~12K tokens by skipping prd-author + prd-audit)")
    return 0


def cmd_graph(args):
    target = Path(args.output_dir) if args.output_dir else None
    if not target:
        brain_root = find_brain()
        runs = sorted((brain_root / "planning").iterdir()) if (brain_root / "planning").exists() else []
        if not runs:
            print("  no planning runs found"); return 0
        target = runs[-1]
    mf = target / "chain-manifest.json"
    if not mf.exists():
        print(f"  no chain-manifest.json in {target}", file=sys.stderr); return 2
    m = json.loads(mf.read_text())
    print(f"\n  Chain graph — {target}")
    print(f"  profile: {m['profile']}\n")
    for i, s in enumerate(m["plan"]):
        connector = "  │\n  ▼\n" if i > 0 else ""
        marker = {"pending":"⏳","placeholder":"📝","done":"✅","skipped":"⏭️"}.get(s["status"], "?")
        print(f"{connector}  {marker}  {s['skill_id']}  [{s['status']}]")
    return 0


def main():
    p = argparse.ArgumentParser(description="operator umbrella for the requirements → tasks chain")
    sub = p.add_subparsers(dest="cmd", required=True)
    pr = sub.add_parser("run")
    pr.add_argument("--pitch", required=True)
    pr.add_argument("--spec-file")
    pr.add_argument("--profile", choices=["solo", "lean", "standard", "full"], default="solo")
    pr.add_argument("--output", default=None)
    pr.add_argument("--skip-prd", choices=["auto", "force", "never"], default="auto")
    pr.add_argument("--with-llm", action="store_true",
                    help="actually call Claude for each step (requires anthropic SDK + ANTHROPIC_API_KEY)")
    pr.add_argument("--model", default="claude-sonnet-4-6", help="model for --with-llm")
    pr.add_argument("--max-tokens", type=int, default=100000, help="chain-wide token budget for --with-llm")
    pr.add_argument("--max-cost", type=float, default=2.00, help="chain-wide cost budget USD for --with-llm")
    pr.add_argument("--step-max-tokens", type=int, default=4000, help="max-tokens per Claude call")
    pr.add_argument("--max-iterations", type=int, default=3,
                    help="multi-iteration self-audit budget per step (Tier α.3)")
    pr.add_argument("--no-cache", action="store_true",
                    help="bypass the skill cache (Tier α.9)")
    pr.add_argument("--dry-run", action="store_true")
    pr.set_defaults(func=cmd_run)
    prs = sub.add_parser("resume"); prs.add_argument("output_dir"); prs.set_defaults(func=cmd_resume)
    ps = sub.add_parser("status"); ps.add_argument("output_dir", nargs="?"); ps.set_defaults(func=cmd_status)
    pe = sub.add_parser("estimate"); pe.add_argument("--pitch", required=True); pe.add_argument("--spec-file"); pe.add_argument("--profile", default="solo"); pe.set_defaults(func=cmd_estimate)
    pg = sub.add_parser("graph"); pg.add_argument("output_dir", nargs="?"); pg.set_defaults(func=cmd_graph)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
