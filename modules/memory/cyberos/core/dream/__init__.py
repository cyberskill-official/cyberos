"""
cyberos.core.dream — out-of-band batch reflection pipeline (TASK-MEMORY-115).

Imports:

* ``runner.run`` — orchestrates a dream pass (snapshot + 4 detectors +
  DreamDiff persistence).
* ``applier.apply`` — replays a DreamDiff back into the chain under
  body-hash preconditions + AGENTS.md §7.7 anchor check.
* ``proposals.DreamProposal`` / ``DreamDiff`` — typed payloads.
"""

from cyberos.core.dream.proposals import (
    DreamProposal,
    DreamDiff,
    ProposalKind,
    generate_proposal_id,
    generate_dream_id,
)

__all__ = [
    "DreamProposal",
    "DreamDiff",
    "ProposalKind",
    "generate_proposal_id",
    "generate_dream_id",
]
