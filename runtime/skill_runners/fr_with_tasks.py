#!/usr/bin/env python3
"""
runtime/skill_runners/fr_with_tasks.py — concrete runner for cuo/cpo/fr-with-tasks.

Tier α.1 — reference implementation. Other 10 skills copy this template,
adjust `skill_id` + `output_filename_pattern` + interview / invariant lists,
and they're done.
"""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from base import BaseSkillRunner


class FrWithTasksRunner(BaseSkillRunner):
    skill_id = "cuo/cpo/fr-with-tasks"
    skill_version = "0.1.0"
    output_filename_pattern = "fr-with-tasks.md"

    # Standalone-interview questions (mirrors STANDALONE_INTERVIEW.md).
    interview_questions = [
        ("target_sprint", "Which sprint? (current / next / unsequenced)", "unsequenced"),
        ("ai_token_budget", "AI-agent token budget per FR?", 30000),
        ("risk_tier_ceiling", "Max EU AI Act tier? (minimal/limited/high_risk)", "limited"),
    ]

    def interview(self, inputs: dict) -> dict:
        """Non-interactive: take provided values, fall back to defaults."""
        out = dict(inputs)
        for key, _question, default in self.interview_questions:
            out.setdefault(key, default)
        return out

    def build_prompt(self, inputs: dict, prior_artefacts=None) -> str:
        """Compose the Claude prompt — pulls SKILL.md + task@1 contract template."""
        skill_md = (self.brain_root / "docs" / "skills" / self.skill_id / "SKILL.md").read_text(encoding="utf-8")
        task_template = ""
        task_contract = self.brain_root / "docs" / "contracts" / "task" / "template.md"
        if task_contract.exists():
            task_template = task_contract.read_text(encoding="utf-8")

        fix_hint_block = ""
        if inputs.get("_fix_hints"):
            fix_hint_block = f"""
# Prior attempt failed validation. Fix these:
{inputs['_fix_hints']}

# Prior body (DO NOT repeat the same errors):
{(inputs.get('_prior_body') or '')[:3000]}
"""

        return f"""You are running the cyberos skill `cuo/cpo/fr-with-tasks`.

# SKILL.md (excerpt)
{skill_md[:6000]}

# task@1 contract body template
{task_template[:1500]}

# Operator inputs
- pitch: {inputs.get('pitch', '?')}
- spec_file_contents (if any): {(inputs.get('spec_text') or '')[:3000]}
- target_sprint: {inputs.get('target_sprint')}
- ai_token_budget: {inputs.get('ai_token_budget')}
- risk_tier_ceiling: {inputs.get('risk_tier_ceiling')}

{fix_hint_block}

# Your job

Emit ONE `feature_request@1` markdown file. Body must include:

1. Frontmatter with: fr_id (FR-001), title, profile: solo, project, status: draft, eu_ai_act_risk_class, client_visible, acceptance_criteria, task_count, tasks (a list of task@1 objects).
2. After frontmatter, the FR body: Problem statement, Users, Success metrics, Scope, Risks, EU AI Act classification, Total estimated effort, Tasks (reference back to frontmatter list).

Each task in the `tasks:` frontmatter list MUST have ALL of:
- id (FR-001-T-MM), title, description (>= 200 chars), preconditions, deliverables,
  acceptance_test (with `shell:` or `assertion:` key, never both, NEVER "TBD"),
  sizing (S/M/L/XL), dependencies, parallelisable (bool), assignable_to (list),
  status: draft. Plus agent_profile + estimated_tokens when "ai-agent" in assignable_to.
  Plus estimated_hours when "human" in assignable_to.

Voice rules: no em dashes. No AI vocabulary (leverage, robust, ensure, comprehensive,
seamless, delve, navigate, tapestry, facilitate, utilize).

Source-attribution rules: paragraphs that paraphrase the operator's pitch carry
`authority: human-confirmed` inline as a marker; paragraphs the skill synthesised
carry `authority: llm-explicit`.

Output ONLY the artefact body. No commentary, no fences."""

    def validate_emit(self, body: str, inputs: dict) -> list[dict]:
        """Run fr-with-tasks-specific INVARIANTS on the emitted body."""
        findings = list(super().validate_emit(body, inputs))

        # INV-001 — frontmatter must include tasks: list
        if "tasks:" not in body:
            findings.append({"invariant": "INV-001",
                             "severity": "CRITICAL",
                             "fix_hint": "frontmatter must declare a `tasks:` list"})
            return findings

        # Parse tasks via yaml
        try:
            import yaml
            if not body.startswith("---"):
                findings.append({"invariant": "frontmatter-missing",
                                 "severity": "CRITICAL",
                                 "fix_hint": "body must start with YAML frontmatter"})
                return findings
            end = body.find("\n---\n", 4)
            if end < 0:
                findings.append({"invariant": "frontmatter-unclosed",
                                 "severity": "CRITICAL",
                                 "fix_hint": "frontmatter must close with `---`"})
                return findings
            fm = yaml.safe_load(body[4:end]) or {}
        except Exception as e:
            findings.append({"invariant": "frontmatter-yaml-invalid",
                             "severity": "CRITICAL",
                             "fix_hint": f"yaml parse failed: {e}"})
            return findings

        tasks = fm.get("tasks") or []
        if not isinstance(tasks, list):
            findings.append({"invariant": "INV-002",
                             "severity": "CRITICAL",
                             "fix_hint": "`tasks:` must be a list"})
            return findings

        # Per-task invariants
        task_ids = set()
        for i, t in enumerate(tasks):
            if not isinstance(t, dict):
                findings.append({"invariant": "INV-002",
                                 "severity": "CRITICAL",
                                 "fix_hint": f"task[{i}] must be a dict"})
                continue
            tid = t.get("id", "")
            if not re.match(r"^FR-\d+-T-\d{2}$", tid):
                findings.append({"invariant": "INV-003",
                                 "severity": "WARN",
                                 "fix_hint": f"task[{i}].id {tid!r} must match FR-NNN-T-MM"})
            if tid in task_ids:
                findings.append({"invariant": "INV-003-dup",
                                 "severity": "CRITICAL",
                                 "fix_hint": f"task id {tid} duplicated"})
            task_ids.add(tid)
            # INV-004 description >= 200 chars
            desc = t.get("description") or ""
            if len(desc) < 200:
                findings.append({"invariant": "INV-004",
                                 "severity": "WARN",
                                 "fix_hint": f"{tid}: description must be >= 200 chars (got {len(desc)})"})
            # INV-005 acceptance_test must be concrete
            at = t.get("acceptance_test") or {}
            if not isinstance(at, dict) or not (at.get("shell") or at.get("assertion")):
                findings.append({"invariant": "INV-005",
                                 "severity": "WARN",
                                 "fix_hint": f"{tid}: acceptance_test must have `shell:` or `assertion:`"})
            # INV-008 assignable_to non-empty
            at_list = t.get("assignable_to") or []
            if not at_list:
                findings.append({"invariant": "INV-008",
                                 "severity": "WARN",
                                 "fix_hint": f"{tid}: assignable_to must include human or ai-agent"})

        # INV-006 — dependency graph acyclic
        # Build edges then DFS for cycles
        edges = {t.get("id"): t.get("dependencies") or [] for t in tasks if isinstance(t, dict)}
        WHITE, GRAY, BLACK = 0, 1, 2
        color = {tid: WHITE for tid in edges}
        def dfs(tid):
            color[tid] = GRAY
            for dep in edges.get(tid, []):
                if dep not in color:
                    continue
                if color[dep] == GRAY:
                    findings.append({"invariant": "INV-006",
                                     "severity": "CRITICAL",
                                     "fix_hint": f"cycle: {tid} ↔ {dep}"})
                    return
                if color[dep] == WHITE:
                    dfs(dep)
            color[tid] = BLACK
        for tid in list(edges):
            if color[tid] == WHITE:
                dfs(tid)

        # INV-013 chain_profile: solo
        if fm.get("profile") != "solo":
            findings.append({"invariant": "INV-013",
                             "severity": "WARN",
                             "fix_hint": f"profile must be 'solo'; got {fm.get('profile')!r}"})

        return findings


