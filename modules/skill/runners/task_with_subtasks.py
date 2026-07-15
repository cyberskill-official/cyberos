#!/usr/bin/env python3
"""
runtime/skill_runners/task_with_subtasks.py — concrete runner for cuo/cpo/task-with-subtasks.

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


class TaskWithSubtasksRunner(BaseSkillRunner):
    skill_id = "cuo/cpo/task-with-subtasks"
    skill_version = "0.1.0"
    output_filename_pattern = "task-with-subtasks.md"

    # Standalone-interview questions (mirrors STANDALONE_INTERVIEW.md).
    interview_questions = [
        ("target_sprint", "Which sprint? (current / next / unsequenced)", "unsequenced"),
        ("ai_token_budget", "AI-agent token budget per task?", 30000),
        ("risk_tier_ceiling", "Max EU AI Act tier? (minimal/limited/high_risk)", "limited"),
    ]

    def interview(self, inputs: dict) -> dict:
        """Non-interactive: take provided values, fall back to defaults."""
        out = dict(inputs)
        for key, _question, default in self.interview_questions:
            out.setdefault(key, default)
        return out

    def build_prompt(self, inputs: dict, prior_artefacts=None) -> str:
        """Compose the Claude prompt — pulls SKILL.md + subtask@1 contract template."""
        skill_md = (self.memory_root / "docs" / "skills" / self.skill_id / "SKILL.md").read_text(encoding="utf-8")
        task_template = ""
        task_contract = self.memory_root / "docs" / "contracts" / "task" / "template.md"
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

        return f"""You are running the cyberos skill `cuo/cpo/task-with-subtasks`.

# SKILL.md (excerpt)
{skill_md[:6000]}

# subtask@1 contract body template
{task_template[:1500]}

# Operator inputs
- pitch: {inputs.get('pitch', '?')}
- spec_file_contents (if any): {(inputs.get('spec_text') or '')[:3000]}
- target_sprint: {inputs.get('target_sprint')}
- ai_token_budget: {inputs.get('ai_token_budget')}
- risk_tier_ceiling: {inputs.get('risk_tier_ceiling')}

{fix_hint_block}

# Your job

Emit ONE `task@1` markdown file using the NEW shape (Batch A, 2026-05-12).

## Frontmatter (slim — registry fields ONLY)

```yaml
---
task_id: TASK-001
title: <one-sentence title>
profile: solo
project: <project-slug>
status: draft
eu_ai_act_risk_class: not_ai | minimal | limited | high
client_visible: true | false
authority: human-confirmed
acceptance_criteria:
  - "<measurable criterion 1>"
  - "<measurable criterion 2>"
task_index:
  - {id: TASK-001-S-01, title: <task title>}
  - {id: TASK-001-S-02, title: <task title>}
---
```

Do NOT put a `tasks:` list inside the frontmatter. Do NOT put `provenance`,
`source_ref`, `confidence`, or any "Source attribution" section anywhere.
Frontmatter is for tools; tasks live as body H2 sections below.

## Body

The body has these H2 sections in order:
1. `# TASK-NNN — <Title>` (one H1 right after the closing `---`)
2. `## Problem statement` — 1-3 paragraphs of prose
3. `## Users` — primary / secondary / tertiary as a paragraph or short list
4. `## Success metrics` — bullet list of measurable targets
5. `## Scope` — with `### In` and `### Out` subsections
6. `## Risks` — bullet list (R1 / R2 / R3 prefix optional)
7. `## EU AI Act classification` — single short paragraph naming the risk class
8. `## Total estimated effort` — short paragraph: N hours human + M tokens AI

Then ONE H2 per task: `## TASK-NNN-T-MM — <Task title>`. Inside each task section:

