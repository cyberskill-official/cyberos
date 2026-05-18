#!/usr/bin/env python3
"""
runtime/skill_runners/base.py — base class for deterministic per-skill runners.

Tier α.1 (Batch 21).

Each skill (fr-with-tasks, feature-request-author, product-requirements-document-author, etc.) gets a concrete
subclass that implements the small fraction of skill logic that is
deterministic (interview loop, INVARIANT checks, content-gate filtering,
audit-fix loop). Only the judgement-driven authoring is delegated to
Claude.

Today's `cyberos chain run --with-llm` sends the whole SKILL.md to
Claude and trusts it to follow. This base class flips the ratio:
~20% LLM judgement, ~80% deterministic per-skill logic.

Subclass contract:

  class FrWithTasksRunner(BaseSkillRunner):
      skill_id = "cuo/cpo/fr-with-tasks"
      interview_questions = [...]   # from STANDALONE_INTERVIEW.md
      invariants = [...]            # from INVARIANTS.md
      output_template = "..."       # from cyberos/docs/contracts/*

      def author_body(self, inputs: dict, llm) -> str:
          ...                        # only thing that calls Claude

      def validate_emit(self, body: str, inputs: dict) -> list[dict]:
          ...                        # INVARIANT enforcement

Then the chain calls:

  runner = FrWithTasksRunner(brain_root, manifest)
  artefact_path = runner.run(inputs, max_iterations=3)
"""
from __future__ import annotations
import importlib.util
import json
import os
import re
import sys
import time
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


@dataclass
class RunResult:
    status: str                       # "PASS" | "HITL_PAUSE" | "EXHAUSTED" | "FAILED"
    artefact_path: Path | None = None
    iterations: int = 0
    findings: list[dict] = field(default_factory=list)
    hitl_question: str | None = None
    tokens_used: int = 0
    cost_usd: float = 0.0


