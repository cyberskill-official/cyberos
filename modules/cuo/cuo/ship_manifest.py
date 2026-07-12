"""ship-manifest@1 helpers (FR-CUO-206).

Pure functions over the contract in
modules/skill/contracts/feature-request/SHIP-MANIFEST.md. The manifest is a CACHE:
resume re-hashes artefacts, human gates always re-ask, deletion is always safe.
"""
from __future__ import annotations

import json
import os
import re
import tempfile

MANIFEST_VERSION = "ship-manifest@1"
STEP_STATUSES = {"pending", "done", "failed", "skipped-conditional"}
HITL_GATES = {None, "review_approval", "final_acceptance"}
GATE_STEPS = {19, 20, 30, 31}  # reviewing->ready_to_test and testing->done transitions
_REQUIRED_ROOT = [
    "manifest_version", "fr_id", "fr_sha256", "workflow_version", "started_at",
    "updated_at", "current_step", "routed_back_count", "steps", "hitl",
]
_PRIORITY_RANK = {"MUST": 0, "SHOULD": 1, "COULD": 2}


def validate(m: dict) -> list:
    """Return a list of schema violations (empty = valid ship-manifest@1)."""
    errs = []
    for k in _REQUIRED_ROOT:
        if k not in m:
            errs.append(f"missing root field: {k}")
    if errs:
        return errs
    if m["manifest_version"] != MANIFEST_VERSION:
        errs.append(f"manifest_version != {MANIFEST_VERSION}")
    if not re.fullmatch(r"[0-9a-f]{64}", str(m["fr_sha256"] or "")):
        errs.append("fr_sha256 is not a hex64 digest")
    if not isinstance(m["current_step"], int) or not 1 <= m["current_step"] <= 31:
        errs.append("current_step outside 1..31")
    if not isinstance(m["routed_back_count"], int) or m["routed_back_count"] < 0:
        errs.append("routed_back_count negative or non-int")
    if m["hitl"].get("gate") not in HITL_GATES:
        errs.append("hitl.gate outside enum")
    for s in m["steps"]:
        if s.get("status") not in STEP_STATUSES:
            errs.append(f"step {s.get('index')}: status outside enum")
        if not isinstance(s.get("index"), int) or not 1 <= s["index"] <= 31:
            errs.append(f"step index outside 1..31: {s.get('index')}")
    return errs


def write_atomic(m: dict, path: str) -> None:
    """Two-phase atomic write: .tmp.<nonce> then rename (memory-protocol discipline)."""
    d = os.path.dirname(path) or "."
    fd, tmp = tempfile.mkstemp(prefix=os.path.basename(path) + ".tmp.", dir=d)
    try:
        with os.fdopen(fd, "w") as f:
            json.dump(m, f, indent=2, sort_keys=True)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmp, path)
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)


def resume_plan(m: dict, workflow_version: str, fr_sha256: str, hash_of) -> dict:
    """Compute the resume plan. hash_of(path) -> hex digest or None.

    Returns {"action": "needs_human"|"resume", "start_step": int, "stale_from": int|None,
             "gate_pending": str|None, "reason": str}.
    The plan NEVER treats hitl.requested_at as an approval (contract: gates re-ask).
    """
    if m["workflow_version"] != workflow_version:
        return {"action": "needs_human", "start_step": None, "stale_from": None,
                "gate_pending": None,
                "reason": f"workflow_version mismatch: manifest {m['workflow_version']} vs {workflow_version}"}
    if m["fr_sha256"] != fr_sha256:
        return {"action": "resume", "start_step": 1, "stale_from": 1, "gate_pending": None,
                "reason": "FR spec changed since run start (fr_sha256 mismatch) - all steps stale"}
    stale_from = None
    for s in sorted(m["steps"], key=lambda x: x["index"]):
        if s["status"] == "done" and s.get("artefact_sha256"):
            if hash_of(s.get("artefact_path")) != s["artefact_sha256"]:
                stale_from = s["index"]
                break
    if stale_from is not None:
        start = stale_from
    else:
        done = {s["index"] for s in m["steps"] if s["status"] in ("done", "skipped-conditional")}
        start = next(i for i in range(1, 32) if i not in done)
    gate = "review_approval" if start in (19, 20) else "final_acceptance" if start in (30, 31) else None
    reason = (f"steps verified, continuing at step {start}/31" if stale_from is None
              else f"artefact for step {stale_from} diverged - redoing from step {stale_from}")
    if gate:
        reason += " (human gate: approval will be re-requested - recorded requested_at is never an approval)"
    return {"action": "resume", "start_step": start, "stale_from": stale_from,
            "gate_pending": gate, "reason": reason}


def select_next(frs: list) -> dict:
    """Deterministic queue selection per SHIP-MANIFEST.md.

    frs: [{id, status, priority, created, depends_on: [...]}]. Returns
    {"picked": id|None, "reason": str}.
    """
    done = {f["id"] for f in frs if f["status"] == "done"}
    eligible = [f for f in frs
                if f["status"] == "ready_to_implement"
                and all(d in done for d in f.get("depends_on", []))]
    if not eligible:
        return {"picked": None, "reason": "queue: no eligible FR (ready_to_implement with all depends_on done)"}
    eligible.sort(key=lambda f: (_PRIORITY_RANK.get(f.get("priority", "COULD"), 9),
                                 str(f.get("created", "")), f["id"]))
    w = eligible[0]
    return {"picked": w["id"],
            "reason": (f"queue: picked {w['id']} (priority={w.get('priority')}, "
                       f"created={w.get('created')}) over {len(eligible) - 1} other eligible FRs")}


def finalize(m: dict, outcome: str) -> dict:
    """Terminal handling. outcome: 'done' -> delete; 'route_back' -> keep, count += 1."""
    if outcome == "done":
        return {"action": "delete_manifest"}
    if outcome == "route_back":
        m = dict(m, routed_back_count=m["routed_back_count"] + 1)
        return {"action": "keep_manifest", "manifest": m}
    raise ValueError(f"unknown outcome: {outcome}")
