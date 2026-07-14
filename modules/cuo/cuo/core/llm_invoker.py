"""LLMInvoker — drives prompt-only SDP skills via LLM API.

Most CyberOS skills (statement-of-work-author, software-requirements-specification-author, architecture-decision-record-author, etc.) are prompt-only:
their SKILL.md body IS the system prompt, and the audit-pair `<skill>-audit`'s
RUBRIC.md is the validation prompt. They cannot be subprocess-invoked because
there's no executable to run.

LLMInvoker reads the skill's SKILL.md body (after YAML frontmatter) and uses it
as the system prompt. The user prompt is the JSON-encoded inputs the supervisor
hands off from the prior step. The LLM response is parsed as the structured
output (best-effort JSON extraction; falls back to wrapping raw text).

Two modes:

  1. **Mock-LLM mode** (default, no API key required) — simulates an LLM
     response by parroting the contract template's H2 headings as keys and
     "[mock-llm output]" as the value. Output keys reflect the actual artefact
     structure the skill produces.

  2. **Anthropic API mode** (when `ANTHROPIC_API_KEY` env var is set OR an
     api_key is passed to __init__) — uses the Anthropic Messages API
     (claude-sonnet-4-6 by default) to actually compute. Requires the `anthropic`
     Python SDK; emits a clear error if missing.

Future modes (Phase 3.1+):
  - OpenAI / Azure OpenAI
  - Local-host (LM Studio / Ollama)
  - Multi-model cascade (LiteLLM)

This is Phase 3 of the v3.0.0 supervisor build. Phase 2's `SubprocessInvoker`
covers Rust-binary subprocess invocation; this adds the LLM-driven path that
prompt-only SDP skills need.
"""

from __future__ import annotations

import json
import os
import re
import time
from pathlib import Path

from cuo.core.invoker import Invoker, StepResult, _strip_role_suffix, _stringify_inputs


# Default Claude model — Sonnet 4.6 for the right cost/quality balance.
_DEFAULT_MODEL = "claude-sonnet-4-6"

# Extract YAML frontmatter to skip it when reading the skill body.
_FRONTMATTER_RE = re.compile(r"\A---\n.*?\n---\n", re.DOTALL)


