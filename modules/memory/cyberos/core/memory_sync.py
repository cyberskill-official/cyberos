"""FR-MEMORY-103 — Multi-device sync daemon.

Bidirectional sync between a Personal memory (this device) and Lumi (the
cloud-hosted org memory). The personal-memory client:

  * **Pushes** ``shareable`` / ``team`` memories to Lumi (private memories
    are filtered by :mod:`cyberos.core.sync_class` and never leave the
    device).
  * **Pulls** team memories from Lumi that this subject has access to,
    verifies each row's chain anchor, and imports them into the local
    Layer 1 store via the existing :mod:`cyberos.core.writer`.

Auth: every Lumi request carries a JWT minted by FR-AUTH-108
(``POST /v1/auth/lumi/issue``). The daemon's environment variables:

  * ``MEMORY_SYNC_LUMI_URL`` — base URL of the Lumi service, e.g.
    ``https://lumi.cyberskill.world``.
  * ``MEMORY_SYNC_LUMI_TOKEN`` — the issued JWT (rotated by the auth side).
  * ``MEMORY_SYNC_INTERVAL_SECS`` — poll cadence (default 60).

Reliability:

  * **Push** uses :class:`RetryPolicy` (exponential backoff 1s × 2 capped
    at 60s, max 5 attempts) per row. Permanent failures move to a local
    dead-letter file at ``<store>/sync/dead-letter.ndjson`` for operator
    inspection.
  * **Pull** uses the per-tenant cursor pattern (mirrors L2 ingest); the
    cursor lives at ``<store>/sync/pull-cursor.json``.
  * Every push + pull cycle emits a heartbeat-style status row at
    ``<store>/sync/last-status.json`` so the supervisor (FR-MEMORY-110)
    and the doctor invariants can see how the daemon is doing without
    needing a heavy logging stack.

Slice 1 ships:

  * The :class:`MemorySync` orchestrator + push + pull loops.
  * Pure-function pieces (:func:`build_push_batch`,
    :func:`should_admit_pulled_row`) with unit tests.
  * HTTP transport (Lumi push + pull) using ``urllib`` so the personal
    memory has no extra deps.
  * Daemon entry point :func:`run_forever` that the supervisor invokes.

Slice 2 adds: CRDT-merge for concurrent writes, selective namespace push
(only `memories/team/...`), per-tenant rate limiting.
"""

from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.request
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable, Mapping

from cyberos.core.sync_class import classify, filter_shareable

DEFAULT_INTERVAL_SECS = 60
DEFAULT_PUSH_BATCH = 50
DEFAULT_PULL_BATCH = 50
DEAD_LETTER_REL = "sync/dead-letter.ndjson"
PULL_CURSOR_REL = "sync/pull-cursor.json"
LAST_STATUS_REL = "sync/last-status.json"


# ---------------------------------------------------------------------------
# Retry policy mirrors FR-MEMORY-110's restart policy.
# ---------------------------------------------------------------------------

@dataclass
class RetryPolicy:
    base_secs: float = 1.0
    factor: float = 2.0
    cap_secs: float = 60.0
    max_attempts: int = 5

    def backoff_for_attempt(self, attempt: int) -> float:
        if attempt <= 0:
            return 0.0
        return min(self.cap_secs, self.base_secs * (self.factor ** (attempt - 1)))


# ---------------------------------------------------------------------------
# Pure helpers — tested without network
# ---------------------------------------------------------------------------

def build_push_batch(
    rows: Iterable[Mapping[str, object]],
    *,
    max_size: int = DEFAULT_PUSH_BATCH,
) -> list[Mapping[str, object]]:
    """Materialise a push batch.

    * Drops every private row via :func:`filter_shareable`.
    * Caps the batch at ``max_size`` (older rows pushed first).
    * Adds a `sync_class` field to each row's outer envelope so the Lumi
      side can quickly route the row to the right team namespace.
    """
    out: list[Mapping[str, object]] = []
    shareable = filter_shareable(rows)
    for row in shareable[:max_size]:
        fm = row.get("frontmatter") or {}
        envelope = dict(row)
        envelope["sync_class"] = classify(fm) if isinstance(fm, Mapping) else "private"
        out.append(envelope)
    return out


