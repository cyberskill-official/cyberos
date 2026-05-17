"""Invoker — runs a single skill given inputs and returns its output.

Phase 2 defines a pluggable `Invoker` interface with two implementations:

- `MockInvoker` (always available) — returns deterministic placeholder output
  shaped per the skill's contract template. Useful for end-to-end workflow
  testing without skill execution. This is the default.

- `SubprocessInvoker` (requires `cyberos-skill` binary on PATH) — invokes the
  Rust CLI via subprocess, passing JSON inputs on stdin and capturing JSON
  outputs from stdout. Works for skills that ship an executable (VN bundles
  with `scripts/<name>.py`); SDP-driven prompt-only skills will likely return
  an empty or instructional body since they need LLM-side execution.

Phase 3 will add an `LLMInvoker` that reads the skill's body (SKILL.md after
frontmatter) as a system prompt, prompts an LLM with the inputs, and parses
the response as the structured output. That's the path that makes prompt-only
SDP skills actually compute.

Per cuo/docs/SPEC.md, each invocation returns a `StepResult` with status +
output + audit fields. Workflow chain hand-off happens via the workflow
output directory: each step's output is written to a JSON file the next
step's `inputs_from` can reference.
"""

from __future__ import annotations

import abc
import hashlib
import json
import shutil
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class StepResult:
    """Outcome of a single skill invocation within a workflow chain."""

    step: int
    skill: str
    status: str  # "OK" | "MOCKED" | "FAILED" | "SKIPPED"
    output: dict = field(default_factory=dict)
    output_path: Path | None = None
    duration_ms: int = 0
    stderr: str = ""
    notes: list[str] = field(default_factory=list)

    @property
    def output_hash(self) -> str:
        """sha256 of the JSON-canonical output — used for audit-chain rows."""
        canonical = json.dumps(self.output, sort_keys=True, separators=(",", ":")).encode("utf-8")
        return hashlib.sha256(canonical).hexdigest()

    def __repr__(self) -> str:
        return f"StepResult(step={self.step}, skill={self.skill!r}, status={self.status!r}, dur={self.duration_ms}ms)"


class Invoker(abc.ABC):
    """Abstract base — implementations choose how to actually invoke a skill."""

    @abc.abstractmethod
    def invoke(
        self,
        skill_name: str,
        inputs: dict,
        skill_root: Path,
        output_dir: Path,
        step_num: int,
    ) -> StepResult:
        """Invoke `skill_name` with `inputs` and write its output to `output_dir`.

        Args:
            skill_name: e.g. "srs-author" — must correspond to a directory at
                `skill_root/<skill_name>/SKILL.md`.
            inputs: dict of input-name → value (file path string OR primitive).
            skill_root: filesystem path to `skill/` (must contain MODULE.md).
            output_dir: directory where step output JSON is written.
            step_num: 1-based step number for filename suffix.

        Returns:
            StepResult with status + output + output_path + duration.
        """
        raise NotImplementedError


class MockInvoker(Invoker):
    """Deterministic placeholder invoker — does not actually run the skill.

    Returns a synthetic output dict shaped from the skill's contract template
    (if `skill/contracts/<base>/template.md` exists). Useful for:

    - Phase 2 end-to-end workflow walking without real skill execution
    - Test scaffolding (the smoke suite uses this)
    - Operator dry-runs that need realistic-looking output for downstream
      step planning

    Output shape: {"skill": ..., "step": ..., "synthetic": True, "inputs": ...,
                   "fields_from_template": [...], "ts_ns": ...}
    """

    def invoke(
        self,
        skill_name: str,
        inputs: dict,
        skill_root: Path,
        output_dir: Path,
        step_num: int,
    ) -> StepResult:
        t0 = time.monotonic_ns()
        result = StepResult(step=step_num, skill=skill_name, status="MOCKED")

        # Confirm the skill exists on disk before mocking.
        skill_dir = skill_root / skill_name
        if not (skill_dir / "SKILL.md").is_file():
            result.status = "FAILED"
            result.notes.append(f"skill not found at {skill_dir}/SKILL.md")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        # If a contract template exists, extract its H2 headings as the output
        # "field" set the MOCK will pretend to have produced.
        base_name = _strip_role_suffix(skill_name)
        template_path = skill_root / "contracts" / base_name / "template.md"
        fields_from_template: list[str] = []
        if template_path.is_file():
            try:
                txt = template_path.read_text(encoding="utf-8")
                fields_from_template = [
                    line[3:].strip()
                    for line in txt.splitlines()
                    if line.startswith("## ") and not line.startswith("## §")
                ]
            except (OSError, UnicodeDecodeError):
                pass

        # Build a deterministic synthetic output.
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"step{step_num:02d}_{skill_name}.json"
        output = {
            "skill": skill_name,
            "step": step_num,
            "synthetic": True,
            "inputs": _stringify_inputs(inputs),
            "fields_from_template": fields_from_template,
            "ts_ns": time.time_ns(),
        }
        try:
            output_path.write_text(json.dumps(output, indent=2, sort_keys=True), encoding="utf-8")
        except OSError as e:
            result.status = "FAILED"
            result.notes.append(f"could not write output: {e}")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        result.output = output
        result.output_path = output_path
        result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
        result.notes.append(
            f"MOCKED (no skill execution); contract template fields: {len(fields_from_template)}"
        )
        return result