class BaseSkillRunner:
    """Subclass and override the four hooks."""
    skill_id: str = "override-me"
    skill_version: str = "0.1.0"
    output_filename_pattern: str = "<skill>.md"   # how to name the emitted file

    def __init__(self, brain_root: Path, manifest: dict | None = None,
                 model: str = "claude-sonnet-4-6",
                 step_max_tokens: int = 4000,
                 input_per_mtok: float = 3.0,
                 output_per_mtok: float = 15.0):
        self.brain_root = Path(brain_root)
        self.manifest = manifest or {}
        self.model = model
        self.step_max_tokens = step_max_tokens
        self.input_per_mtok = input_per_mtok
        self.output_per_mtok = output_per_mtok
        self._anthropic = None
        self._telemetry: list[dict] = []

    # ---- Subclass hooks (override these) ---------------------------------

    def interview(self, inputs: dict) -> dict:
        """Conduct the standalone interview. Default: pass through inputs unchanged."""
        return inputs

    def build_prompt(self, inputs: dict, prior_artefacts: list[str] | None = None) -> str:
        """Compose the Claude prompt. Subclass should reference the SKILL.md."""
        skill_md_path = self.brain_root / "docs" / "skills" / self.skill_id / "SKILL.md"
        skill_md = skill_md_path.read_text(encoding="utf-8") if skill_md_path.exists() else "(SKILL.md missing)"
        prior = "\n\n".join(prior_artefacts or [])
        return f"""You are running the cyberos skill `{self.skill_id}`.

# SKILL.md
{skill_md[:8000]}

# Inputs
{json.dumps(inputs, indent=2, default=str)[:4000]}

{f"# Prior artefacts{chr(10)}{prior[:4000]}" if prior else ""}

# Your task

Emit the artefact this skill is responsible for, following the SKILL.md body shape exactly.
No em dashes. No AI vocabulary. Cite source_ref + authority markers per AGENTS.md §5.1.
Wrap any quoted operator input in <untrusted_content> blocks.

Output ONLY the artefact body. No fences, no commentary."""

    def author_body(self, inputs: dict, llm_call) -> tuple[str, int, int]:
        """Call Claude to produce the body. Returns (body, input_tokens, output_tokens)."""
        prompt = self.build_prompt(inputs, inputs.get("prior_artefacts"))
        return llm_call(prompt)

    def validate_emit(self, body: str, inputs: dict) -> list[dict]:
        """Run INVARIANT checks on the emitted body. Return list of findings.

        Default: minimal voice + content-gate check. Subclasses extend.
        """
        findings = []
        # Voice gate
        for em in ("—", "–"):
            if em in body:
                findings.append({"invariant": "voice-no-em-dash",
                                 "severity": "WARN",
                                 "fix_hint": f"replace {em!r} with comma or sentence break"})
                break
        AI_VOCAB = ["leverage", "robust", "ensure", "comprehensive", "seamless",
                    "delve", "navigate", "tapestry", "facilitate", "utilize"]
        for word in AI_VOCAB:
            if re.search(rf"\b{word}\b", body, re.IGNORECASE):
                findings.append({"invariant": "voice-no-ai-vocab",
                                 "severity": "WARN",
                                 "fix_hint": f"rewrite {word!r} plainly"})
                break  # one is enough; flagging more is noisy
        # Untrusted-content not stripped by mistake
        if "<untrusted_content" in body and "</untrusted_content>" not in body:
            findings.append({"invariant": "untrusted-content-unbalanced",
                             "severity": "CRITICAL",
                             "fix_hint": "close the <untrusted_content> tag"})
        return findings

    def output_path(self, inputs: dict, output_dir: Path) -> Path:
        """Resolve where to write the artefact."""
        skill_name = self.skill_id.split("/")[-1]
        return output_dir / self.output_filename_pattern.replace("<skill>", skill_name)

    # ---- Multi-iteration self-audit loop (Tier α.3) ----------------------

    def run(self, inputs: dict, output_dir: Path,
            max_iterations: int = 3,
            cache: object | None = None) -> RunResult:
        """Main entry. Iterates emit → validate → fix until clean OR exhausted."""
        out_dir = Path(output_dir)
        out_dir.mkdir(parents=True, exist_ok=True)

        # Tier α.9 — cache check
        cache_key = None
        if cache is not None:
            cache_key = cache.compute_key(self.skill_id, self.skill_version, inputs)
            cached = cache.get(cache_key)
            if cached is not None:
                out_path = self.output_path(inputs, out_dir)
                out_path.write_text(cached, encoding="utf-8")
                self._log_telemetry(inputs, "cache-hit", iterations=0, tokens=0, cost=0.0,
                                    status="PASS", out_path=out_path)
                return RunResult(status="PASS", artefact_path=out_path, iterations=0,
                                 findings=[], tokens_used=0, cost_usd=0.0)

        # Interview phase
        try:
            enriched = self.interview(inputs)
        except SystemExit:
            raise
        except Exception as e:
            return RunResult(status="FAILED", findings=[{"phase": "interview", "error": str(e)}])

        # Build the LLM caller
        def llm_call(prompt: str) -> tuple[str, int, int]:
            try:
                if self._anthropic is None:
                    import anthropic  # type: ignore
                    self._anthropic = anthropic.Anthropic()
                msg = self._anthropic.messages.create(
                    model=self.model,
                    max_tokens=self.step_max_tokens,
                    messages=[{"role": "user", "content": prompt}],
                )
                body = "\n".join(b.text for b in msg.content if hasattr(b, "text"))
                return body, msg.usage.input_tokens, msg.usage.output_tokens
            except ImportError:
                raise RuntimeError("anthropic SDK not installed; `pip install anthropic`")
            except Exception as e:
                if "ANTHROPIC_API_KEY" in str(e) or "authentication" in str(e).lower():
                    raise RuntimeError("ANTHROPIC_API_KEY not set or invalid")
                raise

        total_in = total_out = 0
        body = ""
        findings: list[dict] = []
        out_path = self.output_path(enriched, out_dir)

        for it in range(1, max_iterations + 1):
            try:
                body, in_tok, out_tok = self.author_body(enriched, llm_call)
                total_in += in_tok; total_out += out_tok
            except RuntimeError as e:
                return RunResult(status="FAILED", findings=[{"phase": "author", "error": str(e)}],
                                 iterations=it)
            findings = self.validate_emit(body, enriched)
            if not findings:
                # Success
                out_path.write_text(body, encoding="utf-8")
                cost = (total_in / 1_000_000.0 * self.input_per_mtok
                        + total_out / 1_000_000.0 * self.output_per_mtok)
                if cache is not None and cache_key:
                    cache.set(cache_key, body)
                self._log_telemetry(enriched, "PASS", iterations=it,
                                    tokens=total_in + total_out, cost=cost,
                                    status="PASS", out_path=out_path)
                return RunResult(status="PASS", artefact_path=out_path, iterations=it,
                                 findings=[], tokens_used=total_in + total_out, cost_usd=cost)

            # CRITICAL → immediate HITL pause
            if any(f.get("severity") == "CRITICAL" for f in findings):
                cost = (total_in / 1_000_000.0 * self.input_per_mtok
                        + total_out / 1_000_000.0 * self.output_per_mtok)
                self._log_telemetry(enriched, "HITL_PAUSE", iterations=it,
                                    tokens=total_in + total_out, cost=cost,
                                    status="HITL_PAUSE", out_path=None,
                                    findings=findings)
                return RunResult(
                    status="HITL_PAUSE",
                    findings=findings,
                    hitl_question=f"CRITICAL invariant breached: {findings[0].get('invariant')}",
                    iterations=it,
                    tokens_used=total_in + total_out,
                    cost_usd=cost,
                )

            # WARN → re-prompt for fix
            if it < max_iterations:
                fix_hints = "\n".join(f"- {f.get('invariant')}: {f.get('fix_hint')}" for f in findings)
                enriched = dict(enriched)
                enriched["_prior_body"] = body
                enriched["_fix_hints"] = fix_hints
                # build_prompt subclasses may append _fix_hints; default already passes inputs

        # Exhausted
        cost = (total_in / 1_000_000.0 * self.input_per_mtok
                + total_out / 1_000_000.0 * self.output_per_mtok)
        out_path.write_text(body, encoding="utf-8")  # write even when exhausted
        self._log_telemetry(enriched, "EXHAUSTED", iterations=max_iterations,
                            tokens=total_in + total_out, cost=cost,
                            status="EXHAUSTED", out_path=out_path, findings=findings)
        return RunResult(status="EXHAUSTED", artefact_path=out_path, iterations=max_iterations,
                         findings=findings, tokens_used=total_in + total_out, cost_usd=cost)

    # ---- Tier α.8 — uniform telemetry ------------------------------------

    def _log_telemetry(self, inputs: dict, phase: str, **kw):
        log_dir = Path.home() / ".cyberos" / "analytics"
        log_dir.mkdir(parents=True, exist_ok=True)
        log = log_dir / "skill-runs.jsonl"
        row = {
            "ts": datetime.now(ICT).isoformat(timespec="seconds"),
            "skill_id": self.skill_id,
            "skill_version": self.skill_version,
            "phase": phase,
            "model": self.model,
            "input_hash": _input_fingerprint(inputs),
            **{k: (str(v) if isinstance(v, Path) else v) for k, v in kw.items()},
        }
        with open(log, "a") as f:
            f.write(json.dumps(row, default=str) + "\n")
        self._telemetry.append(row)


