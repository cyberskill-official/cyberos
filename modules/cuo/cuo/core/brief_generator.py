"""BriefGenerator — produces execution briefs for host LLM consumption.

When CUO runs inside Claude Code, Cursor, Codex, or another host LLM
environment, the BriefGenerator produces a self-contained markdown runbook
that the host LLM follows using its own tools (Read/Write/Edit/Bash).

CUO does the planning (task resolution, chain validation, input resolution,
condition evaluation, SKILL.md reading). The host LLM does the execution
(code writing, file creation, tests, backlog updates).

The brief is the handoff between these two roles.
"""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from cuo.core.catalog import PersonaEntry, WorkflowEntry
from cuo.core.backlog_reader import parse_backlog
from cuo.core.supervisor import _eval_condition, _find_workflow, _resolve_step_inputs, _resolve_task


# Applier instructions for skills with side effects.
# These are human-readable instructions the host LLM must follow after
# generating the skill's output JSON.
_APPLIER_INSTRUCTIONS: dict[str, str] = {
    "backlog-state-update-author": """After writing the output JSON, you MUST update BACKLOG.md:
1. Find the row with `{task_id}` in `docs/tasks/BACKLOG.md`
2. Replace the Status column (column 5, the cell between the 4th and 5th `|`) with `{new_status}`
3. Write atomically: write to BACKLOG.md.tmp then rename to BACKLOG.md""",
    "coverage-gate-author": """After writing the output JSON, you MUST:
1. Run the project's test suite: `python3 -m pytest --tb=short -q` (or `cargo test --no-fail-fast`)
2. Capture the raw terminal output
3. Add `raw_terminal`, `return_code`, `tests_failed`, `duration_ms`, `cmd` fields to the output JSON""",
    "architecture-decision-record-author": """After writing the output JSON, you MUST also:
1. Write the ADR to `docs/adrs/ADR-{NNN}-{slug}.md` with YAML frontmatter
2. Include: title, adr_id, status, decision_date, decided_by""",
    "implementation-plan-author": """After writing the output JSON, you MUST also:
1. Write the implementation plan to `docs/tasks/{module}/impl-plan-{task_id}.md`
2. If the output contains `code_changes`, apply each entry:
   - `"action": "create"` → mkdir -p parent dir, write the file
   - `"action": "modify"` with `"content"` → overwrite the file
   - `"action": "modify"` with `"diff"` → apply the unified diff""",
    "code-review-author": """After writing the output JSON, you MUST also:
1. Write the code review to `docs/tasks/{module}/code-review-{task_id}.md`
2. Include YAML frontmatter with template, verdict, reviewed_at""",
    "task-audit": """After writing the output JSON, you MUST also:
1. Write the audit report to `{task_file_stem}.audit.md` (sibling to the task spec)
2. Include YAML frontmatter with task_id, audited, auditor, verdict, audited_file_sha256""",
    "observability-injection-author": """After writing the output JSON, you MUST also:
1. Write the observability plan to `docs/tasks/{module}/obs-injection-{task_id}.md`
2. Include YAML frontmatter with template, task_id, language, subscriber""",
}


def _strip_frontmatter(text: str) -> str:
    """Remove YAML frontmatter from a markdown file."""
    return re.sub(r"\A---\n.*?\n---\n", "", text, count=1, flags=re.DOTALL).strip()


_PROJECT_MARKERS = ("pyproject.toml", "package.json", "Cargo.toml", "go.mod")


def _has_project_markers(d: Path) -> bool:
    """Check if a directory contains any project marker file."""
    return any((d / m).is_file() for m in _PROJECT_MARKERS)


def _resolve_project_root(output_dir: Path, project_root: Path | None) -> Path:
    """Resolve the best project root: explicit > output_dir parent > CWD."""
    if project_root is not None:
        return project_root
    for candidate in [output_dir.parent, Path.cwd()]:
        if _has_project_markers(candidate):
            return candidate
    return Path.cwd()