- Description prose (>= 200 chars, multiple paragraphs OK).
- `**Preconditions:**` followed by `- bullet` list (or `- none`).
- `**Deliverables:**` followed by `- bullet` list of concrete outputs.
- `**Acceptance test:**` followed by a fenced code block with info-string `shell` or `assertion`. NEVER "TBD".
- A fenced `task-meta` block (info-string literally `task-meta`) carrying YAML:
  ```task-meta
  sizing: S | M | L | XL
  dependencies: [TASK-NNN-T-MM, ...]
  parallelisable: true | false
  assignable_to: [human] | [ai-agent] | [human, ai-agent]
  agent_profile: <profile-id>    # required when ai-agent in assignable_to
  estimated_tokens: <int>        # required when ai-agent in assignable_to
  estimated_hours: <float>       # required when human in assignable_to
  status: draft
  runbook_hint: <skill-name> | null
  ```

Voice rules: no em dashes (use parentheses or commas). No AI vocabulary
(leverage, robust, ensure, comprehensive, seamless, delve, navigate, tapestry,
facilitate, utilize).

No Source-attribution prose. No Provenance section. Frontmatter `authority`
field is enough.

Output ONLY the artefact body starting with `---`. No commentary, no surrounding fences."""

    def validate_emit(self, body: str, inputs: dict) -> list[dict]:
        """Run task-with-subtasks-specific INVARIANTS on the emitted body.

        Reads tasks via cyberos_fr_parser (Batch A): prefers body-H2 task
        sections; falls back to legacy `tasks:` frontmatter list.
        """
        findings = list(super().validate_emit(body, inputs))

        # Parse frontmatter
        try:
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
            import yaml
            fm = yaml.safe_load(body[4:end]) or {}
            body_after_fm = body[end + 5:]
        except Exception as e:
            findings.append({"invariant": "frontmatter-yaml-invalid",
                             "severity": "CRITICAL",
                             "fix_hint": f"yaml parse failed: {e}"})
            return findings

        # Try body-H2 parser first (new shape), fall back to legacy frontmatter
        import sys as _sys
        from pathlib import Path as _Path
        _sys.path.insert(0, str(_Path(__file__).resolve().parents[1] / "tools"))
        from cyberos_fr_parser import parse_body_tasks
        tasks = parse_body_tasks(body_after_fm)
        if not tasks:
            tasks = fm.get("tasks") or []
            if tasks:
                findings.append({"invariant": "shape-legacy",
                                 "severity": "WARN",
                                 "fix_hint": "task uses legacy frontmatter-tasks shape; "
                                             "regenerate with body H2 sections per Batch A"})
        if not tasks:
            findings.append({"invariant": "INV-001",
                             "severity": "CRITICAL",
                             "fix_hint": "task must declare tasks as body `## TASK-NNN-T-MM —` "
                                         "H2 sections OR a frontmatter `tasks:` list"})
            return findings
        if not isinstance(tasks, list):
            findings.append({"invariant": "INV-002",
                             "severity": "CRITICAL",
                             "fix_hint": "tasks must be a list"})
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
            if not re.match(r"^TASK-\d+-S-\d{2}$", tid):
                findings.append({"invariant": "INV-003",
                                 "severity": "WARN",
                                 "fix_hint": f"task[{i}].id {tid!r} must match TASK-NNN-T-MM"})
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
    # Direct CLI: python3 runtime/skill_runners/task_with_subtasks.py <output_dir> <pitch-or-spec-file>
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("output_dir")
    p.add_argument("--pitch", required=True)
    p.add_argument("--spec-file")
    p.add_argument("--max-iterations", type=int, default=3)
    p.add_argument("--no-cache", action="store_true")
    args = p.parse_args()

    # Find memory root
    cur = Path.cwd().resolve()
    memory_root = None
    while cur != cur.parent:
        if (cur / ".cyberos/memory/store").is_dir():
            memory_root = cur; break
        cur = cur.parent
    if not memory_root:
        sys.exit("no .cyberos/memory/store/ found")

    inputs = {"pitch": args.pitch}
    if args.spec_file:
        inputs["spec_text"] = Path(args.spec_file).read_text(encoding="utf-8")

    from base import SkillCache
    cache = None if args.no_cache else SkillCache()
    runner = TaskWithSubtasksRunner(memory_root)
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