def _input_fingerprint(inputs: dict) -> str:
    """Stable hash of input dict for cache + telemetry."""
    import hashlib
    blob = json.dumps({k: str(v) for k, v in inputs.items() if not k.startswith("_")},
                      sort_keys=True)
    return hashlib.sha256(blob.encode("utf-8")).hexdigest()[:16]


# ---- Tier α.9 — caching ----------------------------------------------------

class SkillCache:
    """Filesystem-backed cache: (skill_id + skill_version + input_hash) → artefact body."""
    def __init__(self, cache_dir: Path | None = None):
        self.cache_dir = Path(cache_dir or Path.home() / ".cyberos" / "skill-cache")
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    def compute_key(self, skill_id: str, version: str, inputs: dict) -> str:
        return f"{skill_id.replace('/', '_')}__{version}__{_input_fingerprint(inputs)}"

    def get(self, key: str) -> str | None:
        p = self.cache_dir / f"{key}.md"
        if p.exists():
            return p.read_text(encoding="utf-8")
        return None

    def set(self, key: str, body: str):
        p = self.cache_dir / f"{key}.md"
        p.write_text(body, encoding="utf-8")


# ---- Tier α.10 — streaming (opt-in helper) --------------------------------

def llm_call_streaming(model: str, prompt: str, max_tokens: int,
                       on_token=None) -> tuple[str, int, int]:
    """Like the inner llm_call, but streams. on_token(text_delta) called per chunk."""
    import anthropic  # type: ignore
    client = anthropic.Anthropic()
    chunks = []
    in_tok = out_tok = 0
    with client.messages.stream(model=model, max_tokens=max_tokens,
                                 messages=[{"role": "user", "content": prompt}]) as stream:
        for delta in stream.text_stream:
            chunks.append(delta)
            if on_token:
                on_token(delta)
        # Final usage
        msg = stream.get_final_message()
        in_tok = msg.usage.input_tokens
        out_tok = msg.usage.output_tokens
    return "".join(chunks), in_tok, out_tok


# ---- Discovery helper for the chain orchestrator --------------------------

def load_runner(skill_id: str, brain_root: Path) -> "BaseSkillRunner | None":
    """Find runtime/skill_runners/<basename>.py for skill_id. Return instance or None."""
    base = skill_id.split("/")[-1].replace("-", "_")
    runner_path = brain_root / "runtime" / "skill_runners" / f"{base}.py"
    if not runner_path.exists():
        return None
    spec = importlib.util.spec_from_file_location(f"skill_runner_{base}", runner_path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    # Find the BaseSkillRunner subclass
    for name in dir(mod):
        obj = getattr(mod, name)
        if isinstance(obj, type) and issubclass(obj, BaseSkillRunner) and obj is not BaseSkillRunner:
            return obj(brain_root)
    return None