if __name__ == "__main__":
    # Direct CLI: python3 runtime/skill_runners/fr_with_tasks.py <output_dir> <pitch-or-spec-file>
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("output_dir")
    p.add_argument("--pitch", required=True)
    p.add_argument("--spec-file")
    p.add_argument("--max-iterations", type=int, default=3)
    p.add_argument("--no-cache", action="store_true")
    args = p.parse_args()

    # Find brain root
    cur = Path.cwd().resolve()
    brain_root = None
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            brain_root = cur; break
        cur = cur.parent
    if not brain_root:
        sys.exit("no .cyberos-memory/ found")

    inputs = {"pitch": args.pitch}
    if args.spec_file:
        inputs["spec_text"] = Path(args.spec_file).read_text(encoding="utf-8")

    from base import SkillCache
    cache = None if args.no_cache else SkillCache()
    runner = FrWithTasksRunner(brain_root)
    result = runner.run(inputs, Path(args.output_dir), max_iterations=args.max_iterations, cache=cache)
    print(json.dumps({
        "status": result.status,
        "iterations": result.iterations,
        "artefact_path": str(result.artefact_path) if result.artefact_path else None,
        "tokens_used": result.tokens_used,
        "cost_usd": round(result.cost_usd, 6),
        "findings": result.findings[:5],
    }, indent=2))
    sys.exit(0 if result.status == "PASS" else 1)
