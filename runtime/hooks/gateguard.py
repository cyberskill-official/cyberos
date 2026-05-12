#!/usr/bin/env python3
"""
gateguard.py — PreToolUse hook for cyberos memory writes.

Implements 3-stage DENY → FORCE → ALLOW gate per gstack `gateguard` skill
(A/B tested +2.25 quality improvement).

Aspect 5.1 of the Layer-1 improvement catalog.

Per the cyberos protocol:
  - LLM self-evaluation doesn't work ("did you violate?" → "no")
  - But forcing investigation ("list all DECs with this tag") changes context
  - The investigation itself creates context that changes the output

3-stage gate:
  1. DENY: first attempt to write rejected with structured error
  2. FORCE: error lists exact facts the agent must gather
  3. ALLOW: retry permitted after fact-gathering pass succeeds

Install in ~/.claude/settings.json under "hooks.PreToolUse":
{
  "hooks": {
    "PreToolUse": [{
      "matcher": "Bash",
      "hooks": [{
        "type": "command",
        "command": "python3 /Users/stephencheng/Projects/CyberSkill/cyberos/runtime/hooks/gateguard.py"
      }]
    }]
  }
}

Reads tool input from stdin (Claude Code hook JSON contract).
Exit 0 = allow. Exit 2 + stderr message = deny with feedback.

Detects cyberos memory-write attempts by inspecting command for:
  - 'brain_writer.py' substring
  - 'outputs/brain_writer.py' substring
  - 'cyberos add' substring

Maintains state via /tmp/cyberos-gateguard-state-${SESSION_ID}.json so
retries can ALLOW after the first DENY.
"""
from __future__ import annotations
import hashlib
import json
import os
import re
import sys
from pathlib import Path
from datetime import datetime

STATE_DIR = Path("/tmp")
STATE_PREFIX = "cyberos-gateguard-state-"

def _state_path():
    sid = os.environ.get("CLAUDE_SESSION_ID") or os.environ.get("CYBEROS_SESSION_ID") or "default"
    return STATE_DIR / f"{STATE_PREFIX}{sid}.json"

def _load_state():
    p = _state_path()
    if not p.exists():
        return {"attempts": {}, "facts_seen": {}}
    try:
        return json.loads(p.read_text())
    except Exception:
        return {"attempts": {}, "facts_seen": {}}

def _save_state(state):
    _state_path().write_text(json.dumps(state, indent=2))

def _is_memory_write(cmd: str) -> str | None:
    """Returns write-type or None. Detects:
       - 'brain_writer.py write ...'
       - 'brain_writer.py create ...'
       - 'cyberos add DEC|REF|FACT|PERSON|PROJECT|PREFERENCE ...'
    """
    if not cmd:
        return None
    # brain_writer write/create
    m = re.search(r"brain_writer\.py\s+(write|create)\b", cmd)
    if m:
        return m.group(1)
    # cyberos add
    m = re.search(r"\bcyberos\s+add\s+(\w+)\b", cmd)
    if m:
        return f"add-{m.group(1).upper()}"
    return None

def _write_signature(cmd: str) -> str:
    """Deterministic ID for a write attempt — used to detect retries."""
    # Hash the command minus timestamps / random IDs
    cleaned = re.sub(r"\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\b", "TS", cmd)
    cleaned = re.sub(r"\bmem_[0-9a-f-]+\b", "ID", cleaned)
    return hashlib.sha256(cleaned.encode()).hexdigest()[:16]

def _force_checklist(write_type: str, cmd: str) -> list[str]:
    """Return the list of facts the agent must gather based on write type."""
    common = [
        "Confirm `.cyberos-memory/` resolves to real Finder path (§0.1 sanity)",
        "Read existing memories with same tags to detect duplicates",
        "Verify `manifest.protocol.sha256` matches canonical SHA of loaded AGENTS.md (§0.5)",
    ]
    if write_type in ("DEC", "add-DEC"):
        return common + [
            "List existing `memories/decisions/` for same domain tag",
            "Confirm no contradicting DEC exists (or use `supersedes:` chain)",
            "Confirm Nygard ADR sections present (Context, Decision, Alternatives, Consequences)",
        ]
    if write_type in ("REF", "add-REF"):
        return common + [
            "Confirm linked DEC-NNN exists",
            "Confirm `runtime/tests/refinements/REF-NNN/{capability,regression}.test.py` exist",
            "Confirm tier (TIER 1/2/3) specified",
            "Confirm AGENTS.md section to amend is named",
        ]
    if write_type in ("FACT", "add-FACT"):
        return common + [
            "Confirm `provenance.source_ref` points to a real file",
            "Recompute source_sha256 — must match `ingestion_coverage.source_sha256`",
            "Confirm coverage ≥ 0.80 OR `intentional_summary: true`",
        ]
    if write_type in ("PERSON", "add-PERSON"):
        return common + [
            "Confirm `consent.has_consent: true` AND `consent.consent_event` references an audit row",
            "Verify body contains NO denylisted content (compensation, gov-ID, home address, health PII)",
            "Verify classification is 'personnel'",
        ]
    if write_type in ("create",):
        return common + [
            "Verify `cyberos verify` passes before write (no pre-existing CRITICAL)",
            "Confirm tags use kebab-case (§5.2)",
        ]
    return common

