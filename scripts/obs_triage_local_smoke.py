#!/usr/bin/env python3
"""Local smoke: drive the real obs.triage-alert skill through a local model, no cloud key.

This isolates the triage reasoning leg (skill prompt -> LLM -> verdict contract) from the MCP gateway
and obs-router. It points cuo's LLMInvoker at an Anthropic-compatible endpoint (LM Studio serves one)
via ANTHROPIC_BASE_URL, then runs one sample alert through the same `handle_triage_request` path
obs-router uses, and prints the verdict.

The verdict is what obs-router would route on: confidence >= 0.70 posts to CHAT, below it pages on-call.
With no live metrics and an empty runbook corpus, the skill is meant to return a calibrated-LOW
confidence with an honest summary - that is correct behavior, and still proves the leg works.

Run from the repo root with LM Studio serving your model on :1234:

  ANTHROPIC_BASE_URL=http://127.0.0.1:1234 \
  ANTHROPIC_API_KEY=lm-studio \
  ANTHROPIC_MODEL=qwen3.6-35b-a3b-uncensored-hauhaucs-aggressive \
  python3 scripts/obs_triage_local_smoke.py

ANTHROPIC_API_KEY can be any non-empty placeholder; LM Studio ignores it. A 35B reasoning model can
take ~50s. If the endpoint path is wrong you will see an API error inside the verdict summary (the path
degrades safely), and we adjust ANTHROPIC_BASE_URL from there.
"""

from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "modules" / "cuo"))


def main() -> int:
    base_url = os.environ.get("ANTHROPIC_BASE_URL")
    if not base_url:
        print(
            "Set ANTHROPIC_BASE_URL to your local Anthropic-compatible endpoint, e.g.\n"
            "  ANTHROPIC_BASE_URL=http://127.0.0.1:1234"
        )
        return 2

    try:
        import anthropic  # noqa: F401
    except ImportError:
        print("The 'anthropic' SDK is not importable in this Python env. Install it:\n"
              "  pip install anthropic\n"
              "(or run with the same interpreter that has the cuo env).")
        return 2

    from cuo.core.llm_invoker import LLMInvoker
    from cuo.triage_server import handle_triage_request, SKILL_HANDLE

    invoker = LLMInvoker()  # reads ANTHROPIC_BASE_URL / ANTHROPIC_API_KEY / ANTHROPIC_MODEL from env
    print(f"invoker mode={invoker.mode}  model={invoker.model}  base_url={invoker.base_url}")
    if invoker.mode != "real":
        print("Invoker is not in 'real' mode - set a non-empty ANTHROPIC_API_KEY (a placeholder is fine).")
        return 2

    skill_root = ROOT / "modules" / "skill"
    if not (skill_root / "obs-triage-alert" / "SKILL.md").is_file():
        print(f"Skill not found under {skill_root}/obs-triage-alert/SKILL.md")
        return 2

    payload = {
        "skill": SKILL_HANDLE,
        "alert": {
            "name": "HighErrorRate",
            "severity": "sev2",
            "fingerprint": "demo-fp-1",
            "trace_id": "",
            "summary": "5xx error ratio on ai-gateway above 2% for 10 minutes after a deploy",
        },
    }

    print("\ncalling the local model (a reasoning model can take ~50s)...\n")
    status, body = handle_triage_request(
        payload,
        invoker=invoker,
        skill_root=skill_root,
        output_dir=Path(tempfile.gettempdir()) / "obs-triage-smoke",
    )

    print(f"HTTP {status}")
    print(json.dumps(body, indent=2))

    conf = body.get("confidence", 0.0) if isinstance(body, dict) else 0.0
    route = "CONFIDENT -> obs-router posts to CHAT" if conf >= 0.70 else "low -> obs-router pages on-call (safe)"
    print(f"\n=> confidence {conf}  ({route})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