class SubprocessInvoker(Invoker):
    """Subprocess invoker — calls `cyberos-skill run <name>` via the OS PATH.

    Passes a JSON inputs object on stdin and parses stdout as JSON output.
    Requires the Rust binary to be built and on PATH. On systems without
    the binary, callers should fall back to `MockInvoker`.

    The Rust CLI uses `--executor auto` by default (WASM if compiled, otherwise
    script). SDP-driven prompt-only skills will likely produce minimal output;
    they're meant to be driven by an LLM, which Phase 3 will add.
    """

    def __init__(self, binary: str | None = None, timeout_s: int = 600):
        self.binary = binary or "cyberos-skill"
        self.timeout_s = timeout_s

    @classmethod
    def is_available(cls, binary: str | None = None) -> bool:
        """Check whether the `cyberos-skill` binary is callable on this system."""
        return shutil.which(binary or "cyberos-skill") is not None

    def invoke(
        self,
        skill_name: str,
        inputs: dict,
        skill_root: Path,
        output_dir: Path,
        step_num: int,
    ) -> StepResult:
        t0 = time.monotonic_ns()
        result = StepResult(step=step_num, skill=skill_name, status="FAILED")

        if not self.is_available(self.binary):
            result.notes.append(
                f"binary {self.binary!r} not on PATH — "
                "build skill/ via `cargo build --release` or use MockInvoker"
            )
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        stdin_json = json.dumps({"skill": skill_name, "step": step_num, "inputs": inputs})

        try:
            proc = subprocess.run(
                [self.binary, "--root", str(skill_root), "run", skill_name],
                input=stdin_json,
                capture_output=True,
                text=True,
                timeout=self.timeout_s,
                cwd=str(skill_root),
            )
        except subprocess.TimeoutExpired:
            result.notes.append(f"subprocess timed out after {self.timeout_s}s")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result
        except (OSError, FileNotFoundError) as e:
            result.notes.append(f"subprocess failed to start: {e}")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        result.stderr = proc.stderr
        result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000

        if proc.returncode != 0:
            result.notes.append(f"subprocess exited {proc.returncode}; stderr: {proc.stderr[:500]}")
            return result

        # Parse stdout as JSON if possible; otherwise wrap raw text.
        try:
            output = json.loads(proc.stdout) if proc.stdout.strip() else {}
        except json.JSONDecodeError:
            output = {"raw_stdout": proc.stdout, "warning": "stdout not valid JSON"}
            result.notes.append("stdout was not JSON — wrapped raw")

        # Persist output to disk for next-step hand-off.
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"step{step_num:02d}_{skill_name}.json"
        try:
            output_path.write_text(json.dumps(output, indent=2, sort_keys=True), encoding="utf-8")
        except OSError as e:
            result.notes.append(f"could not write output: {e}")
            return result

        result.status = "OK"
        result.output = output if isinstance(output, dict) else {"value": output}
        result.output_path = output_path
        return result


def select_invoker(prefer: str = "auto") -> Invoker:
    """Pick an invoker based on environment + preference.

    Args:
        prefer: "auto" | "mock" | "subprocess".
            auto → SubprocessInvoker if binary on PATH, else MockInvoker.
            mock → always MockInvoker.
            subprocess → always SubprocessInvoker (caller checks .invoke notes
                         for binary-not-found errors).

    Returns:
        Invoker instance ready to use.
    """
    prefer = prefer.lower()
    if prefer == "mock":
        return MockInvoker()
    if prefer == "subprocess":
        return SubprocessInvoker()
    # auto
    if SubprocessInvoker.is_available():
        return SubprocessInvoker()
    return MockInvoker()


def _strip_role_suffix(skill_name: str) -> str:
    """Convert 'sow-author' or 'sow-audit' → 'sow' for contract lookups."""
    if skill_name.endswith("-author"):
        return skill_name[: -len("-author")]
    if skill_name.endswith("-audit"):
        return skill_name[: -len("-audit")]
    return skill_name


def _stringify_inputs(inputs: dict) -> dict:
    """Best-effort stringification — Path → str, sets → list, fallback repr()."""
    out: dict = {}
    for k, v in inputs.items():
        if isinstance(v, Path):
            out[k] = str(v)
        elif isinstance(v, (list, tuple)):
            out[k] = [_stringify_one(x) for x in v]
        elif isinstance(v, set):
            out[k] = sorted([_stringify_one(x) for x in v])
        elif isinstance(v, dict):
            out[k] = _stringify_inputs(v)
        elif isinstance(v, (str, int, float, bool, type(None))):
            out[k] = v
        else:
            out[k] = repr(v)
    return out


def _stringify_one(v):
    if isinstance(v, Path):
        return str(v)
    if isinstance(v, dict):
        return _stringify_inputs(v)
    if isinstance(v, (str, int, float, bool, type(None))):
        return v
    return repr(v)