def _detect_project_context(project_root: Path) -> dict:
    """Detect language, framework, and package manager from project root."""
    ctx: dict[str, Any] = {}

    if (project_root / "package.json").is_file():
        try:
            pkg = json.loads((project_root / "package.json").read_text())
            deps = {**pkg.get("dependencies", {}), **pkg.get("devDependencies", {})}
            ctx["language"] = "typescript" if "typescript" in deps else "javascript"
            ctx["package_manager"] = "npm"
            if (project_root / "pnpm-lock.yaml").is_file():
                ctx["package_manager"] = "pnpm"
            elif (project_root / "yarn.lock").is_file():
                ctx["package_manager"] = "yarn"
            if "next" in deps:
                ctx["framework"] = "nextjs"
            elif "react" in deps:
                ctx["framework"] = "react"
            elif "express" in deps:
                ctx["framework"] = "express"
            elif "@nestjs/core" in deps:
                ctx["framework"] = "nestjs"
        except (json.JSONDecodeError, OSError):
            pass

    if (project_root / "pyproject.toml").is_file():
        ctx["language"] = "python"
        if (project_root / "poetry.lock").is_file():
            ctx["package_manager"] = "poetry"
        elif (project_root / "requirements.txt").is_file():
            ctx["package_manager"] = "pip"

    if (project_root / "Cargo.toml").is_file():
        ctx["language"] = "rust"
        ctx["package_manager"] = "cargo"

    if (project_root / "go.mod").is_file():
        ctx["language"] = "go"
        ctx["package_manager"] = "go"

    if not ctx:
        ctx["language"] = "unknown"
    return ctx


def _read_skill_body(skill_name: str, skill_root: Path) -> str:
    """Read the SKILL.md body (after frontmatter) for a skill."""
    skill_md = skill_root / skill_name / "SKILL.md"
    if not skill_md.is_file():
        return f"[SKILL NOT FOUND: {skill_name}]"
    try:
        full = skill_md.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return f"[COULD NOT READ: {skill_name}]"
    return _strip_frontmatter(full)


def _read_skill_frontmatter(skill_name: str, skill_root: Path) -> dict:
    """Parse the YAML frontmatter of a skill's SKILL.md."""
    skill_md = skill_root / skill_name / "SKILL.md"
    if not skill_md.is_file():
        return {}
    try:
        full = skill_md.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return {}
    if not full.startswith("---"):
        return {}
    end = full.find("\n---", 3)
    if end == -1:
        return {}
    try:
        import yaml
        return yaml.safe_load(full[4:end]) or {}
    except Exception:
        return {}


def _format_inputs(inputs: dict) -> str:
    """Format resolved inputs for the brief."""
    parts = []
    for k, v in sorted(inputs.items()):
        if k.startswith("_"):
            continue
        if isinstance(v, (str, int, float, bool)):
            val_repr = str(v)
            if len(val_repr) > 500:
                val_repr = val_repr[:500] + "... [truncated]"
            parts.append(f"- **{k}**: `{val_repr}`")
        elif isinstance(v, dict):
            keys = list(v.keys())[:8]
            parts.append(f"- **{k}**: dict with keys: {keys}")
        elif isinstance(v, list):
            parts.append(f"- **{k}**: list with {len(v)} items")
        elif isinstance(v, Path):
            parts.append(f"- **{k}**: `{v}`")
        else:
            parts.append(f"- **{k}**: {type(v).__name__}")
    return "\n".join(parts) if parts else "(none)"


def _status_transition_for_step(skill_name: str) -> str | None:
    """Return the BACKLOG.md status transition for a step, if any."""
    transitions = {
        "backlog-state-update-author": {
            13: ("implementing", "ready_to_review"),
            15: ("ready_to_review", "reviewing"),
            19: ("reviewing", "ready_to_test"),
            21: ("ready_to_test", "testing"),
            28: ("testing", "done"),
            29: ("testing", "done"),
        },
    }
    return None  # step_num not available here; handled in generate()