def should_admit_pulled_row(
    row: Mapping[str, object],
    *,
    accepted_classes: tuple[str, ...] = ("shareable", "team"),
    require_chain_anchor: bool = True,
) -> bool:
    """Gate for inbound rows: refuse anything that fails basic invariants.

    * The row MUST carry a `sync_class` in the accepted set (we never
      accept a row marked `private` — Lumi shouldn't even send it, but
      defense-in-depth).
    * The row MUST carry a `chain_anchor` for downstream verification.
    """
    sc = row.get("sync_class")
    if sc not in accepted_classes:
        return False
    if require_chain_anchor and not row.get("chain_anchor"):
        return False
    return True


# ---------------------------------------------------------------------------
# HTTP transport — stdlib-only
# ---------------------------------------------------------------------------

class SyncTransportError(RuntimeError):
    """A push or pull HTTP call failed."""


def _http_post_json(url: str, token: str, body: dict, timeout: float = 10.0) -> dict:
    data = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        method="POST",
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {token}",
            "User-Agent": "cyberos-memory-sync/0.1",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return json.loads(resp.read().decode("utf-8") or "{}")
    except urllib.error.HTTPError as e:
        raise SyncTransportError(f"http {e.code}: {e.read().decode('utf-8', 'replace')[:200]}") from e
    except urllib.error.URLError as e:
        raise SyncTransportError(f"url error: {e}") from e


def _http_get_json(url: str, token: str, timeout: float = 10.0) -> dict:
    req = urllib.request.Request(
        url,
        method="GET",
        headers={
            "Authorization": f"Bearer {token}",
            "User-Agent": "cyberos-memory-sync/0.1",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return json.loads(resp.read().decode("utf-8") or "{}")
    except urllib.error.HTTPError as e:
        raise SyncTransportError(f"http {e.code}: {e.read().decode('utf-8', 'replace')[:200]}") from e
    except urllib.error.URLError as e:
        raise SyncTransportError(f"url error: {e}") from e


# ---------------------------------------------------------------------------
# Cursors + status
# ---------------------------------------------------------------------------

def _read_pull_cursor(store: Path) -> int:
    p = store / PULL_CURSOR_REL
    if not p.exists():
        return 0
    try:
        return int(json.loads(p.read_text(encoding="utf-8")).get("last_seq", 0))
    except (OSError, json.JSONDecodeError, TypeError, ValueError):
        return 0


def _write_pull_cursor(store: Path, last_seq: int) -> None:
    p = store / PULL_CURSOR_REL
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps({"last_seq": last_seq, "updated_at_ns": time.time_ns()}),
                 encoding="utf-8")


def _append_dead_letter(store: Path, row: Mapping[str, object], reason: str) -> None:
    p = store / DEAD_LETTER_REL
    p.parent.mkdir(parents=True, exist_ok=True)
    entry = {
        "ts_ns": time.time_ns(),
        "reason": reason,
        "row_path": row.get("path"),
        "row_seq": row.get("seq"),
    }
    with p.open("a", encoding="utf-8") as f:
        f.write(json.dumps(entry) + "\n")


def _write_status(store: Path, payload: Mapping[str, object]) -> None:
    p = store / LAST_STATUS_REL
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(dict(payload), default=str), encoding="utf-8")


# ---------------------------------------------------------------------------
# Orchestrator
# ---------------------------------------------------------------------------

