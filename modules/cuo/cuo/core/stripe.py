"""stripe — categorical key for refinement proposals (TASK-CUO-201).

Format: `<skill_slug>:<signal_id>:<8 hex pattern_hash>`

The pattern_hash is a SHA-256 over a deterministic projection of the evidence
rows — same evidence shape → same stripe id across processes and sessions.
This enables dedup: two proposals with the same stripe are about the same
class of issue.

Projection rules (per evidence-row content):
  * For audit-verdict-failure evidence: hash sorted set of rule_ids that breached
  * For routed-back evidence: hash sorted set of phases where rework triggered
  * For drift evidence: hash sorted set of (input_hash, output_hash[:8]) pairs
  * For coverage-fail evidence: hash sorted set of test_name strings
  * Default: hash sorted list of (op, path)-tuples
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from typing import Optional


# Stripe id format — load-bearing regex; used in dedup glob and validation.
STRIPE_ID_RE = re.compile(r"^([a-z0-9_-]+(?:/[a-z0-9_-]+)?):([a-z_]+):([0-9a-f]{8})$")


@dataclass
class StripeId:
    """Parsed form of a stripe id. Use compute_stripe() to build from evidence."""
    scope: str       # skill slug OR persona/workflow_slug
    signal_id: str
    pattern_hash: str  # always 8 hex chars

    def __str__(self) -> str:
        return f"{self.scope}:{self.signal_id}:{self.pattern_hash}"

    @classmethod
    def parse(cls, s: str) -> Optional["StripeId"]:
        m = STRIPE_ID_RE.match(s)
        if not m:
            return None
        return cls(scope=m.group(1), signal_id=m.group(2), pattern_hash=m.group(3))


def compute_stripe(
    skill_name: str,
    signal_id: str,
    evidence_rows: list[dict],
) -> StripeId:
    """Compute a deterministic stripe id from evidence rows.

    The projection is signal-specific; defaults to a structural shape of the
    rows when no signal-specific projection applies. SHA-256 of the canonical
    JSON, truncated to 8 hex chars.
    """
    projection = _project(signal_id, evidence_rows)
    canon = json.dumps(projection, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    digest = hashlib.sha256(canon.encode("utf-8")).hexdigest()
    return StripeId(
        scope=skill_name,
        signal_id=signal_id,
        pattern_hash=digest[:8],
    )


def _project(signal_id: str, rows: list[dict]) -> list:
    """Return a sortable, JSON-serialisable projection of the evidence.

    Two evidence sets that share their projection produce the same stripe.
    """
    if signal_id == "rule_reversal_streak":
        return sorted({
            (r.get("extra") or {}).get("rule_id", "?")
            for r in rows
        })
    if signal_id == "deterministic_drift":
        return sorted({
            ((r.get("extra") or {}).get("input_hash", "?"),
             str((r.get("extra") or {}).get("output_hash", "?"))[:8])
            for r in rows
        })
    if signal_id == "acceptance_rate_below":
        # The set of failure REASONS (or outcome strings) — multiple FRs failing
        # for the same reason form one stripe.
        return sorted({
            (r.get("extra") or {}).get("outcome", "")
            + ":"
            + (r.get("extra") or {}).get("rework_reason", "")[:50]
            for r in rows
        })
    if signal_id == "hitl_pause_rate_above":
        return sorted({
            (r.get("extra") or {}).get("escalation_reason", "?")
            for r in rows
        })
    if signal_id == "needs_human_rate_above":
        return sorted({
            (r.get("extra") or {}).get("hitl_reason", "?")
            for r in rows
        })
    # Default — structural shape of the rows
    return sorted({
        (r.get("op", ""), (r.get("path") or "")[:64])
        for r in rows
    })
