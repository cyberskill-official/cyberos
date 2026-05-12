#!/usr/bin/env python3
"""
extract_agents_core.py — DECOMMISSIONED in Batch 27 (2026-05-12).

The compact `AGENTS-CORE.md` extraction has been removed. The single source of
truth for the protocol is `docs/memory/AGENTS.md`. Maintaining a separate
compact extract doubled the surface to keep in sync and offered no benefit now
that context windows comfortably hold the full 114 KB protocol.

Running this script is a no-op and exits with an explanatory message.
"""
from __future__ import annotations
import sys


def main() -> int:
    sys.stderr.write(
        "extract_agents_core.py: DECOMMISSIONED in Batch 27 (2026-05-12).\n"
        "\n"
        "The compact AGENTS-CORE.md was removed. Single source of truth:\n"
        "    docs/memory/AGENTS.md\n"
        "\n"
        "If you previously regenerated AGENTS-CORE.md with this script, you\n"
        "no longer need to. Update consumers to read AGENTS.md directly.\n"
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
