"""Memory bridge — records CUO decisions in the BRAIN audit chain.

Phase 1: write a flat memory file under
``<memory-module>/.cyberos-memory/meta/cuo-decisions/<ts>.md``. The file
body is plain Markdown — no frontmatter — so the memory module's
existing reader will treat it as an opaque artefact.

Phase 2 will route this through the memory module's `Writer` so the
decision lands as a proper audit row on the chain with a SHA-256 + a
chain pointer. For now the on-disk file is enough to demonstrate the
contract; the parent's audit-chain integration is a follow-up.
"""

from __future__ import annotations

import json
import time
from pathlib import Path


def record_decision(
    decision_dict: dict,
    result_dict: dict,
    memory_module_root: Path,
) -> Path:
    """Append a CUO decision to the BRAIN.

    Returns the path of the file written. The file is named
    ``<ts_ns>.md``; ts_ns is the nanosecond wall-clock at write time.
    """
    ts_ns = time.time_ns()
    body = (
        "# CUO routing decision\n\n"
        f"_ts_ns_: `{ts_ns}`\n\n"
        "## Decision\n\n"
        "```json\n"
        f"{json.dumps(decision_dict, indent=2, ensure_ascii=False)}\n"
        "```\n\n"
        "## Result\n\n"
        "```json\n"
        f"{json.dumps(result_dict, indent=2, ensure_ascii=False)}\n"
        "```\n"
    )
    path = memory_module_root / ".cyberos-memory" / "meta" / "cuo-decisions"
    path.mkdir(parents=True, exist_ok=True)
    target_file = path / f"{ts_ns}.md"
    target_file.write_text(body, encoding="utf-8")
    return target_file
