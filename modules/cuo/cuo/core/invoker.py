"""Invoker — runs a single skill given inputs and returns its output.

Two invoker implementations:

- `SubprocessInvoker` (requires `cyberos-skill` binary on PATH) — invokes the
  Rust CLI via subprocess, passing JSON inputs on stdin and capturing JSON
  outputs from stdout.

- `LLMInvoker` (requires `ANTHROPIC_API_KEY` or similar) — drives prompt-only
  SDP skills via LLM API. See `llm_invoker.py`.

If neither is available, `select_invoker()` raises an error. No silent fallbacks.

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
        *,
        file_prefix: str = "",
    ) -> StepResult:
        """Invoke `skill_name` with `inputs` and write its output to `output_dir`.

        Args:
            skill_name: e.g. "software-requirements-specification-author" — must correspond to a directory at
                `skill_root/<skill_name>/SKILL.md`.
            inputs: dict of input-name → value (file path string OR primitive).
            skill_root: filesystem path to `skill/` (must contain MODULE.md).
            output_dir: directory where step output JSON is written.
            step_num: 1-based step number for filename suffix.
            file_prefix: optional prefix for output filename (e.g. "TASK-AUTH-001_").

        Returns:
            StepResult with status + output + output_path + duration.
        """
        raise NotImplementedError


class SubprocessInvoker(Invoker):
    """Subprocess invoker — calls `cyberos-skill run <name>` via the OS PATH.

    Passes a JSON inputs object on stdin and parses stdout as JSON output.
    Requires the Rust binary to be built and on PATH.

    The Rust CLI uses `--executor auto` by default (WASM if compiled, otherwise
    script). SDP-driven prompt-only skills will likely produce minimal output;
    they're meant to be driven by an LLM (see `LLMInvoker`).
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
        *,
        file_prefix: str = "",
    ) -> StepResult:
        t0 = time.monotonic_ns()
        result = StepResult(step=step_num, skill=skill_name, status="FAILED")

        if not self.is_available(self.binary):
            result.notes.append(
                f"binary {self.binary!r} not on PATH — "
                "build skill/ via `cargo install --path modules/skill/crates/cli`"
            )
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        stdin_json = json.dumps({"skill": skill_name, "step": step_num, "inputs": _stringify_inputs(inputs)})

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
        output_path = output_dir / f"{file_prefix}step{step_num:02d}_{skill_name}.json"
        try:
            output_path.write_text(json.dumps(output, indent=2, sort_keys=True), encoding="utf-8")
        except OSError as e:
            result.notes.append(f"could not write output: {e}")
            return result

        result.status = "OK"
        result.output = output if isinstance(output, dict) else {"value": output}
        result.output_path = output_path
        return result


class CompositeInvoker(Invoker):
    """Tries SubprocessInvoker first, falls back to LLMInvoker on failure.

    This handles the common case where some skills have scripts/WASM (need
    SubprocessInvoker) and others are prompt-only (need LLMInvoker).
    """

    def __init__(self, primary: Invoker, fallback: Invoker):
        self.primary = primary
        self.fallback = fallback

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
        result = self.primary.invoke(skill_name, inputs, skill_root, output_dir, step_num, file_prefix=file_prefix)
        if result.status == "FAILED":
            # Primary failed — try fallback
            fallback_result = self.fallback.invoke(
                skill_name, inputs, skill_root, output_dir, step_num, file_prefix=file_prefix
            )
            if fallback_result.status != "FAILED":
                return fallback_result
            # Both failed — merge notes so the user sees why each path failed,
            # not just the subprocess error.
            result.notes.extend(
                [f"[fallback {type(self.fallback).__name__}] {n}" for n in fallback_result.notes]
            )
        return result


def select_invoker(prefer: str = "auto") -> Invoker:
    """Pick an invoker based on environment + preference.

    Args:
        prefer: "auto" | "subprocess" | "llm".
            auto → SubprocessInvoker if binary on PATH, else LLMInvoker if
                    API key available, else raises RuntimeError.
            subprocess → always SubprocessInvoker.
            llm → always LLMInvoker.

    Returns:
        Invoker instance ready to use.

    Raises:
        RuntimeError: if no suitable invoker is available.
    """
    prefer = prefer.lower()
    if prefer == "subprocess":
        return SubprocessInvoker()
    if prefer == "llm":
        from cuo.core.llm_invoker import LLMInvoker
        return LLMInvoker()
    # auto — CompositeInvoker tries SubprocessInvoker first (handles script +
    # WASM skills), falls back to LLMInvoker (handles prompt-only skills).
    has_subprocess = SubprocessInvoker.is_available()
    has_llm = False
    try:
        import anthropic  # noqa: F401
        has_llm = True
    except ImportError:
        pass

    if has_subprocess and has_llm:
        from cuo.core.llm_invoker import LLMInvoker
        return CompositeInvoker(SubprocessInvoker(), LLMInvoker())
    if has_subprocess:
        return SubprocessInvoker()
    if has_llm:
        from cuo.core.llm_invoker import LLMInvoker
        return LLMInvoker()
    raise RuntimeError(
        "No skill invoker available. Either:\n"
        "  1. Build and install cyberos-skill: cargo install --path modules/skill/crates/cli\n"
        "  2. Install anthropic SDK: pip install anthropic\n"
        "     Then set ANTHROPIC_API_KEY environment variable"
    )


def _strip_role_suffix(skill_name: str) -> str:
    """Convert 'statement-of-work-author' or 'statement-of-work-audit' → 'sow' for contract lookups."""
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


def detect_host_environment() -> str | None:
    """Detect if running inside a host LLM environment.

    Returns the host type string ('claude-code', 'cursor', 'codex')
    or None if running in a plain terminal.
    """
    if os.environ.get("CLAUDE_CODE_SESSION"):
        return "claude-code"
    if os.environ.get("CURSOR_SESSION"):
        return "cursor"
    if os.environ.get("CODEX_SESSION"):
        return "codex"
    # Also check for generic MCP/tool context indicators
    if os.environ.get("MCP_SERVER_NAME"):
        return "mcp-host"
    return None
