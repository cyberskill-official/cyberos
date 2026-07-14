"""TASK-CHAT-012 — subject export with memory audit hashes."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from hashlib import sha256
from typing import Iterable

from .memory_bridge import ChatMessage, MemoryCaptureRow


@dataclass(frozen=True)
class DsarExport:
    subject_id: str
    messages: list[dict]
    memory_hashes: list[str]
    manifest_hash: str


def export_subject_messages(
    subject_id: str,
    messages: Iterable[ChatMessage],
    captures: Iterable[MemoryCaptureRow],
) -> DsarExport:
    subject_messages = [message for message in messages if message.subject_id == subject_id]
    ids = {message.id for message in subject_messages}
    hashes = sorted(row.body_hash for row in captures if row.source_message_id in ids)
    body = {
        "subject_id": subject_id,
        "messages": [
            {
                "id": msg.id,
                "tenant_id": msg.tenant_id,
                "channel_id": msg.channel_id,
                "body": msg.body,
                "created_at": msg.created_at.isoformat(),
            }
            for msg in sorted(subject_messages, key=lambda m: (m.created_at, m.id))
        ],
        "memory_hashes": hashes,
    }
    manifest_hash = sha256(repr(body).encode("utf-8")).hexdigest()
    return DsarExport(
        subject_id=subject_id,
        messages=body["messages"],
        memory_hashes=hashes,
        manifest_hash=manifest_hash,
    )


def to_dict(export: DsarExport) -> dict:
    return asdict(export)