def _facts_provided(cmd: str, state: dict) -> bool:
    """Heuristic: has the agent done investigation between the DENY and this retry?

    Detects by checking if the agent ran any of these between attempts:
      - `cyberos show` / `cyberos search` / `cyberos verify` / `grep memories/`
      - `cat .cyberos-memory/memories/`
      - `find .cyberos-memory/`

    We can't actually verify from one PreToolUse hook (it sees one call at a time).
    Instead: state file accumulates investigation-evidence commands. The retry
    is allowed if state shows ≥3 investigation commands since last DENY of this sig.
    """
    sig = _write_signature(cmd)
    investigations = state.get("facts_seen", {}).get(sig, [])
    return len(investigations) >= 3

def _record_investigation(cmd: str):
    """If this looks like an investigation command (read-only), record against all pending sigs."""
    if any(s in cmd for s in (
        "cyberos show", "cyberos search", "cyberos verify", "cyberos stats",
        "cat .cyberos-memory", "find .cyberos-memory", "grep -r memories/",
        "ls .cyberos-memory", "cyberos doc-consistency"
    )):
        state = _load_state()
        # Mark all pending denied sigs as having seen this investigation
        for sig in list(state.get("attempts", {}).keys()):
            state.setdefault("facts_seen", {}).setdefault(sig, []).append(cmd[:80])
        _save_state(state)

def main():
    # Read hook input from stdin
    try:
        payload = json.load(sys.stdin)
    except Exception:
        # Not JSON — pass through
        sys.exit(0)

    # Claude Code PreToolUse contract: { "tool_name": "Bash", "tool_input": { "command": "..." } }
    cmd = payload.get("tool_input", {}).get("command", "")

    # Record investigations (no DENY needed)
    _record_investigation(cmd)

    write_type = _is_memory_write(cmd)
    if not write_type:
        sys.exit(0)  # not a memory write — allow

    state = _load_state()
    sig = _write_signature(cmd)

    if sig in state.get("attempts", {}):
        # This is a retry — check if facts gathered
        if _facts_provided(cmd, state):
            # ALLOW
            print(json.dumps({"permissionDecision": "allow", "permissionDecisionReason":
                              f"gateguard ALLOW: retry after {len(state['facts_seen'].get(sig, []))} investigation commands"}))
            sys.exit(0)
        else:
            # Still need facts
            checklist = _force_checklist(write_type, cmd)
            msg = (
                f"⛔ gateguard FORCE: still need investigation before this {write_type} write.\n"
                "Run at least 3 of these read-only commands first:\n"
                + "\n".join(f"  · {c}" for c in checklist)
                + "\nSeen so far: " + str(len(state.get("facts_seen", {}).get(sig, []))) + "/3"
            )
            print(json.dumps({"permissionDecision": "ask", "permissionDecisionReason": msg}))
            sys.exit(0)

    # First attempt — DENY + emit FORCE checklist
    state.setdefault("attempts", {})[sig] = {
        "ts": datetime.utcnow().isoformat() + "Z",
        "write_type": write_type,
        "cmd_preview": cmd[:160],
    }
    _save_state(state)

    checklist = _force_checklist(write_type, cmd)
    msg = (
        f"⛔ gateguard DENY (first attempt) — write_type={write_type}\n"
        "BEFORE retrying this write, INVESTIGATE these facts:\n"
        + "\n".join(f"  · {c}" for c in checklist)
        + "\n\nRationale: LLM self-evaluation doesn't catch what investigation reveals.\n"
        + "Run any 3+ of: cyberos show / search / verify / stats / doc-consistency, or cat/grep the relevant memories.\n"
        + "Then retry the exact same write command — the gate will ALLOW on retry."
    )
    print(json.dumps({"permissionDecision": "ask", "permissionDecisionReason": msg}))
    sys.exit(0)

if __name__ == "__main__":
    main()