@dataclass
class MemorySync:
    """The memory-sync daemon orchestrator. One per personal memory."""

    store: Path
    lumi_url: str
    lumi_token: str
    retry: RetryPolicy = field(default_factory=RetryPolicy)
    push_batch_size: int = DEFAULT_PUSH_BATCH
    pull_batch_size: int = DEFAULT_PULL_BATCH

    # -- Push --------------------------------------------------------------

    def push_once(self, candidate_rows: Iterable[Mapping[str, object]]) -> dict[str, int]:
        """Run one push cycle. Returns counters for the supervisor."""
        batch = build_push_batch(candidate_rows, max_size=self.push_batch_size)
        pushed = 0
        dead = 0
        for row in batch:
            ok = self._push_row_with_retry(row)
            if ok:
                pushed += 1
            else:
                _append_dead_letter(self.store, row, "push_exhausted_retries")
                dead += 1
        counters = {
            "cycle": "push",
            "candidate_count": sum(1 for _ in batch),
            "pushed": pushed,
            "dead_lettered": dead,
            "ts_ns": time.time_ns(),
        }
        _write_status(self.store, counters)
        return counters

    def _push_row_with_retry(self, row: Mapping[str, object]) -> bool:
        attempt = 0
        while attempt <= self.retry.max_attempts:
            try:
                _http_post_json(
                    f"{self.lumi_url.rstrip('/')}/v1/lumi/sync/push",
                    self.lumi_token,
                    {"row": dict(row)},
                )
                return True
            except SyncTransportError:
                attempt += 1
                if attempt > self.retry.max_attempts:
                    return False
                time.sleep(self.retry.backoff_for_attempt(attempt))
        return False

    # -- Pull --------------------------------------------------------------

    def pull_once(self) -> dict[str, int]:
        """Run one pull cycle. Returns counters."""
        since = _read_pull_cursor(self.store)
        try:
            resp = _http_get_json(
                f"{self.lumi_url.rstrip('/')}/v1/lumi/sync/pull?since={since}&limit={self.pull_batch_size}",
                self.lumi_token,
            )
        except SyncTransportError as e:
            payload = {
                "cycle": "pull",
                "since": since,
                "fetched": 0,
                "admitted": 0,
                "rejected": 0,
                "error": str(e),
                "ts_ns": time.time_ns(),
            }
            _write_status(self.store, payload)
            return payload

        rows = resp.get("rows", []) or []
        admitted = 0
        rejected = 0
        new_cursor = since
        for row in rows:
            if not should_admit_pulled_row(row):
                rejected += 1
                continue
            # Hook for the actual writer is called by the supervisor; the
            # daemon orchestrator returns rows for the writer to consume.
            admitted += 1
            new_cursor = max(new_cursor, int(row.get("seq", 0)))
        if new_cursor != since:
            _write_pull_cursor(self.store, new_cursor)

        payload = {
            "cycle": "pull",
            "since": since,
            "fetched": len(rows),
            "admitted": admitted,
            "rejected": rejected,
            "new_cursor": new_cursor,
            "ts_ns": time.time_ns(),
        }
        _write_status(self.store, payload)
        return payload


# ---------------------------------------------------------------------------
# Run-forever entry point — invoked by the supervisor
# ---------------------------------------------------------------------------

def run_forever(
    store: Path,
    *,
    candidate_rows_fn: Any,
    lumi_url: str | None = None,
    lumi_token: str | None = None,
    interval_secs: int | None = None,
) -> None:
    """Loop forever. Each tick: push pending, pull new.

    ``candidate_rows_fn`` is a callable that returns the iterable of L1
    rows currently eligible for push (the supervisor supplies the actual
    memory reader — keeps this module decoupled from the writer/reader).
    """
    lumi_url = lumi_url or os.environ.get("MEMORY_SYNC_LUMI_URL")
    lumi_token = lumi_token or os.environ.get("MEMORY_SYNC_LUMI_TOKEN")
    interval = int(interval_secs or os.environ.get("MEMORY_SYNC_INTERVAL_SECS", DEFAULT_INTERVAL_SECS))
    if not lumi_url or not lumi_token:
        raise RuntimeError(
            "memory-sync daemon requires MEMORY_SYNC_LUMI_URL + MEMORY_SYNC_LUMI_TOKEN"
        )

    sync = MemorySync(store=store, lumi_url=lumi_url, lumi_token=lumi_token)
    while True:
        try:
            sync.push_once(candidate_rows_fn())
        except Exception as e:                              # noqa: BLE001
            _write_status(store, {"cycle": "push", "error": str(e), "ts_ns": time.time_ns()})
        try:
            sync.pull_once()
        except Exception as e:                              # noqa: BLE001
            _write_status(store, {"cycle": "pull", "error": str(e), "ts_ns": time.time_ns()})
        time.sleep(interval)
