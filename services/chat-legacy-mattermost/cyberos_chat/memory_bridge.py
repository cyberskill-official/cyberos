"""FR-CHAT-005 — logical replication to memory Layer-3 ingest."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from hashlib import sha256
from typing import Iterable


def _now() -> datetime:
    return datetime.now(timezone.utc)


@dataclass(frozen=True)
class ChatMessage:
    id: str
    tenant_id: str
    channel_id: str
    subject_id: str
    body: str
    created_at: datetime
    updated_at: datetime | None = None


@dataclass(frozen=True)
class MemoryCaptureRow:
    row_kind: str
    tenant_id: str
    source_message_id: str
    body_hash: str
    memory_path: str
    captured_at: datetime
    lag_ms: int
    idempotency_key: str


class MemoryBridge:
    """Convert chat rows into deterministic memory ingest envelopes."""

    def __init__(self, *, max_lag_ms: int = 5_000) -> None:
        self.max_lag_ms = max_lag_ms
        self._seen: set[str] = set()

    def capture(self, message: ChatMessage, *, captured_at: datetime | None = None) -> MemoryCaptureRow:
        captured_at = captured_at or _now()
        lag_ms = int((captured_at - message.created_at).total_seconds() * 1000)
        key_material = f"{message.tenant_id}:{message.id}:{message.updated_at or message.created_at}"
        idempotency_key = sha256(key_material.encode("utf-8")).hexdigest()
        if idempotency_key in self._seen:
            raise ValueError("duplicate replication event")
        self._seen.add(idempotency_key)
        body_hash = sha256(message.body.encode("utf-8")).hexdigest()
        prefix = body_hash[:4]
        return MemoryCaptureRow(
            row_kind="chat.message_captured",
            tenant_id=message.tenant_id,
            source_message_id=message.id,
            body_hash=body_hash,
            memory_path=f"memories/facts/{prefix[:2]}/{prefix[2:]}/chat-{message.id}.md",
            captured_at=captured_at,
            lag_ms=lag_ms,
            idempotency_key=idempotency_key,
        )

    def capture_many(self, messages: Iterable[ChatMessage]) -> list[MemoryCaptureRow]:
        return [self.capture(message) for message in messages]

    @staticmethod
    def p95_lag_ms(rows: list[MemoryCaptureRow]) -> int:
        if not rows:
            return 0
        ordered = sorted(row.lag_ms for row in rows)
        idx = min(len(ordered) - 1, int((len(ordered) - 1) * 0.95))
        return ordered[idx]

    def assert_sla(self, rows: list[MemoryCaptureRow]) -> None:
        p95 = self.p95_lag_ms(rows)
        if p95 > self.max_lag_ms:
            raise AssertionError(f"memory bridge p95 lag {p95}ms > {self.max_lag_ms}ms")