class LLMInvoker(Invoker):
    """Invoker that drives prompt-only skills via an LLM.

    Default mode is "mock-llm" — no network, deterministic, simulates an LLM
    response shaped like the skill's contract template. Useful for testing the
    LLM-driven path without API costs.

    When `api_key` is supplied (or `ANTHROPIC_API_KEY` env var is set) AND the
    `anthropic` SDK is installed, switches to real API calls.
    """

    def __init__(
        self,
        *,
        model: str | None = None,
        api_key: str | None = None,
        base_url: str | None = None,
        max_tokens: int = 16000,
        mock_only: bool = False,
    ):
        self.api_key = (
            api_key
            or os.environ.get("ANTHROPIC_API_KEY")
            or os.environ.get("ANTHROPIC_AUTH_TOKEN")
        )
        self.base_url = base_url or os.environ.get("ANTHROPIC_BASE_URL")
        self.model = model or os.environ.get("ANTHROPIC_MODEL") or _DEFAULT_MODEL
        self.max_tokens = max_tokens
        self.mock_only = mock_only
        self._client = None  # lazy init

    @property
    def mode(self) -> str:
        """'real' if Anthropic API will be used, 'mock-llm' otherwise."""
        if self.mock_only:
            return "mock-llm"
        if not self.api_key:
            return "unconfigured"
        return "real"

    def invoke(
        self,
        skill_name: str,
        inputs: dict,
        skill_root: Path,
        output_dir: Path,
        step_num: int,
        *,
        file_prefix: str = "",
    ) -> StepResult:
        t0 = time.monotonic_ns()
        result = StepResult(step=step_num, skill=skill_name, status="FAILED")

        skill_dir = skill_root / skill_name
        skill_md = skill_dir / "SKILL.md"
        if not skill_md.is_file():
            result.notes.append(f"skill not found at {skill_md}")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        # Read skill body (after frontmatter) as the system prompt.
        try:
            full = skill_md.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError) as e:
            result.notes.append(f"could not read SKILL.md: {e}")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result
        system_prompt = _FRONTMATTER_RE.sub("", full).strip()

        # For audit skills, also include the RUBRIC.md as a guardrail.
        if skill_name.endswith("-audit"):
            rubric_path = skill_dir / "RUBRIC.md"
            if rubric_path.is_file():
                try:
                    rubric_txt = rubric_path.read_text(encoding="utf-8")
                    system_prompt += "\n\n---\n\n# RUBRIC (validation rules)\n\n" + rubric_txt
                except (OSError, UnicodeDecodeError):
                    pass

        # Build the user prompt from the inputs map.
        user_prompt = self._build_user_prompt(skill_name, inputs)

        # Dispatch to real-API or mock-LLM.
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"{file_prefix}step{step_num:02d}_{skill_name}.json"

        if self.mode == "real":
            output, status, notes = self._call_anthropic(system_prompt, user_prompt)
        elif self.mode == "mock-llm":
            output, status, notes = self._mock_llm(skill_name, skill_root, inputs)
        else:
            # unconfigured — cannot invoke
            result.notes.append(
                "LLMInvoker: no ANTHROPIC_API_KEY set. "
                "Export ANTHROPIC_API_KEY or install cyberos-skill binary."
            )
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        try:
            output_path.write_text(json.dumps(output, indent=2, sort_keys=True), encoding="utf-8")
        except OSError as e:
            result.notes.append(f"could not write output: {e}")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        result.status = status
        result.output = output
        result.output_path = output_path
        result.notes.extend(notes)
        result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
        return result

    # ---------- helpers ----------

    def _build_user_prompt(self, skill_name: str, inputs: dict) -> str:
        """Build the user prompt — JSON-encoded inputs + task framing."""
        stringified = _stringify_inputs(inputs)
        base = (
            f"# Task\n\n"
            f"You are invoking the `{skill_name}` skill within a CUO workflow chain.\n"
            f"Your inputs (resolved from the prior chain step's outputs) are below.\n\n"
            f"# Inputs\n\n```json\n{json.dumps(stringified, indent=2, sort_keys=True)}\n```\n\n"
            f"# Output format — CRITICAL\n\n"
            f"Your ENTIRE response must be a SINGLE valid JSON object. No markdown, no prose, no preamble.\n"
            f"Do NOT use CONTRACT_ECHO format. Do NOT wrap in ``` fences. Output raw JSON only.\n"
            f"For author skills, the keys should be the H2 headings of the artefact template.\n"
            f"For audit skills, return: `{{rule_outcomes: {{...}}, score: 0-10, pass: bool, fixes: [...]}}`.\n"
        )

        # Skill-specific prompts for file-materializing skills.
        if "implementation-plan" in skill_name:
            base += (
                f"\n# Additional output\n\n"
                f"Include a `code_changes` array with file-level changes to implement the plan.\n"
                f"Each entry: `{{\"action\": \"create\"|\"modify\", \"path\": \"<relative-path>\", "
                f"\"content\": \"<full-file-content>\"}}`.\n"
                f"Only include files that need to be created or changed. Use `\"action\": \"modify\"` "
                f"with `\"content\"` for full replacement or `\"diff\"` for unified-diff patches.\n"
            )
        elif "task-audit" in skill_name:
            base += (
                f"\n# Additional output\n\n"
                f"Include an `audit_body` string field containing the full markdown audit report "
                f"(frontmatter will be added by the applier). Include `task_id` and `verdict` fields.\n"
            )
        elif "code-review" in skill_name:
            base += (
                f"\n# Additional output\n\n"
                f"Include a `code_review_body` string field with the full markdown review. "
                f"Include `task_id` and `verdict` fields.\n"
            )
        elif "architecture-decision-record" in skill_name:
            base += (
                f"\n# Additional output\n\n"
                f"Include a `body` string field with the full ADR markdown (frontmatter will be added). "
                f"Include `adr_id`, `title`, `status`, `context`, `decision`, "
                f"`options` (array of `{{name, pros, cons}}`), and `consequences` fields.\n"
            )
        elif "observability-injection" in skill_name:
            base += (
                f"\n# Additional output\n\n"
                f"Include `task_id`, `language`, `subscriber`, `log_points` (array), "
                f"`trace_spans` (array), `error_counters` (array), and `branch_coverage` (object).\n"
            )

        return base

    def _mock_llm(self, skill_name: str, skill_root: Path, inputs: dict) -> tuple[dict, str, list[str]]:
        """Simulate an LLM response by parroting the contract template structure."""
        base_name = _strip_role_suffix(skill_name)
        template_path = skill_root / "contracts" / base_name / "template.md"

        output: dict = {
            "skill": skill_name,
            "step_invocation": "mock-llm",
            "synthetic": True,
            "inputs_received": _stringify_inputs(inputs),
            "model": self.model,
            "ts_ns": time.time_ns(),
        }

        if template_path.is_file():
            try:
                txt = template_path.read_text(encoding="utf-8")
                fields = [
                    line[3:].strip()
                    for line in txt.splitlines()
                    if line.startswith("## ") and not line.startswith("## §")
                ]
                output["artefact_fields"] = {
                    field: "[mock-llm placeholder — real LLM would compute]"
                    for field in fields
                }
            except (OSError, UnicodeDecodeError):
                pass

        # For audit skills, also produce a synthetic rubric outcome.
        if skill_name.endswith("-audit"):
            output["rubric_outcome"] = {
                "score": 10,
                "pass": True,
                "fixes": [],
                "note": "[mock-llm — no real validation performed]",
            }

        notes = [
            f"mock-llm mode (model={self.model} — no API call); "
            f"set ANTHROPIC_API_KEY + install anthropic SDK for real LLM"
        ]
        return output, "MOCKED", notes

    def _call_anthropic(self, system_prompt: str, user_prompt: str) -> tuple[dict, str, list[str]]:
        """Call Anthropic Messages API. Returns (output, status, notes)."""
        anthropic_mod = self._try_import_anthropic()
        if anthropic_mod is None:
            return (
                {"error": "anthropic SDK not installed"},
                "FAILED",
                ["install with `pip install anthropic`"],
            )
        if not self.api_key:
            return (
                {"error": "no API key"},
                "FAILED",
                ["set ANTHROPIC_API_KEY env var or pass api_key= to LLMInvoker()"],
            )

        if self._client is None:
            kwargs = {"api_key": self.api_key}
            if self.base_url:
                kwargs["base_url"] = self.base_url
            self._client = anthropic_mod.Anthropic(**kwargs)

        try:
            msg = self._client.messages.create(
                model=self.model,
                max_tokens=self.max_tokens,
                system=system_prompt,
                messages=[{"role": "user", "content": user_prompt}],
            )
        except Exception as e:  # noqa: BLE001
            return (
                {"error": f"Anthropic API call failed: {e}"},
                "FAILED",
                [f"API error: {e}"],
            )

        # Extract response text. Anthropic SDK returns content as a list of blocks.
        response_text = ""
        for block in msg.content:
            if hasattr(block, "text"):
                response_text += block.text

        # Best-effort JSON parse — LLM may wrap in markdown or include preamble.
        output = self._extract_json(response_text)
        if output is None:
            output = {"raw_response": response_text, "warning": "could not parse as JSON"}
            return output, "OK", [
                f"Anthropic API succeeded but response was not parseable JSON",
            ]

        # Attach metadata.
        if isinstance(output, dict):
            output.setdefault("_llm_meta", {})["model"] = self.model
            output["_llm_meta"]["usage"] = {
                "input_tokens": getattr(msg.usage, "input_tokens", None),
                "output_tokens": getattr(msg.usage, "output_tokens", None),
            }
        return output, "OK", [f"Anthropic API call succeeded (model={self.model})"]

    @staticmethod
    def _try_import_anthropic():
        """Best-effort import of anthropic SDK. Returns module or None."""
        try:
            import anthropic
            return anthropic
        except ImportError:
            return None

    @staticmethod
    def _extract_json(text: str):
        """Extract first JSON object from text (handles ```json fences or preamble)."""
        text = text.strip()
        # Try direct parse first.
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            pass
        # Look for ```json ... ``` fence (explicit json, not ```text or bare ```).
        for fence_match in re.finditer(r"```json\n(.*?)\n```", text, re.DOTALL):
            try:
                return json.loads(fence_match.group(1))
            except json.JSONDecodeError:
                pass
        # Also try ```json without closing fence (truncated output).
        json_start = text.find("```json\n")
        if json_start >= 0:
            content = text[json_start + len("```json\n"):]
            # Strip trailing ``` if present
            if content.rstrip().endswith("```"):
                content = content.rstrip()[:-3].rstrip()
            result = LLMInvoker._try_parse_json_or_repair(content)
            if result is not None:
                return result
        # Look for {...} using brace-depth matching to find the outermost object.
        first_brace = text.find("{")
        if first_brace >= 0:
            result = LLMInvoker._extract_json_from_brace(text, first_brace)
            if result is not None:
                return result
        return None

    @staticmethod
    def _extract_json_from_brace(text: str, start: int):
        """Extract a JSON object starting at text[start] using brace-depth matching."""
        depth = 0
        in_string = False
        escape_next = False
        for i in range(start, len(text)):
            c = text[i]
            if escape_next:
                escape_next = False
                continue
            if c == "\\" and in_string:
                escape_next = True
                continue
            if c == '"' and not escape_next:
                in_string = not in_string
                continue
            if in_string:
                continue
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    candidate = text[start:i + 1]
                    try:
                        return json.loads(candidate)
                    except json.JSONDecodeError:
                        # Object didn't parse — try next {
                        next_brace = text.find("{", start + 1)
                        if next_brace >= 0 and next_brace <= i:
                            return LLMInvoker._extract_json_from_brace(text, next_brace)
                        return None
        # Reached end of text with unclosed braces — try to repair truncated JSON.
        return LLMInvoker._try_parse_json_or_repair(text[start:])

    @staticmethod
    def _try_parse_json_or_repair(text: str):
        """Try to parse JSON; if it fails, attempt to repair truncated JSON."""
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            pass
        # Attempt repair: close open strings, then close containers in nesting order.
        repaired = text.rstrip()
        # Track open containers using a stack (for correct nesting order).
        container_stack: list[str] = []
        in_string = False
        escape_next = False
        string_was_open = False
        for c in repaired:
            if escape_next:
                escape_next = False
                continue
            if c == "\\" and in_string:
                escape_next = True
                continue
            if c == '"':
                in_string = not in_string
                continue
            if in_string:
                continue
            if c == "{":
                container_stack.append("}")
            elif c == "[":
                container_stack.append("]")
            elif c in ("}", "]"):
                if container_stack and container_stack[-1] == c:
                    container_stack.pop()
        # Close open string
        if in_string:
            repaired += '"'
            string_was_open = True
        # Remove trailing comma if any
        repaired = repaired.rstrip()
        if repaired.endswith(","):
            repaired = repaired[:-1].rstrip()
        # Close open containers in reverse nesting order (innermost first)
        for closer in reversed(container_stack):
            repaired += closer
        try:
            return json.loads(repaired)
        except json.JSONDecodeError:
            return None