class BriefGenerator:
    """Generates execution briefs for host LLM consumption."""

    def __init__(
        self,
        persona: PersonaEntry,
        workflow: WorkflowEntry,
        skill_root: Path,
        output_dir: Path,
        task_id: str,
        inputs: dict | None = None,
        project_root: Path | None = None,
        backlog_path: Path | None = None,
    ):
        self.persona = persona
        self.workflow = workflow
        self.skill_root = skill_root
        self.output_dir = output_dir
        self.task_id = task_id
        self.inputs = dict(inputs or {})
        self.project_root = _resolve_project_root(output_dir, project_root)
        self.backlog_path = backlog_path

    def generate(self) -> str:
        """Generate the complete execution brief as a markdown string."""
        parts: list[str] = []

        # 1. Build hand_off (same as execute_chain setup phase)
        hand_off = dict(self.inputs)
        hand_off["_cyberos_root"] = str(self.skill_root.parent.parent)
        hand_off.setdefault("task_id", self.task_id)
        _resolve_task(hand_off)

        # 2. Detect project context
        project_ctx = _detect_project_context(self.project_root)

        # 3. Determine start step from task status
        current_status = "ready_to_implement"
        start_step = 1
        cyberos_root = self.skill_root.parent.parent
        if self.backlog_path is None:
            backlog_path = cyberos_root / "docs" / "tasks" / "BACKLOG.md"
        else:
            backlog_path = self.backlog_path
        if backlog_path.is_file():
            try:
                rows = parse_backlog(backlog_path)
                task_row = next((r for r in rows if r.task_id == self.task_id), None)
                if task_row:
                    current_status = task_row.status
            except Exception:
                pass

        if current_status in ("ready_to_implement", "implementing"):
            start_step = 1
        elif current_status in ("ready_to_review", "reviewing"):
            start_step = 15
        elif current_status in ("ready_to_test", "testing"):
            start_step = 21

        # 4. Build the brief
        task_body = hand_off.get("next_task", "[task content not available]")
        task_file_path = hand_off.get("task_file_path", "unknown")

        # Header
        parts.append(f"# Execution Brief — {self.task_id}")
        parts.append("")
        parts.append(f"**Workflow:** `{self.workflow.workflow_id}`")
        parts.append(f"**Project:** `{self.project_root}`")
        parts.append(f"**task file:** `{task_file_path}`")
        parts.append(f"**task status:** `{current_status}` → start at step {start_step}")
        parts.append(f"**Output dir:** `{self.output_dir}`")
        parts.append("")

        # Project context
        parts.append("## 1. Project Context")
        parts.append("")
        for k, v in sorted(project_ctx.items()):
            parts.append(f"- **{k}:** {v}")
        parts.append("")

        # Task
        parts.append("## 2. Task")
        parts.append("")
        task_preview = task_body[:3000] if len(task_body) > 3000 else task_body
        parts.append(task_preview)
        if len(task_body) > 3000:
            parts.append(f"\n... [truncated, full content in `{task_file_path}`]")
        parts.append("")

        # Workflow overview
        n_steps = len(self.workflow.skill_chain)
        pattern = self.workflow.frontmatter.get("pattern", "linear")
        parts.append(f"## 3. Workflow: {n_steps} steps, pattern={pattern}")
        parts.append("")

        # Steps to skip
        conditional_steps = []
        for step_spec in self.workflow.skill_chain:
            if not isinstance(step_spec, dict):
                continue
            cond = step_spec.get("condition")
            if cond:
                step_num = step_spec.get("step", "?")
                skill = step_spec.get("skill", "")
                conditional_steps.append((step_num, skill, cond))

        if conditional_steps:
            parts.append("## 4. Conditional Steps")
            parts.append("")
            parts.append("Evaluate the condition after the prerequisite steps complete:")
            parts.append("")
            for step_num, skill, cond in conditional_steps:
                parts.append(f"- **Step {step_num}** (`{skill}`): condition = `{cond}`")
            parts.append("")

        # Execution plan
        parts.append("## 5. Execution Plan")
        parts.append("")
        parts.append(f"Follow each step in order. Start at step {start_step}.")
        parts.append("For each step: read the skill instructions, produce the output JSON,")
        parts.append("then perform any side effects noted.")
        parts.append("")

        # Process each step
        step_results_for_conditions: list[Any] = []
        for step_spec in self.workflow.skill_chain:
            if not isinstance(step_spec, dict):
                continue
            step_num = step_spec.get("step", len(step_results_for_conditions) + 1)
            skill_name = step_spec.get("skill", "")

            if not isinstance(skill_name, str) or not skill_name or skill_name.startswith("planned:"):
                continue

            # Phase skipping
            if step_num < start_step:
                parts.append(f"### Step {step_num}: {skill_name} — **SKIPPED** (task already `{current_status}`)")
                parts.append("")
                continue

            # Condition evaluation
            condition = step_spec.get("condition")
            condition_result = ""
            if condition:
                # Simulate condition evaluation with current hand_off
                try:
                    should_run = _eval_condition(condition, hand_off, step_results_for_conditions)
                    condition_result = " — will evaluate at runtime"
                except Exception:
                    condition_result = " — will evaluate at runtime"

            # Inputs
            inputs_from = step_spec.get("inputs_from")
            resolved_inputs = _resolve_step_inputs(inputs_from, hand_off)

            # Skill body
            skill_body = _read_skill_body(skill_name, self.skill_root)
            if len(skill_body) > 4000:
                skill_body_truncated = (
                    skill_body[:4000]
                    + f"\n\n... [truncated. Full SKILL.md at `{self.skill_root / skill_name / 'SKILL.md'}`]"
                )
            else:
                skill_body_truncated = skill_body

            # Output target
            outputs_to = step_spec.get("outputs_to", "")
            output_file = f"step{step_num:02d}_{skill_name}.json"

            # Skill frontmatter for structured output spec
            skill_fm = _read_skill_frontmatter(skill_name, self.skill_root)
            skill_outputs = skill_fm.get("outputs", [])
            audit_fields = skill_fm.get("audit", {}).get("required_fields", [])

            # Render step
            parts.append(f"### Step {step_num}: {skill_name}")
            parts.append("")

            if condition:
                parts.append(f"**Condition:** `{condition}`{condition_result}")
                parts.append("")

            parts.append("**Inputs:**")
            parts.append(_format_inputs(resolved_inputs))
            parts.append("")

            # Structured output expectations
            parts.append(f"**Write output to:** `{self.output_dir}/{output_file}`")
            if skill_outputs:
                for so in skill_outputs:
                    if isinstance(so, dict):
                        fmt = so.get("format", "json")
                        name = so.get("name", "output")
                        parts.append(f"**Output format:** `{name}` = `{fmt}`")
            if audit_fields:
                parts.append(f"**Audit fields:** `{{ {', '.join(audit_fields)} }}`")
            if outputs_to:
                parts.append(f"**Hand-off key:** `{outputs_to}`")
            parts.append("")

            # Applier instructions
            applier = _APPLIER_INSTRUCTIONS.get(skill_name)
            if applier:
                parts.append("**Side Effects:**")
                # Inject task_id context
                applier_text = applier.replace("{task_id}", self.task_id)
                parts.append(applier_text)
                parts.append("")

            parts.append("<details>")
            parts.append(f"<summary>Skill Instructions ({len(skill_body)} chars)</summary>")
            parts.append("")
            parts.append(skill_body_truncated)
            parts.append("")
            parts.append("</details>")
            parts.append("")

            # Update hand_off for next step's condition evaluation
            # Use a mock StepResult-like object
            mock_result = _MockStepResult(step_num, skill_name, "OK" if not condition else "SKIPPED")
            step_results_for_conditions.append(mock_result)

            if outputs_to and isinstance(outputs_to, str):
                hand_off[outputs_to] = f"[output from step {step_num}]"
            hand_off[f"step_{step_num}_ran"] = True

        # Completion
        parts.append("## 6. Completion")
        parts.append("")
        parts.append("The task must reach status `done` in BACKLOG.md.")
        parts.append("If any step fails, update BACKLOG.md status back to `ready_to_implement`")
        parts.append("and note the failure reason.")
        parts.append("")
        parts.append("Output files go in: `" + str(self.output_dir) + "/`")
        parts.append("Deliverable files (.audit.md, ADR, impl-plan, code-review, etc.)")
        parts.append("go alongside their respective task spec files in the project.")
        parts.append("")

        # Output format reference
        parts.append("## 7. Output Format Reference")
        parts.append("")
        parts.append("Each step must produce a JSON file. CUO reads these files to resume the chain.")
        parts.append("")
        for step_spec in self.workflow.skill_chain:
            if not isinstance(step_spec, dict):
                continue
            sn = step_spec.get("step", 0)
            sk = step_spec.get("skill", "")
            if not isinstance(sk, str) or not sk or sk.startswith("planned:"):
                continue
            if sn < start_step:
                continue
            sfm = _read_skill_frontmatter(sk, self.skill_root)
            s_outputs = sfm.get("outputs", [])
            s_af = sfm.get("audit", {}).get("required_fields", [])
            s_out_key = step_spec.get("outputs_to", "")
            fmt_parts = []
            for so in s_outputs:
                if isinstance(so, dict):
                    fmt_parts.append(f"`{so.get('format', 'json')}`")
            fmt_str = ", ".join(fmt_parts) if fmt_parts else "json"
            parts.append(f"| {sn} | `{sk}` | {fmt_str} | {', '.join(f'`{f}`' for f in s_af[:4]) if s_af else '-'} | `{s_out_key}` |")
        parts.append("")

        return "\n".join(parts)


class _MockStepResult:
    """Minimal stand-in for StepResult used in condition evaluation."""

    def __init__(self, step: int, skill: str, status: str):
        self.step = step
        self.skill = skill
        self.status = status
