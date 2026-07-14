"""
cyberos.core.dream.proposals — DreamProposal + DreamDiff types
(TASK-MEMORY-115 §3).

Pure data classes + ID generators. No I/O.

The four proposal kinds (`merge | stale | new | verify`) are closed per
DEC-214 — adding a new kind requires a new FR + protocol amendment so
the apply-side validation surface stays bounded.
"""

from __future__ import annotations

import json
import secrets
import time
from dataclasses import asdict, dataclass, field
from typing import Literal, Optional

ProposalKind = Literal["merge", "stale", "new", "verify"]
_KINDS: frozenset[str] = frozenset({"merge", "stale", "new", "verify"})

# Crockford base32 — sortable (0-9, A-Z minus I, L, O, U) for ULIDs and
# uppercase base32 for proposal IDs. ULIDs are time-sortable + URL-safe.
_CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"


def generate_dream_id() -> str:
    """26-char Crockford base32 ULID.

    First 10 chars encode the millisecond timestamp; last 16 are random.
    Time-sortable, URL-safe, 128-bit.
    """
    ms = int(time.time() * 1000)
    ts = "".join(_CROCKFORD[(ms >> (5 * i)) & 0x1F] for i in range(10))[::-1]
    rand = "".join(_CROCKFORD[secrets.randbelow(32)] for _ in range(16))
    return ts + rand


def generate_proposal_id() -> str:
    """``P`` + 8 random Crockford base32 chars.

    Format: ``^P[0-9A-Z]{8}$`` (uppercase base32; 40 bits of entropy).
    Per TASK-MEMORY-115 §1 #1 (indirectly) + DreamProposal schema.
    """
    return "P" + "".join(_CROCKFORD[secrets.randbelow(32)] for _ in range(8))


@dataclass
class DreamProposal:
    """One detector-emitted proposal."""

    proposal_id: str
    op: ProposalKind
    paths: list[str] = field(default_factory=list)
    into: Optional[str] = None
    content_preview: str = ""
    rationale: str = ""
    input_session_ids: list[str] = field(default_factory=list)
    input_audit_seqs: list[int] = field(default_factory=list)
    precondition_body_hashes: dict[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if self.op not in _KINDS:
            raise ValueError(
                f"DreamProposal.op {self.op!r} not in closed enum {sorted(_KINDS)}"
            )
        if not (self.proposal_id.startswith("P") and len(self.proposal_id) == 9):
            raise ValueError(
                f"proposal_id must match P[0-9A-Z]{{8}}; got {self.proposal_id!r}"
            )
        if not self.rationale.strip():
            raise ValueError("DreamProposal.rationale must be non-empty")


@dataclass
class DreamDiff:
    """The artefact persisted at ``dreams/<ts>/diff.json``."""

    dream_id: str
    scope: str
    since: str           # ISO 8601 timestamp
    input_sessions: list[str] = field(default_factory=list)
    proposals: list[DreamProposal] = field(default_factory=list)
    metrics: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        return {
            "dream_id": self.dream_id,
            "scope": self.scope,
            "since": self.since,
            "input_sessions": list(self.input_sessions),
            "proposals": [asdict(p) for p in self.proposals],
            "metrics": dict(self.metrics),
        }

    def to_json(self, *, indent: int = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent, sort_keys=True)

    @classmethod
    def from_dict(cls, data: dict) -> "DreamDiff":
        props = [DreamProposal(**p) for p in data.get("proposals", [])]
        return cls(
            dream_id=data["dream_id"],
            scope=data.get("scope", "*"),
            since=data.get("since", ""),
            input_sessions=list(data.get("input_sessions", [])),
            proposals=props,
            metrics=dict(data.get("metrics", {})),
        )
